//! Phase 5, Task 6 (R20 item 4, and the report's models): a run stopped part-way starts again clean, and a model's
//! refusal ends the run and is reported with every model the run used. This is defensive testing of Steplet's own
//! sandbox: every test runs only in Steplet's own Linux, against Steplet's own code, with harmless content: one line
//! appended to a text file, a stand-in model that refuses by script, and the made-up key. Each run goes the way the
//! app's Run button sends it (`run_live_locked`, with the one-run lock the test holds), and is judged from Steplet's
//! records, the stand-in model's log and root's looks from outside the fence, never from what the agent says (F47).
//! Since the Nebius plan's Task 6b also: a run after a Stop at the wrong moment of a start step still looks up web
//! addresses.
//!
//! Every test needs Steplet's Linux and the network (each run first fetches a small public repository), and they run
//! one at a time: `cargo test -p steplet-run --offline --test restart -- --ignored --nocapture --test-threads=1`.

mod support;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    ApiKey, Check, Event, Live, Progress, Refusal, Report, RunError, RunLock, RunPaths, RunRequest, RunSetup,
    StopSignal, discard, run_live_locked, wsl,
};
use support::{MADE_UP_KEY, logs_dir, root_sh, scripted_model};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");

/// The scripted model's last words.
const THE_END: &str = "Task 6: the end of the script.";

/// A request for Hello-World whose file declares `commands`, so the checker allows them: they ask the gate and run in
/// the fence. With none, the file has no `allow:` at all. Each is written in single quotes, with a quote inside it
/// doubled.
fn request(commands: &[&str]) -> RunRequest {
    let declared: String = commands.iter().map(|c| format!("    - '{}'\n", c.replace('\'', "''"))).collect();
    let allow = if commands.is_empty() { String::new() } else { format!("allow:\n  commands:\n{declared}") };
    let f = format!("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n{allow}---\n");
    RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: HELLO.0.into(),
        commit: HELLO.1.into(),
        task_text: "Follow the script.".into(),
        max_turns: 20,
        deadline: Duration::from_secs(180),
        allow_model_switch: false,
    }
}

/// What a run needs besides the request: the gate program installed as Steplet does, and the scripted model.
fn setup(distro: &str, port: u16) -> RunSetup {
    RunSetup {
        gate_exe: support::install_gate(distro),
        logs: logs_dir(),
        key: Some(ApiKey::new(MADE_UP_KEY.into())),
        decision_deadline: Duration::from_secs(10),
        model_url: Some(format!("http://127.0.0.1:{port}")),
    }
}

fn bash(command: &str) -> Value {
    json!({"tool": "Bash", "input": {"command": command, "description": "A declared command"}})
}

/// Removes a run when dropped, whether the test passed or failed, so a failed test leaves Steplet's Linux as it found
/// it (a run that ended normally has already removed itself).
struct RemovedAtTheEnd<'a>(&'a RunPaths);

impl Drop for RemovedAtTheEnd<'_> {
    fn drop(&mut self) {
        let _ = discard(self.0);
    }
}

/// Root's look from outside, as a short shell script. It never fails the test where it runs (on the run's own thread,
/// in the middle of a run): an error comes back as the look's text.
fn look(distro: &str, body: &str) -> String {
    match wsl::run(distro, "root", &["/bin/bash", "-c", body], b"") {
        Ok(out) if out.status == Some(0) => out.stdout.trim_end().to_owned(),
        Ok(out) => format!("the look failed: {:?} {}", out.status, out.stderr.trim_end()),
        Err(e) => format!("the look failed: {e}"),
    }
}

/// What a run sent to the screen, and root's looks taken while it ran.
#[derive(Default)]
struct Seen {
    events: Vec<Event>,
    looks: Vec<String>,
}

