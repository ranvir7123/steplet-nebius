//! The gate service (plan Task 8): Steplet's end of the run's named pipe, to which `steplet-gate.exe` relays the
//! agent's permission requests. It is made before the launch, and does not start if anyone else holds the pipe's
//! name. Each connection carries one request. Who connected is checked first (the gate program, Steplet's own
//! Windows account and sign-in session, not restricted, at least medium integrity), then the run's token, and the
//! decision has at most Steplet's deadline to come (F36). A late decision, a broken request and anything unclear
//! are refusals, and nothing is kept from one request to the next but the record of what was asked (F33).
//!
//! Also here: putting the gate program on Steplet's Linux disk, named after its contents.

use std::fmt;
use std::io;
use std::os::windows::io::AsRawHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::sync::oneshot;
use windows_sys::Win32::System::Pipes::WaitNamedPipeW;

use crate::client::{Client, MEDIUM_INTEGRITY, Permissions, wait_until_read};
use crate::types::{GateAnswer, GateLink, GateRequest, RunPaths, is_guid};
use crate::wsl;

/// The longest request Steplet reads: a write's request carries the whole file.
const LONGEST_REQUEST: usize = 16 << 20;
/// Where the gate program lives on Steplet's Linux disk (Task 1).
const GATE_FOLDER: &str = "/opt/steplet/gate";
/// Where each run's own copy of the gate goes; `discard.sh` removes a run's copies with its folders.
const RUN_GATES: &str = "/opt/steplet/gate/runs";
const ERROR_ACCESS_DENIED: i32 = 5;
/// How long sending one answer may take. The gate reads its answer at once; this only ends a client that never does.
const SEND_LIMIT: Duration = Duration::from_secs(5);
/// The longest stopping waits for Windows to close the free end of the pipe.
const CLOSE_LIMIT: Duration = Duration::from_secs(1);

/// The Anthropic key the gate's key mode hands to Claude Code (H2). It is never shown: its `Debug` says only that
/// there is one.
#[derive(Clone)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(key: String) -> ApiKey {
        ApiKey(key)
    }

    /// The key's text, for the report to hide it (F48).
    pub(crate) fn text(&self) -> &str {
        &self.0
    }

    /// Whether this is the key `text`, without handing the key out.
    pub fn is(&self, text: &str) -> bool {
        self.0 == text
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(hidden)")
    }
}

/// One connection to the run's pipe, and what Steplet did with it.
#[derive(Clone, Debug, PartialEq)]
pub enum Asked {
    /// A permission request and Steplet's answer. `took` runs from the connection to the answer. `delivered` is true
    /// once the gate has read the whole answer, and false when it could not (it died first, F39) or did not within
    /// 5 s: such an answer was not acted on.
    Decision { client: Client, request: GateRequest, answer: GateAnswer, took: Duration, delivered: bool },
    /// The key, for Claude Code's `apiKeyHelper`; `served` is true once the gate has read it.
    Key { client: Client, served: bool },
    /// Refused before any decision; `client` is `None` when Windows would not say who it was, and `call` is the tool
    /// call's id when the request was read far enough to name one (the report labels that call, Task 13).
    Refused { client: Option<Client>, reason: String, call: Option<String> },
}

#[derive(Debug)]
pub enum GateError {
    /// Someone else holds the pipe's name, so the run must not start.
    Taken,
    /// The service could not start; the text says why.
    Start(String),
    /// The gate program could not be put on Steplet's Linux disk; the text says why.
    Install(String),
}

