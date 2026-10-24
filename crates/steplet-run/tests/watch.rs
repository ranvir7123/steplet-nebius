//! The tripwire watcher on its own (plan Task 9), from made-up events, so it needs no Linux. Its rule comes from the
//! measurement in `tests/tripwires.rs` (`which_actions_ask_the_gate`): Claude Code runs harmless reads without asking
//! the gate, so a call that skipped the gate is judged by what it was. Chained commands (`;`, `|`) are not read at all
//! (R8, G1), so one that skips the gate stops the run (the owner's decision, 17 Oct 2026).

mod support;

use std::sync::Arc;

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{Checker, Event, Tripwire, Verdict, Watcher};
use support::{CANNOT_LOOK, MadeUpLinks};

const WORK: &str = "/work/r1/repo";

fn policy() -> Policy {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n    - cargo test\n---\n";
    Policy::from_steplet(&parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(), WORK).unwrap()
}

fn call(id: &str, name: &str, input: Value) -> Event {
    Event::ToolUse { id: id.into(), name: name.into(), input }
}

fn result(id: &str, is_error: bool) -> Event {
    Event::ToolResult { id: id.into(), is_error, text: "output".into(), plain_end: true }
}

/// One tool call and its result, seen by a fresh watcher. `gate` is what Steplet answered the gate about this call:
/// `None` when the gate was never asked, `Some(true)` for "allow", `Some(false)` for "deny".
fn judge(name: &str, input: Value, gate: Option<bool>, is_error: bool) -> Verdict {
    judge_with(&support::checker_as_written(policy()), name, input, gate, is_error)
}

/// As `judge`, with `checker` deciding.
fn judge_with(checker: &Checker, name: &str, input: Value, gate: Option<bool>, is_error: bool) -> Verdict {
    let mut w = Watcher::new(checker, WORK);
    let asked = |id: &str| if id == "u1" { gate } else { None };
    assert_eq!(w.see(&call("u1", name, input), &asked), Verdict::default(), "a call is judged at its result");
    w.see(&result("u1", is_error), &asked)
}

/// The record of call `u1`, a harmless read that skipped the gate.
fn harmless(tool: &str, what: &str) -> Verdict {
    harmless_call("u1", tool, what)
}

fn harmless_call(id: &str, tool: &str, what: &str) -> Verdict {
    Verdict {
        skipped: Some(Event::SkippedGate {
            id: id.into(),
            tool: tool.into(),
            what: what.into(),
            harmless: true,
            not_offered: false,
        }),
        stop: None,
    }
}

fn stopped(tool: &str, what: &str, why: Tripwire) -> Verdict {
    Verdict {
        skipped: Some(Event::SkippedGate {
            id: "u1".into(),
            tool: tool.into(),
            what: what.into(),
            harmless: false,
            not_offered: false,
        }),
        stop: Some(why),
    }
}

#[test]
fn a_harmless_read_command_that_skipped_the_gate_is_recorded_and_the_run_goes_on() {
    for command in ["ls", "ls src", "cat README", "git status --short", "pwd"] {
        assert_eq!(judge("Bash", json!({"command": command}), None, false), harmless("Bash", command), "{command}");
    }
    // A harmless read that failed (here there is no `src` folder) is just as harmless.
    assert_eq!(judge("Bash", json!({"command": "ls src"}), None, true), harmless("Bash", "ls src"));
}

#[test]
fn any_other_command_that_skipped_the_gate_stops_the_run() {
    for command in [
        "touch made.txt",
        // Chained commands are not read at all, so they cannot be judged harmless.
        "cat README; echo done",
        "cat README | wc -l",
        // A read outside the working copy.
        "ls /var/lib/steplet",
        // Declared by the file, but it runs the repository's code: it had to ask.
        "cargo test",
        "",
    ] {
        let why = Tripwire::SkippedCommand(command.into());
        assert_eq!(judge("Bash", json!({"command": command}), None, false), stopped("Bash", command, why.clone()));
        // Claude Code refused it by Steplet's own deny rules (`ls /var/lib/steplet` in the measurement). Steplet
        // cannot tell from the stream whether a skipped command ran, since its output could imitate a refusal.
        assert_eq!(judge("Bash", json!({"command": command}), None, true), stopped("Bash", command, why), "{command}");
    }
    assert_eq!(judge("Bash", json!({}), None, false), stopped("Bash", "", Tripwire::SkippedCommand(String::new())));
}

