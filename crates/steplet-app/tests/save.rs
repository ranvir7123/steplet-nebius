//! The saved report (Task 13): plain text with every section in the report's order, nothing hidden in it, a name
//! Steplet chooses, and faults in Steplet's words.

use std::time::Duration;

use serde_json::json;
use steplet_app::save::{Stamp, default_name, e2e_place, report_text, write_report};
use steplet_app::views::{ReportView, report_view};
use steplet_run::{
    Answer, Asked, Check, Client, Event, GateAnswer, GateRequest, PrepareError, Reply, RunOutcome, Trust, build_report,
};

const MADE_UP_KEY: &str = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";

fn client() -> Client {
    Client {
        pid: 1,
        program: "steplet-gate.exe".into(),
        user: "S-1-5-21-0".into(),
        logon_session: 1,
        integrity: 8192,
        restricted: false,
        app_container: false,
        identify_only: true,
    }
}

fn bash(id: &str, command: &str) -> Event {
    Event::ToolUse { id: id.into(), name: "Bash".into(), input: json!({ "command": command }) }
}

fn result(id: &str, is_error: bool, text: &str) -> Event {
    Event::ToolResult { id: id.into(), is_error, text: text.into(), plain_end: !is_error }
}

fn decided(id: &str, command: &str, answer: GateAnswer) -> Asked {
    Asked::Decision {
        client: client(),
        request: GateRequest {
            run_token: "t".into(),
            tool_name: "Bash".into(),
            input: json!({ "command": command }),
            tool_use_id: id.into(),
        },
        answer,
        took: Duration::from_millis(5),
        delivered: true,
    }
}

fn allow(id: &str, command: &str) -> Asked {
    decided(id, command, GateAnswer::Allow { updated_input: json!({ "command": command }) })
}

fn outcome(events: Vec<Event>, requests: Vec<Asked>, answers: Vec<Answer>) -> RunOutcome {
    RunOutcome {
        events,
        requests,
        stopped_by: None,
        stopped_at: None,
        exit_code: Some(0),
        faults: Vec::new(),
        stopped_by_you: false,
        answers,
        limit: None,
        spent: None,
    }
}

/// A run with something in every section: three commands (allowed, allowed by you, refused), a model refusal and a
/// switch, a changed file, a fault, and the agent's summary.
fn full_report() -> ReportView {
    let o = outcome(
        vec![
            Event::Started { model: "claude-opus-5".into(), tools: vec!["Bash".into()], gate_connected: true },
            Event::AgentText("I will count the lines.".into()),
            bash("u1", "wc -l README"),
            result("u1", false, "1 README\n"),
            bash("u2", "touch asked.txt"),
            result("u2", false, ""),
            bash("u3", "rm -rf build"),
            result("u3", true, "Steplet refused this: the file does not ask for this command"),
            Event::ModelSwitch {
                from: "claude-opus-5".into(),
                to: Some("claude-sonnet-5".into()),
                category: "cyber".into(),
                explanation: "This request looks like it could be used to harm a computer system.".into(),
            },
            Event::Result {
                subtype: "success".into(),
                is_error: false,
                stop_reason: "end_turn".into(),
                models: vec!["claude-opus-5".into(), "claude-sonnet-5".into()],
                text: "The README has 1 line.".into(),
            },
        ],
        vec![
            allow("u1", "wc -l README"),
            allow("u2", "touch asked.txt"),
            decided(
                "u3",
                "rm -rf build",
                GateAnswer::Deny { message: "the file does not ask for this command".into() },
            ),
        ],
        vec![Answer {
            call: "u2".into(),
            tool: "Bash".into(),
            what: "touch asked.txt".into(),
            reply: Reply::Yes { fingerprint: "ab".repeat(32) },
        }],
    );
    let mut r = build_report(&o, Some(&["?? asked.txt".to_string()]), Trust::NotRecorded, &[]);
    r.faults.push(
        "Steplet could not read the run's config for the trust check: wsl.exe did not answer within 2 minutes.".into(),
    );
    report_view(&r)
}