/// The running service. Stopping it, or dropping it, closes the pipe: once that returns, no client can connect, unless
/// Windows has not closed the pipe's free end within 1 s (never seen). A connection still being answered then may stay
/// open a while, but no new one can be made.
pub struct GateService {
    records: Arc<Mutex<Vec<Asked>>>,
    /// The tool call ids of the requests being decided now. Locked only after `records` when both are held.
    deciding: Arc<Mutex<Vec<String>>>,
    stopping: Arc<AtomicBool>,
    stop: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl GateService {
    /// Everything asked so far, for the tripwire watcher and the report, in the order Steplet answered: a decision
    /// still being made is not there yet, and a request answered late comes after one asked later and answered
    /// sooner. Each record is written before its answer is sent, and marked delivered (or served) a moment after the
    /// gate has read it; a decision that never is was not acted on.
    pub fn requests(&self) -> Vec<Asked> {
        self.records.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// What Steplet answered the gate about tool call `id`, for the tripwire watcher: `None` if the gate never asked
    /// about it, `Some(true)` if any answer was "allow", `Some(false)` if every one was "deny", or if it is still
    /// being decided (in a live run, while the question is on the receiver's screen): until it is allowed, it is not.
    /// Each answer is on record before it is sent, so this is known before the call can run. A request with no id
    /// matches no call.
    pub fn answered(&self, id: &str) -> Option<bool> {
        if id.is_empty() {
            return None;
        }
        let records = self.records.lock().unwrap_or_else(PoisonError::into_inner);
        let answered = records.iter().fold(None, |seen, asked| match asked {
            Asked::Decision { request, answer, .. } if request.tool_use_id == id => {
                Some(seen == Some(true) || matches!(answer, GateAnswer::Allow { .. }))
            }
            _ => seen,
        });
        // Looked at while `records` is still held, so a decision recorded in between cannot be missed.
        answered.or_else(|| {
            let deciding = self.deciding.lock().unwrap_or_else(PoisonError::into_inner);
            deciding.iter().any(|d| d == id).then_some(false)
        })
    }

    /// From now on every request is refused: Steplet is stopping the run, which takes a moment. Every decision
    /// recorded after this returns is a refusal, including one still being made now; one recorded a moment before
    /// may still be on its way to the gate. The key is still served.
    pub fn refuse_everything(&self) {
        // Set while holding the record, which each decision also holds while it is checked and recorded.
        let _records = self.records.lock().unwrap_or_else(PoisonError::into_inner);
        self.stopping.store(true, Ordering::SeqCst);
    }

    /// Stops the service and returns everything asked. Called once the agent has ended: a decision still being made
    /// then was not acted on.
    pub fn into_requests(mut self) -> Vec<Asked> {
        self.end();
        self.requests()
    }

    pub fn stop(mut self) {
        self.end();
    }

    fn end(&mut self) {
        // From the stop on, every decision is a refusal, including one that finishes while the free end closes.
        self.refuse_everything();
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for GateService {
    fn drop(&mut self) {
        self.end();
    }
}

/// What every connection is checked and answered with.
struct Rules<D> {
    token: String,
    /// The gate program, as Windows names it.
    program: String,
    /// Steplet itself, whose account and sign-in session a client must share.
    me: Client,
    key: Option<ApiKey>,
    deadline: Duration,
    decide: D,
    records: Arc<Mutex<Vec<Asked>>>,
    deciding: Arc<Mutex<Vec<String>>>,
    /// Set once Steplet is stopping the run: every request is then refused.
    stopping: Arc<AtomicBool>,
}

/// Starts the service for `link`: its pipe, token and deadline are the ones the gate is started with. Only
/// `program` (the gate program, as Windows names it) may ask. `decide` answers each request; Steplet's runs pass
/// `decide(&policy, request)`, the one checker (row 29). `key` is served to the gate's key mode.
pub fn serve<D>(link: &GateLink, program: &str, key: Option<ApiKey>, decide: D) -> Result<GateService, GateError>
where
    D: Fn(&GateRequest) -> GateAnswer + Send + Sync + 'static,
{
    let start = |what: &str, e: io::Error| GateError::Start(format!("{what}: {e}"));
    let me = Client::this_program().map_err(|e| start("Steplet could not read its own Windows account", e))?;
    let mut permissions = Permissions::only(&me.user).map_err(|e| start("could not make the pipe's permissions", e))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|e| start("could not start the service", e))?;
    let name = format!(r"\\.\pipe\{}", link.pipe());
    let first = {
        let _inside = runtime.enter();
        end_of_pipe(&name, &mut permissions, true)
    };
    let first = match first {
        Ok(f) => f,
        // The first end must be Steplet's: if the name exists, someone else made it.
        Err(e) if e.raw_os_error() == Some(ERROR_ACCESS_DENIED) => return Err(GateError::Taken),
        Err(e) => return Err(start("could not make the pipe", e)),
    };
    let records = Arc::new(Mutex::new(Vec::new()));
    let deciding = Arc::new(Mutex::new(Vec::new()));
    let stopping = Arc::new(AtomicBool::new(false));
    let rules = Arc::new(Rules {
        token: link.token().into(),
        program: program.into(),
        me,
        key,
        deadline: link.deadline(),
        decide,
        records: records.clone(),
        deciding: deciding.clone(),
        stopping: stopping.clone(),
    });
    let (stop, stopped) = oneshot::channel();
    let thread = std::thread::spawn(move || {
        let accepting = runtime.spawn(accept(first, name.clone(), permissions, rules));
        // Runs the service until it is stopped (or the service value is gone).
        let _ = runtime.block_on(stopped);
        runtime.block_on(close_free_end(accepting, &name));
        // A decision still being made is not waited for: its checker may never return.
        runtime.shutdown_background();
    });
    Ok(GateService { records, deciding, stopping, stop: Some(stop), thread: Some(thread) })
}

/// One end of the pipe, with Steplet's permission list, refusing clients on other computers.
fn end_of_pipe(name: &str, permissions: &mut Permissions, first: bool) -> io::Result<NamedPipeServer> {
    let mut attributes = permissions.attributes();
    // SAFETY: `attributes` and the list it points to live until the call returns.
    unsafe {
        ServerOptions::new()
            .first_pipe_instance(first)
            .reject_remote_clients(true)
            .create_with_security_attributes_raw(name, (&raw mut attributes).cast())
    }
}

/// Keeps one free end of the pipe waiting, and hands each connection to `answer`.
async fn accept<D>(mut server: NamedPipeServer, name: String, mut permissions: Permissions, rules: Arc<Rules<D>>)
where
    D: Fn(&GateRequest) -> GateAnswer + Send + Sync + 'static,
{
    loop {
        let connected = server.connect().await;
        // The next end is made at once; until then a gate finds the pipe busy, and waits.
        let next = match end_of_pipe(&name, &mut permissions, false) {
            Ok(next) => next,
            Err(e) => {
                // With no free end, every later request finds no pipe, which the gate refuses.
                record(
                    &rules.records,
                    Asked::Refused { client: None, reason: format!("the pipe could not stay open: {e}"), call: None },
                );
                return;
            }
        };
        let conn = std::mem::replace(&mut server, next);
        if connected.is_ok() {
            tokio::spawn(answer(conn, rules.clone()));
        }
    }
}

/// Ends `accepting`, which holds the free end of the pipe, and waits until Windows says the pipe has no free end, so
/// that no client can connect once the service has stopped. Dropping the end is not enough: tokio's pipe closes its
/// handle only once the runtime has read Windows' word that the end's wait for a client was cancelled. After the
/// runtime is shut down that happens only when its last thread is gone: a moment later, or, while a checker is stuck,
/// never. Waits at most `CLOSE_LIMIT`.
async fn close_free_end(accepting: tokio::task::JoinHandle<()>, name: &str) {
    accepting.abort();
    let _ = accepting.await;
    let start = Instant::now();
    while has_free_end(name) && start.elapsed() < CLOSE_LIMIT {
        // Lets the runtime read what Windows has reported. tokio does not promise that it does, so Windows is asked
        // each time.
        tokio::task::yield_now().await;
    }
}

/// True while the pipe has an end a client could connect to now. This only asks Windows; it connects nothing. While
/// every end left is busy with a client, Windows takes one tick of its clock (about 15 ms) to say so.
fn has_free_end(name: &str) -> bool {
    let name: Vec<u16> = name.encode_utf16().chain([0]).collect();
    // SAFETY: `name` is a NUL-terminated name that lives until the call returns.
    unsafe { WaitNamedPipeW(name.as_ptr(), 1) != 0 }
}

fn record(records: &Mutex<Vec<Asked>>, asked: Asked) -> usize {
    let mut records = records.lock().unwrap_or_else(PoisonError::into_inner);
    records.push(asked);
    records.len() - 1
}

/// Notes that the gate has read the whole answer to record `n`.
fn mark_read(records: &Mutex<Vec<Asked>>, n: usize) {
    let mut records = records.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(Asked::Decision { delivered: flag, .. } | Asked::Key { served: flag, .. }) = records.get_mut(n) {
        *flag = true;
    }
}

/// One connection: read the request, check who sent it, decide within the deadline, answer.
async fn answer<D>(mut conn: NamedPipeServer, rules: Arc<Rules<D>>)
where
    D: Fn(&GateRequest) -> GateAnswer + Send + Sync + 'static,
{
    let started = Instant::now();
    let refused = |client: Option<Client>, reason: String, call: Option<String>| {
        record(&rules.records, Asked::Refused { client, reason, call });
    };
    let mut line = Vec::new();
    let read = {
        let mut reader = BufReader::new(&mut conn).take(LONGEST_REQUEST as u64 + 1);
        tokio::time::timeout(rules.deadline, reader.read_until(b'\n', &mut line)).await
    };
    match read {
        Err(_) => return refused(None, "no request came before the deadline".into(), None),
        Ok(Err(e)) => return refused(None, format!("the request could not be read: {e}"), None),
        Ok(Ok(_)) if line.len() > LONGEST_REQUEST => {
            refused(None, "the request is longer than 16 MiB".into(), None);
            return send(&mut conn, &deny_json("the request is too long")).await.unwrap_or(());
        }
        Ok(Ok(_)) if line.last() != Some(&b'\n') => {
            return refused(None, "the connection closed before a whole request".into(), None);
        }
        Ok(Ok(_)) => {}
    }
    let parsed: Option<Value> = serde_json::from_slice(&line).ok();
    let wants_key = parsed.as_ref().is_some_and(|v| v["kind"] == "key");
    // The tool call the request is about, as the gate program says: only for labelling that call in the report (and
    // never over a call that ran), and only once the request comes from the run's own gate program.
    let call = || parsed.as_ref().and_then(|v| v.get("tool_use_id")).and_then(Value::as_str).map(str::to_owned);
    let no = |why: &str| if wants_key { json!({"error": why}) } else { deny_json(why) };

    let client = match Client::of_pipe(conn.as_raw_handle()) {
        Ok(c) => c,
        Err(e) => {
            refused(None, format!("Windows would not say who connected: {e}"), None);
            return send(&mut conn, &no("Steplet could not tell who asked")).await.unwrap_or(());
        }
    };
    if let Some(reason) = who_may_not_ask(&client, &rules.me, &rules.program) {
        // A stranger's request names no call of the run's own.
        refused(Some(client), reason, None);
        return send(&mut conn, &no("Steplet does not take requests from this program")).await.unwrap_or(());
    }
    let incoming = match parsed.as_ref().map(read_request) {
        Some(Ok(incoming)) => incoming,
        Some(Err(why)) => {
            refused(Some(client), why.into(), call());
            return send(&mut conn, &no(why)).await.unwrap_or(());
        }
        None => {
            refused(Some(client), "the request is not JSON".into(), None);
            return send(&mut conn, &no("the request is not JSON")).await.unwrap_or(());
        }
    };
    let token = match &incoming {
        Incoming::Key { run_token } => run_token,
        Incoming::Decide(r) => &r.run_token,
    };
    if !same(token, &rules.token) {
        refused(Some(client), "the request carries the wrong run token".into(), call());
        return send(&mut conn, &no("this is not the run's gate")).await.unwrap_or(());
    }

    match incoming {
        Incoming::Key { .. } => {
            let reply = match &rules.key {
                Some(key) => json!({"key": key.0}),
                None => json!({"error": "Steplet has no key for this run"}),
            };
            let n = record(&rules.records, Asked::Key { client, served: false });
            if send(&mut conn, &reply).await.is_ok() && rules.key.is_some() {
                mark_read(&rules.records, n);
            }
        }
        Incoming::Decide(request) => {
            let stopping = || rules.stopping.load(Ordering::SeqCst);
            let refused_while_stopping =
                || GateAnswer::Deny { message: "Steplet is stopping this run, so it refused this".into() };
            let left = rules.deadline.saturating_sub(started.elapsed());
            let asking = rules.clone();
            let asked = request.clone();
            // Known as being decided from now until its answer is on record (see `GateService::answered`).
            rules.deciding.lock().unwrap_or_else(PoisonError::into_inner).push(request.tool_use_id.clone());
            let decided = if stopping() {
                Ok(Ok(refused_while_stopping()))
            } else {
                tokio::time::timeout(left, tokio::task::spawn_blocking(move || (asking.decide)(&asked))).await
            };
            let answer = match decided {
                Ok(Ok(answer)) => answer,
                Ok(Err(_)) => GateAnswer::Deny { message: "Steplet's checker failed, so it refused this".into() },
                Err(_) => GateAnswer::Deny {
                    message: format!(
                        "Steplet did not decide within {} s, so it refused this",
                        rules.deadline.as_secs_f64()
                    ),
                },
            };
            // Recorded before it is sent, so the record of an allowed action exists before the action can run. Whether
            // Steplet has begun stopping the run is checked in the same step (see `refuse_everything`).
            let took = started.elapsed();
            let (n, reply) = {
                let mut records = rules.records.lock().unwrap_or_else(PoisonError::into_inner);
                let answer = if stopping() { refused_while_stopping() } else { answer };
                let reply = match &answer {
                    GateAnswer::Allow { updated_input } => json!({"behavior": "allow", "updatedInput": updated_input}),
                    GateAnswer::Deny { message } => json!({"behavior": "deny", "message": message}),
                };
                let mut deciding = rules.deciding.lock().unwrap_or_else(PoisonError::into_inner);
                if let Some(n) = deciding.iter().position(|d| *d == request.tool_use_id) {
                    deciding.swap_remove(n);
                }
                drop(deciding);
                records.push(Asked::Decision { client, request, answer, took, delivered: false });
                (records.len() - 1, reply)
            };
            if send(&mut conn, &reply).await.is_ok() {
                mark_read(&rules.records, n);
            }
        }
    }
}

/// Why `client` may not ask, if it may not.
fn who_may_not_ask(client: &Client, me: &Client, program: &str) -> Option<String> {
    let why = if !client.program.eq_ignore_ascii_case(program) {
        format!("a program other than the gate connected: {}", client.program)
    } else if client.user != me.user {
        "a program of another Windows account connected".into()
    } else if client.logon_session != me.logon_session {
        "a program of another sign-in session connected".into()
    } else if client.restricted {
        "a program with a restricted token connected".into()
    } else if client.app_container {
        "a program in an app container connected".into()
    } else if client.integrity < MEDIUM_INTEGRITY {
        "a program below medium integrity connected".into()
    } else {
        return None;
    };
    Some(why)
}

enum Incoming {
    Decide(GateRequest),
    Key { run_token: String },
}

fn read_request(v: &Value) -> Result<Incoming, &'static str> {
    let text = |field: &str| v.get(field).and_then(Value::as_str).map(str::to_owned);
    let run_token = text("run_token").ok_or("the request carries no run token")?;
    match v.get("kind").and_then(Value::as_str) {
        Some("key") => Ok(Incoming::Key { run_token }),
        Some("decide") => Ok(Incoming::Decide(GateRequest {
            run_token,
            tool_name: text("tool_name").ok_or("the request names no tool")?,
            input: v.get("input").filter(|i| i.is_object()).cloned().ok_or("the request has no input")?,
            tool_use_id: text("tool_use_id").unwrap_or_default(),
        })),
        _ => Err("the request is of no kind Steplet reads"),
    }
}

/// Compares two tokens in a time that does not depend on where they differ.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn deny_json(why: &str) -> Value {
    json!({"behavior": "deny", "message": format!("Steplet refused this: {why}.")})
}

/// Writes one answer line and returns once the client has read all of it. A client that has not within
/// `SEND_LIMIT` is cut off: an allowed write's answer carries the whole file, more than the pipe holds, and a client
/// that never reads it must not keep its end of the pipe. (The write alone finishes once tokio has taken the data,
/// before the client has it.)
async fn send(conn: &mut NamedPipeServer, reply: &Value) -> io::Result<()> {
    conn.write_all(format!("{reply}\n").as_bytes()).await?;
    let pipe = conn.as_raw_handle() as usize;
    let mut reading = tokio::task::spawn_blocking(move || wait_until_read(pipe));
    match tokio::time::timeout(SEND_LIMIT, &mut reading).await {
        Ok(Ok(read)) => read,
        Ok(Err(_)) => Err(io::Error::other("the wait for the client failed")),
        Err(_) => {
            // Cutting the client off also ends the wait, which must be over before the pipe's handle can close.
            let _ = conn.disconnect();
            let _ = reading.await;
            Err(io::Error::new(io::ErrorKind::TimedOut, "the client did not read its answer"))
        }
    }
}

/// The gate program's file name on Steplet's Linux disk: named after its contents (SHA-256), so each build has a
/// name of its own.
pub fn gate_file_name(program: &[u8]) -> String {
    format!("steplet-gate-{}.exe", &sha256(program)[..16])
}

/// How Windows names a program on Steplet's Linux disk, given its Linux path: as Windows reports a gate program that
/// Claude Code started there.
pub fn gate_on_windows(distribution: &str, linux_path: &str) -> String {
    format!(r"\\wsl.localhost\{distribution}{}", linux_path.replace('/', r"\"))
}

/// Puts the gate program on Steplet's Linux disk as root's, and returns its Linux path. A gate file is never
/// replaced (on 14 Oct 2026 Windows refused to start a program from a path whose file had been replaced, Task 7):
/// one already there is used only if its contents are this program's, and a new one appears whole, by a rename.
pub fn install_gate(distro_id: &str, program: &[u8]) -> Result<String, GateError> {
    if !is_guid(distro_id) {
        return Err(GateError::Install("the distribution id is not a Windows GUID".into()));
    }
    let path = format!("{GATE_FOLDER}/{}", gate_file_name(program));
    let wanted = sha256(program);
    let root = |argv: &[&str], input: &[u8]| {
        wsl::run(distro_id, "root", argv, input).map_err(|e| GateError::Install(format!("Steplet's Linux: {e}")))
    };
    let sum = |root_out: wsl::Output| root_out.stdout.split_whitespace().next().map(str::to_owned);
    let mut found = root(&["/usr/bin/sha256sum", "--", &path], b"")?;
    if found.status != Some(0) {
        let part = format!("{path}.part");
        let install = ["/usr/bin/install", "-D", "-o", "root", "-g", "root", "-m", "755", "/dev/stdin", &part];
        let put = root(&install, program)?;
        if put.status != Some(0) {
            return Err(GateError::Install(format!("could not write the gate program: {}", put.stderr.trim())));
        }
        let moved = root(&["/usr/bin/mv", "-n", "-T", "--", &part, &path], b"")?;
        if moved.status != Some(0) {
            return Err(GateError::Install(format!(
                "could not put the gate program in place: {}",
                moved.stderr.trim()
            )));
        }
        found = root(&["/usr/bin/sha256sum", "--", &path], b"")?;
    }
    match sum(found) {
        Some(got) if got == wanted => Ok(path),
        _ => Err(GateError::Install(format!("{path} on Steplet's Linux is not this gate program"))),
    }
}

/// A fresh copy of the installed gate program (`installed`, where `install_gate` put it) for one run, at a path no
/// run used before: `/opt/steplet/gate/runs/<run id>.<16 random hex digits>.exe`, owned by root and not writable by
/// anyone else, so the agent cannot change the program Windows starts. Windows can come to refuse to start a program
/// from one exact `\wsl.localhost` path ("Error performing inpage operation", seen twice: phase 3 Task 7 and phase
/// 4 Task 9), whatever file is then put there, while the same bytes start from any new path; a gate path used run
/// after run would then stop every later run. Run ids never hold a `.`, so `discard` removes this run's copies and no
/// other run's.
pub fn gate_for_run(paths: &RunPaths, installed: &str) -> Result<String, GateError> {
    let name = installed.strip_prefix(GATE_FOLDER).and_then(|n| n.strip_prefix("/steplet-gate-"));
    let plain = name
        .and_then(|n| n.strip_suffix(".exe"))
        .is_some_and(|h| h.len() == 16 && h.bytes().all(|b| b.is_ascii_hexdigit()));
    if !plain {
        return Err(GateError::Install(format!("{installed} is not where install_gate puts the gate program")));
    }
    let random = crate::run::random_hex::<8>().map_err(|e| GateError::Install(format!("no random name: {e}")))?;
    let path = format!("{RUN_GATES}/{}.{random}.exe", paths.run_id());
    let root = |argv: &[&str]| {
        wsl::run(paths.distro_id(), "root", argv, b"").map_err(|e| GateError::Install(format!("Steplet's Linux: {e}")))
    };
    let copy = ["/usr/bin/install", "-D", "-o", "root", "-g", "root", "-m", "755", "-T", "--", installed, &path];
    let made = root(&copy)?;
    if made.status != Some(0) {
        return Err(GateError::Install(format!(
            "could not copy the gate program for this run: {}",
            made.stderr.trim()
        )));
    }
    let sums = root(&["/usr/bin/sha256sum", "--", installed, &path])?;
    let mut found = sums.stdout.lines().filter_map(|l| l.split_whitespace().next());
    match (sums.status, found.next(), found.next()) {
        (Some(0), Some(a), Some(b)) if a == b => Ok(path),
        _ => Err(GateError::Install(format!("{path} on Steplet's Linux is not a copy of the gate program"))),
    }
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}
