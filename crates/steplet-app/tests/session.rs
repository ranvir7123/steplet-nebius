//! A run from the app (Task 12), with a stand-in for the runner, so no Linux is needed: the file is checked again when
//! Run is pressed (row 30), one run at a time, each question reaches the screen on its own with the taskbar flashing,
//! the receiver's answer goes to the question it answers, closing asks first, a run a crash left is cleared at the
//! next start, and the senders are written when a run starts, never before.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::json;
use sha2::{Digest, Sha256};
use steplet_app::key::{self, Entries};
use steplet_app::pins::{self, Pins};
use steplet_app::session::{AGENT_CAP, AgentFn, DiscardFn, MAX_TURNS, Places, Ready, RunFn, Runner, Screen, Session};
use steplet_app::setup;
use steplet_app::views::{ANOTHER_WINDOW, ProgressView, RED_SENTENCE};
use steplet_core::policy::Policy;
use steplet_run::setup::{State, Wsl};
use steplet_run::{
    AgentSetup, ApiKey, Asker, Checker, GateAnswer, GateRequest, Live, Message, Nanodollars, RealPaths, Report,
    RunError, RunLock, RunOutcome, RunPaths, RunRequest, RunSetup, Trust, build_report,
};

/// The corpus's own time of viewing: 11 Oct 2026. The example files expire at the end of 2026-12-31 or later.
const NOW: i64 = 1_791_720_000;
const MADE_UP_KEY: &str = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";
/// Made-up text in a Nebius key's shape; never a real key.
const MADE_UP_NEBIUS_KEY: &str = "nebius-STEPLET-TEST-MADE-UP-NOT-A-KEY";
const DISTRO: &str = "{44f36236-99b1-42c0-84bd-7ff8392d51ba}";
const SUPER: &str = "nvidia/nemotron-3-super-120b-a12b";
/// The tests share Windows' one-run lock, so they run one at a time.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn alone() -> std::sync::MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner())
}

/// Paths as written: the stand-in runs below ask only about commands, which hold no path to look up.
struct AsWritten;

impl RealPaths for AsWritten {
    fn real(&self, path: &str) -> std::io::Result<String> {
        Ok(path.into())
    }
}

fn checker(policy: &Policy) -> Checker {
    Checker::new(policy.clone(), Arc::new(AsWritten))
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)).unwrap()
}

