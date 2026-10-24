//! What the screens get (Task 9): the preview says exactly what the core says, file text reaches the screens only as
//! display pieces, one entry never looks like two, and no view holds the receiver's key.

use std::path::PathBuf;

use serde_json::{Value, json};
use steplet_app::views::{
    Here, LineStatus, ProgressView, Seg, Switch, event_view, failure, line, preview, question_view, report_view,
    run_failure, setup_failure, setup_line, setup_view,
};
use steplet_core::report::check;
use steplet_run::setup::SetupError;
use steplet_run::setup::{State, Wsl};
use steplet_run::{Answer, Event, GateError, Question, Reply, RunError, RunOutcome, Trust, build_report};

/// The corpus's own time of viewing (crates/steplet-core/tests/corpus.rs), so expiry findings are the expected ones.
const NOW: i64 = 1_791_720_000;
/// Not a key: made-up test data, the same text the runner's tests use.
const MADE_UP_KEY: &str = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";

fn corpus_files() -> Vec<PathBuf> {
    let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus"));
    let mut files = Vec::new();
    for dir in ["ordinary", "hostile"] {
        for e in std::fs::read_dir(root.join(dir)).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "steplet") {
                files.push(p);
            }
        }
    }
    files.sort();
    files
}

fn nothing_here() -> Here {
    Here { identity: None, findings: Vec::new(), not_runnable: None }
}

#[test]
fn the_preview_agrees_with_the_core() {
    let files = corpus_files();
    assert!(files.len() > 100, "the corpus is there: {} files", files.len());
    for p in files {
        let raw = std::fs::read(&p).unwrap();
        let report = check(&raw, NOW);
        let view = preview(&raw, &report, &nothing_here());
        let name = p.display();
        assert_eq!(view.outcome, report.rating.outcome.name(), "{name}");
        let core: Vec<(&str, Option<u32>)> = report.rating.findings.iter().map(|f| (f.rule.id, f.line)).collect();
        let shown: Vec<(&str, Option<u32>)> = view.findings.iter().map(|f| (f.rule.as_str(), f.line)).collect();
        assert_eq!(shown, core, "{name}");
        assert!(view.here.is_empty(), "{name}");
    }
}

/// Every string in `v` that holds `marker`, with the path to it, unless it is the `text` of a display piece.
fn outside_pieces(v: &Value, path: &str, marker: &str, out: &mut Vec<String>) {
    match v {
        Value::String(s) if s.contains(marker) => out.push(path.into()),
        Value::Array(items) => {
            for (i, x) in items.iter().enumerate() {
                outside_pieces(x, &format!("{path}[{i}]"), marker, out);
            }
        }
        Value::Object(fields) => {
            let piece = fields.get("kind").is_some_and(|k| k == "text") && fields.len() == 2;
            for (k, x) in fields {
                if !(piece && k == "text") {
                    outside_pieces(x, &format!("{path}.{k}"), marker, out);
                }
            }
        }
        _ => {}
    }
}

#[test]
fn file_text_reaches_the_screen_only_as_segments() {
    // Every part of this file holds the made-up word Qzx, so wherever the file's text lands, it can be found.
    let file = "---\nsteplet: 1\ntitle: Qzx title\nsender: Qzx sender\nexpires: 2026-12-31\nrepository:\n  url: \
                https://github.com/qzx/qzx.git\n  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d\nsteps:\n  - Check\u{200b}Qzx\nallow:\n  \
                commands:\n    - wc -l Qzx\n  domains:\n    - qzx.example.org\n  read:\n    - qzx\n  write:\n    - qzx\nnever:\n  - \
                Qzx never\n---\nQzx body [Qzx](https://qzx.example.org/Qzx)\n";
    let report = check(file.as_bytes(), NOW);
    let view = preview(file.as_bytes(), &report, &nothing_here());
    let f = view.file.as_ref().expect("the header reads");
    assert_eq!(
        f.steps[0].text,
        vec![
            Seg::Text { text: "Check".into() },
            Seg::Invisible { codepoint: 0x200b },
            Seg::Text { text: "Qzx".into() }
        ]
    );
    let mut stray = Vec::new();
    outside_pieces(&serde_json::to_value(&view).unwrap(), "preview", "Qzx", &mut stray);
    outside_pieces(&serde_json::to_value(&view).unwrap(), "preview", "qzx", &mut stray);
    assert_eq!(stray, Vec::<String>::new(), "file text outside display pieces");
}

