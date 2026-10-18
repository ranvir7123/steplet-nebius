//! The run's report (plan Task 10), built from made-up recorded events, so it needs no Linux. The receiver reads the
//! report after running a stranger's task, so it must be true and safe to read even when the task was hostile:
//! whether the run finished comes from Steplet's own records, never from what the agent or a program says (F35); every
//! model and refusal is listed (D7, E1); the agent's words appear only as "the agent says" (F47); invisible and
//! control characters are removed (T5b); and the receiver's key and the run's token are hidden in every field (F48).
//!
//! The test data imitates what a hostile task could print. It is short, inert and made up: the key is
//! `support::MADE_UP_KEY`, which is not a key, and the token below is not a real run's.
//!
//! The test marked `#[ignore]` needs Steplet's Linux and the network:
//! `cargo test -p steplet-run --offline --test report -- --ignored --nocapture --test-threads=1`.

mod support;

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::display::is_invisible;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    ApiKey, Asked, Check, Client, Event, GateAnswer, GateRequest, Refusal, Report, ReportCommand, RunOutcome, RunPaths,
    RunRequest, RunSetup, Tripwire, Trust, build_report, changed_files, parse_line, raw_log_line, read_trust, run,
};
use support::MADE_UP_KEY;

/// A made-up run token (test data).
const MADE_UP_TOKEN: &str = "t10madeuptoken0123456789abcdef";
const HIDDEN: &str = "[hidden by Steplet]";
const MODEL: &str = "claude-opus-5[1m]";

fn secrets() -> [&'static str; 2] {
    [MADE_UP_KEY, MADE_UP_TOKEN]
}

/// A run that ended on its own, with `events` and nothing else on record.
fn outcome(events: Vec<Event>) -> RunOutcome {
    RunOutcome {
        events,
        requests: Vec::new(),
        stopped_by: None,
        stopped_at: None,
        exit_code: Some(0),
        faults: Vec::new(),
        stopped_by_you: false,
    }
}

fn started() -> Event {
    Event::Started { model: MODEL.into(), tools: vec!["Bash".into()], gate_connected: true }
}

fn finished(text: &str) -> Event {
    result("success", false, "end_turn", text)
}

fn result(subtype: &str, is_error: bool, stop_reason: &str, text: &str) -> Event {
    Event::Result {
        subtype: subtype.into(),
        is_error,
        stop_reason: stop_reason.into(),
        models: vec![MODEL.into()],
        text: text.into(),
    }
}

fn bash(id: &str, command: &str) -> Event {
    Event::ToolUse { id: id.into(), name: "Bash".into(), input: json!({"command": command, "description": "d"}) }
}

/// A result whose record in Claude Code's stream shows a plain end (see `Event::ToolResult`).
fn tool_result(id: &str, is_error: bool, text: &str) -> Event {
    Event::ToolResult { id: id.into(), is_error, text: text.into(), plain_end: true }
}

fn skipped(id: &str, command: &str, harmless: bool) -> Event {
    Event::SkippedGate { id: id.into(), tool: "Bash".into(), what: command.into(), harmless }
}

/// Steplet's answer to the gate about call `id`, as the gate service records it.
fn decision(id: &str, tool: &str, input: Value, answer: GateAnswer) -> Asked {
    let client = Client {
        pid: 1,
        program: "steplet-gate.exe".into(),
        user: "S-1-5-21-0".into(),
        logon_session: 1,
        integrity: 8192,
        restricted: false,
        app_container: false,
        identify_only: true,
    };
    let request =
        GateRequest { run_token: MADE_UP_TOKEN.into(), tool_name: tool.into(), input, tool_use_id: id.into() };
    Asked::Decision { client, request, answer, took: Duration::from_millis(5), delivered: true }
}

fn allow(id: &str, command: &str) -> Asked {
    let input = json!({"command": command});
    decision(id, "Bash", input.clone(), GateAnswer::Allow { updated_input: input })
}

fn deny(id: &str, command: &str, message: &str) -> Asked {
    decision(id, "Bash", json!({"command": command}), GateAnswer::Deny { message: message.into() })
}

fn report(o: &RunOutcome) -> Report {
    build_report(o, Some(&[]), Trust::NotRecorded, &secrets())
}

