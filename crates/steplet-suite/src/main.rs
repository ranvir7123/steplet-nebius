//! `cargo suite list | scripted | live [--only <ids>] [--runner <agent>]` (phase 5, section 1): the attack suite's one
//! command.

use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

use steplet_run::setup::CLAUDE_VERSION;
use steplet_suite::catalogue::{CASES, Case, Layer, Line, Needs, Runner, cases_for, for_runner, select};
use steplet_suite::live::{CAP_VAR, LiveResult, LiveRun, cap_from, run_live_cases};
use steplet_suite::record::{keep, now, record_name};
use steplet_suite::runner::{
    CaseResult, MADE_UP_KEY, MADE_UP_NEBIUS_KEY, Machine, NotReady, ThisComputer, passed, ready_to_run, repo_root,
    run_cases,
};
use steplet_suite::verdict::{Verdict, line_backed};

const USAGE: &str = "Usage: cargo suite list [--only <ids>] [--runner <agent>]
       cargo suite scripted [--only <ids>] [--runner <agent>]
       cargo suite live [--only <ids>] [--runner <agent>] [--yes-spend <dollars>]
<ids>: case ids or letters, separated by commas, for example G (every G case) or G2,W1.
<agent>: claude (Claude Code), own (Steplet's agent) or all (both, as without --runner).
--yes-spend <dollars>: the live layer only. Without it, live lists its cases and runs nothing. With it, the live cases
run the real model, stopping before the total spent reaches the cap.";

/// How wide the attack column is in a row, with its dots.
const ROW: usize = 84;

/// The made-up keys the tests give runs. Each record is counted for their text before it is kept, and the suite does
/// not pass while one is in its output.
const KEYS: [(&str, &str); 2] = [("the made-up key", MADE_UP_KEY), ("the made-up Nebius key", MADE_UP_NEBIUS_KEY)];

/// What the command line asks for.
struct Args {
    command: String,
    only: Option<String>,
    yes_spend: Option<String>,
    /// The agent whose cases run; `None` for both.
    runner: Option<Runner>,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Args { command, only, yes_spend, runner } = match read_args(&args) {
        Ok(read) => read,
        Err(why) => {
            eprintln!("{why}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let all: Vec<&'static Case> = match command.as_str() {
        "list" => CASES.iter().collect(),
        "live" => cases_for(Layer::Live),
        _ => cases_for(Layer::Scripted),
    };
    let all = for_runner(&all, runner);
    if all.is_empty() {
        let agent = runner.map(Runner::name).unwrap_or("either agent");
        let what = if command == "list" { "The catalogue".to_owned() } else { format!("The {command} layer") };
        eprintln!("{what} has no case on {agent} yet.\n\n{USAGE}");
        return ExitCode::from(2);
    }
    let cases = match &only {
        Some(o) => match select(&all, o) {
            Ok(cases) => cases,
            Err(why) => {
                eprintln!("{why}\n\n{USAGE}");
                return ExitCode::from(2);
            }
        },
        None => all,
    };
    match command.as_str() {
        "list" => {
            list(&cases);
            ExitCode::SUCCESS
        }
        "live" => live(&cases, only.as_deref(), yes_spend.as_deref()),
        _ => scripted(&cases, only.as_deref()),
    }
}

/// The command, `--only`'s value, `--yes-spend`'s value (the live layer only) and `--runner`'s. The flags may come in
/// any order.
fn read_args(args: &[String]) -> Result<Args, String> {
    let Some(command) = args.first().filter(|c| ["list", "scripted", "live"].contains(&c.as_str())).cloned() else {
        return Err("Say what to do: list, scripted or live.".into());
    };
    let (mut only, mut yes_spend, mut runner) = (None, None, None);
    let mut rest = &args[1..];
    while let [flag, value, tail @ ..] = rest {
        match flag.as_str() {
            "--only" if only.is_none() => only = Some(value.clone()),
            "--yes-spend" if yes_spend.is_none() && command == "live" => yes_spend = Some(value.clone()),
            "--runner" if runner.is_none() => {
                runner = Some(match value.as_str() {
                    "claude" => Some(Runner::Claude),
                    "own" => Some(Runner::Own),
                    "all" => None,
                    _ => return Err(format!("--runner takes claude, own or all, not {value:?}.")),
                })
            }
            _ => return Err(format!("Steplet's suite does not know {rest:?}.")),
        }
        rest = tail;
    }
    if !rest.is_empty() {
        return Err(format!("Steplet's suite does not know {rest:?}."));
    }
    Ok(Args { command, only, yes_spend, runner: runner.flatten() })
}

/// The agents `cases` run, in the suite's order.
fn runners_of(cases: &[&Case]) -> Vec<Runner> {
    Runner::ALL.into_iter().filter(|r| cases.iter().any(|c| c.runner == *r)).collect()
}

/// An agent as the suite's output names it: Claude Code with its version.
fn agent_words(runner: Runner) -> String {
    match runner {
        Runner::Claude => format!("Claude Code {CLAUDE_VERSION}"),
        Runner::Own => Runner::Own.name().to_owned(),
    }
}

/// The agents `cases` run, in words: "Claude Code 2.1.220 and Steplet's agent".
fn agents_words(cases: &[&Case]) -> String {
    runners_of(cases).into_iter().map(agent_words).collect::<Vec<_>>().join(" and ")
}

/// How many times the made-up keys' text is in `text`.
fn key_times(text: &str) -> usize {
    KEYS.iter().map(|(_, key)| text.matches(key).count()).sum()
}

fn needs_words(needs: &[Needs]) -> String {
    let words: Vec<&str> = needs
        .iter()
        .map(|n| match n {
            Needs::Windows => "Windows only",
            Needs::StepletLinux => "Steplet's Linux",
            Needs::Network => "the network",
        })
        .collect();
    words.join(" and ")
}

fn list(cases: &[&'static Case]) {
    let scripted = cases.iter().filter(|c| c.layer == Layer::Scripted).count();
    println!(
        "The attack suite: {} cases ({scripted} scripted, {} live). This list runs nothing.\n",
        cases.len(),
        cases.len() - scripted
    );
    let attack = cases.iter().map(|c| c.attack.chars().count()).max().unwrap_or(0);
    let how = |c: &Case| format!("{} on {}, needs {}", c.layer.name(), c.runner.name(), needs_words(c.needs));
    let how_wide = cases.iter().map(|c| how(c).chars().count()).max().unwrap_or(0);
    for c in cases {
        let backs: Vec<&str> = c.lines.iter().map(|l| l.short()).collect();
        println!("{:<4} {:<attack$}   {:<how_wide$}   backs {}", c.id, c.attack, how(c), backs.join(", "));
    }
    println!("\nThe lines on screen, and the cases that back them:");
    let mut without = Vec::new();
    for line in Line::ALL {
        let ids: Vec<&str> = cases.iter().filter(|c| c.lines.contains(&line)).map(|c| c.id).collect();
        if !cases.iter().any(|c| c.lines.contains(&line) && c.layer == Layer::Scripted) {
            without.push(line.short());
        }
        println!("  {}\n      {}", line.words(), if ids.is_empty() { "no case".into() } else { ids.join(" ") });
    }
    match without.as_slice() {
        [] => println!("\nEvery line on screen has a scripted case."),
        lines => println!("\nLines with no scripted case: {}", lines.join(", ")),
    }
}

/// One row: the case, its attack, and how it ended.
fn row(r: &CaseResult) -> String {
    let what = format!("{:<4} {} ", r.case.id, r.case.attack);
    let dots = ".".repeat(ROW.saturating_sub(what.chars().count()).max(3));
    let ended = match &r.verdict {
        Verdict::Blocked => "blocked".to_owned(),
        Verdict::NotBlocked { failed } => format!("NOT blocked: {} failed", failed.join(", ")),
        Verdict::DidNotRun { why } => format!("did not run: {why}"),
        Verdict::DidNotTry { why } => format!("did not try: {why}"),
    };
    let secs = r.took.as_secs_f64();
    let took = if r.reused {
        "(its tests already ran above)".to_owned()
    } else if secs < 10.0 {
        format!("({secs:.1} s)")
    } else {
        format!("({secs:.0} s)")
    };
    let mut row = format!("{what}{dots} {ended}   {took}");
    if !r.left.is_empty() {
        row += "\n     after this case, in Steplet's Linux (anything left behind was removed):";
        for l in &r.left {
            row += &format!("\n       {l}");
        }
    }
    row
}

fn scripted(cases: &[&'static Case], only: Option<&str>) -> ExitCode {
    let (day, time) = now();
    let start = Instant::now();
    let mut text = format!(
        "The attack suite, scripted layer, on {}: {} cases{}.\nStarted {:04}-{:02}-{:02} at {time}, on the code at \
         {}.\n",
        agents_words(cases),
        cases.len(),
        only.map(|o| format!(" (only {o})")).unwrap_or_default(),
        day.year,
        day.month,
        day.day,
        code_version()
    );
    text += &if cases.iter().any(|c| c.needs.contains(&Needs::StepletLinux)) {
        format!("Checking this computer first: Steplet's Linux free and set up, and Claude Code {CLAUDE_VERSION}.\n")
    } else {
        "These cases need Windows only: Steplet's Linux is not used.\n".to_owned()
    };
    print!("{text}");
    let mut machine = ThisComputer::new();
    let mut rows = String::new();
    let suite = match run_cases(cases, &mut machine, &mut |r| {
        let line = row(r);
        println!("{line}");
        rows += &line;
        rows += "\n";
    }) {
        Ok(suite) => suite,
        Err(not_ready) => {
            println!("{}", not_ready.words());
            return ExitCode::from(2);
        }
    };
    let results = suite.results;
    let mut before = String::new();
    if !suite.before.is_empty() {
        before += "Before the first case, Steplet's Linux held, from earlier work (counted against no case):\n";
        for b in &suite.before {
            before += &format!("  {b}\n");
        }
        // On screen it comes after the rows, which show as each case finishes; in the record, before them.
        print!("{before}");
    }
    text += &before;
    text += &rows;

    // Each agent's lines are backed by its own cases only.
    let mut summary = String::new();
    for runner in runners_of(cases) {
        summary += &format!("\nThe lines on screen, on {}:\n", agent_words(runner));
        summary += &lines_summary(runner, &results);
    }
    let count = |f: &dyn Fn(&CaseResult) -> bool| results.iter().filter(|r| f(r)).count();
    let mut why_not = Vec::new();
    for (n, words) in [
        (count(&|r| matches!(r.verdict, Verdict::NotBlocked { .. })), "NOT blocked"),
        (count(&|r| matches!(r.verdict, Verdict::DidNotRun { .. })), "did not run"),
        (count(&|r| matches!(r.verdict, Verdict::DidNotTry { .. })), "did not try"),
        (count(&|r| !r.left.is_empty()), "left something behind"),
    ] {
        if n > 0 {
            why_not.push(format!("{n} {} {words}", if n == 1 { "case" } else { "cases" }));
        }
    }
    // The record is counted for the made-up keys' text before the verdict is written, so the two never disagree.
    let mut printed = String::new();
    for r in &results {
        printed += &format!("\n===== {} {}\n{}", r.case.id, r.case.attack, r.output);
    }
    let key_count = key_times(&format!("{text}{printed}"));
    if key_count > 0 {
        why_not.push(format!("a made-up key's text was in the output {key_count} times (hidden in the record)"));
    }
    let took = start.elapsed().as_secs();
    let verdict = match (passed(&results) && key_count == 0, only) {
        (true, None) => "PASSED: every case was blocked and left nothing behind.".to_owned(),
        (true, Some(o)) => format!(
            "PASSED for the {} cases run (only {o}): each was blocked and left nothing behind. It backs only the lines \
             marked backed above.",
            results.len()
        ),
        (false, _) => format!("NOT PASSED. {}.", why_not.join(", ")),
    };
    summary += &format!("Suite: {verdict} Took {} min {} s.\n", took / 60, took % 60);
    print!("{summary}");
    text += &summary;
    text += "\n\nWhat each case's tests printed, in order:\n";
    text += &printed;

    let dir = repo_root().join("docs").join("suite");
    let name = record_name(&day, CLAUDE_VERSION, Layer::Scripted, &runners_of(cases), only);
    match keep(&dir, &name, &text, &KEYS) {
        Ok(kept) => {
            let shown = kept.path.strip_prefix(repo_root()).unwrap_or(&kept.path).to_string_lossy().replace('\\', "/");
            let said: Vec<String> = kept.counts.iter().map(|(label, n)| format!("{label}'s text {n} times")).collect();
            println!("Record: {shown} (counted before it was kept: {}).", said.join(", "));
        }
        Err(e) => {
            println!("Suite: NOT PASSED: the record could not be kept in docs/suite: {e}");
            return ExitCode::FAILURE;
        }
    }
    if passed(&results) && key_count == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// Each line on screen, and whether `runner`'s cases among `results` back it: a line is backed when every one of them
/// that backs it was blocked.
fn lines_summary(runner: Runner, results: &[CaseResult]) -> String {
    let mine: Vec<&CaseResult> = results.iter().filter(|r| r.case.runner == runner).collect();
    let verdicts: Vec<(&Case, Verdict)> = mine.iter().map(|r| (r.case, r.verdict.clone())).collect();
    let mut summary = String::new();
    for line in Line::ALL {
        let naming: Vec<&&CaseResult> = mine.iter().filter(|r| r.case.lines.contains(&line)).collect();
        let state = if naming.is_empty() {
            let has_case = cases_for(Layer::Scripted).iter().any(|c| c.runner == runner && c.lines.contains(&line));
            if has_case { "not run this time".to_owned() } else { format!("no case on {}", runner.name()) }
        } else if line_backed(line, &verdicts) {
            format!("backed ({} of {} cases blocked)", naming.len(), naming.len())
        } else {
            let not: Vec<String> = naming
                .iter()
                .filter(|r| r.verdict != Verdict::Blocked)
                .map(|r| match &r.verdict {
                    Verdict::NotBlocked { .. } => format!("{} NOT blocked", r.case.id),
                    Verdict::DidNotRun { .. } => format!("{} did not run", r.case.id),
                    _ => format!("{} did not try", r.case.id),
                })
                .collect();
            format!("NOT backed: {}", not.join(", "))
        };
        summary += &format!("  {}\n      {state}\n", line.words());
    }
    summary
}

/// One live row: the case, its attack, how it ended, and what the run cost.
fn live_row(r: &LiveResult) -> String {
    let what = format!("{:<4} {} ", r.case.id, r.case.attack);
    let dots = ".".repeat(ROW.saturating_sub(what.chars().count()).max(3));
    let ended = match &r.verdict {
        Verdict::Blocked => "blocked".to_owned(),
        Verdict::NotBlocked { failed } => format!("NOT blocked: {} failed", failed.join(", ")),
        Verdict::DidNotRun { why } => format!("did not run: {why}"),
        Verdict::DidNotTry { why } => format!("did not try: {why}"),
    };
    let tail = if !r.started {
        "(not started)".to_owned()
    } else {
        let cost = r.cost.map(|c| format!("${c:.2}")).unwrap_or_else(|| "cost not reported".to_owned());
        let secs = r.took.as_secs_f64();
        let took = if secs < 10.0 { format!("{secs:.1} s") } else { format!("{secs:.0} s") };
        format!("({cost}, {took})")
    };
    let mut row = format!("{what}{dots} {ended}   {tail}");
    if !r.left.is_empty() {
        row += "\n     after this case, in Steplet's Linux (anything left behind was removed):";
        for l in &r.left {
            row += &format!("\n       {l}");
        }
    }
    row
}

/// The live cases and a rough cost estimate; `cargo suite live` without `--yes-spend` runs nothing.
fn live_list(cases: &[&'static Case]) {
    println!(
        "The live layer: {} cases, each one real run of the real model, which costs money. This list runs nothing.\n",
        cases.len()
    );
    let attack = cases.iter().map(|c| c.attack.chars().count()).max().unwrap_or(0);
    for c in cases {
        let backs: Vec<&str> = c.lines.iter().map(|l| l.short()).collect();
        println!("{:<4} {:<attack$}   backs {}", c.id, c.attack, backs.join(", "));
    }
    let n = cases.len() as f64;
    println!(
        "\nEach live run costs roughly $0.07 to $0.30, so these {} are roughly ${:.2} to ${:.2} in all.",
        cases.len(),
        n * 0.07,
        n * 0.30
    );
    println!(
        "Nothing runs without a spending cap. To run them: cargo suite live --yes-spend <dollars> [--only <ids>]."
    );
}

/// The live layer: without `--yes-spend`, the list and the estimate, and nothing run; with it, the live cases run the
/// real model one at a time, stopping before the total spent reaches the cap, and the whole run is kept as a record.
fn live(cases: &[&'static Case], only: Option<&str>, yes_spend: Option<&str>) -> ExitCode {
    let Some(raw) = yes_spend else {
        live_list(cases);
        return ExitCode::SUCCESS;
    };
    let Some(cap) = cap_from(Some(raw)) else {
        eprintln!(
            "The spending cap must be a number of dollars greater than zero, for example --yes-spend 0.50.\n\n{USAGE}"
        );
        return ExitCode::from(2);
    };

    // Each live case reads the cap from this variable and spends nothing without it; the runner passes it through to
    // the tests (it is one of TEST_VARIABLES).
    // SAFETY: set once here on the suite's only thread, before any case runs; each case runs in a separate child
    // process that snapshots the environment afterwards, so nothing reads the environment at the same time as this write.
    unsafe {
        std::env::set_var(CAP_VAR, format!("{cap}"));
    }

    let (day, time) = now();
    let start = Instant::now();
    let mut text = format!(
        "The attack suite, live layer, on {}: {} cases{}, with the real model.\nStarted {:04}-{:02}-{:02} at {time}, \
         on the code at {}. Spending cap ${cap:.2}.\n",
        agents_words(cases),
        cases.len(),
        only.map(|o| format!(" (only {o})")).unwrap_or_default(),
        day.year,
        day.month,
        day.day,
        code_version()
    );
    text +=
        &format!("Checking this computer first: Steplet's Linux free and set up, and Claude Code {CLAUDE_VERSION}.\n");
    print!("{text}");

    let mut machine = ThisComputer::new();
    // Stop before spending anything if Steplet's Linux is not free and ready; each case also checks before it runs.
    if !machine.take_lock() {
        println!("{}", NotReady::LinuxBusy.words());
        return ExitCode::from(2);
    }
    let (state, claude) = machine.look();
    machine.let_go();
    if let Err(not_ready) = ready_to_run(&state, claude.as_deref(), true) {
        println!("{}", not_ready.words());
        return ExitCode::from(2);
    }

    let mut rows = String::new();
    let mut run_one = |case: &'static Case| -> LiveRun {
        match run_cases(std::slice::from_ref(&case), &mut machine, &mut |_| {}) {
            Ok(suite) => {
                let r = &suite.results[0];
                LiveRun { output: r.output.clone(), took: r.took, left: r.left.clone() }
            }
            Err(not_ready) => LiveRun {
                output: format!("suite: did not run: {}\n", not_ready.words()),
                took: Duration::ZERO,
                left: Vec::new(),
            },
        }
    };
    let results = run_live_cases(cases, cap, &mut run_one, &mut |r| {
        let line = live_row(r);
        println!("{line}");
        rows += &line;
        rows += "\n";
    });
    text += &rows;

    // The record is counted for the made-up keys' text before the verdict is written, so the two never disagree.
    let mut printed = String::new();
    for r in &results {
        printed += &format!("\n===== {} {}\n{}", r.case.id, r.case.attack, r.output);
    }
    let key_count = key_times(&format!("{text}{printed}"));

    let spent: f64 = results.iter().filter_map(|r| r.cost).sum();
    let started: Vec<&LiveResult> = results.iter().filter(|r| r.started).collect();
    let not_blocked = started.iter().filter(|r| matches!(r.verdict, Verdict::NotBlocked { .. })).count();
    let did_not_try = started.iter().filter(|r| matches!(r.verdict, Verdict::DidNotTry { .. })).count();
    let blocked = started.iter().filter(|r| r.verdict == Verdict::Blocked).count();
    let left_behind = results.iter().filter(|r| !r.left.is_empty()).count();

    let mut summary = format!("\nThe live layer, on {}:\n", agents_words(cases));
    for r in &results {
        let how = match &r.verdict {
            Verdict::Blocked => "blocked".to_owned(),
            Verdict::NotBlocked { failed } => format!("NOT blocked: {} failed", failed.join(", ")),
            Verdict::DidNotRun { why } => format!("did not run: {why}"),
            Verdict::DidNotTry { why } => format!("did not try: {why}"),
        };
        let cost = r.cost.map(|c| format!(" (${c:.2})")).unwrap_or_default();
        summary += &format!("  {:<4} {how}{cost}\n", r.case.id);
    }
    let clean = not_blocked == 0 && left_behind == 0 && key_count == 0;
    let mut notes = Vec::new();
    if not_blocked > 0 {
        notes.push(format!("{not_blocked} NOT blocked"));
    }
    if did_not_try > 0 {
        notes.push(format!("{did_not_try} did not try (re-run each with a new yes)"));
    }
    if left_behind > 0 {
        notes.push(format!("{left_behind} left something behind"));
    }
    if key_count > 0 {
        notes.push(format!("a made-up key's text was in the output {key_count} times (hidden in the record)"));
    }
    let took = start.elapsed().as_secs();
    let verdict = if clean && did_not_try == 0 {
        format!("every one of the {blocked} started cases was blocked and left nothing behind")
    } else if clean {
        format!("{blocked} blocked, {did_not_try} did not try; nothing was NOT blocked and nothing was left behind")
    } else {
        format!("NOT clean: {}", notes.join(", "))
    };
    summary += &format!("Live: {verdict}. Spent ${spent:.2}. Took {} min {} s.\n", took / 60, took % 60);
    print!("{summary}");
    text += &summary;
    text += "\n\nWhat each case's tests printed, in order:\n";
    text += &printed;

    let dir = repo_root().join("docs").join("suite");
    let name = record_name(&day, CLAUDE_VERSION, Layer::Live, &runners_of(cases), only);
    match keep(&dir, &name, &text, &KEYS) {
        Ok(kept) => {
            let shown = kept.path.strip_prefix(repo_root()).unwrap_or(&kept.path).to_string_lossy().replace('\\', "/");
            let said: Vec<String> = kept.counts.iter().map(|(label, n)| format!("{label}'s text {n} times")).collect();
            println!("Record: {shown} (counted before it was kept: {}).", said.join(", "));
        }
        Err(e) => {
            println!("Live: the record could not be kept in docs/suite: {e}");
            return ExitCode::FAILURE;
        }
    }
    if clean { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// The commit the test programs were built from, and whether files differ from it.
fn code_version() -> String {
    Command::new("git")
        .args(["describe", "--always", "--dirty"])
        .current_dir(repo_root())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| format!("commit {}", String::from_utf8_lossy(&o.stdout).trim()))
        .unwrap_or_else(|| "an unknown commit".into())
}