/// Whether `e` is the result of the run's call of the declared command `command`.
fn result_of(events: &[Event], e: &Event, command: &str) -> bool {
    let Event::ToolResult { id, .. } = e else { return false };
    events
        .iter()
        .any(|u| matches!(u, Event::ToolUse { id: used, input, .. } if used == id && input["command"] == command))
}

/// Root's count of what is left of runs: in `/work`, in the runs folder and run folders in `/tmp`.
const LEFT: &str = "echo \"$(ls -A /work | wc -l) $(ls -A /var/lib/steplet/runs | wc -l) \
                    $(ls -d /tmp/steplet-run-* 2>/dev/null | wc -l)\"";

/// What a run returns to its test: the report, the stand-in's log of the requests it answered, and `LEFT` counted as
/// soon as the run returned, before the test's own clean-up.
struct Ran {
    report: Report,
    asked: Vec<Value>,
    left: String,
}

/// One run as the app's Run button makes it, with `script` as the stand-in model's steps (`THE_END` added) and
/// `on_progress` seeing each piece of progress.
fn run_once(held: &RunLock, run_id: &str, port: u16, req: &RunRequest, mut script: Vec<Value>, live: Live) -> Ran {
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, run_id).unwrap();
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    script.push(json!({"text": THE_END}));
    let _model = scripted_model(&distro, run_id, &json!({ "steps": script }).to_string(), port);
    let _removed = RemovedAtTheEnd(&paths);
    let start = Instant::now();
    let report = run_live_locked(held, req, &paths, &policy, &setup(&distro, port), Arc::new(live)).unwrap();
    let took = start.elapsed();
    let left = root_sh(&distro, LEFT);
    let asked: Vec<Value> = root_sh(&distro, &format!("cat /opt/steplet-test/{run_id}.model.log"))
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    println!("--- run {run_id}: {:.1} s\n--- the report:\n{report:#?}", took.as_secs_f64());
    println!("--- the stand-in model was asked (agent loop only; model, step, answer):");
    for a in asked.iter().filter(|a| a["kind"] == "agent loop") {
        println!("    {} {} {}", a["model"], a["step"], a["answer"]);
    }
    println!("--- left as soon as the run returned (in /work, the runs folder, /tmp): {left}");
    Ran { report, asked, left }
}

/// `LEFT` when nothing is left.
const NOTHING_LEFT: &str = "0 0 0";

/// Each call of the report as (tool, command, check, exit code, output).
fn calls(r: &Report) -> Vec<(&str, &str, &Check, Option<i32>, &str)> {
    r.commands.iter().map(|c| (c.tool.as_str(), c.command.as_str(), &c.check, c.exit_code, c.output.as_str())).collect()
}

/// The declared command of the restart test's runs: it appends one line to `steps.txt`, inside the fence.
const APPEND: &str = r#"python3 -c "print('one line', file=open('steps.txt', 'a'))""#;
/// Run 2's first call: whether `steps.txt` is there before the run's own change.
const IS_IT_THERE: &str = r#"python3 -c "import os; print(os.path.exists('steps.txt'))""#;
/// Run 2's last call: how many lines `steps.txt` holds.
const LINES: &str = r#"python3 -c "print(len(open('steps.txt').readlines()))""#;
/// Run 1's second call, so the run still has a call to make when Stop is pressed; Steplet refuses it after the Stop,
/// so it never runs.
const WAIT: &str = r#"python3 -c "import time; time.sleep(8)""#;
/// Claude Code's words, as the report shows them, for a command that printed nothing.
const NO_OUTPUT: &str = "(Bash completed with no output)";

