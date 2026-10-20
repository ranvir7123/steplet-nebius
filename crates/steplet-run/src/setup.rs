//! Setting up Steplet's own Linux (decision E3, option D; phase 4 Task 6). `look` reads what is there and changes
//! nothing; `install` makes Steplet's Linux from nothing (H3: Ubuntu 26.04, by name), sets it up as root with the
//! fixed script `setup.sh`, restarts it so WSL's settings for it apply, and puts this build's gate program on its
//! disk (H8); `remove` takes Steplet's Linux away, and nothing of the receiver's is in it. Each takes only Steplet's
//! own Linux names, so nothing here touches the receiver's own Linux; nothing asks for a password.

use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

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
/// The longest each step of `install` and `remove` may take before Steplet ends it: the download (1 to 2 GB, 172 s
/// here, much longer on a slow line), the setup inside (packages and npm: about 2 minutes here), a restart, and the
/// removal of a whole Linux disk.
const INSTALL_LIMIT: Duration = Duration::from_secs(90 * 60);
const SETUP_LIMIT: Duration = Duration::from_secs(30 * 60);
const RESTART_LIMIT: Duration = Duration::from_secs(2 * 60);
const REMOVE_LIMIT: Duration = Duration::from_secs(10 * 60);
/// What `look.sh` checks, in its order, each of which must say `ok`: the packages `setup.sh` installs, then Claude
/// Code, srt, WSL's settings for the Linux, the `steplet` account and Steplet's folders. The gate is looked at apart.
const CHECKS: [&str; 15] = [
    "git",
    "python3",
    "python3-venv",
    "build-essential",
    "ca-certificates",
    "bubblewrap",
    "socat",
    "ripgrep",
    "nodejs",
    "npm",
    "claude-code",
    "srt",
    "wsl.conf",
    "steplet-account",
    "folders",
];

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
    /// What Steplet's Linux lacks (each a short name: `git`, `claude-code`, `wsl.conf`, …). Empty when it lacks
    /// nothing, is not installed, or Steplet could not look.
    pub missing: Vec<String>,
    /// This build's gate program is on its disk.
    pub gate: bool,
    /// Why Steplet could not look (WSL's list of Linuxes could not be read, or the look inside failed or took too
    /// long). Then `missing` and `gate` say nothing about what is there.
    pub trouble: Option<String>,
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

/// What is set up for Steplet's Linux called `name`, with `gate_program` as this build's gate. It changes nothing in
/// that Linux; to look inside, it starts it if it is stopped (WSL stops it again once it is idle).
pub fn look(name: &str, gate_program: &[u8]) -> State {
    let wsl_state = setup_program("wsl.exe")
        .and_then(|mut c| c.arg("--version").stdin(Stdio::null()).output())
        .map(|out| read_wsl_version(&out.stdout))
        .unwrap_or(Wsl::Missing);
    let mut state = State { wsl: wsl_state, linux: None, missing: Vec::new(), gate: false, trouble: None };
    if !matches!(state.wsl, Wsl::Ready { .. }) {
        return state;
    }
    if !own_name(name) {
        state.trouble = Some(format!("{name:?} is not a name of Steplet's own Linux"));
        return state;
    }
    state.linux = match wsl::find_distribution(name) {
        Ok(found) => found,
        Err(e) => {
            state.trouble = Some(format!("Steplet could not read WSL's list of Linuxes: {e}"));
            return state;
        }
    };
    let Some(id) = state.linux.clone() else { return state };
    let gate = gate_file_name(gate_program);
    let argv = ["/bin/bash", "-s", "--", CLAUDE_VERSION, SRT_VERSION, &gate];
    let looked = read_look(wsl::run_within(&id, "root", &argv, LOOK.as_bytes(), LOOK_LIMIT));
    State { missing: looked.missing, gate: looked.gate, trouble: looked.trouble, ..state }
}

/// What a look inside found.
struct Looked {
    missing: Vec<String>,
    gate: bool,
    trouble: Option<String>,
}

