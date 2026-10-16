use serde_json::json;
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{GateAnswer, GateRequest, decide};

fn policy() -> Policy {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n    - cargo test\n---\n";
    Policy::from_steplet(&parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(), "/work/r1/repo").unwrap()
}

fn req(tool: &str, input: serde_json::Value) -> GateRequest {
    GateRequest { run_token: "t".into(), tool_name: tool.into(), input, tool_use_id: "u1".into() }
}

fn allowed(a: &GateAnswer) -> bool {
    matches!(a, GateAnswer::Allow { .. })
}

#[test]
fn maps_requests_to_the_checker() {
    let p = policy();
    assert!(allowed(&decide(&p, &req("Bash", json!({"command": "cargo test"})))));
    assert!(
        !allowed(&decide(&p, &req("Bash", json!({"command": "cargo test --release"})))),
        "an Ask from the checker becomes a refusal with the reason, so a run with no receiver cannot allow it"
    );
    assert!(!allowed(&decide(&p, &req("WebFetch", json!({"url": "https://example.com"})))));
    assert!(allowed(&decide(&p, &req("Read", json!({"file_path": "/work/r1/repo/README.md"})))));
    assert!(allowed(&decide(&p, &req("Grep", json!({"pattern": "x", "path": "/work/r1/repo/src"})))));
    assert!(!allowed(&decide(&p, &req("Write", json!({"file_path": "/work/r1/repo/.git/config", "content": "x"})))));
}

#[test]
fn an_allow_hands_back_exactly_the_request() {
    let p = policy();
    let input = json!({"command": "cargo test", "description": "Run the tests"});
    match decide(&p, &req("Bash", input.clone())) {
        GateAnswer::Allow { updated_input } => {
            assert_eq!(updated_input, input, "R7: the approval is bound to the exact request")
        }
        other => panic!("expected an allow, got {other:?}"),
    }
}

#[test]
fn anything_unclear_is_refused_with_a_reason() {
    let p = policy();
    for (tool, input) in [
        ("WebSearch", json!({"query": "x"})),
        ("Bash", json!({})),
        ("Bash", json!({"command": 5})),
        ("Read", json!({"path": "/work/r1/repo/README.md"})),
        ("Glob", json!({"pattern": "*.rs"})),
        ("Glob", json!({"pattern": "/etc/**", "path": "/work/r1/repo"})),
        ("Glob", json!({"pattern": "../../**", "path": "/work/r1/repo"})),
        ("Grep", json!({"pattern": "x", "path": "/work/r1/repo", "glob": "/home/**"})),
    ] {
        match decide(&p, &req(tool, input.clone())) {
            GateAnswer::Deny { message } => assert!(!message.is_empty(), "{tool} {input}: a refusal says why"),
            other => panic!("{tool} {input}: expected a refusal, got {other:?}"),
        }
    }
}
