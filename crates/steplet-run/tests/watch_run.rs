//! Following a run to its end (`watch_run`, plan Task 9), with stand-in agents, so it needs no Linux: when Steplet
//! stops the run, what happens when stopping fails, and when the watch ends. The gate service is the real one, with
//! this test program in the gate's place (as in `tests/gate.rs`).

use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{Event, GateAnswer, GateLink, GateService, RunOutcome, RunningAgent, Tripwire, serve, watch_run};
use windows_sys::Win32::Storage::FileSystem::SECURITY_IDENTIFICATION;

const WORK: &str = "/work/r1/repo";
const TOKEN: &str = "t9watchtoken";
const COULD_NOT_STOP: &str = "Steplet could not stop the run: Steplet's Linux does not answer";
const TIME_LIMIT: &str = "the run did not end at its time limit, so Steplet stopped it";

/// A stand-in for a running agent. It hands out `script`, then nothing (or, with `chatter`, an event every 10 ms,
/// for ever); it ends when stopped or killed, or by itself at `ends_at`. It notes each thing done to it.
#[derive(Default)]
struct Stand {
    script: VecDeque<Event>,
    chatter: bool,
    /// After the script, its outputs are closed: no more events will come.
    outputs_closed: bool,
    ends_at: Option<Instant>,
    stop_fails: bool,
    kill_fails: bool,
    /// Asking whether it has ended fails.
    ended_fails: bool,
    /// `finish` waits until it has ended, as the real one's wait for `wsl.exe` does: for ever, if it never ends.
    wait_in_finish: bool,
    /// The gate's pipe, asked once when the stop begins.
    ask_on_stop: Option<String>,
    ended: bool,
    notes: Arc<Mutex<Vec<String>>>,
}

impl Stand {
    fn note(&self, what: String) {
        self.notes.lock().unwrap().push(what);
    }

    fn has_ended(&self) -> bool {
        self.ended || self.ends_at.is_some_and(|t| Instant::now() >= t)
    }
}

impl RunningAgent for Stand {
    fn next_event(&mut self, wait: Duration) -> Result<Event, RecvTimeoutError> {
        if let Some(e) = self.script.pop_front() {
            return Ok(e);
        }
        if self.outputs_closed {
            return Err(RecvTimeoutError::Disconnected);
        }
        if self.chatter {
            thread::sleep(Duration::from_millis(10));
            return Ok(Event::Stderr("chatter".into()));
        }
        thread::sleep(wait);
        Err(RecvTimeoutError::Timeout)
    }

    fn ended(&mut self) -> io::Result<bool> {
        if self.ended_fails {
            return Err(io::Error::other("Windows gave no answer"));
        }
        Ok(self.has_ended())
    }

    fn stop(&mut self) -> io::Result<()> {
        self.note("stop".into());
        if let Some(pipe) = &self.ask_on_stop {
            let answer = ask(pipe, &json!({"command": "touch made.txt"}));
            self.note(format!("asked the gate: {}", answer["behavior"]));
        }
        if self.stop_fails {
            return Err(io::Error::other("Steplet's Linux does not answer"));
        }
        self.ended = true;
        Ok(())
    }

    fn kill(&mut self) -> io::Result<()> {
        self.note("kill".into());
        if self.kill_fails {
            return Err(io::Error::other("access is denied"));
        }
        self.ended = true;
        Ok(())
    }

    fn finish(self) -> (io::Result<Option<i32>>, Vec<Event>) {
        self.note(format!("finish, ended: {}", self.has_ended()));
        while self.wait_in_finish && !self.has_ended() {
            thread::sleep(Duration::from_millis(50));
        }
        (Ok(Some(124)), Vec::new())
    }
}

/// A pipe name no other test uses.
fn pipe_name() -> String {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    format!("steplet-t9w-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))
}

/// The gate service on `pipe`, expecting this test program as the gate, allowing everything it is asked.
fn gate(pipe: &str) -> GateService {
    let log = std::env::temp_dir().join(format!("{pipe}.gate.log"));
    let link = GateLink::new(
        "/opt/steplet/gate/steplet-gate.exe",
        pipe,
        TOKEN,
        &log.to_string_lossy(),
        Duration::from_secs(5),
    )
    .unwrap();
    let me = std::env::current_exe().unwrap().to_string_lossy().into_owned();
    serve(&link, &me, None, |r| GateAnswer::Allow { updated_input: r.input.clone() }).unwrap()
}

/// Asks the gate service on `pipe` about one command, as the gate program does.
fn ask(pipe: &str, input: &Value) -> Value {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .security_qos_flags(SECURITY_IDENTIFICATION)
        .open(format!(r"\\.\pipe\{pipe}"))
        .unwrap();
    let request =
        json!({"kind": "decide", "run_token": TOKEN, "tool_name": "Bash", "input": input, "tool_use_id": "u1"});
    writeln!(f, "{request}").unwrap();
    let mut answer = String::new();
    BufReader::new(&f).read_line(&mut answer).unwrap();
    serde_json::from_str(&answer).unwrap()
}

fn policy() -> Policy {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n---\n";
    Policy::from_steplet(&parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(), WORK).unwrap()
}

/// Follows `stand` with `watch_run`, which must return within 10 s; returns the outcome and what was done to it.
fn watch(stand: Stand, stop_after: Duration) -> (RunOutcome, Vec<String>) {
    let notes = stand.notes.clone();
    let pipe = pipe_name();
    let gate = gate(&pipe);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(watch_run(stand, gate, &policy(), WORK, stop_after));
    });
    let outcome = rx.recv_timeout(Duration::from_secs(10)).expect("watch_run returned within 10 s");
    let notes = notes.lock().unwrap().clone();
    (outcome, notes)
}

