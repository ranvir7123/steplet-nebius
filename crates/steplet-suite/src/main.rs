//! `cargo suite list | scripted | live [--only <ids>]` (phase 5, section 1): the attack suite's one command.

use std::process::{Command, ExitCode};
use std::time::Instant;

use steplet_run::setup::CLAUDE_VERSION;
use steplet_suite::catalogue::{CASES, Case, Layer, Line, Needs, cases_for, select};
use steplet_suite::record::{keep, now, record_name};
use steplet_suite::runner::{CaseResult, MADE_UP_KEY, ThisComputer, passed, repo_root, run_cases};
use steplet_suite::verdict::{Verdict, line_backed};

const USAGE: &str = "Usage: cargo suite list [--only <ids>]
       cargo suite scripted [--only <ids>]
       cargo suite live [--only <ids>] [--yes-spend <dollars>]   (the live layer comes with phase 5, Task 8)
<ids>: case ids or letters, separated by commas, for example G (every G case) or G2,W1.";

/// How wide the attack column is in a row, with its dots.
const ROW: usize = 84;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (command, only) = match read_args(&args) {
        Ok(read) => read,
        Err(why) => {
            eprintln!("{why}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    if command == "live" {
        println!("The live layer is not built yet: it comes with phase 5, Task 8. Nothing was run.");
        return ExitCode::from(2);
    }
    let all: Vec<&'static Case> = if command == "list" { CASES.iter().collect() } else { cases_for(Layer::Scripted) };
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
    if command == "list" {
        list(&cases);
        return ExitCode::SUCCESS;
    }
    scripted(&cases, only.as_deref())
}

/// The command and `--only`'s value.
fn read_args(args: &[String]) -> Result<(String, Option<String>), String> {
    let Some(command) = args.first().filter(|c| ["list", "scripted", "live"].contains(&c.as_str())) else {
        return Err("Say what to do: list, scripted or live.".into());
    };
    match &args[1..] {
        [] => Ok((command.clone(), None)),
        [flag, ids] if flag == "--only" => Ok((command.clone(), Some(ids.clone()))),
        // Not built yet: whatever follows, `live` only says so.
        _ if command == "live" => Ok((command.clone(), None)),
        rest => Err(format!("Steplet's suite does not know {rest:?}.")),
    }
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
    let how = |c: &Case| format!("{}, needs {}", c.layer.name(), needs_words(c.needs));
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
        "The attack suite, scripted layer, on Claude Code {CLAUDE_VERSION}: {} cases{}.\nStarted {:04}-{:02}-{:02} at \
         {time}, on the code at {}.\n",
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
    let results = match run_cases(cases, &mut machine, &mut |r| {
        let line = row(r);
        println!("{line}");
        rows += &line;
        rows += "\n";
    }) {
        Ok(results) => results,
        Err(not_ready) => {
            println!("{}", not_ready.words());
            return ExitCode::from(2);
        }
    };
    text += &rows;

    let verdicts: Vec<(&Case, Verdict)> = results.iter().map(|r| (r.case, r.verdict.clone())).collect();
    let mut summary = format!("\nThe lines on screen, on Claude Code {CLAUDE_VERSION}:\n");
    for line in Line::ALL {
        let naming: Vec<&CaseResult> = results.iter().filter(|r| r.case.lines.contains(&line)).collect();
        let state = if naming.is_empty() {
            "not run this time".to_owned()
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
    // The record is counted for the made-up key's text before the verdict is written, so the two never disagree.
    let mut printed = String::new();
    for r in &results {
        printed += &format!("\n===== {} {}\n{}", r.case.id, r.case.attack, r.output);
    }
    let key_times = format!("{text}{printed}").matches(MADE_UP_KEY).count();
    if key_times > 0 {
        why_not.push(format!("the made-up key's text was in the output {key_times} times (hidden in the record)"));
    }
    let took = start.elapsed().as_secs();
    let verdict = match (passed(&results) && key_times == 0, only) {
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
    let name = record_name(&day, CLAUDE_VERSION, Layer::Scripted, only);
    match keep(&dir, &name, &text, &[("the made-up key", MADE_UP_KEY)]) {
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
    if passed(&results) && key_times == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
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
