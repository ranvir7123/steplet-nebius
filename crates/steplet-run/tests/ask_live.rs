//! The ask path in real runs (phase 4 Task 4), with the scripted model and the screen played by this test program.
//! Every test needs Steplet's Linux and the network, and they run one at a time:
//! `cargo test -p steplet-run --offline --test ask_live -- --ignored --nocapture --test-threads=1`.

mod support;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    Answers, ApiKey, Asked, BACKSTOP, Check, Live, Progress, Reply, Report, RunPaths, RunRequest, RunSetup, StopSignal,
    discard, launch, prepare, run_live, watch_run,
};
use support::{MADE_UP_KEY, allow_everything, logs_dir, scripted_model, serve_gate};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");
const THE_END: &str = "Task 4: the end of the script.";

/// A request for Hello-World whose file holds `allow` (the lines under `allow:`, if any).
fn request(allow: &str) -> RunRequest {
    let f = format!("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n{allow}---\n");
    RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: HELLO.0.into(),
        commit: HELLO.1.into(),
        task_text: "Follow the script.".into(),
        max_turns: 20,
        deadline: Duration::from_secs(180),
    }
}

/// Step 1 (a record): once Steplet has allowed a request that the file did not ask for, does Claude Code 2.1.220 ask
/// the gate again when the agent makes the very same request? The gate here allows everything, as a receiver who says
/// yes to every question would; Claude Code cannot tell the two apart. Each kind of request gets a run of its own, so
/// a second call that skipped the gate (which stops the run) cannot hide the others.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn does_claude_code_ask_again_after_a_yes() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let pairs: [(&str, &str, Value); 4] = [
        ("p4t4m1", "Bash", json!({"command": "touch asked.txt", "description": "Make a file"})),
        ("p4t4m2", "Write", json!({"file_path": "/work/p4t4m2/repo/notes.txt", "content": "made-up test text\n"})),
        ("p4t4m3", "Read", json!({"file_path": "/etc/hostname"})),
        ("p4t4m4", "WebFetch", json!({"url": "https://example.com/", "prompt": "Summarise it."})),
    ];
    let mut table = Vec::new();
    for (n, (run_id, tool, input)) in pairs.into_iter().enumerate() {
        let port = 47960 + n as u16;
        let paths = RunPaths::new(&distro, run_id).unwrap();
        let req = request("");
        let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
        let script =
            json!({"steps": [{"tool": tool, "input": input}, {"tool": tool, "input": input}, {"text": THE_END}]});
        let _model = scripted_model(&distro, run_id, &script.to_string(), port);
        let (link, gate) = serve_gate(&distro, run_id, Duration::from_secs(10), allow_everything);
        prepare(&req, &paths, &link).unwrap();
        let handle = launch(&req, &paths, &logs_dir(), Some(&format!("http://127.0.0.1:{port}")), &[]).unwrap();
        let outcome =
            watch_run(handle, gate, &policy, paths.work_repo(), req.deadline + BACKSTOP, &Live::unattended(), &[]);
        discard(&paths).unwrap();
        let calls: Vec<String> = outcome
            .events
            .iter()
            .filter_map(|e| match e {
                steplet_run::Event::ToolUse { id, .. } => Some(id.clone()),
                _ => None,
            })
            .collect();
        let reached = |id: &str| {
            outcome.requests.iter().any(|a| matches!(a, Asked::Decision { request, .. } if request.tool_use_id == id))
        };
        let row = format!(
            "{tool}: calls {}; first reached the gate: {}; second reached the gate: {}; stopped by: {:?}",
            calls.len(),
            calls.first().is_some_and(|id| reached(id)),
            calls.get(1).is_some_and(|id| reached(id)),
            outcome.stopped_by
        );
        println!("--- {row}");
        table.push(row);
        assert_eq!(calls.len(), 2, "{tool}: both calls were made");
    }
    println!("--- the record:\n{}", table.join("\n"));
}

/// A live run of Hello-World, with the screen played by this test program: it answers every question yes (`say_yes`)
/// or never answers, within an ask limit of `limit`. Returns the report and what was sent to the screen.
fn run_with_screen(run_id: &str, script: &Value, port: u16, say_yes: bool, limit: Duration) -> (Report, Vec<Progress>) {
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, run_id).unwrap();
    let req = request("");
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let _model = scripted_model(&distro, run_id, &script.to_string(), port);
    let answers = Answers::default();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let (keep, giving) = (seen.clone(), answers.clone());
    let live = Arc::new(Live {
        on_progress: Box::new(move |p| {
            if let Progress::Asking(q) = &p
                && say_yes
            {
                // The screen is another thread; the receiver reads for a moment, then says yes.
                let (giving, id, fingerprint) = (giving.clone(), q.id, q.fingerprint.clone());
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(300));
                    giving.give(id, Reply::Yes { fingerprint });
                });
            }
            keep.lock().unwrap().push(p);
        }),
        stop: StopSignal::new(),
        answers: Some(answers),
        ask_limit: limit,
    });
    let setup = RunSetup {
        gate_exe: support::install_gate(&distro),
        logs: logs_dir(),
        key: Some(ApiKey::new(MADE_UP_KEY.into())),
        decision_deadline: limit + Duration::from_secs(15),
        model_url: Some(format!("http://127.0.0.1:{port}")),
    };
    let report = run_live(&req, &paths, &policy, &setup, live).unwrap();
    let seen = seen.lock().unwrap().clone();
    (report, seen)
}

/// The ask screen in a real run: the agent runs a command the file does not ask for; the screen says yes to the
/// exact request it was shown, and the command runs, inside the fence, as that request.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_yes_on_the_ask_screen_lets_exactly_that_command_run() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": "touch asked.txt", "description": "Make a file"}},
        {"text": THE_END}]});
    let (r, seen) = run_with_screen("p4t4a", &script, 47965, true, Duration::from_secs(20));
    println!("--- sent to the screen: {seen:#?}\n--- the report:\n{r:#?}");
    let asked = seen.iter().filter(|p| matches!(p, Progress::Asking(_))).count();
    assert_eq!(asked, 1, "one question");
    let c = &r.commands[0];
    assert_eq!(
        (c.command.as_str(), &c.check, c.exit_code),
        ("touch asked.txt", &Check::AllowedByYou { reused: false }, Some(0))
    );
    assert_eq!(r.changed_files, Some(vec!["?? asked.txt".to_owned()]), "Steplet's own git status saw the file");
    assert!(matches!(r.answers[..], [steplet_run::Answer { reply: Reply::Yes { .. }, .. }]), "{:?}", r.answers);
    assert!(r.completed);
}

/// No answer within the ask limit is a no: the command does not run.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn no_answer_refuses_it() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": "touch asked.txt", "description": "Make a file"}},
        {"text": THE_END}]});
    let (r, _) = run_with_screen("p4t4b", &script, 47966, false, Duration::from_secs(2));
    println!("--- the report:\n{r:#?}");
    let c = &r.commands[0];
    assert_eq!((c.command.as_str(), &c.check), ("touch asked.txt", &Check::Refused));
    assert!(c.output.contains("no answer in time"), "{:?}", c.output);
    assert_eq!(r.changed_files, Some(vec![]), "asked.txt was never made");
    assert!(matches!(r.answers[..], [steplet_run::Answer { reply: Reply::NoAnswer, .. }]), "{:?}", r.answers);
}
