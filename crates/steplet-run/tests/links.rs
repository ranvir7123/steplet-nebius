//! Phase 5, Task 3 (phase 3's paused Task 3): the agent's own file tools (Read, Glob, Grep, Write, Edit) run outside
//! the fence, so these tests check them directly against links in the working copy that point out of it. This is
//! defensive testing of Steplet's own sandbox: every test runs only in Steplet's own Linux, against Steplet's own code,
//! and what the links lead to is harmless: two notes the test puts there itself, each holding a made-up word
//! (`support::marker`); the public file `/etc/hostname`; a Windows path that does not exist in Steplet's Linux; and
//! the run's own folder. Judged from Steplet's records and from outside the fence (root looks at the notes after the
//! run), never from what the agent says (F47).
//!
//! The working copy is a tiny repository (`support/links_fixture/`): a README, `notes/`, and two small programs:
//! `make_links.py` makes the links in `notes/` from the run's own id, and `look.py` writes down what a program the task
//! runs sees at the run's own folder.
//!
//! Every test needs Steplet's Linux and the network (each run first fetches a small public repository), and they run
//! one at a time: `cargo test -p steplet-run --offline --test links -- --ignored --nocapture --test-threads=1`.

mod support;

use std::sync::Mutex;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{Asked, Event, RunHandle, RunPaths, RunRequest, call_what, decide, discard, launch, prepare, wsl};
use support::{MADE_UP_KEY, Marker, logs_dir, root_sh, scripted_model, serve_gate};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");

/// The scripted model's last words.
const THE_END: &str = "Task 3: the end of the script.";

/// The declared commands: one looks at the run's own folder from inside the fence, one makes the links.
const LOOK: &str = "python3 look.py";
const MAKE_LINKS: &str = "python3 make_links.py";

/// The fixture's files, as they go into the working copy. The links are not among them: `make_links.py` makes them in
/// Steplet's Linux, since their targets name the run's own folders.
const FIXTURE: &[(&str, &[u8])] = &[
    ("README.md", include_bytes!("support/links_fixture/README.md")),
    ("notes/README.md", include_bytes!("support/links_fixture/notes/README.md")),
    ("make_links.py", include_bytes!("support/links_fixture/make_links.py")),
    ("look.py", include_bytes!("support/links_fixture/look.py")),
];

/// The made-up words of the two notes outside the working copy. No tool call names either in full, so a word in an
/// answer was read from its note (or from a record of that read), never echoed from the call itself.
const HOME_WORD: &str = "steplet-marker-run-home";
const TMP_WORD: &str = "steplet-marker-run-tmp";

/// A request for Hello-World whose file declares the two commands and lets the task change files in `notes/`, so a
/// write to a link there is allowed by the checker: whether it lands outside the copy is then up to the file tools.
fn request() -> RunRequest {
    let f = format!(
        "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n    - {LOOK}\n    \
         - {MAKE_LINKS}\n  write:\n    - notes\n---\n"
    );
    RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: HELLO.0.into(),
        commit: HELLO.1.into(),
        task_text: "Follow the script.".into(),
        max_turns: 40,
        deadline: Duration::from_secs(180),
        allow_model_switch: false,
    }
}

/// Runs `command` as the steplet account with nobody's settings, as `prepare.sh` runs git.
const AS_STEPLET: &str = "setpriv --reuid=steplet --regid=steplet --init-groups --inh-caps=-all --no-new-privs -- \
                          /usr/bin/env -i PATH=/usr/bin:/bin HOME=/nonexistent GIT_CONFIG_NOSYSTEM=1 \
                          GIT_CONFIG_GLOBAL=/dev/null";

/// Replaces the prepared working copy's files with the fixture's, as a new commit (test only: P3 lets Steplet clone
/// only from public code hosts, and the fixture is on none). With `links`, the steplet account first runs
/// `make_links.py` outside the fence, so the links arrive with the repository, as a stranger's repository would bring
/// them. Prints the copy's commit and what `notes/` holds.
fn plant(distro: &str, paths: &RunPaths, links: bool) {
    let repo = paths.work_repo();
    let git = format!(
        "{AS_STEPLET} git -c core.hooksPath=/dev/null -c user.name=steplet-test -c user.email=test@example.invalid -C \
         {repo}"
    );
    root_sh(distro, &format!("set -e\n{git} rm -q -r -- ."));
    for (path, bytes) in FIXTURE {
        support::install(distro, bytes, &format!("{repo}/{path}"), "644");
    }
    let make = if links { format!("cd {repo}\n{AS_STEPLET} python3 -I make_links.py\n") } else { String::new() };
    let body = format!(
        "set -e
chown -R steplet:steplet {repo}
chmod -R go-w {repo}
{make}{git} add -A
{git} commit -q -m 'Test data: a README, notes and two small programs'
{git} log --oneline -1
find {repo}/notes -mindepth 1 -printf '%P -> %l\\n' | sort"
    );
    println!("--- the working copy, committed (notes/ entries, with each link's target):\n{}", root_sh(distro, &body));
}

