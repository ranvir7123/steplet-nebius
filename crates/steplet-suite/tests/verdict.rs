//! How the suite judges a case (phase 5 Task 1): only a case whose every test said `ok` is blocked; a case that could
//! not run, or a test that gave no result, is never shown as blocked; the suite stops before anything when another run
//! holds Steplet's Linux or another Claude Code is installed; and what a case leaves behind fails the suite. Made-up
//! outputs and a stand-in for this computer: none of these needs Steplet's Linux.

#[path = "../../steplet-run/tests/support/mod.rs"]
mod support;

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use steplet_run::setup::{CLAUDE_VERSION, State, Wsl};
use steplet_suite::catalogue::{Case, Layer, Line, Needs, TestRef};
use steplet_suite::runner::{
    CaseResult, Machine, NotReady, leftovers, output_of, passed, ready_to_run, removal_script, run_cases,
    test_environment,
};
use steplet_suite::verdict::{TestOutcome, Verdict, line_backed, read_results, verdict};

const fn test(name: &'static str) -> TestRef {
    TestRef { package: "steplet-run", program: "made_up", name }
}

static TWO: Case = Case {
    id: "G90",
    attack: "A made-up case with two tests",
    lines: &[Line::Checks],
    tests: &[test("first_check"), test("second_check")],
    layer: Layer::Scripted,
    needs: &[Needs::Windows],
};

static ON_LINUX: Case = Case {
    id: "F90",
    attack: "A made-up case on Steplet's Linux",
    lines: &[Line::ChangeFiles],
    tests: &[test("a_linux_check")],
    layer: Layer::Scripted,
    needs: &[Needs::StepletLinux],
};

static SHARING: Case = Case {
    id: "Y90",
    attack: "A made-up case naming F90's test too",
    lines: &[Line::YourFiles],
    tests: &[test("a_linux_check"), test("another_linux_check")],
    layer: Layer::Scripted,
    needs: &[Needs::StepletLinux],
};

fn outcomes(pairs: &[(&str, TestOutcome)]) -> HashMap<String, TestOutcome> {
    pairs.iter().map(|(n, o)| ((*n).to_owned(), o.clone())).collect()
}

/// Steplet's Linux as it is when everything is set up.
fn ready() -> State {
    State {
        wsl: Wsl::Ready { version: "2.6.3.0".into() },
        linux: Some("{made-up-id}".into()),
        missing: Vec::new(),
        gate: true,
        trouble: None,
    }
}

/// A stand-in for this computer: it notes every call, and every test it is asked to run says `ok`. The lock is free
/// while `lock_free` says so (its last answer repeats); a removal clears what is left unless `stubborn`.
struct StandIn {
    lock_free: Vec<bool>,
    claude: Option<String>,
    left: String,
    stubborn: bool,
    calls: Vec<String>,
}

impl StandIn {
    fn new(lock_free: bool, claude: &str, left: &str) -> StandIn {
        StandIn {
            lock_free: vec![lock_free],
            claude: Some(claude.into()),
            left: left.into(),
            stubborn: false,
            calls: Vec::new(),
        }
    }
}

impl Machine for StandIn {
    fn take_lock(&mut self) -> bool {
        self.calls.push("lock".into());
        if self.lock_free.len() > 1 { self.lock_free.remove(0) } else { self.lock_free[0] }
    }
    fn let_go(&mut self) {
        self.calls.push("let go".into());
    }
    fn look(&mut self) -> (State, Option<String>) {
        self.calls.push("look".into());
        (ready(), self.claude.clone())
    }
    fn build(&mut self, package: &str, programs: &[&str]) -> Result<(), String> {
        self.calls.push(format!("build {package} {programs:?}"));
        Ok(())
    }
    fn run_tests(&mut self, package: &str, program: &str, names: &[&str]) -> String {
        self.calls.push(format!("run {package} {program} {names:?}"));
        let rows: String = names.iter().map(|n| format!("test {n} ... ok\n")).collect();
        format!("{rows}\n{}", summary(names.len(), 0, 0))
    }
    fn after_case_look(&mut self) -> String {
        self.calls.push("after".into());
        self.left.clone()
    }
    fn remove(&mut self, left: &[String]) {
        self.calls.push(format!("remove {left:?}"));
        if !self.stubborn {
            self.left.clear();
        }
    }
}

