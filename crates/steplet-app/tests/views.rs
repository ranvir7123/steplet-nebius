//! What the screens get (Task 9): the preview says exactly what the core says, file text reaches the screens only as
//! display pieces, one entry never looks like two, and no view holds the receiver's key.

use std::path::PathBuf;

use serde_json::{Value, json};
use steplet_app::views::{Here, Seg, line, preview, report_view};
use steplet_core::report::check;
use steplet_run::{Answer, Event, Reply, RunOutcome, Trust, build_report};

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
