//! A run from the app (Task 12), with a stand-in for the runner, so no Linux is needed: the file is checked again when
//! Run is pressed (row 30), one run at a time, each question reaches the screen on its own with the taskbar flashing,
//! the receiver's answer goes to the question it answers, closing asks first, a run a crash left is cleared at the
//! next start, and the senders are written when a run starts, never before.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::json;
use sha2::{Digest, Sha256};
use steplet_app::pins::{self, Pins};
use steplet_app::session::{Places, Ready, RunFn, Screen, Session};
use steplet_app::views::{ProgressView, RED_SENTENCE};
use steplet_core::policy::Policy;
use steplet_run::{
    ApiKey, Asker, GateAnswer, GateRequest, Live, Report, RunError, RunLock, RunOutcome, RunPaths, RunRequest,
    RunSetup, Trust, build_report,
};

/// The corpus's own time of viewing: 11 Oct 2026. The example files expire at the end of 2026-12-31 or later.
const NOW: i64 = 1_791_720_000;
const MADE_UP_KEY: &str = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";
const DISTRO: &str = "{44f36236-99b1-42c0-84bd-7ff8392d51ba}";
/// The tests share Windows' one-run lock, so they run one at a time.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn alone() -> std::sync::MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner())
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
        gate_exe: "/opt/steplet/gate/steplet-gate-0123456789abcdef.exe".into(),
        key: ApiKey::new(MADE_UP_KEY.into()),
    })
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

fn empty_report() -> Report {
    let o = RunOutcome {
        events: Vec::new(),
        requests: Vec::new(),
        stopped_by: None,
        stopped_at: None,
        exit_code: Some(0),
        faults: Vec::new(),
        stopped_by_you: false,
        answers: Vec::new(),
    };
    build_report(&o, Some(&[]), Trust::NotRecorded, &[])
}

/// A stand-in run that ends at once.
fn quick() -> RunFn {
    Arc::new(|_: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, _: Arc<Live>| Ok(empty_report()))
}

/// A stand-in run that waits until Stop is pressed.
fn until_stopped() -> RunFn {
    Arc::new(|_: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, live: Arc<Live>| {
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
    Session::new(run, Arc::new(move || now), places(dir))
}

#[test]
fn start_run_checks_the_file_again_now() {
    let _one = alone();
    let dir = scratch("expired");
    let raw = read("examples/run-tests.steplet");
    // The example expires at the end of 2027-09-30; it was previewed before that, and Run is pressed after.
    let after_expiry = 1_822_521_600 + 2 * 86_400;
    let refused =
        session(quick(), after_expiry, &dir).start(&raw, &sha(&raw), None, ready(), Arc::new(Seen::default()));
    assert_eq!(refused, Err("This file expired after you opened it.".to_string()));
}

#[test]
fn start_run_refuses_bytes_that_changed() {
    let _one = alone();
    let dir = scratch("changed");
    let raw = read("examples/run-tests.steplet");
    let mut changed = raw.clone();
    changed.extend_from_slice(b"\nOne more line the sender did not write.\n");
    let refused = session(quick(), NOW, &dir).start(&changed, &sha(&raw), None, ready(), Arc::new(Seen::default()));
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
    let s = session(quick(), NOW, &dir);
    let start = |typed: Option<&str>| s.start(&raw, &sha(&raw), typed, ready(), Arc::new(Seen::default()));
    let (wrong, dotted) = (RED_SENTENCE.replace("risks", "risk"), format!("{RED_SENTENCE}."));
    for typed in [None, Some(wrong.as_str()), Some(dotted.as_str())] {
        assert_eq!(
            start(typed),
            Err("This file is red: type the sentence exactly before you run it.".to_string()),
            "{typed:?}"
        );
    }
    let screen = Arc::new(Seen::default());
    assert_eq!(s.start(&raw, &sha(&raw), Some(RED_SENTENCE), ready(), screen.clone()), Ok(()));
    wait_until("the run's end", || screen.ended());
}

#[test]
fn run_is_refused_while_another_window_runs() {
    let _one = alone();
    let dir = scratch("busy");
    let raw = read("examples/run-tests.steplet");
    let _other_window = RunLock::take().expect("no run here yet");
    let refused = session(quick(), NOW, &dir).start(&raw, &sha(&raw), None, ready(), Arc::new(Seen::default()));
    assert_eq!(refused, Err("Another Steplet window is running a file. Finish or stop it first.".to_string()));
}

#[test]
fn an_answer_reaches_the_runner_bound_to_its_question() {
    let _one = alone();
    let dir = scratch("answer");
    let raw = read("examples/run-tests.steplet");
    let decided: Arc<Mutex<Vec<GateAnswer>>> = Arc::default();
    let keep = decided.clone();
    let run: RunFn = Arc::new(move |_: &RunRequest, _: &RunPaths, policy: &Policy, _: &RunSetup, live: Arc<Live>| {
        let asker = Asker::new(live, &[]);
        for (command, id) in [("touch first.txt", "toolu_01"), ("touch second.txt", "toolu_02")] {
            let answer = asker.decide(policy, &bash(command, id));
            keep.lock().unwrap().push(answer);
        }
        Ok(empty_report())
    });
    let screen = Arc::new(Seen::default());
    let s = session(run, NOW, &dir);
    s.start(&raw, &sha(&raw), None, ready(), screen.clone()).unwrap();
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
    let run: RunFn = Arc::new(|_: &RunRequest, _: &RunPaths, policy: &Policy, _: &RunSetup, live: Arc<Live>| {
        let asker = Arc::new(Asker::new(live, &[]));
        let second = {
            let (asker, policy) = (asker.clone(), policy.clone());
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(100));
                asker.decide(&policy, &bash("touch second.txt", "toolu_02"))
            })
        };
        asker.decide(policy, &bash("touch first.txt", "toolu_01"));
        second.join().unwrap();
        Ok(empty_report())
    });
    let screen = Arc::new(Seen::default());
    let s = session(run, NOW, &dir);
    s.start(&raw, &sha(&raw), None, ready(), screen.clone()).unwrap();
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
    s.start(&raw, &sha(&raw), None, ready(), screen.clone()).unwrap();
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
    let cleared: Arc<Mutex<Vec<String>>> = Arc::default();
    let keep = cleared.clone();
    let note = Session::clear_leftover(&p, &move |paths: &RunPaths| {
        keep.lock().unwrap().push(format!("{} {}", paths.distro_id(), paths.run_id()));
        Ok(())
    });
    assert_eq!(*cleared.lock().unwrap(), [format!("{DISTRO} app-1791720000-77")], "that run was stopped and removed");
    assert_eq!(
        note.as_deref(),
        Some(
            "Steplet was closed while a file was running. It stopped what was left of that run and removed its \
             folders. Nothing of it is left."
        )
    );
    assert!(!p.marker.exists(), "the marker is gone");
    assert_eq!(Session::clear_leftover(&p, &|_: &RunPaths| Ok(())), None, "nothing left the next time");
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
    Session::new(quick(), Arc::new(|| NOW), p).start(&raw, &sha(&raw), None, ready(), screen.clone()).unwrap();
    wait_until("the run's end", || screen.ended());
    let Pins::Known(known) = pins::load(&pins_path) else { panic!("the pins file reads") };
    assert_eq!(known.iter().map(|k| (k.sender.as_str(), k.runs)).collect::<Vec<_>>(), [("Steplet test vectors", 1)]);
}

