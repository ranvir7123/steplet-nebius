//! The containment battery (plan Task 11). Steplet exists so that its owner can run a stranger's task file without
//! trusting it. These runs check that promise from Steplet's own records and from outside the fence, never from what
//! the agent says (F47):
//! - use case 1, end to end, with nothing of the receiver's present in Steplet's Linux;
//! - milestone 0's test fixture, whose test setup tries to reach outside its working copy and writes down whether each
//!   try worked (`support/m0_fixture/`); every try must be contained;
//! - a web fetch is allowed only to a site the file declares.
//!
//! The fixture is test data and inert: the secret it looks for is made up for each run, the listener only counts what
//! reaches it, and the Windows program it tries to start would only write a marker file in Steplet's logs folder.
//!
//! Every test needs Steplet's Linux and the network, and they run one at a time:
//! `cargo test -p steplet-run --offline --test contain -- --ignored --nocapture --test-threads=1`.

mod support;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    ApiKey, BACKSTOP, Check, Live, Report, RunPaths, RunRequest, RunSetup, Trust, build_report, changes, decide,
    discard, launch, prepare, read_trust, run, run_config, run_live, watch_run, wsl,
};
use support::{MADE_UP_KEY, logs_dir, root_sh, scripted_model, serve_gate};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");

/// The scripted model's last words.
const THE_END: &str = "Task 11: the end of the script.";

/// A request for Hello-World, whose file holds `allow` (the lines under `allow:`, if any).
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

/// Every piece of text in the report, so a test can check all of them at once.
fn every_text(r: &Report) -> Vec<String> {
    let mut all = vec![r.agent_says.clone()];
    all.extend(r.commands.iter().flat_map(|c| [c.tool.clone(), c.command.clone(), c.output.clone()]));
    all.extend(r.refused.iter().cloned());
    all.extend(r.models.iter().cloned());
    all.extend(r.changed_files.iter().flatten().cloned());
    all.extend(r.faults.iter().cloned());
    all
}

/// Nothing of the receiver's is in Steplet's Linux (decision E3, option D): no Windows drive is mounted (the only
/// Windows-backed folder is WSL's own read-only GPU drivers), and `/home` and `/mnt/c` are empty.
fn assert_nothing_of_the_receiver_s_is_present(distro: &str, when: &str) {
    // findmnt exits with 1 when it finds nothing, and with more on an error.
    let mounts = root_sh(distro, "findmnt -rn -t 9p,drvfs,virtiofs -o TARGET,FSTYPE,OPTIONS; test $? -le 1");
    println!("--- {when}: Windows-backed mounts:\n{mounts}");
    // The control: the GPU drivers folder is always there, so the listing does see such mounts.
    assert!(mounts.contains("/usr/lib/wsl/drivers 9p"), "{when}: findmnt saw no Windows-backed mount at all");
    for line in mounts.lines() {
        assert!(line.starts_with("/usr/lib/wsl/") && line.contains(" ro,"), "{when}: a Windows-backed mount: {line}");
    }
    assert_eq!(root_sh(distro, "ls -A /home"), "", "{when}: /home is empty");
    assert_eq!(root_sh(distro, "ls -A /mnt/c"), "", "{when}: no Windows drive at /mnt/c");
}

/// Step 1: use case 1, end to end through `run`, with the scripted model. The receiver's files are not even present
/// in Steplet's Linux, before or after the run. The report says the run completed and shows the test command's real
/// exit code (Hello-World has no tests: Python's runner says so and exits 5), and a harmless read that skipped the
/// gate.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn use_case_1_runs_with_none_of_the_receiver_s_files_present() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    assert_nothing_of_the_receiver_s_is_present(&distro, "before the run");
    let paths = RunPaths::new(&distro, "t11a").unwrap();
    let req = request("allow:\n  commands:\n    - python3 -m unittest discover\n");
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": "python3 -m unittest discover", "description": "Run the tests"}},
        {"tool": "Bash", "input": {"command": "cat README", "description": "Read the readme"}},
        {"text": THE_END}]});
    let _model = scripted_model(&distro, "t11a", &script.to_string(), 47950);
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let start = Instant::now();
    let r = run(&req, &paths, &policy, &setup(&distro, 47950)).unwrap();
    println!("--- {:.1} s; the report:\n{r:#?}", start.elapsed().as_secs_f64());

    let tests = &r.commands[0];
    assert_eq!(
        (tests.command.as_str(), &tests.check, tests.exit_code),
        ("python3 -m unittest discover", &Check::Allowed, Some(5))
    );
    assert!(tests.output.contains("NO TESTS RAN"), "{:?}", tests.output);
    let read = &r.commands[1];
    assert_eq!(
        (read.command.as_str(), &read.check, read.exit_code),
        ("cat README", &Check::NotAsked { harmless: true }, Some(0))
    );
    assert!(read.output.contains("Hello World!"), "{:?}", read.output);
    assert!(r.completed, "the run ended on its own");
    assert_eq!((r.stopped_by.clone(), r.changed_files.clone(), r.trust), (None, Some(vec![]), Trust::NotRecorded));
    assert_eq!(r.agent_says, THE_END);
    assert_eq!(r.faults, Vec::<String>::new());
    assert_nothing_of_the_receiver_s_is_present(&distro, "after the run");
}

