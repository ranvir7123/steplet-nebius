//! Setting up Steplet's own Linux (decision E3, option D; phase 4 Task 6). `look` reads what is there and changes
//! nothing; `install` makes Steplet's Linux from nothing (H3: Ubuntu 26.04, by name), sets it up as root with the
//! fixed script `setup.sh`, restarts it so WSL's settings for it apply, and puts this build's gate program on its
//! disk (H8); `remove` takes Steplet's Linux away, and nothing of the receiver's is in it. Nothing here touches the
//! receiver's own Linux or asks for a password.

use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::gate::{gate_file_name, install_gate};
use crate::wsl;

/// The name Steplet gives its own Linux.
pub const LINUX: &str = "steplet";
/// The Claude Code and srt versions this Steplet was tested with, installed exactly.
pub const CLAUDE_VERSION: &str = "2.1.220";
pub const SRT_VERSION: &str = "0.0.77";
/// The Linux image Steplet's Linux is made from (H3).
const IMAGE: &str = "Ubuntu-26.04";
/// The oldest WSL Steplet takes: the version it was tested with (`--name`, `--location` and the layout of phase 3).
const OLDEST_WSL: [u32; 4] = [2, 6, 3, 0];
const SETUP: &str = include_str!("setup.sh");
const LOOK: &str = include_str!("look.sh");
/// The Windows variables setup's own calls of `wsl.exe` get besides `SystemRoot`: where Windows' own folders and this
/// user's folders are. Without the user's folders `wsl.exe --install` stops with "Access is denied"
/// (`Wsl/InstallDistro/E_ACCESSDENIED`), and without `SystemDrive` it wrote its cache into a new folder named
/// `%SystemDrive%` inside the folder it was started from (both measured, 18 Oct 2026). None of them is a secret, and
/// none reaches Steplet's Linux: these calls start nothing inside it (C8 is about the commands run there).
const WINDOWS_FOLDERS: [&str; 8] =
    ["SystemDrive", "ProgramData", "ALLUSERSPROFILE", "APPDATA", "LOCALAPPDATA", "USERPROFILE", "TEMP", "TMP"];
/// The longest `look` waits for its look inside Steplet's Linux (which may first have to start).
const LOOK_LIMIT: Duration = Duration::from_secs(60);

/// Windows' Linux feature, as `wsl.exe --version` says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Wsl {
    /// Not installed, or only the old built-in `wsl.exe` that does not know `--version`.
    Missing,
    /// Older than the version Steplet was tested with.
    TooOld {
        version: String,
    },
    Ready {
        version: String,
    },
}

/// What is set up on this computer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub wsl: Wsl,
    /// The id of Steplet's Linux, if it is installed.
    pub linux: Option<String>,
    /// What Steplet's Linux lacks (each a short name: `git`, `claude-code`, `wsl.conf`, …), or why Steplet could not
    /// look. Empty when it lacks nothing, or is not installed.
    pub missing: Vec<String>,
    /// This build's gate program is on its disk.
    pub gate: bool,
}