/// C6: a run stopped part-way starts again clean from a fresh working copy, with no duplicate effects inside it. Run 1
/// appends one line to `steps.txt`, and the test presses Stop as soon as that call's result reaches the screen. Run 2
/// of the same file, with the same run id (so the same folders: if Steplet's preparing ever kept a copy it found
/// there, run 2 would pick up run 1's): its first call asks whether `steps.txt` is there, then it appends the line and
/// counts the lines. Pass: report 1 says you stopped it, with no fault; Steplet removed run 1 as soon as it was
/// stopped, and run 2 as soon as it ended (root's count, taken before the test's own clean-up); run 2's copy held only
/// the repository's own files when it was prepared (root's look from outside) and its program found no `steps.txt`
/// before its own change; after it, the file holds one line, not two (the program's count and root's look). The
/// control: when Stop was pressed, run 1's line was in its copy (root's look) and Steplet's own list of changed files
/// names `steps.txt`, so a fresh copy is not one that never changed.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_run_stopped_after_its_first_change_starts_again_clean() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let held = support::hold_linux();
    let distro = support::steplet_linux();
    let repo = RunPaths::new(&distro, "p5t6i").unwrap().work_repo().to_owned();
    let req = request(&[APPEND, IS_IT_THERE, LINES, WAIT]);

    // Run 1: Stop as soon as the append's result is on the screen, then root counts the file's lines.
    let first = Arc::new(Mutex::new(Seen::default()));
    let stop = StopSignal::new();
    let live = {
        let (seen, pressing, distro, repo) = (first.clone(), stop.clone(), distro.clone(), repo.clone());
        Live {
            on_progress: Box::new(move |p| {
                let Progress::Event(e) = p else { return };
                let mut seen = seen.lock().unwrap();
                if result_of(&seen.events, &e, APPEND) {
                    pressing.stop();
                    let lines = look(&distro, &format!("wc -l < {repo}/steps.txt"));
                    seen.looks.push(lines);
                }
                seen.events.push(e);
            }),
            stop,
            ..Live::unattended()
        }
    };
    let Ran { report: report_1, left: left_1, .. } =
        run_once(&held, "p5t6i", 47944, &req, vec![bash(APPEND), bash(WAIT)], live);
    println!("--- run 1, root's look when Stop was pressed (lines in steps.txt): {:?}", first.lock().unwrap().looks);

    // Run 2: root looks at the copy once it is prepared, and counts the lines once the append has run.
    let second = Arc::new(Mutex::new(Seen::default()));
    let live = {
        let (seen, distro, repo) = (second.clone(), distro.clone(), repo.clone());
        Live {
            on_progress: Box::new(move |p| {
                let mut seen = seen.lock().unwrap();
                match p {
                    Progress::Prepared(_) => {
                        let copy = look(&distro, &format!("ls -A {repo}"));
                        seen.looks.push(copy);
                    }
                    Progress::Event(e) => {
                        if result_of(&seen.events, &e, APPEND) {
                            let lines = look(&distro, &format!("wc -l < {repo}/steps.txt"));
                            seen.looks.push(lines);
                        }
                        seen.events.push(e);
                    }
                    _ => {}
                }
            }),
            ..Live::unattended()
        }
    };
    let Ran { report: report_2, left: left_2, .. } =
        run_once(&held, "p5t6i", 47945, &req, vec![bash(IS_IT_THERE), bash(APPEND), bash(LINES)], live);
    let looks_2 = second.lock().unwrap().looks.clone();
    println!("--- run 2, root's looks (the prepared copy, then lines in steps.txt): {looks_2:?}");

    // The control: run 1 changed its copy before the Stop.
    assert_eq!(first.lock().unwrap().looks, ["1"], "run 1's line was in its copy when Stop was pressed");
    assert_eq!(report_1.changed_files, Some(vec!["?? steps.txt".to_owned()]), "Steplet's own list names the change");
    assert!(report_1.stopped_by_you && !report_1.completed, "report 1 says you stopped it: {report_1:?}");
    assert_eq!(calls(&report_1).first(), Some(&("Bash", APPEND, &Check::Allowed, Some(0), NO_OUTPUT)));
    assert!(report_1.faults.is_empty(), "{:?}", report_1.faults);
    assert_eq!(left_1, NOTHING_LEFT, "Steplet removed run 1 as soon as it was stopped");

    assert_eq!(looks_2.first().map(String::as_str), Some(".git\nREADME"), "run 2's copy was fresh when prepared");
    assert_eq!(
        calls(&report_2),
        [
            ("Bash", IS_IT_THERE, &Check::Allowed, Some(0), "False"),
            ("Bash", APPEND, &Check::Allowed, Some(0), NO_OUTPUT),
            ("Bash", LINES, &Check::Allowed, Some(0), "1"),
        ],
        "run 2's program found no steps.txt before its own change, and one line after it"
    );
    assert_eq!(looks_2.get(1).map(String::as_str), Some("1"), "after run 2 the file holds one line, not two");
    assert!(report_2.completed && !report_2.stopped_by_you, "{report_2:?}");
    assert_eq!(report_2.changed_files, Some(vec!["?? steps.txt".to_owned()]));
    assert!(report_2.faults.is_empty(), "{:?}", report_2.faults);
    assert_eq!(left_2, NOTHING_LEFT, "Steplet removed run 2 as soon as it ended");
}