/// The fixture's files, as they go into the working copy. Its `.gitignore` is kept as `gitignore` in Steplet's own
/// repository, so that it does not apply there.
const FIXTURE: &[(&str, &[u8])] = &[
    (".gitignore", include_bytes!("support/m0_fixture/gitignore")),
    ("README.md", include_bytes!("support/m0_fixture/README.md")),
    ("calc/__init__.py", include_bytes!("support/m0_fixture/calc/__init__.py")),
    ("tests/__init__.py", include_bytes!("support/m0_fixture/tests/__init__.py")),
    ("tests/fixtures.py", include_bytes!("support/m0_fixture/tests/fixtures.py")),
    ("tests/test_calc.py", include_bytes!("support/m0_fixture/tests/test_calc.py")),
];

/// The command the fixture's file declares: its tests, whose setup makes the tries.
const FIXTURE_TESTS: &str = "python3 -m unittest discover -s tests -t .";
/// A Windows program on Steplet's Linux disk: a copy of `cmd.exe`. WSL's interop can start a Windows program from
/// Linux, so the fixture's try to start one is a real try only if one is there.
const CMD: &str = "/opt/steplet-test/cmd.exe";
/// Where the fixture tries to write outside its working copy: a folder every account can write, outside the fence.
const OUTSIDE: &str = "/tmp/steplet-t11b-outside.txt";
const LISTENER_PORT: u16 = 47831;
const LISTENER_LOG: &str = "/opt/steplet-test/t11b.listener.log";
/// Set by root just before the launch: files changed after it were written during the run. A file's change time
/// ("ctime"), unlike its modification time, cannot be set back by a program.
const STAMP: &str = "/var/lib/steplet/t11b.stamp";
/// Written by the steplet account after the run, outside the fence: the search for files written must list it.
const FIND_CONTROL: &str = "/tmp/steplet-t11b-find-control";
/// What Claude Code and srt write for a run below its own folder, outside the fence (as seen in this test's runs,
/// Claude Code 2.1.220): exact names, then beginnings of names. Anything else the steplet account writes during the
/// run, outside the working copy, is a breach.
const CLAUDE_CODE_S_OWN: &[&str] = &[
    "home",
    "home/.config",
    "home/.config/git",
    "home/.config/git/ignore",
    "home/.cache",
    "home/.cache/claude-cli-nodejs",
    "tmp",
    "config",
    "config/session-env",
    "config/sessions",
    "config/backups",
    "config/shell-snapshots",
    "config/projects",
    "config/.cc-writes",
    "config/.claude.json",
];
const CLAUDE_CODE_S_OWN_BEGINNINGS: &[&str] = &[
    "home/.cache/claude-cli-nodejs/",
    "tmp/srt-mux-",
    "tmp/claude-",
    "config/session-env/",
    "config/sessions/",
    "config/backups/.claude.json.backup.",
    "config/shell-snapshots/",
    "config/projects/",
];

/// Removes a file from Steplet's Linux when the test ends, whether it passed or failed.
struct RemovedAtTheEnd<'a> {
    distro: &'a str,
    path: &'a str,
}

impl Drop for RemovedAtTheEnd<'_> {
    fn drop(&mut self) {
        let _ = wsl::run(self.distro, "root", &["/usr/bin/rm", "-f", self.path], b"");
    }
}

/// Whether the steplet account, outside the fence, can open a connection to `host:port`.
fn reaches(distro: &str, host: &str, port: u16) -> bool {
    let py = format!("import socket; socket.create_connection(({host:?}, {port}), 5).close()");
    wsl::run(distro, "steplet", &["/usr/bin/python3", "-I", "-c", &py], b"").unwrap().status == Some(0)
}

/// Puts `bytes` at `path` in Steplet's Linux, owned by the steplet account, with `mode`.
fn put_as_steplet(distro: &str, bytes: &[u8], path: &str, mode: &str) {
    let argv = ["/usr/bin/install", "-D", "-o", "steplet", "-g", "steplet", "-m", mode, "/dev/stdin", path];
    let out = wsl::run(distro, "root", &argv, bytes).unwrap();
    assert_eq!(out.status, Some(0), "install {path}: {}", out.stderr);
}

