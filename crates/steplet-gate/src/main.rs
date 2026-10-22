//! Steplet's gate (plan Task 8). Claude Code starts it inside Steplet's Linux, through WSL interop, as its
//! permission-prompt MCP server, speaking JSON-RPC lines on standard input and output. It holds no logic and no
//! state: each request goes to Steplet over the run's named pipe, on a connection of its own, and anything other than
//! a clear answer from Steplet in time is a refusal (F33, F36, F38).
//!
//! `steplet-gate --pipe <name> --token <run token> --log <Windows path> --deadline-ms <n> --steplet-pid <n>`
//!
//! `steplet-gate --key --pipe <name> --token <run token> --deadline-ms <n> --steplet-pid <n>` is Claude Code's
//! `apiKeyHelper` (H2): it asks Steplet for the key over the same pipe and prints it with no line ending, so the key is
//! never written to a file inside Linux. Anything unclear prints nothing and ends with exit code 1. Wrong options end
//! with exit code 2.
//!
//! `--steplet-pid` is the Windows process number of the Steplet that started the run. Another program of the same
//! Windows account can add its own end to Steplet's pipe (phase 3), so before sending anything on a connection the gate
//! asks Windows which program holds the other end, and talks only to that Steplet (phase 5, K5): a request carries the
//! run's token. Any other program there gets nothing, and the request is refused, so such a program can make a run's
//! requests fail (as any program of the account could, by ending Steplet), but cannot answer them.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use windows_sys::Win32::System::Pipes::GetNamedPipeServerProcessId;

const ERROR_FILE_NOT_FOUND: i32 = 2;
const ERROR_PIPE_BUSY: i32 = 231;
/// Opening the pipe with this lets Steplet see which Windows account the gate runs as, and nothing more: Steplet
/// cannot act as the gate.
const SECURITY_IDENTIFICATION: u32 = 0x0001_0000;
/// The gate waits this long past Steplet's deadline, so that Steplet's own refusal at its deadline normally comes
/// first; the gate's timer is for when Steplet's end of the pipe stops answering altogether.
const GRACE: Duration = Duration::from_secs(2);
/// Steplet's own longest deadline: an hour.
const LONGEST_DEADLINE_MS: u64 = 3_600_000;
/// The longest answer the gate reads. An allowed write carries the file's whole content back, and Steplet reads
/// requests of up to 16 MiB.
const LONGEST_ANSWER: u64 = 17 << 20;

/// The command line, checked.
struct Options {
    key: bool,
    pipe: String,
    token: String,
    log: Option<String>,
    deadline: Duration,
    /// The Steplet that started the run, as Windows numbers its processes.
    steplet: u32,
}

fn options(args: &[String]) -> Result<Options, String> {
    let mut key = false;
    let (mut pipe, mut token, mut log, mut deadline, mut steplet) = (None, None, None, None, None);
    let mut args = args.iter();
    while let Some(name) = args.next() {
        let slot = match name.as_str() {
            "--key" if key => return Err("--key is given twice".into()),
            "--key" => {
                key = true;
                continue;
            }
            "--pipe" => &mut pipe,
            "--token" => &mut token,
            "--log" => &mut log,
            "--deadline-ms" => &mut deadline,
            "--steplet-pid" => &mut steplet,
            other => return Err(format!("unknown option {other:?}")),
        };
        if slot.is_some() {
            return Err(format!("{name} is given twice"));
        }
        *slot = Some(args.next().ok_or_else(|| format!("{name} needs a value"))?.clone());
    }
    let pipe = pipe.ok_or("--pipe is missing")?;
    if !plain_word(&pipe) || pipe == "." || pipe == ".." {
        return Err("--pipe must be a plain name".into());
    }
    let token = token.ok_or("--token is missing")?;
    let deadline = deadline
        .ok_or("--deadline-ms is missing")?
        .parse::<u64>()
        .ok()
        .filter(|&ms| (1..=LONGEST_DEADLINE_MS).contains(&ms))
        .ok_or("--deadline-ms must be a whole number of milliseconds, from 1 to an hour")?;
    match (key, &log) {
        (false, None) => return Err("--log is missing".into()),
        (true, Some(_)) => return Err("--log is not used with --key".into()),
        _ => {}
    }
    let steplet = steplet
        .ok_or("--steplet-pid is missing")?
        .parse::<u32>()
        .ok()
        .filter(|&pid| pid != 0)
        .ok_or("--steplet-pid must be the process number of the Steplet that started the run")?;
    let pipe = format!(r"\\.\pipe\{pipe}");
    Ok(Options { key, pipe, token, log, deadline: Duration::from_millis(deadline), steplet })
}

