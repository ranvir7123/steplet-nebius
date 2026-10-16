//! The working copy, prepared inside Steplet's own Linux (plan Task 6). The tests marked `#[ignore]` need that Linux
//! (set up by `tests/support/setup_steplet.sh`) and the network, and run only when asked:
//! `cargo test -p steplet-run --offline --test prepare -- --ignored`. They share `/work`, so they run one at a time.

use std::sync::Mutex;
use std::time::Duration;

use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_run::{GateLink, PrepareError, RunPaths, RunRequest, discard, mcp_json, prepare, settings_json, wsl};

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
        assert!(!matches!(got, Err(PrepareError::Refused(_)) | Ok(())), "{url} {commit}: {got:?}");
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
        "config\nhome\nmcp.json\nsettings.json\ntmp"
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
    assert_eq!(file(paths.settings()), settings_json(&req.steplet, &paths, &g), "the settings file, byte for byte");
    assert_eq!(file(paths.mcp()), mcp_json(&g), "the MCP config, byte for byte");
    for home in [paths.run_home(), paths.run_config(), paths.run_tmp()] {
        assert_eq!(run(&distro, "root", &["/usr/bin/ls", "-A", home]), "", "{home} starts empty");
    }

    discard(&paths).unwrap();
    assert_eq!(
        run(&distro, "root", &["/usr/bin/ls", "-A", "/work", "/var/lib/steplet/runs"]),
        "/var/lib/steplet/runs:\n\n/work:"
    );
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn agent_settings_files_the_commit_carries_are_checked_out_then_removed() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let distro = steplet_linux();
    let paths = RunPaths::new(&distro, "t6b").unwrap();
    prepare(&request(ACTION.0, ACTION.1), &paths, &gate("t6b")).unwrap();

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
