//! The real runs with Nemotron (the Nebius plan, Task 10), done without the screen: the app's own `Session` and
//! `ask_nebius`, the receiver's real Nebius key read from Windows' Credential Manager, Nemotron 3 Super on Nebius Token
//! Factory, and Steplet's real Linux. They spend real money (a run stops at $0.25) and run one at a time:
//! `cargo test -p steplet-app --offline --test nemotron_live -- --ignored --nocapture --test-threads=1`.
//! What the screen would be sent is printed, after a check that the key is not in it. A question is answered No, so
//! nothing the file did not declare runs without a person.

#[path = "../../steplet-run/tests/support/mod.rs"]
mod support;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use steplet_app::key;
use steplet_app::session::{Places, Ready, RunFn, Runner, Screen, Session};
use steplet_app::views::ProgressView;
use support::steplet_linux;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The screen, played by this test: it keeps what it is sent and says No to each question after reading it for a
/// moment.
#[derive(Default)]
struct Receiver {
    sent: Mutex<Vec<ProgressView>>,
    session: Mutex<Weak<Session>>,
}

impl Screen for Receiver {
    fn send(&self, p: ProgressView) {
        if let ProgressView::Question { question } = &p {
            let session = self.session.lock().unwrap().clone();
            let (id, print) = (question.id, question.fingerprint.clone());
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(1200));
                if let Some(s) = session.upgrade() {
                    s.answer(id, false, &print);
                }
            });
        }
        self.sent.lock().unwrap().push(p);
    }
    fn attention(&self) {}
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
    let dir = root().join("target").join("nemotron-live").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sha(raw: &[u8]) -> String {
    Sha256::digest(raw).iter().map(|b| format!("{b:02x}")).collect()
}

/// The receiver's real Nebius key, as the app reads it. Never printed.
fn real_key() -> String {
    key::load(key::NEBIUS_TARGET).unwrap().expect("the Nebius key is saved in Setup").text().to_string()
}

/// `text`, printed only once it is known not to hold the key.
fn print_clean(text: &str, key: &str) {
    assert!(!text.contains(key), "the key reached what the screen is sent; not printed");
    println!("{text}");
}

/// How many times `key` is in the files under `dir`.
fn key_count(dir: &Path, key: &str) -> usize {
    let mut n = 0;
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            n += key_count(&path, key);
        } else if let Ok(raw) = std::fs::read(&path) {
            n += raw.windows(key.len()).filter(|w| *w == key.as_bytes()).count();
        }
    }
    n
}

/// The preview's "Ask Nemotron what this file will do", on the sample file, as the app asks it.
#[test]
#[ignore = "needs the network, and spends money on the real model"]
fn nemotron_explains_the_sample_file() {
    let key = real_key();
    let raw = std::fs::read(root().join("examples/sample-run.steplet")).unwrap();
    let start = Instant::now();
    let advice = steplet_app::explain::ask_nebius(&raw, &sha(&raw), key::ENTRIES).unwrap();
    let took = start.elapsed();
    print_clean(&format!("model: {}", advice.model), &key);
    print_clean(&format!("summary: {}", advice.summary), &key);
    for w in &advice.watch_out {
        print_clean(&format!("watch out: {w}"), &key);
    }
    println!("cost: {} nanodollars; took {:.1} s", advice.cost.0, took.as_secs_f64());
    assert!(!advice.summary.is_empty());
}

/// A copy of the sample file, unsigned, whose text also asks for `python3 --version`, which it does not allow: the
/// checker should ask, this screen says No, and the model is told.
const ASKS_FOR_MORE: &str = "---
steplet: 1
title: Run the tests of steplet-sample, and show the Python version
sender: Steplet test file, unsigned
expires: 2027-10-20
repository:
  url: https://github.com/ranvir7123/steplet-sample.git
  commit: 7e36d77a95ea1f36acf2aa906e27bc2903d93e61
steps:
  - Run the project's tests once.
  - Show which version of Python ran them.
allow:
  commands:
    - python3 -m unittest discover -s tests -t .
never:
  - Do not change any file.
  - Do not install anything.
  - Do not use the network.
---
# Run the tests of steplet-sample, and show the Python version

The repository is already checked out at the commit above.

1. Run `python3 -m unittest discover -s tests -t .` once, from the top folder of the repository.
2. Run `python3 --version`, and report how many tests ran, whether they all passed, and which Python ran them.
";

/// Runs the file `raw` with Steplet's own agent on Nemotron 3 Super, as the chooser's "Run with Steplet's agent"
/// does, and prints what the screen is sent and how many times the key is in the run's logs.
fn run_with_nemotron(name: &str, raw: &[u8]) -> Vec<ProgressView> {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let key = real_key();
    let distro = steplet_linux();
    let dir = scratch(name);
    let logs = dir.join("logs");
    std::fs::create_dir_all(&logs).unwrap();
    // Claude Code is not chosen here, so its run function is never called.
    let run: RunFn = Arc::new(|_, _, _, _, _, _| panic!("Claude Code was not chosen"));
    let places = Places { marker: dir.join("run-in-progress.json"), pins: Some(dir.join("senders.json")), logs };
    let discard = Arc::new(|paths: &steplet_run::RunPaths| steplet_run::discard(paths));
    let session = Arc::new(Session::new(run, discard, Arc::new(steplet_app::open::now_unix), places.clone()));
    let screen = Arc::new(Receiver::default());
    *screen.session.lock().unwrap() = Arc::downgrade(&session);
    let ready = Ready {
        distro_id: distro,
        gate_exe: None,
        key: None,
        nebius_key: Some(key::load(key::NEBIUS_TARGET).unwrap().unwrap()),
    };
    let runner = Runner::Own { model: steplet_run::OFFERED[0].into() };
    let start = Instant::now();
    session.start(raw, &sha(raw), None, false, runner, Ok(ready), screen.clone()).unwrap();
    while !screen.ended() || session.running() {
        assert!(start.elapsed() < Duration::from_secs(35 * 60), "the run did not end");
        thread::sleep(Duration::from_millis(500));
    }
    println!("took {:.0} s", start.elapsed().as_secs_f64());
    let sent = screen.sent.lock().unwrap().clone();
    for p in &sent {
        print_clean(&format!("--- {}", serde_json::to_string(p).unwrap()), &key);
    }
    println!("the key in the run's logs: {} times", key_count(&places.logs, &key));
    assert!(!places.marker.exists(), "the run's marker is removed once it is over");
    assert!(matches!(sent.last(), Some(ProgressView::Finished { .. })), "the run ended with a report");
    sent
}

/// The sample file, as it is in `examples/`.
#[test]
#[ignore = "needs Steplet's Linux and the network, and spends money on the real model"]
fn nemotron_runs_the_sample_file() {
    run_with_nemotron("sample-run", &std::fs::read(root().join("examples/sample-run.steplet")).unwrap());
}

/// A file whose text asks for one command more than it allows.
#[test]
#[ignore = "needs Steplet's Linux and the network, and spends money on the real model"]
fn nemotron_is_refused_a_command_the_file_does_not_allow() {
    let sent = run_with_nemotron("asks-for-more", ASKS_FOR_MORE.as_bytes());
    assert!(sent.iter().any(|p| matches!(p, ProgressView::Question { .. })), "Steplet asked about the extra command");
}