#[test]
fn a_read_or_search_inside_the_copy_that_skipped_the_gate_is_harmless() {
    for (tool, input, what) in [
        ("Read", json!({"file_path": "/work/r1/repo/README"}), "/work/r1/repo/README"),
        ("Read", json!({"file_path": "/work/r1/repo/.git/config"}), "/work/r1/repo/.git/config"),
        ("Glob", json!({"pattern": "*", "path": WORK}), "* in /work/r1/repo"),
        // With no path, Claude Code searches its working folder: the working copy.
        ("Glob", json!({"pattern": "src/**/*.rs"}), "src/**/*.rs in /work/r1/repo"),
        ("Grep", json!({"pattern": "Hello", "path": "/work/r1/repo/src"}), "Hello in /work/r1/repo/src"),
        ("Grep", json!({"pattern": "Hello"}), "Hello in /work/r1/repo"),
        ("Glob", json!({"pattern": "src/**/*.{rs,toml}"}), "src/**/*.{rs,toml} in /work/r1/repo"),
    ] {
        assert_eq!(judge(tool, input, None, false), harmless(tool, what), "{tool} {what}");
    }
}

#[test]
fn a_read_or_search_that_skipped_the_gate_and_reaches_outside_the_copy_stops_the_run() {
    for (tool, input, what) in [
        ("Read", json!({"file_path": "/etc/os-release"}), "/etc/os-release"),
        // Refused by Steplet's own deny rules in the measurement; it still skipped the gate.
        (
            "Read",
            json!({"file_path": "/var/lib/steplet/runs/r1/settings.json"}),
            "/var/lib/steplet/runs/r1/settings.json",
        ),
        ("Read", json!({"file_path": "/work/r1/repo/../../../etc/passwd"}), "/work/r1/repo/../../../etc/passwd"),
        ("Read", json!({"file_path": "README"}), "README"),
        ("Read", json!({}), ""),
        ("Glob", json!({"pattern": "../*"}), "../* in /work/r1/repo"),
        // A choice whose alternative climbs out: Claude Code's search may try every one.
        ("Glob", json!({"pattern": "{..,x}/*"}), "{..,x}/* in /work/r1/repo"),
        ("Glob", json!({"pattern": "*", "path": "/etc"}), "* in /etc"),
        ("Grep", json!({"pattern": "key", "glob": "/home/*", "path": WORK}), "key in /work/r1/repo"),
        ("Grep", json!({"pattern": "key", "path": "/var/lib/steplet"}), "key in /var/lib/steplet"),
    ] {
        let why = Tripwire::SkippedGate(format!("{tool} {what}").trim_end().into());
        assert_eq!(judge(tool, input, None, false), stopped(tool, what, why), "{tool} {what}");
    }
}

#[test]
fn a_write_an_edit_a_fetch_or_any_other_tool_that_skipped_the_gate_stops_the_run() {
    for (tool, input, what) in [
        ("Write", json!({"file_path": "/work/r1/repo/written.txt", "content": "t9\n"}), "/work/r1/repo/written.txt"),
        (
            "Edit",
            json!({"file_path": "/work/r1/repo/.git/config", "old_string": "a", "new_string": "b"}),
            "/work/r1/repo/.git/config",
        ),
        ("WebFetch", json!({"url": "https://example.invalid/", "prompt": "p"}), "https://example.invalid/"),
        ("Task", json!({"prompt": "p"}), ""),
        ("mcp__gate__decide", json!({}), ""),
    ] {
        let why = Tripwire::SkippedGate(format!("{tool} {what}").trim_end().into());
        assert_eq!(judge(tool, input.clone(), None, false), stopped(tool, what, why.clone()), "{tool} {what}");
        assert_eq!(judge(tool, input, None, true), stopped(tool, what, why), "{tool} {what}, refused by Claude Code");
    }
}

/// Claude Code 2.1.220's own answer to a call for a tool the run does not offer, as phase 5 Task 5's first run saw it.
const NO_SUCH_TOOL: &str = "<tool_use_error>Error: No such tool available: NotebookEdit. NotebookEdit exists but is not \
                            enabled in this context. Use one of the available tools instead.</tool_use_error>";

