//! A session's real runs (Task 12): the app's own `Session`, the runner's `run_live`, Steplet's Linux and the scripted
//! model (the runner's test helpers, shared through `#[path]`); the screen is played by this test program. They need
//! Steplet's Linux and the network, and run one at a time:
//! `cargo test -p steplet-app --offline --test session_live -- --ignored --nocapture --test-threads=1`.

#[path = "../../steplet-run/tests/support/mod.rs"]
mod support;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use steplet_app::session::{Places, Ready, RunFn, Screen, Session};
use steplet_app::views::{ProgressView, Replies};
use steplet_core::policy::Policy;
use steplet_run::{ApiKey, Live, RunLock, RunPaths, RunRequest, RunSetup};
use support::{MADE_UP_KEY, scripted_model, steplet_linux};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
const THE_END: &str = "Task 12: the end of the script.";

/// The screen, played by this test: it keeps what it is sent, counts the taskbar's flashes, and says yes to each
/// question after reading it for a moment (longer than the second the Yes button waits).
#[derive(Default)]
struct Receiver {
    sent: Mutex<Vec<ProgressView>>,
    session: Mutex<Weak<Session>>,
    flashes: Mutex<u32>,
}

impl Screen for Receiver {
    fn send(&self, p: ProgressView) {
        if let ProgressView::Question { question } = &p {
            let session = self.session.lock().unwrap().clone();
            let (id, print) = (question.id, question.fingerprint.clone());
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(1200));
                if let Some(s) = session.upgrade() {
                    assert!(s.answer(id, true, &print), "the question was open");
                }
            });
        }
        self.sent.lock().unwrap().push(p);
    }
    fn attention(&self) {
        *self.flashes.lock().unwrap() += 1;
    }
    fn close(&self) {}
}

impl Receiver {
    fn ended(&self) -> bool {
        self.sent
            .lock()
            .unwrap()
            .iter()
            .any(|p| matches!(p, ProgressView::Finished { .. } | ProgressView::Failed { .. }))
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = root().join("target").join("session-live").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sha(raw: &[u8]) -> String {
    Sha256::digest(raw).iter().map(|b| format!("{b:02x}")).collect()
}

/// Runs the corpus's example file (Hello-World, `wc -l README` allowed) through a session, as the app does, with the
/// scripted model following `script` on `port`, and `key` as the receiver's key. Returns what the screen was sent.
fn run_session(name: &str, script: &Value, port: u16, key: &str) -> Arc<Receiver> {
    let distro = steplet_linux();
    let _model = scripted_model(&distro, name, &script.to_string(), port);
    let url = format!("http://127.0.0.1:{port}");
    // The app's own run function, pointed at the scripted model instead of Anthropic.
    let run: RunFn = Arc::new(
        move |held: &RunLock,
              req: &RunRequest,
              paths: &RunPaths,
              policy: &Policy,
              setup: &RunSetup,
              live: Arc<Live>| {
            let setup = RunSetup { model_url: Some(url.clone()), ..setup.clone() };
            steplet_run::run_live_locked(held, req, paths, policy, &setup, live)
        },
    );
    let dir = scratch(name);
    let places = Places {
        marker: dir.join("run-in-progress.json"),
        pins: Some(dir.join("senders.json")),
        logs: support::logs_dir(),
    };
    let discard = Arc::new(|paths: &RunPaths| steplet_run::discard(paths));
    let session = Arc::new(Session::new(run, discard, Arc::new(steplet_app::open::now_unix), places.clone()));
    let screen = Arc::new(Receiver::default());
    *screen.session.lock().unwrap() = Arc::downgrade(&session);
    let raw = std::fs::read(root().join("examples/run-tests.steplet")).unwrap();
    let ready =
        Ready { distro_id: distro.clone(), gate_exe: support::install_gate(&distro), key: ApiKey::new(key.into()) };
    session.start(&raw, &sha(&raw), None, Ok(ready), screen.clone()).unwrap();
    let start = Instant::now();
    while !screen.ended() || session.running() {
        assert!(start.elapsed() < Duration::from_secs(300), "the run did not end");
        thread::sleep(Duration::from_millis(200));
    }
    assert!(!places.marker.exists(), "the run's marker is removed once it is over");
    screen
}

/// Use case 1 in the app's own session: the agent runs the declared command, then one the file does not ask for; the
/// screen gets the question once (with one flash of the taskbar), says yes, and the run finishes.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_session_runs_use_case_1_with_the_scripted_model_and_asks_once() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": "wc -l README", "description": "Count the lines"}},
        {"tool": "Bash", "input": {"command": "touch asked.txt", "description": "Make a file"}},
        {"text": THE_END}]});
    let screen = run_session("p4t12a", &script, 47970, MADE_UP_KEY);
    let sent = screen.sent.lock().unwrap();
    for p in sent.iter() {
        println!("--- {}", serde_json::to_string(p).unwrap());
    }
    assert!(matches!(sent.first(), Some(ProgressView::Prepared { .. })), "prepared first");
    let questions: Vec<_> = sent.iter().filter(|p| matches!(p, ProgressView::Question { .. })).collect();
    assert_eq!(questions.len(), 1, "one question");
    assert_eq!(*screen.flashes.lock().unwrap(), 1, "one flash of the taskbar");
    let closed: Vec<&str> = sent
        .iter()
        .filter_map(|p| match p {
            ProgressView::QuestionClosed { how, .. } => Some(how.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(closed, [Replies::default().yes.as_str()]);
    let calls = sent.iter().filter(|p| matches!(p, ProgressView::Event { event } if matches!(event, steplet_app::views::EventView::Call { .. }))).count();
    assert_eq!(calls, 2, "both commands reached the screen");
    let Some(ProgressView::Finished { report }) = sent.last() else { panic!("the run finished: {:?}", sent.last()) };
    assert!(report.completed, "{report:?}");
}

/// A key Anthropic does not accept (the scripted model answers HTTP 401, as Anthropic does). Measured first: what
/// Claude Code 2.1.220 puts on its stream then is printed here.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_wrong_key_says_so() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = json!({"steps": [{"text": THE_END}]});
    let screen = run_session("p4t12b", &script, 47971, "sk-ant-api03-STEPLET-TEST-WRONG-MADE-UP-NOT-A-KEY");
    let sent = screen.sent.lock().unwrap();
    for p in sent.iter() {
        println!("--- {}", serde_json::to_string(p).unwrap());
    }
    let Some(ProgressView::Finished { report }) = sent.last() else { panic!("the run ended: {:?}", sent.last()) };
    let faults: Vec<String> =
        report.faults.iter().map(|f| f.iter().map(|s| format!("{s:?}")).collect::<String>()).collect();
    assert!(faults.iter().any(|f| f.contains("Anthropic did not accept the key. Change it in Setup.")), "{faults:?}");
    assert!(!report.completed);
}
