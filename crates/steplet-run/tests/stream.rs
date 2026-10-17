//! Reading Claude Code's event stream into Steplet's events (plan Task 7). `support/recorded_stream.jsonl` holds raw
//! lines copied unchanged from phase 0's recorded runs, Claude Code 2.1.220: lines 1 to 6 from `L1.jsonl` (the
//! real-model run), 7 from `S1b.jsonl`, 8 to 10 from `S1t.jsonl` (a run stopped mid-command), 11 from
//! `S6R-run1.jsonl` and 12 from `S6R-off-run1.jsonl` (the model refusals). The key's text appears in it 0 times.

use serde_json::json;
use steplet_run::{Event, parse_line};

const RECORDED: &str = include_str!("support/recorded_stream.jsonl");

fn line(n: usize) -> &'static str {
    RECORDED.lines().nth(n - 1).expect("the fixture has this line")
}

#[test]
fn the_start_names_the_model_the_tools_and_whether_the_gate_connected() {
    match parse_line(line(1)).as_slice() {
        [Event::Started { model, tools, gate_connected }] => {
            assert_eq!(model, "claude-opus-5[1m]");
            assert_eq!(tools.len(), 28);
            assert!(tools.iter().any(|t| t == "Bash"));
            assert!(*gate_connected);
        }
        other => panic!("{other:?}"),
    }
    let failed = r#"{"type":"system","subtype":"init","model":"m","tools":["Bash"],"mcp_servers":[{"name":"gate","status":"failed"}]}"#;
    assert!(matches!(parse_line(failed).as_slice(), [Event::Started { gate_connected: false, .. }]));
    let missing = r#"{"type":"system","subtype":"init","model":"m","tools":[],"mcp_servers":[]}"#;
    assert!(matches!(parse_line(missing).as_slice(), [Event::Started { gate_connected: false, .. }]));
}

#[test]
fn the_agent_s_words_and_tool_calls() {
    assert_eq!(parse_line(line(2)), vec![], "the model's thinking is not an event");
    assert_eq!(parse_line(line(3)), vec![Event::AgentText("I'll check out the commit, then run the tests.".into())]);
    assert_eq!(
        parse_line(line(4)),
        vec![Event::ToolUse {
            id: "toolu_01R1W54jVZG5BorCJAazSih8".into(),
            name: "Bash".into(),
            input: json!({
                "command": "git -C /home/tester/steplet-work/L1/repo checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"",
                "description": "Check out the specified commit"
            }),
        }]
    );
    // Every kind of tool call is a tool call, and a message is read by its type alone, whatever else it carries:
    // a call Steplet does not see is a call Task 9's tripwires cannot check.
    for (line, name) in [
        (
            r#"{"type":"assistant","subtype":"x","message":{"content":[{"type":"tool_use","id":"t1","name":"Bash","input":{}}]}}"#,
            "Bash",
        ),
        (
            r#"{"type":"assistant","message":{"content":[{"type":"server_tool_use","id":"t1","name":"web_search","input":{}}]}}"#,
            "web_search",
        ),
        (
            r#"{"type":"assistant","message":{"content":[{"type":"mcp_tool_use","id":"t1","name":"other","input":{}}]}}"#,
            "other",
        ),
    ] {
        assert!(matches!(parse_line(line).as_slice(), [Event::ToolUse { name: n, .. }] if n == name), "{line}");
    }
    let subtyped = r#"{"type":"user","subtype":"x","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"r"}]}}"#;
    assert!(matches!(parse_line(subtyped).as_slice(), [Event::ToolResult { .. }]), "{subtyped}");
    let two = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"a"},{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"/x"}}]}}"#;
    assert_eq!(
        parse_line(two),
        vec![
            Event::AgentText("a".into()),
            Event::ToolUse { id: "t1".into(), name: "Read".into(), input: json!({"file_path": "/x"}) }
        ],
        "every block of a message, in order"
    );
}

