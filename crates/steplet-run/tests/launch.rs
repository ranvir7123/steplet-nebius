//! Starting the agent, reading its events, and stopping it (plan Task 7). The tests marked `#[ignore]` need Steplet's
//! Linux and the network (for the clone); since Task 8 they use the real gate, answering "allow" to everything, as
//! the phase 0 stand-in host did. They run one at a time:
//! `cargo test -p steplet-run --offline --test launch -- --ignored --test-threads=1`.

mod support;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_run::{
    Event, GateLink, GateService, LaunchError, RunHandle, RunPaths, RunRequest, discard, launch, prepare,
};
use support::{Helper, allow_everything, gate_log, install_gate, logs_dir, root_sh, scripted_model, serve_gate};

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn request(seconds: u64) -> RunRequest {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n---\n";
    RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: HELLO.0.into(),
        commit: HELLO.1.into(),
        task_text: "Follow the script.".into(),
        max_turns: 6,
        deadline: Duration::from_secs(seconds),
    }
}

/// Everything the run sends until its channel closes, or `limit` passes.
fn collect(run: &RunHandle, limit: Duration) -> Vec<Event> {
    let start = Instant::now();
    let mut events = Vec::new();
    while let Some(left) = limit.checked_sub(start.elapsed()) {
        match run.events().recv_timeout(left) {
            Ok(e) => events.push(e),
            Err(_) => break,
        }
    }
    events
}

/// The events a person would follow, without Claude Code's other notices and its error output.
fn main_line(events: &[Event]) -> Vec<&Event> {
    events.iter().filter(|e| !matches!(e, Event::Other(_) | Event::Stderr(_))).collect()
}

fn print_all(events: &[Event]) {
    for e in events {
        let shown = format!("{e:?}");
        println!("    {}", if shown.len() > 220 { format!("{}...", &shown[..220]) } else { shown });
    }
}

/// One prepared run with the scripted model and the gate, until the returned guards are dropped.
fn start(
    run_id: &str,
    port: u16,
    script: &str,
    seconds: u64,
    task: &str,
) -> (RunHandle, RunPaths, GateService, Helper) {
    let distro = support::steplet_linux();
    let (gate, host) = serve_gate(&distro, run_id, Duration::from_secs(10), allow_everything);
    let model = scripted_model(&distro, run_id, script, port);
    let paths = RunPaths::new(&distro, run_id).unwrap();
    let mut req = request(seconds);
    req.task_text = task.into();
    prepare(&req, &paths, &gate).unwrap();
    let handle = launch(&req, &paths, &logs_dir(), Some(&format!("http://127.0.0.1:{port}"))).unwrap();
    (handle, paths, host, model)
}