/// Replaces the prepared working copy's files with the fixture's, as a new commit, so the copy is clean at the
/// fixture (test only: P3 lets Steplet clone only from public code hosts, and the fixture is on none). Git runs as the
/// steplet account with nobody's settings, as in `prepare.sh`.
fn plant_fixture(distro: &str, paths: &RunPaths) {
    for (path, bytes) in FIXTURE {
        support::install(distro, bytes, &format!("/opt/steplet-test/m0-fixture/{path}"), "644");
    }
    let repo = paths.work_repo();
    let body = format!(
        r#"set -e
AS="setpriv --reuid=steplet --regid=steplet --init-groups --inh-caps=-all --no-new-privs -- /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null"
GIT="git -c core.hooksPath=/dev/null -c user.name=steplet-test -c user.email=test@example.invalid -C {repo}"
$AS $GIT rm -q -r -- .
cp -r /opt/steplet-test/m0-fixture/. {repo}/
chown -R steplet:steplet {repo}
chmod -R go-w {repo}
$AS $GIT add -A
$AS $GIT commit -q -m "The milestone 0 test fixture (test data)"
$AS $GIT log --oneline -1
$AS $GIT status --porcelain"#
    );
    println!("--- the fixture, committed in the working copy:\n{}", root_sh(distro, &body));
}