fn sha(raw: &[u8]) -> String {
    Sha256::digest(raw).iter().map(|b| format!("{b:02x}")).collect()
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("session-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn places(dir: &std::path::Path) -> Places {
    Places { marker: dir.join("run-in-progress.json"), pins: Some(dir.join("senders.json")), logs: dir.join("logs") }
}

fn ready() -> Result<Ready, String> {
    Ok(Ready {
        distro_id: DISTRO.into(),
        gate_exe: Some("/opt/steplet/gate/steplet-gate-0123456789abcdef.exe".into()),
        key: Some(ApiKey::new(MADE_UP_KEY.into())),
        nebius_key: Some(ApiKey::new(MADE_UP_NEBIUS_KEY.into())),
    })
}

/// Steplet's own agent on Nemotron 3 Super.
fn own() -> Runner {
    Runner::Own { model: SUPER.into() }
}

/// The screen, played by the test: what it was sent, and how often the taskbar flashed.
#[derive(Default)]
struct Seen {
    sent: Mutex<Vec<ProgressView>>,
    flashes: Mutex<u32>,
    closed: Mutex<bool>,
}

impl Screen for Seen {
    fn send(&self, p: ProgressView) {
        self.sent.lock().unwrap().push(p);
    }
    fn attention(&self) {
        *self.flashes.lock().unwrap() += 1;
    }
    fn close(&self) {
        *self.closed.lock().unwrap() = true;
    }
}

impl Seen {
    fn questions(&self) -> Vec<u32> {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .filter_map(|p| if let ProgressView::Question { question } = p { Some(question.id) } else { None })
            .collect()
    }
    fn ended(&self) -> bool {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .any(|p| matches!(p, ProgressView::Finished { .. } | ProgressView::Failed { .. }))
    }
}

fn wait_until(what: &str, done: impl Fn() -> bool) {
    let start = std::time::Instant::now();
    while !done() {
        assert!(start.elapsed() < Duration::from_secs(10), "waited too long for: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn empty_outcome() -> RunOutcome {
    RunOutcome {
        events: Vec::new(),
        requests: Vec::new(),
        stopped_by: None,
        stopped_at: None,
        exit_code: Some(0),
        faults: Vec::new(),
        stopped_by_you: false,
        answers: Vec::new(),
        limit: None,
        spent: None,
    }
}

fn empty_report() -> Report {
    build_report(&empty_outcome(), Some(&[]), Trust::NotRecorded, &[])
}

/// A stand-in run that ends at once.
fn quick() -> RunFn {
    Arc::new(|_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, _: Arc<Live>| Ok(empty_report()))
}

/// A stand-in run that waits until Stop is pressed.
fn until_stopped() -> RunFn {
    Arc::new(|_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, live: Arc<Live>| {
        while !live.stop.is_set() {
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(empty_report())
    })
}

fn bash(command: &str, id: &str) -> GateRequest {
    GateRequest {
        run_token: String::new(),
        tool_name: "Bash".into(),
        input: json!({ "command": command }),
        tool_use_id: id.into(),
    }
}

fn session(run: RunFn, now: i64, dir: &std::path::Path) -> Session {
    Session::new(run, nothing_to_discard(), Arc::new(move || now), places(dir))
}

/// Removing a leftover run, as the tests see it: nothing to do.
fn nothing_to_discard() -> DiscardFn {
    Arc::new(|_: &RunPaths| Ok(()))
}

/// Removing a leftover run, recorded: "distro run-id" for each run removed.
fn recording_discard() -> (DiscardFn, Arc<Mutex<Vec<String>>>) {
    let cleared: Arc<Mutex<Vec<String>>> = Arc::default();
    let keep = cleared.clone();
    let discard: DiscardFn = Arc::new(move |paths: &RunPaths| {
        keep.lock().unwrap().push(format!("{} {}", paths.distro_id(), paths.run_id()));
        Ok(())
    });
    (discard, cleared)
}

#[test]
fn start_run_checks_the_file_again_now() {
    let _one = alone();
    let dir = scratch("expired");
    let raw = read("examples/run-tests.steplet");
    // The example expires at the end of 2027-09-30; it was previewed before that, and Run is pressed after.
    let after_expiry = 1_822_521_600 + 2 * 86_400;
    let refused = session(quick(), after_expiry, &dir).start(
        &raw,
        &sha(&raw),
        None,
        false,
        Runner::ClaudeCode,
        ready(),
        Arc::new(Seen::default()),
    );
    assert_eq!(refused, Err("This file expired after you opened it.".to_string()));
}

#[test]
fn start_run_refuses_bytes_that_changed() {
    let _one = alone();
    let dir = scratch("changed");
    let raw = read("examples/run-tests.steplet");
    let mut changed = raw.clone();
    changed.extend_from_slice(b"\nOne more line the sender did not write.\n");
    let refused = session(quick(), NOW, &dir).start(
        &changed,
        &sha(&raw),
        None,
        false,
        Runner::ClaudeCode,
        ready(),
        Arc::new(Seen::default()),
    );
    assert_eq!(
        refused,
        Err("This file changed after you opened it. Open it again to see what it asks for now.".to_string())
    );
}

#[test]
fn red_needs_the_exact_sentence_on_the_rust_side_too() {
    let _one = alone();
    let dir = scratch("red");
    let raw = read("corpus/hostile/hide.link-mismatch.hit.steplet");
    let s = session(quick(), NOW, &dir).with_agent(quick_agent());
    // The same sentence, whichever runs the file (the Nebius plan, Task 8).
    for runner in [Runner::ClaudeCode, own()] {
        let start = |typed: Option<&str>| {
            s.start(&raw, &sha(&raw), typed, false, runner.clone(), ready(), Arc::new(Seen::default()))
        };
        let (wrong, dotted) = (RED_SENTENCE.replace("risks", "risk"), format!("{RED_SENTENCE}."));
        for typed in [None, Some(wrong.as_str()), Some(dotted.as_str())] {
            assert_eq!(
                start(typed),
                Err("This file is red: type the sentence exactly before you run it.".to_string()),
                "{runner:?} {typed:?}"
            );
        }
        let screen = Arc::new(Seen::default());
        assert_eq!(
            s.start(&raw, &sha(&raw), Some(RED_SENTENCE), false, runner.clone(), ready(), screen.clone()),
            Ok(())
        );
        wait_until("the run's end", || screen.ended() && !s.running());
    }
}

#[test]
fn run_is_refused_while_another_window_runs() {
    let _one = alone();
    let dir = scratch("busy");
    let raw = read("examples/run-tests.steplet");
    let _other_window = RunLock::take().expect("no run here yet");
    let refused = session(quick(), NOW, &dir).start(
        &raw,
        &sha(&raw),
        None,
        false,
        Runner::ClaudeCode,
        ready(),
        Arc::new(Seen::default()),
    );
    assert_eq!(refused, Err(ANOTHER_WINDOW.to_string()));
}

#[test]
fn an_answer_reaches_the_runner_bound_to_its_question() {
    let _one = alone();
    let dir = scratch("answer");
    let raw = read("examples/run-tests.steplet");
    let decided: Arc<Mutex<Vec<GateAnswer>>> = Arc::default();
    let keep = decided.clone();
    let run: RunFn =
        Arc::new(move |_: &RunLock, _: &RunRequest, _: &RunPaths, policy: &Policy, _: &RunSetup, live: Arc<Live>| {
            let asker = Asker::new(live, &[]);
            for (command, id) in [("touch first.txt", "toolu_01"), ("touch second.txt", "toolu_02")] {
                let answer = asker.decide(&checker(policy), &bash(command, id));
                keep.lock().unwrap().push(answer);
            }
            Ok(empty_report())
        });
    let screen = Arc::new(Seen::default());
    let s = session(run, NOW, &dir);
    s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen.clone()).unwrap();
    let question = |n: usize| {
        wait_until("a question", || screen.questions().len() > n);
        screen
            .sent
            .lock()
            .unwrap()
            .iter()
            .filter_map(|p| if let ProgressView::Question { question } = p { Some(question.clone()) } else { None })
            .nth(n)
            .unwrap()
    };
    let first = question(0);
    assert!(!s.answer(first.id + 7, true, &first.fingerprint), "no such question is open");
    // A yes that carries another request's fingerprint is a no.
    assert!(s.answer(first.id, true, "not-this-request"));
    let second = question(1);
    assert!(s.answer(second.id, true, &second.fingerprint));
    wait_until("both decisions", || decided.lock().unwrap().len() == 2);
    let decided = decided.lock().unwrap();
    assert!(matches!(decided[0], GateAnswer::Deny { .. }), "{:?}", decided[0]);
    assert_eq!(decided[1], GateAnswer::Allow { updated_input: json!({ "command": "touch second.txt" }) });
}

#[test]
fn a_question_flashes_the_taskbar_and_waits_its_turn() {
    let _one = alone();
    let dir = scratch("turn");
    let raw = read("examples/run-tests.steplet");
    let run: RunFn =
        Arc::new(|_: &RunLock, _: &RunRequest, _: &RunPaths, policy: &Policy, _: &RunSetup, live: Arc<Live>| {
            let asker = Arc::new(Asker::new(live, &[]));
            let second = {
                let (asker, policy) = (asker.clone(), policy.clone());
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(100));
                    asker.decide(&checker(&policy), &bash("touch second.txt", "toolu_02"))
                })
            };
            asker.decide(&checker(policy), &bash("touch first.txt", "toolu_01"));
            second.join().unwrap();
            Ok(empty_report())
        });
    let screen = Arc::new(Seen::default());
    let s = session(run, NOW, &dir);
    s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen.clone()).unwrap();
    wait_until("the first question", || screen.questions().len() == 1);
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(screen.questions().len(), 1, "the second waits behind the first");
    assert_eq!(*screen.flashes.lock().unwrap(), 1, "the taskbar flashed for the first");
    assert!(
        screen.sent.lock().unwrap().iter().any(|p| matches!(p, ProgressView::Waiting { count: 1 })),
        "the screen is told one more is waiting"
    );
    let first = screen.questions()[0];
    assert!(s.answer(first, false, ""));
    wait_until("the second question", || screen.questions().len() == 2);
    assert_eq!(*screen.flashes.lock().unwrap(), 2, "and again for the second");
    assert!(s.answer(screen.questions()[1], false, ""));
    wait_until("the run's end", || screen.ended());
}

