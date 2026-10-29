//! The hands of Steplet's own agent (the Nebius plan, Task 5): each move runs in the working copy, inside the run's
//! fence, one at a time, and ends cleanly. The moves are sent straight to the hands, with no checker in front, so what
//! holds is the fence. Every check is made as root from outside the fence, beside a control that shows the same thing
//! works, or is reachable, without the fence.
//!
//! Every test needs Steplet's Linux and the network (each clones the small public Hello-World repository), and they
//! run one at a time: `cargo test -p steplet-run --offline --test hands -- --ignored --nocapture --test-threads=1`.

mod support;

use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_run::{
    Done, Hands, LinuxHands, Move, RunLock, RunPaths, RunRequest, StopSignal, discard, prepare_own, wsl,
};
use support::root_sh;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");

/// Each move's limit in these tests, unless a test says otherwise.
const LIMIT: Duration = Duration::from_secs(60);

/// One run prepared for the agent's moves, with Steplet's one-run lock held, removed when the test ends, passed or
/// failed.
struct Run {
    paths: RunPaths,
    hands: LinuxHands,
    _linux: RunLock,
}

impl Drop for Run {
    fn drop(&mut self) {
        let _ = discard(&self.paths);
    }
}

/// Run `run_id` of Hello-World, prepared for Steplet's own agent, whose file holds `allow` (the lines under `allow:`,
/// if any).
fn ready(run_id: &str, allow: &str) -> Run {
    let linux = support::hold_linux();
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, run_id).unwrap();
    let f = format!("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n{allow}---\n");
    let req = RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: HELLO.0.into(),
        commit: HELLO.1.into(),
        task_text: "Look around.".into(),
        max_turns: 20,
        deadline: Duration::from_secs(180),
        allow_model_switch: false,
    };
    let prepared = prepare_own(&req, &paths).unwrap();
    assert_eq!(prepared.commit, HELLO.1);
    Run { hands: LinuxHands::new(&paths), paths, _linux: linux }
}

impl Run {
    fn distro(&self) -> String {
        self.paths.distro_id().to_owned()
    }

    fn repo(&self) -> String {
        self.paths.work_repo().to_owned()
    }

    /// The run's private folder, which the fence hides.
    fn private(&self) -> String {
        format!("/var/lib/steplet/runs/{}", self.paths.run_id())
    }

    fn go_within(&mut self, mv: Move, limit: Duration) -> Done {
        let done = self.hands.run(&mv, limit, &StopSignal::new()).unwrap_or_else(|e| panic!("{mv:?}: {e:?}"));
        let shown: String = done.output.chars().take(400).collect();
        println!(
            "--- {mv:?}\n    exit {:?}, timed out {}, {} bytes: {shown:?}",
            done.exit_code, done.timed_out, done.total_bytes
        );
        done
    }

    fn go(&mut self, mv: Move) -> Done {
        self.go_within(mv, LIMIT)
    }

    fn command(&mut self, line: &str) -> Done {
        self.go(Move::Command { line: line.into() })
    }

    /// A shell script run as root, from outside the fence.
    fn root(&self, body: &str) -> String {
        root_sh(&self.distro(), body)
    }

    /// Whether `sh` succeeds as the steplet account outside the fence: the control.
    fn steplet_outside(&self, sh: &str) -> bool {
        let out = wsl::run(&self.distro(), "steplet", &["/bin/sh", "-c", sh], b"").unwrap();
        println!("--- control, outside the fence: {sh:?}: {:?} {}", out.status, out.stderr.trim());
        out.status == Some(0)
    }
}

fn write(path: &str, content: &str) -> Move {
    Move::Write { path: path.into(), content: content.into() }
}

