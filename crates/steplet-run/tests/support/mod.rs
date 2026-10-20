//! Test harness for the runner's integration tests (plan Task 7 onwards): Steplet's own Linux, the scripted model
//! (`fake_model.py`), and the gate (Task 8): the gate program built from this source, and the service. Test support
//! only.
#![allow(dead_code)] // each test file uses part of it

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use steplet_run::{ApiKey, GateAnswer, GateLink, GateRequest, GateService, gate_on_windows, serve, wsl};

pub fn steplet_linux() -> String {
    wsl::find_distribution("steplet").unwrap().expect("Steplet's Linux is installed")
}

/// Runs one program in Steplet's Linux; it must succeed. Returns its output, trimmed.
pub fn run(distro: &str, user: &str, argv: &[&str]) -> String {
    let out = wsl::run(distro, user, argv, b"").unwrap();
    assert_eq!(out.status, Some(0), "{argv:?}: {}{}", out.stderr, out.stdout);
    out.stdout.trim_end().to_owned()
}

/// Runs a short shell script as root. Like Steplet's own scripts, the body is one function, so no program it starts
/// can read the script from standard input.
pub fn root_sh(distro: &str, body: &str) -> String {
    let script = format!("main() {{\n{body}\n}}\nmain < /dev/null\n");
    let out = wsl::run(distro, "root", &["/bin/bash", "-s"], script.as_bytes()).unwrap();
    assert_eq!(out.status, Some(0), "{body}: {}", out.stderr);
    out.stdout.trim_end().to_owned()
}

/// Puts `bytes` at `path` in Steplet's Linux, owned by root, with the given mode.
pub fn install(distro: &str, bytes: &[u8], path: &str, mode: &str) {
    let argv = ["/usr/bin/install", "-D", "-o", "root", "-g", "root", "-m", mode, "/dev/stdin", path];
    let out = wsl::run(distro, "root", &argv, bytes).unwrap();
    assert_eq!(out.status, Some(0), "install {path}: {}", out.stderr);
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// The gate program, built from this worktree's source into a target folder of its own (so this build does not wait
/// on the one running the tests). Built once per test program. McAfee sometimes holds a freshly built program for a
/// moment (`LNK1104`), so the build is tried up to three times.
pub fn gate_program() -> PathBuf {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let target = repo_root().join("target").join("gate-for-tests");
            let mut said = String::new();
            for _ in 0..3 {
                let out = Command::new(env!("CARGO"))
                    .args(["build", "--release", "--offline", "-p", "steplet-gate", "--target-dir"])
                    .arg(&target)
                    .current_dir(repo_root())
                    .stdin(Stdio::null())
                    .output()
                    .unwrap();
                if out.status.success() {
                    return target.join("release").join("steplet-gate.exe");
                }
                said = String::from_utf8_lossy(&out.stderr).into_owned();
                std::thread::sleep(Duration::from_secs(2));
            }
            panic!("the gate program did not build:\n{said}")
        })
        .clone()
}

/// The file name the gate program has on Steplet's Linux disk (named after its contents).
pub fn gate_file_name() -> String {
    steplet_run::gate_file_name(&std::fs::read(gate_program()).unwrap())
}

/// Puts this worktree's gate program on Steplet's Linux disk, as Steplet does, and returns where.
pub fn install_gate(distro: &str) -> String {
    steplet_run::install_gate(distro, &std::fs::read(gate_program()).unwrap()).unwrap()
}

/// A made-up key the gate's key mode serves in tests; never a real one.
pub const MADE_UP_KEY: &str = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";

/// The gate for run `run_id`: the gate program on Steplet's Linux disk, and the service on Windows answering with
/// `decide` within `deadline` and serving the made-up key.
pub fn serve_gate(
    distro: &str,
    run_id: &str,
    deadline: Duration,
    decide: impl Fn(&GateRequest) -> GateAnswer + Send + Sync + 'static,
) -> (GateLink, GateService) {
    // Like a product run, each test run starts the gate from a fresh copy of its own (gate_for_run); the run's
    // `discard` removes it.
    let paths = steplet_run::RunPaths::new(distro, run_id).unwrap();
    let exe = steplet_run::gate_for_run(&paths, &install_gate(distro)).unwrap();
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    // At least 20 characters, so `count_key.py` can count it.
    let token = format!("{run_id}-token-{nanos:016x}");
    let link = GateLink::new(&exe, &format!("steplet-{run_id}"), &token, &gate_log(run_id), deadline).unwrap();
    let service =
        serve(&link, &gate_on_windows("steplet", &exe), Some(ApiKey::new(MADE_UP_KEY.into())), decide).unwrap();
    (link, service)
}