/// Rust's last line for a test program that ran to its end.
fn summary(passed: usize, failed: usize, ignored: usize) -> String {
    let word = if failed == 0 { "ok" } else { "FAILED" };
    format!(
        "test result: {word}. {passed} passed; {failed} failed; {ignored} ignored; 0 measured; 0 filtered out; finished \
         in 0.01s\n"
    )
}

#[test]
fn blocked_only_when_every_test_of_the_case_says_ok() {
    let both = outcomes(&[("first_check", TestOutcome::Ok), ("second_check", TestOutcome::Ok)]);
    assert_eq!(verdict(&TWO, &both), Verdict::Blocked);
    let one_failed = outcomes(&[("first_check", TestOutcome::Ok), ("second_check", TestOutcome::Failed)]);
    assert_eq!(verdict(&TWO, &one_failed), Verdict::NotBlocked { failed: vec!["second_check".into()] });
    // A failure counts before a test that could not run: it is the one to see.
    let both_wrong =
        outcomes(&[("first_check", TestOutcome::DidNotRun("x".into())), ("second_check", TestOutcome::Failed)]);
    assert_eq!(verdict(&TWO, &both_wrong), Verdict::NotBlocked { failed: vec!["second_check".into()] });

    assert!(line_backed(Line::Checks, &[(&TWO, Verdict::Blocked)]));
    assert!(!line_backed(Line::Checks, &[(&TWO, verdict(&TWO, &one_failed))]));
    assert!(!line_backed(Line::Checks, &[]), "a line no case was run for this time is not backed");
    assert!(!line_backed(Line::Checks, &[(&ON_LINUX, Verdict::Blocked)]), "only cases of that line count");
}

#[test]
fn a_case_that_did_not_run_is_never_shown_as_blocked() {
    // Rust's own output with --test-threads=1 --nocapture: the test's lines sit between its name and its result.
    let output = "\nrunning 2 tests\n\
                  test first_check ... ok\n\
                  test second_check ... suite: did not run: the network was not reachable\n\
                  thread 'second_check' panicked at crates\\steplet-run\\tests\\support\\mod.rs:9:5:\n\
                  did not run: the network was not reachable\n\
                  FAILED\n\nfailures:\n\nfailures:\n    second_check\n\n\
                  test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.20s\n";
    let results = read_results(output);
    assert_eq!(results["first_check"], TestOutcome::Ok);
    assert_eq!(results["second_check"], TestOutcome::DidNotRun("the network was not reachable".into()));
    let v = verdict(&TWO, &results);
    assert_eq!(v, Verdict::DidNotRun { why: "the network was not reachable".into() });
    assert!(!line_backed(Line::Checks, &[(&TWO, v)]));

    // Even a test that printed the line and then said ok did not run its case.
    let said_ok = format!(
        "test first_check ... suite: did not run: the scripted model did not start\nok\n\n{}",
        summary(1, 0, 0)
    );
    assert_eq!(
        read_results(&said_ok)["first_check"],
        TestOutcome::DidNotRun("the scripted model did not start".into())
    );
}

#[test]
fn a_missing_or_ignored_test_is_not_blocked() {
    let missing = outcomes(&[("first_check", TestOutcome::Ok)]);
    assert_eq!(verdict(&TWO, &missing), Verdict::DidNotRun { why: "second_check gave no result".into() });
    let ignored = outcomes(&[("first_check", TestOutcome::Ok), ("second_check", TestOutcome::Ignored)]);
    assert_eq!(verdict(&TWO, &ignored), Verdict::DidNotRun { why: "second_check was set aside (ignored)".into() });
    let output = format!(
        "test first_check ... ok\ntest second_check ... ignored, needs Steplet's Linux\n\n{}",
        summary(1, 0, 1)
    );
    let results = read_results(&output);
    assert_eq!(results["second_check"], TestOutcome::Ignored);
    assert!(!line_backed(Line::Checks, &[(&TWO, verdict(&TWO, &results))]));
    // The list of failures at the end wins over a stray "ok" line a test printed itself.
    let stray = format!(
        "test first_check ... \nok\nthread 'first_check' panicked at x.rs:1:1:\nboom\nFAILED\n\
         test second_check ... ok\n\nfailures:\n    first_check\n\n{}",
        summary(1, 1, 0)
    );
    assert_eq!(read_results(&stray)["first_check"], TestOutcome::Failed);
}

