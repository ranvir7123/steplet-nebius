//! The gate program on its own (plan Task 8): it speaks MCP to Claude Code on its standard input and output, and
//! relays each request to Steplet over the run's pipe. Here a stand-in plays Steplet's end of the pipe, so these
//! tests need only Windows: `cargo test -p steplet-gate --offline`.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use windows_sys::Win32::Foundation::{ERROR_PIPE_CONNECTED, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

const GATE: &str = env!("CARGO_BIN_EXE_steplet-gate");
const TOKEN: &str = "t8gatetoken0123";

/// A pipe name no other test uses.
fn pipe_name() -> String {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    format!("steplet-gate-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))
}

/// What the stand-in for Steplet does with one connection, after reading its request line.
#[derive(Clone)]
enum Reply {
    /// Writes this line, then closes.
    Line(String),
    /// Closes without answering.
    Close,
    /// Keeps the connection open for this long without answering.
    Hold(Duration),
}

/// Steplet's end of the pipe, played by the test: it answers connection n with `replies[n]` (the last one again
/// after that) and keeps every request line it read.
struct StandIn {
    seen: Arc<Mutex<Vec<String>>>,
}

impl StandIn {
    fn start(pipe: &str, replies: Vec<Reply>) -> StandIn {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let name: Vec<u16> = format!(r"\\.\pipe\{pipe}").encode_utf16().chain([0]).collect();
        let (ready_tx, ready) = mpsc::channel();
        let kept = seen.clone();
        thread::spawn(move || {
            for n in 0.. {
                let mode = PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS;
                // SAFETY: `name` is a NUL-terminated wide string; no security attributes (the default list).
                let h = unsafe {
                    CreateNamedPipeW(
                        name.as_ptr(),
                        PIPE_ACCESS_DUPLEX,
                        mode,
                        PIPE_UNLIMITED_INSTANCES,
                        65536,
                        65536,
                        0,
                        std::ptr::null(),
                    )
                };
                assert_ne!(h, INVALID_HANDLE_VALUE, "the stand-in could not make its pipe");
                if n == 0 {
                    ready_tx.send(()).unwrap();
                }
                // SAFETY: `h` is the pipe instance just made, in blocking mode.
                let connected = unsafe { ConnectNamedPipe(h, std::ptr::null_mut()) } != 0
                    || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
                assert!(connected, "the stand-in's pipe did not connect");
                // SAFETY: `h` is an open handle that nothing else owns; the File closes it.
                let conn = unsafe { File::from_raw_handle(h as RawHandle) };
                let reply = replies.get(n).or(replies.last()).cloned().unwrap();
                let kept = kept.clone();
                thread::spawn(move || answer(conn, reply, &kept));
            }
        });
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        StandIn { seen }
    }

    /// The request lines read so far, as JSON.
    fn seen(&self) -> Vec<Value> {
        self.seen.lock().unwrap().iter().map(|l| serde_json::from_str(l).expect("the gate sends JSON")).collect()
    }
}

fn answer(conn: File, reply: Reply, seen: &Mutex<Vec<String>>) {
    let mut reader = BufReader::new(&conn);
    let mut line = String::new();
    if reader.read_line(&mut line).unwrap_or(0) == 0 {
        return;
    }
    seen.lock().unwrap().push(line.trim_end().to_owned());
    match reply {
        Reply::Line(text) => {
            let _ = (&conn).write_all(format!("{text}\n").as_bytes());
        }
        Reply::Close => {}
        Reply::Hold(d) => thread::sleep(d),
    }
}

/// The gate as Claude Code starts it, with its standard input and output piped.
struct Gate {
    child: Child,
    input: Option<ChildStdin>,
    lines: Receiver<String>,
}

impl Gate {
    fn start(args: &[&str]) -> Gate {
        let mut child = Command::new(GATE)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("the gate starts");
        let input = child.stdin.take();
        let out = child.stdout.take().unwrap();
        let (tx, lines) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                let _ = tx.send(line);
            }
        });
        Gate { child, input, lines }
    }

    /// The gate in MCP mode on `pipe`, logging to a fresh file, with Steplet's deadline; this test program, which
    /// holds the stand-in's end of the pipe, is the Steplet it is told about.
    fn mcp(pipe: &str, deadline_ms: u64) -> (Gate, std::path::PathBuf) {
        Gate::mcp_naming(pipe, deadline_ms, std::process::id())
    }

    /// `mcp`, told that Steplet is the program with process number `steplet`.
    fn mcp_naming(pipe: &str, deadline_ms: u64, steplet: u32) -> (Gate, std::path::PathBuf) {
        let log = std::env::temp_dir().join(format!("{pipe}.gate.log"));
        let _ = std::fs::remove_file(&log);
        let (ms, pid) = (deadline_ms.to_string(), steplet.to_string());
        let log_arg = log.to_str().unwrap();
        let gate = Gate::start(&[
            "--pipe",
            pipe,
            "--token",
            TOKEN,
            "--log",
            log_arg,
            "--deadline-ms",
            &ms,
            "--steplet-pid",
            &pid,
        ]);
        (gate, log)
    }

    fn send(&mut self, message: &Value) {
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{message}").unwrap();
        input.flush().unwrap();
    }

    fn send_raw(&mut self, line: &str) {
        let input = self.input.as_mut().unwrap();
        writeln!(input, "{line}").unwrap();
        input.flush().unwrap();
    }

    /// The next line the gate writes, as JSON, within `limit`.
    fn reply(&self, limit: Duration) -> Option<Value> {
        self.lines.recv_timeout(limit).ok().map(|l| serde_json::from_str(&l).expect("the gate writes JSON"))
    }

    /// Asks the gate's tool to decide on `tool` with `input`; returns the decision Claude Code would read, and how
    /// long it took.
    fn decide(&mut self, id: u64, tool: &str, input: Value) -> (Value, Duration) {
        let started = Instant::now();
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": "decide",
            "arguments": {"tool_name": tool, "input": input, "tool_use_id": format!("toolu_t8_{id}")}}}));
        let reply = self.reply(Duration::from_secs(20)).expect("the gate answers");
        assert_eq!(reply["id"], id, "{reply}");
        let content = reply["result"]["content"].as_array().expect("a tool result");
        assert_eq!(content.len(), 1, "{reply}");
        assert_eq!(content[0]["type"], "text", "{reply}");
        let decision = serde_json::from_str(content[0]["text"].as_str().unwrap()).expect("the decision is JSON");
        (decision, started.elapsed())
    }

    /// Closes standard input, as Claude Code does at the end of a run; returns the exit code.
    fn finish(mut self) -> Option<i32> {
        drop(self.input.take());
        let start = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status.code();
            }
            assert!(start.elapsed() < Duration::from_secs(10), "the gate did not end when its input closed");
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Gate {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn is_refusal(decision: &Value) -> bool {
    decision["behavior"] == "deny"
        && decision["message"].as_str().is_some_and(|m| !m.is_empty())
        && decision.as_object().unwrap().len() == 2
}

#[test]
fn the_handshake_offers_one_tool_and_ignores_what_it_cannot_read() {
    let (mut gate, _) = Gate::mcp(&pipe_name(), 2000);
    let wait = Duration::from_secs(5);
    gate.send(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}}));
    let init = gate.reply(wait).unwrap();
    assert_eq!(init["id"], 1);
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["capabilities"], json!({"tools": {}}));
    assert_eq!(init["result"]["serverInfo"]["name"], "steplet-gate");

    gate.send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    gate.send_raw("this line is not JSON");
    gate.send(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    let list = gate.reply(wait).unwrap();
    assert_eq!(list["id"], 2, "the notification and the unreadable line got no reply: {list}");
    let tools = list["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "decide");
    assert_eq!(tools[0]["inputSchema"]["required"], json!(["tool_name", "input"]));

    gate.send(&json!({"jsonrpc": "2.0", "id": 3, "method": "ping"}));
    assert_eq!(gate.reply(wait).unwrap(), json!({"jsonrpc": "2.0", "id": 3, "result": {}}));
    gate.send(&json!({"jsonrpc": "2.0", "id": 4, "method": "resources/list"}));
    assert_eq!(gate.reply(wait).unwrap()["error"]["code"], -32601);
    // Only the gate's own tool decides; a call to any other name is an error, and nothing is asked of Steplet.
    gate.send(
        &json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "other", "arguments": {}}}),
    );
    let other = gate.reply(wait).unwrap();
    assert_eq!(other["id"], 5);
    assert!(other.get("error").is_some(), "{other}");
    assert_eq!(gate.finish(), Some(0), "the gate ends when Claude Code closes its input");
}

