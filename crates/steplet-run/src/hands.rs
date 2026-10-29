//! The hands of Steplet's own agent (the Nebius plan, Task 5). Each move the model asks for, once Steplet's checker
//! has allowed it, runs in Steplet's Linux through `start-hands.sh`: one at a time, as the steplet account, inside the
//! run's fence (`fence_json`), the file tools as well as the commands. Whatever a move leaves running is ended before
//! the next one. The model and its key stay on Windows; only the move goes in, and only what it printed comes out.

use std::io::{Read, Write};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::launch::{QUICK_CALL, ROOT_COMMON, ROOT_COMMON_PATH, install};
use crate::live::StopSignal;
use crate::types::RunPaths;
use crate::wsl;

const START_HANDS: &str = include_str!("start-hands.sh");
const START_HANDS_PATH: &str = "/opt/steplet/bin/start-hands";
const HANDS_PY: &str = include_str!("hands.py");
const HANDS_PY_PATH: &str = "/opt/steplet/bin/hands.py";

/// The most of a move's output `start-hands.sh` hands back; the rest is only counted.
pub const MOST_OUTPUT: usize = 1 << 20;
/// How much longer than a move's own limit Steplet waits for its answer: for Linux to start, the fence to be built and
/// taken down, and what the move left to be ended. Past it, the move is stopped as if Stop were pressed.
const BACKSTOP: Duration = Duration::from_secs(60);
/// The most of `start-hands.sh`'s error output kept, for its reason when it fails.
const MOST_ERROR: usize = 64 * 1024;

/// One move of the agent: a command line, run by bash in the working copy, or one of the file tools.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Move {
    Command {
        line: String,
    },
    /// From line `offset` (the first is 1), at most `limit` lines.
    Read {
        path: String,
        offset: Option<u64>,
        limit: Option<u64>,
    },
    Write {
        path: String,
        content: String,
    },
    /// Replaces the one place `old` is found; none or more than one is a refusal, and nothing changes.
    Edit {
        path: String,
        old: String,
        new: String,
    },
    /// Down `depth` levels (1 to 3).
    List {
        path: String,
        depth: u8,
    },
    /// The files whose path matches the glob `pattern`.
    Find {
        path: String,
        pattern: String,
    },
    /// The lines matching `regex`, in the files whose path matches `glob`, if given.
    Search {
        path: String,
        regex: String,
        glob: Option<String>,
    },
}

impl Move {
    /// The move's kind, as `start-hands.sh` takes it.
    fn kind(&self) -> &'static str {
        match self {
            Move::Command { .. } => "command",
            Move::Read { .. } => "read",
            Move::Write { .. } => "write",
            Move::Edit { .. } => "edit",
            Move::List { .. } => "list",
            Move::Find { .. } => "find",
            Move::Search { .. } => "search",
        }
    }

    /// The move's input: the command line itself, or the file tool's JSON.
    fn input(&self) -> Vec<u8> {
        let v = match self {
            Move::Command { line } => return line.clone().into_bytes(),
            Move::Read { path, offset, limit } => json!({ "path": path, "offset": offset, "limit": limit }),
            Move::Write { path, content } => json!({ "path": path, "content": content }),
            Move::Edit { path, old, new } => json!({ "path": path, "old": old, "new": new }),
            Move::List { path, depth } => json!({ "path": path, "depth": depth }),
            Move::Find { path, pattern } => json!({ "path": path, "pattern": pattern }),
            Move::Search { path, regex, glob } => json!({ "path": path, "regex": regex, "glob": glob }),
        };
        v.to_string().into_bytes()
    }
}

/// What one move did, as root saw it from outside the fence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Done {
    /// The move's exit code; `None` when its time ran out.
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    /// What it printed, its output and error output together, cut at `MOST_OUTPUT` bytes.
    pub output: String,
    /// How many bytes it printed in all.
    pub total_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandsError {
    /// Steplet's Linux could not be reached, the move could not be started, or a program would not stop.
    Linux(String),
    /// The run's layout is not as prepare made it, or WSL's shared folders could not be closed.
    Layout(String),
    /// `start-hands.sh` answered, but not in its own form.
    Unreadable(String),
    /// Stop was pressed while the move ran; it and whatever it started have been ended.
    Stopped,
}

/// Runs the agent's moves. Steplet's Linux in the app; a stand-in in the tests of the agent loop.
pub trait Hands: Send {
    fn run(&mut self, mv: &Move, limit: Duration, stop: &StopSignal) -> Result<Done, HandsError>;
}

