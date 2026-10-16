//! Runs one program inside Steplet's Linux through `wsl.exe`. `wsl.exe` gets `SystemRoot` as its only Windows
//! variable (C8): with an empty environment it fails, and the whole environment would hand Windows variables over.
//! Both Windows programs are taken from the Windows folder itself, never looked up on `PATH`.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// What one program inside Steplet's Linux did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The exit code; `None` when the program was ended by a signal.
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Starts `argv` as `user` in the distribution `distro_id`, in `/`, with its input and both outputs piped.
pub fn spawn(distro_id: &str, user: &str, argv: &[&str]) -> io::Result<Child> {
    windows_program("wsl.exe")?
        .args(["--distribution-id", distro_id, "-u", user, "--cd", "/", "--exec"])
        .args(argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

/// Runs `argv` as `user` in the distribution `distro_id`, starting in `/`, with `stdin` as its whole input.
pub fn run(distro_id: &str, user: &str, argv: &[&str], stdin: &[u8]) -> io::Result<Output> {
    let mut child = spawn(distro_id, user, argv)?;
    let mut input = child.stdin.take().expect("standard input is piped");
    let output = std::thread::scope(|s| {
        // Written from its own thread so a program that answers before reading all of it cannot stall both sides.
        // A program that stops reading early ends the write; its exit code tells what happened.
        s.spawn(move || {
            let _ = input.write_all(stdin);
        });
        child.wait_with_output()
    })?;
    Ok(Output { status: output.status.code(), stdout: text(&output.stdout), stderr: text(&output.stderr) })
}

/// The id WSL's registry gives the distribution called `name` for this Windows account, if it is installed.
pub fn find_distribution(name: &str) -> io::Result<Option<String>> {
    let lxss = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Lxss";
    let out = windows_program("reg.exe")?.args(["query", lxss, "/s", "/v", "DistributionName"]).output()?;
    // `reg` prints each key, then its values: `    DistributionName    REG_SZ    steplet`.
    let mut key = "";
    for line in text(&out.stdout).lines() {
        if line.starts_with("HKEY_") {
            key = line.rsplit('\\').next().unwrap_or("");
        } else if line.split_whitespace().eq(["DistributionName", "REG_SZ", name]) {
            return Ok(Some(key.to_owned()));
        }
    }
    Ok(None)
}

fn windows_program(name: &str) -> io::Result<Command> {
    let root = std::env::var_os("SystemRoot")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Windows did not say where its own folder is"))?;
    let mut command = Command::new(PathBuf::from(&root).join("System32").join(name));
    command.env_clear().env("SystemRoot", root);
    Ok(command)
}

/// Program output as text. `wsl.exe`'s own messages come in UTF-16, so the zero bytes between their letters are
/// dropped; any byte that is still not UTF-8 becomes the replacement character.
pub(crate) fn text(bytes: &[u8]) -> String {
    let kept: Vec<u8> = bytes.iter().copied().filter(|&b| b != 0).collect();
    String::from_utf8_lossy(&kept).into_owned()
}