#[test]
fn closing_during_a_run_asks_first() {
    let _one = alone();
    let dir = scratch("close");
    let raw = read("examples/run-tests.steplet");
    let screen = Arc::new(Seen::default());
    let s = session(until_stopped(), NOW, &dir);
    assert!(s.may_close(), "nothing runs yet");
    s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen.clone()).unwrap();
    assert!(!s.may_close(), "a run is going on: the window asks first");
    s.stop_and_close();
    wait_until("the window to close after the run", || *screen.closed.lock().unwrap());
    assert!(screen.ended() && s.may_close());
}

#[test]
fn a_run_left_by_a_crash_is_cleared_at_start() {
    let _one = alone();
    let dir = scratch("crash");
    let p = places(&dir);
    std::fs::write(&p.marker, format!(r#"{{"run_id": "app-1791720000-77", "distro_id": "{DISTRO}"}}"#)).unwrap();
    let (discard, cleared) = recording_discard();
    let s = Session::new(quick(), discard, Arc::new(|| NOW), p.clone());
    let note = s.clear_leftover();
    assert_eq!(*cleared.lock().unwrap(), [format!("{DISTRO} app-1791720000-77")], "that run was stopped and removed");
    assert_eq!(
        note.as_deref(),
        Some(
            "Steplet was closed while a file was running. It stopped what was left of that run and removed its \
             folders. Nothing of it is left."
        )
    );
    assert!(!p.marker.exists(), "the marker is gone");
    assert_eq!(s.clear_leftover(), None, "nothing left the next time");
}

#[test]
fn the_pins_are_written_when_run_starts_and_not_before() {
    let _one = alone();
    let dir = scratch("pins");
    let p = places(&dir);
    let raw = read("corpus/hostile/sig.invalid.near.steplet");
    let pins_path = p.pins.clone().unwrap();
    // Opening previews the file and writes nothing.
    let _ = steplet_app::open::preview_file(&raw, NOW, &pins::load(&pins_path));
    assert!(!pins_path.exists(), "opening wrote no pin");
    let screen = Arc::new(Seen::default());
    Session::new(quick(), nothing_to_discard(), Arc::new(|| NOW), p)
        .start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen.clone())
        .unwrap();
    wait_until("the run's end", || screen.ended());
    let Pins::Known(known) = pins::load(&pins_path) else { panic!("the pins file reads") };
    assert_eq!(known.iter().map(|k| (k.sender.as_str(), k.runs)).collect::<Vec<_>>(), [("Steplet test vectors", 1)]);
}

#[test]
fn a_run_that_cannot_start_says_why_in_words() {
    let _one = alone();
    let dir = scratch("cannot");
    let raw = read("examples/run-tests.steplet");
    let run: RunFn = Arc::new(|_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, _: Arc<Live>| {
        Err(RunError::Busy)
    });
    let screen = Arc::new(Seen::default());
    session(run, NOW, &dir).start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen.clone()).unwrap();
    wait_until("the run's end", || screen.ended());
    let sent = screen.sent.lock().unwrap();
    let Some(ProgressView::Failed { failure, .. }) = sent.last() else { panic!("{sent:?}") };
    assert_eq!(failure.reason, ANOTHER_WINDOW);
    assert!(!failure.reason.contains("Busy") && !failure.reason.contains('{'), "words, not code");
}

