//! Phase 5, Task 5 (R20 item 3): nothing the task or the agent does during a run can weaken it. Agent settings files
//! written during the run take no effect, the pinned Claude Code in Steplet's Linux is the same after a run that tried
//! to add files to it, and a tool the run does not offer never runs. This is defensive testing of Steplet's own
//! sandbox: every test runs only in Steplet's own Linux, against Steplet's own code, with harmless content: a hook
//! whose only effect would be to write a file holding a made-up word, a small text file the task tries to add next to
//! Claude Code, and the made-up key. Judged from Steplet's records and from outside the fence (root looks before the
//! run is removed), never from what the agent says (F47).
//!
//! Every test needs Steplet's Linux and the network (each run first fetches a small public repository), and they run
//! one at a time: `cargo test -p steplet-run --offline --test settings_live -- --ignored --nocapture --test-threads=1`.

mod support;

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    Asked, BACKSTOP, Check, Event, GateAnswer, Live, Report, ReportCommand, RunOutcome, RunPaths, RunRequest, Tripwire,
    Trust, build_report, decide, discard, launch, prepare, setup, watch_run,
};
use support::{AS_STEPLET, MADE_UP_KEY, every_event, logs_dir, root_sh, scripted_model, serve_gate};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

const HELLO: (&str, &str) = ("https://github.com/octocat/Hello-World.git", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d");

/// The scripted model's last words.
const THE_END: &str = "Task 5: the end of the script.";

/// A request for Hello-World whose file declares `commands`, which the checker allows: they ask the gate and run in
/// the fence.
fn request(commands: &[&str]) -> RunRequest {
    let declared: String = commands.iter().map(|c| format!("    - {c}\n")).collect();
    let f = format!(
        "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n{declared}---\n"
    );
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

fn bash(command: &str) -> Value {
    json!({"tool": "Bash", "input": {"command": command, "description": "A declared command"}})
}

/// Removes a run when dropped, whether the test passed or failed, so a failed test leaves Steplet's Linux as it found
/// it.
struct RemovedAtTheEnd<'a>(&'a RunPaths);

impl Drop for RemovedAtTheEnd<'_> {
    fn drop(&mut self) {
        let _ = discard(self.0);
    }
}

/// What a run left for a test to judge.
struct Ran {
    outcome: RunOutcome,
    report: Report,
    /// Root's look from outside, once the run had ended and before it was removed.
    look: String,
    work: String,
}

/// One run as Steplet makes it: the gate decides with the run's checker, and the tripwire watcher follows the run and
/// stops it at the first tripwire. `steps` gives the scripted calls, from the working copy's path; `ready` runs once
/// `prepare` has made the run, before the launch; `look` is root's look from outside once the run has ended.
fn run_as_steplet(
    run_id: &str,
    port: u16,
    req: &RunRequest,
    steps: &dyn Fn(&str) -> Vec<Value>,
    ready: &dyn Fn(&str, &RunPaths),
    look: &dyn Fn(&str, &RunPaths) -> String,
) -> Ran {
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, run_id).unwrap();
    let checker = support::checker(&distro, Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap());
    let deciding = checker.clone();
    let (link, gate) = serve_gate(&distro, run_id, Duration::from_secs(10), move |r| decide(&deciding, r));
    let mut script = steps(paths.work_repo());
    script.push(json!({"text": THE_END}));
    let _model = scripted_model(&distro, run_id, &json!({ "steps": script }).to_string(), port);
    let _removed = RemovedAtTheEnd(&paths);
    prepare(req, &paths, &link).unwrap();
    ready(&distro, &paths);
    let secrets = [MADE_UP_KEY, link.token()];

    let start = Instant::now();
    let handle = launch(req, &paths, &logs_dir(), Some(&format!("http://127.0.0.1:{port}")), &secrets).unwrap();
    let outcome =
        watch_run(handle, gate, &checker, paths.work_repo(), req.deadline + BACKSTOP, &Live::unattended(), &secrets);
    let took = start.elapsed();
    let look = look(&distro, &paths);
    let report = build_report(&outcome, None, Trust::Unknown, &secrets);
    print_run(&outcome, &report, took);
    Ran { outcome, report, look, work: paths.work_repo().into() }
}

