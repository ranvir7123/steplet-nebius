//! The gate service, Steplet's end of the run's named pipe (plan Task 8). The tests without `#[ignore]` play the
//! gate's part from this test program, so the service is told to expect this program as the gate; they need only
//! Windows. The tests marked `#[ignore]` run the real gate program, started by Claude Code inside Steplet's Linux
//! with the scripted model, and need that Linux and the network:
//! `cargo test -p steplet-run --offline --test gate -- --ignored --nocapture --test-threads=1`.

mod support;

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    ApiKey, Asked, Event, GateAnswer, GateError, GateLink, GateRequest, GateService, RunPaths, RunRequest, decide,
    discard, gate_for_run, launch, prepare, serve,
};
use support::{MADE_UP_KEY, logs_dir, scripted_model};
use windows_sys::Win32::Storage::FileSystem::SECURITY_IDENTIFICATION;

const TOKEN: &str = "t8servicetoken0123";
const ERROR_ACCESS_DENIED: i32 = 5;
const ERROR_FILE_NOT_FOUND: i32 = 2;
const ERROR_PIPE_BUSY: i32 = 231;

/// A pipe name no other test uses.
fn pipe_name() -> String {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    format!("steplet-t8-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))
}

/// This test program, as Windows names it: in these tests it plays the gate.
fn this_program() -> String {
    std::env::current_exe().unwrap().to_string_lossy().into_owned()
}

fn link(pipe: &str, deadline: Duration) -> GateLink {
    let log = logs_dir().join(format!("{pipe}.gate.log"));
    GateLink::new("/opt/steplet/gate/steplet-gate.exe", pipe, TOKEN, &log.to_string_lossy(), deadline).unwrap()
}

/// The checker for a file that declares one command.
fn policy(work: &str) -> Policy {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n    - touch declared.txt\n---\n";
    Policy::from_steplet(&parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(), work).unwrap()
}

/// The service for `pipe`, expecting this test program as the gate, deciding with the checker.
fn service(pipe: &str, deadline: Duration) -> GateService {
    let p = policy("/work/t8/repo");
    serve(&link(pipe, deadline), &this_program(), Some(ApiKey::new(MADE_UP_KEY.into())), move |r| decide(&p, r))
        .unwrap()
}

/// Opens the pipe as the gate does: read and write, letting Steplet identify this program but not act as it.
fn connect(pipe: &str) -> io::Result<File> {
    let start = Instant::now();
    loop {
        let opened = OpenOptions::new()
            .read(true)
            .write(true)
            .security_qos_flags(SECURITY_IDENTIFICATION)
            .open(format!(r"\\.\pipe\{pipe}"));
        match opened {
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) && start.elapsed() < Duration::from_secs(5) => {
                thread::sleep(Duration::from_millis(10));
            }
            other => return other,
        }
    }
}

/// Sends one request line and reads the one answer line.
fn ask(pipe: &str, request: &Value) -> io::Result<Value> {
    ask_raw(pipe, request.to_string().as_bytes())
}

fn ask_raw(pipe: &str, line: &[u8]) -> io::Result<Value> {
    let mut f = connect(pipe)?;
    f.write_all(line)?;
    f.write_all(b"\n")?;
    let mut answer = String::new();
    BufReader::new(&f).read_line(&mut answer)?;
    serde_json::from_str(&answer).map_err(|e| io::Error::other(format!("{e}: {answer:?}")))
}

fn decision(tool: &str, input: Value, id: &str) -> Value {
    json!({"kind": "decide", "run_token": TOKEN, "tool_name": tool, "input": input, "tool_use_id": id})
}

fn key_request(token: &str) -> Value {
    json!({"kind": "key", "run_token": token})
}

fn is_refusal(answer: &Value) -> bool {
    answer["behavior"] == "deny" && answer["message"].as_str().is_some_and(|m| !m.is_empty())
}

/// True once every answer in the record has been read by its client.
fn all_read(asked: &[Asked]) -> bool {
    asked.iter().all(|a| match a {
        Asked::Decision { delivered, .. } => *delivered,
        Asked::Key { served, .. } => *served,
        Asked::Refused { .. } => true,
    })
}

