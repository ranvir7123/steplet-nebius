//! A live run (phase 4 Task 3): what the screen is sent as the run goes, and the receiver's Stop. Each event is
//! cleaned exactly as the report cleans it before the screen sees it, so the live log is as safe to read as the report.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::prepare::Prepared;
use crate::types::Event;

/// The receiver's Stop button. Clones share one signal.
#[derive(Clone, Debug, Default)]
pub struct StopSignal(Arc<AtomicBool>);

impl StopSignal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stop(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_set(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// One piece of progress, for the screen. Every text in it is already cleaned (control and invisible characters
/// removed, the key and the run's token hidden).
#[derive(Clone, Debug, PartialEq)]
pub enum Progress {
    /// The working copy is ready, and these agent settings files were removed from it (D3).
    Prepared(Prepared),
    /// One event of the run, or Steplet's own record of a tool call that skipped the gate, in order.
    Event(Event),
}

/// Who follows a run as it goes: where its progress goes, and the Stop button.
pub struct Live {
    pub on_progress: Box<dyn Fn(Progress) + Send + Sync>,
    pub stop: StopSignal,
}

impl Live {
    /// A run no one watches: progress goes nowhere, and no one presses Stop (phase 3's runs, and the tests).
    pub fn unattended() -> Live {
        Live { on_progress: Box::new(|_| {}), stop: StopSignal::new() }
    }
}