/// Prints a run for the record: how it ended, every hook event, what reached the gate and the answer, and each call as
/// the report shows it.
fn print_run(outcome: &RunOutcome, report: &Report, took: Duration) {
    let shown = |t: &str, n: usize| t.replace('\n', " | ").chars().take(n).collect::<String>();
    println!(
        "--- {:.1} s; stopped by {:?}; completed {}; faults {:?}",
        took.as_secs_f64(),
        outcome.stopped_by,
        report.completed,
        report.faults
    );
    let hooks: Vec<&Event> = outcome.events.iter().filter(|e| matches!(e, Event::HookEvent { .. })).collect();
    println!("--- hook events: {hooks:?}");
    for e in &outcome.events {
        if let Event::Started { tools, .. } = e {
            println!("--- the tools Claude Code says the run has: {tools:?}");
        }
    }
    println!("--- the gate was asked:");
    for a in &outcome.requests {
        if let Asked::Decision { request, answer, .. } = a {
            println!("    {} {}: {}", request.tool_name, request.input, shown(&format!("{answer:?}"), 140));
        }
    }
    println!("--- each call, as the report shows it:");
    for c in &report.commands {
        println!(
            "    {} {:?}: {:?}, exit code {:?}, error {:?}",
            c.tool, c.command, c.check, c.exit_code, c.ended_in_error
        );
        println!("        {}", shown(&c.output, 300));
    }
    println!("--- the agent says: {:?}", report.agent_says);
}

/// The report's call to `tool` about `what`; there is exactly one.
fn the_call<'a>(report: &'a Report, tool: &str, what: &str) -> &'a ReportCommand {
    let found: Vec<&ReportCommand> = report.commands.iter().filter(|c| c.tool == tool && c.command == what).collect();
    assert_eq!(found.len(), 1, "one {tool} call about {what:?}: {:?}", report.commands);
    found[0]
}

/// The fixture for the settings test: a README and `make_settings.py`.
const SETTINGS_FIXTURE: &[(&str, &[u8])] = &[
    ("README.md", include_bytes!("support/settings_fixture/README.md")),
    ("make_settings.py", include_bytes!("support/settings_fixture/make_settings.py")),
];

const MAKE_SETTINGS: &str = "python3 make_settings.py";
const TOUCH: &str = "touch allowed.txt";

/// The word a hook in `make_settings.py`'s files writes when it runs. No file of the test holds it whole.
const HOOK_WORD: &str = "steplet-marker-hook-ran";

/// Root's look from outside at what the settings test watches: each settings file `make_settings.py` writes, each file
/// a hook would write, and every file in Steplet's Linux holding the hook's word, as `<what>: <found>` lines.
fn look_at_settings(distro: &str, paths: &RunPaths) -> String {
    let repo = paths.work_repo();
    let config = paths.run_config();
    root_sh(
        distro,
        &format!(
            "for f in {repo}/.claude/settings.json {repo}/.claude/settings.local.json {config}/settings.json \
             {repo}/hook-ran-project.txt {repo}/hook-ran-local.txt {repo}/hook-ran-user.txt; do
    if test -e $f; then echo \"$f: there\"; else echo \"$f: missing\"; fi
done
grep -rlsF -- {HOOK_WORD} /work /var/lib/steplet /tmp /var/tmp /home | sed 's/^/holds the word: /'
true"
        ),
    )
}

