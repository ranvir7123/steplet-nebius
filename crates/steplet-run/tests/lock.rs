//! One run at a time on this computer (phase 4 Task 3): `prepare` empties `/work`, so a second run would delete the
//! first one's working copy. The lock is a named Windows object, so it holds across windows and programs.

use std::thread;

use steplet_run::{RunError, RunLock};

#[test]
fn a_second_run_cannot_start_while_one_is_running() {
    let first = RunLock::take().expect("no run holds the lock");
    assert!(matches!(RunLock::take(), Err(RunError::Busy)), "a second run, from the same program");
    let elsewhere = thread::spawn(|| matches!(RunLock::take(), Err(RunError::Busy))).join().unwrap();
    assert!(elsewhere, "a second run, from another thread");
    drop(first);
    let again = RunLock::take().expect("the lock is free again once the first run ends");
    drop(again);
}