/// The record once `ready` holds for it, or after 2 s: a record is marked read a moment after its client has
/// the answer.
fn requests_once(gate: &GateService, ready: impl Fn(&[Asked]) -> bool) -> Vec<Asked> {
    let start = Instant::now();
    loop {
        let asked = gate.requests();
        if ready(&asked) || start.elapsed() > Duration::from_secs(2) {
            return asked;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn refused_reasons(gate: &GateService) -> Vec<String> {
    gate.requests()
        .into_iter()
        .filter_map(|a| match a {
            Asked::Refused { reason, .. } => Some(reason),
            _ => None,
        })
        .collect()
}

#[test]
fn the_checker_decides_each_request_and_the_service_records_who_asked() {
    let pipe = pipe_name();
    let gate = service(&pipe, Duration::from_secs(5));
    let declared = json!({"command": "touch declared.txt", "description": "Make the marker"});
    let allowed = ask(&pipe, &decision("Bash", declared.clone(), "toolu_a")).unwrap();
    assert_eq!(allowed, json!({"behavior": "allow", "updatedInput": declared}), "exactly the request (R7)");
    let refused = ask(&pipe, &decision("Bash", json!({"command": "touch undeclared.txt"}), "toolu_b")).unwrap();
    assert!(is_refusal(&refused), "{refused}");
    assert!(refused["message"].as_str().unwrap().contains("the file does not ask for this command"), "{refused}");

    let asked = requests_once(&gate, |a| a.len() == 2 && all_read(a));
    println!("--- recorded: {asked:#?}");
    assert!(!format!("{asked:?}").contains(TOKEN), "the run's token is never shown, even in the record");
    let shown_link = format!("{:?}", link(&pipe, Duration::from_secs(5)));
    assert!(!shown_link.contains(TOKEN), "nor in the gate link: {shown_link}");
    assert_eq!(asked.len(), 2);
    for (a, (id, allow)) in asked.iter().zip([("toolu_a", true), ("toolu_b", false)]) {
        let Asked::Decision { client, request, answer, took, delivered } = a else { panic!("{a:?}") };
        assert_eq!(request.tool_use_id, id);
        assert_eq!(matches!(answer, GateAnswer::Allow { .. }), allow, "{answer:?}");
        assert!(*delivered, "the answer reached the client");
        assert!(*took < Duration::from_secs(1), "{took:?}");
        assert_eq!(client.pid, std::process::id());
        assert!(client.program.eq_ignore_ascii_case(&this_program()), "{}", client.program);
        assert!(!client.restricted && !client.app_container, "{client:?}");
        assert!(client.identify_only, "the client let Steplet identify it, nothing more: {client:?}");
    }
}

/// What the tripwire watcher (Task 9) asks when a tool call's result arrives: was the gate asked about this call,
/// and was any answer "allow"? Each answer is on record before it is sent, so no waiting is needed.
#[test]
fn the_service_says_for_each_tool_call_whether_it_was_asked_and_what_it_answered() {
    let pipe = pipe_name();
    let gate = service(&pipe, Duration::from_secs(5));
    let declared = json!({"command": "touch declared.txt"});
    let undeclared = json!({"command": "touch undeclared.txt"});
    assert!(!is_refusal(&ask(&pipe, &decision("Bash", declared.clone(), "toolu_a")).unwrap()));
    assert!(is_refusal(&ask(&pipe, &decision("Bash", undeclared.clone(), "toolu_b")).unwrap()));
    // Asked twice about one call (a gate retrying): one "allow" is enough for it to have been allowed.
    assert!(is_refusal(&ask(&pipe, &decision("Bash", undeclared, "toolu_c")).unwrap()));
    assert!(!is_refusal(&ask(&pipe, &decision("Bash", declared.clone(), "toolu_c")).unwrap()));
    let no_id = json!({"kind": "decide", "run_token": TOKEN, "tool_name": "Bash", "input": declared});
    assert!(!is_refusal(&ask(&pipe, &no_id).unwrap()));

    assert_eq!(gate.answered("toolu_a"), Some(true));
    assert_eq!(gate.answered("toolu_b"), Some(false));
    assert_eq!(gate.answered("toolu_c"), Some(true));
    assert_eq!(gate.answered("toolu_never_asked"), None);
    assert_eq!(gate.answered(""), None, "a request without an id matches no tool call");

    let asked = gate.into_requests();
    assert_eq!(asked.len(), 5, "{asked:#?}");
    let after = connect(&pipe).map(drop);
    assert_eq!(after.err().and_then(|e| e.raw_os_error()), Some(ERROR_FILE_NOT_FOUND), "the service has stopped");
}

/// Stopping a run takes a moment (Task 9 measured 1 to 2 s); from the tripwire on, nothing more is allowed.
#[test]
fn once_steplet_is_stopping_the_run_every_request_is_refused() {
    let pipe = pipe_name();
    let gate = service(&pipe, Duration::from_secs(5));
    let declared = json!({"command": "touch declared.txt"});
    assert!(!is_refusal(&ask(&pipe, &decision("Bash", declared.clone(), "toolu_a")).unwrap()));
    gate.refuse_everything();
    let refused = ask(&pipe, &decision("Bash", declared, "toolu_b")).unwrap();
    assert!(is_refusal(&refused), "{refused}");
    assert!(refused["message"].as_str().unwrap().contains("Steplet is stopping this run"), "{refused}");
    assert_eq!(gate.answered("toolu_b"), Some(false), "the refusal is on record");
    assert!(ask(&pipe, &key_request(TOKEN)).unwrap()["key"].is_string(), "the key is still served");
    let asked = gate.into_requests();
    assert_eq!(asked.len(), 3, "{asked:#?}");
    let after = connect(&pipe).map(drop);
    assert_eq!(after.err().and_then(|e| e.raw_os_error()), Some(ERROR_FILE_NOT_FOUND), "the service has stopped");
}

#[test]
fn a_wrong_run_token_is_refused_and_the_key_goes_only_to_the_run_s_gate() {
    let pipe = pipe_name();
    let gate = service(&pipe, Duration::from_secs(5));
    let mut wrong = decision("Read", json!({"file_path": "/work/t8/repo/README"}), "toolu_c");
    wrong["run_token"] = "t8wrongtoken".into();
    let refused = ask(&pipe, &wrong).unwrap();
    assert!(is_refusal(&refused), "{refused}");
    let no_key = ask(&pipe, &key_request("t8wrongtoken")).unwrap();
    assert!(no_key.get("key").is_none() && no_key["error"].is_string(), "{no_key}");
    let key = ask(&pipe, &key_request(TOKEN)).unwrap();
    assert_eq!(key, json!({"key": MADE_UP_KEY}));

    let asked = requests_once(&gate, |a| a.len() == 3 && all_read(a));
    let shown = format!("{asked:?}");
    println!("--- recorded: {shown}");
    assert!(!shown.contains(MADE_UP_KEY), "the key is never in the record");
    assert_eq!(refused_reasons(&gate).len(), 2, "both wrong-token requests: {shown}");
    assert!(refused_reasons(&gate).iter().all(|r| r.contains("run token")), "{shown}");
    assert!(matches!(asked.last(), Some(Asked::Key { served: true, .. })), "{shown}");

    // A service given no key serves none.
    let bare_pipe = pipe_name();
    let p = policy("/work/t8/repo");
    let bare = serve(&link(&bare_pipe, Duration::from_secs(5)), &this_program(), None, move |r| decide(&p, r)).unwrap();
    let none = ask(&bare_pipe, &key_request(TOKEN)).unwrap();
    assert!(none.get("key").is_none(), "{none}");
    let bare_asked = requests_once(&bare, |a| !a.is_empty());
    assert!(matches!(bare_asked.as_slice(), [Asked::Key { served: false, .. }]), "{bare_asked:?}");
}

#[test]
fn a_program_other_than_the_gate_is_refused() {
    let pipe = pipe_name();
    let p = policy("/work/t8/repo");
    let elsewhere = r"\\wsl.localhost\steplet\opt\steplet\gate\steplet-gate-0000000000000000.exe";
    let key = Some(ApiKey::new(MADE_UP_KEY.into()));
    let gate = serve(&link(&pipe, Duration::from_secs(5)), elsewhere, key, move |r| decide(&p, r)).unwrap();
    let declared = json!({"command": "touch declared.txt"});
    let refused = ask(&pipe, &decision("Bash", declared, "toolu_d")).unwrap();
    assert!(is_refusal(&refused), "the right token from the wrong program is refused: {refused}");
    let no_key = ask(&pipe, &key_request(TOKEN)).unwrap();
    assert!(no_key.get("key").is_none(), "{no_key}");
    let asked = gate.requests();
    println!("--- recorded: {asked:#?}");
    assert_eq!(asked.len(), 2);
    for a in &asked {
        let Asked::Refused { client: Some(client), reason, .. } = a else { panic!("{a:?}") };
        assert!(reason.contains("program"), "{reason}");
        assert!(client.program.eq_ignore_ascii_case(&this_program()));
    }
}

/// Tokens made from this test program's own, for the clients below (Windows calls, test support only).
mod tokens {
    use std::ptr::null;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        CreateRestrictedToken, CreateWellKnownSid, DuplicateTokenEx, GetLengthSid, GetTokenInformation, PSID,
        RevertToSelf, SID_AND_ATTRIBUTES, SecurityImpersonation, SetTokenInformation, TOKEN_ADJUST_DEFAULT,
        TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_IMPERSONATE, TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TOKEN_USER,
        TokenImpersonation, TokenIntegrityLevel, TokenPrimary, TokenUser, WinLowLabelSid, WinWorldSid,
    };
    use windows_sys::Win32::System::SystemServices::SE_GROUP_INTEGRITY;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken, SetThreadToken};

    /// A token handle, closed when dropped.
    pub struct Token(HANDLE);

    impl Drop for Token {
        fn drop(&mut self) {
            // SAFETY: the handle is open and owned by this value.
            unsafe { CloseHandle(self.0) };
        }
    }

    fn check(ok: i32, what: &str) {
        assert_ne!(ok, 0, "{what}: {}", std::io::Error::last_os_error());
    }

    pub fn own() -> Token {
        let mut h = std::ptr::null_mut();
        let access = TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY | TOKEN_ADJUST_DEFAULT | TOKEN_IMPERSONATE;
        // SAFETY: the current process's pseudo handle; `h` receives a new handle.
        check(unsafe { OpenProcessToken(GetCurrentProcess(), access, &mut h) }, "open this program's token");
        Token(h)
    }

    /// This program's user, as a SID's bytes.
    pub fn user(token: &Token) -> Vec<u8> {
        let mut buf = vec![0u64; 64];
        let mut len = 0;
        // SAFETY: `buf` is 512 aligned bytes, enough for a TOKEN_USER and its SID.
        check(
            unsafe { GetTokenInformation(token.0, TokenUser, buf.as_mut_ptr().cast(), 512, &mut len) },
            "read the user",
        );
        // SAFETY: the call filled `buf` with a TOKEN_USER whose SID points inside `buf`.
        unsafe {
            let sid = (*buf.as_ptr().cast::<TOKEN_USER>()).User.Sid;
            std::slice::from_raw_parts(sid.cast::<u8>(), GetLengthSid(sid) as usize).to_vec()
        }
    }

    pub fn everyone() -> Vec<u8> {
        well_known(WinWorldSid)
    }

    fn well_known(kind: i32) -> Vec<u8> {
        let mut sid = vec![0u8; 68];
        let mut len = 68;
        // SAFETY: `sid` has `len` bytes.
        check(
            unsafe { CreateWellKnownSid(kind, null::<u8>() as PSID, sid.as_mut_ptr().cast(), &mut len) },
            "make a SID",
        );
        sid.truncate(len as usize);
        sid
    }

    /// A restricted version of `token`: every access is also checked against `sids` alone.
    pub fn restricted(token: &Token, sids: &mut [Vec<u8>]) -> Token {
        let list: Vec<SID_AND_ATTRIBUTES> =
            sids.iter_mut().map(|s| SID_AND_ATTRIBUTES { Sid: s.as_mut_ptr().cast(), Attributes: 0 }).collect();
        let mut h = std::ptr::null_mut();
        // SAFETY: `list` points into `sids`, which outlive the call.
        let ok = unsafe {
            CreateRestrictedToken(token.0, 0, 0, null(), 0, null(), list.len() as u32, list.as_ptr(), &mut h)
        };
        check(ok, "make a restricted token");
        Token(h)
    }

    /// A copy of `token` at low integrity.
    pub fn low_integrity(token: &Token) -> Token {
        let mut h = std::ptr::null_mut();
        let access = TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY | TOKEN_ADJUST_DEFAULT | TOKEN_IMPERSONATE;
        // SAFETY: `token` is open; `h` receives a new handle.
        check(
            unsafe { DuplicateTokenEx(token.0, access, null(), SecurityImpersonation, TokenPrimary, &mut h) },
            "copy the token",
        );
        let mut low = well_known(WinLowLabelSid);
        let label = TOKEN_MANDATORY_LABEL {
            Label: SID_AND_ATTRIBUTES { Sid: low.as_mut_ptr().cast(), Attributes: SE_GROUP_INTEGRITY as u32 },
        };
        let size = (std::mem::size_of::<TOKEN_MANDATORY_LABEL>() + low.len()) as u32;
        // SAFETY: `label` points at `low`, which outlives the call.
        check(
            unsafe { SetTokenInformation(h, TokenIntegrityLevel, (&raw const label).cast(), size) },
            "lower the integrity",
        );
        Token(h)
    }

    /// Runs `f` on this thread as `token`, then goes back to this program's own token.
    pub fn as_token<R>(token: &Token, f: impl FnOnce() -> R) -> R {
        let mut imp = std::ptr::null_mut();
        // SAFETY: `token` is open; `imp` receives a new handle.
        check(
            unsafe {
                DuplicateTokenEx(
                    token.0,
                    TOKEN_IMPERSONATE | TOKEN_QUERY,
                    null(),
                    SecurityImpersonation,
                    TokenImpersonation,
                    &mut imp,
                )
            },
            "make an impersonation token",
        );
        let imp = Token(imp);
        // SAFETY: a null thread means this thread; `imp` stays open while it is used.
        check(unsafe { SetThreadToken(null(), imp.0) }, "take on the token");
        let r = f();
        // SAFETY: undoes the SetThreadToken above.
        check(unsafe { RevertToSelf() }, "go back to this program's token");
        r
    }
}

#[test]
fn a_restricted_or_low_integrity_client_is_refused() {
    let pipe = pipe_name();
    let gate = service(&pipe, Duration::from_secs(5));
    let declared = || decision("Bash", json!({"command": "touch declared.txt"}), "toolu_e");
    let own = tokens::own();
    let mut with_user = vec![tokens::user(&own), tokens::everyone()];
    let mut everyone_only = vec![tokens::everyone()];
    let cases = [
        ("restricted, but its list still names this user", tokens::restricted(&own, &mut with_user)),
        ("restricted to Everyone", tokens::restricted(&own, &mut everyone_only)),
        ("low integrity", tokens::low_integrity(&own)),
    ];
    let control = ask(&pipe, &declared()).unwrap();
    assert_eq!(control["behavior"], "allow", "the same request from this program's own token: {control}");
    let mut outcomes = Vec::new();
    for (what, token) in &cases {
        let got = tokens::as_token(token, || ask(&pipe, &declared()));
        println!("--- {what}: {got:?}");
        outcomes.push(got);
    }
    // The first gets through Windows' own check, so Steplet's check must refuse it.
    let first = outcomes[0].as_ref().expect("Windows lets it connect");
    assert!(is_refusal(first), "{first}");
    let reasons = refused_reasons(&gate);
    assert!(reasons.iter().any(|r| r.contains("restricted")), "{reasons:?}");
    // The pipe's own permission list refuses the other two before they connect.
    for (n, got) in outcomes.iter().enumerate().skip(1) {
        let err = got.as_ref().expect_err(cases[n].0);
        assert_eq!(err.raw_os_error(), Some(ERROR_ACCESS_DENIED), "{}: {err}", cases[n].0);
    }
    println!("--- recorded: {:#?}", gate.requests());
}

#[test]
fn someone_holding_the_pipe_name_first_stops_the_gate_from_starting() {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX};
    use windows_sys::Win32::System::Pipes::{CreateNamedPipeW, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES};

    let create = |pipe: &str, first: bool| {
        let name: Vec<u16> = format!(r"\\.\pipe\{pipe}").encode_utf16().chain([0]).collect();
        let mode = PIPE_ACCESS_DUPLEX | if first { FILE_FLAG_FIRST_PIPE_INSTANCE } else { 0 };
        // SAFETY: `name` is NUL-terminated; default security.
        let h = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                mode,
                PIPE_TYPE_BYTE,
                PIPE_UNLIMITED_INSTANCES,
                4096,
                4096,
                0,
                std::ptr::null(),
            )
        };
        (h != INVALID_HANDLE_VALUE).then_some(h).ok_or_else(io::Error::last_os_error)
    };
    let pipe = pipe_name();
    let squatter = create(&pipe, true).expect("the test takes the name first");
    let p = policy("/work/t8/repo");
    let got = serve(&link(&pipe, Duration::from_secs(5)), &this_program(), None, move |r| decide(&p, r));
    println!("--- the name was taken first: {:?}", got.as_ref().err());
    assert!(matches!(got, Err(GateError::Taken)), "the run must not start on a pipe someone else holds");
    // SAFETY: the handle made above.
    unsafe { CloseHandle(squatter) };
    let started = service(&pipe, Duration::from_secs(5));
    let answer = ask(&pipe, &decision("Bash", json!({"command": "touch declared.txt"}), "toolu_f")).unwrap();
    assert_eq!(answer["behavior"], "allow", "once the name is free, the gate starts: {answer}");

    // While Steplet serves, a program at low integrity cannot add its own end to Steplet's pipe.
    let low = tokens::low_integrity(&tokens::own());
    let added_low = tokens::as_token(&low, || create(&pipe, false));
    println!("--- a low-integrity program adds an end to Steplet's pipe: {added_low:?}");
    assert_eq!(added_low.as_ref().err().and_then(io::Error::raw_os_error), Some(ERROR_ACCESS_DENIED));
    // A program of the same Windows account, at the same level, can (measured and recorded, not defended here:
    // Windows does not keep one account's programs apart).
    let added_same = create(&pipe, false);
    println!("--- a program of the same account adds an end to Steplet's pipe: {added_same:?}");
    if let Ok(h) = added_same {
        // SAFETY: the handle made above.
        unsafe { CloseHandle(h) };
    }
    drop(started);
}