/// The stand-in model's refusal, with the fields Claude Code 2.1.220 reads (phase 0's S6 script); a repeat request at
/// the same step gets an answer instead.
fn refusal() -> Value {
    let details = json!({"type": "refusal", "category": "cyber", "explanation": "Task 6: a scripted refusal."});
    json!({"refusal": {"message": {"stop_details": details}, "delta": {"stop_details": details}},
           "text_after_refusal": "Task 6: the answer after the refusal."})
}

/// The models the stand-in was asked by in the agent loop, each once, in order, with what it answered each.
fn asked_by(asked: &[Value]) -> Vec<(String, String)> {
    let mut seen: Vec<(String, String)> = Vec::new();
    for a in asked.iter().filter(|a| a["kind"] == "agent loop") {
        let pair = (a["model"].as_str().unwrap_or("").to_owned(), a["answer"].as_str().unwrap_or("").to_owned());
        if !seen.contains(&pair) {
            seen.push(pair);
        }
    }
    seen
}

/// A model's name without Claude Code's mark for its context size (`[1m]`), as the API request names it.
fn plain(model: &str) -> &str {
    model.split('[').next().unwrap_or(model)
}

/// An unattended run's `Live` that keeps every event the run sends to the screen.
fn keeping_events() -> (Live, Arc<Mutex<Vec<Event>>>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let keep = events.clone();
    let on_progress = Box::new(move |p| {
        if let Progress::Event(e) = p {
            keep.lock().unwrap().push(e);
        }
    });
    (Live { on_progress, ..Live::unattended() }, events)
}

/// How the run ended, as Claude Code's own result records say it (`stop_reason`), in order.
fn ends(events: &Mutex<Vec<Event>>) -> Vec<String> {
    let events = events.lock().unwrap();
    events
        .iter()
        .filter_map(|e| match e {
            Event::Result { stop_reason, .. } => Some(stop_reason.clone()),
            _ => None,
        })
        .collect()
}