/// S4: agent settings files written into the working copy during the run take no effect. A declared command runs
/// `make_settings.py`, which tries to write `.claude/settings.json` and `.claude/settings.local.json` into the working
/// copy and the settings file in the run's own config folder (the one Claude Code reads for its user), each holding
/// one hook that would write a file holding a made-up word, and waits 5 seconds before it ends; then one more call is
/// made, before which each hook would run. Claude Code does load a settings file that changes during a run, a few
/// seconds later (seen with the fence switched off on purpose in this task), hence the wait. Pass: the fence refuses
/// all three writes (as the program says), no hook event, no file a hook writes and no made-up word anywhere. The
/// refusals are checked on their own, so the test does not hang on how soon Claude Code would load a file. The
/// control: the same program, run by the steplet account outside the fence before a run's launch, writes all three
/// files; in that run, without the watcher, the hook in the user settings file runs (a hook event and its file holding
/// the word, so both looks see a hook that runs), while the files in the working copy, there from the start, still
/// take no effect.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn settings_written_into_the_copy_during_the_run_take_no_effect() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let _linux = support::hold_linux();
    let distro = support::steplet_linux();
    let req = request(&[MAKE_SETTINGS, TOUCH]);
    let plant = |distro: &str, paths: &RunPaths| {
        let commit = support::plant(distro, paths, SETTINGS_FIXTURE, "", "Test data: a README and a small program");
        println!("--- the working copy, committed: {commit}");
    };

    // The control.
    {
        let paths = RunPaths::new(&distro, "p5t5k").unwrap();
        let checker = support::checker(&distro, Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap());
        let (link, gate) = serve_gate(&distro, "p5t5k", Duration::from_secs(10), move |r| decide(&checker, r));
        let script = json!({"steps": [bash(TOUCH), {"text": THE_END}]});
        let _model = scripted_model(&distro, "p5t5k", &script.to_string(), 47991);
        let _removed = RemovedAtTheEnd(&paths);
        prepare(&req, &paths, &link).unwrap();
        plant(&distro, &paths);
        let wrote = root_sh(&distro, &format!("cd {}\n{AS_STEPLET} python3 -I make_settings.py", paths.work_repo()));
        println!("--- the control: make_settings.py, run outside the fence before the launch, says:\n{wrote}");
        assert_eq!(
            wrote.lines().filter(|l| l.starts_with("wrote ")).count(),
            3,
            "outside the fence, all three are written"
        );
        let handle =
            launch(&req, &paths, &logs_dir(), Some("http://127.0.0.1:47991"), &[MADE_UP_KEY, link.token()]).unwrap();
        let events = every_event(&handle, Duration::from_secs(150));
        let code = handle.wait().unwrap();
        drop(gate);
        let look = look_at_settings(&distro, &paths);
        let hooks: Vec<&Event> = events.iter().filter(|e| matches!(e, Event::HookEvent { .. })).collect();
        println!("--- the control: exit code {code:?}; hook events: {hooks:?}\n--- root's look:\n{look}");
        assert!(!hooks.is_empty(), "the hook in the user settings file sent hook events");
        let user_hook = format!("{}/hook-ran-user.txt: there", paths.work_repo());
        assert!(look.lines().any(|l| l == user_hook), "the hook in the user settings file ran: {look}");
        assert!(look.lines().any(|l| l.starts_with("holds the word: ")), "the look found the word it wrote: {look}");
        for which in ["project", "local"] {
            let missing = format!("{}/hook-ran-{which}.txt: missing", paths.work_repo());
            assert!(
                look.lines().any(|l| l == missing),
                "the {which} settings file in the working copy took no effect: {look}"
            );
        }
    }

    // The test: the same program, run by a declared command during the run, inside the fence.
    let ran =
        run_as_steplet("p5t5a", 47990, &req, &|_| vec![bash(MAKE_SETTINGS), bash(TOUCH)], &plant, &look_at_settings);
    println!("--- root's look from outside, after the run:\n{}", ran.look);
    let made = the_call(&ran.report, "Bash", MAKE_SETTINGS);
    println!("--- what make_settings.py said inside the fence (which writes the fence refused):\n{}", made.output);
    assert_eq!((&made.check, made.exit_code), (&Check::Allowed, Some(0)), "the declared command ran: {made:?}");
    let user_settings = format!("{}/settings.json", RunPaths::new(&distro, "p5t5a").unwrap().run_config());
    let refused: Vec<&str> = made
        .output
        .lines()
        .filter_map(|l| l.strip_prefix("could not write "))
        .map(|l| l.split(": ").next().unwrap_or(l))
        .collect();
    assert_eq!(
        refused,
        [".claude/settings.json", ".claude/settings.local.json", user_settings.as_str()],
        "the fence refused every write: {}",
        made.output
    );
    assert!(!made.output.lines().any(|l| l.starts_with("wrote ")), "nothing written: {}", made.output);
    let next = the_call(&ran.report, "Bash", TOUCH);
    assert_eq!((&next.check, next.exit_code), (&Check::Allowed, Some(0)), "the call after it ran: {next:?}");

    let hooks: Vec<&Event> = ran.outcome.events.iter().filter(|e| matches!(e, Event::HookEvent { .. })).collect();
    assert_eq!(hooks, Vec::<&Event>::new(), "no hook event");
    for which in ["project", "local", "user"] {
        let missing = format!("{}/hook-ran-{which}.txt: missing", ran.work);
        assert!(ran.look.lines().any(|l| l == missing), "no hook from the {which} settings file ran: {}", ran.look);
    }
    assert!(!ran.look.contains("holds the word: "), "no file holds the hook's word: {}", ran.look);
    assert_eq!(ran.outcome.stopped_by, None);
    assert!(ran.report.completed, "the run ended on its own");
    assert_eq!(ran.report.agent_says, THE_END);
}

