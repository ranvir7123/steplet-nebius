//! Spike stand-in for the Steplet app (milestone 1, phase 0). Throwaway.
//! `launch` starts one run through wsl.exe the way the app will, and answers the gate over a named pipe.
//! `serve` runs the pipe alone, for the gate's self-test.
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::process::Command;

const WSL: &str = r"C:\Windows\System32\wsl.exe";
const LOGS: &str = r"C:\dev\m1\logs";

fn opt(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn req(args: &[String], name: &str) -> String {
    opt(args, name).unwrap_or_else(|| panic!("missing {name}"))
}

fn secs(args: &[String], name: &str) -> Option<Duration> {
    opt(args, name).map(|v| Duration::from_secs(v.parse().expect("whole seconds")))
}

fn stamp() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}.{:03}", d.as_secs(), d.subsec_millis())
}

fn append(path: &Path, line: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{} {line}", stamp());
    }
}

#[derive(Clone, Copy, Debug)]
enum EnvMode {
    Empty,
    SystemRoot,
    Inherit,
}

/// wsl.exe with the Windows environment the run asks for. The app's choice is Empty (milestone plan, section 3).
fn wsl_command(mode: EnvMode) -> Command {
    let mut c = Command::new(WSL);
    match mode {
        EnvMode::Inherit => {}
        EnvMode::Empty => {
            c.env_clear();
        }
        EnvMode::SystemRoot => {
            c.env_clear().env("SystemRoot", r"C:\Windows");
        }
    }
    c
}

struct Gate {
    mode: String,
    token: String,
    deadline: Duration,
    log: PathBuf,
}