/// Every piece of text in the report except the tool names, so a test can check all of them at once.
fn every_text(r: &Report) -> Vec<String> {
    let mut all = vec![r.agent_says.clone()];
    all.extend(r.commands.iter().flat_map(|c| [c.command.clone(), c.output.clone()]));
    all.extend(r.refused.iter().cloned());
    all.extend(r.models.iter().cloned());
    all.extend(r.refusals.iter().flat_map(|f| {
        [f.model.clone(), f.switched_to.clone().unwrap_or_default(), f.category.clone(), f.explanation.clone()]
    }));
    all.extend(r.changed_files.iter().flatten().cloned());
    all.extend(r.faults.iter().cloned());
    all.extend(r.stopped_by.iter().map(|t| format!("{t:?}")));
    all
}

#[test]
fn a_run_that_ended_on_its_own_is_completed_with_the_command_s_real_exit_code() {
    let mut o = outcome(vec![
        started(),
        bash("u1", "python3 -m unittest discover"),
        tool_result("u1", true, "Exit code 5\n\nRan 0 tests in 0.000s\n\nNO TESTS RAN"),
        Event::AgentText("No tests ran.".into()),
        finished("No tests ran."),
    ]);
    o.requests = vec![allow("u1", "python3 -m unittest discover")];
    let r = report(&o);
    assert!(r.completed, "{r:?}");
    assert_eq!(r.stopped_by, None);
    assert_eq!(
        r.commands,
        vec![ReportCommand {
            tool: "Bash".into(),
            command: "python3 -m unittest discover".into(),
            check: Check::Allowed,
            exit_code: Some(5),
            output: "Exit code 5\n\nRan 0 tests in 0.000s\n\nNO TESTS RAN".into(),
            after_stop: false,
        }]
    );
    assert_eq!(r.agent_says, "No tests ran.");
    assert_eq!(r.models, vec![MODEL.to_owned()]);
    assert_eq!(r.changed_files, Some(Vec::new()));
    assert_eq!(r.trust, Trust::NotRecorded);
}

/// F35, E1: a refused run says `success`, so the outcome is judged from `is_error` and `stop_reason` as well.
#[test]
fn completed_is_false_when_the_result_reports_an_error_whatever_its_subtype_says() {
    for (subtype, is_error, stop_reason, completed) in [
        ("success", false, "end_turn", true),
        ("success", true, "end_turn", false),
        ("success", false, "refusal", false),
        ("success", false, "max_tokens", false),
        ("error_during_execution", true, "", false),
        ("error_during_execution", false, "end_turn", false),
        ("error_max_turns", true, "end_turn", false),
    ] {
        let r = report(&outcome(vec![started(), result(subtype, is_error, stop_reason, "Done.")]));
        assert_eq!(r.completed, completed, "{subtype} is_error={is_error} stop_reason={stop_reason}");
    }
}

#[test]
fn completed_is_false_when_a_tripwire_fired_even_if_the_result_says_success() {
    let mut o = outcome(vec![started(), Event::HookEvent { name: "PreToolUse:Bash".into() }, finished("Done.")]);
    o.stopped_by = Some(Tripwire::Hook("PreToolUse:Bash".into()));
    o.stopped_at = Some(2);
    o.exit_code = Some(124);
    let r = report(&o);
    assert!(!r.completed);
    assert_eq!(r.stopped_by, Some(Tripwire::Hook("PreToolUse:Bash".into())));
}

#[test]
fn completed_needs_exactly_one_result_exit_code_0_no_fault_and_no_stop() {
    assert!(!report(&outcome(vec![started()])).completed, "no result");
    assert!(!report(&outcome(vec![started(), finished("a"), finished("b")])).completed, "two results");
    let ok = outcome(vec![started(), finished("Done.")]);
    assert!(report(&ok).completed);
    for (code, why) in [(Some(1), "exit code 1"), (Some(124), "the time limit"), (None, "ended by a signal")] {
        assert!(!report(&RunOutcome { exit_code: code, ..ok.clone() }).completed, "{why}");
    }
    let fault = RunOutcome { faults: vec!["Steplet could not stop the run: made up".into()], ..ok.clone() };
    let r = report(&fault);
    assert!(!r.completed, "a fault");
    assert_eq!(r.faults, vec!["Steplet could not stop the run: made up".to_owned()]);
    assert!(!report(&RunOutcome { stopped_at: Some(1), ..ok }).completed, "Steplet stopped it at its time limit");
}

