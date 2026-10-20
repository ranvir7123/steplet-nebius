//! The glue between the Setup screen and the runner's setup (Task 11): one install or removal at a time, and the mark
//! that one is going on is cleared however it ends, a crash of the work included.

use std::sync::{Arc, Barrier};

use steplet_app::setup::OneAtATime;
use steplet_app::views::failure;

#[test]
fn a_crashed_install_does_not_block_the_next_one() {
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