#[test]
fn a_run_whose_events_never_pause_is_still_stopped_at_the_backstop() {
    let stand = Stand { chatter: true, ..Stand::default() };
    let (outcome, notes) = watch(stand, Duration::from_millis(300));
    assert_eq!(notes, ["stop", "finish, ended: true"]);
    assert_eq!(outcome.faults, [TIME_LIMIT]);
    assert_eq!(outcome.stopped_by, None);
}

#[test]
fn a_run_that_steplet_s_linux_will_not_stop_is_ended_from_windows() {
    let stand = Stand { stop_fails: true, ..Stand::default() };
    let (outcome, notes) = watch(stand, Duration::from_millis(100));
    assert_eq!(notes, ["stop", "kill", "finish, ended: true"]);
    assert_eq!(outcome.faults, [TIME_LIMIT, COULD_NOT_STOP]);
}

#[test]
fn after_a_tripwire_a_stop_that_fails_ends_in_a_kill() {
    let hook = Event::HookEvent { name: "PreToolUse:Bash".into() };
    let stand = Stand { script: VecDeque::from([hook]), stop_fails: true, ..Stand::default() };
    let (outcome, notes) = watch(stand, Duration::from_secs(60));
    assert_eq!(notes, ["stop", "kill", "finish, ended: true"]);
    assert_eq!(outcome.stopped_by, Some(Tripwire::Hook("PreToolUse:Bash".into())));
    assert_eq!(outcome.faults, [COULD_NOT_STOP]);
}

/// When neither Steplet's Linux nor Windows can end the run, waiting for its end could take for ever. Steplet then
/// reads only what has come, says so, and goes on, so the run's folders are still removed and a report is made.
#[test]
fn a_run_that_neither_linux_nor_windows_can_end_does_not_hold_steplet_for_ever() {
    let hook = Event::HookEvent { name: "PreToolUse:Bash".into() };
    let stand = Stand {
        script: VecDeque::from([hook]),
        stop_fails: true,
        kill_fails: true,
        wait_in_finish: true,
        ..Stand::default()
    };
    let (outcome, notes) = watch(stand, Duration::from_secs(60));
    assert_eq!(notes, ["stop", "kill"], "never waited for an end that cannot come");
    assert_eq!(
        outcome.faults,
        [
            COULD_NOT_STOP,
            "Steplet could not end the run from Windows either: access is denied",
            "Steplet did not wait for the run to end, since it could not end it",
        ]
    );
    assert_eq!(outcome.exit_code, None);
}

/// When Steplet cannot tell whether the run has ended, it may still be running: Steplet stops it, as at a tripwire,
/// before waiting for it.
#[test]
fn when_steplet_cannot_tell_whether_the_run_ended_it_stops_it_first() {
    let stand = Stand { ended_fails: true, wait_in_finish: true, ..Stand::default() };
    let (outcome, notes) = watch(stand, Duration::from_secs(60));
    assert_eq!(notes, ["stop", "finish, ended: true"]);
    assert_eq!(outcome.faults, ["Steplet could not tell whether the run had ended: Windows gave no answer"]);
    assert_eq!(outcome.stopped_at, Some(0), "what came after was shaped by the stop");
}

#[test]
fn the_watch_lasts_until_the_agent_has_ended_even_when_its_outputs_close_first() {
    let stand =
        Stand { outputs_closed: true, ends_at: Some(Instant::now() + Duration::from_millis(400)), ..Stand::default() };
    let (outcome, notes) = watch(stand, Duration::from_secs(60));
    assert_eq!(notes, ["finish, ended: true"], "not stopped, and finished only once it had ended");
    assert_eq!(outcome.faults, Vec::<String>::new());
}

#[test]
fn at_a_tripwire_the_gate_refuses_before_the_stop_begins() {
    let hook = Event::HookEvent { name: "PreToolUse:Bash".into() };
    let pipe = pipe_name();
    // This test makes its own gate service, so the stand-in can ask it while it is being stopped.
    let gate = gate(&pipe);
    let notes = Arc::new(Mutex::new(Vec::new()));
    let stand =
        Stand { script: VecDeque::from([hook]), ask_on_stop: Some(pipe), notes: notes.clone(), ..Stand::default() };
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(watch_run(stand, gate, &policy(), WORK, Duration::from_secs(60)));
    });
    let outcome = rx.recv_timeout(Duration::from_secs(10)).expect("watch_run returned within 10 s");
    assert!(matches!(outcome.stopped_by, Some(Tripwire::Hook(_))));
    assert_eq!(*notes.lock().unwrap(), ["stop", "asked the gate: \"deny\"", "finish, ended: true"]);
}