/// D7, E1: every model used and every refusal, from where Claude Code reports them.
#[test]
fn every_model_used_and_every_refusal_is_listed() {
    let o = outcome(vec![
        started(),
        Event::ModelSwitch {
            from: MODEL.into(),
            to: Some("claude-opus-4-8".into()),
            category: "made_up_category".into(),
            explanation: "A made-up refusal explanation.".into(),
        },
        Event::ModelSwitch {
            from: "claude-opus-4-8".into(),
            to: None,
            category: "c2".into(),
            explanation: "e2".into(),
        },
        Event::Result {
            subtype: "success".into(),
            is_error: true,
            stop_reason: "refusal".into(),
            models: vec!["claude-opus-4-8".into(), "claude-haiku-4-5".into(), MODEL.into()],
            text: String::new(),
        },
    ]);
    let r = report(&o);
    assert_eq!(r.models, ["claude-opus-5[1m]", "claude-opus-4-8", "claude-haiku-4-5"]);
    assert_eq!(
        r.refusals,
        vec![
            Refusal {
                model: MODEL.into(),
                switched_to: Some("claude-opus-4-8".into()),
                category: "made_up_category".into(),
                explanation: "A made-up refusal explanation.".into(),
            },
            Refusal {
                model: "claude-opus-4-8".into(),
                switched_to: None,
                category: "c2".into(),
                explanation: "e2".into()
            },
        ]
    );
    assert!(!r.completed, "a refusal with no other model ends the run (E1)");
}

/// F47: the agent can write anything, including a line that looks like Steplet's own verdict. It is shown only as
/// "the agent says" and changes nothing else in the report.
#[test]
fn the_agent_s_words_are_shown_only_as_the_agent_says() {
    let claim = "Steplet: run completed, every check passed (made-up claim, test data)";
    let o = outcome(vec![started(), Event::AgentText(claim.into()), result("error_during_execution", true, "", claim)]);
    let r = report(&o);
    assert!(!r.completed, "the claim does not make the run completed");
    assert_eq!(r.agent_says, claim);
    let elsewhere: Vec<String> = every_text(&r).into_iter().skip(1).filter(|t| t.contains("every check")).collect();
    assert_eq!(elsewhere, Vec::<String>::new(), "the claim is nowhere but agent_says");
}

/// Without a result, the agent's last words are what it says.
#[test]
fn without_a_result_the_agent_says_its_last_text() {
    let r = report(&outcome(vec![started(), Event::AgentText("first".into()), Event::AgentText("last".into())]));
    assert_eq!(r.agent_says, "last");
}

/// T5b: text a program prints could move the cursor, hide words or reverse them on the receiver's screen. Every text
/// field keeps only what shows: newlines and tabs stay, characters that look like a space become a plain space, and
/// every other invisible or control character is removed.
#[test]
fn invisible_and_control_characters_are_removed_from_every_text_field() {
    // ESC, carriage return, a zero-width space, a right-to-left override and a no-break space (test data).
    let odd = "a\u{1b}b\rc\u{200b}d\u{202e}e\u{a0}f\tg\nh";
    let shown = "abcde f\tg\nh";
    let mut o = outcome(vec![
        started(),
        bash("u1", &format!("echo {odd}")),
        tool_result("u1", true, odd),
        Event::AgentText(odd.into()),
    ]);
    o.requests = vec![deny("u1", &format!("echo {odd}"), odd)];
    o.faults = vec![odd.into()];
    o.stopped_by = Some(Tripwire::SkippedCommand(odd.into()));
    o.stopped_at = Some(3);
    let r = build_report(&o, Some(&[format!("?? {odd}")]), Trust::NotRecorded, &secrets());
    for text in every_text(&r) {
        assert!(!text.chars().any(is_invisible), "an invisible character is left in {text:?}");
    }
    assert_eq!(r.agent_says, shown);
    assert_eq!(r.commands[0].command, format!("echo {shown}"));
    assert_eq!(r.commands[0].output, shown);
    assert_eq!(r.faults, vec![shown.to_owned()]);
    assert_eq!(r.changed_files, Some(vec![format!("?? {shown}")]));
    assert_eq!(r.stopped_by, Some(Tripwire::SkippedCommand(shown.into())));
    assert!(r.refused[0].ends_with(shown), "{:?}", r.refused);
}

