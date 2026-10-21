//! The glue between the Setup screen and the runner's setup (Task 11): one install or removal at a time, never while
//! a file runs in any Steplet window, and the mark that one is going on is cleared however it ends, a crash of the
//! work included.

use std::sync::{Arc, Barrier, Mutex};

use steplet_app::setup::OneAtATime;
use steplet_app::views::{ANOTHER_WINDOW, failure};
use steplet_run::{RunError, RunLock};

/// Setup holds Windows' one-run lock while it works, so the tests take turns.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn alone() -> std::sync::MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn a_crashed_install_does_not_block_the_next_one() {
    let _one = alone();
    let one = OneAtATime::default();
    let crashed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        one.run(|| -> Result<(), _> { panic!("a made-up crash inside the install") })
    }));
    assert!(crashed.is_err(), "the crash reached the caller");
    assert_eq!(one.run(|| Ok(7)), Ok(7), "the next one runs");
    assert_eq!(one.run(|| Err::<(), _>(failure("made up", None))), Err(failure("made up", None)));
    assert_eq!(one.run(|| Ok(8)), Ok(8), "and after a failure too");
}

#[test]
fn a_second_install_while_one_runs_is_refused() {
    let _one = alone();
    let one = Arc::new(OneAtATime::default());
    let (started, finish) = (Arc::new(Barrier::new(2)), Arc::new(Barrier::new(2)));
    let first = {
        let (one, started, finish) = (one.clone(), started.clone(), finish.clone());
        std::thread::spawn(move || {
            one.run(|| {
                started.wait();
                finish.wait();
                Ok(1)
            })
        })
    };
    started.wait();
    let second = one.run(|| Ok(2)).expect_err("refused while the first runs");
    assert_eq!(second.reason, "Steplet is already changing its Linux. Wait for that to finish.");
    finish.wait();
    assert_eq!(first.join().unwrap(), Ok(1));
}

/// The final review: each double-clicked file opens in a Steplet program of its own (S9), so one window's Setup could
/// remove Steplet's Linux, or set it up again, under another window's run. Windows' one-run lock keeps them apart,
/// both ways.
#[test]
fn setup_waits_while_another_window_runs_a_file() {
    let _one = alone();
    // Another window's run holds the lock (the same Windows object, whichever program takes it).
    let run = RunLock::take().expect("no file runs when the test starts");
    let refused = OneAtATime::default()
        .run(|| -> Result<(), _> { panic!("Setup changed Steplet's Linux while a file ran") })
        .expect_err("refused while a file runs");
    assert_eq!(refused.reason, ANOTHER_WINDOW);
    drop(run);
    let a_run_meanwhile = OneAtATime::default().run(|| Ok(matches!(RunLock::take(), Err(RunError::Busy))));
    assert_eq!(a_run_meanwhile, Ok(true), "no file starts while Setup changes Steplet's Linux");
}
