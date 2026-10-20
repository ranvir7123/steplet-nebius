//! Runs one program inside Steplet's Linux through `wsl.exe`. `wsl.exe` gets `SystemRoot` as its only Windows
//! variable (C8): with an empty environment it fails, and the whole environment would hand Windows variables over.
//! Both Windows programs are taken from the Windows folder itself, never looked up on `PATH`.

use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// What one program inside Steplet's Linux did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// The exit code; `None` when the program was ended by a signal.
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    /// Standard output exactly as it came, zero bytes included, for output that uses them as separators (`git status
    /// -z`). `stdout` drops them.
    pub stdout_bytes: Vec<u8>,
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
    finish(spawn(distro_id, user, argv)?, stdin, None)
}

/// As `run`, but gives up after `limit`: `wsl.exe` is then ended and the result is a `TimedOut` error. For calls that
/// must not hold Steplet for ever when its Linux does not answer. What the program started inside Linux may go on.
pub fn run_within(distro_id: &str, user: &str, argv: &[&str], stdin: &[u8], limit: Duration) -> io::Result<Output> {
    finish(spawn(distro_id, user, argv)?, stdin, Some(limit))
}

fn finish(mut child: Child, stdin: &[u8], limit: Option<Duration>) -> io::Result<Output> {
    let (Some(mut input), Some(stdout), Some(stderr)) = (child.stdin.take(), child.stdout.take(), child.stderr.take())
    else {
        let _ = child.kill();
        return Err(io::Error::other("the program's input and outputs were not connected"));
    };
    thread::scope(|s| {
        // Written from its own thread so a program that answers before reading all of it cannot stall both sides.
        // A program that stops reading early ends the write; its exit code tells what happened.
        s.spawn(move || {
            let _ = input.write_all(stdin);
        });
        let out = s.spawn(move || read_all(stdout));
        let err = s.spawn(move || read_all(stderr));
        let status = match limit {
            None => child.wait()?,
            Some(limit) => {
                let start = Instant::now();
                loop {
                    if let Some(status) = child.try_wait()? {
                        break status;
                    }
                    if start.elapsed() >= limit {
                        // Ending `wsl.exe` closes its outputs, which ends the readers.
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            format!("Steplet's Linux did not answer within {} s", limit.as_secs_f64()),
                        ));
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            }
        };
        let (out, err) = (out.join().unwrap_or_default(), err.join().unwrap_or_default());
        Ok(Output { status: status.code(), stdout: text(&out), stderr: text(&err), stdout_bytes: out })
    })
}

fn read_all(mut from: impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    let _ = from.read_to_end(&mut bytes);
    bytes
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

/// A Windows program from the Windows folder itself, with `SystemRoot` as its only variable (C8).
pub(crate) fn windows_program(name: &str) -> io::Result<Command> {
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