const CLAUDE_CLI: &str = "/opt/steplet/claude-cli";

/// The declared command that tries to add a file next to Claude Code, and the file the agent's Write tool tries to add.
const ADD_BY_COMMAND: &str = "touch /opt/steplet/claude-cli/added-by-the-task.txt";
const ADDED_BY_WRITE: &str = "/opt/steplet/claude-cli/added-by-the-agent.txt";

/// Every entry below `folder`, as root sees it from outside: kind, mode, owner, path and a link's target, then the
/// SHA-256 of every file. Sorted, so the same folder gives the same text.
fn fingerprints(distro: &str, folder: &str) -> String {
    root_sh(
        distro,
        &format!(
            "set -e -o pipefail
cd {folder}
find . -printf '%y %m %u:%g %p -> %l\\n' | LC_ALL=C sort
find . -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum"
        ),
    )
}

/// The SHA-256 of `text`, for the record: a fingerprint listing is too long to print.
fn digest(distro: &str, text: &str) -> String {
    let out = steplet_run::wsl::run(distro, "root", &["/usr/bin/sha256sum"], text.as_bytes()).unwrap();
    out.stdout.split_whitespace().next().unwrap_or_default().to_owned()
}

/// The control's folder, removed when the test ends, whether it passed or failed.
const FINGERPRINT_CONTROL: &str = "/tmp/steplet-fingerprint-control";

struct ControlFolder<'a>(&'a str);

impl Drop for ControlFolder<'_> {
    fn drop(&mut self) {
        let _ = steplet_run::wsl::run(self.0, "root", &["/usr/bin/rm", "-rf", "--", FINGERPRINT_CONTROL], b"");
    }
}

/// `claude --version` in Steplet's Linux, as Steplet's setup checks it, run by the steplet account with a home that
/// does not exist, so it can leave nothing behind.
fn claude_version(distro: &str) -> String {
    root_sh(distro, &format!("{AS_STEPLET} DISABLE_AUTOUPDATER=1 {CLAUDE_CLI}/bin/claude --version"))
}

