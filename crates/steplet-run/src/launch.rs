//! Starting the agent (plan section 3's launch, Task 7) and reading what it does. The run starts as root through
//! `start-run.sh`, which closes WSL's shared folders (H1) and then becomes `steplet` for good to start Claude Code,
//! with the task on its standard input. The event stream becomes `Event`s on a channel, and is also kept unchanged
//! in Steplet's logs folder on Windows (section 3), with the error output beside it.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::Child;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::stream::parse_line;
use crate::types::{BadValue, Event, RunPaths, RunRequest};
use crate::wsl;

const START_RUN: &str = include_str!("start-run.sh");
const START_RUN_PATH: &str = "/opt/steplet/bin/start-run";

#[derive(Debug)]
pub enum LaunchError {
    /// The request holds a value Steplet will not hand to its Linux.
    Refused(BadValue),
    /// The run could not be started; the text says why.
    Start(String),
}

/// One running agent. Dropping it while the agent still runs stops the run, which can take up to about 15 s.
pub struct RunHandle {
    child: Child,
    events: Receiver<Event>,
    threads: Vec<JoinHandle<()>>,
    distro_id: String,
    /// The start script's command line for this run, until it becomes the agent (`exec`).
    start_marker: String,
    /// The run's settings path, which appears in the command line of Claude Code and of its time limit, and in no
    /// other run's. Both markers are plain text to `pkill`: run ids and paths hold only letters, digits, `-`, `/`
    /// and `.` (any `.` also matches itself).
    marker: String,
    /// Set once the run has ended and whatever it left running is gone.
    finished: bool,
}

/// Starts the agent on the prepared run `paths`. `logs` is the Windows folder that keeps the run's stream. The
/// scripted model's address (tests only) must be `http://127.0.0.1:<port>`.
pub fn launch(
    req: &RunRequest,
    paths: &RunPaths,
    logs: &Path,
    model_url: Option<&str>,
) -> Result<RunHandle, LaunchError> {
    let seconds = req.deadline.as_secs();
    if seconds == 0 {
        return Err(LaunchError::Refused(BadValue("time limit")));
    }
    if req.max_turns == 0 {
        return Err(LaunchError::Refused(BadValue("most turns")));
    }
    if model_url.is_some_and(|u| !is_local_model(u)) {
        return Err(LaunchError::Refused(BadValue("model address")));
    }
    let start = |what: &str, e: io::Error| LaunchError::Start(format!("{what}: {e}"));
    std::fs::create_dir_all(logs).map_err(|e| start("could not make the logs folder", e))?;
    let log = |ext: &str| File::create(logs.join(format!("{}.{ext}", paths.run_id())));
    let stream_log = log("jsonl").map_err(|e| start("could not open the stream log", e))?;
    let error_log = log("stderr.txt").map_err(|e| start("could not open the error log", e))?;

    // The start script is written fresh for every run, so the one in Linux is always this program's.
    let install = ["/usr/bin/install", "-D", "-o", "root", "-g", "root", "-m", "755", "/dev/stdin", START_RUN_PATH];
    let out = wsl::run(paths.distro_id(), "root", &install, START_RUN.as_bytes())
        .map_err(|e| start("could not reach Steplet's Linux", e))?;
    if out.status != Some(0) {
        return Err(LaunchError::Start(format!("could not install the start script: {}", out.stderr.trim())));
    }

    let (secs, turns) = (seconds.to_string(), req.max_turns.to_string());
    let mut argv = vec![START_RUN_PATH, paths.run_id(), &secs, &turns];
    argv.extend(model_url);
    let mut child =
        wsl::spawn(paths.distro_id(), "root", &argv).map_err(|e| start("could not start Steplet's Linux", e))?;
    let (Some(mut input), Some(stdout), Some(stderr)) = (child.stdin.take(), child.stdout.take(), child.stderr.take())
    else {
        let _ = child.kill();
        return Err(LaunchError::Start("the agent's input and outputs were not connected".into()));
    };
    let task = req.task_text.clone().into_bytes();
    let (tx, events) = mpsc::channel();
    let threads = vec![
        // The task goes on standard input, which is then closed: dropping `input` closes it (C8, F1).
        thread::spawn(move || {
            let _ = input.write_all(&task);
        }),
        read_lines(stdout, stream_log, tx.clone(), parse_line),
        read_lines(stderr, error_log, tx, |line| vec![Event::Stderr(line.into())]),
    ];
    Ok(RunHandle {
        child,
        events,
        threads,
        distro_id: paths.distro_id().into(),
        start_marker: format!("{START_RUN_PATH} {} ", paths.run_id()),
        marker: paths.settings().into(),
        finished: false,
    })
}

