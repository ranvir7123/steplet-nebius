//! The working copy, prepared inside Steplet's own Linux (plan Task 6). The tests marked `#[ignore]` need that Linux
//! (set up by `src/setup.sh`, phase 4 Task 6) and the network, and run only when asked:
//! `cargo test -p steplet-run --offline --test prepare -- --ignored`. They share `/work`, so they run one at a time.

use std::sync::Mutex;
use std::time::Duration;

use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_run::{
    GateLink, PrepareError, Prepared, RunPaths, RunRequest, changes, discard, mcp_json, prepare, settings_json, wsl,
};

/// Tiny and unchanged since 2011; phase 0's S4 used the same commit.
const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");
/// Its commit tracks `.claude/settings.json`, `.claude/commands/` and `.claude/agents/`.
const ACTION: (&str, &str) =
    ("https://github.com/anthropics/claude-code-action.git", "34968b5981d5d170904281aa3df6f3c0d4a28855");

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn request(url: &str, commit: &str) -> RunRequest {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n---\n";
    RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: url.into(),
        commit: commit.into(),
        task_text: "Run the tests.".into(),
        max_turns: 4,
        deadline: Duration::from_secs(60),
        allow_model_switch: false,
    }
}

fn gate(run_id: &str) -> GateLink {
    let log = format!(r"C:\Users\Test User\AppData\Local\Steplet\logs\{run_id}.gate.log");
    let deadline = Duration::from_secs(30);
    GateLink::new("/opt/steplet/gate/steplet-gate.exe", &format!("steplet-{run_id}"), "0123abcd", &log, deadline)
        .unwrap()
}

fn steplet_linux() -> String {
    wsl::find_distribution("steplet").unwrap().expect("Steplet's Linux is installed")
}

/// Runs one program in Steplet's Linux and returns its output, trimmed.
fn run(distro: &str, user: &str, argv: &[&str]) -> String {
    let out = wsl::run(distro, user, argv, b"").unwrap();
    assert_eq!(out.status, Some(0), "{argv:?}: {}", out.stderr);
    out.stdout.trim_end().to_owned()
}

fn owner_and_mode(distro: &str, path: &str) -> String {
    run(distro, "root", &["/usr/bin/stat", "-c", "%U %G %a", path])
}

#[test]
fn a_request_steplet_could_misread_is_refused_before_linux_is_touched() {
    // A made-up distribution: if a value got through, `wsl.exe` would be started and fail with another error.
    let paths = RunPaths::new("{00000000-0000-0000-0000-000000000000}", "t6x").unwrap();
    for (url, commit) in [
        ("http://github.com/octocat/Hello-World.git", HELLO.1),
        ("HTTPS://github.com/octocat/Hello-World.git", HELLO.1),
        ("https://github.com:443/octocat/Hello-World.git", HELLO.1),
        ("https://user@github.com/octocat/Hello-World.git", HELLO.1),
        ("https://github.com/octocat/Hello-World.git?x=1", HELLO.1),
        ("https://github.com/octocat/Hello World.git", HELLO.1),
        ("https://127.0.0.1/octocat/Hello-World.git", HELLO.1),
        ("file:///work/other/repo", HELLO.1),
        // P3, as the user decided on 14 Oct 2026: milestone 1 clones from four public code hosts only.
        ("https://printer.local/octocat/Hello-World.git", HELLO.1),
        ("https://git.example.invalid/octocat/Hello-World.git", HELLO.1),
        ("https://github.com.example.invalid/octocat/Hello-World.git", HELLO.1),
        ("https://gist.github.com/octocat/Hello-World.git", HELLO.1),
        (HELLO.0, "7fd1a60"),
        (HELLO.0, "7FD1A60B01F91B314F59955A4E4D4E80D8EDF11D"),
        (HELLO.0, "main"),
        (HELLO.0, "--orphan"),
    ] {
        let got = prepare(&request(url, commit), &paths, &gate("t6x"));
        assert!(matches!(got, Err(PrepareError::Refused(_))), "{url} {commit}: {got:?}");
    }
}