#[derive(Debug)]
pub enum SetupError {
    /// A name Steplet will not hand to `wsl.exe`.
    Refused(&'static str),
    /// A step failed; `detail` is the end of what it printed.
    Step { step: &'static str, detail: String },
}

/// The version in `wsl.exe --version`'s output, as UTF-8 or as the UTF-16 `wsl.exe` writes. The first line names
/// WSL's own version in Windows' language; its number is what counts.
pub fn read_wsl_version(output: &[u8]) -> Wsl {
    let text = wsl::text(output);
    let Some(first) = text.lines().next().filter(|l| l.contains("WSL")) else { return Wsl::Missing };
    let number = first.split(|c: char| !(c.is_ascii_digit() || c == '.')).find_map(|word| {
        let parts: Vec<u32> = word.split('.').map(str::parse).collect::<Result<_, _>>().ok()?;
        (parts.len() >= 3).then_some((word.to_owned(), parts))
    });
    let Some((version, mut parts)) = number else { return Wsl::Missing };
    parts.resize(4, 0);
    if parts[..] < OLDEST_WSL[..] { Wsl::TooOld { version } } else { Wsl::Ready { version } }
}

/// What is set up for Steplet's Linux called `name`, with `gate_program` as this build's gate. Changes nothing.
pub fn look(name: &str, gate_program: &[u8]) -> State {
    let wsl_state = setup_program("wsl.exe")
        .and_then(|mut c| c.arg("--version").stdin(Stdio::null()).output())
        .map(|out| read_wsl_version(&out.stdout))
        .unwrap_or(Wsl::Missing);
    let mut state = State { wsl: wsl_state, linux: None, missing: Vec::new(), gate: false };
    if !matches!(state.wsl, Wsl::Ready { .. }) {
        return state;
    }
    state.linux = wsl::find_distribution(name).ok().flatten();
    let Some(id) = state.linux.clone() else { return state };
    let gate = gate_file_name(gate_program);
    let argv = ["/bin/bash", "-s", "--", CLAUDE_VERSION, SRT_VERSION, &gate];
    match wsl::run_within(&id, "root", &argv, LOOK.as_bytes(), LOOK_LIMIT) {
        Ok(out) if out.status == Some(0) => {
            for line in out.stdout.lines() {
                match line.split(' ').collect::<Vec<_>>()[..] {
                    ["ok", "gate"] => state.gate = true,
                    ["missing", "gate"] => {}
                    ["missing", what] => state.missing.push(what.into()),
                    _ => {}
                }
            }
        }
        Ok(out) => state.missing.push(format!("could not look: {}", last_lines(&out.stderr))),
        Err(e) => state.missing.push(format!("could not look: {e}")),
    }
    state
}

/// Makes Steplet's Linux called `name` from nothing, with its disk in `location`, and returns its id. Each line of
/// progress goes to `line`. Safe to run again on a Linux whose setup failed half way: every step of `setup.sh` can be
/// repeated.
pub fn install(name: &str, location: &Path, gate_program: &[u8], line: &dyn Fn(&str)) -> Result<String, SetupError> {
    let args = install_args(name, location)?;
    if wsl::find_distribution(name).ok().flatten().is_none() {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        windows("install Steplet's Linux", &args, line)?;
    }
    let id = wsl::find_distribution(name).ok().flatten().ok_or(SetupError::Step {
        step: "find Steplet's Linux",
        detail: format!("WSL lists no Linux called {name}"),
    })?;
    let setup = wsl::spawn(&id, "root", &["/bin/bash", "-s", "--", CLAUDE_VERSION, SRT_VERSION])
        .map_err(|e| SetupError::Step { step: "start the setup", detail: e.to_string() })?;
    stream("set up Steplet's Linux", setup, SETUP.as_bytes(), line)?;
    // WSL reads its settings for a Linux (`/etc/wsl.conf`) when the Linux starts.
    windows("restart Steplet's Linux", &["--terminate", name], line)?;
    line("--- the gate program of this build");
    let at = install_gate(&id, gate_program)
        .map_err(|e| SetupError::Step { step: "put the gate program on its disk", detail: format!("{e:?}") })?;
    line(&format!("gate: {at}"));
    Ok(id)
}

/// Takes Steplet's Linux called `name` away, with everything in it.
pub fn remove(name: &str) -> Result<(), SetupError> {
    if !plain_name(name) {
        return Err(SetupError::Refused("Linux name"));
    }
    windows("remove Steplet's Linux", &["--unregister", name], &|_| {})
}

/// Runs `wsl.exe args` as setup starts it (`setup_program`), sending each line of its output to `line`.
fn windows(step: &'static str, args: &[&str], line: &dyn Fn(&str)) -> Result<(), SetupError> {
    let fail = |detail: String| SetupError::Step { step, detail };
    let mut command = setup_program("wsl.exe").map_err(|e| fail(e.to_string()))?;
    // Its own messages in UTF-8, not UTF-16.
    command.env("WSL_UTF8", "1").args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = command.spawn().map_err(|e| fail(e.to_string()))?;
    stream(step, child, b"", line)
}

/// Feeds `input` to `child`, sends each line it prints to `line`, and fails with the end of what it said unless it
/// ends with exit code 0.
fn stream(
    step: &'static str,
    mut child: std::process::Child,
    input: &[u8],
    line: &dyn Fn(&str),
) -> Result<(), SetupError> {
    let fail = |detail: String| SetupError::Step { step, detail };
    let (stdout, stderr) = (child.stdout.take(), child.stderr.take());
    let input = input.to_vec();
    let writer = child.stdin.take().map(|mut stdin| {
        thread::spawn(move || {
            let _ = io::Write::write_all(&mut stdin, &input);
        })
    });
    let errors = stderr.map(|mut e| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = e.read_to_end(&mut bytes);
            wsl::text(&bytes)
        })
    });
    let mut said = Vec::new();
    if let Some(out) = stdout {
        let mut out = BufReader::new(out);
        let mut buf = Vec::new();
        while matches!(out.read_until(b'\n', &mut buf), Ok(n) if n > 0) {
            let text = wsl::text(&buf);
            let text = text.trim_end_matches(['\r', '\n']);
            line(text);
            said.push(text.to_owned());
            buf.clear();
        }
    }
    let status = child.wait().map_err(|e| fail(e.to_string()))?;
    if let Some(w) = writer {
        let _ = w.join();
    }
    let errors = errors.and_then(|e| e.join().ok()).unwrap_or_default();
    if status.success() {
        return Ok(());
    }
    let mut all = said.join("\n");
    all.push('\n');
    all.push_str(&errors);
    Err(fail(last_lines(&all)))
}