/// A test program that ended before Rust's last line (it crashed, or was ended) gave no result at all, whatever lines
/// came before: a test may itself print a line that reads like a result. So do results that do not add up to the
/// counts in that last line.
#[test]
fn nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up() {
    let cut = read_results("test first_check ... ok\ntest second_check ... ");
    assert_eq!(cut, HashMap::new());
    assert_eq!(verdict(&TWO, &cut), Verdict::DidNotRun { why: "first_check gave no result".into() });
    // A test printed a line reading "ok" of its own, then its program crashed.
    let crashed = read_results("test first_check ... ok\ntest second_check ... \nok\n");
    assert_eq!(crashed, HashMap::new());
    assert!(matches!(verdict(&TWO, &crashed), Verdict::DidNotRun { .. }));
    // Results that do not add up to Rust's own count: two read as passed, Rust counted one.
    let two_for_one = format!("test first_check ... ok\ntest second_check ... ok\n\n{}", summary(1, 0, 0));
    assert_eq!(read_results(&two_for_one), HashMap::new());
    // While a test runs, a line it prints that looks like another test's result is its own output, not a result.
    let printed = format!("test first_check ... \ntest second_check ... ok\nok\n\n{}", summary(1, 0, 0));
    let printed = read_results(&printed);
    assert_eq!(printed.get("first_check"), Some(&TestOutcome::Ok));
    assert_eq!(printed.get("second_check"), None);
    // The near-miss: the same two tests, each really passed.
    let both = format!("test first_check ... ok\ntest second_check ... ok\n\n{}", summary(2, 0, 0));
    assert_eq!(verdict(&TWO, &read_results(&both)), Verdict::Blocked);
}

/// A sample test for the reader's own check below. Each acts only when that check runs it (`STEPLET_SUITE_SAMPLE`);
/// set aside otherwise, and harmless when someone runs every set-aside test.
const SAMPLE: &str = "STEPLET_SUITE_SAMPLE";

#[test]
#[ignore = "a sample for the_reader_reads_rust_s_own_test_output"]
fn sample_that_passes() {
    if std::env::var_os(SAMPLE).is_some() {
        println!("an ordinary line of output");
    }
}

#[test]
#[ignore = "a sample for the_reader_reads_rust_s_own_test_output"]
fn sample_that_fails() {
    if std::env::var_os(SAMPLE).is_some() {
        panic!("a made-up failure");
    }
}

#[test]
#[ignore = "a sample for the_reader_reads_rust_s_own_test_output"]
fn sample_that_did_not_run() {
    if std::env::var_os(SAMPLE).is_some() {
        support::did_not_run("the network was not reachable");
    }
}

#[test]
#[ignore = "a sample for the_reader_reads_rust_s_own_test_output"]
fn sample_that_did_not_try() {
    if std::env::var_os(SAMPLE).is_some() {
        println!("suite: did not try: the model refused");
    }
}