#[test]
fn a_plain_request_gets_past_the_checks() {
    // The same made-up distribution: a request that passes the checks fails later, at `wsl.exe`, not as refused.
    let paths = RunPaths::new("{00000000-0000-0000-0000-000000000000}", "t6x").unwrap();
    let sha256 = "a".repeat(64);
    for (url, commit) in [
        HELLO,
        ACTION,
        ("https://gitlab.com/group/sub_group/project.name~1.git", HELLO.1),
        ("https://codeberg.org/someone/project.git", HELLO.1),
        ("https://bitbucket.org/someone/project.git", HELLO.1),
        ("https://github.com/octocat/Hello-World", sha256.as_str()),
    ] {
        let got = prepare(&request(url, commit), &paths, &gate("t6x"));
        assert!(!matches!(got, Err(PrepareError::Refused(_)) | Ok(_)), "{url} {commit}: {got:?}");
    }
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn prepare_makes_one_fresh_working_copy_at_the_declared_commit() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    // Left over from an earlier run: prepare must remove it.
    run(&distro, "root", &["/usr/bin/install", "-d", "/work/t6-old/repo", "/var/lib/steplet/runs/t6-old"]);
    let paths = RunPaths::new(&distro, "t6a").unwrap();
    let (req, g) = (request(HELLO.0, HELLO.1), gate("t6a"));
    prepare(&req, &paths, &g).unwrap();
    let listing = ["/usr/bin/ls", "-la", "/work", "/work/t6a", "/work/t6a/repo", "/var/lib/steplet/runs/t6a"];
    println!("{}", run(&distro, "root", &listing));

    let repo = paths.work_repo();
    assert_eq!(run(&distro, "steplet", &["/usr/bin/git", "-C", repo, "rev-parse", "HEAD"]), HELLO.1);
    assert_eq!(run(&distro, "steplet", &["/usr/bin/git", "-C", repo, "status", "--porcelain"]), "", "a clean copy");
    assert_eq!(run(&distro, "root", &["/usr/bin/ls", "-A", repo]), ".git\nREADME");
    assert_eq!(run(&distro, "root", &["/usr/bin/ls", "-A", "/work"]), "t6a", "/work holds this run only");
    assert_eq!(
        run(&distro, "root", &["/usr/bin/ls", "-A", "/var/lib/steplet/runs/t6a"]),
        "config\nhome\nmcp.json\nsettings.json"
    );
    assert_eq!(run(&distro, "root", &["/usr/bin/ls", "-A", "/var/lib/steplet/runs"]), "t6a");
    assert_eq!(run(&distro, "root", &["/usr/bin/ls", "-A", repo.strip_suffix("/repo").unwrap()]), "repo");
    for (path, want) in [
        ("/work/t6a", "root root 755"),
        (repo, "steplet steplet 755"),
        ("/var/lib/steplet/runs/t6a", "root root 755"),
        (paths.run_home(), "steplet steplet 700"),
        (paths.run_config(), "steplet steplet 700"),
        (paths.run_tmp(), "steplet steplet 700"),
        (paths.settings(), "root root 644"),
        (paths.mcp(), "root root 644"),
    ] {
        assert_eq!(owner_and_mode(&distro, path), want, "{path}");
    }
    let file = |path: &str| wsl::run(&distro, "root", &["/usr/bin/cat", path], b"").unwrap().stdout;
    assert_eq!(
        file(paths.settings()),
        settings_json(&req.steplet, &paths, &g, req.allow_model_switch),
        "the settings file, byte for byte"
    );
    assert_eq!(file(paths.mcp()), mcp_json(&g), "the MCP config, byte for byte");
    for home in [paths.run_home(), paths.run_config(), paths.run_tmp()] {
        assert_eq!(run(&distro, "root", &["/usr/bin/ls", "-A", home]), "", "{home} starts empty");
    }

    discard(&paths).unwrap();
    assert_eq!(
        run(&distro, "root", &["/usr/bin/ls", "-A", "/work", "/var/lib/steplet/runs"]),
        "/var/lib/steplet/runs:\n\n/work:"
    );
    let tmp = format!("test -e {} && echo present || echo absent", paths.run_tmp());
    assert_eq!(run(&distro, "root", &["/bin/sh", "-c", &tmp]), "absent", "the run's temporary folder is gone too");
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn agent_settings_files_the_commit_carries_are_checked_out_then_removed() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    let paths = RunPaths::new(&distro, "t6b").unwrap();
    let prepared = prepare(&request(ACTION.0, ACTION.1), &paths, &gate("t6b")).unwrap();
    assert_eq!(prepared.removed, [".claude"], "the prepare screen lists what was removed");

    let repo = paths.work_repo();
    let git = |args: &[&str]| run(&distro, "steplet", &[&["/usr/bin/git", "-C", repo][..], args].concat());
    assert_eq!(git(&["rev-parse", "HEAD"]), ACTION.1);
    let tracked = git(&["ls-tree", "-r", "--name-only", "HEAD", ".claude"]);
    assert!(tracked.contains(".claude/settings.json") && tracked.contains(".claude/commands/"), "{tracked}");
    let status = git(&["status", "--porcelain"]);
    assert!(
        !status.is_empty() && status.lines().all(|l| l.starts_with(" D .claude/")),
        "only .claude is gone: {status}"
    );
    assert_eq!(run(&distro, "root", &["/bin/sh", "-c", &format!("test -e {repo}/.claude || echo absent")]), "absent");
    assert_eq!(
        owner_and_mode(&distro, &format!("{repo}/action.yml")),
        "steplet steplet 644",
        "the rest is checked out"
    );

    discard(&paths).unwrap();
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_wrong_address_or_commit_names_the_step_and_runs_nothing() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    let paths = RunPaths::new(&distro, "t6c").unwrap();
    let missing = "https://github.com/octocat/steplet-test-no-such-repository.git";
    for (url, commit, step, says) in [
        (missing, HELLO.1, "clone the repository", missing),
        (HELLO.0, "0000000000000000000000000000000000000001", "find the declared commit", HELLO.0),
    ] {
        match prepare(&request(url, commit), &paths, &gate("t6c")) {
            Err(PrepareError::Step { step: got, detail }) => {
                println!("--- {url} at {commit}: could not {got}:\n{detail}");
                assert_eq!(got, step, "{url} {commit}: {detail}");
                assert!(detail.contains(says), "the message names {says}: {detail}");
            }
            other => panic!("{url} {commit}: expected a failed step, got {other:?}"),
        }
        let checked_out =
            ["/usr/bin/find", paths.work_repo(), "-mindepth", "1", "-maxdepth", "1", "!", "-name", ".git"];
        assert_eq!(run(&distro, "root", &checked_out), "", "{url} {commit}: no files were checked out");
    }
    discard(&paths).unwrap();
}