fn outcome(events: Vec<Event>) -> RunOutcome {
    RunOutcome {
        events,
        requests: Vec::new(),
        stopped_by: None,
        stopped_at: None,
        exit_code: Some(0),
        faults: Vec::new(),
        stopped_by_you: false,
        answers: Vec::new(),
    }
}

#[test]
fn a_newline_inside_one_entry_shows_as_a_badge() {
    let changed = vec!["?? a\nb".to_string()];
    let report = build_report(&outcome(Vec::new()), Some(&changed), Trust::NotRecorded, &[]);
    let view = report_view(&report);
    assert_eq!(
        view.changed_files.unwrap()[0],
        vec![Seg::Text { text: "?? a".into() }, Seg::Invisible { codepoint: 0x0a }, Seg::Text { text: "b".into() }]
    );
    // A block of text keeps its line breaks as line breaks.
    assert_eq!(line("one"), vec![Seg::Text { text: "one".into() }]);
    assert_eq!(steplet_app::views::shown("a\nb"), vec![Seg::Text { text: "a\nb".into() }]);
}

#[test]
fn no_view_holds_the_key() {
    let k = MADE_UP_KEY;
    let events = vec![
        Event::Started { model: format!("model-{k}"), tools: vec!["Bash".into()], gate_connected: true },
        Event::AgentText(format!("the key is {k}")),
        Event::ToolUse { id: "c1".into(), name: "Bash".into(), input: json!({"command": format!("echo {k}")}) },
        Event::ToolResult { id: "c1".into(), is_error: false, text: format!("{k}\n"), plain_end: true },
        Event::ModelSwitch {
            from: format!("m-{k}"),
            to: Some(format!("n-{k}")),
            category: format!("c-{k}"),
            explanation: format!("e-{k}"),
        },
        Event::Result {
            subtype: "success".into(),
            is_error: false,
            stop_reason: "end_turn".into(),
            models: vec![format!("m-{k}")],
            text: format!("done {k}"),
        },
    ];
    let mut o = outcome(events);
    o.faults.push(format!("a fault naming {k}"));
    o.answers.push(Answer { call: "c1".into(), tool: "Bash".into(), what: format!("echo {k}"), reply: Reply::No });
    let changed = vec![format!("?? {k}.txt")];
    let report = build_report(&o, Some(&changed), Trust::NotRecorded, &[k]);
    let text = serde_json::to_string(&report_view(&report)).unwrap();
    assert!(text.contains("[hidden by Steplet]"), "the key's places are marked");
    assert!(!text.contains(k), "{text}");
    assert!(!text.contains("STEPLET-TEST-MADE-UP"), "no part of the key either");
}

fn stray_text(v: &impl serde::Serialize, marker: &str) -> Vec<String> {
    let mut stray = Vec::new();
    outside_pieces(&serde_json::to_value(v).unwrap(), "view", marker, &mut stray);
    stray
}

fn trouble_state(trouble: Option<&str>, wsl: Wsl, linux: bool) -> State {
    State {
        wsl,
        linux: linux.then(|| "{44f36236-99b1-42c0-84bd-7ff8392d51ba}".into()),
        missing: Vec::new(),
        gate: false,
        trouble: trouble.map(Into::into),
    }
}

fn ready() -> Wsl {
    Wsl::Ready { version: "2.6.3.0".into() }
}

#[test]
fn what_a_program_printed_when_setup_could_not_look_is_file_text() {
    // setup::State.trouble ends with what programs inside the Linux and wsl.exe printed (Task 6 review).
    let view =
        setup_view(&trouble_state(Some("wsl.exe said: Qzx \u{1b}[31m✅ all fine"), ready(), true), true, Switch::Off);
    assert_eq!(stray_text(&view, "Qzx"), Vec::<String>::new(), "program output outside display pieces");
    assert_eq!(view.tools.status, LineStatus::Unknown, "could not look, which is not the same as missing");
}