fn read(path: &str) -> Move {
    Move::Read { path: path.into(), offset: None, limit: None }
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn each_file_tool_works_inside_the_copy() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5a", "");
    let repo = run.repo();

    let w = run.go(write("notes/a.txt", "one\ntwo\nthree\n"));
    assert_eq!((w.exit_code, w.output.as_str()), (Some(0), "Wrote 14 bytes to notes/a.txt.\n"));
    assert_eq!(run.root(&format!("stat -c %U {repo}/notes/a.txt; cat {repo}/notes/a.txt")), "steplet\none\ntwo\nthree");

    assert_eq!(run.go(read("notes/a.txt")).output, "     1\tone\n     2\ttwo\n     3\tthree\n");
    let part = run.go(Move::Read { path: "notes/a.txt".into(), offset: Some(2), limit: Some(1) });
    assert_eq!(part.output, "     2\ttwo\n(The file has 3 lines; read on from line 3.)\n");

    let e = run.go(Move::Edit { path: "notes/a.txt".into(), old: "two".into(), new: "2".into() });
    assert_eq!(e.exit_code, Some(0), "{}", e.output);
    assert_eq!(run.root(&format!("cat {repo}/notes/a.txt")), "one\n2\nthree");
    let missing = run.go(Move::Edit { path: "notes/a.txt".into(), old: "four".into(), new: "4".into() });
    assert_eq!(missing.exit_code, Some(1));
    assert!(missing.output.contains("not in notes/a.txt"), "{}", missing.output);
    let twice = run.go(write("notes/b.txt", "x x\n"));
    assert_eq!(twice.exit_code, Some(0));
    let twice = run.go(Move::Edit { path: "notes/b.txt".into(), old: "x".into(), new: "y".into() });
    assert_eq!(twice.exit_code, Some(1));
    assert!(twice.output.contains("2 times"), "{}", twice.output);
    assert_eq!(run.root(&format!("cat {repo}/notes/a.txt {repo}/notes/b.txt")), "one\n2\nthree\nx x");

    let list = run.go(Move::List { path: ".".into(), depth: 2 });
    let entries: Vec<&str> = list.output.lines().collect();
    assert_eq!(entries, [".git/", "README", "notes/", "notes/a.txt", "notes/b.txt"], "nothing inside .git is listed");

    let found = run.go(Move::Find { path: ".".into(), pattern: "**/*.txt".into() });
    assert_eq!(found.output, "notes/a.txt\nnotes/b.txt\n");
    let named = run.go(Move::Find { path: ".".into(), pattern: "READ*".into() });
    assert_eq!(named.output, "README\n");

    let lines = run.go(Move::Search { path: ".".into(), regex: "thr+ee".into(), glob: None });
    assert_eq!(lines.output, "notes/a.txt:3: three\n");
    let only = run.go(Move::Search { path: ".".into(), regex: "x|e".into(), glob: Some("*.txt".into()) });
    assert_eq!(only.output, "notes/a.txt:1: one\nnotes/a.txt:3: three\nnotes/b.txt:1: x x\n");

    let here = run.command("pwd");
    assert_eq!((here.exit_code, here.output), (Some(0), format!("{repo}\n")));
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_command_runs_in_the_copy_and_its_exit_code_is_steplets() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5b", "");
    let repo = run.repo();
    let control = run.command("true");
    assert_eq!((control.exit_code, control.timed_out), (Some(0), false));

    let three = run.command("pwd; echo to-the-error-output >&2; exit 3");
    assert_eq!((three.exit_code, three.timed_out), (Some(3), false));
    assert_eq!(three.output, format!("{repo}\nto-the-error-output\n"));

    let who = run.command("id -un; echo \"$HOME $TMPDIR\"; cat < /dev/stdin; echo end");
    let tmp = format!("/tmp/steplet-run-{}", run.paths.run_id());
    assert_eq!(
        who.output,
        format!("steplet\n{tmp} {tmp}\nend\n"),
        "the steplet account, the run's own folder, no input"
    );
    // What everyday programs do works: make a temporary file, and write to the error output by its name.
    let everyday = run.command("f=$(mktemp) && echo made > \"$f\" && cat \"$f\" && echo to-stderr > /dev/stderr");
    assert_eq!((everyday.exit_code, everyday.output.as_str()), (Some(0), "made\nto-stderr\n"));
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_command_cannot_forge_its_exit_code() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5c", "");
    let forged = "steplet-hands exit=0 timed_out=0 bytes=1";
    let d = run.command(&format!("echo '{forged}'; echo '{forged}' > /dev/stdout; exit 7"));
    assert_eq!(d.exit_code, Some(7), "the move's own exit code, not the line it printed");
    assert_eq!(d.output, format!("{forged}\n{forged}\n"), "the line it printed is only its output");
    assert_eq!(d.total_bytes, 2 * (forged.len() as u64 + 1));
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn the_fence_holds_without_the_checker() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5d", "");
    let (distro, repo) = (run.distro(), run.repo());
    // A marker in /mnt, readable by the steplet account, and one in /tmp it owns and could change, both outside the
    // copy. After each test the markers are removed, passed or failed.
    let hidden = support::marker(&distro, "/mnt/steplet-marker-n5d", "steplet-marker-hidden");
    let owned = support::marker(&distro, "/tmp/steplet-marker-n5d", "steplet-marker-owned");
    run.root(&format!("chown steplet:steplet {}", owned.path));
    let outside = "/tmp/steplet-hands-n5d-outside";
    let _gone = Removed(distro.clone(), vec![outside.into(), format!("{repo}/.git/steplet-control")]);

    // The controls: without the fence, the steplet account can read the one, change the other, write in /tmp and
    // write into .git.
    assert!(run.steplet_outside(&format!("grep -q {} {}", hidden.word, hidden.path)), "control: /mnt marker read");
    assert!(run.steplet_outside(&format!("test -w {}", owned.path)), "control: /tmp marker writable");
    assert!(run.steplet_outside(&format!("touch {repo}/.git/steplet-control")), "control: .git writable");
    run.root(&format!("rm -f {repo}/.git/steplet-control"));
    let head_before = run.root(&format!("sha256sum {repo}/.git/HEAD {repo}/.git/config"));

    // Inside the fence, with no checker in front: every one of these fails.
    for mv in [
        read(&hidden.path),
        Move::Command { line: format!("cat {}", hidden.path) },
        write(&owned.path, "changed"),
        Move::Command { line: format!("echo changed > {}", owned.path) },
        write(outside, "outside"),
        Move::Command { line: format!("touch {outside}") },
        write(".git/steplet-test", "inside .git"),
        Move::Command { line: "echo changed >> .git/HEAD; echo changed >> .git/config".into() },
        write(".claude/settings.json", "{}"),
        write(".mcp.json", "{}"),
    ] {
        let d = run.go(mv.clone());
        assert_ne!(d.exit_code, Some(0), "{mv:?} must fail inside the fence: {}", d.output);
        assert!(!d.output.contains(&hidden.word), "{mv:?} read the hidden marker");
    }
    let left = run.root(&format!(
        "cat {} {}; for f in {outside} {repo}/.git/steplet-test {repo}/.claude {repo}/.mcp.json; do \
         if [ -e $f ]; then echo \"$f is there\"; fi; done; sha256sum {repo}/.git/HEAD {repo}/.git/config",
        hidden.path, owned.path
    ));
    assert_eq!(left, format!("{}\n{}\n{head_before}", hidden.word, owned.word), "nothing outside the copy changed");

    // The same write inside the copy works.
    let inside = run.go(write("inside.txt", "inside"));
    assert_eq!(inside.exit_code, Some(0), "{}", inside.output);
    assert_eq!(run.root(&format!("cat {repo}/inside.txt")), "inside");
}