#[test]
fn launch_refuses_values_it_would_misread() {
    // A made-up distribution: a value that got through would reach `wsl.exe` and fail another way.
    let paths = RunPaths::new("{00000000-0000-0000-0000-000000000000}", "t7x").unwrap();
    let logs = std::env::temp_dir().join("steplet-test-t7x-logs");
    for url in ["http://example.com:1", "http://127.0.0.1", "http://127.0.0.1:1/x", "https://127.0.0.1:443", "-x"] {
        let got = launch(&request(60), &paths, &logs, Some(url));
        assert!(matches!(got, Err(LaunchError::Refused(_))), "{url}");
    }
    assert!(matches!(launch(&request(0), &paths, &logs, None), Err(LaunchError::Refused(_))), "no time limit");
    let mut no_turns = request(60);
    no_turns.max_turns = 0;
    assert!(matches!(launch(&no_turns, &paths, &logs, None), Err(LaunchError::Refused(_))), "no turns");
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_scripted_run_streams_its_events_in_order() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = r#"{"steps": [
        {"tool": "Bash", "input": {"command": "git checkout 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d 2>&1; echo \"EXIT=$?\"", "description": "Check out the declared commit"}},
        {"tool": "Bash", "input": {"command": "cat README; echo \"EXIT=$?\"", "description": "Run the tests"}},
        {"text": "Task 7: the scripted run is done."}]}"#;
    let task = "Follow the script.\nSecond line of the Steplet test task t7a, which must arrive whole.";
    let (handle, paths, _host, _model) = start("t7a", 47910, script, 120, task);
    let events = collect(&handle, Duration::from_secs(120));
    let code = handle.wait().unwrap();
    println!("--- every event of run t7a:");
    print_all(&events);
    println!("--- exit code: {code:?}");

    assert_eq!(code, Some(0));
    let line = main_line(&events);
    let kinds: Vec<&str> = line
        .iter()
        .map(|e| match e {
            Event::Started { .. } => "Started",
            Event::ToolUse { .. } => "ToolUse",
            Event::ToolResult { .. } => "ToolResult",
            Event::AgentText(_) => "AgentText",
            Event::Result { .. } => "Result",
            _ => "something else",
        })
        .collect();
    assert_eq!(kinds, ["Started", "ToolUse", "ToolResult", "ToolUse", "ToolResult", "AgentText", "Result"]);
    match line[0] {
        Event::Started { tools, gate_connected, .. } => {
            assert!(*gate_connected, "the gate's MCP server connected");
            assert_eq!(tools, &["Bash", "Edit", "Glob", "Grep", "Read", "WebFetch", "Write"], "H5's seven tools only");
        }
        other => panic!("{other:?}"),
    }
    assert!(
        matches!(line[4], Event::ToolResult { text, .. } if text.contains("Hello World") && text.contains("EXIT=0"))
    );
    assert!(matches!(line[6], Event::Result { subtype, is_error: false, .. } if subtype == "success"));
    let log = std::fs::read_to_string(logs_dir().join("t7a.jsonl")).unwrap();
    assert!(log.lines().all(|l| serde_json::from_str::<serde_json::Value>(l).is_ok()), "the stream is kept unchanged");
    println!("--- {} lines kept in {}", log.lines().count(), logs_dir().join("t7a.jsonl").display());
    // The task reached Claude Code whole, so nothing before the agent's start read any of it (F1).
    let model_log = support::run(paths.distro_id(), "root", &["/usr/bin/cat", "/opt/steplet-test/t7a.model.log"]);
    let first: Vec<String> = model_log
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter_map(|e| e.get("first_user_texts").cloned())
        .flat_map(|texts| texts.as_array().cloned().unwrap_or_default())
        .filter_map(|t| t.as_str().map(str::to_owned))
        .collect();
    println!("--- the first user message, as the scripted model received it: {first:?}");
    assert!(first.iter().any(|t| t.contains(task)), "the whole task arrived: {first:?}");
    discard(&paths).unwrap();

    // Name lookup still works in the same running Linux after a run closed its shared folders: WSL puts the lookup
    // file's link back for each new session, so a second clone needs the copy the start script left.
    let distro = paths.distro_id();
    let after = root_sh(
        distro,
        r#"echo "$(findmnt -rn -o TARGET,SOURCE /mnt/wsl); $(ls -A /mnt/wsl); $(readlink /etc/resolv.conf)""#,
    );
    println!("--- after the run, in a new session: {after}");
    let next = RunPaths::new(distro, "t7a2").unwrap();
    let gate_exe = install_gate(distro);
    let gate =
        GateLink::new(&gate_exe, "steplet-t7a2", "t7a2token", &gate_log("t7a2"), Duration::from_secs(10)).unwrap();
    prepare(&request(60), &next, &gate).expect("a second clone right after a run");
    discard(&next).unwrap();
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn stop_ends_a_run_mid_command_and_leaves_nothing_running() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = r#"{"steps": [
        {"tool": "Bash", "input": {"command": "sleep 120; echo slept", "description": "Stay busy"}},
        {"text": "Not reached."}]}"#;
    let (mut handle, paths, _host, _model) = start("t7b", 47911, script, 300, "Follow the script.");
    let mut events = Vec::new();
    while !events.iter().any(|e| matches!(e, Event::ToolUse { .. })) {
        events.push(handle.events().recv_timeout(Duration::from_secs(60)).expect("the run reaches its command"));
    }
    std::thread::sleep(Duration::from_secs(3));

    // While the command runs, from outside the run: H1 was applied, and the agent has no powers.
    let distro = paths.distro_id();
    let check = r#"echo "mounts under /mnt: [$(findmnt -rn -o TARGET,SOURCE | grep -E '^/mnt(/| )' | tr '\n' ' ')]"
echo "/mnt/wsl holds: [$(ls -A /mnt/wsl | tr '\n' ' ')]"
echo "resolv.conf: $(readlink /etc/resolv.conf || echo plain file)"
echo "name lookup: $(getent hosts github.com > /dev/null && echo works || echo fails)"
echo "/tmp/claude: $([ -e /tmp/claude ] && echo present || echo absent)"
for pid in $(pgrep -f SETTINGS); do
  echo "process $(cat /proc/$pid/comm): user $(stat -c %U /proc/$pid) $(grep -E '^(CapEff|CapBnd|NoNewPrivs)' /proc/$pid/status | tr -s '\t\n' '  ')"