#[test]
fn what_a_failed_program_printed_is_file_text() {
    // git prints what the repository's host sends, so a failure's detail is not Steplet's words.
    let f = failure("Steplet could not fetch the repository. Nothing ran.", Some("remote: Qzx ✅ Verified by Steplet"));
    let p = ProgressView::Failed { stage: "Fetching the repository".into(), failure: f };
    assert_eq!(stray_text(&p, "Qzx"), Vec::<String>::new());
}

#[test]
fn setup_says_why_it_cannot_look_yet() {
    let text = |state: &State| setup_view(state, false, Switch::Off).tools.text;
    assert_eq!(
        text(&trouble_state(None, ready(), false)),
        "Steplet can look at the tools in its Linux once its Linux is installed."
    );
    assert_eq!(
        text(&trouble_state(None, Wsl::TooOld { version: "2.0.9.0".into() }, false)),
        "Steplet can look at the tools in its Linux once Windows' Linux feature is updated."
    );
    assert_eq!(
        text(&trouble_state(None, Wsl::Missing, false)),
        "Steplet can look at the tools in its Linux once Windows' Linux feature is there."
    );
    assert_eq!(
        setup_view(&trouble_state(Some("x"), ready(), true), false, Switch::Off).linux.text,
        "Steplet could not look at its own Linux. What went wrong:"
    );
    assert_eq!(
        text(&trouble_state(Some("x"), ready(), true)),
        "Steplet could not look at the tools in its Linux either."
    );
}

#[test]
fn live_and_report_views_keep_outside_text_in_pieces() {
    // Every text field the agent, a command or the model fills holds the word Qzx.
    let q = "Qzx";
    let input = json!({ "command": format!("echo {q}"), format!("{q}-key"): format!("{q} value") });
    let events = vec![
        Event::Started { model: format!("{q}-model"), tools: vec![q.into()], gate_connected: true },
        Event::AgentText(format!("{q} says")),
        Event::ToolUse { id: "c1".into(), name: format!("Bash{q}"), input: input.clone() },
        Event::ToolResult { id: "c1".into(), is_error: true, text: format!("Exit code 1\n{q}"), plain_end: false },
        Event::SkippedGate { id: "c1".into(), tool: q.into(), what: q.into(), harmless: false },
        Event::ModelSwitch { from: q.into(), to: Some(q.into()), category: q.into(), explanation: q.into() },
        Event::HookEvent { name: q.into() },
        Event::Stderr(q.into()),
        Event::Result {
            subtype: "success".into(),
            is_error: false,
            stop_reason: "end_turn".into(),
            models: vec![q.into()],
            text: q.into(),
        },
    ];
    for e in &events {
        let view = event_view(e, Some(("Bash", &input, false)));
        assert_eq!(stray_text(&view, q), Vec::<String>::new(), "{e:?}");
    }
    let question = Question {
        id: 1,
        tool: format!("Bash{q}"),
        what: format!("echo {q}"),
        input: input.clone(),
        reason: "the file does not ask for this command".into(),
        fingerprint: "f".into(),
        asked_at: std::time::Instant::now(),
    };
    assert_eq!(stray_text(&question_view(&question, std::time::Duration::from_secs(300)), q), Vec::<String>::new());
    let mut o = outcome(events);
    o.faults.push(format!("{q} fault"));
    o.stopped_by = Some(steplet_run::Tripwire::SkippedCommand(q.into()));
    o.answers.push(Answer { call: "c1".into(), tool: q.into(), what: q.into(), reply: Reply::No });
    let changed = vec![format!("?? {q}")];
    let report = build_report(&o, Some(&changed), Trust::NotRecorded, &[]);
    assert_eq!(stray_text(&report_view(&report), q), Vec::<String>::new());
}