/// The two notes outside the working copy: one in the run's own folder (its home), one in its temporary folder, each
/// holding its own made-up word. Placed by root once `prepare` has made those folders, then given to the steplet
/// account, so that a write that followed a link could change them. Removed when dropped (and with the run's folders).
fn place_notes(distro: &str, paths: &RunPaths) -> [Marker; 2] {
    let home = support::marker(distro, &format!("{}/run-home-note.txt", paths.run_home()), HOME_WORD);
    let tmp = support::marker(distro, &format!("{}/run-tmp-note.txt", paths.run_tmp()), TMP_WORD);
    root_sh(distro, &format!("chown steplet:steplet {} {}", home.path, tmp.path));
    // The control: outside the fence, the steplet account can read and change both, so a call that followed a link to
    // either could read or change it.
    for note in [&home, &tmp] {
        for check in ["-r", "-w"] {
            let out = wsl::run(distro, "steplet", &["/usr/bin/test", check, &note.path], b"").unwrap();
            assert_eq!(
                out.status,
                Some(0),
                "outside the fence, the steplet account passes `test {check}` on {}",
                note.path
            );
        }
    }
    [home, tmp]
}

/// What root sees of a file from outside the fence: its kind, owner and SHA-256, or that it is missing.
fn fingerprint(distro: &str, path: &str) -> String {
    root_sh(
        distro,
        &format!(
            "if test -e {path}; then stat -c '%F %U' {path}; sha256sum < {path} | cut -c1-64; else echo missing; fi"
        ),
    )
    .replace('\n', " ")
}

fn read(path: &str) -> Value {
    json!({"tool": "Read", "input": {"file_path": path}})
}

/// Changes the word "marker" to "changed": a note that went through it would hold another word afterwards.
fn edit(path: &str) -> Value {
    json!({"tool": "Edit", "input": {"file_path": path, "old_string": "marker", "new_string": "changed"}})
}

fn write(path: &str) -> Value {
    json!({"tool": "Write", "input": {"file_path": path, "content": "made-up test text\n"}})
}

fn glob(path: &str) -> Value {
    json!({"tool": "Glob", "input": {"pattern": "**/*", "path": path}})
}

/// Searches for the start that both made-up words share, and shows the lines found.
fn grep(path: &str) -> Value {
    json!({"tool": "Grep", "input": {"pattern": "steplet-marker", "path": path, "output_mode": "content"}})
}

