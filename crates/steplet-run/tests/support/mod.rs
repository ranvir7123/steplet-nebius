//! Test harness for the runner's integration tests (plan Task 7 onwards): Steplet's own Linux, the scripted model
//! (`fake_model.py`), and the phase 0 stand-in gate (`spikes/`), until Task 8's gate replaces it. Test support only.
//! The stand-ins are built with `cargo build --release --offline -p spike-gate -p spike-host` in `spikes/`.
#![allow(dead_code)] // each test file uses part of it

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use steplet_run::wsl;

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

fn stand_in(name: &str) -> PathBuf {
    let exe = repo_root().join("spikes").join("target").join("release").join(name);
    assert!(exe.exists(), "build the stand-ins first: {}", exe.display());
    exe
}

/// Puts this worktree's build of the stand-in gate on Steplet's Linux disk (Task 1: a root-owned copy, not a mounted
/// folder) and returns where. The file is named after its contents and never replaced in place: on 14 Oct 2026 the
/// program at `/opt/steplet/gate/spike-gate.exe` stopped starting through interop ("Invalid argument") while the
/// same bytes started from any other path, for a reason not found (results file, Task 7).
pub fn install_stand_in_gate(distro: &str) -> String {
    let bytes = std::fs::read(stand_in("spike-gate.exe")).unwrap();
    // FNV-1a: a name for this build, not a security check.
    let name = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
    let path = format!("/opt/steplet/gate/spike-gate-{name:016x}.exe");
    if wsl::run(distro, "root", &["/usr/bin/test", "-e", &path], b"").unwrap().status != Some(0) {
        install(distro, &bytes, &path, "755");
    }
    path
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

/// The scripted model, inside Steplet's Linux as root on `127.0.0.1:<port>`, following `script` (JSON).
pub fn scripted_model(distro: &str, name: &str, script: &str, port: u16) -> Helper {
    install(distro, include_bytes!("fake_model.py"), "/opt/steplet-test/fake_model.py", "644");
    let script_path = format!("/opt/steplet-test/{name}.script.json");
    install(distro, script.as_bytes(), &script_path, "644");
    let log = format!("/opt/steplet-test/{name}.model.log");
    run(distro, "root", &["/usr/bin/rm", "-f", &log]);
    let port = port.to_string();
    let mut child = wsl::spawn(
        distro,
        "root",
        &["/usr/bin/python3", "-I", "/opt/steplet-test/fake_model.py", &port, &script_path, &log],
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

/// The phase 0 stand-in host, serving the gate's pipe in `allow` mode with a made-up key (never a real one).
pub fn stand_in_host(pipe: &str, token: &str) -> Helper {
    let key = std::env::temp_dir().join("steplet-test-made-up-key.txt");
    std::fs::write(&key, "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY").unwrap();
    let mut child = Command::new(stand_in("spike-host.exe"))
        .args(["serve", "--pipe", pipe, "--token", token, "--gate-mode", "allow", "--seconds", "900"])
        .arg("--key-file")
        .arg(&key)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    // The host says when its pipe exists; the rest of what it prints is read and dropped, so it never blocks.
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    out.read_line(&mut first).unwrap();
    assert!(first.contains("created"), "the stand-in host did not create its pipe: {first}");
    std::thread::spawn(move || std::io::copy(&mut out, &mut std::io::sink()));
    Helper { child, end_in_linux: None }
}