#[test]
fn setup_lines_lose_terminal_codes_but_keep_other_hidden_characters() {
    let text = |s: &[Seg]| -> String {
        s.iter()
            .map(|p| match p {
                Seg::Text { text } => text.clone(),
                Seg::Invisible { codepoint } => format!("<U+{codepoint:04X}>"),
                Seg::Link { .. } => "<link>".into(),
            })
            .collect()
    };
    // Colour codes, as systemd printed them in Task 6's real install log.
    assert_eq!(
        text(&setup_line("\u{1b}[0;1;31mFailed to connect to system scope bus\u{1b}[0m")),
        "Failed to connect to system scope bus"
    );
    // A progress bar redrawn in place shows its last drawing.
    assert_eq!(text(&setup_line("\u{1b}[2K[===    ] 41.0%\r\u{1b}[2K[=======] 100.0%")), "[=======] 100.0%");
    // Anything else that shows nothing is still a badge, a lone escape character too.
    assert_eq!(text(&setup_line("pay\u{200b}ments \u{1b}done")), "pay<U+200B>ments <U+001B>done");
}

#[test]
fn a_setup_failure_says_where_it_stopped_in_words() {
    let f = setup_failure(&SetupError::Step {
        step: "set up Steplet's Linux",
        detail: "\u{1b}[0;1;31mE: Failed to fetch\u{1b}[0m".into(),
    });
    assert_eq!(
        f.reason,
        "Setup stopped while trying to set up Steplet's Linux. Trying again is safe: Steplet picks up where it \
         stopped. What it printed last:"
    );
    assert_eq!(f.detail, Some(vec![Seg::Text { text: "E: Failed to fetch".into() }]), "file text, codes left out");
    let refused = setup_failure(&SetupError::Refused("Linux name"));
    assert_eq!(refused.reason, "Steplet will not hand this Linux name to WSL.");
    assert_eq!(refused.detail, None);
}

/// Phase 5 Task 2: the run that does not start because another program holds its gate channel's name
/// (`a_run_whose_gate_channel_name_is_taken_does_not_start`) says so in words.
#[test]
fn a_run_whose_gate_channel_name_is_taken_says_so_in_words() {
    let (_, f) = run_failure(&RunError::Gate(GateError::Taken));
    assert_eq!(f.reason, "Another program holds the name of this run's safety gate, so Steplet did not start it.");
    assert_eq!(f.detail, None);
}

#[test]
fn every_setup_check_has_plain_words() {
    let named: Vec<&str> = steplet_app::views::CHECK_WORDS.iter().map(|(check, _)| *check).collect();
    assert_eq!(named, steplet_run::setup::CHECKS, "one entry for each check look.sh makes, in its order");
    let state = State {
        wsl: Wsl::Ready { version: "2.6.3.0".into() },
        linux: Some("{made-up-id}".into()),
        missing: vec!["claude-code".into(), "wsl.conf".into()],
        gate: true,
        trouble: None,
    };
    let view = setup_view(&state, true, Switch::Off);
    assert_eq!(view.tools.status, LineStatus::Missing);
    assert_eq!(
        view.tools.text,
        "Missing in Steplet's Linux: Claude Code 2.1.220 and WSL's settings for Steplet's Linux."
    );
}

#[test]
fn setup_says_what_to_do_next() {
    use steplet_app::views::Offer;
    let st = |wsl: Wsl, linux: bool, missing: &[&str], gate: bool, trouble: Option<&str>| State {
        wsl,
        linux: linux.then(|| "{made-up-id}".to_string()),
        missing: missing.iter().map(|m| m.to_string()).collect(),
        gate,
        trouble: trouble.map(str::to_string),
    };
    let ready = || Wsl::Ready { version: "2.6.3.0".into() };
    let next = |s: State| {
        let v = setup_view(&s, true, Switch::Off);
        (v.wsl_fix.map(|f| f.command), v.install)
    };
    // Windows' own feature: Steplet shows the command to type, and offers no install of its own Linux yet.
    assert_eq!(
        next(st(Wsl::Missing, false, &[], false, None)),
        (Some("wsl --install --no-distribution".into()), Offer::None)
    );
    assert_eq!(
        next(st(Wsl::TooOld { version: "2.0.9.0".into() }, false, &[], false, None)),
        (Some("wsl --update".into()), Offer::None)
    );
    assert_eq!(next(st(ready(), false, &[], false, None)), (None, Offer::Install));
    assert_eq!(next(st(ready(), true, &["srt"], true, None)), (None, Offer::Finish));
    assert_eq!(next(st(ready(), true, &[], false, None)), (None, Offer::Finish));
    assert_eq!(next(st(ready(), true, &[], true, None)), (None, Offer::None));
    // Steplet could not look: it does not guess, and offers Check again only.
    assert_eq!(next(st(ready(), false, &[], false, Some("WSL did not answer"))), (None, Offer::None));
}