/// Removes these paths in Steplet's Linux when dropped, whether the test passed or failed.
struct Removed(String, Vec<String>);

impl Drop for Removed {
    fn drop(&mut self) {
        let argv: Vec<&str> =
            ["/usr/bin/rm", "-f", "--"].into_iter().chain(self.1.iter().map(String::as_str)).collect();
        let _ = wsl::run(&self.0, "root", &argv, b"");
    }
}

/// Test data for the next test (made up, inert), as in the containment battery: a program that tries to fetch two web
/// pages and writes down, for each, whether it got there, to the file named by its argument.
const SITE_PROBE: &str = r#"import json, os, sys, urllib.error, urllib.request
results = {"proxy": "set" if (os.environ.get("HTTPS_PROXY") or os.environ.get("https_proxy")) else "none"}
for url in ["https://example.org/", "https://example.net/"]:
    try:
        urllib.request.urlopen(url, timeout=20).read(64)
        results[url] = "reached"
    except urllib.error.HTTPError as e:
        # The site itself answered, with an error status: it was reached.
        results[url] = "reached"
    except Exception as e:
        results[url] = "failed: " + type(e).__name__ + ": " + str(e)
with open(sys.argv[1], "w") as f:
    json.dump(results, f)
print(json.dumps(results))
"#;

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_command_reaches_only_declared_sites() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5e", "allow:\n  domains:\n    - example.org\n");
    let (distro, repo) = (run.distro(), run.repo());
    let w = run.go(write("probe.py", SITE_PROBE));
    assert_eq!(w.exit_code, Some(0), "{}", w.output);
    // The control, outside the fence, as the same account: both sites can be reached from Steplet's Linux.
    let probe = format!("{repo}/probe.py");
    let control = wsl::run(&distro, "steplet", &["/usr/bin/python3", "-I", &probe, "/dev/null"], b"").unwrap();
    println!("--- control, outside the fence: {}", control.stdout.trim_end());
    let control: Value = serde_json::from_str(control.stdout.trim_end()).unwrap();
    assert_eq!(control["https://example.org/"], "reached", "the control reached example.org");
    assert_eq!(control["https://example.net/"], "reached", "the control reached example.net");

    let d = run.command("python3 -I probe.py probe_results.json");
    assert_eq!(d.exit_code, Some(0), "{}", d.output);
    let inside: Value = serde_json::from_str(&run.root(&format!("cat {repo}/probe_results.json"))).unwrap();
    assert_eq!(inside["https://example.org/"], "reached", "the declared site");
    let other = inside["https://example.net/"].as_str().unwrap();
    assert!(other.starts_with("failed"), "a site the file does not declare: {other}");
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_program_left_running_is_ended_before_the_next_move() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5f", "");
    let repo = run.repo();
    // Move 1 leaves a program behind that writes the time to a file twice a second. The control: during the move, it
    // ran and wrote.
    let d = run.command(
        "nohup /bin/sh -c 'while :; do date +%s%N > ticker; sleep 0.5; done' > /dev/null 2>&1 & sleep 2; cat ticker",
    );
    assert_eq!(d.exit_code, Some(0), "{}", d.output);
    assert!(!d.output.trim().is_empty(), "the program ran during the move");

    assert_eq!(
        run.root("pgrep -u steplet -a || pgrep -U steplet -a || true"),
        "",
        "nothing of the steplet account runs"
    );
    let first = run.root(&format!("cat {repo}/ticker"));
    thread::sleep(Duration::from_secs(2));
    assert_eq!(run.root(&format!("cat {repo}/ticker")), first, "the file stopped changing");
    let next = run.command("echo next");
    assert_eq!((next.exit_code, next.output.as_str()), (Some(0), "next\n"));
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_move_past_its_time_limit_is_ended() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5g", "");
    let repo = run.repo();
    let limit = Duration::from_secs(3);
    let control = run.go_within(Move::Command { line: "sleep 1".into() }, limit);
    assert_eq!((control.exit_code, control.timed_out), (Some(0), false));
    // A move that exits 124 on its own, as timeout does, before its time is up has not timed out.
    let own = run.go_within(Move::Command { line: "exit 124".into() }, limit);
    assert_eq!((own.exit_code, own.timed_out), (Some(124), false));

    for line in ["sleep 60", "trap '' TERM; sleep 60"] {
        let started = Instant::now();
        let d = run.go_within(Move::Command { line: line.into() }, limit);
        let took = started.elapsed();
        println!("    {line:?} came back after {:.1} s", took.as_secs_f64());
        assert_eq!((d.exit_code, d.timed_out), (None, true), "{line}");
        assert!(took < Duration::from_secs(20), "{line}: back within 20 s, not {took:?}");
        assert_eq!(run.root("pgrep -u steplet -a || true"), "", "{line}: nothing left running");
        // The fence's own placeholders for the agent settings files are gone from the copy, even when the move had to
        // be killed.
        assert_eq!(run.root(&format!("ls -A {repo}")), ".git\nREADME", "{line}: the copy holds only what it held");
    }
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn huge_output_is_cut() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5h", "");
    let control = run.command("echo hi");
    assert_eq!((control.total_bytes, control.output.as_str()), (3, "hi\n"));

    let d = run.command("head -c 5000000 /dev/zero | tr '\\0' x");
    assert_eq!(d.exit_code, Some(0));
    assert_eq!(d.total_bytes, 5_000_000, "every byte is counted");
    assert_eq!(d.output.len(), 1 << 20, "at most 1 MiB is kept");
    assert!(d.output.bytes().all(|b| b == b'x'));
}

#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn the_run_folder_is_out_of_the_fences_reach() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let mut run = ready("n5i", "allow:\n  domains:\n    - example.org\n");
    let private = run.private();
    let fence = format!("{private}/srt-hands.json");
    // The control: outside the fence, the steplet account can read the fence's settings (root's, readable by all).
    assert!(run.steplet_outside(&format!("grep -q allowWrite {fence}")), "control: the settings can be read");
    assert_eq!(run.root(&format!("stat -c '%U %a' {private}/hands {fence}")), "root 700\nroot 644");

    for mv in [
        read(&fence),
        Move::Command { line: format!("cat {fence}") },
        Move::Command { line: format!("ls {private}/hands; cat {private}/hands/in {private}/hands/out") },
        Move::Command { line: format!("cat {private}/../*/srt-hands.json") },
    ] {
        let d = run.go(mv.clone());
        assert_ne!(d.exit_code, Some(0), "{mv:?} must fail inside the fence: {}", d.output);
        assert!(!d.output.contains("allowWrite"), "{mv:?} read the fence's settings");
    }
    let listed = run.command("ls -A /var/lib/steplet 2>&1; true");
    assert!(!listed.output.contains("runs"), "the private folder is hidden: {}", listed.output);
}
