//! The tripwires (plan Task 9): a hook event, a command that skipped the checker, or any other action that skipped
//! the gate stops the run. The tests marked `#[ignore]` need Steplet's Linux and the network; they run one at a time:
//! `cargo test -p steplet-run --offline --test tripwires -- --ignored --nocapture --test-threads=1`.

mod support;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    ApiKey, Asked, BACKSTOP, Check, Event, GateAnswer, Live, ReportCommand, RunOutcome, RunPaths, RunRequest, RunSetup,
    Tripwire, decide, discard, launch, prepare, run, watch_run,
};
use support::{MADE_UP_KEY, logs_dir, scripted_model, serve_gate};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");

fn request() -> RunRequest {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n---\n";
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

/// A measurement, kept as a test: which of the agent's actions Claude Code 2.1.220 sends through the gate with
/// Steplet's settings, and which it runs or refuses on its own. The tripwires are designed from this.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn which_actions_ask_the_gate() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "t9m").unwrap();
    let repo = paths.work_repo().to_owned();
    let script = serde_json::json!({"steps": [
        {"tool": "Bash", "input": {"command": "ls", "description": "List"}},
        {"tool": "Bash", "input": {"command": "cat README", "description": "Read"}},
        {"tool": "Bash", "input": {"command": "ls src", "description": "List a folder that is not there"}},
        {"tool": "Bash", "input": {"command": "git status --short", "description": "Git status"}},
        {"tool": "Bash", "input": {"command": "cat README; echo done", "description": "Two reads"}},
        {"tool": "Bash", "input": {"command": "cat README | wc -l", "description": "A pipe"}},
        {"tool": "Bash", "input": {"command": "ls /var/lib/steplet", "description": "List outside"}},
        {"tool": "Bash", "input": {"command": "touch made.txt", "description": "Write"}},
        {"tool": "Read", "input": {"file_path": format!("{repo}/README")}},
        {"tool": "Glob", "input": {"pattern": "*", "path": repo}},
        {"tool": "Grep", "input": {"pattern": "Hello", "path": repo}},
        {"tool": "Read", "input": {"file_path": "/etc/os-release"}},
        {"tool": "Read", "input": {"file_path": paths.settings()}},
        {"tool": "Read", "input": {"file_path": format!("{repo}/.git/config")}},
        {"tool": "Edit", "input": {"file_path": format!("{repo}/.git/config"), "old_string": "[core]", "new_string": "[core]\n\t# t9 test edit"}},
        {"tool": "Write", "input": {"file_path": format!("{repo}/written.txt"), "content": "t9\n"}},
        {"text": "Task 9: measured."}]});
    let policy = Policy::from_steplet(&request().steplet, paths.work_repo()).unwrap();
    let checker = support::checker(&distro, policy);
    let (link, gate) = serve_gate(&distro, "t9m", Duration::from_secs(10), move |r| decide(&checker, r));
    let _model = scripted_model(&distro, "t9m", &script.to_string(), 47930);
    let req = request();
    prepare(&req, &paths, &link).unwrap();
    let handle = launch(&req, &paths, &logs_dir(), Some("http://127.0.0.1:47930"), &[]).unwrap();
    let start = Instant::now();
    let mut events = Vec::new();
    while start.elapsed() < Duration::from_secs(150) {
        match handle.events().recv_timeout(Duration::from_secs(1)) {
            Ok(e) => events.push(e),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => break,
        }
    }
    let code = handle.wait().unwrap();
    std::thread::sleep(Duration::from_millis(500));
    let asked = gate.requests();
    println!("--- exit code {code:?}; each tool call, whether it asked the gate, and what came back:");
    let mut rows = Vec::new();
    for e in &events {
        let Event::ToolUse { id, name, input } = e else { continue };
        let answer = asked.iter().find_map(|a| match a {
            Asked::Decision { request, answer, .. } if &request.tool_use_id == id => Some(format!("{answer:?}")),
            _ => None,
        });
        let result = events.iter().find_map(|r| match r {
            Event::ToolResult { id: rid, is_error, text, .. } if rid == id => Some((*is_error, text.clone())),
            _ => None,
        });
        let what =
            input.get("command").or(input.get("file_path")).or(input.get("pattern")).cloned().unwrap_or_default();
        let (is_error, text) = result.unwrap_or((true, "(no result)".into()));
        let text: String = text.replace('\n', " | ").chars().take(150).collect();
        let shown_answer: String = answer.clone().unwrap_or_else(|| "-".into()).chars().take(60).collect();
        println!(
            "    {name:6} {:46} asked={:3} answer={shown_answer}\n           is_error={is_error} {text}",
            what.to_string(),
            if answer.is_some() { "yes" } else { "NO" }
        );
        rows.push((name.clone(), what.to_string(), answer.is_some(), is_error));
    }
    let made = steplet_run::wsl::run(&distro, "root", &["/usr/bin/ls", "-A", paths.work_repo()], b"").unwrap();
    println!("--- the working copy holds: {}", made.stdout.split_whitespace().collect::<Vec<_>>().join(" "));
    assert_eq!(rows.len(), 16, "every scripted tool call was made");
    let git_config = support::run(&distro, "root", &["/usr/bin/cat", &format!("{}/.git/config", paths.work_repo())]);
    println!("--- .git/config still has no test edit: {}", !git_config.contains("t9 test edit"));
    discard(&paths).unwrap();
}

