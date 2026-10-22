//! Following a run to its end (`watch_run`, plan Task 9), with stand-in agents, so it needs no Linux: when Steplet
//! stops the run, what happens when stopping fails, and when the watch ends. The gate service is the real one, with
//! this test program in the gate's place (as in `tests/gate.rs`). Since phase 4 Task 3 also: what a live run sends to
//! the screen as it happens, and the receiver's Stop.

mod support;

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
use steplet_run::{
    Event, GateAnswer, GateLink, GateService, Live, Progress, RunOutcome, RunningAgent, StopSignal, Tripwire, Trust,
    build_report, serve, watch_run,
};
use support::MADE_UP_KEY;
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
        // What it had not yet handed out, as the real one hands over the events read after the end.
        (Ok(Some(124)), self.script.into_iter().collect())
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
    watch_live(stand, stop_after, Live::unattended(), &[])
}

/// As `watch`, for a live run: `live` gets what the screen would, cleaned of `secrets`.
fn watch_live(stand: Stand, stop_after: Duration, live: Live, secrets: &[&str]) -> (RunOutcome, Vec<String>) {
    let notes = stand.notes.clone();
    let pipe = pipe_name();
    let gate = gate(&pipe);
    let secrets: Vec<String> = secrets.iter().map(|s| (*s).to_owned()).collect();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let secrets: Vec<&str> = secrets.iter().map(String::as_str).collect();
        let _ =
            tx.send(watch_run(stand, gate, &support::checker_as_written(policy()), WORK, stop_after, &live, &secrets));
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
        let _ = tx.send(watch_run(
            stand,
            gate,
            &support::checker_as_written(policy()),
            WORK,
            Duration::from_secs(60),
            &Live::unattended(),
            &[],
        ));
    });
    let outcome = rx.recv_timeout(Duration::from_secs(10)).expect("watch_run returned within 10 s");
    assert!(matches!(outcome.stopped_by, Some(Tripwire::Hook(_))));
    assert_eq!(*notes.lock().unwrap(), ["stop", "asked the gate: \"deny\"", "finish, ended: true"]);
}

/// What a hidden secret shows as.
const HIDDEN: &str = "[hidden by Steplet]";

/// A live run's `Live`, which keeps everything sent to the screen.
fn collecting() -> (Live, Arc<Mutex<Vec<Progress>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let keep = seen.clone();
    let live = Live {
        on_progress: Box::new(move |p| keep.lock().unwrap().push(p)),
        stop: StopSignal::new(),
        ..Live::unattended()
    };
    (live, seen)
}

/// Phase 4 Task 3: each event reaches the screen as it comes, in order, followed by Steplet's own record of a call
/// that skipped the gate, and cleaned exactly as the report cleans the same text (the escape gone, the key hidden).
#[test]
fn each_event_reaches_the_screen_cleaned_and_in_order() {
    let ls = json!({"command": "ls"});
    let script = VecDeque::from([
        Event::AgentText(format!("\u{1b}[2Kdone {MADE_UP_KEY}")),
        Event::ToolUse { id: "u1".into(), name: "Bash".into(), input: ls.clone() },
        Event::ToolResult { id: "u1".into(), is_error: false, text: "a\u{1b}[31mb".into(), plain_end: true },
    ]);
    let ends_at = Some(Instant::now() + Duration::from_millis(300));
    let stand = Stand { script, outputs_closed: true, ends_at, ..Stand::default() };
    let (live, seen) = collecting();
    let (outcome, _) = watch_live(stand, Duration::from_secs(60), live, &[MADE_UP_KEY]);
    let seen = seen.lock().unwrap().clone();
    assert_eq!(
        seen,
        [
            Progress::Event(Event::AgentText(format!("[2Kdone {HIDDEN}"))),
            Progress::Event(Event::ToolUse { id: "u1".into(), name: "Bash".into(), input: ls }),
            Progress::Event(Event::ToolResult {
                id: "u1".into(),
                is_error: false,
                text: "a[31mb".into(),
                plain_end: true
            }),
            Progress::Event(Event::SkippedGate {
                id: "u1".into(),
                tool: "Bash".into(),
                what: "ls".into(),
                harmless: true
            }),
        ]
    );
    let report = build_report(&outcome, None, Trust::Unknown, &[MADE_UP_KEY]);
    assert_eq!(seen[0], Progress::Event(Event::AgentText(report.agent_says.clone())), "as the report cleans it");
    assert!(matches!(&seen[2], Progress::Event(Event::ToolResult { text, .. }) if *text == report.commands[0].output));
}

