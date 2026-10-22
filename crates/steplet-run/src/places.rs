//! Where a path of the agent's file tools really leads (row 30, phase 5 Task 3). Claude Code 2.1.220 follows a link
//! in the working copy itself, sees that its real place is outside, and asks the gate, but it sends the gate the path
//! as written, inside the copy. So the checker decides on the real place, looked up in Steplet's Linux, never on the
//! path as written.

use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdin};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use crate::wsl;

/// How long one look-up may take before Steplet gives up on it (and refuses the request). Well inside the gate's room
/// for a question (`ASK_MARGIN`, 15 s, in run.rs), so a look-up never takes the receiver's time to answer.
const LOOK_UP_LIMIT: Duration = Duration::from_secs(10);

/// The look-up program: for each line on its input, a path in hexadecimal, it prints the path's real place in
/// hexadecimal on a line of its own, following every link; a part of the path that does not exist is kept as written.
/// Hexadecimal, so no character of a path (a line break, say) can mix with the lines, and the program holds no quote
/// or backslash to pass through a command line.
const LOOK_UP: &str = "import os,sys;o=sys.stdout;[(o.write(os.path.realpath(bytes.fromhex(l)).hex()+chr(10)),o.flush()) \
                       for l in sys.stdin]";

/// Where paths really lead.
pub trait RealPaths: Send + Sync {
    /// The real place of the absolute path `path`, following every link in it.
    fn real(&self, path: &str) -> io::Result<String>;
}

/// Steplet's Linux. Each path is looked up there as the steplet account, the account the agent's file tools run as,
/// so a folder that account cannot enter is not looked through, as those tools cannot either. One look-up program
/// serves the whole run, so a look-up takes a moment, not a new start of `wsl.exe`: the watcher makes one for every
/// read that skipped the gate, and does not look at Stop meanwhile. Every clean-up ends the steplet account's
/// programs, this one too; the next look-up starts it again. It ends when this is dropped.
pub struct LinuxPaths {
    distro_id: String,
    program: Mutex<Option<LookUp>>,
}

impl LinuxPaths {
    pub fn new(distro_id: &str) -> LinuxPaths {
        LinuxPaths { distro_id: distro_id.into(), program: Mutex::new(None) }
    }
}

impl RealPaths for LinuxPaths {
    fn real(&self, path: &str) -> io::Result<String> {
        // Linux would end the path at a zero byte, so the place looked up would not be the one asked about.
        if path.contains('\0') {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "the path holds a zero byte"));
        }
        let mut kept = self.program.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            let (mut program, fresh) = match kept.take() {
                Some(program) => (program, false),
                None => (LookUp::start(&self.distro_id)?, true),
            };
            match program.ask(path) {
                Ok(place) => {
                    *kept = Some(program);
                    return Ok(place);
                }
                // A program a clean-up has ended is started again, once; one that does not answer is not.
                Err(e) if !fresh && e.kind() != io::ErrorKind::TimedOut => continue,
                Err(e) => return Err(e),
            }
        }
    }
}

/// The look-up program, running in Steplet's Linux: its input, and its answers, one line each.
struct LookUp {
    child: Child,
    input: Option<ChildStdin>,
    answers: Receiver<String>,
}

impl LookUp {
    fn start(distro_id: &str) -> io::Result<LookUp> {
        let mut child = wsl::spawn(distro_id, "steplet", &["/usr/bin/python3", "-I", "-c", LOOK_UP])?;
        // Its error output is closed: it writes there only when it fails, and then it ends anyway.
        drop(child.stderr.take());
        let (Some(input), Some(output)) = (child.stdin.take(), child.stdout.take()) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::other("the look-up program's input and output were not connected"));
        };
        let (send, answers) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(output).lines() {
                let Ok(line) = line else { break };
                // A line that is not an answer (`wsl.exe` can print notices of its own) is passed over.
                let answer = !line.is_empty() && line.bytes().all(|b| b.is_ascii_hexdigit());
                if answer && send.send(line).is_err() {
                    break;
                }
            }
        });
        Ok(LookUp { child, input: Some(input), answers })
    }

    fn ask(&mut self, path: &str) -> io::Result<String> {
        let input = self.input.as_mut().ok_or_else(|| io::Error::other("the look-up program's input is closed"))?;
        writeln!(input, "{}", hex(path.as_bytes()))?;
        input.flush()?;
        let line = match self.answers.recv_timeout(LOOK_UP_LIMIT) {
            Ok(line) => line,
            Err(RecvTimeoutError::Timeout) => {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "Steplet's Linux did not answer in time"));
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, "the look-up program has ended"));
            }
        };
        let bytes = unhex(&line)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "the look-up program's answer was cut short"))?;
        String::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "the real place is not text Steplet can read"))
    }
}

impl Drop for LookUp {
    /// Closing its input ends the program; if it has not ended within 2 s, its `wsl.exe` is ended.
    fn drop(&mut self) {
        drop(self.input.take());
        let start = Instant::now();
        while matches!(self.child.try_wait(), Ok(None)) && start.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(20));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The bytes of `text`, two hexadecimal digits each; `None` for an odd count.
fn unhex(text: &str) -> Option<Vec<u8>> {
    let digits = text.as_bytes();
    if !digits.len().is_multiple_of(2) {
        return None;
    }
    digits.chunks(2).map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()).collect()
}