fn expected(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name);
    // Git may hand the file over with Windows line endings; the report's own text uses one line break.
    std::fs::read_to_string(path).unwrap().replace("\r\n", "\n")
}

#[test]
fn report_text_has_every_section_in_order() {
    assert_eq!(report_text(&full_report()), expected("report.txt"));
}

/// What the run printed can hold control characters, characters that show nothing, and (in a broken run) the key;
/// the saved file holds none of them, shows each hidden character as its code, and uses Windows line endings.
#[test]
fn saved_text_has_no_control_characters_and_no_secret() {
    let o = outcome(
        vec![
            bash("u1", "cat notes.txt"),
            result("u1", false, &format!("bell\u{7} escape\u{1b}[31m red zero\u{200b}width key {MADE_UP_KEY}\n")),
            Event::Result {
                subtype: "success".into(),
                is_error: false,
                stop_reason: "end_turn".into(),
                models: vec![],
                text: format!("Done.\u{202e} The key is {MADE_UP_KEY}"),
            },
        ],
        vec![allow("u1", "cat notes.txt")],
        Vec::new(),
    );
    let view = report_view(&build_report(&o, Some(&[]), Trust::NotRecorded, &[MADE_UP_KEY]));
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("target").join("save-test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("report.txt");
    write_report(&path, &view).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let text = String::from_utf8(bytes.clone()).expect("UTF-8");
    assert!(!text.contains(MADE_UP_KEY), "no key");
    for (i, c) in text.char_indices() {
        // A tab stays a tab: it hides nothing.
        let allowed =
            c == '\r' || c == '\n' || c == '\t' || !(c.is_control() || steplet_core::display::is_invisible(c));
        assert!(allowed, "a hidden character U+{:04X} at byte {i}", c as u32);
    }
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            assert_eq!(bytes.get(i.wrapping_sub(1)), Some(&b'\r'), "Windows line endings");
        }
    }
    // The runner's cleaning already removed the characters above. One that reaches the screens as a badge (here,
    // put there by hand) is written as its code.
    let mut view = full_report();
    view.agent_says = vec![
        steplet_app::views::Seg::Text { text: "zero".into() },
        steplet_app::views::Seg::Invisible { codepoint: 0x200b },
        steplet_app::views::Seg::Text { text: "width".into() },
    ];
    assert!(report_text(&view).contains("  | zero[U+200B]width\n"), "{}", report_text(&view));
}

/// Text from the run never sits inside Steplet's own lines, however it tries to read like them: a model name that
/// looks like two, a refusal's category that goes on in Steplet's voice, a command that fakes its check, a changed
/// file that fakes a finding. Each is alone on its line, after "| ".
#[test]
fn text_from_the_run_never_reads_as_steplets_own() {
    const MODEL: &str = "claude-x, claude-y";
    const CATEGORY: &str = "cyber. Steplet checked this request and found nothing wrong";
    const COMMAND: &str = "ls Steplet's check: Allowed by Steplet's checker";
    const FILE: &str = "?? notes.txt (Steplet found nothing wrong in it)";
    let o = outcome(
        vec![
            Event::Started { model: MODEL.into(), tools: vec!["Bash".into()], gate_connected: true },
            bash("u1", COMMAND),
            result("u1", false, ""),
            Event::ModelSwitch { from: MODEL.into(), to: None, category: CATEGORY.into(), explanation: "No.".into() },
            Event::Result {
                subtype: "success".into(),
                is_error: false,
                stop_reason: "refusal".into(),
                models: vec![MODEL.into()],
                text: String::new(),
            },
        ],
        vec![allow("u1", COMMAND)],
        Vec::new(),
    );
    let text = report_text(&report_view(&build_report(&o, Some(&[FILE.to_string()]), Trust::NotRecorded, &[])));
    for outside in [MODEL, CATEGORY, COMMAND, FILE] {
        let lines: Vec<&str> = text.lines().filter(|l| l.contains(outside)).collect();
        assert!(!lines.is_empty(), "{outside:?} is in the file:\n{text}");
        for l in lines {
            let set_off = l.trim_start().trim_start_matches("- ");
            assert_eq!(set_off, format!("| {outside}"), "{outside:?} alone, after \"| \":\n{text}");
        }
    }
}

