//! How a case is judged (phase 5, section 3.3), from Rust's own test output: a case is blocked only if every one of
//! its tests said `ok`. A test that could not start its case says so (`suite: did not run: <why>`, from
//! `support::did_not_run`), and a live case the model did not try says `suite: did not try: <why>`; neither is ever
//! counted as blocked.

use std::collections::HashMap;

use crate::catalogue::{Case, Line};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Blocked,
    NotBlocked { failed: Vec<String> },
    DidNotRun { why: String },
    DidNotTry { why: String },
}

/// One test's result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestOutcome {
    Ok,
    Failed,
    Ignored,
    DidNotRun(String),
    DidNotTry(String),
}

/// Reads one test program's output: Rust's `test <name> ... ok|FAILED|ignored` lines and Steplet's `suite: …` lines.
/// With `--nocapture` a test's own lines come between its name and its result; without it they come after, under
/// `---- <name> stdout ----`; while a test runs, a line it prints that looks like a test's line is its own output. A
/// test named in the list of failures at the end failed, whatever line said otherwise. Nothing counts unless the program
/// ran to its end (Rust's last line, `test result: …`, once) and the results read add up to the counts in that line: a
/// program that crashed, or a test whose own lines read like results, gives no result at all.
pub fn read_results(output: &str) -> HashMap<String, TestOutcome> {
    let mut results: HashMap<String, TestOutcome> = HashMap::new();
    let mut notes: Vec<(String, TestOutcome)> = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    // The test whose name was printed and whose result has not come yet, and the test whose kept output is shown.
    let mut pending: Option<String> = None;
    let mut shown: Option<String> = None;
    let mut in_failures = false;
    let mut counted: Vec<Option<[usize; 3]>> = Vec::new();
    for line in output.lines().map(|l| l.trim_end_matches('\r')) {
        if let Some((name, after)) = test_line(line).filter(|_| pending.is_none()) {
            shown = None;
            match result_word(after) {
                Some(outcome) => {
                    results.insert(name.to_owned(), outcome);
                    pending = None;
                }
                None => {
                    note(after, name, &mut notes);
                    pending = Some(name.to_owned());
                }
            }
        } else if let Some(outcome) = result_word(line).filter(|_| pending.is_some()) {
            results.insert(pending.take().unwrap(), outcome);
        } else if let Some(name) = line.strip_prefix("---- ").and_then(|l| l.strip_suffix(" stdout ----")) {
            shown = Some(name.to_owned());
        } else if line == "failures:" || line == "successes:" {
            in_failures = line == "failures:";
            shown = None;
        } else if let Some(rest) = line.strip_prefix("test result: ") {
            in_failures = false;
            shown = None;
            counted.push(counts(rest));
        } else if let Some(name) = line.strip_prefix("    ").filter(|n| in_failures && is_test_name(n)) {
            failures.push(name.to_owned());
        } else if let Some(name) = shown.as_ref().or(pending.as_ref()) {
            note(line, &name.clone(), &mut notes);
        }
    }
    for name in failures {
        if results.contains_key(&name) {
            results.insert(name, TestOutcome::Failed);
        }
    }
    let read = [TestOutcome::Ok, TestOutcome::Failed, TestOutcome::Ignored]
        .map(|o| results.values().filter(|r| **r == o).count());
    if counted != [Some(read)] {
        return HashMap::new();
    }
    for (name, note) in notes {
        match (note, results.get(&name)) {
            (TestOutcome::DidNotRun(why), _) => {
                results.insert(name, TestOutcome::DidNotRun(why));
            }
            (TestOutcome::DidNotTry(why), Some(TestOutcome::Ok)) => {
                results.insert(name, TestOutcome::DidNotTry(why));
            }
            _ => {}
        }
    }
    results
}

/// The passed, failed and ignored counts in Rust's last line, after `test result: `.
fn counts(rest: &str) -> Option<[usize; 3]> {
    let (_, numbers) = rest.split_once(". ")?;
    let mut found = [None; 3];
    for part in numbers.split("; ") {
        let (n, what) = part.split_once(' ')?;
        if let Some(i) = ["passed", "failed", "ignored"].iter().position(|w| *w == what) {
            found[i] = n.parse().ok();
        }
    }
    Some([found[0]?, found[1]?, found[2]?])
}

/// `test <name> ... <rest>`: the name, and what follows it on the line.
fn test_line(line: &str) -> Option<(&str, &str)> {
    let (name, rest) = line.strip_prefix("test ")?.split_once(" ...")?;
    is_test_name(name).then(|| (name, rest.strip_prefix(' ').unwrap_or(rest)))
}

fn is_test_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')
}

fn result_word(text: &str) -> Option<TestOutcome> {
    match text {
        "ok" => Some(TestOutcome::Ok),
        "FAILED" => Some(TestOutcome::Failed),
        t if t == "ignored" || t.starts_with("ignored, ") => Some(TestOutcome::Ignored),
        _ => None,
    }
}

/// A `suite: …` line a test printed, kept for `name`.
fn note(line: &str, name: &str, notes: &mut Vec<(String, TestOutcome)>) {
    let line = line.trim();
    if let Some(why) = line.strip_prefix("suite: did not run: ") {
        notes.push((name.to_owned(), TestOutcome::DidNotRun(why.trim().to_owned())));
    } else if let Some(why) = line.strip_prefix("suite: did not try: ") {
        notes.push((name.to_owned(), TestOutcome::DidNotTry(why.trim().to_owned())));
    }
}

/// The case's verdict from its tests' results. A failure is what to see first; then a test that could not run, was
/// set aside, or gave no result; then a live case the model did not try. Only every test `ok` is blocked.
pub fn verdict(case: &Case, results: &HashMap<String, TestOutcome>) -> Verdict {
    let mut failed = Vec::new();
    let mut not_run = None;
    let mut not_tried = None;
    for t in case.tests {
        match results.get(t.name) {
            Some(TestOutcome::Ok) => {}
            Some(TestOutcome::Failed) => failed.push(t.name.to_owned()),
            Some(TestOutcome::DidNotRun(why)) => {
                not_run.get_or_insert_with(|| why.clone());
            }
            Some(TestOutcome::Ignored) => {
                not_run.get_or_insert_with(|| format!("{} was set aside (ignored)", t.name));
            }
            None => {
                not_run.get_or_insert_with(|| format!("{} gave no result", t.name));
            }
            Some(TestOutcome::DidNotTry(why)) => {
                not_tried.get_or_insert_with(|| why.clone());
            }
        }
    }
    match (failed.is_empty(), not_run, not_tried) {
        (false, _, _) => Verdict::NotBlocked { failed },
        (true, Some(why), _) => Verdict::DidNotRun { why },
        (true, None, Some(why)) => Verdict::DidNotTry { why },
        (true, None, None) => Verdict::Blocked,
    }
}

/// A line on screen is backed when at least one case naming it was run, and every case naming it was blocked.
pub fn line_backed(line: Line, verdicts: &[(&Case, Verdict)]) -> bool {
    let mut naming = verdicts.iter().filter(|(c, _)| c.lines.contains(&line)).peekable();
    naming.peek().is_some() && naming.all(|(_, v)| *v == Verdict::Blocked)
}