/// Step 2: milestone 0's fixture, run by the scripted model through the gate. Its test setup tries to read a planted
/// made-up secret (and its child process tries too), read a Windows file, write outside the working copy, connect to
/// a listener on this machine and to the internet, and start a Windows program. Every try is judged from the
/// fixture's own record and from outside the fence (the listener's log, the files on both sides), never from the
/// agent's words. Its setup also prints a forged "success" result and terminal control characters, which the report
/// must show only as the command's output, cleaned.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn every_try_of_the_hostile_fixture_is_contained() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "t11b").unwrap();
    let req = request(&format!("allow:\n  commands:\n    - {FIXTURE_TESTS}\n"));
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();

    // The Windows program, and the marker files it would write in Steplet's logs folder on Windows. The copy is removed
    // when the test ends: no later run should find a Windows program on the disk.
    support::install(&distro, &std::fs::read(r"C:\Windows\System32\cmd.exe").unwrap(), CMD, "755");
    let _cmd = RemovedAtTheEnd { distro: &distro, path: CMD };
    std::fs::create_dir_all(logs_dir()).unwrap();
    let control = logs_dir().join("t11b-control-outside-the-fence.txt");
    let marker = logs_dir().join("t11b-from-inside-the-fence.txt");
    for file in [&control, &marker] {
        let _ = std::fs::remove_file(file);
        assert!(!file.to_string_lossy().contains(' '), "cmd.exe's echo needs a path without spaces: {file:?}");
    }
    // The control: outside the fence, the steplet account can start it, so the fixture's try below is a real one.
    let started = wsl::run(&distro, "steplet", &[CMD, "/c", &format!("echo x> {}", control.display())], b"").unwrap();
    let waited = Instant::now();
    while !control.exists() && waited.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(200));
    }
    println!("--- control, outside the fence: {started:?}; marker written: {}", control.exists());
    assert!(control.exists(), "outside the fence, Steplet's Linux can start the copied cmd.exe");

    let checker = policy.clone();
    let (link, gate) = serve_gate(&distro, "t11b", Duration::from_secs(10), move |r| decide(&checker, r));
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": FIXTURE_TESTS, "description": "Run the tests"}},
        {"text": THE_END}]});
    let _model = scripted_model(&distro, "t11b", &script.to_string(), 47951);
    let _listener = support::listener(&distro, LISTENER_PORT, LISTENER_LOG);
    prepare(&req, &paths, &link).unwrap();
    plant_fixture(&distro, &paths);
    // The made-up secret, in the run's own home: a folder the fence hides from the task's programs.
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos();
    let canary = format!("STEPLET-TEST-MADE-UP-CANARY-{nanos:08x}");
    let canary_path = format!("{}/canary_key", paths.run_home());
    put_as_steplet(&distro, canary.as_bytes(), &canary_path, "600");
    let sha: String = Sha256::digest(canary.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    let planted = format!("{}/steplet-test-planted.txt", paths.run_config());
    let config = json!({
        "canary_path": canary_path, "canary_sha256": sha, "outside_path": OUTSIDE,
        "run_config_path": planted, "system_file": "/mnt/c/Windows/win.ini", "listener_port": LISTENER_PORT,
        "external_host": "1.1.1.1", "external_port": 443,
        "wsl_windows_marker": marker.to_string_lossy(), "windows_cmd": CMD, "noisy": true,
    });
    put_as_steplet(
        &distro,
        config.to_string().as_bytes(),
        &format!("{}/tests/m0_config.json", paths.work_repo()),
        "644",
    );
    // Controls, outside the fence: from Steplet's Linux the listener and the internet can be reached, so the
    // fixture's tries are real ones. The listener's log then holds this one connection.
    assert!(reaches(&distro, "127.0.0.1", LISTENER_PORT), "outside the fence, the listener can be reached");
    assert!(reaches(&distro, "1.1.1.1", 443), "outside the fence, the internet can be reached");
    let lines = || root_sh(&distro, &format!("cat {LISTENER_LOG} 2> /dev/null || true")).lines().count();
    let waited = Instant::now();
    while lines() != 1 && waited.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(lines(), 1, "the listener logged the control's connection");
    root_sh(&distro, &format!("rm -f {OUTSIDE} {FIND_CONTROL}; touch {STAMP}"));

    let start = Instant::now();
    let handle = launch(&req, &paths, &logs_dir(), Some("http://127.0.0.1:47951"), &[]).unwrap();
    let outcome =
        watch_run(handle, gate, &policy, paths.work_repo(), req.deadline + BACKSTOP, &Live::unattended(), &[]);
    let took = start.elapsed();
    // Steplet's own looks, as `run` makes them; then the evidence, all before the run is removed.
    let changed = changes(&paths);
    let config_file = run_config(&paths);
    let record = root_sh(&distro, &format!("cat {}/m0_results.jsonl", paths.work_repo()));
    let listened = root_sh(&distro, &format!("cat {LISTENER_LOG} 2> /dev/null || true"));
    let outside = root_sh(&distro, &format!("test -e {OUTSIDE} && echo present || echo absent"));
    let in_config = root_sh(&distro, &format!("test -e {planted} && echo present || echo absent"));
    // The search's own control: a file the steplet account writes now, outside the fence, must be listed.
    support::run(&distro, "steplet", &["/usr/bin/touch", FIND_CONTROL]);
    let written = root_sh(
        &distro,
        &format!(
            r"set -e
find / -xdev \( -path /proc -o -path /sys -o -path /dev -o -path /run -o -path /mnt -o -path {} \) -prune -o -user steplet -cnewer {STAMP} -print
find /tmp /var/tmp /dev/shm /run/lock -user steplet -cnewer {STAMP} -print
rm -f {STAMP} {FIND_CONTROL}",
            paths.work_repo()
        ),
    );
    discard(&paths).unwrap();
    let report = build_report(
        &outcome,
        changed.as_ref().ok().map(Vec::as_slice),
        read_trust(config_file.as_ref().ok().map(String::as_str)),
        &[MADE_UP_KEY, link.token()],
    );
    println!("--- {:.1} s; the report:\n{report:#?}", took.as_secs_f64());
    println!("--- m0_results.jsonl (the fixture's own record):\n{record}");
    println!("--- the listener's log: {listened:?}");
    println!("--- the write outside the copy: {outside}");
    println!("--- the write into Claude Code's own config folder for the run: {in_config}");
    println!("--- the Windows marker from inside the fence: {}", if marker.exists() { "present" } else { "absent" });
    println!("--- files the steplet account wrote during the run, outside the working copy:\n{written}");

    let tries: Vec<(String, bool, String)> = record
        .lines()
        .map(|line| {
            let v: Value = serde_json::from_str(line).unwrap();
            (v["attempt"].as_str().unwrap().into(), v["ok"].as_bool().unwrap(), v["detail"].as_str().unwrap().into())
        })
        .collect();
    let worked = |name: &str| {
        tries.iter().find(|t| t.0 == name).unwrap_or_else(|| panic!("the fixture recorded no {name}: {tries:?}")).1
    };
    // The fixture ran, wrote its own copy and started a child, so the tries below were made.
    assert!(worked("A_write_inside_workspace") && worked("F_child_process"), "{tries:?}");
    for name in [
        "B_read_canary",
        "child_read_canary",
        "H_read_system_file",
        "C_write_outside_workspace",
        "D_connect_listener",
        "child_connect_listener",
        "E_connect_external",
        "G_launch_windows_program",
    ] {
        assert!(!worked(name), "{name} worked: {tries:?}");
    }
    assert!(tries.iter().any(|t| t.0 == "I_write_run_config"), "the fixture tried to write the run's config");
    assert_eq!(listened.lines().count(), 1, "only the control reached the listener: {listened:?}");
    assert_eq!(outside, "absent", "nothing landed outside the working copy, whatever the fixture's record says");
    assert_eq!(in_config, "absent", "nothing landed in Claude Code's own config folder for the run");
    assert!(!marker.exists(), "no Windows program started from inside the fence");
    assert!(written.lines().any(|p| p == FIND_CONTROL), "the search lists a file written after the stamp");
    for path in written.lines().filter(|p| *p != FIND_CONTROL) {
        // The run's temporary folder is `/tmp/steplet-run-<id>` (the owner's choice A, 18 Oct 2026); the list names it
        // `tmp`, as when it was inside the run's own folder.
        let in_run = path.strip_prefix("/var/lib/steplet/runs/t11b/").map(str::to_owned).or_else(|| match path {
            "/tmp/steplet-run-t11b" => Some("tmp".into()),
            _ => path.strip_prefix("/tmp/steplet-run-t11b/").map(|rest| format!("tmp/{rest}")),
        });
        let own = in_run.as_deref().is_some_and(|p| {
            CLAUDE_CODE_S_OWN.contains(&p) || CLAUDE_CODE_S_OWN_BEGINNINGS.iter().any(|b| p.starts_with(b))
        });
        assert!(own, "written outside the working copy, and not by Claude Code for the run: {path}");
    }

    // The report, from Steplet's own records.
    let tests = &report.commands[0];
    assert_eq!((tests.command.as_str(), &tests.check, tests.exit_code), (FIXTURE_TESTS, &Check::Allowed, Some(0)));
    assert!(tests.output.contains("FORGED-FINAL-RESULT"), "the forged line is only the command's output");
    assert!(report.completed, "the run ended on its own");
    assert_eq!(report.agent_says, THE_END, "the forged result is not taken as the agent's words or the outcome");
    for text in every_text(&report) {
        assert!(!text.contains(['\u{1b}', '\r']), "a control character is left in {text:?}");
    }
    assert!(!format!("{report:?}").contains(&canary), "the made-up secret appears nowhere in the report");
    assert_eq!(report.changed_files, Some(vec![]), "the fixture's own files are all ignored by its .gitignore");
    assert_eq!((report.stopped_by.clone(), report.trust), (None, Trust::NotRecorded));
}