impl RunHandle {
    /// The run's events, in order. The channel closes when the agent has ended and both outputs are read.
    pub fn events(&self) -> &Receiver<Event> {
        &self.events
    }

    /// Stops the run (F37). For up to 10 s, once a second: the start script is killed if it has not yet become the
    /// agent (so the agent never starts), and the end signal goes to Claude Code and its time limit. Then every
    /// program of the steplet account is killed, and `wsl.exe` itself last.
    pub fn stop(&mut self) -> io::Result<()> {
        let start = Instant::now();
        while self.child.try_wait()?.is_none() && start.elapsed() < Duration::from_secs(10) {
            self.root(&["/usr/bin/pkill", "-KILL", "-f", &self.start_marker])?;
            self.root(&["/usr/bin/pkill", "-TERM", "-f", &self.marker])?;
            self.ended_within(Duration::from_secs(1))?;
        }
        if self.child.try_wait()?.is_none() {
            self.root(&["/usr/bin/pkill", "-KILL", "-u", "steplet"])?;
            if !self.ended_within(Duration::from_secs(5))? {
                self.child.kill()?;
            }
        }
        self.end_leftovers()
    }

    /// Waits for the run to end, then ends whatever it left running, which would otherwise keep its outputs open,
    /// and gives the readers up to 10 s to finish. Returns the exit code: 124 at the time limit, and also after a
    /// stop, so whether Steplet stopped the run is the caller's own record, not this code.
    pub fn wait(mut self) -> io::Result<Option<i32>> {
        let status = self.child.wait()?;
        self.end_leftovers()?;
        let deadline = Instant::now() + Duration::from_secs(10);
        for t in self.threads.drain(..) {
            while !t.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(50));
            }
            // A reader still blocked after that is left to end on its own.
            if t.is_finished() {
                let _ = t.join();
            }
        }
        Ok(status.code())
    }

    /// Kills every program of the steplet account: one run at a time, so all of them are this run's.
    fn end_leftovers(&mut self) -> io::Result<()> {
        if !self.finished {
            self.root(&["/usr/bin/pkill", "-KILL", "-u", "steplet"])?;
            self.finished = true;
        }
        Ok(())
    }

    /// Runs one program as root in Steplet's Linux; its exit code is not needed (`pkill` says 1 when nothing matched).
    fn root(&self, argv: &[&str]) -> io::Result<()> {
        wsl::run(&self.distro_id, "root", argv, b"").map(drop)
    }

    fn ended_within(&mut self, limit: Duration) -> io::Result<bool> {
        let start = Instant::now();
        while start.elapsed() < limit {
            if self.child.try_wait()?.is_some() {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(false)
    }
}

impl Drop for RunHandle {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Reads `from` line by line: each line is written to `log` unchanged and turned into events by `read`.
fn read_lines(
    from: impl Read + Send + 'static,
    mut log: File,
    tx: Sender<Event>,
    read: fn(&str) -> Vec<Event>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut from = BufReader::new(from);
        let mut buf = Vec::new();
        while matches!(from.read_until(b'\n', &mut buf), Ok(n) if n > 0) {
            let _ = log.write_all(&buf);
            let line = wsl::text(&buf);
            for e in read(line.trim_end_matches(['\r', '\n'])) {
                let _ = tx.send(e);
            }
            buf.clear();
        }
    })
}

fn is_local_model(url: &str) -> bool {
    url.strip_prefix("http://127.0.0.1:")
        .is_some_and(|port| (1..=5).contains(&port.len()) && port.bytes().all(|b| b.is_ascii_digit()))
}