/// E1: the stand-in model refuses at its first step, and Steplet does not let Claude Code switch models (the default).
/// Pass: the run ends at once and does not complete; Claude Code's own record says it ended on the refusal, and
/// nothing else ended it (no tripwire, no Stop, no fault); the report lists one model, and the refusal with that model
/// and its category, and no other model; no tool call ran, the copy did not change, and Steplet removed the run. The
/// control: the stand-in's log shows it was asked by that one model only, refused, and was never sent a tool's result.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_model_refusal_ends_the_run_and_the_report_names_the_model() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let held = support::hold_linux();
    let req = request(&[]);
    let (live, events) = keeping_events();
    let start = Instant::now();
    let Ran { report, asked, left } = run_once(&held, "p5t6m", 47946, &req, vec![refusal()], live);
    let took = start.elapsed();

    let models = asked_by(&asked);
    assert_eq!(models.len(), 1, "the stand-in was asked by one model, and only refused: {models:?}");
    let (model, answer) = &models[0];
    assert_eq!(answer, "refusal");
    assert!(asked.iter().all(|a| a["kind"] != "agent loop" || a["step"] == 0), "no tool's result was ever sent");
    assert!(took < Duration::from_secs(90), "the run ended at once, not at its time limit: {took:?}");
    assert!(!report.completed, "{report:?}");
    assert_eq!(ends(&events), ["refusal"], "Claude Code's own record: the run ended on the refusal");
    assert!(report.stopped_by.is_none() && !report.stopped_by_you, "nothing else ended it: {report:?}");
    assert!(report.faults.is_empty(), "the run ended without a fault: {:?}", report.faults);
    assert_eq!(report.models.iter().map(|m| plain(m)).collect::<Vec<_>>(), [model.as_str()], "{:?}", report.models);
    assert_eq!(
        report.refusals,
        [Refusal {
            model: report.models[0].clone(),
            switched_to: None,
            category: "cyber".into(),
            explanation: "Task 6: a scripted refusal.".into(),
        }]
    );
    assert!(report.commands.is_empty(), "no tool call ran: {:?}", report.commands);
    assert_eq!(report.changed_files, Some(vec![]), "Steplet looked at the copy after the run: nothing changed");
    assert_eq!(left, NOTHING_LEFT, "Steplet removed the run");
}

/// E1, D7: the same refusal with `allow_model_switch`, so Claude Code asks another model at the same step, which
/// answers. Pass: the report lists both models (the one that refused first) and the refusal with the switch; Claude
/// Code's own record says the run ended normally, and nothing else ended it (no tripwire, no Stop, no fault); no tool
/// call ran, the copy did not change, and Steplet removed the run. The control: the stand-in's log shows the first
/// model refused and a second, different one answered.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn with_another_model_allowed_both_models_are_listed() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let held = support::hold_linux();
    let mut req = request(&[]);
    req.allow_model_switch = true;
    let (live, events) = keeping_events();
    let Ran { report, asked, left } = run_once(&held, "p5t6s", 47947, &req, vec![refusal()], live);

    let models = asked_by(&asked);
    assert_eq!(models.len(), 2, "two models, one after the other: {models:?}");
    let ((first, refused), (second, answered)) = (&models[0], &models[1]);
    assert_eq!((refused.as_str(), answered.as_str()), ("refusal", "text"));
    assert_ne!(first, second, "the second model is another one");
    assert_eq!(
        report.models.iter().map(|m| plain(m)).collect::<Vec<_>>(),
        [first.as_str(), second.as_str()],
        "{:?}",
        report.models
    );
    assert_eq!(
        report.refusals,
        [Refusal {
            model: report.models[0].clone(),
            switched_to: Some(report.models[1].clone()),
            category: "cyber".into(),
            explanation: "Task 6: a scripted refusal.".into(),
        }]
    );
    assert!(report.completed, "the other model's answer ends the run normally: {report:?}");
    assert_eq!(ends(&events), ["end_turn"], "Claude Code's own record: the run ended normally");
    assert!(report.stopped_by.is_none() && !report.stopped_by_you, "nothing else ended it: {report:?}");
    assert!(report.faults.is_empty(), "the run ended without a fault: {:?}", report.faults);
    assert!(report.commands.is_empty(), "no tool call ran: {:?}", report.commands);
    assert_eq!(report.changed_files, Some(vec![]), "Steplet looked at the copy after the run: nothing changed");
    assert_eq!(left, NOTHING_LEFT, "Steplet removed the run");
}

/// The name lookup tests' declared command (the Nebius plan, Task 6b): it reads the repository's README.
const READ_ME: &str = "cat README";