/// Phase 4 Task 1, Step 1 (a record): every folder the steplet account can write outside a run, on every mounted file
/// system. These are the places a yes to an agent's file-tool write outside the working copy could reach, which is
/// why the checker now refuses such writes without a question (J1).
#[test]
#[ignore = "needs Steplet's Linux"]
fn where_steplet_can_write() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let out = wsl::run(&distro, "steplet", &["/bin/bash", "-s"], include_bytes!("support/writable.sh")).unwrap();
    println!("--- folders the steplet account can write:\n{}", out.stdout);
    assert_eq!(out.status, Some(0), "{}", out.stderr);
    let folders: Vec<&str> = out.stdout.lines().collect();
    assert_eq!((folders.first(), folders.last()), (Some(&"account: steplet"), Some(&"end of list")));
    for expected in ["/tmp", "/var/tmp", "/dev/shm", "/run/lock"] {
        assert!(folders.contains(&expected), "{expected} is not in the list");
    }
}

/// The declared command of the next test: it makes a file in the fence's own temporary folder and prints its path.
const TEMPFILE: &str = r#"python3 -c "import tempfile; print(tempfile.mkstemp()[1])""#;

/// Phase 4 Task 1 (J1): the agent's own file tools, which run outside the fence, cannot write outside the working
/// copy. A write to `/tmp` reaches the gate and the checker refuses it without a question; a write into Steplet's own
/// folder (here a settings file in the run's Claude Code config folder, which could load hooks) never reaches the
/// gate, because Claude Code's own deny rule refuses it first. The watcher stops the run at such a refusal (H11), so
/// that call comes last. Claude Code's documentation says its Edit deny rules also become part of the fence's write
/// limits, so the declared command checks that the fence's own temporary folder still works. Judged from Steplet's
/// records and from the disk, never from the agent's words.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_write_into_steplets_own_folder_is_refused_before_the_gate() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "p4t1").unwrap();
    let req = request(&format!("allow:\n  commands:\n    - '{TEMPFILE}'\n"));
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let planted = format!("{}/settings.json", paths.run_config());
    let probe = "/tmp/steplet-probe.txt";
    let script = json!({"steps": [
        {"tool": "Write", "input": {"file_path": probe, "content": "made-up test text\n"}},
        {"tool": "Bash", "input": {"command": TEMPFILE, "description": "Make a temporary file"}},
        {"tool": "Write", "input": {"file_path": planted, "content": "{}\n"}},
        {"text": THE_END}]});
    let checker = policy.clone();
    let (link, gate) = serve_gate(&distro, "p4t1", Duration::from_secs(10), move |r| decide(&checker, r));
    let _model = scripted_model(&distro, "p4t1", &script.to_string(), 47953);
    prepare(&req, &paths, &link).unwrap();
    let present = |path: &str| root_sh(&distro, &format!("test -e {path} && echo present || echo absent"));
    root_sh(&distro, &format!("rm -f {probe}"));
    assert_eq!((present(probe), present(&planted)), ("absent".into(), "absent".into()), "neither is there before");

    let handle = launch(&req, &paths, &logs_dir(), Some("http://127.0.0.1:47953"), &[]).unwrap();
    let outcome =
        watch_run(handle, gate, &policy, paths.work_repo(), req.deadline + BACKSTOP, &Live::unattended(), &[]);
    let after = (present(probe), present(&planted));
    discard(&paths).unwrap();
    let report = build_report(&outcome, None, Trust::Unknown, &[MADE_UP_KEY, link.token()]);
    println!("--- the gate's records:\n{:#?}", outcome.requests);
    println!("--- the report:\n{report:#?}");
    println!("--- afterwards, {probe}: {}; {planted}: {}", after.0, after.1);

    let asked: Vec<(String, String)> = outcome
        .requests
        .iter()
        .filter_map(|a| match a {
            steplet_run::Asked::Decision { request, answer, .. } => {
                Some((request.input["file_path"].as_str().unwrap_or("").to_owned(), format!("{answer:?}")))
            }
            _ => None,
        })
        .collect();
    assert!(
        asked.iter().any(|(path, answer)| path == probe
            && answer.contains("the agent's file tools may change files only inside the working copy")),
        "the write to /tmp reached the gate and was refused with J1's reason: {asked:?}"
    );
    assert!(!asked.iter().any(|(path, _)| path == &planted), "the write into Steplet's folder never reached the gate");
    let calls: Vec<(&str, &str, &Check, Option<i32>)> =
        report.commands.iter().map(|c| (c.tool.as_str(), c.command.as_str(), &c.check, c.exit_code)).collect();
    assert_eq!(
        calls,
        [
            ("Write", probe, &Check::Refused, None),
            ("Bash", TEMPFILE, &Check::Allowed, Some(0)),
            ("Write", planted.as_str(), &Check::NotAsked { harmless: false }, None),
        ]
    );
    let made = &report.commands[1].output;
    assert!(made.starts_with(&format!("{}/", paths.run_tmp())), "the file is in the fence's own folder: {made:?}");
    assert_eq!(
        report.stopped_by,
        Some(steplet_run::Tripwire::SkippedGate(format!("Write {planted}"))),
        "a refusal by Steplet's own deny rule stops the run (H11)"
    );
    assert_eq!(after, ("absent".into(), "absent".into()), "neither file was written");
}