#[test]
fn steplet_s_answer_reaches_claude_code_and_the_request_carries_the_run_s_token() {
    let pipe = pipe_name();
    let input = json!({"command": "cargo test", "description": "Run the tests"});
    let allow = json!({"behavior": "allow", "updatedInput": input}).to_string();
    let deny = json!({"behavior": "deny", "message": "the file does not ask for this command"}).to_string();
    let steplet = StandIn::start(&pipe, vec![Reply::Line(allow), Reply::Line(deny)]);
    let (mut gate, _) = Gate::mcp(&pipe, 2000);

    let (first, took) = gate.decide(1, "Bash", input.clone());
    println!("--- allowed in {} ms: {first}", took.as_millis());
    assert_eq!(first, json!({"behavior": "allow", "updatedInput": input}));
    let (second, _) = gate.decide(2, "Bash", json!({"command": "rm -rf src"}));
    assert_eq!(second, json!({"behavior": "deny", "message": "the file does not ask for this command"}));

    let seen = steplet.seen();
    println!("--- what Steplet received: {seen:?}");
    assert_eq!(
        seen[0],
        json!({"kind": "decide", "run_token": TOKEN, "tool_name": "Bash", "input": input, "tool_use_id": "toolu_t8_1"})
    );
    assert_eq!(seen.len(), 2, "one connection per request");
    assert_eq!(gate.finish(), Some(0));
}