/// Reads `look.sh`'s output. A check counts as there only when the look said `ok` for it, so a look that said
/// nothing, or something else, finds everything missing; a look that failed or took too long is trouble.
fn read_look(out: io::Result<wsl::Output>) -> Looked {
    let trouble = |why: String| Looked {
        missing: Vec::new(),
        gate: false,
        trouble: Some(format!("Steplet could not look inside its Linux: {why}")),
    };
    let out = match out {
        Ok(out) if out.status == Some(0) => out,
        Ok(out) => {
            let why = last_lines(&out.stderr);
            return trouble(if why.is_empty() { format!("its look ended with {:?}", out.status) } else { why });
        }
        Err(e) => return trouble(e.to_string()),
    };
    let said_ok: Vec<&str> = out
        .stdout
        .lines()
        .filter_map(|l| {
            let mut words = l.split(' ');
            (words.next() == Some("ok")).then(|| words.next()).flatten()
        })
        .collect();
    Looked {
        missing: CHECKS.iter().filter(|c| !said_ok.contains(c)).map(|c| (*c).to_owned()).collect(),
        gate: said_ok.contains(&"gate"),
        trouble: None,
    }
}

/// Makes Steplet's Linux called `name` from nothing, with its disk in `location`, and returns its id. Each line of
/// progress goes to `line`. Safe to run again on a Linux whose setup failed half way: every step of `setup.sh` can be
/// repeated. Each step is ended if it takes longer than its limit (`INSTALL_LIMIT` and the rest).
pub fn install(name: &str, location: &Path, gate_program: &[u8], line: &dyn Fn(&str)) -> Result<String, SetupError> {
    let args = install_args(name, location)?;
    let find = |step: &'static str| {
        wsl::find_distribution(name).map_err(|e| SetupError::Step {
            step,
            detail: format!("Steplet could not read WSL's list of Linuxes: {e}"),
        })
    };
    if find("look for Steplet's Linux")?.is_none() {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        windows("install Steplet's Linux", &args, INSTALL_LIMIT, line)?;
    }
    let id = find("find Steplet's Linux")?.ok_or(SetupError::Step {
        step: "find Steplet's Linux",
        detail: format!("WSL lists no Linux called {name}"),
    })?;
    let setup = wsl::spawn(&id, "root", &["/bin/bash", "-s", "--", CLAUDE_VERSION, SRT_VERSION])
        .map_err(|e| SetupError::Step { step: "start the setup", detail: e.to_string() })?;
    stream("set up Steplet's Linux", setup, SETUP.as_bytes(), SETUP_LIMIT, line)?;
    // WSL reads its settings for a Linux (`/etc/wsl.conf`) when the Linux starts.
    windows("restart Steplet's Linux", &["--terminate", name], RESTART_LIMIT, line)?;
    line("--- the gate program of this build");
    let at = install_gate(&id, gate_program)
        .map_err(|e| SetupError::Step { step: "put the gate program on its disk", detail: format!("{e:?}") })?;
    line(&format!("gate: {at}"));
    Ok(id)
}

/// Takes Steplet's Linux called `name` away, with everything in it.
pub fn remove(name: &str) -> Result<(), SetupError> {
    if !own_name(name) {
        return Err(SetupError::Refused("Linux name"));
    }
    windows("remove Steplet's Linux", &["--unregister", name], REMOVE_LIMIT, &|_| {})
}

/// Runs `wsl.exe args` as setup starts it (`setup_program`), sending each line of its output to `line`, and ends it
/// if it takes longer than `limit`.
fn windows(step: &'static str, args: &[&str], limit: Duration, line: &dyn Fn(&str)) -> Result<(), SetupError> {
    let fail = |detail: String| SetupError::Step { step, detail };
    let mut command = setup_program("wsl.exe").map_err(|e| fail(e.to_string()))?;
    // Its own messages in UTF-8, not UTF-16.
    command.env("WSL_UTF8", "1").args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = command.spawn().map_err(|e| fail(e.to_string()))?;
    stream(step, child, b"", limit, line)
}