/// S5: the pinned Claude Code is unchanged after a run in which a declared command and the agent's Write tool each
/// try to add a file next to it. Pass: both refused (the command fails inside the fence; the gate refuses the write),
/// neither file is there, the fingerprints of every file under `/opt/steplet/claude-cli` are the same before and after,
/// and `claude --version` still says the pinned version. The control: the same fingerprints, taken of a folder of the
/// test's own, change when a file is added there and when a file's bytes change.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn the_pinned_claude_code_is_unchanged_after_a_run() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let _linux = support::hold_linux();
    let distro = support::steplet_linux();

    let _control = ControlFolder(&distro);
    root_sh(&distro, &format!("mkdir -p {FINGERPRINT_CONTROL}/bin\necho one > {FINGERPRINT_CONTROL}/bin/one"));
    let first = fingerprints(&distro, FINGERPRINT_CONTROL);
    root_sh(&distro, &format!("echo two > {FINGERPRINT_CONTROL}/added.txt"));
    let added = fingerprints(&distro, FINGERPRINT_CONTROL);
    root_sh(&distro, &format!("echo One > {FINGERPRINT_CONTROL}/bin/one"));
    let changed = fingerprints(&distro, FINGERPRINT_CONTROL);
    println!("--- the control: a file added, then a file's bytes changed:\n{first}\n--\n{added}\n--\n{changed}");
    assert!(first != added && added != changed, "the fingerprints see a file added and a file changed");

    let before = fingerprints(&distro, CLAUDE_CLI);
    let version_before = claude_version(&distro);
    println!(
        "--- before the run: {} entries under {CLAUDE_CLI}, listing SHA-256 {}; {version_before}",
        before.lines().count(),
        digest(&distro, &before)
    );
    assert!(version_before.contains(setup::CLAUDE_VERSION), "{version_before}");

    let req = request(&[ADD_BY_COMMAND]);
    let write = json!({"tool": "Write", "input": {"file_path": ADDED_BY_WRITE, "content": "made-up test text\n"}});
    let ran = run_as_steplet(
        "p5t5c",
        47992,
        &req,
        &|_| vec![bash(ADD_BY_COMMAND), write.clone()],
        &|_, _| {},
        &|distro, _| fingerprints(distro, CLAUDE_CLI),
    );
    let after = ran.look;
    let version_after = claude_version(&distro);
    println!(
        "--- after the run: {} entries under {CLAUDE_CLI}, listing SHA-256 {}; {version_after}",
        after.lines().count(),
        digest(&distro, &after)
    );

    let command = the_call(&ran.report, "Bash", ADD_BY_COMMAND);
    assert_eq!(command.check, Check::Allowed, "the file declares the command, so the checker allowed it: {command:?}");
    assert!(command.exit_code.is_some_and(|c| c != 0), "it failed inside the fence: {command:?}");
    let written = the_call(&ran.report, "Write", ADDED_BY_WRITE);
    assert_eq!((&written.check, written.ended_in_error), (&Check::Refused, Some(true)), "{written:?}");
    let refusal = ran.outcome.requests.iter().find_map(|a| match a {
        Asked::Decision { request, answer, .. } if request.tool_name == "Write" => Some(answer),
        _ => None,
    });
    assert!(
        matches!(refusal, Some(GateAnswer::Deny { message }) if message.contains("only inside the working copy")),
        "the gate refused the write: {refusal:?}"
    );
    assert_eq!(after, before, "every file under {CLAUDE_CLI} is the same after the run");
    assert!(version_after.contains(setup::CLAUDE_VERSION), "{version_after}");
    assert_eq!(ran.outcome.stopped_by, None);
    assert!(ran.report.completed, "the run ended on its own");
}

/// The tool the next test's scripted model asks for: not one of the seven the run offers (H5).
const NOT_OFFERED: &str = "NotebookEdit";

/// The notebook it asks to add a cell to, put in the working copy before the run.
const NOTEBOOK: &str = "made-up-notes.ipynb";
const NOTEBOOK_TEXT: &str = include_str!("support/settings_fixture/made-up-notes.ipynb");

