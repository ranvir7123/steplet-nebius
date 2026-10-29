//! A live run (phase 4 Task 3): what the screen is sent as the run goes, and the receiver's Stop. Each event is
//! cleaned exactly as the report cleans it before the screen sees it, so the live log is as safe to read as the report.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::agent::Spent;
use crate::ask::{Answers, Question, Reply};
use crate::prepare::Prepared;
use crate::report::Check;
use crate::types::Event;

/// The receiver's Stop button. Clones share one signal. Steplet also sets it when it stops a run itself (a tripwire,
/// the time limit) and when a run is over, so that every question still on screen is taken back at once.
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
    /// The run waits for the receiver's answer to this question (R7).
    Asking(Question),
    /// The question is closed, and how.
    Answered { question: u64, reply: Reply },
    /// Steplet decided the agent's call `call` (its id on Claude Code's stream): its check, as the report says it,
    /// and Steplet's reason when it refused. Said as the decision is made; the report, built afterwards from Steplet's
    /// own records, is the last word on what happened to the call.
    Checked { call: String, check: Check, reason: Option<String> },
    /// Steplet's own agent: what the run has spent on the model so far, after each answer.
    Spent(Spent),
}

/// Who follows a run as it goes: where its progress goes, and the Stop button. `on_progress` is called on the run's
/// own threads, the one that watches the agent among them, so it must only hand the progress on (to a queue, an
/// event) and return: while it runs, Steplet does not look at the Stop button or the time limit.
pub struct Live {
    pub on_progress: Box<dyn Fn(Progress) + Send + Sync>,
    pub stop: StopSignal,
    /// Where the receiver's answers come in; `None` when no one can be asked, and every "ask" is then a refusal.
    pub answers: Option<Answers>,
    /// How long a question waits for its answer; no answer by then is a no (S1).
    pub ask_limit: Duration,
}

impl Live {
    /// A run no one watches: progress goes nowhere, no one presses Stop, and no one can be asked (phase 3's runs, and
    /// the tests).
    pub fn unattended() -> Live {
        Live { on_progress: Box::new(|_| {}), stop: StopSignal::new(), answers: None, ask_limit: Duration::ZERO }
    }
}