#[test]
fn a_call_that_skipped_the_gate_reaches_the_screen_with_its_check() {
    use steplet_app::session::{ASK_LIMIT, RUN_LIMIT, Relay};
    use steplet_app::views::Limits;
    use steplet_run::{Event, Progress};
    let mut relay = Relay::new(Limits { ask: ASK_LIMIT, run: RUN_LIMIT });
    let skipped = |harmless, not_offered| {
        Progress::Event(Event::SkippedGate {
            id: "toolu_s".into(),
            tool: "Bash".into(),
            what: "cat README".into(),
            harmless,
            not_offered,
        })
    };
    for (harmless, not_offered, kind, words) in [
        (true, false, "not_asked", "Ran without asking Steplet: a harmless read"),
        (false, false, "ran_without_asking", "Ran without asking Steplet: Steplet stopped the run"),
        (false, true, "not_offered", "A tool this run does not offer, so it did not run: Steplet stopped the run"),
    ] {
        let views = relay.relay(&skipped(harmless, not_offered)).views;
        let checks: Vec<_> = views
            .iter()
            .filter_map(|v| match v {
                ProgressView::Checked { call, check, .. } => {
                    Some((call.as_str(), check.kind.as_str(), check.words.as_str()))
                }
                _ => None,
            })
            .collect();
        assert_eq!(checks, [("toolu_s", kind, words)], "{views:?}");
    }
}

/// Row 22, on the Rust side too: a sender this computer knows, turning up with another key, makes the file red here
/// (the preview's colour is the worst of the file's findings and this computer's). Starting it needs the sentence, and
/// a refused start keeps the old pin.
#[test]
fn a_known_sender_with_another_key_needs_the_sentence_too() {
    let _one = alone();
    let dir = scratch("key-changed");
    let p = places(&dir);
    let pins_path = p.pins.clone().unwrap();
    const OTHER_KEY: &str = "SHA256:Q2hhbmdlZEtleUNoYW5nZWRLZXlDaGFuZ2VkS2V5Q2g";
    pins::record(&pins_path, "Steplet test vectors", OTHER_KEY).unwrap();
    let raw = read("corpus/hostile/sig.invalid.near.steplet");
    let preview = steplet_app::open::preview_file(&raw, NOW, &pins::load(&pins_path));
    assert_eq!(preview.colour, "red", "the preview is red here");
    let s = Session::new(quick(), nothing_to_discard(), Arc::new(|| NOW), p);
    assert_eq!(
        s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), Arc::new(Seen::default())),
        Err("This file is red: type the sentence exactly before you run it.".to_string())
    );
    let Pins::Known(known) = pins::load(&pins_path) else { panic!("the pins file reads") };
    assert_eq!(known.iter().map(|k| k.key.as_str()).collect::<Vec<_>>(), [OTHER_KEY], "the old pin stays");
    let screen = Arc::new(Seen::default());
    assert_eq!(
        s.start(&raw, &sha(&raw), Some(RED_SENTENCE), false, Runner::ClaudeCode, ready(), screen.clone()),
        Ok(())
    );
    wait_until("the run's end", || screen.ended());
}

// ------------------------------------------------------------------------------------------------------------------
// Task 12's review

/// Review 1: the clean-up at start never touches a run that another Steplet window is doing right now (a double-click
/// on a second file starts a second Steplet). Only a marker no run holds the lock for is a crash's leftover.
#[test]
fn clearing_at_start_leaves_a_run_in_another_window_alone() {
    let _one = alone();
    let dir = scratch("other-window");
    let p = places(&dir);
    std::fs::write(&p.marker, format!(r#"{{"run_id": "app-1791720000-77", "distro_id": "{DISTRO}"}}"#)).unwrap();
    let (discard, cleared) = recording_discard();
    let s = Session::new(quick(), discard, Arc::new(|| NOW), p.clone());
    let other_window = RunLock::take().unwrap();
    assert_eq!(s.clear_leftover(), None, "no note: nothing was cleared");
    assert!(cleared.lock().unwrap().is_empty(), "the other window's run was not removed");
    assert!(p.marker.exists(), "its marker stays");
    drop(other_window);
    assert!(s.clear_leftover().is_some(), "once no run holds the lock, it is a leftover");
    assert_eq!(cleared.lock().unwrap().len(), 1);
}

/// Review 5: after a crash inside this window's run, the next Run first removes what that run left, and only then
/// starts (its marker is never written over).
#[test]
fn a_crashed_run_is_cleared_before_the_next_run_starts() {
    let _one = alone();
    let dir = scratch("crash-then-run");
    let p = places(&dir);
    let raw = read("examples/run-tests.steplet");
    let runs = Arc::new(Mutex::new(0));
    let count = runs.clone();
    let crash_once: RunFn =
        Arc::new(move |_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, _: Arc<Live>| {
            let mut n = count.lock().unwrap();
            *n += 1;
            if *n == 1 {
                drop(n);
                panic!("a made-up crash inside the run");
            }
            Ok(empty_report())
        });
    let (discard, cleared) = recording_discard();
    // Each start gets its own second, so the two runs have different ids.
    let clock = Arc::new(std::sync::atomic::AtomicI64::new(NOW));
    let tick = clock.clone();
    let now: Arc<dyn Fn() -> i64 + Send + Sync> = Arc::new(move || tick.load(std::sync::atomic::Ordering::SeqCst));
    let s = Session::new(crash_once, discard, now, p.clone());
    let first = Arc::new(Seen::default());
    s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), first.clone()).unwrap();
    wait_until("the crash's end", || first.ended() && !s.running());
    assert!(p.marker.exists(), "the crashed run's marker stays");
    let crashed: serde_json::Value = serde_json::from_slice(&std::fs::read(&p.marker).unwrap()).unwrap();
    clock.store(NOW + 5, std::sync::atomic::Ordering::SeqCst);
    let second = Arc::new(Seen::default());
    s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), second.clone()).unwrap();
    wait_until("the second run's end", || second.ended() && !s.running());
    assert_eq!(
        *cleared.lock().unwrap(),
        [format!("{DISTRO} {}", crashed["run_id"].as_str().unwrap())],
        "the crashed run was removed before the next one"
    );
    assert!(matches!(second.sent.lock().unwrap().last(), Some(ProgressView::Finished { .. })), "and the next one ran");
    assert!(!p.marker.exists(), "the second run's marker is gone after it");
}