/// F48: if a program printed the receiver's key or the run's token, neither may reach the report, in any field. The
/// key below is the made-up one; one copy has a zero-width space inside it, which must not let it through.
#[test]
fn the_key_and_the_run_s_token_are_hidden_in_every_field() {
    let split_key = format!("{}\u{200b}{}", &MADE_UP_KEY[..10], &MADE_UP_KEY[10..]);
    let both = format!("key {MADE_UP_KEY} token {MADE_UP_TOKEN} split {split_key}");
    let mut o = outcome(vec![
        Event::Started { model: both.clone(), tools: Vec::new(), gate_connected: true },
        bash("u1", &format!("echo {both}")),
        tool_result("u1", false, &both),
        bash("u2", &format!("printf {both}")),
        tool_result("u2", true, &both),
        Event::AgentText(both.clone()),
        Event::ModelSwitch {
            from: both.clone(),
            to: Some(both.clone()),
            category: both.clone(),
            explanation: both.clone(),
        },
        Event::Result {
            subtype: "success".into(),
            is_error: false,
            stop_reason: "end_turn".into(),
            models: vec![both.clone()],
            text: both.clone(),
        },
    ]);
    o.requests = vec![
        allow("u1", &format!("echo {both}")),
        deny("u2", &format!("printf {both}"), &both),
        Asked::Refused { client: None, reason: both.clone() },
    ];
    o.faults = vec![both.clone()];
    o.stopped_by = Some(Tripwire::Hook(both.clone()));
    let r = build_report(&o, Some(&[format!("?? {both}")]), Trust::NotRecorded, &secrets());
    let whole = format!("{r:?}");
    assert!(!whole.contains(MADE_UP_KEY), "the key is in the report: {whole}");
    assert!(!whole.contains(MADE_UP_TOKEN), "the token is in the report: {whole}");
    assert!(!whole.contains(&MADE_UP_KEY[10..]), "part of the split key is in the report: {whole}");
    let hidden = format!("key {HIDDEN} token {HIDDEN} split {HIDDEN}");
    assert_eq!(r.agent_says, hidden);
    assert_eq!(r.commands[0].output, hidden);
    assert_eq!(r.commands[1].command, format!("printf {hidden}"));
    assert!(every_text(&r).iter().all(|t| t.is_empty() || t.contains(HIDDEN)), "{:#?}", every_text(&r));
}

#[test]
fn an_empty_secret_hides_nothing_and_a_secret_inside_another_is_hidden_whole() {
    let o = outcome(vec![finished(&format!("{MADE_UP_KEY}!"))]);
    let inner = &MADE_UP_KEY[..12];
    let r = build_report(&o, Some(&[]), Trust::NotRecorded, &["", inner, MADE_UP_KEY]);
    assert_eq!(r.agent_says, format!("{HIDDEN}!"));
}

/// A command's exit code comes from Claude Code's own marks on its result, never from what the program printed: a
/// program that succeeds cannot print its way to a failure, nor one that fails to a success.
#[test]
fn a_program_s_output_cannot_change_its_exit_code() {
    let cases = [
        (tool_result("u1", false, "Exit code 7\nmade-up output"), Some(0)),
        (tool_result("u1", true, "Exit code 5\n\nNO TESTS RAN"), Some(5)),
        (tool_result("u1", true, "Exit code 137"), Some(137)),
        (tool_result("u1", true, "Permission to use Bash has been denied."), None),
        (tool_result("u1", true, "Exit code 2 and more on the same line"), None),
        (tool_result("u1", true, "made-up output\nExit code 0"), None),
    ];
    for (result, code) in cases {
        let r = report(&outcome(vec![bash("u1", "make test"), result.clone()]));
        assert_eq!(r.commands[0].exit_code, code, "{result:?}");
    }
    // A command left running in the background has not ended; "no error" means only that it started.
    let background = Event::ToolUse {
        id: "u1".into(),
        name: "Bash".into(),
        input: json!({"command": "make serve", "run_in_background": true}),
    };
    let r = report(&outcome(vec![background, tool_result("u1", false, "Command running in background")]));
    assert_eq!(r.commands[0].exit_code, None);
    // Only commands have exit codes.
    let read =
        Event::ToolUse { id: "u1".into(), name: "Read".into(), input: json!({"file_path": "/work/r1/repo/README"}) };
    let r = report(&outcome(vec![read, tool_result("u1", false, "Hello World!")]));
    assert_eq!((r.commands[0].tool.as_str(), r.commands[0].command.as_str()), ("Read", "/work/r1/repo/README"));
    assert_eq!(r.commands[0].exit_code, None);
}