done
echo "root processes of the run: $(pgrep -u root -f SETTINGS | wc -l)""#;
    let seen = root_sh(distro, &check.replace("SETTINGS", paths.settings()));
    println!("--- seen from outside while the command ran:\n{seen}");
    assert!(seen.contains("mounts under /mnt: [/mnt/wsl steplet-wsl ]"), "{seen}");
    assert!(seen.contains("/mnt/wsl holds: [resolv.conf ]") && seen.contains("name lookup: works"), "{seen}");
    assert!(seen.contains("/tmp/claude: absent"), "{seen}");
    let processes: Vec<&str> = seen.lines().filter(|l| l.starts_with("process ")).collect();
    assert!(!processes.is_empty(), "the agent's processes are found: {seen}");
    for p in &processes {
        assert!(
            p.contains("user steplet")
                && p.contains("CapEff: 0000000000000000")
                && p.contains("CapBnd: 0000000000000000")
                && p.contains("NoNewPrivs: 1"),
            "{p}"
        );
    }
    assert!(seen.ends_with("root processes of the run: 0"), "{seen}");

    let stopped = Instant::now();
    handle.stop().unwrap();
    events.extend(collect(&handle, Duration::from_secs(30)));
    let code = handle.wait().unwrap();
    println!("--- stopped after {:.1} s; exit code {code:?}; every event:", stopped.elapsed().as_secs_f64());
    print_all(&events);
    let results: Vec<&Event> = events.iter().filter(|e| matches!(e, Event::Result { .. })).collect();
    assert_eq!(results.len(), 1, "exactly one result");
    assert!(matches!(results[0], Event::Result { subtype, is_error: true, .. } if subtype == "error_during_execution"));
    let left = wsl_count(distro, &["/usr/bin/pgrep", "-u", "steplet"]);
    println!("--- programs of the steplet account left: {left}");
    assert_eq!(left, 0);
    discard(&paths).unwrap();
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn the_time_limit_ends_a_run() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = r#"{"steps": [
        {"tool": "Bash", "input": {"command": "sleep 120; echo slept", "description": "Stay busy"}},
        {"text": "Not reached."}]}"#;
    let started = Instant::now();
    let (handle, paths, _host, _model) = start("t7c", 47912, script, 15, "Follow the script.");
    let events = collect(&handle, Duration::from_secs(60));
    let code = handle.wait().unwrap();
    println!("--- run t7c ended after {:.1} s (prepare included), exit code {code:?}", started.elapsed().as_secs_f64());
    print_all(&events);
    assert_eq!(code, Some(124), "timeout's own code: the limit was reached");
    assert_eq!(wsl_count(paths.distro_id(), &["/usr/bin/pgrep", "-u", "steplet"]), 0);
    discard(&paths).unwrap();
}

/// How many lines a program prints in Steplet's Linux (as root), whatever its exit code.
fn wsl_count(distro: &str, argv: &[&str]) -> usize {
    steplet_run::wsl::run(distro, "root", argv, b"").unwrap().stdout.lines().count()
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_stop_right_after_launch_never_lets_the_agent_act() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = r#"{"steps": [
        {"tool": "Bash", "input": {"command": "echo ran > /work/t7d/repo/acted.txt", "description": "Act"}},
        {"text": "Not reached."}]}"#;
    let (mut handle, paths, _host, _model) = start("t7d", 47913, script, 300, "Follow the script.");
    // At once: the run is still Steplet's root start script, before it becomes the agent.
    handle.stop().unwrap();
    let events = collect(&handle, Duration::from_secs(30));
    let code = handle.wait().unwrap();
    println!("--- stopped at once; exit code {code:?}; every event:");
    print_all(&events);
    assert!(!events.iter().any(|e| matches!(e, Event::ToolResult { .. })), "no tool ran");
    let acted = support::root_sh(paths.distro_id(), "test -e /work/t7d/repo/acted.txt && echo yes || echo no");
    let left = wsl_count(paths.distro_id(), &["/usr/bin/pgrep", "-u", "steplet"]);
    let start_scripts = wsl_count(paths.distro_id(), &["/usr/bin/pgrep", "-f", "/opt/steplet/bin/start-run t7d "]);
    println!("--- acted: {acted}; programs of the steplet account left: {left}; start scripts left: {start_scripts}");
    assert_eq!((acted.as_str(), left, start_scripts), ("no", 0, 0));
    discard(&paths).unwrap();
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn whatever_a_run_leaves_running_ends_with_it() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let script = r#"{"steps": [
        {"tool": "Bash", "input": {"command": "sleep 300", "run_in_background": true, "description": "Leave something running"}},
        {"text": "Done, with something left running."}]}"#;
    let (handle, paths, _host, _model) = start("t7e", 47914, script, 300, "Follow the script.");
    let mut events = Vec::new();
    while !events.iter().any(|e| matches!(e, Event::Result { .. })) {
        events.push(handle.events().recv_timeout(Duration::from_secs(60)).expect("the run ends"));
    }
    let before = wsl_count(paths.distro_id(), &["/usr/bin/pgrep", "-u", "steplet"]);
    let waited = Instant::now();
    let code = handle.wait().unwrap();
    let after = wsl_count(paths.distro_id(), &["/usr/bin/pgrep", "-u", "steplet"]);
    println!("--- events:");
    print_all(&events);
    println!(
        "--- steplet programs when the result came: {before}; after wait ({:.1} s, exit code {code:?}): {after}",
        waited.elapsed().as_secs_f64()
    );
    assert_eq!(after, 0, "nothing of the run is left");
    assert!(waited.elapsed() < Duration::from_secs(30), "wait does not hang on a leftover's open output");
    discard(&paths).unwrap();
}