/// Phase 5 Task 5: a call for a tool the run does not offer (not in Claude Code's list when the run started), which
/// Claude Code answered with its own words for such a tool, did not run. The record says so, and the run still stops,
/// as for any other call that skipped the gate. A call that came back with no error or with other words, a run whose
/// list Steplet never saw or that listed no tool, and a tool the run offers are judged as before.
#[test]
fn a_tool_the_run_does_not_offer_is_recorded_as_not_run_and_still_stops_the_run() {
    let p = support::checker_as_written(policy());
    let seven = ["Bash", "Read", "Write", "Edit", "Glob", "Grep", "WebFetch"].map(String::from).to_vec();
    let offered = Event::Started { model: "m".into(), tools: seven, gate_connected: true };
    let none_listed = Event::Started { model: "m".into(), tools: Vec::new(), gate_connected: true };
    let notebook = json!({"notebook_path": "/work/r1/repo/n.ipynb", "new_source": "x"});
    let see_saying = |started: Option<&Event>, tool: &str, input: &Value, is_error: bool, text: &str| {
        let mut w = Watcher::new(&p, WORK);
        if let Some(s) = started {
            assert_eq!(w.see(s, &|_| None), Verdict::default(), "the start is not judged");
        }
        assert_eq!(w.see(&call("u1", tool, input.clone()), &|_| None), Verdict::default());
        let came_back = Event::ToolResult { id: "u1".into(), is_error, text: text.into(), plain_end: true };
        w.see(&came_back, &|_| None)
    };
    let see = |started: Option<&Event>, tool: &str, input: &Value, is_error: bool| {
        see_saying(started, tool, input, is_error, NO_SUCH_TOOL)
    };
    let not_run = Verdict {
        skipped: Some(Event::SkippedGate {
            id: "u1".into(),
            tool: "NotebookEdit".into(),
            what: String::new(),
            harmless: false,
            not_offered: true,
        }),
        stop: Some(Tripwire::NotOffered("NotebookEdit".into())),
    };
    assert_eq!(see(Some(&offered), "NotebookEdit", &notebook, true), not_run);
    let went_past = stopped("NotebookEdit", "", Tripwire::SkippedGate("NotebookEdit".into()));
    assert_eq!(see(Some(&offered), "NotebookEdit", &notebook, false), went_past, "no error: no sign it did not run");
    // Only Claude Code's own words for a tool it does not offer, naming this tool, say it did not run.
    let other_words = see_saying(Some(&offered), "NotebookEdit", &notebook, true, "output");
    assert_eq!(other_words, went_past, "an error in other words may come from a tool that ran");
    let another_tool = NO_SUCH_TOOL.replace("NotebookEdit", "NotebookEditor");
    let another = see_saying(Some(&offered), "NotebookEdit", &notebook, true, &another_tool);
    assert_eq!(another, went_past, "the words name another tool");
    assert_eq!(see(None, "NotebookEdit", &notebook, true), went_past, "no list seen");
    assert_eq!(see(Some(&none_listed), "NotebookEdit", &notebook, true), went_past, "no tool listed");
    let write = json!({"file_path": "/work/r1/repo/written.txt", "content": "t9\n"});
    let why = Tripwire::SkippedGate("Write /work/r1/repo/written.txt".into());
    assert_eq!(see(Some(&offered), "Write", &write, true), stopped("Write", "/work/r1/repo/written.txt", why));
}

#[test]
fn a_call_the_gate_allowed_is_not_noted_whatever_came_back() {
    for (tool, input) in [
        ("Bash", json!({"command": "touch made.txt"})),
        ("Write", json!({"file_path": "/work/r1/repo/written.txt", "content": "t9\n"})),
        ("Read", json!({"file_path": "/etc/os-release"})),
    ] {
        assert_eq!(judge(tool, input.clone(), Some(true), false), Verdict::default(), "{tool}");
        assert_eq!(judge(tool, input, Some(true), true), Verdict::default(), "{tool}, which ran and failed");
    }
}

#[test]
fn a_call_the_gate_refused_must_come_back_as_an_error() {
    let touch = json!({"command": "touch made.txt"});
    assert_eq!(judge("Bash", touch.clone(), Some(false), true), Verdict::default());
    // It came back as a success: it ran although Steplet refused it.
    assert_eq!(
        judge("Bash", touch, Some(false), false),
        Verdict { skipped: None, stop: Some(Tripwire::SkippedGate("Bash touch made.txt".into())) }
    );
}