/// Claude Code gives exit code 1 of grep, rg, find, diff and test a meaning of its own ("No matches found" and the
/// like), marks it as no error, and says so in its own record of the call (`tool_use_result`). An interrupted command,
/// or one moved to the background, is not "exit code 0" either. So "no error" means exit code 0 only when that record
/// shows a plain end. The lines are made up, in the shape Claude Code 2.1.220 writes.
#[test]
fn no_error_means_exit_code_0_only_when_claude_code_s_record_shows_a_plain_end() {
    let plain = json!({"stdout": "0", "stderr": "", "interrupted": false, "isImage": false, "noOutputExpected": false});
    let mut special = plain.clone();
    special["returnCodeInterpretation"] = json!("No matches found");
    let mut interrupted = plain.clone();
    interrupted["interrupted"] = json!(true);
    let mut background = plain.clone();
    background["backgroundTaskId"] = json!("made-up-1");
    let mut timed_out = plain.clone();
    timed_out["timedOutAfterMs"] = json!(1000);
    let block = json!({"type": "tool_result", "tool_use_id": "u1", "content": "0", "is_error": false});
    let line = |record: &Value, blocks: Vec<Value>| {
        json!({"type": "user", "message": {"role": "user", "content": blocks}, "tool_use_result": record}).to_string()
    };
    let other = json!({"type": "tool_result", "tool_use_id": "u2", "content": "x", "is_error": false});
    let cases = [
        (line(&plain, vec![block.clone()]), Some(0)),
        (line(&special, vec![block.clone()]), None),
        (line(&interrupted, vec![block.clone()]), None),
        (line(&background, vec![block.clone()]), None),
        (line(&timed_out, vec![block.clone()]), None),
        (line(&Value::Null, vec![block.clone()]), None),
        // Two results on one line: the record cannot be told apart, so neither gets an exit code from it.
        (line(&plain, vec![block.clone(), other]), None),
    ];
    for (line, code) in cases {
        let mut events = vec![bash("u1", "grep -c made-up-absent-text README")];
        events.extend(parse_line(&line));
        let r = report(&outcome(events));
        assert_eq!(r.commands[0].exit_code, code, "{line}");
    }
}

/// Steplet refused a call at the gate, but its result came back with no error: it ran anyway. The watcher stops the
/// run for it (Task 9), and the report must show that it ran, not only that it was refused, even while the run was
/// already being stopped: it is a breach of its own.
#[test]
fn a_refused_call_that_ran_anyway_is_shown_as_having_run() {
    for stopping in [false, true] {
        let mut o = outcome(vec![
            started(),
            Event::HookEvent { name: "PreToolUse:Bash".into() },
            bash("u1", "touch made.txt"),
            tool_result("u1", false, ""),
            skipped("u1", "touch made.txt", false),
        ]);
        o.requests = vec![deny("u1", "touch made.txt", "Steplet refused: the file does not declare this command")];
        o.stopped_by = Some(Tripwire::SkippedGate("Bash touch made.txt".into()));
        o.stopped_at = stopping.then_some(2);
        let r = report(&o);
        let c = &r.commands[0];
        assert_eq!((&c.check, c.after_stop), (&Check::RefusedButRan, false), "stopping: {stopping}");
        assert_eq!(
            r.refused,
            ["Bash touch made.txt: Steplet refused: the file does not declare this command (it ran anyway)"],
            "stopping: {stopping}"
        );
    }
}