/// Phase 4 Task 3: the receiver's Stop stops the run as a tripwire would, once, and the report says the receiver
/// stopped it, not a tripwire.
#[test]
fn stop_signal_stops_the_run_and_the_report_says_you_stopped_it() {
    let stand = Stand { chatter: true, ..Stand::default() };
    let stop = StopSignal::new();
    let pressed = stop.clone();
    // The receiver presses Stop as soon as the first event is on screen.
    let live = Live { on_progress: Box::new(move |_| pressed.stop()), stop, ..Live::unattended() };
    let (outcome, notes) = watch_live(stand, Duration::from_secs(60), live, &[]);
    assert_eq!(notes, ["stop", "finish, ended: true"], "stopped once");
    assert_eq!((outcome.stopped_by.clone(), outcome.faults.clone()), (None, vec![]));
    let report = build_report(&outcome, None, Trust::Unknown, &[]);
    assert!(report.stopped_by_you, "{report:?}");
    assert!(!report.completed);
}

/// Task 3's review: the receiver presses Stop, and an event already on its way then sets off a tripwire (here a write
/// that did not ask the gate). The run was stopped by the receiver, not by the tripwire, so the report says so; the
/// write still shows as what it is, a call that skipped the checker, and not as cut short by the stop.
#[test]
fn a_tripwire_after_your_stop_does_not_take_the_stop_from_you() {
    let write = json!({"file_path": "/work/r1/repo/x.txt", "content": "made up"});
    let script = VecDeque::from([
        Event::Stderr("first".into()),
        Event::ToolUse { id: "u9".into(), name: "Write".into(), input: write },
        Event::ToolResult { id: "u9".into(), is_error: false, text: "written".into(), plain_end: true },
    ]);
    let stand = Stand { script, ..Stand::default() };
    let stop = StopSignal::new();
    let pressed = stop.clone();
    let live = Live { on_progress: Box::new(move |_| pressed.stop()), stop, ..Live::unattended() };
    let (outcome, notes) = watch_live(stand, Duration::from_secs(60), live, &[]);
    assert_eq!(notes, ["stop", "finish, ended: true"]);
    assert!(outcome.stopped_by_you);
    assert_eq!(outcome.stopped_by, None, "the stop was the receiver's");
    let report = build_report(&outcome, None, Trust::Unknown, &[]);
    let w = &report.commands[0];
    assert_eq!((&w.check, w.after_stop), (&steplet_run::Check::NotAsked { harmless: false }, false));
}

/// Phase 4 Task 4: when Steplet stops a run itself (here at a tripwire), it also sets the Stop signal, so every
/// question still on the receiver's screen is taken back at once; the report still says a tripwire stopped it, not
/// the receiver.
#[test]
fn a_tripwire_takes_back_every_open_question() {
    let hook = Event::HookEvent { name: "PreToolUse:Bash".into() };
    let stand = Stand { script: VecDeque::from([hook]), ..Stand::default() };
    let stop = StopSignal::new();
    let live = Live { stop: stop.clone(), ..Live::unattended() };
    let (outcome, notes) = watch_live(stand, Duration::from_secs(60), live, &[]);
    assert_eq!(notes, ["stop", "finish, ended: true"]);
    assert!(stop.is_set(), "open questions are taken back");
    assert_eq!((outcome.stopped_by, outcome.stopped_by_you), (Some(Tripwire::Hook("PreToolUse:Bash".into())), false));
}