#[test]
fn a_decision_that_never_comes_is_refused_at_steplet_s_deadline() {
    let pipe = pipe_name();
    let p = policy("/work/t8/repo");
    let stall = move |r: &GateRequest| {
        if r.tool_use_id == "toolu_stall" {
            thread::sleep(Duration::from_secs(3600));
        }
        decide(&p, r)
    };
    let gate = serve(&link(&pipe, Duration::from_millis(1500)), &this_program(), None, stall).unwrap();
    let started = Instant::now();
    let late = ask(&pipe, &decision("Bash", json!({"command": "touch declared.txt"}), "toolu_stall")).unwrap();
    let waited = started.elapsed();
    println!("--- the decision never came: refused after {} ms: {late}", waited.as_millis());
    assert!(is_refusal(&late), "{late}");
    assert!(waited >= Duration::from_millis(1500) && waited < Duration::from_millis(2500), "{waited:?}");
    let next = ask(&pipe, &decision("Bash", json!({"command": "touch declared.txt"}), "toolu_next")).unwrap();
    assert_eq!(next["behavior"], "allow", "a stuck decision does not hold up the next: {next}");
    let asked = gate.requests();
    let Asked::Decision { answer: GateAnswer::Deny { message }, took, .. } = &asked[0] else { panic!("{asked:?}") };
    println!("--- recorded: took {} ms, {message:?}", took.as_millis());
    assert!(*took >= Duration::from_millis(1500), "{took:?}");
}