/// Letters, digits, `.`, `_` and `-`, not starting with `-`: the same rule Steplet uses when it writes the name.
fn plain_word(s: &str) -> bool {
    (1..=128).contains(&s.len())
        && !s.starts_with('-')
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let o = match options(&args) {
        Ok(o) => o,
        Err(why) => {
            eprintln!("steplet-gate: {why}");
            std::process::exit(2);
        }
    };
    if o.key {
        std::process::exit(print_key(&o));
    }
    serve_claude_code(&o);
}

/// Asks Steplet once: one connection, one request line, one answer line, all within `limit`.
fn ask(o: &Options, request: &Value, limit: Duration) -> Result<Value, String> {
    let until = Instant::now() + limit;
    let (tx, rx) = mpsc::channel();
    let (pipe, line, steplet) = (o.pipe.clone(), format!("{request}\n"), o.steplet);
    // On a thread of its own, so a Steplet that never answers cannot hold the gate past `limit`. Such a thread stays
    // blocked until Steplet lets go of the connection or the gate ends.
    thread::spawn(move || {
        let _ = tx.send(exchange(&pipe, steplet, &line, until));
    });
    rx.recv_timeout(limit).unwrap_or_else(|_| Err("Steplet did not answer in time".into()))
}

fn exchange(pipe: &str, steplet: u32, line: &str, until: Instant) -> Result<Value, String> {
    let f = connect(pipe, until)?;
    // Nothing is sent before Windows has said the other end is held by the Steplet that started the run.
    match server_process(&f) {
        Ok(pid) if pid == steplet => {}
        Ok(pid) => {
            return Err(format!(
                "the program at Steplet's end is not the Steplet that started this run (process {pid}, not {steplet})"
            ));
        }
        Err(e) => return Err(format!("Windows would not say which program is at Steplet's end ({e})")),
    }
    (&f).write_all(line.as_bytes()).map_err(|e| format!("the request could not be sent ({e})"))?;
    let mut answer = Vec::new();
    BufReader::new(&f)
        .take(LONGEST_ANSWER)
        .read_until(b'\n', &mut answer)
        .map_err(|e| format!("Steplet's answer could not be read ({e})"))?;
    if answer.last() != Some(&b'\n') {
        return Err("Steplet closed the connection without a whole answer".into());
    }
    serde_json::from_slice(&answer).map_err(|_| "Steplet's answer could not be read".into())
}

/// Opens the run's pipe. Steplet always keeps a free end of the pipe, so "busy" lasts only moments.
fn connect(pipe: &str, until: Instant) -> Result<File, String> {
    loop {
        let opened = OpenOptions::new().read(true).write(true).security_qos_flags(SECURITY_IDENTIFICATION).open(pipe);
        match opened {
            Ok(f) => return Ok(f),
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) && Instant::now() < until => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) if e.raw_os_error() == Some(ERROR_FILE_NOT_FOUND) => {
                return Err("Steplet is not serving this run".into());
            }
            Err(e) => return Err(format!("Steplet could not be reached ({e})")),
        }
    }
}