/// The last 15 lines of `text`.
fn last_lines(text: &str) -> String {
    let lines: Vec<&str> = text.trim_end().lines().collect();
    lines[lines.len().saturating_sub(15)..].join("\n")
}

/// `program` from Windows' own folder, the way setup starts it: `SystemRoot` and `WINDOWS_FOLDERS` as its only
/// variables, started in Windows' System32 folder. This user cannot write there, so nothing the program writes by a
/// relative path can land among the receiver's own files.
fn setup_program(program: &str) -> io::Result<Command> {
    let mut command = wsl::windows_program(program)?;
    for name in WINDOWS_FOLDERS {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    let system32 = Path::new(command.get_program()).parent().map(Path::to_path_buf);
    if let Some(folder) = system32 {
        command.current_dir(folder);
    }
    Ok(command)
}

/// The arguments of `wsl.exe --install` for Steplet's Linux called `name`, with its disk in `location`. Refused: a
/// name `wsl.exe` could read as an option, and a place that is not a full path (`wsl.exe` would take it from the
/// folder it starts in).
fn install_args(name: &str, location: &Path) -> Result<Vec<String>, SetupError> {
    if !plain_name(name) {
        return Err(SetupError::Refused("Linux name"));
    }
    if !location.is_absolute() {
        return Err(SetupError::Refused("install location"));
    }
    let location = location.to_string_lossy().into_owned();
    Ok(["--install", IMAGE, "--name", name, "--location", &location, "--no-launch"].map(str::to_owned).to_vec())
}

/// 1 to 64 lower-case letters, digits and `-`, not starting with `-`: a name `wsl.exe` cannot read as an option.
fn plain_name(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && !name.starts_with('-')
        && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(test)]
mod tests {
    use std::os::windows::process::CommandExt;

    use super::*;

    /// A Windows program started the way setup starts `wsl.exe` knows where Windows' own folders and this user's
    /// folders are, gets none of Steplet's other variables, and starts in Windows' System32 folder, never in the folder
    /// Steplet was started from. Without `SystemDrive`, `wsl.exe --install` wrote its cache into a new folder named
    /// `%SystemDrive%` inside the folder it was started from (measured, 18 Oct 2026).
    #[test]
    fn setup_s_windows_programs_know_windows_folders_and_start_in_system32() {
        let var = |name: &str| std::env::var(name).unwrap();
        let out = setup_program("cmd.exe")
            .unwrap()
            .raw_arg("/d /c echo [%SystemDrive%] [%ProgramData%] [%LOCALAPPDATA%] [%TEMP%] [%CD%] [%USERNAME%]")
            .output()
            .unwrap();
        let expected = format!(
            "[{}] [{}] [{}] [{}] [{}\\System32] [%USERNAME%]",
            var("SystemDrive"),
            var("ProgramData"),
            var("LOCALAPPDATA"),
            var("TEMP"),
            var("SystemRoot")
        );
        // Windows' folder names ignore case: `SystemRoot` says `C:\WINDOWS`, the folder itself is `C:\Windows`.
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim_end().to_lowercase(), expected.to_lowercase());
    }

    #[test]
    fn the_install_command_is_exact_and_refuses_what_wsl_could_misread() {
        let place = Path::new(r"C:\Users\made-up\AppData\Local\Steplet\wsl");
        assert_eq!(
            install_args("steplet", place).unwrap(),
            [
                "--install",
                "Ubuntu-26.04",
                "--name",
                "steplet",
                "--location",
                r"C:\Users\made-up\AppData\Local\Steplet\wsl",
                "--no-launch"
            ]
        );
        for name in ["", "-steplet", "Steplet", "step let", "steplet;x", &"a".repeat(65)] {
            assert!(matches!(install_args(name, place), Err(SetupError::Refused("Linux name"))), "{name:?}");
        }
        // Not a full path: `wsl.exe` would take it from the folder it starts in.
        for relative in ["wsl", r"Steplet\wsl", r"\Steplet\wsl", "C:wsl"] {
            let refused = install_args("steplet", Path::new(relative));
            assert!(matches!(refused, Err(SetupError::Refused("install location"))), "{relative:?}");
        }
    }
}