/// A Stop that ends a start step just after it removed WSL's `/mnt/wsl`, before Steplet's own folder is back.
const FIRST_GAP: &str = "umount -R -l /mnt/wsl";
/// A Stop that ends a start step just after Steplet's own folder is back, before the lookup file is in it.
const SECOND_GAP: &str = "umount -R -l /mnt/wsl && mount -t tmpfs -o size=64k,mode=755 steplet-wsl /mnt/wsl";

/// One run that reads the README, as the app's Run button makes it. It must complete with no fault, its one command
/// run and ended with exit code 0, and Steplet must have removed it.
fn reads_its_readme(held: &RunLock, run_id: &str, port: u16) {
    let Ran { report, left, .. } =
        run_once(held, run_id, port, &request(&[READ_ME]), vec![bash(READ_ME)], Live::unattended());
    assert!(report.completed && report.faults.is_empty(), "run {run_id} completed with no fault: {report:?}");
    let ran = calls(&report);
    assert_eq!(ran.len(), 1, "{ran:?}");
    let (tool, command, _, code, output) = ran[0];
    assert_eq!((tool, command, code), ("Bash", READ_ME, Some(0)), "{output}");
    assert!(output.contains("Hello World"), "{output}");
    assert_eq!(left, NOTHING_LEFT, "Steplet removed run {run_id}");
}

/// Breaks name lookup as a Stop at the wrong moment of a start step does, with `break_it` (root's shell). The control:
/// the lookup file no longer reads, and no name looks up.
fn stop_in_the_gap(distro: &str, break_it: &str) {
    root_sh(distro, break_it);
    let reads = root_sh(distro, "cat /etc/resolv.conf > /dev/null 2>&1 && echo reads || echo 'does not read'");
    let lookup = support::lookup(distro);
    println!("--- after the break ({break_it}): the lookup file {reads}; a lookup {lookup}");
    assert_eq!((reads.as_str(), lookup.as_str()), ("does not read", "fails"), "the break took name lookup away");
}

/// Root's look at name lookup after a run: what is mounted at `/mnt/wsl`, what it holds, and the lookup file's text.
const LOOKUP_LOOK: &str = "findmnt -rn -o SOURCE /mnt/wsl; ls -A /mnt/wsl; cat /etc/resolv.conf";

/// The Nebius plan, Task 6b (N36), from the gap `break_it`. Run 1 reads the README, and so leaves Steplet's copy of the
/// lookup file; the test then breaks name lookup as the Stop does (the control: nothing looks up). Pass: run 2
/// completes, its command allowed and ended with exit code 0, and Steplet removed it; afterwards `/mnt/wsl` is
/// Steplet's own folder holding only the lookup file, with the text it had before the Stop, and names look up.
fn after_a_stop_the_next_run_looks_up_names(run_id: &str, port: u16, break_it: &str) {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let held = support::hold_linux();
    let distro = support::steplet_linux();
    let found = support::lookup_put_back(&distro);
    reads_its_readme(&held, run_id, port);
    stop_in_the_gap(&distro, break_it);
    reads_its_readme(&held, run_id, port + 1);
    let after = root_sh(&distro, LOOKUP_LOOK);
    println!("--- after run 2 (/mnt/wsl's source, what it holds, the lookup file):\n{after}");
    assert_eq!(after, format!("steplet-wsl\nresolv.conf\n{}", found.text));
    assert_eq!(support::lookup(&distro), "works", "names look up after run 2");
}

/// A Stop that ends a run's start step just after it removed WSL's `/mnt/wsl` takes away the only lookup file. Before
/// Task 6b every later run then failed at its first web address, until Windows or WSL restarted.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_run_after_a_stop_in_the_gap_still_looks_up_names() {
    after_a_stop_the_next_run_looks_up_names("n6ba", 47972, FIRST_GAP);
}

/// The same from the start step's second gap: Steplet's own folder is back at `/mnt/wsl`, empty.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_run_after_a_stop_just_after_the_new_folder_still_looks_up_names() {
    after_a_stop_the_next_run_looks_up_names("n6bb", 47974, SECOND_GAP);
}