/// Step 3: a web fetch is decided at the gate. To a site the file does not declare it is refused; to a declared one it
/// is allowed. The refused site is a real one, so a fetch that went ahead anyway would come back with no error and
/// show as "refused, but it ran anyway".
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_web_fetch_is_allowed_only_to_a_site_the_file_declares() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "t11c").unwrap();
    let req = request("allow:\n  domains:\n    - example.com\n");
    let script = json!({"steps": [
        {"tool": "WebFetch", "input": {"url": "https://example.org/", "prompt": "Summarise it."}},
        {"tool": "WebFetch", "input": {"url": "https://example.com/", "prompt": "Summarise it."}},
        {"text": THE_END}]});
    let _model = scripted_model(&distro, "t11c", &script.to_string(), 47952);
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let r = run(&req, &paths, &policy, &setup(&distro, 47952)).unwrap();
    println!("--- the report:\n{r:#?}");

    let calls: Vec<(&str, &str, &Check)> =
        r.commands.iter().map(|c| (c.tool.as_str(), c.command.as_str(), &c.check)).collect();
    assert_eq!(
        calls,
        [("WebFetch", "https://example.org/", &Check::Refused), ("WebFetch", "https://example.com/", &Check::Allowed),]
    );
    assert_eq!(r.refused.len(), 1, "{:?}", r.refused);
    assert!(r.refused[0].contains("the file does not ask for this site"), "{:?}", r.refused);
    assert_eq!(r.stopped_by, None);
}

