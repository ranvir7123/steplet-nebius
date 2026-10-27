//! The live layer (phase 5, Task 8), judged without Steplet's Linux and without money: the spending cap, the cost read
//! from a case's output, the cap stopping the run before it overspends, a case the model did not try, and the record
//! hiding the key. The real-model runs themselves are the `#[ignore]` tests in `steplet-app/tests/suite_live.rs`.

use std::time::Duration;

use steplet_suite::catalogue::{Case, Layer, cases_for};
use steplet_suite::live::{LiveRun, cap_from, cap_from_env, cost_of, run_live_cases};
use steplet_suite::record::keep;
use steplet_suite::runner::{MADE_UP_KEY, repo_root};
use steplet_suite::verdict::Verdict;

/// One test program's output when its single test passed and printed its cost, as the live case bodies do.
fn ran_and_cost(name: &str, cost: f64) -> String {
    format!(
        "test {name} ... ok\nsuite: cost: {cost}\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 \
         filtered out; finished in 0.10s\n"
    )
}

/// One test program's output when its test passed but the model did not take the path the case needed, so it printed
/// `suite: did not try:` (the test still ran every outside check, so Rust says `ok`).
fn did_not_try(name: &str, cost: f64) -> String {
    format!(
        "test {name} ... \nsuite: did not try: the model never asked to fetch\nsuite: cost: {cost}\nok\ntest result: \
         ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s\n"
    )
}

/// Without a cap in the environment there is no cap, and only a number greater than zero is one: this is the single
/// gate each live case checks before it spends anything.
#[test]
fn live_cases_spend_nothing_without_a_cap() {
    assert_eq!(cap_from(None), None, "no variable, no cap");
    assert_eq!(cap_from(Some("")), None, "an empty value is no cap");
    assert_eq!(cap_from(Some("   ")), None, "blank is no cap");
    assert_eq!(cap_from(Some("nonsense")), None, "words are no cap");
    assert_eq!(cap_from(Some("0")), None, "zero is no cap");
    assert_eq!(cap_from(Some("-1")), None, "less than zero is no cap");
    assert_eq!(cap_from(Some("2")), Some(2.0), "a number is a cap");
    assert_eq!(cap_from(Some(" 0.50 ")), Some(0.5), "spaces around a number are trimmed");

    // The live layer is never run through its command without --yes-spend, so the variable is unset here.
    assert!(
        std::env::var("STEPLET_SUITE_CAP_USD").is_err(),
        "this test assumes the suite's own tests run without a spending cap set"
    );
    assert_eq!(cap_from_env(), None, "with no variable set, there is no cap");
}

/// The cost of a run is the `suite: cost:` line a live case prints (read from Claude Code's own total); its absence
/// means no cost to add.
#[test]
fn the_cost_of_a_run_is_read_from_its_output() {
    assert_eq!(cost_of(&ran_and_cost("a_case", 0.07)), Some(0.07));
    assert_eq!(cost_of("test a_case ... ok\n"), None, "a run that printed no cost adds nothing");
    assert_eq!(cost_of("suite: cost: 0.1\nsuite: cost: 0.2\n"), Some(0.2), "the last cost printed is the one");
}

/// The cap is checked before each case: three runs costing 0.9, 1.2 and 1.1 against a cap of 2 start only the first
/// two; the third is never started, and its row says the cap was reached.
#[test]
fn the_live_layer_stops_at_the_spending_cap() {
    let scripted = cases_for(Layer::Scripted);
    let three: Vec<&'static Case> = scripted[..3].to_vec();
    let costs = [0.9, 1.2, 1.1];
    let mut call = 0usize;
    let mut run_one = |case: &'static Case| {
        let out = ran_and_cost(case.tests[0].name, costs[call]);
        call += 1;
        LiveRun { output: out, took: Duration::from_secs(1), left: Vec::new() }
    };
    let results = run_live_cases(&three, 2.0, &mut run_one, &mut |_| {});

    assert_eq!(call, 2, "only the first two cases were started");
    assert_eq!(results.len(), 3, "every case has a row, started or not");
    assert!(results[0].started && results[1].started, "the first two started");
    assert_eq!(results[0].verdict, Verdict::Blocked);
    assert_eq!(results[1].verdict, Verdict::Blocked);
    assert_eq!(results[0].cost, Some(0.9));
    assert_eq!(results[1].cost, Some(1.2));

    assert!(!results[2].started, "the third was not started");
    assert_eq!(results[2].cost, None, "a case that was not started cost nothing");
    match &results[2].verdict {
        Verdict::DidNotRun { why } => assert!(why.contains("cap"), "the row says the cap was reached: {why}"),
        other => panic!("the third case should be did-not-run because of the cap, not {other:?}"),
    }
}

/// A live case where the model did not take the path the case needed is shown as did-not-try, never as blocked, and
/// its cost still counts.
#[test]
fn a_live_case_the_model_did_not_try_is_shown_as_such() {
    let one: Vec<&'static Case> = cases_for(Layer::Scripted)[..1].to_vec();
    let mut run_one = |case: &'static Case| LiveRun {
        output: did_not_try(case.tests[0].name, 0.03),
        took: Duration::ZERO,
        left: Vec::new(),
    };
    let results = run_live_cases(&one, 2.0, &mut run_one, &mut |_| {});

    assert_eq!(results.len(), 1);
    assert!(results[0].started, "it was started");
    assert_eq!(results[0].cost, Some(0.03), "a did-not-try run still costs what it cost");
    match &results[0].verdict {
        Verdict::DidNotTry { .. } => {}
        other => panic!("a did-not-try case must be shown as did-not-try, not {other:?}"),
    }
    assert_ne!(results[0].verdict, Verdict::Blocked, "did-not-try is never blocked");
}

/// The live record is kept through the same `keep` the scripted layer uses, so a key that reaches it is counted and
/// hidden: a record with the made-up key planted is counted once and is not clean, and the kept file does not hold the
/// key's text.
#[test]
fn the_live_record_never_holds_the_key() {
    let dir = repo_root().join("target").join("live-record-test");
    let _ = std::fs::remove_dir_all(&dir);
    let text =
        format!("A live run printed a key it should not have: {MADE_UP_KEY} — and here it is again {MADE_UP_KEY}.");
    let kept = keep(&dir, "a-live-record.txt", &text, &[("the made-up key", MADE_UP_KEY)]).expect("the record is kept");

    assert_eq!(kept.counts, vec![("the made-up key".to_owned(), 2)], "the key's text was counted before keeping");
    assert!(!kept.clean(), "a record that held the key is not clean");
    let on_disk = std::fs::read_to_string(&kept.path).expect("the kept file reads back");
    assert_eq!(on_disk.matches(MADE_UP_KEY).count(), 0, "the kept file does not hold the key's text");
    assert!(on_disk.contains("[hidden by Steplet]"), "the key's place is marked hidden");
    let _ = std::fs::remove_dir_all(&dir);
}