#[test]
fn setup_says_what_went_wrong_once() {
    let view =
        setup_view(&trouble_state(Some("WSL did not answer within 60 seconds"), ready(), true), true, Switch::Off);
    assert!(view.linux.detail.is_some(), "the first line it could not look at says what went wrong");
    assert_eq!((view.tools.detail.clone(), view.gate.detail.clone()), (None, None), "and only that line");
    assert_eq!(view.tools.text, "Steplet could not look at the tools in its Linux either.");
    assert_eq!(view.gate.text, "Steplet could not look at its gate program either.");
}

#[test]
fn steplets_own_words_carry_no_decision_codes() {
    // "(D1)", "(R16)" and the like are the decision log's names, not words for a receiver.
    let coded = |text: &str| -> Vec<String> {
        text.split('(')
            .skip(1)
            .filter_map(|rest| rest.split(')').next())
            .filter(|inner| {
                let letters = inner.chars().take_while(|c| c.is_ascii_uppercase()).count();
                (1..=2).contains(&letters)
                    && inner.len() > letters
                    && inner[letters..].chars().all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
                    && inner[letters..].starts_with(|c: char| c.is_ascii_digit())
            })
            .map(str::to_string)
            .collect()
    };
    let example = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/run-tests.steplet")).unwrap();
    let file = check(&example, NOW).steplet.unwrap();
    let chooser = serde_json::to_string(&steplet_app::views::chooser(
        &steplet_run::promises(&file, steplet_run::NetworkMode::Nat),
        true,
    ))
    .unwrap();
    let mut setups = String::new();
    for (wsl, linux, trouble) in [
        (Wsl::Missing, false, None),
        (Wsl::TooOld { version: "2.0.9.0".into() }, false, None),
        (ready(), false, None),
        (ready(), true, Some("WSL did not answer")),
        (ready(), true, None),
    ] {
        setups += &serde_json::to_string(&setup_view(&trouble_state(trouble, wsl, linux), false, Switch::Off)).unwrap();
    }
    assert_eq!(coded(&chooser), Vec::<String>::new(), "the chooser");
    assert_eq!(coded(&setups), Vec::<String>::new(), "the setup screen");
    assert_eq!(coded("as decided (D1) and (R16)"), ["D1", "R16"], "the check itself finds them");
}

/// A call refused after Steplet asked the receiver says how the question ended, never "Steplet's checker".
#[test]
fn a_refusal_after_asking_says_how_it_ended() {
    use steplet_app::views::check_view;
    use steplet_run::{Check, NotYes};
    let words = |how| check_view(&Check::RefusedWhenAsked { how });
    for (how, said) in [
        (NotYes::SaidNo, "Refused: you said no"),
        (NotYes::NoAnswer, "Refused: no answer in time"),
        (NotYes::TakenBack, "Refused: Steplet took the question back as the run stopped"),
    ] {
        let v = words(how);
        assert_eq!((v.kind.as_str(), v.words.as_str()), ("refused_by_you", said));
    }
    assert_eq!(check_view(&Check::Refused).words, "Refused by Steplet's checker");
}

/// A call that ran without asking Steplet and was no harmless read stopped the run: its kind says so, so the screen
/// marks it as it marks a refusal that ran, not as a harmless read.
#[test]
fn a_call_that_ran_without_asking_is_not_marked_as_a_harmless_read() {
    use steplet_app::views::check_view;
    use steplet_run::Check;
    assert_eq!(check_view(&Check::NotAsked { harmless: true }).kind, "not_asked");
    assert_eq!(check_view(&Check::NotAsked { harmless: false }).kind, "ran_without_asking");
}