/// Test data for Task 10 (made up, inert): a program that git starts as a "clean filter" while it lists the working
/// copy. It notes whether it could read the run's settings file (which the agent cannot read) and write outside the
/// working copy, then hands git a made-up text, so the listing shows that it ran.
const PROBE: &str = "head -c 1 /var/lib/steplet/runs/t10b/settings.json > /dev/null 2>&1 && echo read-the-run-s-settings >> /tmp/steplet-t10b-probe
echo wrote-outside-the-copy >> /tmp/steplet-t10b-probe
cat > /dev/null
echo made-up filtered text
";

/// Task 10: the agent writes the working copy, so Steplet's own listing of it after the run (`changes`) must reach no
/// more than the agent could: whatever git starts while it lists the copy runs inside the same fence as the checkout
/// (no network, nothing readable but the copy, nothing writable). Here the copy is set up, as root and in this test
/// only, as if every guard on `.git` had failed: its git config names the probe above as a filter for README (the
/// reviewer's route was a submodule's own `.git`, which the agent can write).
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn steplet_s_own_listing_of_the_copy_runs_inside_the_fence() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    let paths = RunPaths::new(&distro, "t10b").unwrap();
    prepare(&request(HELLO.0, HELLO.1), &paths, &gate("t10b")).unwrap();
    let plant = "set -e