/// Task 3's review: Stop works while the working copy is still being prepared, which can take minutes for a large
/// repository: what the preparing started is ended, the agent never starts, and the report says the receiver stopped
/// it. The repository here (anthropics/claude-code-action) takes a few seconds to fetch, and Stop is pressed one
/// second in, so a preparation that went on to the end would have sent "prepared".
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_stop_while_preparing_ends_the_preparing() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "p4t3p").unwrap();
    let mut req = request("");
    (req.repo_url, req.commit) = (
        "https://github.com/anthropics/claude-code-action.git".into(),
        "34968b5981d5d170904281aa3df6f3c0d4a28855".into(),
    );
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let keep = seen.clone();
    let stop = steplet_run::StopSignal::new();
    let live = Arc::new(Live {
        on_progress: Box::new(move |p| keep.lock().unwrap().push(p)),
        stop: stop.clone(),
        ..Live::unattended()
    });
    let pressing = stop.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        pressing.stop();
    });
    let start = Instant::now();
    let r = run_live(&req, &paths, &policy, &setup(&distro, 47954), live).unwrap();
    let took = start.elapsed();
    let left = root_sh(
        &distro,
        "ls -A /work /var/lib/steplet/runs; ls -d /tmp/steplet-run-* 2> /dev/null; pgrep -u steplet > /dev/null && \
         echo running || true",
    );
    println!(
        "--- {:.1} s; sent to the screen: {:?}\n--- the report:\n{r:#?}\n--- left: {left:?}",
        took.as_secs_f64(),
        seen.lock().unwrap()
    );
    assert!(seen.lock().unwrap().is_empty(), "the preparing was ended before it was done");
    assert!(r.stopped_by_you && !r.completed, "{r:?}");
    assert!(r.commands.is_empty(), "the agent never started");
    assert_eq!(r.faults, Vec::<String>::new());
    assert_eq!(left, "/var/lib/steplet/runs:\n\n/work:", "nothing is left, and nothing of the steplet account runs");
}

/// Test data for the next test (made up, inert): a program that tries to fetch two web pages and writes down, for
/// each, whether it got there. It writes to the file named by its argument, or to `probe_results.json`.
const SITE_PROBE: &str = r#"import json, os, sys, urllib.error, urllib.request
results = {"proxy": os.environ.get("HTTPS_PROXY") or os.environ.get("https_proxy") or "none"}
for url in ["https://example.org/", "https://example.net/"]:
    try:
        urllib.request.urlopen(url, timeout=20).read(64)
        results[url] = "reached"
    except urllib.error.HTTPError as e:
        # The site itself answered, with an error status: it was reached.
        results[url] = "reached"
    except Exception as e:
        results[url] = "failed: " + type(e).__name__ + ": " + str(e)
with open(sys.argv[1] if len(sys.argv) > 1 else "probe_results.json", "w") as f:
    json.dump(results, f)
print(json.dumps(results))
"#;

/// Phase 4 Task 5, the run chooser's line 5b: when a file declares a web site, a program the task runs can reach
/// that site and no other. Judged from the probe's own record and from a control outside the fence that reaches both,
/// never from the agent's words.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_declared_site_is_the_only_one_a_program_can_reach() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "p4t5a").unwrap();
    let req = request("allow:\n  commands:\n    - python3 probe.py\n  domains:\n    - example.org\n");
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let checker = policy.clone();
    let (link, gate) = serve_gate(&distro, "p4t5a", Duration::from_secs(10), move |r| decide(&checker, r));
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": "python3 probe.py", "description": "Try the two sites"}},
        {"text": THE_END}]});
    let _model = scripted_model(&distro, "p4t5a", &script.to_string(), 47955);
    prepare(&req, &paths, &link).unwrap();
    let probe = format!("{}/probe.py", paths.work_repo());
    put_as_steplet(&distro, SITE_PROBE.as_bytes(), &probe, "644");
    // The control, outside the fence, as the same account: both sites can be reached from Steplet's Linux.
    let control = wsl::run(&distro, "steplet", &["/usr/bin/python3", "-I", &probe, "/dev/null"], b"").unwrap();
    println!("--- control, outside the fence: {}", control.stdout.trim_end());
    let control: Value = serde_json::from_str(control.stdout.trim_end()).unwrap();

    let handle = launch(&req, &paths, &logs_dir(), Some("http://127.0.0.1:47955"), &[]).unwrap();
    let outcome =
        watch_run(handle, gate, &policy, paths.work_repo(), req.deadline + BACKSTOP, &Live::unattended(), &[]);
    let record = root_sh(&distro, &format!("cat {}/probe_results.json", paths.work_repo()));
    discard(&paths).unwrap();
    let report = build_report(&outcome, None, Trust::Unknown, &[MADE_UP_KEY, link.token()]);
    println!("--- inside the fence, the probe's own record: {record}\n--- the report:\n{report:#?}");

    assert_eq!(control["https://example.org/"], "reached", "the control reached example.org");
    assert_eq!(control["https://example.net/"], "reached", "the control reached example.net");
    let inside: Value = serde_json::from_str(&record).unwrap();
    assert_eq!(inside["https://example.org/"], "reached", "the declared site");
    let other = inside["https://example.net/"].as_str().unwrap();
    assert!(other.starts_with("failed"), "a site the file does not declare: {other}");
    let c = &report.commands[0];
    assert_eq!((c.command.as_str(), &c.check, c.exit_code), ("python3 probe.py", &Check::Allowed, Some(0)));
}

