//! The live layer (phase 5, Task 8): the same attacks as the scripted layer, run with the real model instead of the
//! scripted stand-in. It costs money, so nothing runs without a spending cap: `cargo suite live` lists the live cases
//! and runs nothing, and only `--yes-spend <dollars>` lets the runs happen. The cap is checked before each case, so
//! the layer stops before it overspends. Each live case is one real run, at most a few turns; the real-model runs
//! themselves are the `#[ignore]` tests in `steplet-app/tests/suite_live.rs`, and the runner here starts them one at a
//! time, reads what each cost, and judges each from its tests' results exactly as the scripted layer does.

use std::collections::HashMap;
use std::time::Duration;

use crate::catalogue::Case;
use crate::verdict::{TestOutcome, Verdict, read_results, verdict};

/// The environment variable that carries the spending cap, in US dollars, to a live case. A live case that does not
/// find a cap here spends nothing: this is the one gate every live case checks before it starts a real run.
pub const CAP_VAR: &str = "STEPLET_SUITE_CAP_USD";

/// The cap in `value`, in US dollars: a number greater than zero, with any spaces around it trimmed. Everything else —
/// no value, an empty or blank value, words, zero, or less than zero — is no cap.
pub fn cap_from(value: Option<&str>) -> Option<f64> {
    value.and_then(|v| v.trim().parse::<f64>().ok()).filter(|c| c.is_finite() && *c > 0.0)
}

/// The cap in the environment (`STEPLET_SUITE_CAP_USD`), by the same rule as [`cap_from`]. `None` means no cap, so a
/// live case must spend nothing.
pub fn cap_from_env() -> Option<f64> {
    cap_from(std::env::var(CAP_VAR).ok().as_deref())
}

/// What a live run cost, in US dollars: the number on the last `suite: cost: <dollars>` line the case printed (it
/// reads it from Claude Code's own `total_cost_usd`). `None` when the case printed no cost — it did not start a run, or
/// did not reach a cost.
pub fn cost_of(output: &str) -> Option<f64> {
    output
        .lines()
        .filter_map(|l| l.trim().strip_prefix("suite: cost: "))
        .filter_map(|v| v.trim().parse::<f64>().ok())
        .next_back()
}

/// One real run of a live case, as the runner gets it back: everything its test program printed, how long it took, and
/// what the look after it found left in Steplet's Linux.
pub struct LiveRun {
    pub output: String,
    pub took: Duration,
    pub left: Vec<String>,
}

/// How one live case ended.
pub struct LiveResult {
    pub case: &'static Case,
    pub verdict: Verdict,
    /// What the run cost, in US dollars; `None` when the case was not started or printed no cost.
    pub cost: Option<f64>,
    /// Whether the case was started at all (a case stopped by the cap is not).
    pub started: bool,
    pub took: Duration,
    pub output: String,
    pub left: Vec<String>,
}

/// Runs `cases` in order with the real model, calling `run_one` to start each one and `row` as each finishes. Before
/// each case the money spent so far is checked against `cap`: once it reaches the cap, no further case is started —
/// each is shown as did-not-run, with the cap as the reason — so the layer never spends past the cap. A started case
/// is judged from its tests' results exactly as the scripted layer judges a case, and its cost is added to the total.
pub fn run_live_cases(
    cases: &[&'static Case],
    cap: f64,
    run_one: &mut dyn FnMut(&'static Case) -> LiveRun,
    row: &mut dyn FnMut(&LiveResult),
) -> Vec<LiveResult> {
    let mut spent = 0.0_f64;
    let mut results = Vec::new();
    for &case in cases {
        let result = if spent >= cap {
            let why =
                format!("the spending cap of ${cap:.2} was reached (${spent:.2} spent), so this case was not started");
            LiveResult {
                case,
                verdict: Verdict::DidNotRun { why },
                cost: None,
                started: false,
                took: Duration::ZERO,
                output: String::new(),
                left: Vec::new(),
            }
        } else {
            let run = run_one(case);
            let read = read_results(&run.output);
            let mine: HashMap<String, TestOutcome> =
                case.tests.iter().filter_map(|t| read.get(t.name).map(|o| (t.name.to_owned(), o.clone()))).collect();
            let cost = cost_of(&run.output);
            if let Some(c) = cost {
                spent += c;
            }
            LiveResult {
                case,
                verdict: verdict(case, &mine),
                cost,
                started: true,
                took: run.took,
                output: run.output,
                left: run.left,
            }
        };
        row(&result);
        results.push(result);
    }
    results
}