/// A `now` that takes its time on its first use, so a test can act while a start is still checking the file.
fn slow_now(first: Duration) -> Arc<dyn Fn() -> i64 + Send + Sync> {
    let waited = Arc::new(std::sync::atomic::AtomicBool::new(false));
    Arc::new(move || {
        if !waited.swap(true, std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(first);
        }
        NOW
    })
}

/// Review 2: two starts at once (a double click on Run) start one run, and the other is refused.
#[test]
fn two_starts_at_once_start_one_run() {
    let _one = alone();
    let dir = scratch("two-starts");
    let raw = read("examples/run-tests.steplet");
    let s = Arc::new(Session::new(
        until_stopped(),
        nothing_to_discard(),
        slow_now(Duration::from_millis(300)),
        places(&dir),
    ));
    let screen = Arc::new(Seen::default());
    let starts: Vec<_> = (0..2)
        .map(|_| {
            let (s, raw, screen) = (s.clone(), raw.clone(), screen.clone());
            std::thread::spawn(move || s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen))
        })
        .collect();
    let mut got: Vec<Result<(), String>> = starts.into_iter().map(|t| t.join().unwrap()).collect();
    got.sort();
    assert_eq!(got, [Ok(()), Err("A file is running in this window already.".to_string())]);
    s.stop();
    wait_until("the run's end", || screen.ended() && !s.running());
}

/// Review 3: Stop pressed while the start is still checking the file is kept: nothing runs. Closing then asks first.
#[test]
fn stop_pressed_while_starting_is_kept() {
    let _one = alone();
    let dir = scratch("stop-while-starting");
    let raw = read("examples/run-tests.steplet");
    let ran = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let saw = ran.clone();
    let run: RunFn =
        Arc::new(move |_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, live: Arc<Live>| {
            saw.store(true, std::sync::atomic::Ordering::SeqCst);
            while !live.stop.is_set() {
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(empty_report())
        });
    let s = Arc::new(Session::new(run, nothing_to_discard(), slow_now(Duration::from_millis(300)), places(&dir)));
    let screen = Arc::new(Seen::default());
    let starting = {
        let (s, raw, screen) = (s.clone(), raw.clone(), screen.clone());
        std::thread::spawn(move || s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen))
    };
    std::thread::sleep(Duration::from_millis(100));
    assert!(!s.may_close(), "closing while it starts asks first");
    s.stop();
    assert_eq!(starting.join().unwrap(), Ok(()));
    wait_until("the run's end", || screen.ended() && !s.running());
    assert!(!ran.load(std::sync::atomic::Ordering::SeqCst), "the agent never started");
    let sent = screen.sent.lock().unwrap();
    let Some(ProgressView::Failed { failure, .. }) = sent.last() else { panic!("{sent:?}") };
    assert_eq!(failure.reason, "You stopped the run before it started. Nothing ran.");
}

/// Review 3: "Stop and close" when the run has just ended closes at once, and leaves nothing behind that would close
/// the window at the end of the next run.
#[test]
fn closing_when_no_run_goes_on_closes_now_and_only_now() {
    let _one = alone();
    let dir = scratch("close-now");
    let raw = read("examples/run-tests.steplet");
    let s = session(quick(), NOW, &dir);
    assert!(!s.stop_and_close(), "nothing runs: the window closes now");
    let screen = Arc::new(Seen::default());
    s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen.clone()).unwrap();
    wait_until("the run's end", || screen.ended() && !s.running());
    assert!(!*screen.closed.lock().unwrap(), "the next run's end does not close the window");
}

/// A screen that takes a moment for each piece, as a busy window might.
#[derive(Default)]
struct Slow(Seen);

impl Screen for Slow {
    fn send(&self, p: ProgressView) {
        std::thread::sleep(Duration::from_millis(40));
        self.0.send(p);
    }
    fn attention(&self) {}
    fn close(&self) {}
}