cd /work/t10b/repo
printf '%s' \"$1\" > .steplet-test-probe.sh
printf 'README filter=probe\\n' > .gitattributes
git config -f .git/config filter.probe.clean 'sh .steplet-test-probe.sh'
touch -d 2001-01-01 README
rm -f /tmp/steplet-t10b-probe";
    run(&distro, "root", &["/bin/sh", "-c", plant, "sh", PROBE]);

    let listed = changes(&paths);
    let reached = "cat /tmp/steplet-t10b-probe 2> /dev/null; rm -f /tmp/steplet-t10b-probe";
    let reached = run(&distro, "root", &["/bin/sh", "-c", reached]);
    discard(&paths).unwrap();
    println!("--- listed: {listed:?}\n--- the probe reached: {reached:?}");
    assert_eq!(reached, "", "what the probe reached outside the working copy");
    let mut listed = listed.unwrap();
    listed.sort();
    assert_eq!(
        listed,
        [" M README", "?? .gitattributes", "?? .steplet-test-probe.sh"],
        "the probe ran (README shows its made-up text), and nothing of the fence's own is listed"
    );
}

/// Phase 4 Task 2: only the names `prepare.sh` knows count, each once and in its order, whatever else was printed.
#[test]
fn removed_files_counts_only_names_it_knows() {
    let said = "removed: .mcp.json\nremoved: .claude\nremoved: secrets\nremoved: .claude\n removed: .codex\nnoise\n";
    assert_eq!(steplet_run::removed_files(said), [".claude", ".mcp.json"]);
    assert_eq!(steplet_run::removed_files("steplet-prepare: ready at 7fd1a60b\n"), Vec::<String>::new());
}

/// Phase 4 Task 2: `prepare` says which commit the copy is at, and that it removed nothing from Hello-World.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn prepare_reports_the_commit_and_no_removed_files_for_hello_world() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    let paths = RunPaths::new(&distro, "p4t2a").unwrap();
    let prepared = prepare(&request(HELLO.0, HELLO.1), &paths, &gate("p4t2a"));
    discard(&paths).unwrap();
    let prepared = prepared.unwrap();
    assert_eq!(prepared, Prepared { commit: HELLO.1.into(), removed: vec![] });
}

/// Runs the strip entry of `prepare.sh` on its own, as root, with `arg` as its argument.
fn strip(distro: &str, arg: &str) -> wsl::Output {
    let script = include_bytes!("../src/prepare.sh");
    wsl::run(distro, "root", &["/bin/bash", "-s", "--", "strip", arg], script).unwrap()
}

/// Makes `copy` as root, with made-up agent settings files and one file of the task's, owned by the steplet account.
fn plant_agent_files(distro: &str, copy: &str) {
    let plant = format!(
        "set -e
rm -rf {copy}
mkdir -p {copy}/.claude {copy}/.codex {copy}/src
echo '{{}}' > {copy}/.claude/settings.json
echo '{{}}' > {copy}/.mcp.json
echo 'made-up = true' > {copy}/.codex/config.toml
echo 'kept' > {copy}/src/keep.txt
chown -R steplet:steplet {copy}"
    );
    run(distro, "root", &["/bin/sh", "-c", &plant]);
}