/// Feeds `input` to `child` and sends each line it prints to `line` as it comes. Fails, with its exit code and the end
/// of what it printed, unless it ends with exit code 0; ends it, and fails, if it takes longer than `limit` or its
/// output cannot be read.
fn stream(
    step: &'static str,
    mut child: Child,
    input: &[u8],
    limit: Duration,
    line: &dyn Fn(&str),
) -> Result<(), SetupError> {
    let fail = |detail: String| SetupError::Step { step, detail };
    let ended = |child: &mut Child, detail: String| {
        let _ = child.kill();
        let _ = child.wait();
        Err(fail(detail))
    };
    let started = Instant::now();
    let (stdout, stderr) = (child.stdout.take(), child.stderr.take());
    if let Some(mut stdin) = child.stdin.take() {
        let input = input.to_vec();
        thread::spawn(move || {
            let _ = io::Write::write_all(&mut stdin, &input);
        });
    }
    let errors = stderr.map(|mut e| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = e.read_to_end(&mut bytes);
            wsl::text(&bytes)
        })
    });
    // Its lines are read on a thread of their own, so the limit holds while it prints nothing.
    let (send, lines) = mpsc::channel::<io::Result<String>>();
    if let Some(out) = stdout {
        thread::spawn(move || {
            let mut out = BufReader::new(out);
            let mut buf = Vec::new();
            loop {
                buf.clear();
                let got = match out.read_until(b'\n', &mut buf) {
                    Ok(0) => break,
                    Ok(_) => Ok(wsl::text(&buf).trim_end_matches(['\r', '\n']).to_owned()),
                    Err(e) => Err(e),
                };
                let failed = got.is_err();
                if send.send(got).is_err() || failed {
                    break;
                }
            }
        });
    } else {
        drop(send);
    }
    let mut said = Vec::new();
    loop {
        match lines.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(text)) => {
                line(&text);
                said.push(text);
            }
            Ok(Err(e)) => return ended(&mut child, format!("Steplet could not read what it printed: {e}")),
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }
        if started.elapsed() > limit {
            return ended(&mut child, too_long(limit));
        }
    }
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() > limit => return ended(&mut child, too_long(limit)),
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(e) => return ended(&mut child, e.to_string()),
        }
    };
    if status.success() {
        return Ok(());
    }
    let errors = errors.and_then(|e| e.join().ok()).unwrap_or_default();
    let code = status.code().map_or_else(|| "no exit code".to_owned(), |c| format!("exit code {c}"));
    let mut all = said.join("\n");
    all.push('\n');
    all.push_str(&errors);
    Err(fail(format!("{code}\n{}", last_lines(&all))))
}

/// Why a step was ended at its limit.
fn too_long(limit: Duration) -> String {
    let secs = limit.as_secs();
    let took = if secs < 120 { format!("{secs} seconds") } else { format!("{} minutes", secs / 60) };
    format!("it took longer than {took}, so Steplet ended it")
}

/// The last 15 lines of `text`.
fn last_lines(text: &str) -> String {
    let lines: Vec<&str> = text.trim_end().lines().collect();
    lines[lines.len().saturating_sub(15)..].join("\n")
}

/// `program` from Windows' own folder, the way setup starts it: `SystemRoot` and `WINDOWS_FOLDERS` as its only
/// variables, started in Windows' System32 folder. A normal Windows account cannot write there (an administrator can,
/// and Steplet does not run as one), so nothing the program writes by a relative path can land among the receiver's
/// own files.
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
/// name that is not one of Steplet's own (`own_name`), and a place that is not a full path (`wsl.exe` would take it
/// from the folder it starts in) or not text (it could not be handed on exactly).
fn install_args(name: &str, location: &Path) -> Result<Vec<String>, SetupError> {
    if !own_name(name) {
        return Err(SetupError::Refused("Linux name"));
    }
    let Some(location) = location.to_str().filter(|_| location.is_absolute()) else {
        return Err(SetupError::Refused("install location"));
    };
    Ok(["--install", IMAGE, "--name", name, "--location", location, "--no-launch"].map(str::to_owned).to_vec())
}