/// Task 12: while Claude Code tries again after an error from the model's service, the live log says so in
/// Steplet's words (a wrong key takes about three minutes of tries); the error at the end shows nothing of its own,
/// since the report's fault says it.
#[test]
fn each_try_after_a_service_error_is_said_in_steplets_words() {
    use std::time::Duration;
    use steplet_app::views::{EventView, Limits, progress_view};
    use steplet_run::Progress;
    let limits = Limits { ask: Duration::from_secs(300), run: Duration::from_secs(1800) };
    let said = |status, attempt| match progress_view(
        &Progress::Event(Event::ApiRetry { status, error: "x".into(), attempt, max: 10 }),
        None,
        limits,
    ) {
        Some(ProgressView::Event { event: EventView::Retry { words } }) => words,
        other => panic!("{other:?}"),
    };
    assert_eq!(said(Some(401), 1), "Anthropic did not accept the key. Claude Code tries again (1 of 10).");
    assert_eq!(said(Some(529), 3), "Anthropic's service answered with error 529. Claude Code tries again (3 of 10).");
    assert_eq!(said(None, 2), "Claude Code could not reach Anthropic's service. It tries again (2 of 10).");
    assert_eq!(progress_view(&Progress::Event(Event::ApiError { status: 401 }), None, limits), None);
}

/// E1: Rust says when "Run again, allowing another model" is on offer: after a model refused and Claude Code did not
/// switch, and never after a run that finished or stopped.
#[test]
fn another_model_is_offered_only_after_a_model_refused() {
    let refusal = Event::ModelSwitch {
        from: "claude-opus-5".into(),
        to: None,
        category: "cyber".into(),
        explanation: "No.".into(),
    };
    let ended = |stop_reason: &str| Event::Result {
        subtype: "success".into(),
        is_error: false,
        stop_reason: stop_reason.into(),
        models: vec!["claude-opus-5".into()],
        text: String::new(),
    };
    let offers = |o: RunOutcome| report_view(&build_report(&o, Some(&[]), Trust::NotRecorded, &[])).offers_model_switch;
    assert!(offers(outcome(vec![refusal.clone(), ended("refusal")])));
    assert!(!offers(outcome(vec![ended("end_turn")])));
    assert!(!offers(outcome(Vec::new())));
    let mut stopped = outcome(vec![refusal, ended("refusal")]);
    stopped.stopped_by_you = true;
    assert!(!offers(stopped));
}

/// Found by looking (Task 14): Windows' own words reach the screen without the code number Rust adds to them.
#[test]
fn windows_errors_read_without_their_code_number() {
    use steplet_app::views::windows_says;
    assert_eq!(windows_says("Access is denied. (os error 5)"), "Windows says: Access is denied.");
    assert_eq!(windows_says("The disk is full."), "Windows says: The disk is full.");
    assert_eq!(windows_says("a (os error 5) in the middle"), "Windows says: a (os error 5) in the middle");
    // Task 14 review: only Rust's own code number goes, and an empty message says so.
    assert_eq!(
        windows_says("File (os error 5) is bad (see docs)"),
        "Windows says: File (os error 5) is bad (see docs)"
    );
    assert_eq!(windows_says(" (os error 5)"), "Windows gave no reason.");
    assert_eq!(windows_says(""), "Windows gave no reason.");
}

/// Found by looking (Task 14): the help for Windows' Linux feature says what is wrong with it, missing or too old.
#[test]
fn the_wsl_help_says_whether_it_is_missing_or_too_old() {
    let fix = |wsl| setup_view(&trouble_state(None, wsl, false), false, Switch::Off).wsl_fix.expect("a fix");
    let missing = fix(Wsl::Missing);
    assert_eq!(missing.command, "wsl --install --no-distribution");
    assert!(missing.why.starts_with("Windows' Linux feature has to be on"), "{}", missing.why);
    assert_eq!(missing.then, "Then restart Windows, and press Check again.");
    let old = fix(Wsl::TooOld { version: "2.0.9.0".into() });
    assert_eq!(old.command, "wsl --update");
    assert!(old.why.contains("too old") && !old.why.contains("has to be on"), "{}", old.why);
    assert_eq!(old.then, "Then press Check again.");
    assert!(setup_view(&trouble_state(None, ready(), false), false, Switch::Off).wsl_fix.is_none());
}