#[test]
fn an_answer_of_the_wrong_shape_is_a_refusal() {
    // F38: Claude Code itself refuses anything but the two exact shapes; the gate does too, before it gets there.
    let pipe = pipe_name();
    let wrong = [
        "not JSON".to_owned(),
        json!({"behavior": "allow"}).to_string(),
        json!({"behavior": "allow", "updatedInput": "rm -rf /"}).to_string(),
        json!({"behavior": "allow", "updatedInput": {"command": "x"}, "extra": 1}).to_string(),
        json!({"behavior": "ALLOW", "updatedInput": {}}).to_string(),
        json!({"behavior": "deny"}).to_string(),
        json!({"behavior": "deny", "message": ""}).to_string(),
        json!(["allow"]).to_string(),
        json!({"key": "sk-made-up"}).to_string(),
    ];
    let steplet = StandIn::start(&pipe, wrong.iter().cloned().map(Reply::Line).collect());
    let (mut gate, _) = Gate::mcp(&pipe, 2000);
    for (n, answer) in wrong.iter().enumerate() {
        let (decision, _) = gate.decide(n as u64 + 1, "Write", json!({"file_path": "/work/r/repo/a", "content": "x"}));
        assert!(is_refusal(&decision), "{answer} became {decision}");
    }
    assert_eq!(steplet.seen().len(), wrong.len());
}