/// The scripted model's last words; a run stopped early never gets to them.
const NOT_REACHED: &str = "Task 9: the end of the script.";

/// A script of `first`, then `then` 30 times (each a turn of its own, so the run is still going when the tripwire
/// stops it), then the last words.
fn script_after(first: Value, then: &str) -> Value {
    let mut steps = vec![first];
    steps.extend(
        (0..30).map(|i| json!({"tool": "Bash", "input": {"command": then, "description": format!("Step {i}")}})),
    );
    steps.push(json!({"text": NOT_REACHED}));
    json!({"steps": steps})
}

/// A request whose file declares one writing command, which the checker allows: it asks the gate.
fn long_request() -> RunRequest {
    let f = "---
steplet: 1
title: t
sender: s
expires: 2026-12-31
steps:
  - a
allow:
  commands:
    - touch allowed.txt
---
";
    RunRequest { steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(), max_turns: 60, ..request() }
}

/// What `run` needs besides the request: the gate program installed as Steplet does, and the scripted model.
fn setup(distro: &str, port: u16) -> RunSetup {
    RunSetup {
        gate_exe: support::install_gate(distro),
        logs: logs_dir(),
        key: Some(ApiKey::new(MADE_UP_KEY.into())),
        decision_deadline: Duration::from_secs(10),
        model_url: Some(format!("http://127.0.0.1:{port}")),
    }
}