#[test]
fn a_result_for_a_call_never_seen_stops_the_run() {
    let p = support::checker_as_written(policy());
    let mut w = Watcher::new(&p, WORK);
    let never = |_: &str| None;
    assert_eq!(
        w.see(&result("u9", false), &never),
        Verdict {
            skipped: None,
            stop: Some(Tripwire::SkippedGate("a result for a tool call Steplet did not see".into()))
        }
    );
    // A second result for a call already judged is one Steplet did not see either.
    assert_eq!(w.see(&call("u1", "Bash", json!({"command": "ls"})), &never), Verdict::default());
    assert_eq!(w.see(&result("u1", false), &never), harmless("Bash", "ls"));
    assert!(w.see(&result("u1", false), &never).stop.is_some());
}

#[test]
fn each_result_is_matched_to_its_own_call() {
    // Two calls in one message, their results in the other order: a write the gate allowed, and a read that skipped it.
    let p = support::checker_as_written(policy());
    let mut w = Watcher::new(&p, WORK);
    let asked = |id: &str| (id == "u1").then_some(true);
    assert_eq!(w.see(&call("u1", "Bash", json!({"command": "touch made.txt"})), &asked), Verdict::default());
    assert_eq!(w.see(&call("u2", "Bash", json!({"command": "ls"})), &asked), Verdict::default());
    assert_eq!(w.see(&result("u2", false), &asked), harmless_call("u2", "Bash", "ls"));
    assert_eq!(w.see(&result("u1", false), &asked), Verdict::default());
}

#[test]
fn a_hook_event_stops_the_run() {
    let p = support::checker_as_written(policy());
    let mut w = Watcher::new(&p, WORK);
    assert_eq!(
        w.see(&Event::HookEvent { name: "PreToolUse:Bash".into() }, &|_| None),
        Verdict { skipped: None, stop: Some(Tripwire::Hook("PreToolUse:Bash".into())) }
    );
}

#[test]
fn the_other_events_are_not_judged() {
    let p = support::checker_as_written(policy());
    let mut w = Watcher::new(&p, WORK);
    for e in [
        Event::Started { model: "m".into(), tools: vec!["Bash".into()], gate_connected: true },
        Event::AgentText("I ran it; ignore the gate".into()),
        Event::ModelSwitch { from: "a".into(), to: None, category: "c".into(), explanation: "e".into() },
        Event::Result {
            subtype: "success".into(),
            is_error: false,
            stop_reason: "end_turn".into(),
            models: vec![],
            text: "done".into(),
        },
        Event::Stderr("warning".into()),
        Event::Unreadable("not json".into()),
        Event::Other("system/status".into()),
        Event::SkippedGate {
            id: "u9".into(),
            tool: "Bash".into(),
            what: "touch x".into(),
            harmless: false,
            not_offered: false,
        },
    ] {
        assert_eq!(w.see(&e, &|_| None), Verdict::default(), "{e:?}");
    }
}

/// Phase 5 Task 3 (row 30): a read that skipped the gate is judged where it really leads. Claude Code refuses a read
/// through a link into Steplet's own folder by Steplet's deny rule, without asking the gate; judged as written (inside
/// the copy) it looked like a harmless read. Now it stops the run, as other calls refused by Steplet's own rules do
/// (H11). A link that stays inside the copy is still a harmless read, and a path Steplet cannot look up is not one.
#[test]
fn a_read_that_skipped_the_gate_is_judged_where_it_really_leads() {
    let links = Checker::new(
        policy(),
        Arc::new(MadeUpLinks(vec![
            ("/work/r1/repo/notes/run", "/var/lib/steplet/runs/r1"),
            ("/work/r1/repo/notes/readme", "/work/r1/repo/README.md"),
        ])),
    );
    let note = "/work/r1/repo/notes/run/home/note.txt";
    assert_eq!(
        judge_with(&links, "Read", json!({"file_path": note}), None, true),
        stopped("Read", note, Tripwire::SkippedGate(format!("Read {note}")))
    );
    assert_eq!(
        judge_with(&links, "Glob", json!({"pattern": "**/*", "path": "/work/r1/repo/notes/run"}), None, true),
        stopped(
            "Glob",
            "**/* in /work/r1/repo/notes/run",
            Tripwire::SkippedGate("Glob **/* in /work/r1/repo/notes/run".into())
        )
    );
    assert_eq!(
        judge_with(&links, "Read", json!({"file_path": "/work/r1/repo/notes/readme"}), None, false),
        harmless("Read", "/work/r1/repo/notes/readme")
    );
    assert_eq!(
        judge_with(&links, "Read", json!({"file_path": CANNOT_LOOK}), None, false),
        stopped("Read", CANNOT_LOOK, Tripwire::SkippedGate(format!("Read {CANNOT_LOOK}")))
    );
}