/// Task 4's review: while Steplet is still deciding a request (in a live run, while the question is on the
/// receiver's screen), the gate service already knows the tool call, and counts it as not allowed. A result Claude
/// Code gives for it meanwhile (when the run is cut short) is then judged as a refused call, not as one that skipped
/// the gate.
#[test]
fn a_request_still_being_decided_counts_as_not_allowed() {
    let pipe = pipe_name();
    let p = policy("/work/t8/repo");
    let (go, wait) = std::sync::mpsc::channel::<()>();
    let wait = Mutex::new(wait);
    let slow = move |r: &GateRequest| {
        if r.tool_use_id == "toolu_slow" {
            let _ = wait.lock().unwrap().recv_timeout(Duration::from_secs(10));
        }
        decide(&p, r)
    };
    let gate = serve(&link(&pipe, Duration::from_secs(20)), &this_program(), None, slow).unwrap();
    let asking = {
        let pipe = pipe.clone();
        thread::spawn(move || ask(&pipe, &decision("Bash", json!({"command": "touch declared.txt"}), "toolu_slow")))
    };
    let start = Instant::now();
    while gate.answered("toolu_slow").is_none() && start.elapsed() < Duration::from_secs(5) {
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(gate.answered("toolu_slow"), Some(false), "while it is being decided");
    go.send(()).unwrap();
    assert_eq!(asking.join().unwrap().unwrap()["behavior"], "allow");
    assert_eq!(gate.answered("toolu_slow"), Some(true), "once allowed");
}

#[test]
fn a_client_that_says_nothing_or_nonsense_is_refused_and_the_next_is_answered() {
    let pipe = pipe_name();
    let gate = service(&pipe, Duration::from_millis(1000));
    let silent = connect(&pipe).unwrap();
    let silent_started = Instant::now();
    let declared = || decision("Bash", json!({"command": "touch declared.txt"}), "toolu_g");
    assert_eq!(ask(&pipe, &declared()).unwrap()["behavior"], "allow", "a silent client holds up no one");
    let mut rest = Vec::new();
    let closed = BufReader::new(&silent).read_to_end(&mut rest);
    println!("--- the silent client was let go after {} ms: {closed:?}", silent_started.elapsed().as_millis());
    assert!(silent_started.elapsed() < Duration::from_secs(3), "dropped at the deadline");

    let too_big = format!("{{\"kind\": \"decide\", \"pad\": \"{}\"}}", "x".repeat(17 << 20));
    for (what, line) in [
        ("not JSON", b"this is not JSON".to_vec()),
        ("not an object", b"[1, 2]".to_vec()),
        ("no kind", json!({"run_token": TOKEN}).to_string().into_bytes()),
        ("no tool", json!({"kind": "decide", "run_token": TOKEN, "input": {}}).to_string().into_bytes()),
        ("tool not text", decision("Bash", json!({}), "toolu_h").to_string().replace("\"Bash\"", "7").into_bytes()),
        ("larger than 16 MiB", too_big.into_bytes()),
    ] {
        let got = ask_raw(&pipe, &line);
        println!("--- {what}: {}", got.as_ref().map(Value::to_string).unwrap_or_else(|e| e.to_string()));
        // Steplet stops reading a request at its size limit, so the client may find the pipe closed mid-write.
        let closed_early = what.starts_with("larger") && got.is_err();
        assert!(closed_early || got.is_ok_and(|a| is_refusal(&a)), "{what}");
    }
    assert_eq!(ask(&pipe, &declared()).unwrap()["behavior"], "allow", "no state between requests");
    let reasons = refused_reasons(&gate);
    println!("--- refused: {reasons:?}");
    assert_eq!(reasons.len(), 7, "the silent client and the six unreadable requests");
}

#[test]
fn an_answer_the_client_never_read_is_recorded_as_not_delivered() {
    // What Steplet sees when the gate dies mid-request (F39): the answer cannot be sent.
    let pipe = pipe_name();
    let p = policy("/work/t8/repo");
    let slow = move |r: &GateRequest| {
        thread::sleep(Duration::from_millis(300));
        decide(&p, r)
    };
    let gate = serve(&link(&pipe, Duration::from_secs(5)), &this_program(), None, slow).unwrap();
    {
        let mut f = connect(&pipe).unwrap();
        writeln!(f, "{}", decision("Bash", json!({"command": "touch declared.txt"}), "toolu_i")).unwrap();
    } // closed before the answer comes
    let start = Instant::now();
    while gate.requests().is_empty() && start.elapsed() < Duration::from_secs(5) {
        thread::sleep(Duration::from_millis(20));
    }
    let asked = gate.requests();
    println!("--- recorded: {asked:?}");
    assert!(matches!(asked.as_slice(), [Asked::Decision { delivered: false, .. }]), "{asked:?}");
}

#[test]
fn a_client_that_never_reads_its_answer_is_let_go() {
    // An allowed write carries the whole file back, more than the pipe holds; a client that never reads it must
    // not keep its end of the pipe for ever.
    let pipe = pipe_name();
    let gate = serve(&link(&pipe, Duration::from_secs(1)), &this_program(), None, |r: &GateRequest| {
        GateAnswer::Allow { updated_input: r.input.clone() }
    })
    .unwrap();
    let content = "x".repeat(4 << 20);
    let mut f = connect(&pipe).unwrap();
    let request = decision("Write", json!({"file_path": "/work/t8/repo/big.txt", "content": content}), "toolu_j");
    writeln!(f, "{request}").unwrap();
    thread::sleep(Duration::from_secs(9));
    let mut got = Vec::new();
    let read = BufReader::new(&f).read_to_end(&mut got);
    println!("--- read after 9 s: {} of about {} bytes ({read:?})", got.len(), content.len());
    assert!(got.len() < content.len(), "Steplet stopped sending and closed the connection");
    let asked = gate.requests();
    assert!(matches!(asked.as_slice(), [Asked::Decision { delivered: false, .. }]), "not delivered");
}

#[test]
fn the_pipe_is_gone_once_the_service_stops() {
    // Many times over: before 17 Oct 2026 the free end of the pipe outlived `stop` in about 1 round in 10.
    const ROUNDS: usize = 500;
    let pipes: Vec<String> = (0..ROUNDS).map(|_| pipe_name()).collect();
    let mut still_there = Vec::new();
    for (round, pipe) in pipes.iter().enumerate() {
        let gate = service(pipe, Duration::from_secs(5));
        assert!(ask(pipe, &key_request(TOKEN)).is_ok(), "round {round}");
        gate.stop();
        let after = connect(pipe).map(drop);
        if after.as_ref().err().and_then(io::Error::raw_os_error) != Some(ERROR_FILE_NOT_FOUND) {
            still_there.push((round, after));
        }
    }
    let first = &still_there[..still_there.len().min(5)];
    println!("--- the pipe was still there after stop in {} of {ROUNDS} rounds, first {first:?}", still_there.len());
    assert!(still_there.is_empty());
    let pipe = &pipes[ROUNDS - 1];
    let again = service(pipe, Duration::from_secs(5));
    assert!(ask(pipe, &key_request(TOKEN)).is_ok(), "the same name can serve again");
    drop(again);
}

#[test]
fn the_pipe_is_gone_once_the_service_stops_even_with_a_checker_that_never_returns() {
    // The stuck checker keeps one of the service's threads for good: stopping must still close the pipe, and must
    // not wait for that thread.
    let pipe = pipe_name();
    let stall = |_: &GateRequest| {
        thread::sleep(Duration::from_secs(3600));
        GateAnswer::Deny { message: "too late".into() }
    };
    let gate = serve(&link(&pipe, Duration::from_millis(500)), &this_program(), None, stall).unwrap();
    let late = ask(&pipe, &decision("Bash", json!({"command": "touch declared.txt"}), "toolu_stuck")).unwrap();
    assert!(is_refusal(&late), "{late}");
    let started = Instant::now();
    gate.stop();
    let took = started.elapsed();
    let after = connect(&pipe).map(drop);
    println!("--- stop took {} ms; after stop: {after:?}", took.as_millis());
    assert!(took < Duration::from_millis(500), "stop did not wait for the stuck checker: {took:?}");
    assert_eq!(after.err().and_then(|e| e.raw_os_error()), Some(ERROR_FILE_NOT_FOUND));
    // No end of the old pipe is left: the first end of a new one can have the name.
    let again = service(&pipe, Duration::from_secs(5));
    assert!(ask(&pipe, &key_request(TOKEN)).is_ok(), "the same name can serve again");
    drop(again);
}

#[test]
fn the_gate_program_s_windows_name_is_its_path_in_steplet_s_linux() {
    assert_eq!(
        steplet_run::gate_on_windows("steplet", "/opt/steplet/gate/steplet-gate-0123456789abcdef.exe"),
        r"\\wsl.localhost\steplet\opt\steplet\gate\steplet-gate-0123456789abcdef.exe"
    );
}

// ---- Live: the real gate, started by Claude Code inside Steplet's Linux ----

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");

fn run_request(task_seconds: u64) -> RunRequest {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n    - touch declared.txt\n---\n";
    RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: HELLO.0.into(),
        commit: HELLO.1.into(),
        task_text: "Follow the script.".into(),
        max_turns: 8,
        deadline: Duration::from_secs(task_seconds),
        allow_model_switch: false,
    }
}