/// The id of the first tool call running `command`.
fn id_of(outcome: &RunOutcome, command: &str) -> String {
    outcome
        .events
        .iter()
        .find_map(|e| match e {
            Event::ToolUse { id, input, .. } if input["command"] == command => Some(id.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no tool call ran {command:?}"))
}

fn asked_the_gate(outcome: &RunOutcome, id: &str) -> bool {
    outcome.requests.iter().any(|a| matches!(a, Asked::Decision { request, .. } if request.tool_use_id == id))
}

fn tool_calls(outcome: &RunOutcome) -> usize {
    outcome.events.iter().filter(|e| matches!(e, Event::ToolUse { .. })).count()
}

fn reached_the_end(outcome: &RunOutcome) -> bool {
    outcome.events.iter().any(|e| matches!(e, Event::AgentText(t) if t == NOT_REACHED))
}

fn print_outcome(outcome: &RunOutcome, took: Duration) {
    println!(
        "--- {:.1} s; exit code {:?}; stopped_by {:?}; faults {:?}; every event:",
        took.as_secs_f64(),
        outcome.exit_code,
        outcome.stopped_by,
        outcome.faults
    );
    for e in &outcome.events {
        let shown: String = format!("{e:?}").chars().take(170).collect();
        println!("    {shown}");
    }
    println!("--- the gate was asked:");
    for a in &outcome.requests {
        if let Asked::Decision { request, answer, .. } = a {
            let shown: String = format!("{answer:?}").chars().take(90).collect();
            println!("    {} {} {shown}", request.tool_use_id, request.tool_name);
        }
    }
}

/// Rewrites run `paths`' settings file as root, as a hostile change would (test only): `change` is Python that
/// edits the settings, `s`. The file stays root's, with its mode.
fn rewrite_settings(paths: &RunPaths, change: &str) {
    let py = format!(
        "import json\np = {:?}\ns = json.load(open(p))\n{change}\njson.dump(s, open(p, 'w'), indent=2)\n",
        paths.settings()
    );
    let out =
        steplet_run::wsl::run(paths.distro_id(), "root", &["/usr/bin/python3", "-I", "-"], py.as_bytes()).unwrap();
    assert_eq!(out.status, Some(0), "{}", out.stderr);
    let now = support::run(paths.distro_id(), "root", &["/usr/bin/stat", "-c", "%U %a", paths.settings()]);
    assert_eq!(now, "root 644");
}

/// A run made by `run`'s own steps, with the settings changed by `change` between `prepare` and the launch.
fn run_with_changed_settings(name: &str, port: u16, script: &Value, change: &str) -> (RunOutcome, Duration) {
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, name).unwrap();
    let req = long_request();
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let checker = support::checker(&distro, policy.clone());
    let deciding = checker.clone();
    let (link, gate) = serve_gate(&distro, name, Duration::from_secs(10), move |r| decide(&deciding, r));
    let _model = scripted_model(&distro, name, &script.to_string(), port);
    prepare(&req, &paths, &link).unwrap();
    rewrite_settings(&paths, change);
    println!("--- the run's settings now say: {}", support::run(&distro, "root", &["/usr/bin/cat", paths.settings()]));
    let start = Instant::now();
    let handle = launch(&req, &paths, &logs_dir(), Some(&format!("http://127.0.0.1:{port}")), &[]).unwrap();
    let outcome =
        watch_run(handle, gate, &checker, paths.work_repo(), req.deadline + BACKSTOP, &Live::unattended(), &[]);
    let took = start.elapsed();
    discard(&paths).unwrap();
    (outcome, took)
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_harmless_read_that_skips_the_gate_does_not_stop_the_run() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "t9a").unwrap();
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": "ls src", "description": "List src"}},
        {"text": NOT_REACHED}]});
    let _model = scripted_model(&distro, "t9a", &script.to_string(), 47931);
    let req = request();
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let start = Instant::now();
    // `run` returns the receiver's report (Task 10); the skip is read from it.
    let report = run(&req, &paths, &policy, &setup(&distro, 47931)).unwrap();
    println!("--- {:.1} s; the report:\n{report:#?}", start.elapsed().as_secs_f64());

    assert_eq!(report.stopped_by, None);
    let ls: Vec<&ReportCommand> = report.commands.iter().filter(|c| c.command == "ls src").collect();
    assert!(
        matches!(ls[..], [ReportCommand { check: Check::NotAsked { harmless: true }, after_stop: false, .. }]),
        "`ls src` skipped the gate as a harmless read, and the skip is recorded, not dropped: {ls:?}"
    );
    assert_eq!(ls[0].exit_code, Some(2), "and it ran: the folder is not there");
    assert_eq!(report.agent_says, NOT_REACHED, "the run went on to its end");
    assert!(report.completed);
    assert_eq!(report.faults, Vec::<String>::new());
    let left = support::root_sh(&distro, "ls -A /work; ls -A /var/lib/steplet/runs");
    assert_eq!(left, "", "run() removed the run");
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_hook_in_the_run_s_settings_stops_the_run() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    // After the hook, a command the file declares: the checker would allow it, but Steplet is stopping the run.
    let script =
        script_after(json!({"tool": "Bash", "input": {"command": "ls", "description": "List"}}), "touch allowed.txt");
    // An inert hook (test data): it runs `true` before each command.
    let hook =
        r#"s["hooks"] = {"PreToolUse": [{"matcher": "Bash", "hooks": [{"type": "command", "command": "true"}]}]}"#;
    let (outcome, took) = run_with_changed_settings("t9b", 47932, &script, hook);
    print_outcome(&outcome, took);

    let hooks: Vec<&Event> = outcome.events.iter().filter(|e| matches!(e, Event::HookEvent { .. })).collect();
    println!("--- the hook events, as Claude Code 2.1.220 sends them: {hooks:?}");
    assert!(matches!(&outcome.stopped_by, Some(Tripwire::Hook(_))), "{:?}", outcome.stopped_by);
    let answers: Vec<&GateAnswer> = outcome
        .requests
        .iter()
        .filter_map(|a| match a {
            Asked::Decision { answer, .. } => Some(answer),
            _ => None,
        })
        .collect();
    println!("--- requests that reached the gate after the hook: {}", answers.len());
    // The scripted model is quick, so its next commands reach the gate while the stop is under way (5 in the run
    // pasted in the results file). With none, this test would show nothing about the gate during a stop.
    assert!(!answers.is_empty(), "a request reached the gate while Steplet was stopping the run");
    for answer in &answers {
        assert!(
            matches!(answer, GateAnswer::Deny { message } if message.contains("Steplet is stopping this run")),
            "from the tripwire on, the gate refuses everything: {answer:?}"
        );
    }
    assert!(!reached_the_end(&outcome), "the run was stopped before the script's end");
    assert!(tool_calls(&outcome) < 31, "{} of 31 tool calls were made", tool_calls(&outcome));
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_writing_command_made_to_skip_the_gate_stops_the_run() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script =
        script_after(json!({"tool": "Bash", "input": {"command": "touch made.txt", "description": "Write"}}), "pwd");
    // An allow rule Steplet never writes (R4): it lets this command run without asking the gate.
    let allow = r#"s["permissions"]["allow"] = ["Bash(touch:*)"]"#;
    let (outcome, took) = run_with_changed_settings("t9c", 47933, &script, allow);
    print_outcome(&outcome, took);

    let id = id_of(&outcome, "touch made.txt");
    assert!(!asked_the_gate(&outcome, &id), "the command skipped the gate");
    assert_eq!(outcome.stopped_by, Some(Tripwire::SkippedCommand("touch made.txt".into())));
    let note = Event::SkippedGate {
        id: id.clone(),
        tool: "Bash".into(),
        what: "touch made.txt".into(),
        harmless: false,
        not_offered: false,
    };
    assert!(outcome.events.contains(&note));
    assert!(!reached_the_end(&outcome), "the run was stopped before the script's end");
    assert!(tool_calls(&outcome) < 31, "{} of 31 tool calls were made", tool_calls(&outcome));
}