#[test]
fn tool_results_say_whether_they_failed() {
    assert_eq!(
        parse_line(line(5)),
        vec![Event::ToolResult {
            id: "toolu_01R1W54jVZG5BorCJAazSih8".into(),
            is_error: false,
            text: "fatal: Unable to create '/home/tester/steplet-work/L1/repo/.git/index.lock': Read-only file system\nEXIT=128"
                .into(),
            plain_end: true,
        }]
    );
    assert_eq!(
        parse_line(line(7)),
        vec![Event::ToolResult {
            id: "toolu_spike_7194c2d2c7104b749582".into(),
            is_error: true,
            text: "Steplet's checker refused: no answer".into(),
            plain_end: false,
        }]
    );
    let blocks = r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":[{"type":"text","text":"a"},{"type":"image","source":{}},{"type":"text","text":"b"}]}]}}"#;
    assert_eq!(
        parse_line(blocks),
        vec![Event::ToolResult { id: "t1".into(), is_error: false, text: "a\nb".into(), plain_end: false }],
        "text blocks joined, other blocks left out, a missing is_error read as false, no record of the end"
    );
}

#[test]
fn the_end_of_a_run_as_claude_code_reports_it() {
    match parse_line(line(6)).as_slice() {
        [Event::Result { subtype, is_error, stop_reason, models, text }] => {
            assert_eq!((subtype.as_str(), *is_error, stop_reason.as_str()), ("success", false, "end_turn"));
            assert_eq!(models, &["claude-opus-5[1m]"]);
            assert!(text.starts_with("Checkout: failed"), "{text}");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        parse_line(line(10)),
        vec![Event::Result {
            subtype: "error_during_execution".into(),
            is_error: true,
            stop_reason: "tool_use".into(),
            models: vec!["claude-opus-5[1m]".into()],
            text: String::new(),
        }]
    );
}

#[test]
fn a_refusal_shows_whether_the_model_switched() {
    match parse_line(line(11)).as_slice() {
        [Event::ModelSwitch { from, to, category, explanation }] => {
            assert_eq!(
                (from.as_str(), to.as_deref(), category.as_str()),
                ("claude-opus-5[1m]", Some("claude-opus-4-8"), "cyber")
            );
            assert!(explanation.starts_with("This request triggered restrictions"), "{explanation}");
        }
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        parse_line(line(12)).as_slice(),
        [Event::ModelSwitch { to: None, category, .. }] if category == "cyber"
    ));
}

#[test]
fn every_hook_event_is_seen() {
    // No recorded 2.1.220 stream holds a hook event. These follow milestone 0's field names (logs/T3a_repo_files.txt);
    // Task 9's live hook test checks the real shape. Anything naming a hook counts, so a new form stops the run too.
    for (hook, name) in [
        (
            r#"{"type":"system","subtype":"hook_started","hook_id":"h1","hook_name":"SessionStart:startup","hook_event":"SessionStart"}"#,
            "SessionStart:startup",
        ),
        (
            r#"{"type":"system","subtype":"hook_response","hook_name":"PreToolUse:Bash","exit_code":0}"#,
            "PreToolUse:Bash",
        ),
        (r#"{"type":"system","subtype":"hook_progress"}"#, "system/hook_progress"),
        (r#"{"type":"hook_event","hook_name":"Stop"}"#, "Stop"),
    ] {
        assert_eq!(parse_line(hook), vec![Event::HookEvent { name: name.into() }], "{hook}");
    }
}

#[test]
fn nothing_is_dropped_and_nothing_panics() {
    assert_eq!(parse_line(line(8)), vec![Event::Other("user/text".into())]);
    assert_eq!(parse_line(line(9)), vec![Event::Other("system/task_notification".into())]);
    assert_eq!(
        parse_line(r#"{"type":"assistant","message":{"content":"plain"}}"#),
        vec![Event::Other("assistant".into())]
    );
    assert_eq!(parse_line(""), vec![], "a blank line");
    for odd in ["not json", "[1, 2]", "\"text\"", "{\"type\": 5}", "{", "\u{1b}[31mred\u{1b}[0m"] {
        let got = parse_line(odd);
        assert!(matches!(got.as_slice(), [Event::Unreadable(_)] | [Event::Other(_)]), "{odd:?}: {got:?}");
    }
    let every = RECORDED.lines().flat_map(parse_line).count();
    assert_eq!(every, 11, "12 lines, of which only the thinking one gives no event");
}