/// Everything a run sends until its channel closes or `limit` passes.
fn collect(run: &steplet_run::RunHandle, limit: Duration) -> Vec<Event> {
    let start = Instant::now();
    let mut events = Vec::new();
    while let Some(left) = limit.checked_sub(start.elapsed()) {
        match run.events().recv_timeout(left) {
            Ok(e) => events.push(e),
            Err(_) => break,
        }
    }
    events
}

fn print_short<T: std::fmt::Debug>(items: &[T]) {
    for item in items {
        let shown = format!("{item:?}");
        let cut: String = shown.chars().take(240).collect();
        println!("    {cut}{}", if cut.len() < shown.len() { "..." } else { "" });
    }
}

/// One prepared and launched run with the real gate, deciding with `decide`; returns its events, its exit code,
/// the gate service (still running, so a decision still being made can finish), the run's paths and the scripted
/// model's log.
fn live_run(
    run_id: &str,
    port: u16,
    script: &str,
    deadline: Duration,
    decide_with: impl Fn(&RunPaths) -> Box<dyn Fn(&GateRequest) -> GateAnswer + Send + Sync>,
) -> (Vec<Event>, Option<i32>, GateService, RunPaths, String) {
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, run_id).unwrap();
    let (link, gate) = support::serve_gate(&distro, run_id, deadline, decide_with(&paths));
    let _model = scripted_model(&distro, run_id, script, port);
    let req = run_request(120);
    prepare(&req, &paths, &link).unwrap();
    let started = Instant::now();
    let handle = launch(&req, &paths, &logs_dir(), Some(&format!("http://127.0.0.1:{port}")), &[]).unwrap();
    let events = collect(&handle, Duration::from_secs(120));
    let code = handle.wait().unwrap();
    println!("--- run {run_id}: exit code {code:?} after {:.1} s; every event:", started.elapsed().as_secs_f64());
    print_short(&events);
    let model_log = support::run(&distro, "root", &["/usr/bin/cat", &format!("/opt/steplet-test/{run_id}.model.log")]);
    (events, code, gate, paths, model_log)
}