/// Every event of a launched run, until it ends or `limit` passes, without the tripwire watcher: a measurement sees
/// every call the script makes, where a real run would stop at the first tripwire.
fn every_event(handle: &RunHandle, limit: Duration) -> Vec<Event> {
    let start = Instant::now();
    let mut events = Vec::new();
    while start.elapsed() < limit {
        match handle.events().recv_timeout(Duration::from_secs(1)) {
            Ok(e) => events.push(e),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    events
}

/// One tool call of a run: the tool, what it was about, what the gate saw and answered (`None`: it was never asked),
/// whether Claude Code's answer was an error, and the answer's text.
struct Call {
    tool: String,
    what: String,
    gate: Option<(String, String)>,
    error: bool,
    answer: String,
}

fn calls(events: &[Event], asked: &[Asked]) -> Vec<Call> {
    events
        .iter()
        .filter_map(|e| {
            let Event::ToolUse { id, name, input } = e else { return None };
            let gate = asked.iter().find_map(|a| match a {
                Asked::Decision { request, answer, .. } if &request.tool_use_id == id => {
                    Some((call_what(&request.tool_name, &request.input), format!("{answer:?}")))
                }
                _ => None,
            });
            let (error, answer) = events
                .iter()
                .find_map(|r| match r {
                    Event::ToolResult { id: rid, is_error, text, .. } if rid == id => Some((*is_error, text.clone())),
                    _ => None,
                })
                .unwrap_or((true, "(no answer)".into()));
            Some(Call { tool: name.clone(), what: call_what(name, input), gate, error, answer })
        })
        .collect()
}

/// The made-up words (and, as `hostname`, the public file's text) that an answer holds.
fn words_in(answer: &str, hostname: &str) -> Vec<&'static str> {
    let mut found: Vec<&'static str> = [HOME_WORD, TMP_WORD].into_iter().filter(|w| answer.contains(w)).collect();
    if answer.contains(hostname) {
        found.push("hostname");
    }
    found
}

/// Step 1, a measurement kept as a record: what each of the agent's file tools does with each link out of the
/// working copy, with Steplet's settings and Claude Code 2.1.220. For each call: whether it reached the gate, with
/// which path, and the answer; Claude Code's answer, and which made-up word it holds; and, from outside after the run,
/// whether either note changed. Also what a declared command sees at the run's own folder from inside the fence
/// (phase 3 Task 2's open "C line"). The computer's name (`/etc/hostname`) is never printed: the table says only
/// whether an answer held it. Records only; the tests below pin the result.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn how_the_agent_s_file_tools_meet_links_out_of_the_copy() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let _linux = support::hold_linux();
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "p5t3m").unwrap();
    let req = request();
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let note = |name: &str| format!("{}/notes/{name}", paths.work_repo());
    let probe = format!("{}/links-probe.txt", paths.run_config());
    let steps = vec![
        read(&note("run-home.txt")),
        edit(&note("run-home.txt")),
        write(&note("run-home.txt")),
        read(&note("run-tmp.txt")),
        edit(&note("run-tmp.txt")),
        write(&note("run-tmp.txt")),
        read(&note("hostname.txt")),
        read("/etc/hostname"),
        read(&note("windows.txt")),
        read(&note("run-folder/home/run-home-note.txt")),
        write(&note("run-folder/config/links-probe.txt")),
        glob(&note("run-folder")),
        glob(&format!("{}/notes", paths.work_repo())),
        grep(&note("run-folder")),
        grep(&format!("{}/notes", paths.work_repo())),
        grep(&note("run-home.txt")),
        grep(&note("run-tmp.txt")),
        json!({"tool": "Bash", "input": {"command": LOOK, "description": "Look at the run's folder"}}),
        json!({"text": THE_END}),
    ];
    let scripted_calls = steps.len() - 1;
    let script = json!({ "steps": steps });
    let checker = policy.clone();
    let (link, gate) = serve_gate(&distro, "p5t3m", Duration::from_secs(10), move |r| decide(&checker, r));
    let _model = scripted_model(&distro, "p5t3m", &script.to_string(), 47980);
    prepare(&req, &paths, &link).unwrap();
    plant(&distro, &paths, true);
    let notes = place_notes(&distro, &paths);
    let before: Vec<String> = notes.iter().map(|n| fingerprint(&distro, &n.path)).collect();
    let hostname = root_sh(&distro, "cat /etc/hostname");
    assert!(!hostname.is_empty(), "/etc/hostname holds the computer's name");

    let start = Instant::now();
    let handle =
        launch(&req, &paths, &logs_dir(), Some("http://127.0.0.1:47980"), &[MADE_UP_KEY, link.token()]).unwrap();
    let events = every_event(&handle, Duration::from_secs(200));
    let code = handle.wait().unwrap();
    let took = start.elapsed();
    std::thread::sleep(Duration::from_millis(500));
    let asked = gate.requests();
    drop(gate);
    let after: Vec<String> = notes.iter().map(|n| fingerprint(&distro, &n.path)).collect();
    let probe_after = fingerprint(&distro, &probe);
    let notes_after = root_sh(&distro, &format!("find {} -mindepth 1 -printf '%P -> %l\\n' | sort", note("")));
    discard(&paths).unwrap();

    let hide = |t: &str| {
        t.replace(&hostname, "[the computer's name]").replace(MADE_UP_KEY, "[key]").replace(link.token(), "[token]")
    };
    let shown = |t: &str, n: usize| hide(t).replace('\n', " | ").chars().take(n).collect::<String>();
    let calls = calls(&events, &asked);
    println!(
        "--- {:.1} s, exit code {code:?}; each call, whether it reached the gate, and Claude Code's answer:",
        took.as_secs_f64()
    );
    for (i, c) in calls.iter().enumerate() {
        let gate = match &c.gate {
            Some((saw, answer)) => format!("asked, the gate saw {:?}: {}", shown(saw, 90), shown(answer, 110)),
            None => "never asked".into(),
        };
        println!("{:>2}. {} {}\n      gate: {gate}", i + 1, c.tool, shown(&c.what, 110));
        println!("      answer (error: {}): {}", c.error, shown(&c.answer, 260));
        println!("      made-up words in the answer: {:?}", words_in(&c.answer, &hostname));
    }
    for a in &asked {
        if let Asked::Refused { reason, call, .. } = a {
            println!("--- refused by the gate before any decision: {reason} ({call:?})");
        }
    }
    println!("--- from outside, after the run:");
    for (n, (b, a)) in notes.iter().zip(before.iter().zip(&after)) {
        println!("    {}: before {b}; after {a}; {}", n.path, if a == b { "unchanged" } else { "CHANGED" });
    }
    println!("    {probe}: {probe_after}");
    println!("    notes/ in the working copy now:\n{}", hide(&notes_after));

    assert_eq!(calls.len(), scripted_calls, "every scripted call was made");
    let look = calls.last().unwrap();
    assert_eq!((look.tool.as_str(), look.what.as_str()), ("Bash", LOOK));
    println!("--- what the declared command saw from inside the fence (look.py):\n{}", hide(&look.answer));
}
