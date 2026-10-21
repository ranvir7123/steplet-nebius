//! One run at a time on this computer (phase 4 Task 3): `prepare` empties `/work`, so a second run would delete the
//! first one's working copy. The lock is a named Windows object, so it holds across windows and programs.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{Live, RunError, RunLock, RunPaths, RunRequest, RunSetup, run_live};

/// The lock is one for the whole computer, so the tests that take it take turns.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

#[test]
fn a_second_run_cannot_start_while_one_is_running() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let first = RunLock::take().expect("no run holds the lock");
    assert!(matches!(RunLock::take(), Err(RunError::Busy)), "a second run, from the same program");
    let elsewhere = thread::spawn(|| matches!(RunLock::take(), Err(RunError::Busy))).join().unwrap();
    assert!(elsewhere, "a second run, from another thread");
    drop(first);
    let again = RunLock::take().expect("the lock is free again once the first run ends");
    drop(again);
}

/// Task 3's review: `run_live` takes the lock before anything else, so a second run is refused before it touches
/// Steplet's Linux (the distribution here is made up: anything that reached WSL would fail otherwise).
#[test]
fn run_live_is_refused_while_another_run_holds_the_lock() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n---\n";
    let s = parse(&canonicalize(f.as_bytes()).unwrap()).unwrap();
    let paths = RunPaths::new("{00000000-0000-0000-0000-000000000000}", "p4t3x").unwrap();
    let policy = Policy::from_steplet(&s, paths.work_repo()).unwrap();
    let req = RunRequest {
        steplet: s,
        repo_url: "https://github.com/octocat/Hello-World.git".into(),
        commit: "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d".into(),
        task_text: "made up".into(),
        max_turns: 1,
        deadline: Duration::from_secs(60),
        allow_model_switch: false,
    };
    let setup = RunSetup {
        gate_exe: "/opt/steplet/gate/steplet-gate.exe".into(),
        logs: std::env::temp_dir(),
        key: None,
        decision_deadline: Duration::from_secs(10),
        model_url: None,
    };
    let held = RunLock::take().expect("no run holds the lock");
    let got = run_live(&req, &paths, &policy, &setup, Arc::new(Live::unattended()));
    drop(held);
    assert!(matches!(got, Err(RunError::Busy)), "{got:?}");
}