/// The hands in Steplet's Linux, for the run `paths` that `prepare_own` prepared.
pub struct LinuxHands {
    distro_id: String,
    run_id: String,
    /// Whether this run's copies of the scripts are in place; they are written fresh for every run.
    installed: bool,
}

impl LinuxHands {
    pub fn new(paths: &RunPaths) -> LinuxHands {
        LinuxHands { distro_id: paths.distro_id().into(), run_id: paths.run_id().into(), installed: false }
    }

    /// Ends every program of the steplet account, as each move's end does: for a move stopped from Windows.
    fn end_leftovers(&self) -> Result<(), String> {
        let argv = ["/bin/bash", "-c", &format!(". {ROOT_COMMON_PATH} && stop_steplet")];
        match wsl::run_within(&self.distro_id, "root", &argv, b"", QUICK_CALL) {
            Ok(out) if out.status == Some(0) => Ok(()),
            Ok(out) => Err(format!("a program of the steplet account would not stop: {}", out.stderr.trim())),
            Err(e) => Err(format!("could not reach Steplet's Linux to end the move: {e}")),
        }
    }
}

impl Hands for LinuxHands {
    fn run(&mut self, mv: &Move, limit: Duration, stop: &StopSignal) -> Result<Done, HandsError> {
        if !self.installed {
            let scripts = [
                (ROOT_COMMON, ROOT_COMMON_PATH, "644"),
                (START_HANDS, START_HANDS_PATH, "755"),
                (HANDS_PY, HANDS_PY_PATH, "644"),
            ];
            install(&self.distro_id, &scripts).map_err(HandsError::Linux)?;
            self.installed = true;
        }
        let secs = limit.as_secs().max(1).to_string();
        let argv = [START_HANDS_PATH, &self.run_id, &secs, mv.kind()];
        let mut child = wsl::spawn(&self.distro_id, "root", &argv)
            .map_err(|e| HandsError::Linux(format!("could not start Steplet's Linux: {e}")))?;
        let (Some(mut input), Some(stdout), Some(stderr)) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take())
        else {
            let _ = child.kill();
            return Err(HandsError::Linux("the move's input and outputs were not connected".into()));
        };
        let bytes = mv.input();
        let deadline = Instant::now() + limit + BACKSTOP;
        let (status, out, err) = thread::scope(|s| {
            // Written from its own thread, and closed when written: start-hands reads it whole before the move starts.
            s.spawn(move || {
                let _ = input.write_all(&bytes);
            });
            let out = s.spawn(move || read_up_to(stdout, MOST_OUTPUT + 4096));
            let err = s.spawn(move || read_up_to(stderr, MOST_ERROR));
            let status = loop {
                match child.try_wait() {
                    Ok(Some(status)) => break Ok(status),
                    Ok(None) if stop.is_set() || Instant::now() >= deadline => {
                        // Ending `wsl.exe` closes its outputs, which ends the readers; root's script then finds no
                        // one to answer, and the move itself is ended from a call of its own.
                        let _ = child.kill();
                        let _ = child.wait();
                        break Err(if stop.is_set() {
                            HandsError::Stopped
                        } else {
                            HandsError::Linux(format!("the move did not end within {} s", (limit + BACKSTOP).as_secs()))
                        });
                    }
                    Ok(None) => thread::sleep(Duration::from_millis(20)),
                    Err(e) => break Err(HandsError::Linux(format!("could not follow the move: {e}"))),
                }
            };
            (status, out.join().unwrap_or_default(), err.join().unwrap_or_default())
        });
        let status = match status {
            Ok(status) => status,
            Err(e) => {
                self.end_leftovers().map_err(HandsError::Linux)?;
                return Err(e);
            }
        };
        let said = || {
            // The script writes to its error output; `wsl.exe` writes its own failures to standard output.
            let err = wsl::text(&err);
            let said = if err.trim().is_empty() { wsl::text(&out) } else { err };
            said.lines().rev().map(str::trim).find(|l| !l.is_empty()).unwrap_or("it said nothing").to_owned()
        };
        match status.code() {
            Some(0) => read_done(&out),
            Some(30 | 31) => Err(HandsError::Layout(said())),
            code => Err(HandsError::Linux(format!("start-hands ended with {code:?}: {}", said()))),
        }
    }
}