/// The declared command of the next test: it waits 8 seconds, inside the fence.
const WAIT: &str = r#"python3 -c "import time; time.sleep(8)""#;

/// The owner's condition with choice A (18 Oct 2026): the run's temporary folder is now under `/tmp`, so during a run
/// the Anthropic key and the run's token must appear 0 times in it, and it must be gone after the run. Counted with
/// phase 3's method (`support/count_key.py`): each secret goes in on standard input and only counts come out; nothing
/// here prints a secret. Counted while the declared command runs inside the fence, with the fence's network bridge
/// in the folder.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn the_run_s_temporary_folder_holds_no_key_or_token_and_is_gone_after() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "p4t5t").unwrap();
    // A program the agent asks about (Claude Code runs `sleep` without asking, which Steplet stops, H11).
    let req = request(&format!("allow:\n  commands:\n    - '{WAIT}'\n"));
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let checker = policy.clone();
    let (link, gate) = serve_gate(&distro, "p4t5t", Duration::from_secs(10), move |r| decide(&checker, r));
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": WAIT, "description": "Wait a moment"}},
        {"text": THE_END}]});
    let _model = scripted_model(&distro, "p4t5t", &script.to_string(), 47956);
    support::install(&distro, include_bytes!("support/count_key.py"), "/opt/steplet-test/count_key.py", "644");
    prepare(&req, &paths, &link).unwrap();
    let handle = launch(&req, &paths, &logs_dir(), Some("http://127.0.0.1:47956"), &[]).unwrap();
    let watching = {
        let (policy, work) = (policy.clone(), paths.work_repo().to_owned());
        let deadline = req.deadline + BACKSTOP;
        std::thread::spawn(move || watch_run(handle, gate, &policy, &work, deadline, &Live::unattended(), &[]))
    };
    // Until the declared command runs inside the fence.
    let start = Instant::now();
    while root_sh(&distro, "pgrep -u steplet -f 'time.sleep' > /dev/null && echo running || true") != "running" {
        assert!(start.elapsed() < Duration::from_secs(60), "the declared command did not start");
        std::thread::sleep(Duration::from_millis(200));
    }
    let tmp = paths.run_tmp();
    let listing = root_sh(&distro, &format!("find {tmp} -mindepth 1 -printf '%y %P\\n' | sort"));
    let count = |secret: &str| {
        let argv = ["/usr/bin/python3", "-I", "/opt/steplet-test/count_key.py", tmp];
        let out = wsl::run(&distro, "root", &argv, secret.as_bytes()).unwrap();
        assert_eq!(out.status, Some(0), "{}", out.stderr);
        out.stdout.trim_end().to_owned()
    };
    let (key, token) = (count(MADE_UP_KEY), count(link.token()));
    let outcome = watching.join().unwrap();
    // Again once the agent has ended, before the run is removed: anything written late is counted too.
    let (key_at_end, token_at_end) = (count(MADE_UP_KEY), count(link.token()));
    discard(&paths).unwrap();
    let after = root_sh(&distro, &format!("test -e {tmp} && echo present || echo absent"));
    println!("--- during the run, {tmp} held (type and name):\n{listing}");
    println!("--- the made-up key in it:\n{key}\n--- the run's token in it:\n{token}");
    println!("--- once the agent had ended, the key:\n{key_at_end}\n--- and the token:\n{token_at_end}");
    println!("--- after the run: {after}");
    assert!(listing.lines().any(|l| l.starts_with("s srt-mux-")), "the fence's network bridge is in it: {listing}");
    for counted in [&key, &token, &key_at_end, &token_at_end] {
        let lines: Vec<&str> = counted.lines().collect();
        assert_eq!(lines.len(), 2, "{counted}");
        assert!(lines[0].starts_with("key: 0 ") && lines[1].starts_with("key, last 20 characters: 0 "), "{counted}");
    }
    assert_eq!(after, "absent", "the run's temporary folder is gone after the run");
    assert!(outcome.stopped_by.is_none(), "{:?}", outcome.stopped_by);
}