/// Found by looking (Task 14): a command that failed stands out from one that passed, and a command refused before it
/// ran says so instead of "Exit code not known".
#[test]
fn each_command_says_how_it_ended() {
    let ended = |events: Vec<Event>, requests| {
        let o = RunOutcome { requests, ..outcome(events) };
        let c = &report_view(&build_report(&o, Some(&[]), Trust::NotRecorded, &[])).commands[0];
        (c.ended.clone(), c.failed)
    };
    let bash =
        |command: &str| Event::ToolUse { id: "u1".into(), name: "Bash".into(), input: json!({ "command": command }) };
    let result = |is_error: bool, text: &str| Event::ToolResult {
        id: "u1".into(),
        is_error,
        text: text.into(),
        plain_end: !is_error,
    };
    let decided = |answer| {
        vec![steplet_run::Asked::Decision {
            client: client(),
            request: steplet_run::GateRequest {
                run_token: "t".into(),
                tool_name: "Bash".into(),
                input: json!({ "command": "make test" }),
                tool_use_id: "u1".into(),
            },
            answer,
            took: std::time::Duration::from_millis(5),
            delivered: true,
        }]
    };
    let allow = || decided(steplet_run::GateAnswer::Allow { updated_input: json!({ "command": "make test" }) });
    assert_eq!(ended(vec![bash("make test"), result(false, "ok\n")], allow()), ("Exit code 0".into(), false));
    assert_eq!(
        ended(vec![bash("make test"), result(true, "Exit code 2\nfailed\n")], allow()),
        ("Exit code 2".into(), true)
    );
    let deny = decided(steplet_run::GateAnswer::Deny { message: "the file does not ask for this command".into() });
    assert_eq!(
        ended(vec![bash("make test"), result(true, "Steplet refused this")], deny),
        ("It did not run".into(), false)
    );
    // Task 14 review: the same words as the live log for a tool with no exit code, and for a call with no result.
    let write = Event::ToolUse {
        id: "u1".into(),
        name: "Write".into(),
        input: json!({ "file_path": "/work/r1/repo/a.txt", "content": "a" }),
    };
    assert_eq!(ended(vec![write.clone(), result(false, "File created")], Vec::new()), ("Done".into(), false));
    assert_eq!(
        ended(vec![write.clone(), result(true, "Permission denied")], Vec::new()),
        ("Ended with an error".into(), true)
    );
    assert_eq!(ended(vec![write], Vec::new()), ("No result came back".into(), false));
}

fn client() -> steplet_run::Client {
    steplet_run::Client {
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

/// The live log makes a failed command stand out too, from the same exit code.
#[test]
fn the_live_log_marks_a_failed_command() {
    let input = json!({ "command": "make test" });
    let live = |tool: &str, is_error: bool, text: &str, refused_first: bool| {
        let e = Event::ToolResult { id: "u1".into(), is_error, text: text.into(), plain_end: !is_error };
        match event_view(&e, Some((tool, &input, refused_first))) {
            Some(steplet_app::views::EventView::Result { ended, failed, .. }) => (ended, failed),
            other => panic!("not a result: {other:?}"),
        }
    };
    assert_eq!(live("Bash", true, "Exit code 2\nfailed\n", false), ("Exit code 2".into(), true));
    assert_eq!(live("Bash", false, "ok\n", false), ("Exit code 0".into(), false));
    // Task 14 review: the live log and the report say the same, from one rule in Rust.
    assert_eq!(live("Write", false, "File created", false), ("Done".into(), false));
    assert_eq!(live("Write", true, "Permission denied", false), ("Ended with an error".into(), true));
    assert_eq!(
        live("Bash", true, "Steplet refused this: the receiver said no", true),
        ("It did not run".into(), false)
    );
}