/// Review 4: progress made on two threads at once (the gate's and the event stream's) reaches the screen in the order
/// it was made: here the last word on waiting questions is that none waits.
#[test]
fn progress_reaches_the_screen_in_the_order_it_was_made() {
    let _one = alone();
    let dir = scratch("order");
    let raw = read("examples/run-tests.steplet");
    let run: RunFn =
        Arc::new(|_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, live: Arc<Live>| {
            use steplet_run::{Progress, Question, Reply};
            let q = |id: u64, what: &str| Question {
                id,
                tool: "Bash".into(),
                what: what.into(),
                input: json!({ "command": what }),
                reason: "the file does not ask for this command".into(),
                fingerprint: format!("f{id}"),
                asked_at: std::time::Instant::now(),
            };
            (live.on_progress)(Progress::Asking(q(1, "touch a")));
            (live.on_progress)(Progress::Asking(q(2, "touch b")));
            // Question 1 is answered (the second comes on screen, none waits) while question 2 is then taken back.
            std::thread::scope(|t| {
                t.spawn(|| (live.on_progress)(Progress::Answered { question: 1, reply: Reply::No }));
                std::thread::sleep(Duration::from_millis(10));
                t.spawn(|| (live.on_progress)(Progress::Answered { question: 2, reply: Reply::Withdrawn }));
            });
            Ok(empty_report())
        });
    let screen = Arc::new(Slow::default());
    session(run, NOW, &dir).start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen.clone()).unwrap();
    wait_until("the run's end", || screen.0.ended());
    let sent = screen.0.sent.lock().unwrap();
    let kinds: Vec<String> = sent
        .iter()
        .map(|p| match p {
            ProgressView::Question { question } => format!("question {}", question.id),
            ProgressView::QuestionClosed { id, .. } => format!("closed {id}"),
            ProgressView::Waiting { count } => format!("waiting {count}"),
            ProgressView::Finished { .. } => "finished".into(),
            other => format!("{other:?}"),
        })
        .collect();
    assert_eq!(kinds, ["question 1", "waiting 1", "closed 1", "question 2", "waiting 0", "closed 2", "finished"]);
}

/// A run whose model refused, and Claude Code did not switch: the report Steplet offers another model after.
fn refused_report() -> Report {
    let o = RunOutcome {
        events: vec![
            steplet_run::Event::ModelSwitch {
                from: "claude-opus-5".into(),
                to: None,
                category: "cyber".into(),
                explanation: "This request looks like it could be used to harm a computer system.".into(),
            },
            steplet_run::Event::Result {
                subtype: "success".into(),
                is_error: false,
                stop_reason: "refusal".into(),
                models: vec!["claude-opus-5".into()],
                text: "I can't help with that.".into(),
            },
        ],
        ..empty_outcome()
    };
    build_report(&o, Some(&[]), Trust::NotRecorded, &[])
}