/// N36: with name lookup broken and no copy of the lookup file to put back (a Stop that broke it before Steplet kept a
/// copy), the run is refused before anything is fetched, with words that say why, and nothing is left. The test
/// removes the copy; the next run that finds lookup working makes it again.
#[test]
#[ignore = "needs Steplet's Linux"]
fn with_no_copy_a_run_after_a_stop_in_the_gap_is_refused_in_plain_words() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let held = support::hold_linux();
    let distro = support::steplet_linux();
    let _found = support::lookup_put_back(&distro);
    root_sh(&distro, "rm -f /var/lib/steplet/resolv.conf");
    stop_in_the_gap(&distro, FIRST_GAP);
    let req = request(&[READ_ME]);
    let paths = RunPaths::new(&distro, "n6bc").unwrap();
    let _removed = RemovedAtTheEnd(&paths);
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let got = run_live_locked(&held, &req, &paths, &policy, &setup(&distro, 47976), Arc::new(Live::unattended()));
    println!("--- the run, with lookup broken and no copy: {:?}", got.as_ref().err());
    match got {
        Err(RunError::ListedSite { site, why }) => {
            assert_eq!(site, "");
            assert_eq!(
                why,
                "Steplet will not run this file: Steplet could not keep name lookup working in its Linux (the name \
                 lookup file is gone, and Steplet has no copy of it)."
            );
        }
        other => panic!("the run must be refused before it starts: {other:?}"),
    }
    assert_eq!(root_sh(&distro, LEFT), NOTHING_LEFT, "nothing was prepared");
}

/// N36: Steplet's copy of the lookup file is root's alone. The next run looks up web addresses with whatever it holds,
/// so the steplet account, which runs the agent's commands, must not be able to change it. After a run, the copy is
/// root's, 644, and holds the lookup text; the steplet account, outside the fence (where it reaches most), cannot add
/// to it, remove it, or put another file in its place or beside it, and the copy is unchanged. The control: the same
/// account writes and removes a file of its own in `/tmp`.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn the_lookup_copy_is_roots_alone() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let held = support::hold_linux();
    let distro = support::steplet_linux();
    let found = support::lookup_put_back(&distro);
    reads_its_readme(&held, "n6bd", 47977);
    const COPY_LOOK: &str = "stat -c '%U:%G %a' /var/lib/steplet/resolv.conf; cat /var/lib/steplet/resolv.conf";
    let copy = root_sh(&distro, COPY_LOOK);
    println!("--- Steplet's copy after a run (owner, mode, text):\n{copy}");
    assert_eq!(copy, format!("root:root 644\n{}", found.text));

    let as_steplet = |sh: &str| {
        let out = wsl::run(&distro, "steplet", &["/bin/sh", "-c", sh], b"").unwrap();
        println!("--- as the steplet account: {sh:?}: {:?} {}", out.status, out.stderr.trim());
        out.status == Some(0)
    };
    assert!(as_steplet("echo x > /tmp/n6bd-control && rm /tmp/n6bd-control"), "the control");
    for sh in [
        "echo 'nameserver 192.0.2.1' >> /var/lib/steplet/resolv.conf",
        "rm -f /var/lib/steplet/resolv.conf",
        "echo 'nameserver 192.0.2.1' > /tmp/n6bd-other && mv -f /tmp/n6bd-other /var/lib/steplet/resolv.conf",
        "ln -s /tmp/n6bd-other /var/lib/steplet/resolv.conf.new",
    ] {
        assert!(!as_steplet(sh), "the steplet account changed Steplet's copy: {sh}");
    }
    root_sh(&distro, "rm -f /tmp/n6bd-other");
    assert_eq!(root_sh(&distro, COPY_LOOK), copy, "the copy is unchanged");
}