/// The reader against what this Rust really prints, with `--nocapture` (the runner's way: a test's lines come as it
/// prints them) and with `--show-output` (Rust keeps each test's lines and shows them after): this test program runs
/// its own four samples. A future Rust that changed its result lines fails here.
#[test]
fn the_reader_reads_rust_s_own_test_output() {
    let names = ["sample_that_passes", "sample_that_fails", "sample_that_did_not_run", "sample_that_did_not_try"];
    for mode in ["--nocapture", "--show-output"] {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["--include-ignored", "--exact", "--test-threads=1", mode]).args(names).env(SAMPLE, "1");
        let log = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("samples{mode}.log"));
        let output = output_of(&mut command, &log).unwrap();
        println!("--- {mode}:\n{output}");
        let results = read_results(&output);
        assert_eq!(results.len(), 4, "{results:?}");
        assert_eq!(results["sample_that_passes"], TestOutcome::Ok);
        assert_eq!(results["sample_that_fails"], TestOutcome::Failed);
        assert_eq!(results["sample_that_did_not_run"], TestOutcome::DidNotRun("the network was not reachable".into()));
        assert_eq!(results["sample_that_did_not_try"], TestOutcome::DidNotTry("the model refused".into()));
    }
}

#[test]
fn the_suite_refuses_another_claude_code_version() {
    let found = NotReady::ClaudeVersion { found: "2.1.231".into() };
    assert_eq!(ready_to_run(&ready(), Some("2.1.231"), true), Err(found.clone()));
    // Steplet's own look calls Claude Code missing when its version is not the pinned one: the same answer.
    let says_missing = State { missing: vec!["claude-code".into()], ..ready() };
    assert_eq!(ready_to_run(&says_missing, Some("2.1.231"), true), Err(found.clone()));
    assert_eq!(ready_to_run(&ready(), Some(CLAUDE_VERSION), true), Ok(()), "the near-miss: the pinned version");
    assert!(matches!(ready_to_run(&says_missing, None, true), Err(NotReady::LinuxMissing { .. })));
    assert!(matches!(
        ready_to_run(&State { linux: None, ..ready() }, Some(CLAUDE_VERSION), true),
        Err(NotReady::LinuxMissing { .. })
    ));

    let mut machine = StandIn::new(true, "2.1.231", "");
    let cases: Vec<&'static Case> = vec![&ON_LINUX];
    assert_eq!(run_cases(&cases, &mut machine, &mut |_| ()).unwrap_err(), found);
    assert_eq!(machine.calls, ["lock", "look", "let go"], "nothing was built or run");
}

#[test]
fn the_suite_starts_nothing_while_a_run_holds_the_lock() {
    assert_eq!(ready_to_run(&ready(), Some(CLAUDE_VERSION), false), Err(NotReady::LinuxBusy));
    let mut machine = StandIn::new(false, CLAUDE_VERSION, "");
    let cases: Vec<&'static Case> = vec![&TWO, &ON_LINUX];
    assert_eq!(run_cases(&cases, &mut machine, &mut |_| ()).unwrap_err(), NotReady::LinuxBusy);
    assert_eq!(machine.calls, ["lock"], "it did not even look inside Steplet's Linux");

    // The near-miss: cases on Windows alone need neither the lock nor Steplet's Linux.
    let mut machine = StandIn::new(false, CLAUDE_VERSION, "");
    let cases: Vec<&'static Case> = vec![&TWO];
    let results = run_cases(&cases, &mut machine, &mut |_| ()).unwrap();
    assert_eq!(results[0].verdict, Verdict::Blocked);
    assert!(!machine.calls.iter().any(|c| c == "lock" || c == "look" || c == "after"), "{:?}", machine.calls);
}