/// G13: a tool the run does not offer never runs. The scripted model reads a notebook in the working copy, then asks
/// for `NotebookEdit`, to add a cell to it. Pass: it never runs (the notebook's bytes are the same after the run, and
/// nothing from it came back without an error), no gate request names it, Claude Code's own list of the run's tools
/// does not hold it, and the report shows the call as not run (its first run said "ran without asking Steplet", fixed
/// in this task). The control: a declared command just before it makes a file in the working copy, which reaches the
/// gate, comes back without an error, and the same look sees.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn a_tool_the_run_does_not_offer_never_runs() {
    let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    let _linux = support::hold_linux();
    let distro = support::steplet_linux();
    let req = request(&[TOUCH]);
    let ran = run_as_steplet(
        "p5t5d",
        47993,
        &req,
        &|work| {
            let notebook = format!("{work}/{NOTEBOOK}");
            let cell = json!({"notebook_path": notebook, "new_source": "print('added by a tool not offered')",
                              "cell_type": "code", "edit_mode": "insert"});
            vec![
                bash(TOUCH),
                json!({"tool": "Read", "input": {"file_path": notebook}}),
                json!({"tool": NOT_OFFERED, "input": cell}),
            ]
        },
        &|distro, paths| {
            let commit =
                support::plant(distro, paths, &[(NOTEBOOK, NOTEBOOK_TEXT.as_bytes())], "", "Test data: a notebook");
            println!("--- the working copy, committed: {commit}");
        },
        &|distro, paths| {
            root_sh(distro, &format!("cd {} && ls -A | LC_ALL=C sort && sha256sum {NOTEBOOK}", paths.work_repo()))
        },
    );
    println!("--- root's look at the working copy, after the run:\n{}", ran.look);

    // The control.
    let touched = the_call(&ran.report, "Bash", TOUCH);
    assert_eq!((&touched.check, touched.ended_in_error), (&Check::Allowed, Some(false)), "{touched:?}");
    assert!(ran.look.lines().any(|l| l == "allowed.txt"), "the look sees the file the control made: {}", ran.look);

    let ids: Vec<&String> = ran
        .outcome
        .events
        .iter()
        .filter_map(|e| match e {
            Event::ToolUse { id, name, .. } if name == NOT_OFFERED => Some(id),
            _ => None,
        })
        .collect();
    assert_eq!(ids.len(), 1, "the scripted model asked for {NOT_OFFERED} once");
    let planted = format!("{}  {NOTEBOOK}", digest(&distro, NOTEBOOK_TEXT));
    assert!(ran.look.lines().any(|l| l == planted), "the notebook is as it was put there: {planted}\n{}", ran.look);
    let ran_ok =
        ran.outcome.events.iter().any(|e| matches!(e, Event::ToolResult { id, is_error: false, .. } if id == ids[0]));
    assert!(!ran_ok, "nothing from {NOT_OFFERED} came back without an error");
    let named =
        ran.outcome.requests.iter().map(|a| format!("{a:?}")).any(|a| a.contains(NOT_OFFERED) || a.contains(ids[0]));
    assert!(!named, "no gate request names {NOT_OFFERED} or its call");
    let offered: Vec<&Vec<String>> = ran
        .outcome
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Started { tools, .. } => Some(tools),
            _ => None,
        })
        .collect();
    assert!(!offered.is_empty() && offered.iter().all(|t| !t.iter().any(|t| t == NOT_OFFERED)), "{offered:?}");

    // The report shows the call as not run, not as one that ran without asking Steplet; the run still stops, as for
    // any call that skipped the gate.
    let call = the_call(&ran.report, NOT_OFFERED, "");
    assert_eq!((&call.check, call.ended_in_error), (&Check::NotOffered, Some(true)), "{call:?}");
    assert_eq!(ran.outcome.stopped_by, Some(Tripwire::NotOffered(NOT_OFFERED.into())));
}