#[test]
fn a_pipe_closed_without_an_answer_and_a_missing_pipe_are_refusals() {
    let pipe = pipe_name();
    let ok = json!({"behavior": "allow", "updatedInput": {"file_path": "/work/r/repo/README"}}).to_string();
    let _steplet = StandIn::start(&pipe, vec![Reply::Close, Reply::Line(ok)]);
    let (mut gate, _) = Gate::mcp(&pipe, 2000);
    let (closed, _) = gate.decide(1, "Read", json!({"file_path": "/work/r/repo/README"}));
    assert!(is_refusal(&closed), "{closed}");
    // No state between requests: the next one is answered as if nothing had happened.
    let (next, _) = gate.decide(2, "Read", json!({"file_path": "/work/r/repo/README"}));
    assert_eq!(next["behavior"], "allow", "{next}");

    let (mut lone, _) = Gate::mcp(&pipe_name(), 2000);
    let (missing, took) = lone.decide(1, "Read", json!({"file_path": "/work/r/repo/README"}));
    println!("--- no pipe: refused in {} ms: {missing}", took.as_millis());
    assert!(is_refusal(&missing), "{missing}");
    assert!(took < Duration::from_secs(1), "no pipe is a refusal at once, not at the deadline");
}

#[test]
fn no_answer_from_steplet_is_a_refusal_at_steplet_s_deadline() {
    // F36: Claude Code waits for ever on a gate that never answers, so the gate keeps its own timer: Steplet's
    // deadline plus a short grace, so that Steplet's own refusal at its deadline normally comes first.
    let pipe = pipe_name();
    let ok = json!({"behavior": "allow", "updatedInput": {"file_path": "/work/r/repo/README"}}).to_string();
    let _steplet = StandIn::start(&pipe, vec![Reply::Hold(Duration::from_secs(60)), Reply::Line(ok)]);
    let (mut gate, log) = Gate::mcp(&pipe, 1000);
    let (held, took) = gate.decide(1, "Read", json!({"file_path": "/work/r/repo/README"}));
    println!("--- Steplet held the request: refused after {} ms: {held}", took.as_millis());
    assert!(is_refusal(&held), "{held}");
    assert!(took >= Duration::from_millis(1000), "not before the deadline: {took:?}");
    assert!(took < Duration::from_secs(5), "within the deadline and its grace: {took:?}");
    let (next, _) = gate.decide(2, "Read", json!({"file_path": "/work/r/repo/README"}));
    assert_eq!(next["behavior"], "allow", "the held request does not hold up the next: {next}");
    assert_eq!(gate.finish(), Some(0));
    println!("--- gate log:\n{}", std::fs::read_to_string(&log).unwrap());
}

#[test]
fn the_log_names_each_decision_but_never_its_input() {
    let pipe = pipe_name();
    let input = json!({"file_path": "/work/r/repo/notes.txt", "content": "T8-INPUT-TEXT-NOT-FOR-THE-LOG"});
    let allow = json!({"behavior": "allow", "updatedInput": input}).to_string();
    let _steplet = StandIn::start(&pipe, vec![Reply::Line(allow)]);
    let (mut gate, log) = Gate::mcp(&pipe, 2000);
    gate.decide(7, "Write", input);
    assert_eq!(gate.finish(), Some(0));
    let text = std::fs::read_to_string(&log).unwrap();
    println!("--- gate log:\n{text}");
    assert!(text.lines().next().unwrap().contains("started"), "{text}");
    assert!(text.contains("tool=Write id=toolu_t8_7 behavior=allow"), "{text}");
    assert!(!text.contains("T8-INPUT-TEXT"), "the input is not logged");
    assert!(!text.contains(TOKEN), "the token is not logged");
}

/// Runs the gate's key mode against `replies`; returns its exit code and everything it printed.
fn key_mode(pipe: &str, deadline_ms: u64) -> (Option<i32>, Vec<u8>, Duration) {
    key_mode_naming(pipe, deadline_ms, std::process::id())
}

