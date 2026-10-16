//! The tripwires (plan Task 9): a hook event, a command that skipped the checker, or any other action that skipped
//! the gate stops the run. The tests marked `#[ignore]` need Steplet's Linux and the network; they run one at a time:
//! `cargo test -p steplet-run --offline --test tripwires -- --ignored --nocapture --test-threads=1`.

mod support;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{Asked, Event, RunPaths, RunRequest, decide, discard, launch, prepare};
use support::{logs_dir, scripted_model, serve_gate};

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
    let (link, gate) = serve_gate(&distro, "t9m", Duration::from_secs(10), move |r| decide(&policy, r));
    let _model = scripted_model(&distro, "t9m", &script.to_string(), 47930);
    let req = request();
    prepare(&req, &paths, &link).unwrap();
    let handle = launch(&req, &paths, &logs_dir(), Some("http://127.0.0.1:47930")).unwrap();
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
            Event::ToolResult { id: rid, is_error, text } if rid == id => Some((*is_error, text.clone())),
            _ => None,
        });
        let what = input.get("command").or(input.get("file_path")).or(input.get("pattern")).cloned().unwrap_or_default();
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
