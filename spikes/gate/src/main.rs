//! Spike stand-in for steplet-gate.exe (milestone 1, phase 0). Throwaway.
//! Claude Code starts it through WSL interop as its permission-prompt MCP server, over standard input and output.
//! It keeps nothing between requests: each decision is asked of the host over the named pipe,
//! and anything other than a clear answer from the host becomes "deny".
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

const ERROR_PIPE_BUSY: i32 = 231;

fn stamp() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}.{:03}", d.as_secs(), d.subsec_millis())
}

fn ask_host(pipe: &str, token: &str, a: &Value) -> Value {
    let deny = |why: &str| json!({"behavior": "deny", "message": format!("Steplet's checker refused: {why}")});
    let request = json!({"run_token": token, "tool_name": a["tool_name"], "input": a["input"], "tool_use_id": a["tool_use_id"]});
    let mut conn = None;
    for _ in 0..40 {
        match std::fs::OpenOptions::new().read(true).write(true).open(pipe) {
            Ok(f) => {
                conn = Some(f);
                break;
            }
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => std::thread::sleep(Duration::from_millis(25)),
            Err(e) => return deny(&format!("cannot reach Steplet ({e})")),
        }
    }
    let Some(mut f) = conn else { return deny("Steplet is busy") };
    if writeln!(f, "{request}").is_err() {
        return deny("broken pipe");
    }
    let mut line = String::new();
    match BufReader::new(&f).read_line(&mut line) {
        Ok(n) if n > 0 => serde_json::from_str::<Value>(&line)
            .ok()
            .filter(|v| v["behavior"] == "allow" || v["behavior"] == "deny")
            .unwrap_or_else(|| deny("unreadable answer")),
        _ => deny("no answer"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let opt = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let pipe = format!(r"\\.\pipe\{}", opt("--pipe").expect("--pipe"));
    let token = opt("--token").expect("--token");
    let log_path = opt("--log").expect("--log");
    let crash_marker = format!("{log_path}.crashed");
    let crash_once = args.iter().any(|a| a == "--crash-once");
    let log = |line: &str| {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            let _ = writeln!(f, "{} pid={} {line}", stamp(), std::process::id());
        }
    };
    let mut names: Vec<String> = std::env::vars_os().map(|(k, _)| k.to_string_lossy().into_owned()).collect();
    names.sort();
    log(&format!("started; {} environment variables: {}", names.len(), names.join(",")));

    let mut stdout = std::io::stdout();
    for line in BufReader::new(std::io::stdin().lock()).lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            log("a line on standard input was not JSON");
            continue;
        };
        let result = match msg["method"].as_str().unwrap_or("") {
            "initialize" => Some(json!({
                "protocolVersion": msg["params"]["protocolVersion"],
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "steplet-spike-gate", "version": "0.0.0"}
            })),
            "tools/list" => Some(json!({"tools": [{
                "name": "decide",
                "description": "Steplet spike permission gate",
                "inputSchema": {"type": "object", "properties": {
                    "tool_name": {"type": "string"}, "input": {"type": "object"}, "tool_use_id": {"type": "string"}
                }, "required": ["tool_name", "input"]}
            }]})),
            "tools/call" => {
                if crash_once && std::fs::OpenOptions::new().write(true).create_new(true).open(&crash_marker).is_ok() {
                    log("crashing on purpose (first decision only)");
                    std::process::exit(3);
                }
                let a = &msg["params"]["arguments"];
                let started = Instant::now();
                let answer = ask_host(&pipe, &token, a);
                let input: String = a["input"].to_string().chars().take(2000).collect();
                log(&format!("tool={} behavior={} ms={} input={input}", a["tool_name"], answer["behavior"], started.elapsed().as_millis()));
                Some(json!({"content": [{"type": "text", "text": answer.to_string()}]}))
            }
            "ping" => Some(json!({})),
            _ => None,
        };
        let Some(id) = msg.get("id").cloned() else { continue }; // notifications get no reply
        let reply = match result {
            Some(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
            None => json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": "method not found"}}),
        };
        if writeln!(stdout, "{reply}").and_then(|()| stdout.flush()).is_err() {
            log("standard output closed");
            break;
        }
    }
    log("standard input closed; exiting");
}
