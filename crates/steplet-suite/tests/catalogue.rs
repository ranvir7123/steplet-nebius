//! The catalogue (phase 5 Task 1): every known attack names tests that exist, every line on screen has a scripted
//! case, the run chooser and the suite name the same tests, and each case says which agent its tests run (the Nebius
//! plan, Task 9). None of these needs Steplet's Linux.

use std::path::Path;

use steplet_core::canon::canonicalize;
use steplet_core::header::{Steplet, parse};
use steplet_run::{NetworkMode, RunnerKind, promises, promises_for};
use steplet_suite::catalogue::{CASES, Case, Layer, Line, Needs, Runner, TestRef, cases_for, for_runner, select};

fn steplet(allow: &str) -> Steplet {
    let f = format!("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n{allow}---\n");
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap()
}

/// The lines of `crates/<package>/tests/<program>.rs`.
fn test_file(t: &TestRef) -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(t.package)
        .join("tests")
        .join(format!("{}.rs", t.program));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    text.lines().map(str::to_owned).collect()
}

/// The attribute lines just above `fn <name>(` in its test file, if it is a test there (marked `#[test]` within the
/// three lines above, as `every_promise_names_tests_that_exist` checks).
fn attributes(t: &TestRef) -> Option<Vec<String>> {
    let lines = test_file(t);
    let n = lines.iter().position(|l| l.starts_with(&format!("fn {}(", t.name)))?;
    let above = lines[n.saturating_sub(3)..n].to_vec();
    above.iter().any(|l| l == "#[test]").then_some(above)
}

/// The line a chooser line backs, from its words. A chooser line this does not know fails the test that uses it.
fn line_of(text: &str) -> Line {
    match text {
        t if t.starts_with("Steplet checks the agent's commands") => Line::Checks,
        t if t.starts_with("The agent can fetch") || t.starts_with("The agent cannot fetch") => Line::Fetch,
        t if t.starts_with("The agent and the programs this task runs can change files only") => Line::ChangeFiles,
        t if t.starts_with("Your own files are not inside") => Line::YourFiles,
        t if t.starts_with("Programs this task runs can connect")
            || t.starts_with("Programs this task runs cannot connect") =>
        {
            Line::Connect
        }
        t if t.starts_with("Programs this task runs cannot start Windows programs") => Line::WindowsPrograms,
        t => panic!("a chooser line the suite does not know: {t:?}"),
    }
}

/// The suite's line for one of Steplet's agent's chooser lines, for the two lines that are the agent's alone (the
/// Nebius plan, Task 9). The agent's other lines rest on tests the suite does not replay as cases of their own.
fn agent_line_of(text: &str) -> Option<Line> {
    match text {
        t if t.starts_with("Your Nebius key stays on Windows") => Some(Line::KeyOnWindows),
        t if t.starts_with("Anything the agent leaves running is ended") => Some(Line::EndedBeforeNextMove),
        _ => None,
    }
}

#[test]
fn every_case_names_tests_that_exist() {
    for case in CASES {
        assert!(!case.tests.is_empty(), "{} names no test", case.id);
        for t in case.tests {
            assert!(
                attributes(t).is_some(),
                "{} names {}::{}::{}, which is not a test there",
                case.id,
                t.package,
                t.program,
                t.name
            );
        }
    }
    let not_a_test = TestRef { package: "steplet-run", program: "promises", name: "steplet" };
    assert!(attributes(&not_a_test).is_none(), "the check itself: a function that is not a test does not count");
}

#[test]
fn every_line_on_screen_has_a_scripted_case() {
    for line in Line::ALL {
        let backing: Vec<&str> =
            cases_for(Layer::Scripted).iter().filter(|c| c.lines.contains(&line)).map(|c| c.id).collect();
        assert!(!backing.is_empty(), "{line:?} has no scripted case");
    }
}

/// Every test a chooser line names (for a file with sites and one without, in every WSL network mode) is in a case
/// backing that same line, so the suite replays everything the chooser rests on.
#[test]
fn the_chooser_and_the_suite_name_the_same_tests() {
    let modes = [NetworkMode::Nat, NetworkMode::Other, NetworkMode::Unknown];
    for s in [steplet(""), steplet("allow:\n  domains:\n    - example.org\n")] {
        for p in modes.iter().flat_map(|m| promises(&s, *m)) {
            let line = line_of(p.text);
            for test in p.tests {
                let backed = CASES.iter().any(|c| c.lines.contains(&line) && c.tests.iter().any(|t| t.name == *test));
                assert!(backed, "the chooser's {line:?} line names {test}, which no case of that line replays");
            }
        }
    }
    // Steplet's agent's own two lines, the same way.
    let mut seen = Vec::new();
    for s in [steplet(""), steplet("allow:\n  domains:\n    - example.org\n")] {
        for p in modes.iter().flat_map(|m| promises_for(RunnerKind::Own, &s, *m)) {
            let Some(line) = agent_line_of(p.text) else { continue };
            seen.push(line);
            for test in p.tests {
                let backed = CASES.iter().any(|c| c.lines.contains(&line) && c.tests.iter().any(|t| t.name == *test));
                assert!(backed, "Steplet's agent's {line:?} line names {test}, which no case of that line replays");
            }
        }
    }
    for line in [Line::KeyOnWindows, Line::EndedBeforeNextMove] {
        assert!(seen.contains(&line), "the chooser shows Steplet's agent's {line:?} line");
    }
}