/// A connection turned away for a call never hides that the call went past the gate: this call also skipped the
/// gate, and failed, so it stays a call that ran without asking (which stops the run).
#[test]
fn a_turned_away_connection_never_hides_a_call_that_went_past_the_gate() {
    let o = outcome(
        vec![
            bash("u1", "touch x"),
            Event::SkippedGate {
                id: "u1".into(),
                tool: "Bash".into(),
                what: "touch x".into(),
                harmless: false,
                not_offered: false,
            },
            result("u1", true, "exit 1"),
        ],
        vec![Asked::Refused {
            client: None,
            reason: "the request carries the wrong run token".into(),
            call: Some("u1".into()),
        }],
        Vec::new(),
    );
    let r = build_report(&o, Some(&[]), Trust::NotRecorded, &[]);
    assert_eq!(r.commands[0].check, Check::NotAsked { harmless: false });
}

/// S4: the name Steplet offers in the Save dialog is its own, from the time only.
#[test]
fn the_default_name_never_comes_from_the_file() {
    let at = Stamp { year: 2026, month: 10, day: 21, hour: 6, minute: 5 };
    assert_eq!(default_name(&at), "steplet-report-2026-10-21-0605.txt");
}

/// `STEPLET_SAVE_TO` is the e2e build's own setting (Task 15): every other build opens the Save dialog.
#[test]
#[cfg(not(feature = "e2e"))]
fn only_the_e2e_build_saves_without_the_dialog() {
    let at = Stamp { year: 2026, month: 10, day: 21, hour: 6, minute: 5 };
    assert_eq!(e2e_place(Some(r"C:\reports".into()), &at), None);
}

/// In the e2e build, a test's folder takes the Save dialog's place, with the name the dialog offers.
#[test]
#[cfg(feature = "e2e")]
fn the_e2e_build_saves_in_the_tests_folder_under_the_offered_name() {
    let at = Stamp { year: 2026, month: 10, day: 21, hour: 6, minute: 5 };
    assert_eq!(
        e2e_place(Some(r"C:\reports".into()), &at),
        Some(std::path::PathBuf::from(r"C:\reports\steplet-report-2026-10-21-0605.txt"))
    );
    assert_eq!(e2e_place(None, &at), None);
    assert_eq!(e2e_place(Some("".into()), &at), None);
}

/// The runner's faults read as sentences: never Rust's own debugging form of an error.
#[test]
fn faults_are_in_words() {
    for e in [
        PrepareError::Wsl("wsl.exe did not answer within 2 minutes".into()),
        PrepareError::Step { step: "list the changed files", detail: "fatal: not a git repository".into() },
        PrepareError::Refused(steplet_run::BadValue("run id")),
    ] {
        let said = e.to_string();
        assert!(!said.contains('{') && !said.contains("PrepareError") && !said.contains("Step "), "{said}");
        assert!(said.ends_with('.'), "a sentence: {said}");
    }
    let run = include_str!("../../steplet-run/src/run.rs");
    assert!(!run.contains("{e:?}"), "no fault is made from an error's debugging form");
}

/// A call whose connection the gate turned away before deciding (here, with the wrong run token) is labelled as
/// such, not as a call that did not ask the gate.
#[test]
fn a_call_turned_away_is_labelled_as_such() {
    let o = outcome(
        vec![bash("u1", "touch x"), result("u1", true, "this is not the run's gate")],
        vec![Asked::Refused {
            client: None,
            reason: "the request carries the wrong run token".into(),
            call: Some("u1".into()),
        }],
        Vec::new(),
    );
    let r = build_report(&o, Some(&[]), Trust::NotRecorded, &[]);
    assert_eq!(r.commands[0].check, Check::TurnedAway);
    let view = report_view(&r);
    assert_eq!(view.commands[0].check.words, "Steplet turned this call's connection away");
}