/// The gate's record once it holds `decisions` decisions (or 10 s have passed) and every answer in it has been
/// read (or 2 s more have passed), printed.
fn settled(gate: &GateService, decisions_wanted: usize) -> Vec<Asked> {
    let start = Instant::now();
    while decisions(&gate.requests()).len() < decisions_wanted && start.elapsed() < Duration::from_secs(10) {
        thread::sleep(Duration::from_millis(50));
    }
    let asked = requests_once(gate, all_read);
    println!("--- the gate's record:");
    print_short(&asked);
    asked
}

fn exists(paths: &RunPaths, file: &str) -> bool {
    let path = format!("{}/{file}", paths.work_repo());
    steplet_run::wsl::run(paths.distro_id(), "root", &["/usr/bin/test", "-e", &path], b"").unwrap().status == Some(0)
}

fn decisions(asked: &[Asked]) -> Vec<(&GateRequest, &GateAnswer, Duration, bool)> {
    asked
        .iter()
        .filter_map(|a| match a {
            Asked::Decision { request, answer, took, delivered, .. } => Some((request, answer, *took, *delivered)),
            _ => None,
        })
        .collect()
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_declared_command_runs_and_what_the_file_did_not_ask_for_does_not() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = r#"{"steps": [
        {"tool": "Bash", "input": {"command": "touch declared.txt", "description": "Declared"}},
        {"tool": "Bash", "input": {"command": "touch undeclared.txt", "description": "Not declared"}},
        {"tool": "Write", "input": {"file_path": "/work/t8a/repo/written.txt", "content": "t8\n"}},
        {"text": "Task 8: done."}]}"#;
    let checker = |paths: &RunPaths| -> Box<dyn Fn(&GateRequest) -> GateAnswer + Send + Sync> {
        let p = policy(paths.work_repo());
        Box::new(move |r| decide(&p, r))
    };
    let (events, code, gate, paths, model_log) = live_run("t8a", 47920, script, Duration::from_secs(10), checker);
    let asked = settled(&gate, 3);
    let markers: Vec<bool> =
        ["declared.txt", "undeclared.txt", "written.txt"].iter().map(|f| exists(&paths, f)).collect();
    println!("--- declared.txt, undeclared.txt, written.txt exist: {markers:?}");
    assert_eq!(code, Some(0));
    assert_eq!(markers, [true, false, false]);

    // Every tool call asked the gate, and the gate's answers are the checker's.
    let uses: Vec<(&str, &str)> = events
        .iter()
        .filter_map(|e| match e {
            Event::ToolUse { id, name, .. } => Some((id.as_str(), name.as_str())),
            _ => None,
        })
        .collect();
    let decided = decisions(&asked);
    assert_eq!(decided.len(), 3, "{asked:?}");
    for ((id, name), (request, answer, took, delivered)) in uses.iter().zip(&decided) {
        println!("--- {name} {id}: {answer:?} in {} ms", took.as_millis());
        assert_eq!((request.tool_use_id.as_str(), request.tool_name.as_str()), (*id, *name));
        assert!(*delivered);
    }
    let allowed: Vec<bool> = decided.iter().map(|d| matches!(d.1, GateAnswer::Allow { .. })).collect();
    assert_eq!(allowed, [true, false, false]);

    // The key went to Claude Code through the gate's key mode, and only that way (H2).
    assert!(asked.iter().any(|a| matches!(a, Asked::Key { served: true, .. })), "{asked:?}");
    assert!(model_log.contains("\"api_key_matches_expected\": true"), "the scripted model got the made-up key");
    assert!(!model_log.contains("\"api_key_matches_expected\": false"));

    // Who the gate is, as Windows sees it (the Task 1 gap): every connection came from the gate program.
    let me = steplet_run::Client::this_program().unwrap();
    println!("--- Steplet itself: {me:?}");
    for a in &asked {
        let client = match a {
            Asked::Decision { client, .. } | Asked::Key { client, .. } => client,
            Asked::Refused { .. } => panic!("nothing was refused before deciding: {a:?}"),
        };
        println!("--- the gate: {client:?}");
        assert_eq!((client.user.as_str(), client.logon_session), (me.user.as_str(), me.logon_session));
        assert_eq!(client.integrity, me.integrity);
        assert!(!client.restricted && !client.app_container && client.identify_only);
    }
    discard(&paths).unwrap();
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_gate_that_dies_mid_request_refuses_it_and_the_restarted_gate_answers_the_next() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = r#"{"steps": [
        {"tool": "Write", "input": {"file_path": "/work/t8b/repo/first.txt", "content": "1\n"}},
        {"tool": "Write", "input": {"file_path": "/work/t8b/repo/second.txt", "content": "2\n"}},
        {"text": "Task 8: done."}]}"#;
    // The first decision ends the gate program from outside before answering (F39); every other is allowed.
    let killer = |paths: &RunPaths| -> Box<dyn Fn(&GateRequest) -> GateAnswer + Send + Sync> {
        let first = std::sync::atomic::AtomicBool::new(true);
        // The run's own copy of the gate (gate_for_run): "<run id>.<random>.exe".
        let name = format!("{}.*", paths.run_id());
        Box::new(move |r: &GateRequest| {
            if first.swap(false, Ordering::SeqCst) {
                let out = std::process::Command::new(r"C:\Windows\System32\taskkill.exe")
                    .args(["/F", "/IM", &name])
                    .output()
                    .unwrap();
                println!("--- taskkill /F /IM {name}: {}", String::from_utf8_lossy(&out.stdout).trim());
                thread::sleep(Duration::from_millis(500));
            }
            GateAnswer::Allow { updated_input: r.input.clone() }
        })
    };
    let (events, code, gate, paths, _) = live_run("t8b", 47921, script, Duration::from_secs(10), killer);
    let asked = settled(&gate, 2);
    let markers = [exists(&paths, "first.txt"), exists(&paths, "second.txt")];
    println!("--- first.txt, second.txt exist: {markers:?}; exit code {code:?}");
    assert_eq!(markers, [false, true], "the request the gate died on was refused; the next was answered");
    // The record is in the order Steplet answered, and the first answer came last (the gate was ended first), so
    // each decision is found by its request's id.
    let ids: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            Event::ToolUse { id, .. } => Some(id.as_str()),
            _ => None,
        })
        .collect();
    let by_id = |id: &str| {
        asked
            .iter()
            .find_map(|a| match a {
                Asked::Decision { client, request, delivered, .. } if request.tool_use_id == id => {
                    Some((client.pid, *delivered))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no decision for {id}: {asked:?}"))
    };
    let (first, second) = (by_id(ids[0]), by_id(ids[1]));
    println!("--- gate process id and delivered, first write: {first:?}; second: {second:?}");
    assert_eq!((first.1, second.1), (false, true), "the first answer never reached the gate");
    assert_ne!(first.0, second.0, "Claude Code started the gate again");
    let failed = events.iter().find_map(|e| match e {
        Event::ToolResult { is_error: true, text, .. } => Some(text.clone()),
        _ => None,
    });
    println!("--- what Claude Code said about the first: {failed:?}");
    assert!(failed.is_some());
    discard(&paths).unwrap();
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_decision_that_never_comes_is_refused_at_the_deadline_in_a_real_run() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = r#"{"steps": [
        {"tool": "Write", "input": {"file_path": "/work/t8c/repo/late.txt", "content": "late\n"}},
        {"text": "Task 8: done."}]}"#;
    let stall = |_: &RunPaths| -> Box<dyn Fn(&GateRequest) -> GateAnswer + Send + Sync> {
        Box::new(|_| {
            thread::sleep(Duration::from_secs(3600));
            GateAnswer::Deny { message: "not reached".into() }
        })
    };
    let started = Instant::now();
    let (events, code, gate, paths, _) = live_run("t8c", 47922, script, Duration::from_secs(3), stall);
    let asked = settled(&gate, 1);
    let total = started.elapsed();
    println!(
        "--- late.txt exists: {}; the run took {:.1} s (prepare included)",
        exists(&paths, "late.txt"),
        total.as_secs_f64()
    );
    assert!(!exists(&paths, "late.txt"));
    assert_eq!(code, Some(0), "the run went on and ended by itself");
    let decided = decisions(&asked);
    assert_eq!(decided.len(), 1, "{asked:?}");
    let (_, answer, took, delivered) = decided[0];
    println!("--- the decision: {answer:?} after {} ms, delivered {delivered}", took.as_millis());
    assert!(matches!(answer, GateAnswer::Deny { .. }) && delivered);
    assert!(took >= Duration::from_secs(3) && took < Duration::from_secs(4), "{took:?}");
    assert!(events.iter().any(|e| matches!(e, Event::Result { .. })));
    discard(&paths).unwrap();
}