#[test]
fn ids_are_unique_and_follow_their_line() {
    let mut seen = std::collections::HashSet::new();
    for case in CASES {
        assert!(seen.insert(case.id), "{} is used twice", case.id);
        let (letters, number) = case.id.split_at(case.id.find(|c: char| c.is_ascii_digit()).expect("a number"));
        let letter = match case.layer {
            Layer::Live => 'L',
            Layer::Scripted => case.lines.first().expect("a case backs at least one line").letter(),
        };
        assert_eq!(letters, letter.to_string(), "{}: its letter follows its first line", case.id);
        assert!(number.chars().all(|c| c.is_ascii_digit()) && !number.starts_with('0'), "{}", case.id);
        assert!(!case.attack.is_empty() && !case.needs.is_empty(), "{}", case.id);
        // An attack is one row of the suite's table.
        assert!(!case.attack.contains('\n') && !case.attack.contains("  "), "{}: {:?}", case.id, case.attack);
    }
}

/// A case needs Steplet's Linux, or the network, exactly when one of its tests is set aside for it: the runner checks
/// Steplet's Linux before such a case, so a case that hid the need would run on a machine nobody looked at.
#[test]
fn what_a_case_needs_follows_its_tests() {
    for case in CASES {
        let reasons: Vec<String> =
            case.tests.iter().flat_map(|t| attributes(t).unwrap()).filter(|l| l.starts_with("#[ignore")).collect();
        let linux = reasons.iter().any(|r| r.contains("needs Steplet's Linux"));
        let network = reasons.iter().any(|r| r.contains("the network"));
        assert_eq!(case.needs.contains(&Needs::StepletLinux), linux, "{}: {reasons:?}", case.id);
        assert_eq!(case.needs.contains(&Needs::Network), network, "{}: {reasons:?}", case.id);
        assert_eq!(case.needs.contains(&Needs::Windows), !linux, "{}: Windows alone, or Steplet's Linux", case.id);
    }
}

/// The runner reads each test's result by its name, so one name must mean one test, wherever cases name it.
#[test]
fn a_test_name_means_one_test() {
    let mut seen: std::collections::HashMap<&str, (&str, &str)> = std::collections::HashMap::new();
    for t in CASES.iter().flat_map(|c| c.tests) {
        let place = *seen.entry(t.name).or_insert((t.package, t.program));
        assert_eq!(place, (t.package, t.program), "{} names two tests", t.name);
    }
}

/// Steplet's own agent's cases are the ones whose tests run it (`agent_linux.rs` in Steplet's Linux, `agent.rs` with a
/// stand-in model); every other case runs Claude Code.
#[test]
fn each_case_says_which_agent_its_tests_run() {
    for case in CASES {
        let own = case.tests.iter().filter(|t| matches!(t.program, "agent_linux" | "agent")).count();
        assert!(own == 0 || own == case.tests.len(), "{}: its tests run one agent", case.id);
        let runner = if own > 0 { Runner::Own } else { Runner::Claude };
        assert_eq!(case.runner, runner, "{}", case.id);
    }
    for runner in [Runner::Claude, Runner::Own] {
        assert!(cases_for(Layer::Scripted).iter().any(|c| c.runner == runner), "{runner:?} has scripted cases");
    }
}

/// The agent's own lines on the run chooser are backed by its cases only: Claude Code runs never send a Nebius key,
/// and the agent's moves are what end anything left running.
#[test]
fn the_agents_own_lines_are_backed_by_its_cases() {
    for line in [Line::KeyOnWindows, Line::EndedBeforeNextMove] {
        let backing: Vec<&Case> = CASES.iter().filter(|c| c.lines.contains(&line)).collect();
        assert!(!backing.is_empty(), "{line:?} has a case");
        assert!(backing.iter().all(|c| c.runner == Runner::Own), "{line:?}");
    }
}

#[test]
fn runner_picks_the_cases_of_one_agent() {
    let ids = |cases: Vec<&Case>| cases.iter().map(|c| c.id).collect::<Vec<_>>();
    let all = cases_for(Layer::Scripted);
    let own = for_runner(&all, Some(Runner::Own));
    let claude = for_runner(&all, Some(Runner::Claude));
    assert!(!own.is_empty() && own.iter().all(|c| c.runner == Runner::Own), "{:?}", ids(own));
    assert!(!claude.is_empty() && claude.iter().all(|c| c.runner == Runner::Claude), "{:?}", ids(claude));
    assert_eq!(own.len() + claude.len(), all.len());
    assert_eq!(ids(for_runner(&all, None)), ids(all.clone()), "all: every case, in the catalogue's order");
    let in_order: Vec<&str> = all.iter().filter(|c| c.runner == Runner::Own).map(|c| c.id).collect();
    assert_eq!(ids(for_runner(&all, Some(Runner::Own))), in_order, "in the catalogue's order");
}

#[test]
fn only_picks_cases_by_id_or_by_their_letter() {
    let ids = |cases: Vec<&Case>| cases.iter().map(|c| c.id).collect::<Vec<_>>();
    let all = cases_for(Layer::Scripted);
    let g: Vec<&str> = all.iter().filter(|c| c.id.starts_with('G')).map(|c| c.id).collect();
    assert!(g.len() > 2);
    assert_eq!(ids(select(&all, "G").unwrap()), g);
    assert_eq!(ids(select(&all, "G1").unwrap()), ["G1"], "G1 is not G11 or G12");
    assert_eq!(ids(select(&all, "W1,G2").unwrap()), ["G2", "W1"], "in the catalogue's order");
    assert_eq!(select(&all, "Q7").unwrap_err(), "no case has the id or letter \"Q7\"");
    assert_eq!(select(&all, "").unwrap_err(), "no case has the id or letter \"\"");
}