fn gate_from(args: &[String], id: &str) -> Gate {
    Gate {
        mode: opt(args, "--gate-mode").unwrap_or_else(|| "allow".into()),
        token: req(args, "--token"),
        deadline: Duration::from_millis(opt(args, "--deadline-ms").map(|v| v.parse().expect("milliseconds")).unwrap_or(10_000)),
        log: Path::new(LOGS).join(format!("{id}.host-gate.log")),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = match args.get(1).map(String::as_str) {
        Some("launch") => launch(&args).await,
        Some("serve") => serve(&args).await,
        _ => {
            eprintln!("usage: spike-host launch|serve [options]");
            2
        }
    };
    std::process::exit(code);
}

async fn serve(args: &[String]) -> i32 {
    if let Err(e) = start_pipe(&req(args, "--pipe"), gate_from(args, "selftest")) {
        println!("host: could not create the pipe: {e}");
        return 1;
    }
    tokio::time::sleep(secs(args, "--seconds").unwrap_or(Duration::from_secs(10))).await;
    0
}

fn start_pipe(pipe: &str, gate: Gate) -> std::io::Result<()> {
    let name = format!(r"\\.\pipe\{pipe}");
    // first_pipe_instance: fail if anyone else already holds this name.
    let first = ServerOptions::new().first_pipe_instance(true).reject_remote_clients(true).create(&name)?;
    println!("host: pipe {name} created, gate mode {}", gate.mode);
    tokio::spawn(accept_loop(first, name, Arc::new(gate)));
    Ok(())
}

async fn accept_loop(mut server: NamedPipeServer, name: String, gate: Arc<Gate>) {
    let mut first = true;
    loop {
        if let Err(e) = server.connect().await {
            append(&gate.log, &format!("connect failed: {e}"));
            return;
        }
        let conn = server;
        server = match ServerOptions::new().reject_remote_clients(true).create(&name) {
            Ok(s) => s,
            Err(e) => {
                append(&gate.log, &format!("next pipe instance failed: {e}"));
                return;
            }
        };
        if first {
            first = false;
            tokio::spawn(record_gate_identity(gate.log.clone()));
        }
        tokio::spawn(answer(conn, gate.clone()));
    }
}

/// Which Windows account and session the gate runs under, as the app would see it.
async fn record_gate_identity(log: PathBuf) {
    match Command::new("tasklist").args(["/v", "/fo", "csv", "/fi", "imagename eq spike-gate.exe"]).output().await {
        Ok(o) => append(&log, &format!("tasklist:\n{}", String::from_utf8_lossy(&o.stdout))),
        Err(e) => append(&log, &format!("tasklist failed: {e}")),
    }
}

async fn answer(conn: NamedPipeServer, g: Arc<Gate>) {
    let started = Instant::now();
    let (read_half, mut write_half) = tokio::io::split(conn);
    let mut line = String::new();
    match BufReader::new(read_half).read_line(&mut line).await {
        Ok(n) if n > 0 => {}
        other => {
            append(&g.log, &format!("no request read: {other:?}"));
            return;
        }
    }
    let request: Value = match serde_json::from_str(&line) {
        Ok(v) => v,
        Err(e) => {
            append(&g.log, &format!("request is not JSON: {e}"));
            return;
        }
    };
    let deny = |why: &str| json!({"behavior": "deny", "message": format!("Steplet spike host: {why}")});
    let token_ok = request["run_token"].as_str() == Some(g.token.as_str());
    let reply = if !token_ok {
        deny("wrong run token")
    } else {
        match g.mode.as_str() {
            "allow" => json!({"behavior": "allow", "updatedInput": request["input"]}),
            "deny" => deny("denied by the spike"),
            "drop" => {
                append(&g.log, &format!("tool={} mode=drop: closed without answering", request["tool_name"]));
                return;
            }
            "stall" => {
                // The app's own deadline (F36): no decision in time means refuse.
                let _ = tokio::time::timeout(g.deadline, std::future::pending::<()>()).await;
                deny("no decision before the deadline")
            }
            other => deny(&format!("unknown mode {other}")),
        }
    };
    let _ = write_half.write_all(format!("{reply}\n").as_bytes()).await;
    let _ = write_half.flush().await;
    append(
        &g.log,
        &format!("tool={} token_ok={token_ok} mode={} behavior={} ms={}", request["tool_name"], g.mode, reply["behavior"], started.elapsed().as_millis()),
    );
}

async fn pkill(distro: &str, mode: EnvMode, signal: &str, marker: &str) {
    let out = wsl_command(mode).args(["--distribution-id", distro, "--exec", "/usr/bin/pkill", signal, "-f", marker]).output().await;
    println!("host: pkill {signal} -f {marker}: {:?}", out.map(|o| o.status.code()));
}

async fn launch(args: &[String]) -> i32 {
    let id = req(args, "--id");
    let run_dir = PathBuf::from(req(args, "--run-dir"));
    let distro = req(args, "--distro-id");
    let mode = match opt(args, "--env-mode").as_deref() {
        Some("inherit") => EnvMode::Inherit,
        Some("systemroot") => EnvMode::SystemRoot,
        _ => EnvMode::Empty,
    };
    let argv: Vec<String> = std::fs::read_to_string(run_dir.join("argv.txt"))
        .expect("argv.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    let task = std::fs::read(run_dir.join("task.txt")).expect("task.txt");

    if let Some(pipe) = opt(args, "--pipe") {
        if let Err(e) = start_pipe(&pipe, gate_from(args, &id)) {
            println!("host: could not create the gate pipe: {e}");
            return 1;
        }
    }

    let logs = Path::new(LOGS);
    let mut cmd = wsl_command(mode);
    cmd.arg("--distribution-id").arg(&distro).arg("--cd").arg(req(args, "--cd")).arg("--exec").args(&argv);
    cmd.stdin(Stdio::piped())
        .stdout(std::fs::File::create(logs.join(format!("{id}.jsonl"))).expect("stream file"))
        .stderr(std::fs::File::create(logs.join(format!("{id}.stderr.txt"))).expect("stderr file"));
    let started = Instant::now();
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            println!("host: wsl.exe did not start: {e}");
            return 1;
        }
    };
    println!("host: wsl.exe started (pid {:?}, environment {mode:?}); task sent on its input, {} bytes", child.id(), task.len());
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&task).await;
        // stdin is dropped here, which closes it: Claude Code reads the task and nothing more.
    }

    let marker = format!("runs/{id}/settings.json");
    let stop_after = secs(args, "--stop-after-s");
    let kill_claude_after = secs(args, "--kill-claude-after-s");
    let kill_wsl_after = secs(args, "--kill-wsl-after-s");
    let limit = secs(args, "--timeout-s").unwrap_or(Duration::from_secs(900));
    let (mut stopped, mut killed, mut wsl_killed) = (false, false, false);
    let status = loop {
        tokio::select! {
            s = child.wait() => break s,
            _ = tokio::time::sleep(Duration::from_millis(250)) => {
                let t = started.elapsed();
                if stop_after.is_some_and(|d| !stopped && t >= d) {
                    stopped = true;
                    println!("host: stop signal at {:.1} s", t.as_secs_f64());
                    pkill(&distro, mode, "-TERM", &marker).await;
                }
                if kill_claude_after.is_some_and(|d| !killed && t >= d) {
                    killed = true;
                    println!("host: hard kill of Claude Code at {:.1} s", t.as_secs_f64());
                    pkill(&distro, mode, "-KILL", &marker).await;
                }
                if !wsl_killed && (kill_wsl_after.is_some_and(|d| t >= d) || t >= limit) {
                    wsl_killed = true;
                    println!("host: killing wsl.exe at {:.1} s", t.as_secs_f64());
                    let _ = child.start_kill();
                }
            }
        }
    };
    println!("host: wsl.exe ended: {:?}, after {:.1} s", status.map(|s| s.code()), started.elapsed().as_secs_f64());
    0
}