/// Found while running Task 9's Linux checks (phase 4, 21 Oct 2026): Windows can come to refuse to start a program from
/// one exact `\wsl.localhost\steplet\...` path ("Error performing inpage operation"), whatever file is put there,
/// while the same bytes start from any new path. A gate path used by run after run can stop working for good, and
/// every run then fails closed. So each run starts the gate from a fresh copy of its own, removed with the run.
#[test]
#[ignore = "needs Steplet's Linux"]
fn each_run_starts_the_gate_from_a_path_of_its_own() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let installed = support::install_gate(&distro);
    let paths = RunPaths::new(&distro, "tgp1").unwrap();
    let a = gate_for_run(&paths, &installed).unwrap();
    let b = gate_for_run(&paths, &installed).unwrap();
    assert_ne!(a, b, "two runs never share a gate path");
    for p in [&a, &b] {
        assert!(p.starts_with("/opt/steplet/gate/runs/tgp1.") && p.ends_with(".exe"), "{p}");
        let facts = support::root_sh(
            &distro,
            &format!("stat -c '%U %G %a' {p}; sha256sum {p} {installed} | cut -d' ' -f1 | uniq | wc -l"),
        );
        assert_eq!(facts, "root root 755\n1", "{p}: owned by root, not writable by the agent, the same program");
        let started = support::root_sh(&distro, &format!("{p} --version 2>&1; echo \"exit $?\""));
        assert!(started.contains("unknown option") && started.ends_with("exit 2"), "{p} starts: {started}");
    }
    discard(&paths).unwrap();
    let left = support::root_sh(&distro, "ls /opt/steplet/gate/runs/ | grep -c '^tgp1[.]' || true");
    assert_eq!(left, "0", "the run's gate copies are removed with the run");
}