/// Answers every request with "allow", as the phase 0 stand-in host's `allow` mode did: for tests of the launch,
/// which are not about the checker.
pub fn allow_everything(r: &GateRequest) -> GateAnswer {
    GateAnswer::Allow { updated_input: r.input.clone() }
}

/// Steplet's logs folder on Windows (plan section 3).
pub fn logs_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("LOCALAPPDATA is set")).join("Steplet").join("logs")
}

pub fn gate_log(run_id: &str) -> String {
    logs_dir().join(format!("{run_id}.gate.log")).to_string_lossy().into_owned()
}

/// A helper program that runs until it is dropped.
pub struct Helper {
    child: Child,
    /// Run as root in Steplet's Linux when the helper is dropped, to end its Linux side.
    end_in_linux: Option<(String, Vec<String>)>,
}

impl Drop for Helper {
    fn drop(&mut self) {
        if let Some((distro, argv)) = &self.end_in_linux {
            let argv: Vec<&str> = argv.iter().map(String::as_str).collect();
            let _ = wsl::run(distro, "root", &argv, b"");
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The listener (`listener.py`), inside Steplet's Linux as root on `127.0.0.1:<port>`: it logs every connection that
/// reaches it in `log`. A program the task runs must never reach it.
pub fn listener(distro: &str, port: u16, log: &str) -> Helper {
    install(distro, include_bytes!("listener.py"), "/opt/steplet-test/listener.py", "644");
    run(distro, "root", &["/usr/bin/rm", "-f", log]);
    let port = port.to_string();
    let mut child =
        wsl::spawn(distro, "root", &["/usr/bin/python3", "-I", "/opt/steplet-test/listener.py", &port, log]).unwrap();
    drop(child.stdin.take());
    // It says when it listens; a test connection would be logged as one that reached it.
    let mut first = String::new();
    std::io::BufRead::read_line(&mut std::io::BufReader::new(child.stdout.take().unwrap()), &mut first).unwrap();
    assert!(first.starts_with("listening on"), "the listener did not start: {first:?}");
    Helper {
        child,
        end_in_linux: Some((distro.into(), vec!["/usr/bin/pkill".into(), "-f".into(), format!("listener.py {port} ")])),
    }
}

/// The scripted model, inside Steplet's Linux as root on `127.0.0.1:<port>`, following `script` (JSON).
pub fn scripted_model(distro: &str, name: &str, script: &str, port: u16) -> Helper {
    install(distro, include_bytes!("fake_model.py"), "/opt/steplet-test/fake_model.py", "644");
    let script_path = format!("/opt/steplet-test/{name}.script.json");
    install(distro, script.as_bytes(), &script_path, "644");
    let log = format!("/opt/steplet-test/{name}.model.log");
    run(distro, "root", &["/usr/bin/rm", "-f", &log]);
    // The model checks the key it is sent against the made-up key's SHA-256, so not even that key is on the disk.
    let key_hash: String = Sha256::digest(MADE_UP_KEY.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    let hash_path = "/opt/steplet-test/made-up-key.sha256";
    install(distro, key_hash.as_bytes(), hash_path, "644");
    let port = port.to_string();
    let mut child = wsl::spawn(
        distro,
        "root",
        &["/usr/bin/python3", "-I", "/opt/steplet-test/fake_model.py", &port, &script_path, &log, hash_path],
    )
    .unwrap();
    drop(child.stdin.take());
    let helper = Helper {
        child,
        end_in_linux: Some((
            distro.into(),
            vec!["/usr/bin/pkill".into(), "-f".into(), format!("fake_model.py {port} ")],
        )),
    };
    let probe = format!("import socket; socket.create_connection(('127.0.0.1', {port}), 1)");
    let start = Instant::now();
    while wsl::run(distro, "root", &["/usr/bin/python3", "-I", "-c", &probe], b"").unwrap().status != Some(0) {
        assert!(start.elapsed() < Duration::from_secs(20), "the scripted model did not start");
        std::thread::sleep(Duration::from_millis(300));
    }
    helper
}