/// Each call shows what Steplet did with it: allowed or refused at the gate, run without asking as a harmless read
/// (G2), or not judged because no result came. Everything the gate refused is also listed with Steplet's reason.
#[test]
fn each_call_shows_steplet_s_check_and_every_refusal_is_listed() {
    let mut o = outcome(vec![
        bash("u1", "cargo test"),
        tool_result("u1", false, "ok"),
        bash("u2", "touch made.txt"),
        tool_result("u2", true, "Steplet refused: the file does not declare this command"),
        bash("u3", "ls"),
        tool_result("u3", false, "README"),
        skipped("u3", "ls", true),
        bash("u4", "cat README"),
    ]);
    o.requests = vec![
        deny("u2", "touch made.txt", "Steplet refused: the file does not declare this command"),
        allow("u1", "cargo test"),
        Asked::Refused { client: None, reason: "the run token was wrong".into() },
    ];
    let r = report(&o);
    let checks: Vec<(&str, &Check)> = r.commands.iter().map(|c| (c.command.as_str(), &c.check)).collect();
    assert_eq!(
        checks,
        [
            ("cargo test", &Check::Allowed),
            ("touch made.txt", &Check::Refused),
            ("ls", &Check::NotAsked { harmless: true }),
            ("cat README", &Check::NotJudged),
        ]
    );
    assert_eq!(r.commands[3].output, "");
    assert_eq!(
        r.refused,
        [
            "Bash touch made.txt: Steplet refused: the file does not declare this command",
            "a connection to Steplet's gate: the run token was wrong",
        ]
    );
}

/// Task 9: while Steplet is stopping a run, calls already under way come back cut short, and new ones are refused.
/// They are marked as made during the stop, and the one tripwire stays the reason, not one breach per call.
#[test]
fn calls_cut_short_by_the_stop_are_marked_and_the_tripwire_stays_the_reason() {
    let mut o = outcome(vec![
        started(),
        bash("u1", "ls"),
        tool_result("u1", false, "README"),
        skipped("u1", "ls", true),
        bash("u2", "touch a.txt"),
        Event::HookEvent { name: "PreToolUse:Bash".into() },
        tool_result("u2", true, "made-up interrupted"),
        skipped("u2", "touch a.txt", false),
        bash("u3", "touch b.txt"),
        tool_result("u3", true, "Steplet is stopping this run"),
        bash("u4", "touch c.txt"),
    ]);
    o.requests = vec![deny("u3", "touch b.txt", "Steplet is stopping this run")];
    o.stopped_by = Some(Tripwire::Hook("PreToolUse:Bash".into()));
    o.stopped_at = Some(6);
    o.exit_code = Some(124);
    let r = report(&o);
    let marked: Vec<(&str, bool)> = r.commands.iter().map(|c| (c.command.as_str(), c.after_stop)).collect();
    assert_eq!(marked, [("ls", false), ("touch a.txt", true), ("touch b.txt", true), ("touch c.txt", true)]);
    assert_eq!(r.stopped_by, Some(Tripwire::Hook("PreToolUse:Bash".into())));
    assert!(!r.completed);
}

/// A call that skipped the checker, was no harmless read, and came back with no error ran outside the checker. Even
/// when its result came while Steplet was stopping the run (two calls at once, or one started during the stop), it is
/// a breach of its own, not something the stop cut short. One that came back with an error was cut short.
#[test]
fn a_call_that_skipped_the_checker_and_ran_during_the_stop_is_not_marked_as_cut_short() {
    let mut o = outcome(vec![
        started(),
        bash("u1", "cat a | head"),
        tool_result("u1", false, "a"),
        skipped("u1", "cat a | head", false),
        bash("u2", "cat b; ls"),
        tool_result("u2", false, "b"),
        skipped("u2", "cat b; ls", false),
        bash("u3", "touch c.txt"),
        tool_result("u3", true, "made-up interrupted"),
        skipped("u3", "touch c.txt", false),
    ]);
    o.stopped_by = Some(Tripwire::SkippedCommand("cat a | head".into()));
    o.stopped_at = Some(4);
    o.exit_code = Some(124);
    let r = report(&o);
    let marked: Vec<(&str, bool)> = r.commands.iter().map(|c| (c.command.as_str(), c.after_stop)).collect();
    assert_eq!(marked, [("cat a | head", false), ("cat b; ls", false), ("touch c.txt", true)]);
}

/// Phase 0 S6: the fence stops a blocked attempt without telling Steplet, so the report says so and never claims the
/// fence stopped a particular thing.
#[test]
fn the_report_says_that_blocked_fence_attempts_are_not_visible_to_steplet() {
    let r = report(&outcome(vec![started(), finished("Done.")]));
    assert!(
        r.notes.iter().any(|n| n.to_lowercase().contains("blocked fence attempts are not visible to steplet")),
        "{:?}",
        r.notes
    );
}