/// A name of Steplet's own Linux: `steplet`, or `steplet-` and a plain name. Setup takes no other, so it never sets up,
/// looks inside or removes a Linux of the receiver's.
fn own_name(name: &str) -> bool {
    plain_name(name) && (name == LINUX || name.strip_prefix("steplet-").is_some_and(plain_name))
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
        // Not text: it could not be handed on exactly.
        let mut wide: Vec<u16> = r"C:\Steplet\".encode_utf16().collect();
        wide.push(0xd800);
        let not_text =
            std::path::PathBuf::from(<std::ffi::OsString as std::os::windows::ffi::OsStringExt>::from_wide(&wide));
        assert!(matches!(install_args("steplet", &not_text), Err(SetupError::Refused("install location"))));
    }

    /// Task 6's review: setup changes only Steplet's own Linux, so it takes only Steplet's own names: `steplet`, or
    /// `steplet-` and more (the tests' `steplet-setup-test`). The receiver's own Linux is never set up or removed.
    #[test]
    fn only_steplets_own_linux_names_are_taken() {
        for name in ["steplet", "steplet-setup-test", "steplet-2"] {
            assert!(own_name(name), "{name:?}");
        }
        for name in ["debian", "ubuntu", "stepletx", "my-steplet", "steplet-", "steplet--x", "-steplet", "Steplet", ""]
        {
            assert!(!own_name(name), "{name:?}");
        }
        let place = Path::new(r"C:\Users\made-up\AppData\Local\Steplet\wsl");
        assert!(matches!(install_args("debian", place), Err(SetupError::Refused("Linux name"))));
        // A made-up name no Linux has: even before this check, `wsl.exe` would only have said it found none.
        assert!(matches!(remove("not-steplet-made-up"), Err(SetupError::Refused("Linux name"))));
    }

    /// Task 6's review: the commands Steplet runs inside its Linux still get `SystemRoot` as their only Windows
    /// variable (C8); the folder variables are for setup's own calls of `wsl.exe` only.
    #[test]
    fn programs_for_inside_linux_get_systemroot_only() {
        let out = wsl::windows_program("cmd.exe")
            .unwrap()
            .raw_arg("/d /c echo [%SystemRoot%] [%SystemDrive%] [%TEMP%] [%LOCALAPPDATA%]")
            .output()
            .unwrap();
        let root = std::env::var("SystemRoot").unwrap();
        let said = String::from_utf8_lossy(&out.stdout);
        assert_eq!(said.trim_end(), format!("[{root}] [%SystemDrive%] [%TEMP%] [%LOCALAPPDATA%]"));
    }

    fn piped(mut command: Command) -> std::process::Child {
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap()
    }

    /// Task 6's review: a step that hangs (a stalled download, say) is ended at its limit and said to have taken too
    /// long, and its lines reach the screen while it runs.
    #[test]
    fn a_step_that_takes_too_long_is_ended_at_its_limit() {
        let mut ping = setup_program("ping.exe").unwrap();
        ping.args(["-n", "30", "127.0.0.1"]);
        let child = piped(ping);
        let pid = child.id();
        let lines = std::cell::Cell::new(0);
        let start = std::time::Instant::now();
        let result = stream("a test step", child, b"", Duration::from_secs(3), &|_| lines.set(lines.get() + 1));
        let took = start.elapsed();
        assert!(took < Duration::from_secs(8), "{took:?}");
        match result {
            Err(SetupError::Step { step, detail }) => {
                assert_eq!(step, "a test step");
                assert_eq!(detail, "it took longer than 3 seconds, so Steplet ended it");
            }
            other => panic!("{other:?}"),
        }
        assert!(lines.get() >= 2, "lines arrived while it ran: {}", lines.get());
        let mut list = setup_program("tasklist.exe").unwrap();
        let listed = list.args(["/FI", &format!("PID eq {pid}"), "/NH"]).output().unwrap();
        assert!(!String::from_utf8_lossy(&listed.stdout).to_lowercase().contains("ping.exe"), "ping was ended");
    }

    /// Task 6's review: a failed step says its exit code, and the end of what it printed on both outputs.
    #[test]
    fn a_failed_step_says_its_exit_code_and_the_end_of_what_it_printed() {
        let mut cmd = setup_program("cmd.exe").unwrap();
        cmd.raw_arg("/d /c echo first line& echo the reason it failed 1>&2& exit 3");
        let result = stream("a test step", piped(cmd), b"", Duration::from_secs(30), &|_| {});
        match result {
            Err(SetupError::Step { detail, .. }) => {
                assert!(detail.starts_with("exit code 3\n"), "{detail}");
                assert!(detail.contains("first line") && detail.contains("the reason it failed"), "{detail}");
            }
            other => panic!("{other:?}"),
        }
    }

    fn output(status: Option<i32>, stdout: &str, stderr: &str) -> io::Result<wsl::Output> {
        let stdout_bytes = stdout.as_bytes().to_vec();
        Ok(wsl::Output { status, stdout: stdout.into(), stderr: stderr.into(), stdout_bytes })
    }

    /// Task 6's review: a check counts as done only when `look.sh` said `ok` for it. A look that said nothing, or
    /// something else, finds everything missing; a look that could not be made is trouble, not a missing part.
    #[test]
    fn look_counts_only_what_it_saw() {
        let all_ok: String = CHECKS.iter().map(|c| format!("ok {c} 1.0\n")).collect();
        let seen = |out| {
            let l = read_look(out);
            (l.missing, l.gate, l.trouble)
        };
        assert_eq!(seen(output(Some(0), &format!("{all_ok}ok gate\n"), "")), (vec![], true, None));
        assert_eq!(seen(output(Some(0), &format!("{all_ok}missing gate\n"), "")), (vec![], false, None));
        let no_srt = all_ok.replace("ok srt 1.0\n", "missing srt\n");
        assert_eq!(seen(output(Some(0), &no_srt, "")), (vec!["srt".to_owned()], false, None));
        let everything: Vec<String> = CHECKS.iter().map(|c| (*c).to_owned()).collect();
        assert_eq!(seen(output(Some(0), "", "")), (everything.clone(), false, None), "it said nothing");
        assert_eq!(seen(output(Some(0), "ok\nokgit\nready\n", "")), (everything, false, None), "nothing it knows");
        let trouble = Some("Steplet could not look inside its Linux: made-up reason".to_owned());
        assert_eq!(seen(output(Some(1), &all_ok, "made-up reason\n")), (vec![], false, trouble));
        let timed_out = read_look(Err(io::Error::new(io::ErrorKind::TimedOut, "made-up time-out")));
        assert_eq!(timed_out.trouble.as_deref(), Some("Steplet could not look inside its Linux: made-up time-out"));
    }

    /// `CHECKS` names what `look.sh` checks, in its order: the packages of its loop, then each `say` of its own.
    #[test]
    fn the_checks_are_the_ones_look_sh_makes() {
        let packages = LOOK.lines().map(str::trim).find(|l| l.starts_with("for p in git ")).unwrap();
        let mut named: Vec<&str> =
            packages["for p in ".len()..].split(';').next().unwrap().split_whitespace().collect();
        for l in LOOK.lines().map(str::trim) {
            if let Some(rest) = l.strip_prefix("say ").or_else(|| l.split_once("; then say ").map(|(_, r)| r)) {
                let word = rest.split_whitespace().next().unwrap();
                if word != "\"$p\"" && !named.contains(&word) {
                    named.push(word);
                }
            }
        }
        assert_eq!(named, CHECKS);
    }
}