/// `key_mode`, told that Steplet is the program with process number `steplet`.
fn key_mode_naming(pipe: &str, deadline_ms: u64, steplet: u32) -> (Option<i32>, Vec<u8>, Duration) {
    let started = Instant::now();
    let (ms, pid) = (deadline_ms.to_string(), steplet.to_string());
    let out = Command::new(GATE)
        .args(["--key", "--pipe", pipe, "--token", TOKEN, "--deadline-ms", &ms, "--steplet-pid", &pid])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .unwrap();
    (out.status.code(), out.stdout, started.elapsed())
}

#[test]
fn the_key_mode_prints_the_key_alone_or_nothing() {
    // H2: Claude Code's apiKeyHelper. A made-up key; never a real one.
    let pipe = pipe_name();
    let key = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";
    let steplet = StandIn::start(
        &pipe,
        vec![
            Reply::Line(json!({"key": key}).to_string()),
            Reply::Line(json!({"error": "wrong run token"}).to_string()),
            Reply::Line(json!({"key": ""}).to_string()),
            Reply::Line("not JSON".into()),
            Reply::Close,
            Reply::Hold(Duration::from_secs(60)),
        ],
    );
    let (code, printed, _) = key_mode(&pipe, 1000);
    assert_eq!((code, printed.as_slice()), (Some(0), key.as_bytes()), "the key and nothing else, no line ending");
    assert_eq!(steplet.seen()[0], json!({"kind": "key", "run_token": TOKEN}));
    for case in ["an error", "an empty key", "not JSON", "closed"] {
        let (code, printed, _) = key_mode(&pipe, 1000);
        assert_eq!((code, printed.len()), (Some(1), 0), "{case}");
    }
    let (code, printed, took) = key_mode(&pipe, 1000);
    println!("--- key mode, Steplet held the request: exit code {code:?} after {} ms", took.as_millis());
    assert_eq!((code, printed.len()), (Some(1), 0), "held");
    assert!(took < Duration::from_secs(5), "{took:?}");
    let (code, printed, _) = key_mode(&pipe_name(), 1000);
    assert_eq!((code, printed.len()), (Some(1), 0), "no pipe");
}

/// Phase 3 measured that another program of the same Windows account can add its own end to Steplet's pipe (K5). The
/// gate is told which program Steplet is, and talks to no other: its requests carry the run's token.
#[test]
fn the_gate_talks_only_to_the_steplet_that_started_the_run() {
    let pipe = pipe_name();
    let allow = json!({"behavior": "allow", "updatedInput": {"file_path": "/work/r/repo/README"}}).to_string();
    let key = json!({"key": "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY"}).to_string();
    // This test program holds the pipe's ends, and would allow anything and hand over a key.
    let other_end = StandIn::start(&pipe, vec![Reply::Line(allow), Reply::Line(key)]);
    // The program the gate is told is Steplet: any other running program, here an idle gate of its own.
    let (named, _) = Gate::mcp(&pipe_name(), 2000);
    let steplet = named.child.id();
    let (mut gate, log) = Gate::mcp_naming(&pipe, 2000, steplet);
    let (decision, _) = gate.decide(1, "Read", json!({"file_path": "/work/r/repo/README"}));
    println!("--- told Steplet is process {steplet}, at an end held by process {}: {decision}", std::process::id());
    assert!(is_refusal(&decision), "{decision}");
    let message = decision["message"].as_str().unwrap();
    assert!(message.contains("the program at Steplet's end is not the Steplet that started this run"), "{message}");
    assert_eq!(gate.reply(Duration::from_millis(500)), None, "nothing but the refusal reaches Claude Code");
    let (code, printed, _) = key_mode_naming(&pipe, 2000, steplet);
    assert_eq!((code, printed.len()), (Some(1), 0), "the key mode takes no key from another program");
    assert_eq!(other_end.seen(), Vec::<Value>::new(), "the other program got no request, so not the run's token");
    assert_eq!(gate.finish(), Some(0));
    let text = std::fs::read_to_string(&log).unwrap();
    println!("--- gate log:\n{text}");
    let why = "refused by the gate itself: the program at Steplet's end is not the Steplet that started this run";
    assert!(text.contains(why), "{text}");
}