#[test]
fn leftovers_after_a_case_are_named_and_fail_the_suite() {
    // The look's records: `<kind>\t<what>`, each ended by a NUL, the one character no file name can hold.
    let listing = "test-file\t/opt/steplet-test/listener.py\0\
                   test-file\t/opt/steplet-test/cmd.exe\0\
                   test-file\t/opt/steplet-test/t11b.listener.log\0\
                   left\t/work/t11b\0";
    assert_eq!(leftovers(listing), ["/opt/steplet-test/cmd.exe", "/work/t11b"]);
    let clean = "test-file\t/opt/steplet-test/fake_model.py\0test-file\t/opt/steplet-test/t7a.script.json\0\
                 test-file\t/opt/steplet-test/t7a.model.log\0test-file\t/opt/steplet-test/made-up-key.sha256\0";
    assert_eq!(leftovers(clean), Vec::<String>::new());
    assert_eq!(
        leftovers(
            "program\t812\t/usr/bin/python3 -I /opt/steplet-test/listener.py 47831 x\0marker\t/tmp/steplet-m\0\
             something odd\0could-not-look\tthe look inside Steplet's Linux failed\0"
        ),
        [
            "program 812 /usr/bin/python3 -I /opt/steplet-test/listener.py 47831 x",
            "/tmp/steplet-m",
            "could not read: something odd",
            "could not look: the look inside Steplet's Linux failed"
        ]
    );

    let cases: Vec<&'static Case> = vec![&ON_LINUX];
    let mut machine = StandIn::new(true, CLAUDE_VERSION, listing);
    let results = run_cases(&cases, &mut machine, &mut |_| ()).unwrap();
    assert_eq!(results[0].verdict, Verdict::Blocked, "its test passed");
    assert_eq!(results[0].left, ["left behind: /opt/steplet-test/cmd.exe", "left behind: /work/t11b"]);
    // The look, the removal and the look again all happen while the suite holds Steplet's one-run lock.
    let after_tests = machine.calls.iter().position(|c| c.starts_with("run ")).unwrap() + 1;
    assert_eq!(
        machine.calls[after_tests..],
        ["lock", "after", r#"remove ["/opt/steplet-test/cmd.exe", "/work/t11b"]"#, "after", "let go"]
    );
    assert!(!passed(&results), "something was left, so the suite does not pass");

    // What the clean-up could not remove is named as still there.
    let mut machine = StandIn::new(true, CLAUDE_VERSION, "left\t/work/t11b\0");
    machine.stubborn = true;
    let results = run_cases(&cases, &mut machine, &mut |_| ()).unwrap();
    assert_eq!(results[0].left, ["left behind: /work/t11b", "still there after the clean-up: /work/t11b"]);
    assert!(!passed(&results));

    // The control: the same case with nothing left passes.
    let mut machine = StandIn::new(true, CLAUDE_VERSION, clean);
    let results = run_cases(&cases, &mut machine, &mut |_| ()).unwrap();
    assert!(results[0].left.is_empty() && passed(&results));
    assert!(!machine.calls.iter().any(|c| c.starts_with("remove")));
}

/// A Steplet window may start a run between two cases. The look after a case then finds the lock taken, and looks at
/// and removes nothing: that run's working copy is not a leftover. The suite does not pass, since that case was not
/// looked after.
#[test]
fn the_look_after_a_case_touches_nothing_while_another_run_holds_the_lock() {
    let cases: Vec<&'static Case> = vec![&ON_LINUX];
    let mut machine = StandIn::new(true, CLAUDE_VERSION, "left\t/work/a-window-s-run\0");
    machine.lock_free = vec![true, false];
    let results = run_cases(&cases, &mut machine, &mut |_| ()).unwrap();
    let after_tests = machine.calls.iter().position(|c| c.starts_with("run ")).unwrap() + 1;
    assert_eq!(machine.calls[after_tests..], ["lock"], "no look, no removal");
    assert_eq!(
        results[0].left,
        [
            "could not look after this case: Steplet's Linux is in use by another run, so nothing was looked at or removed"
        ]
    );
    assert!(!passed(&results));
}

/// The clean-up after a case ends the programs it named, by number, first; then removes only what the look named that
/// sits directly inside one of the folders it looks in: never one of those folders, Steplet's runs folder, anything
/// deeper (a link on the way could lead out), a path with a way out, a quote or a control character, or anything else.
#[test]
fn the_clean_up_removes_only_what_the_look_named_below_its_places() {
    let left: Vec<String> = [
        "/work/t11b",
        "/opt/steplet-test/cmd.exe",
        "program 812 /usr/bin/python3 -I /opt/steplet-test/listener.py 47831 x",
        "/var/lib/steplet/runs",
        "/var/lib/steplet/t11b.stamp",
        "/work",
        "/work/../etc",
        "/work/it's",
        "/etc/hostname",
        "/tmp/steplet-dir/inner/marker.txt",
        "/home/steplet/notes.txt",
        "/tmp/steplet-a\nprogram 1 x",
        "could not read: something odd",
        "program x1 y",
    ]
    .map(str::to_owned)
    .into();
    assert_eq!(
        removal_script(&left),
        "kill -KILL 812\nrm -rf -- '/work/t11b'\nrm -rf -- '/opt/steplet-test/cmd.exe'\n\
         rm -rf -- '/var/lib/steplet/t11b.stamp'\n"
    );
}

/// A program the task runs can give a file a name holding a line break. In the look's records it stays one name: it
/// cannot add a record of its own (a program to end, a path to remove), and a name with a control character is shown,
/// never removed.
#[test]
fn a_name_with_a_line_break_cannot_add_a_record_of_its_own() {
    let listing = "left\t/tmp/steplet-a\nprogram\t1\tx\0marker\t/tmp/steplet-b\nleft /etc\0left\t/work/t11b\0";
    let left = leftovers(listing);
    assert_eq!(
        left,
        [
            "could not read: left\t/tmp/steplet-a?program\t1\tx",
            "could not read: marker\t/tmp/steplet-b?left /etc",
            "/work/t11b"
        ]
        .map(|s| s.replace('\t', "?"))
    );
    assert_eq!(removal_script(&left), "rm -rf -- '/work/t11b'\n", "nothing ended, nothing else removed");
}

/// The test programs get only the Windows variables they need (where Windows, your folders, cargo and Rust are), not
/// everything of whoever started the suite: a gate test writes the names of the variables it was given into its log,
/// and that log goes into the record.
#[test]
fn test_programs_get_only_the_windows_variables_they_need() {
    let given = [
        ("Path", "C:\\made-up"),
        ("SystemRoot", "C:\\Windows"),
        ("LOCALAPPDATA", "C:\\made-up\\Local"),
        ("GITHUB_PERSONAL_ACCESS_TOKEN", "made-up"),
        ("CLAUDECODE", "1"),
        ("ANTHROPIC_BASE_URL", "http://127.0.0.1:1"),
        ("RUSTUP_TOOLCHAIN", "1.98.0"),
        ("cargo_home", "C:\\made-up\\.cargo"),
    ];
    let kept = test_environment(given.iter().map(|(k, v)| (OsString::from(k), OsString::from(v))));
    let kept: Vec<(String, String)> =
        kept.iter().map(|(k, v)| (k.to_string_lossy().into_owned(), v.to_string_lossy().into_owned())).collect();
    let expected = [
        ("Path", "C:\\made-up"),
        ("SystemRoot", "C:\\Windows"),
        ("LOCALAPPDATA", "C:\\made-up\\Local"),
        ("RUSTUP_TOOLCHAIN", "1.98.0"),
        ("cargo_home", "C:\\made-up\\.cargo"),
    ]
    .map(|(k, v)| (k.to_owned(), v.to_owned()));
    assert_eq!(kept, expected, "Windows reads variable names without regard to case");
}

/// A test that several cases name (the fixture's test backs five lines) runs once per suite run, and each case is
/// judged from that one result.
#[test]
fn a_test_named_by_two_cases_runs_once() {
    let cases: Vec<&'static Case> = vec![&ON_LINUX, &SHARING];
    let mut machine = StandIn::new(true, CLAUDE_VERSION, "");
    let mut rows: Vec<String> = Vec::new();
    let results = run_cases(&cases, &mut machine, &mut |r: &CaseResult| rows.push(r.case.id.to_owned())).unwrap();
    let runs: Vec<&String> = machine.calls.iter().filter(|c| c.starts_with("run ")).collect();
    assert_eq!(
        runs,
        [r#"run steplet-run made_up ["a_linux_check"]"#, r#"run steplet-run made_up ["another_linux_check"]"#]
    );
    assert_eq!(machine.calls.iter().filter(|c| c.starts_with("build")).count(), 1, "built once: {:?}", machine.calls);
    assert_eq!(rows, ["F90", "Y90"], "one row per case, as each finishes");
    assert!(results.iter().all(|r| r.verdict == Verdict::Blocked) && passed(&results));
}