/// Task 13 (E1): "Run again, allowing another model" reaches the run only right after a model refused this file's
/// run in this window. Any other start never allows it, whatever the page asks for.
#[test]
fn another_model_is_allowed_only_right_after_a_model_refused_this_file() {
    let _one = alone();
    let dir = scratch("allow-switch");
    let raw = read("examples/run-tests.steplet");
    let other = read("corpus/ordinary/pytest-minimal.steplet");
    let seen: Arc<Mutex<Vec<bool>>> = Arc::default();
    let refuse = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (keep, refusing) = (seen.clone(), refuse.clone());
    let run: RunFn =
        Arc::new(move |_: &RunLock, req: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, _: Arc<Live>| {
            keep.lock().unwrap().push(req.allow_model_switch);
            Ok(if refusing.load(std::sync::atomic::Ordering::SeqCst) { refused_report() } else { empty_report() })
        });
    let s = session(run, NOW, &dir);
    let start = |raw: &[u8], allow: bool| {
        let screen = Arc::new(Seen::default());
        let started = s.start(raw, &sha(raw), None, allow, Runner::ClaudeCode, ready(), screen.clone());
        if started.is_ok() {
            wait_until("the run's end", || screen.ended() && !s.running());
        }
        started
    };
    const NOT_NOW: &str = "Steplet lets Claude Code switch models only right after a model refused this file's run.";
    // No model has refused yet.
    assert_eq!(start(&raw, true), Err(NOT_NOW.into()));
    assert_eq!(start(&raw, false), Ok(()));
    // A model refuses this file's run: another model may be allowed now, for this file only.
    refuse.store(true, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(start(&raw, false), Ok(()));
    refuse.store(false, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(start(&other, true), Err(NOT_NOW.into()));
    assert_eq!(start(&raw, true), Ok(()));
    // That run finished without a refusal: the next start may not switch again.
    assert_eq!(start(&raw, true), Err(NOT_NOW.into()));
    assert_eq!(*seen.lock().unwrap(), [false, false, true]);
}

/// Task 14 review: the live log says how a call ended in the report's words, so a call Steplet or the receiver
/// refused first says "It did not run" in both.
#[test]
fn a_refused_call_ends_in_the_live_log_as_in_the_report() {
    use steplet_app::session::{ASK_LIMIT, RUN_LIMIT, Relay};
    use steplet_app::views::{EventView, Limits};
    use steplet_run::{Check, Event, NotYes, Progress};
    let mut relay = Relay::new(Limits { ask: ASK_LIMIT, run: RUN_LIMIT });
    let input = json!({ "command": "rm -rf build" });
    let mut ended = Vec::new();
    for p in [
        Progress::Event(Event::ToolUse { id: "u1".into(), name: "Bash".into(), input: input.clone() }),
        Progress::Checked { call: "u1".into(), check: Check::RefusedWhenAsked { how: NotYes::SaidNo }, reason: None },
        Progress::Event(Event::ToolResult {
            id: "u1".into(),
            is_error: true,
            text: "Steplet refused this: the receiver said no".into(),
            plain_end: false,
        }),
        Progress::Event(Event::ToolUse { id: "u2".into(), name: "Bash".into(), input: json!({ "command": "ls" }) }),
        Progress::Checked { call: "u2".into(), check: Check::Allowed, reason: None },
        Progress::Event(Event::ToolResult { id: "u2".into(), is_error: false, text: "a\n".into(), plain_end: true }),
    ] {
        for v in relay.relay(&p).views {
            if let ProgressView::Event { event: EventView::Result { ended: words, .. } } = v {
                ended.push(words);
            }
        }
    }
    assert_eq!(ended, ["It did not run", "Exit code 0"]);
}

const GATE_EXE: &str = "/opt/steplet/gate/steplet-gate-0123456789abcdef.exe";
const NEEDS_NEBIUS: &str = "Needs a Nebius key. Add it in Setup.";
const NEEDS_ANTHROPIC: &str = "Needs an Anthropic key. Add it in Setup.";

/// Setup as it looks when everything is there.
fn set_up() -> State {
    State {
        wsl: Wsl::Ready { version: "2.6.3.0".into() },
        linux: Some(DISTRO.into()),
        missing: Vec::new(),
        gate: true,
        trouble: None,
    }
}

/// Forgets a made-up Credential Manager entry when the test ends, however it ends.
struct Forget(&'static str);

impl Drop for Forget {
    fn drop(&mut self) {
        let _ = key::forget(self.0);
    }
}

/// A stand-in for Steplet's own agent that ends at once.
fn quick_agent() -> AgentFn {
    Arc::new(|_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: AgentSetup, _: Arc<Live>| Ok(empty_report()))
}

/// A stand-in for Steplet's own agent that notes it ran.
fn noting_agent(ran: Arc<std::sync::atomic::AtomicBool>) -> AgentFn {
    Arc::new(move |_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: AgentSetup, _: Arc<Live>| {
        ran.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(empty_report())
    })
}

/// A stand-in Claude Code run that notes it ran.
fn noting_run(ran: Arc<std::sync::atomic::AtomicBool>) -> RunFn {
    Arc::new(move |_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, _: Arc<Live>| {
        ran.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(empty_report())
    })
}

/// What the stand-in agent was given, and what the model it was given answered.
struct AgentSaw {
    model: String,
    cap: Nanodollars,
    max_steps: u32,
    secrets: Vec<String>,
    answer: Result<Option<String>, String>,
}

/// A stand-in for Steplet's own agent that asks its model one question and keeps what it was given.
fn asking_agent() -> (AgentFn, Arc<Mutex<Option<AgentSaw>>>) {
    let saw: Arc<Mutex<Option<AgentSaw>>> = Arc::default();
    let keep = saw.clone();
    let agent: AgentFn =
        Arc::new(move |_: &RunLock, _: &RunRequest, _: &RunPaths, _: &Policy, mut setup: AgentSetup, _: Arc<Live>| {
            let answer =
                setup.model.next(&[Message::user("Say hello.")], &[], 16).map(|r| r.text).map_err(|e| format!("{e:?}"));
            *keep.lock().unwrap() = Some(AgentSaw {
                model: setup.model.id().into(),
                cap: setup.cap,
                max_steps: setup.max_steps,
                secrets: setup.secrets.clone(),
                answer,
            });
            Ok(empty_report())
        });
    (agent, saw)
}

/// A stand-in Nebius on this computer: it answers one chat with "Hello from the stand-in." and keeps the request it
/// was sent, head and body, as text. No real call leaves this computer.
fn stand_in_nebius() -> (String, Arc<Mutex<Option<String>>>) {
    use std::io::{BufRead, BufReader, Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://127.0.0.1:{}/v1/", listener.local_addr().unwrap().port());
    let got: Arc<Mutex<Option<String>>> = Arc::default();
    let keep = got.clone();
    std::thread::spawn(move || {
        let Ok((stream, _)) = listener.accept() else { return };
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let (mut request, mut length) = (String::new(), 0);
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                length = value.trim().parse().unwrap();
            }
            request.push_str(&line);
            if line == "\r\n" {
                break;
            }
        }
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        request.push_str(&String::from_utf8(body).unwrap());
        *keep.lock().unwrap() = Some(request);
        let answer = json!({
            "id": "chatcmpl-1",
            "model": SUPER,
            "choices": [{ "index": 0, "message": { "role": "assistant", "content": "Hello from the stand-in." },
                          "finish_reason": "stop" }],
            "usage": { "prompt_tokens": 12, "completion_tokens": 5, "total_tokens": 17 }
        })
        .to_string();
        let mut out = stream;
        let _ = write!(out, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}", answer.len());
    });
    (base, got)
}

/// The value of the request's header `name`, as the stand-in Nebius saw it.
fn header<'a>(request: &'a str, name: &str) -> Option<&'a str> {
    let head = request.split("\r\n\r\n").next().unwrap_or_default();
    head.lines()
        .skip(1)
        .find_map(|l| l.split_once(':').filter(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.trim()))
}