/// The Windows process number of the program holding the other end of the connection `f`.
fn server_process(f: &File) -> std::io::Result<u32> {
    let mut pid = 0;
    // SAFETY: `f` is an open pipe handle for the whole call, and `pid` is a place for one number.
    if unsafe { GetNamedPipeServerProcessId(f.as_raw_handle(), &mut pid) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(pid)
}

/// Steplet's answer in the shape Claude Code reads: exactly `{behavior: "allow", updatedInput: {…}}` or
/// `{behavior: "deny", message: "…"}` (F38). Anything else is a refusal.
fn decision(answer: Result<Value, String>) -> (Value, Option<String>) {
    let unreadable = "Steplet's answer could not be read";
    let answer = match answer {
        Ok(a) => a,
        Err(why) => return (refusal(&why), Some(why)),
    };
    let Some(fields) = answer.as_object().filter(|f| f.len() == 2) else {
        return (refusal(unreadable), Some(unreadable.into()));
    };
    let input = fields.get("updatedInput").filter(|v| v.is_object());
    let message = fields.get("message").and_then(Value::as_str).filter(|m| !m.is_empty());
    match (fields.get("behavior").and_then(Value::as_str), input, message) {
        (Some("allow"), Some(input), _) => (json!({"behavior": "allow", "updatedInput": input}), None),
        (Some("deny"), _, Some(message)) => (json!({"behavior": "deny", "message": message}), None),
        _ => (refusal(unreadable), Some(unreadable.into())),
    }
}

fn refusal(why: &str) -> Value {
    json!({"behavior": "deny", "message": format!("Steplet refused this: {why}.")})
}

/// The MCP server, until Claude Code closes standard input.
fn serve_claude_code(o: &Options) {
    let log = Log(o.log.clone().unwrap_or_default());
    let mut names: Vec<String> = std::env::vars_os().map(|(k, _)| k.to_string_lossy().into_owned()).collect();
    names.sort();
    log.line(&format!("started; {} Windows variables: {}", names.len(), plain(&names.join(","))));
    let mut stdout = std::io::stdout();
    for line in BufReader::new(std::io::stdin().lock()).lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            log.line("a line on standard input was not JSON");
            continue;
        };
        let result = match msg["method"].as_str().unwrap_or("") {
            "initialize" => Ok(json!({
                "protocolVersion": msg["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "steplet-gate", "version": env!("CARGO_PKG_VERSION")}
            })),
            "tools/list" => Ok(json!({"tools": [{
                "name": "decide",
                "description": "Steplet's permission gate: Steplet decides whether the agent may take this action.",
                "inputSchema": {"type": "object", "properties": {
                    "tool_name": {"type": "string"}, "input": {"type": "object"}, "tool_use_id": {"type": "string"}
                }, "required": ["tool_name", "input"]}
            }]})),
            "tools/call" if msg["params"]["name"] == "decide" => Ok(decide(o, &msg["params"]["arguments"], &log)),
            "tools/call" => Err((-32602, "this server has only the tool decide")),
            "ping" => Ok(json!({})),
            _ => Err((-32601, "method not found")),
        };
        let Some(id) = msg.get("id").cloned() else { continue }; // notifications get no reply
        let reply = match result {
            Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
            Err((code, message)) => json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}),
        };
        if writeln!(stdout, "{reply}").and_then(|()| stdout.flush()).is_err() {
            log.line("standard output closed");
            break;
        }
    }
    log.line("standard input closed; ending");
}

/// One permission request, asked of Steplet; returns the MCP tool result.
fn decide(o: &Options, a: &Value, log: &Log) -> Value {
    let started = Instant::now();
    let request = json!({"kind": "decide", "run_token": o.token, "tool_name": a["tool_name"], "input": a["input"],
        "tool_use_id": a["tool_use_id"]});
    let (answer, own_refusal) = decision(ask(o, &request, o.deadline + GRACE));
    let name = |field: &str| plain(a[field].as_str().unwrap_or("?"));
    log.line(&format!(
        "tool={} id={} behavior={} ms={}",
        name("tool_name"),
        name("tool_use_id"),
        answer["behavior"].as_str().unwrap_or("?"),
        started.elapsed().as_millis()
    ));
    if let Some(why) = own_refusal {
        log.line(&format!("refused by the gate itself: {}", plain(&why)));
    }
    json!({"content": [{"type": "text", "text": answer.to_string()}]})
}

/// Claude Code's `apiKeyHelper`: prints the key alone, or nothing. Returns the exit code.
fn print_key(o: &Options) -> i32 {
    let answer = ask(o, &json!({"kind": "key", "run_token": o.token}), o.deadline + GRACE);
    let key = match &answer {
        Ok(a) => a.get("key").and_then(Value::as_str).filter(|k| !k.is_empty()),
        Err(_) => None,
    };
    let Some(key) = key else {
        let why = answer.err().unwrap_or_else(|| "Steplet did not hand over a key".into());
        eprintln!("steplet-gate: no key: {why}");
        return 1;
    };
    let mut out = std::io::stdout();
    if out.write_all(key.as_bytes()).and_then(|()| out.flush()).is_ok() { 0 } else { 1 }
}

/// The gate's log on Windows. It names each decision, never its input or the run's token.
struct Log(String);

impl Log {
    fn line(&self, text: &str) {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&self.0) {
            let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
            let _ = writeln!(f, "{}.{:03} pid={} {text}", t.as_secs(), t.subsec_millis(), std::process::id());
        }
    }
}

/// Text for one log line: anything that is not a printable character becomes `?`, so no value can start a line of
/// its own.
fn plain(s: &str) -> String {
    s.chars().map(|c| if c.is_control() { '?' } else { c }).collect()
}