/// The changed files come from Steplet's own `git status --porcelain=v1 -z` after the run, never from the agent.
#[test]
fn changed_files_are_read_from_steplet_s_own_git_status() {
    assert_eq!(changed_files(" M README\0?? made.txt\0D  old.txt\0"), [" M README", "?? made.txt", "D  old.txt"]);
    assert_eq!(changed_files(""), Vec::<String>::new());
    assert_eq!(changed_files("?? a name with spaces\0"), ["?? a name with spaces"]);
    // When Steplet could not look, the report says so instead of "nothing changed".
    let r = build_report(&outcome(vec![finished("")]), None, Trust::NotRecorded, &secrets());
    assert_eq!(r.changed_files, None);
}

/// D3: after each run, the run's fresh Claude Code config must record no trust for any folder. A config Steplet
/// cannot read gives "unknown", never "not recorded".
#[test]
fn trust_is_read_from_the_run_s_config() {
    assert_eq!(read_trust(None), Trust::Unknown);
    assert_eq!(read_trust(Some("not json")), Trust::Unknown);
    assert_eq!(read_trust(Some(r#"{"projects": []}"#)), Trust::Unknown);
    assert_eq!(read_trust(Some(r#"{"userID": "made-up", "firstStartTime": "2026-10-17"}"#)), Trust::NotRecorded);
    assert_eq!(
        read_trust(Some(r#"{"projects": {"/work/r1/repo": {"hasTrustDialogAccepted": false}}}"#)),
        Trust::NotRecorded
    );
    assert_eq!(
        read_trust(Some(r#"{"projects": {"/elsewhere": {"hasTrustDialogAccepted": true}}}"#)),
        Trust::Recorded,
        "trust for any folder counts"
    );
    let r = build_report(&outcome(vec![finished("")]), Some(&[]), Trust::Recorded, &secrets());
    assert_eq!(r.trust, Trust::Recorded);
}

/// The scripted model's last words.
const THE_END: &str = "Task 10: the end of the script.";

/// Use case 1, end to end through `run`, with the scripted model: the repository's test command runs, and the report
/// shows its real exit code and output (Hello-World has no tests, so Python's runner says so and exits 5). A declared
/// command makes a file, which Steplet's own `git status` lists. The run's config is checked for trust before the
/// run is removed (D3).
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn use_case_1_report_shows_the_test_command_s_real_exit_code_and_no_trust() {
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "t10a").unwrap();
    let f = "---
steplet: 1
title: t
sender: s
expires: 2026-12-31
steps:
  - a
allow:
  commands:
    - python3 -m unittest discover
    - touch made.txt
---
";
    let req = RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: "https://github.com/octocat/Hello-World.git".into(),
        commit: "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d".into(),
        task_text: "Follow the script.".into(),
        max_turns: 20,
        deadline: Duration::from_secs(180),
    };
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": "python3 -m unittest discover", "description": "Run the tests"}},
        {"tool": "Bash", "input": {"command": "touch made.txt", "description": "Make a file"}},
        {"text": THE_END}]});
    let _model = support::scripted_model(&distro, "t10a", &script.to_string(), 47940);
    let setup = RunSetup {
        gate_exe: support::install_gate(&distro),
        logs: support::logs_dir(),
        key: Some(ApiKey::new(MADE_UP_KEY.into())),
        decision_deadline: Duration::from_secs(10),
        model_url: Some("http://127.0.0.1:47940".into()),
    };
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let start = Instant::now();
    let r = run(&req, &paths, &policy, &setup).unwrap();
    println!("--- {:.1} s; the report:\n{r:#?}", start.elapsed().as_secs_f64());

    let tests = &r.commands[0];
    assert_eq!(
        (tests.command.as_str(), &tests.check, tests.exit_code, tests.after_stop),
        ("python3 -m unittest discover", &Check::Allowed, Some(5), false)
    );
    assert!(tests.output.contains("NO TESTS RAN"), "{:?}", tests.output);
    assert_eq!((r.commands[1].command.as_str(), r.commands[1].exit_code), ("touch made.txt", Some(0)));
    assert_eq!(r.commands.len(), 2);
    assert_eq!(r.changed_files, Some(vec!["?? made.txt".to_owned()]), "from Steplet's own git status");
    assert_eq!(r.trust, Trust::NotRecorded, "the run's config recorded no trust (D3)");
    assert!(r.completed, "the run ended on its own; the tests' own exit code is 5");
    assert_eq!(r.agent_says, THE_END);
    assert_eq!((r.stopped_by.clone(), r.refused.len(), r.refusals.len()), (None, 0, 0));
    assert!(!r.models.is_empty());
    assert_eq!(r.faults, Vec::<String>::new());
    let left = support::root_sh(&distro, "ls -A /work; ls -A /var/lib/steplet/runs");
    assert_eq!(left, "", "the run was removed after Steplet's looks");
}

/// Real Claude Code, with the scripted model: `grep -c` that finds nothing prints 0 and exits 1, and Claude Code
/// marks that as no error (exit code 1 of grep means "no matches" to it). The report must not show exit code 0 for it.
/// `tail -f` never ends by itself; Claude Code's own time limit for the call (2 s here) ends it or moves it to the
/// background, and that is not exit code 0 either. Claude Code's own record of each call is printed from the stream.
#[test]
#[ignore = "needs Steplet's Linux and the network"]
fn no_error_from_claude_code_does_not_always_mean_exit_code_0() {
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, "t10c").unwrap();
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n---\n";
    let req = RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: "https://github.com/octocat/Hello-World.git".into(),
        commit: "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d".into(),
        task_text: "Follow the script.".into(),
        max_turns: 20,
        deadline: Duration::from_secs(180),
    };
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": "grep -c made-up-absent-text README", "description": "Count"}},
        {"tool": "Bash", "input": {"command": "tail -f README", "description": "Follow", "timeout": 2000}},
        {"text": THE_END}]});
    let _model = support::scripted_model(&distro, "t10c", &script.to_string(), 47941);
    let setup = RunSetup {
        gate_exe: support::install_gate(&distro),
        logs: support::logs_dir(),
        key: Some(ApiKey::new(MADE_UP_KEY.into())),
        decision_deadline: Duration::from_secs(10),
        model_url: Some("http://127.0.0.1:47941".into()),
    };
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();
    let r = run(&req, &paths, &policy, &setup).unwrap();
    println!("--- the report:\n{r:#?}");
    let stream = std::fs::read_to_string(support::logs_dir().join("t10c.jsonl")).unwrap();
    for line in stream.lines().filter(|l| l.contains("\"tool_use_result\"")) {
        let v: Value = serde_json::from_str(line).unwrap();
        println!(
            "--- Claude Code's record: is_error {} {}",
            v["message"]["content"][0]["is_error"], v["tool_use_result"]
        );
    }

    let grep = &r.commands[0];
    assert_eq!(
        (grep.command.as_str(), &grep.check, grep.output.trim()),
        ("grep -c made-up-absent-text README", &Check::NotAsked { harmless: true }, "0")
    );
    assert_eq!(grep.exit_code, None, "grep found nothing and exited 1; Claude Code called that no error");
    assert_eq!(r.commands[1].command, "tail -f README");
    assert_ne!(r.commands[1].exit_code, Some(0), "a command ended by Claude Code's time limit did not exit 0");
}

/// Phase 4 Task 3 (J6 A): the raw logs Steplet keeps on Windows hide the key and the run's token in each line as it
/// is written; every other byte stays exactly as Claude Code wrote it, escapes and bytes that are not text included.
#[test]
fn raw_log_lines_hide_the_key_and_the_token() {
    let line = format!("\u{1b}[2K{{\"text\":\"key {MADE_UP_KEY}, token {MADE_UP_TOKEN}\\u001b\"}}\r\n");
    let want = format!("\u{1b}[2K{{\"text\":\"key {HIDDEN}, token {HIDDEN}\\u001b\"}}\r\n");
    assert_eq!(raw_log_line(line.as_bytes(), &secrets()), want.as_bytes());
    let not_text: &[u8] = b"{\"type\":\"system\"}\xff\xfe\n";
    assert_eq!(raw_log_line(not_text, &secrets()), not_text);
    assert_eq!(raw_log_line(b"", &secrets()), b"");
}