/// `start-hands.sh`'s answer: its own first line, then what the move printed. Only the first line counts, so a line
/// the move printed in the same form is part of its output, never its result.
fn read_done(out: &[u8]) -> Result<Done, HandsError> {
    let (first, rest) = match out.iter().position(|&b| b == b'\n') {
        Some(at) => (&out[..at], &out[at + 1..]),
        None => (out, &[][..]),
    };
    let unreadable = || HandsError::Unreadable(String::from_utf8_lossy(&first[..first.len().min(200)]).into_owned());
    let line = std::str::from_utf8(first).map_err(|_| unreadable())?;
    let fields: Vec<&str> = line.split(' ').collect();
    let [head, exit, timed, bytes] = fields.as_slice() else { return Err(unreadable()) };
    let (Some(exit), Some(timed), Some(bytes)) =
        (digits(exit, "exit="), digits(timed, "timed_out="), digits(bytes, "bytes="))
    else {
        return Err(unreadable());
    };
    if *head != "steplet-hands" || !matches!(timed, "0" | "1") {
        return Err(unreadable());
    }
    let (Ok(exit), Ok(total_bytes)) = (exit.parse::<i32>(), bytes.parse::<u64>()) else { return Err(unreadable()) };
    let timed_out = timed == "1";
    let kept = &rest[..rest.len().min(MOST_OUTPUT)];
    Ok(Done {
        exit_code: (!timed_out).then_some(exit),
        timed_out,
        output: String::from_utf8_lossy(kept).into_owned(),
        total_bytes,
    })
}

/// The digits after `name` in `field`, if that is all there is.
fn digits<'a>(field: &'a str, name: &str) -> Option<&'a str> {
    field.strip_prefix(name).filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
}

/// Everything `from` gives until it closes, keeping at most `most` bytes: the rest is read and dropped, so the other
/// side never waits on Steplet.
fn read_up_to(mut from: impl Read, most: usize) -> Vec<u8> {
    let mut kept = Vec::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        match from.read(&mut buffer) {
            Ok(0) | Err(_) => return kept,
            Ok(n) => {
                let room = most.saturating_sub(kept.len());
                kept.extend_from_slice(&buffer[..n.min(room)]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_root_s_first_line_is_the_result() {
        let done = read_done(b"steplet-hands exit=7 timed_out=0 bytes=40\nsteplet-hands exit=0 timed_out=0 bytes=1\n")
            .unwrap();
        assert_eq!(done.exit_code, Some(7));
        assert_eq!(done.output, "steplet-hands exit=0 timed_out=0 bytes=1\n");
        assert_eq!(done.total_bytes, 40);
    }

    #[test]
    fn start_hands_clears_the_agent_files_prepare_removes() {
        let line = format!("for f in {}; do", crate::prepare::AGENT_FILES.join(" "));
        assert!(crate::prepare::PREPARE.contains(&line), "prepare's strip: {line}");
        assert!(START_HANDS.contains(&line), "start-hands: {line}");
    }

    #[test]
    fn a_move_out_of_time_has_no_exit_code() {
        let done = read_done(b"steplet-hands exit=124 timed_out=1 bytes=0\n").unwrap();
        assert_eq!((done.exit_code, done.timed_out, done.output.as_str()), (None, true, ""));
    }

    #[test]
    fn an_answer_in_any_other_form_is_unreadable() {
        for bad in [
            &b""[..],
            b"hello\n",
            b"steplet-hands exit=0 timed_out=0\n",
            b"steplet-hands exit=0 timed_out=2 bytes=1\n",
            b"steplet-hands exit=-1 timed_out=0 bytes=1\n",
            b"steplet-hands exit=0 timed_out=0 bytes=1 more=1\n",
            b"steplet-hands  exit=0 timed_out=0 bytes=1\n",
            b"steplet-hands exit= timed_out=0 bytes=1\n",
        ] {
            assert!(matches!(read_done(bad), Err(HandsError::Unreadable(_))), "{:?}", String::from_utf8_lossy(bad));
        }
    }

    #[test]
    fn each_file_tool_sends_its_fields_as_json() {
        let read = Move::Read { path: "a.txt".into(), offset: Some(2), limit: None };
        assert_eq!(read.kind(), "read");
        assert_eq!(String::from_utf8(read.input()).unwrap(), r#"{"limit":null,"offset":2,"path":"a.txt"}"#);
        let command = Move::Command { line: "pwd; exit 3".into() };
        assert_eq!((command.kind(), command.input()), ("command", b"pwd; exit 3".to_vec()));
    }
}