/// The Nebius plan, Task 8: the runner chosen on the chooser is the one that runs. Steplet's agent gets the model
/// chosen, its spending cap and step limit, and the Nebius key the owner saved in Setup, read from Windows'
/// Credential Manager. The key goes only to Nebius (here a stand-in on this computer, so no real call) and into the
/// secrets the agent keeps out of the report and logs. Nothing else needs changing once the key is saved.
#[test]
fn start_run_passes_the_runner_and_model() {
    let _one = alone();
    let dir = scratch("own-runner");
    const ENTRY: &str = "Steplet-test/session-nebius-key";
    let entries = Entries { nebius: ENTRY, anthropic: "Steplet-test/session-no-anthropic-key" };
    let _forget = Forget(ENTRY);
    key::save(ENTRY, MADE_UP_NEBIUS_KEY).unwrap();
    let saved = setup::ready_for(&set_up(), &own(), entries).unwrap();
    assert_eq!(saved.distro_id, DISTRO);
    assert!(saved.nebius_key.as_ref().is_some_and(|k| k.is(MADE_UP_NEBIUS_KEY)));
    // Steplet's agent needs neither the gate program nor an Anthropic key.
    assert_eq!(saved.gate_exe, None);
    assert!(saved.key.is_none());

    let (base, request) = stand_in_nebius();
    let (agent, saw) = asking_agent();
    let claude_ran = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let s = session(noting_run(claude_ran.clone()), NOW, &dir).with_agent(agent).with_nebius_at(&base);
    let raw = read("examples/run-tests.steplet");
    let screen = Arc::new(Seen::default());
    assert_eq!(s.start(&raw, &sha(&raw), None, false, own(), Ok(saved), screen.clone()), Ok(()));
    wait_until("the run's end", || screen.ended() && !s.running());

    let saw = saw.lock().unwrap().take().expect("Steplet's agent did not run");
    assert_eq!(saw.model, SUPER);
    assert_eq!(saw.cap, AGENT_CAP);
    assert_eq!(AGENT_CAP, Nanodollars::from_cents(25));
    assert_eq!(saw.max_steps, MAX_TURNS);
    assert_eq!(MAX_TURNS, 40);
    assert_eq!(saw.secrets, [MADE_UP_NEBIUS_KEY]);
    assert_eq!(saw.answer, Ok(Some("Hello from the stand-in.".to_string())));
    assert!(!claude_ran.load(std::sync::atomic::Ordering::SeqCst), "Claude Code ran instead");
    let request = request.lock().unwrap().clone().expect("nothing reached the stand-in Nebius");
    assert!(request.starts_with("POST /v1/chat/completions "), "{request}");
    assert_eq!(header(&request, "authorization"), Some(format!("Bearer {MADE_UP_NEBIUS_KEY}").as_str()));
    let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["model"], SUPER);

    // Only the models on the list; Ultra has a price but is not offered yet.
    for model in ["gpt-4o", "nvidia/Nemotron-3-Ultra-550b-a55b"] {
        let other = Runner::Own { model: model.into() };
        assert_eq!(
            s.start(&raw, &sha(&raw), None, false, other, ready(), Arc::new(Seen::default())),
            Err("Steplet's agent does not offer this model. Choose one from the list.".to_string()),
            "{model}"
        );
    }
    let no_key = Ready { nebius_key: None, ..ready().unwrap() };
    assert_eq!(
        s.start(&raw, &sha(&raw), None, false, own(), Ok(no_key), Arc::new(Seen::default())),
        Err(NEEDS_NEBIUS.into())
    );
    key::forget(ENTRY).unwrap();
    assert_eq!(setup::ready_for(&set_up(), &own(), entries).map(|_| ()), Err(NEEDS_NEBIUS.into()));
}

/// The Nebius plan, Task 8: Claude Code still runs as before, with the gate program and the Anthropic key, and
/// Steplet's agent does not run in its place.
#[test]
fn the_claude_code_runner_still_starts() {
    let _one = alone();
    let dir = scratch("claude-runner");
    let seen: Arc<Mutex<Option<(String, bool, u32)>>> = Arc::default();
    let keep = seen.clone();
    let run: RunFn =
        Arc::new(move |_: &RunLock, req: &RunRequest, _: &RunPaths, _: &Policy, setup: &RunSetup, _: Arc<Live>| {
            let key = setup.key.as_ref().is_some_and(|k| k.is(MADE_UP_KEY));
            *keep.lock().unwrap() = Some((setup.gate_exe.clone(), key, req.max_turns));
            Ok(empty_report())
        });
    let agent_ran = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let s = session(run, NOW, &dir).with_agent(noting_agent(agent_ran.clone()));
    let raw = read("examples/run-tests.steplet");
    let screen = Arc::new(Seen::default());
    assert_eq!(s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, ready(), screen.clone()), Ok(()));
    wait_until("the run's end", || screen.ended() && !s.running());
    assert_eq!(*seen.lock().unwrap(), Some((GATE_EXE.to_string(), true, MAX_TURNS)));
    assert!(!agent_ran.load(std::sync::atomic::Ordering::SeqCst), "Steplet's agent ran instead");

    let no_key = Ready { key: None, ..ready().unwrap() };
    let refused = s.start(&raw, &sha(&raw), None, false, Runner::ClaudeCode, Ok(no_key), Arc::new(Seen::default()));
    assert_eq!(refused, Err(NEEDS_ANTHROPIC.into()));
    let entries =
        Entries { nebius: "Steplet-test/session-no-nebius-key", anthropic: "Steplet-test/session-no-anthropic-key" };
    assert_eq!(setup::ready_for(&set_up(), &Runner::ClaudeCode, entries).map(|_| ()), Err(NEEDS_ANTHROPIC.into()));
}