/// Phase 4 Task 2: the strip step of `prepare.sh`, on its own, on a made-up copy in a run's place. It removes every
/// agent settings file and folder it knows, prints one line for each it removed, in its fixed order, and leaves
/// everything else.
#[test]
#[ignore = "needs Steplet's Linux"]
fn the_strip_step_lists_what_it_removed() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    let copy = "/work/p4t2b/repo";
    plant_agent_files(&distro, copy);
    let out = strip(&distro, "p4t2b");
    let left = run(&distro, "root", &["/usr/bin/find", copy, "-mindepth", "1"]);
    run(&distro, "root", &["/usr/bin/rm", "-rf", "/work/p4t2b"]);
    println!("--- the strip step said:\n{}{}\n--- left in the copy:\n{left}", out.stdout, out.stderr);
    assert_eq!(out.status, Some(0), "{}", out.stderr);
    assert_eq!(out.stdout, "removed: .claude\nremoved: .codex\nremoved: .mcp.json\n");
    assert_eq!(steplet_run::removed_files(&out.stdout), [".claude", ".codex", ".mcp.json"]);
    let mut left: Vec<&str> = left.lines().collect();
    left.sort();
    assert_eq!(left, [format!("{copy}/src"), format!("{copy}/src/keep.txt")], "everything else is left");
}

/// The security check of Task 2: the strip step works only on a run's own working copy, `/work/<run id>/repo`. It
/// takes the run id, as Steplet's other scripts do, and refuses anything else without removing anything.
#[test]
#[ignore = "needs Steplet's Linux"]
fn the_strip_step_refuses_anything_but_a_run_s_working_copy() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    let outside = "/opt/steplet-test/strip-outside";
    plant_agent_files(&distro, outside);
    let mut said = Vec::new();
    for arg in [outside, "../../opt/steplet-test/strip-outside", "", "-x", "p4t2x/../.."] {
        let out = strip(&distro, arg);
        said.push((arg, out.status, out.stdout, out.stderr.trim_end().to_owned()));
    }
    let left = run(&distro, "root", &["/usr/bin/find", outside, "-maxdepth", "1", "-name", ".*"]);
    run(&distro, "root", &["/usr/bin/rm", "-rf", outside]);
    println!("--- the strip step said: {said:#?}\n--- left: {left}");
    for (arg, status, stdout, _) in &said {
        assert_eq!((status, stdout.as_str()), (&Some(14), ""), "{arg:?} was refused, and nothing was removed");
    }
    let mut left: Vec<&str> = left.lines().collect();
    left.sort();
    assert_eq!(left, [format!("{outside}/.claude"), format!("{outside}/.codex"), format!("{outside}/.mcp.json")]);
}

/// The security check of Task 2: while the strip step runs, no program of the steplet account may still be running,
/// or it could put an agent settings file back after its removal was said. Here a made-up loop of that account keeps
/// making `.claude` again; the strip step ends it first, so `.claude` stays gone.
#[test]
#[ignore = "needs Steplet's Linux"]
fn the_strip_step_stops_the_steplet_account_first() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    let copy = "/work/p4t2c/repo";
    plant_agent_files(&distro, copy);
    let again = format!("while :; do mkdir -p {copy}/.claude 2> /dev/null; sleep 0.05; done");
    let mut looping = wsl::spawn(&distro, "steplet", &["/bin/bash", "-c", &again]).unwrap();
    drop(looping.stdin.take());
    let out = strip(&distro, "p4t2c");
    std::thread::sleep(Duration::from_millis(500));
    let left =
        run(&distro, "root", &["/bin/sh", "-c", &format!("test -e {copy}/.claude && echo present || echo absent")]);
    let running = run(&distro, "root", &["/bin/sh", "-c", "pgrep -u steplet > /dev/null && echo running || echo none"]);
    run(&distro, "root", &["/bin/sh", "-c", "pkill -KILL -u steplet; rm -rf /work/p4t2c"]);
    let _ = looping.kill();
    let _ = looping.wait();
    println!(
        "--- the strip step said:\n{}{}\n--- .claude afterwards: {left}; steplet programs: {running}",
        out.stdout, out.stderr
    );
    assert_eq!(out.status, Some(0), "{}", out.stderr);
    assert_eq!((left.as_str(), running.as_str()), ("absent", "none"));
}