/// The near miss of the test above: told the program that does hold Steplet's end, the gate asks it as before.
#[test]
fn the_gate_answers_when_steplet_s_end_is_the_one_named() {
    let pipe = pipe_name();
    let input = json!({"file_path": "/work/r/repo/README"});
    let allow = json!({"behavior": "allow", "updatedInput": input}).to_string();
    let key = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";
    let steplet = StandIn::start(&pipe, vec![Reply::Line(allow), Reply::Line(json!({"key": key}).to_string())]);
    let (mut gate, _) = Gate::mcp_naming(&pipe, 2000, std::process::id());
    let (decision, _) = gate.decide(1, "Read", input.clone());
    assert_eq!(decision, json!({"behavior": "allow", "updatedInput": input}));
    let (code, printed, _) = key_mode_naming(&pipe, 2000, std::process::id());
    assert_eq!((code, printed.as_slice()), (Some(0), key.as_bytes()));
    assert_eq!(steplet.seen().len(), 2, "both requests reached Steplet");
    assert_eq!(gate.finish(), Some(0));
}

#[test]
fn options_it_does_not_know_are_refused() {
    let log = std::env::temp_dir().join("steplet-gate-test-options.log");
    let log = log.to_str().unwrap();
    let me = std::process::id().to_string();
    let me = me.as_str();
    for args in [
        vec!["--token", TOKEN, "--log", log, "--deadline-ms", "1000", "--steplet-pid", me],
        vec!["--pipe", "p", "--log", log, "--deadline-ms", "1000", "--steplet-pid", me],
        vec!["--pipe", "p", "--token", TOKEN, "--deadline-ms", "1000", "--steplet-pid", me],
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--steplet-pid", me],
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--deadline-ms", "0", "--steplet-pid", me],
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--deadline-ms", "soon", "--steplet-pid", me],
        // More than an hour, Steplet's own limit; the largest number would overflow the gate's clock.
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--deadline-ms", "3600001", "--steplet-pid", me],
        vec![
            "--pipe",
            "p",
            "--token",
            TOKEN,
            "--log",
            log,
            "--deadline-ms",
            "18446744073709551615",
            "--steplet-pid",
            me,
        ],
        vec![
            "--pipe",
            "p",
            "--token",
            TOKEN,
            "--log",
            log,
            "--deadline-ms",
            "1000",
            "--steplet-pid",
            me,
            "--crash-once",
        ],
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--deadline-ms", "1000", "--steplet-pid", me, "extra"],
        vec![
            "--pipe",
            "p",
            "--pipe",
            "q",
            "--token",
            TOKEN,
            "--log",
            log,
            "--deadline-ms",
            "1000",
            "--steplet-pid",
            me,
        ],
        vec!["--key", "--pipe", "p", "--token", TOKEN, "--steplet-pid", me],
        // Without Steplet's process number, or with one that names no program, the gate does not start.
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--deadline-ms", "1000"],
        vec!["--key", "--pipe", "p", "--token", TOKEN, "--deadline-ms", "1000"],
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--deadline-ms", "1000", "--steplet-pid", "0"],
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--deadline-ms", "1000", "--steplet-pid", "steplet"],
        vec!["--pipe", "p", "--token", TOKEN, "--log", log, "--deadline-ms", "1000", "--steplet-pid", "4294967296"],
        vec![
            "--pipe",
            "p",
            "--token",
            TOKEN,
            "--log",
            log,
            "--deadline-ms",
            "1000",
            "--steplet-pid",
            me,
            "--steplet-pid",
            me,
        ],
    ] {
        let out = Command::new(GATE).args(&args).stdin(Stdio::null()).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        let mut said = String::new();
        out.stderr.as_slice().read_to_string(&mut said).unwrap();
        assert!(said.starts_with("steplet-gate: "), "{args:?}: {said}");
    }
}