#[test]
fn a_run_that_cannot_start_says_why_in_words() {
    let _one = alone();
    let dir = scratch("cannot");
    let raw = read("examples/run-tests.steplet");
    let run: RunFn =
        Arc::new(|_: &RunRequest, _: &RunPaths, _: &Policy, _: &RunSetup, _: Arc<Live>| Err(RunError::Busy));
    let screen = Arc::new(Seen::default());
    session(run, NOW, &dir).start(&raw, &sha(&raw), None, ready(), screen.clone()).unwrap();
    wait_until("the run's end", || screen.ended());
    let sent = screen.sent.lock().unwrap();
    let Some(ProgressView::Failed { failure, .. }) = sent.last() else { panic!("{sent:?}") };
    assert_eq!(failure.reason, "Another Steplet window is running a file. Finish or stop it first.");
    assert!(!failure.reason.contains("Busy") && !failure.reason.contains('{'), "words, not code");
}

#[test]
fn a_call_that_skipped_the_gate_reaches_the_screen_with_its_check() {
    use steplet_app::session::{ASK_LIMIT, RUN_LIMIT, Relay};
    use steplet_app::views::Limits;
    use steplet_run::{Event, Progress};
    let mut relay = Relay::new(Limits { ask: ASK_LIMIT, run: RUN_LIMIT });
    let skipped = |harmless| {
        Progress::Event(Event::SkippedGate {
            id: "toolu_s".into(),
            tool: "Bash".into(),
            what: "cat README".into(),
            harmless,
        })
    };
    for (harmless, kind, words) in [
        (true, "not_asked", "Ran without asking Steplet: a harmless read"),
        (false, "ran_without_asking", "Ran without asking Steplet: Steplet stopped the run"),
    ] {
        let views = relay.relay(&skipped(harmless)).views;
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
    let s = Session::new(quick(), Arc::new(|| NOW), p);
    assert_eq!(
        s.start(&raw, &sha(&raw), None, ready(), Arc::new(Seen::default())),
        Err("This file is red: type the sentence exactly before you run it.".to_string())
    );
    let Pins::Known(known) = pins::load(&pins_path) else { panic!("the pins file reads") };
    assert_eq!(known.iter().map(|k| k.key.as_str()).collect::<Vec<_>>(), [OTHER_KEY], "the old pin stays");
    let screen = Arc::new(Seen::default());
    assert_eq!(s.start(&raw, &sha(&raw), Some(RED_SENTENCE), ready(), screen.clone()), Ok(()));
    wait_until("the run's end", || screen.ended());
}
