mod support;

use std::sync::Arc;

use serde_json::json;
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{Checker, GateAnswer, GateRequest, decide};
use support::{CANNOT_LOOK, MadeUpLinks};

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
    let p = support::checker_as_written(policy());
    assert!(allowed(&decide(&p, &req("Bash", json!({"command": "cargo test"})))));
    assert!(
        !allowed(&decide(&p, &req("Bash", json!({"command": "cargo test --release"})))),
        "an Ask from the checker becomes a refusal with the reason, so a run with no receiver cannot allow it"
    );
    assert!(!allowed(&decide(&p, &req("WebFetch", json!({"url": "https://example.com"})))));
    assert!(allowed(&decide(&p, &req("Read", json!({"file_path": "/work/r1/repo/README.md"})))));
    assert!(allowed(&decide(&p, &req("Grep", json!({"pattern": "x", "path": "/work/r1/repo/src"})))));
    // A choice of endings (`{rs,toml}`) stays inside the folder.
    assert!(allowed(&decide(&p, &req("Glob", json!({"pattern": "**/*.{rs,toml}", "path": "/work/r1/repo"})))));
    assert!(!allowed(&decide(&p, &req("Write", json!({"file_path": "/work/r1/repo/.git/config", "content": "x"})))));
}

#[test]
fn an_allow_hands_back_exactly_the_request() {
    let p = support::checker_as_written(policy());
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
    let p = support::checker_as_written(policy());
    for (tool, input) in [
        ("WebSearch", json!({"query": "x"})),
        ("Bash", json!({})),
        ("Bash", json!({"command": 5})),
        ("Read", json!({"path": "/work/r1/repo/README.md"})),
        ("Glob", json!({"pattern": "*.rs"})),
        ("Glob", json!({"pattern": "/etc/**", "path": "/work/r1/repo"})),
        ("Glob", json!({"pattern": "../../**", "path": "/work/r1/repo"})),
        ("Grep", json!({"pattern": "x", "path": "/work/r1/repo", "glob": "/home/**"})),
        // A choice (`{a,b}`) whose alternative climbs out or starts at a root: a search may try every one.
        ("Glob", json!({"pattern": "{..,x}/*", "path": "/work/r1/repo"})),
        ("Glob", json!({"pattern": "{../../..,x}/etc/*", "path": "/work/r1/repo"})),
        ("Glob", json!({"pattern": "src/{x,/etc}/*", "path": "/work/r1/repo"})),
        ("Grep", json!({"pattern": "x", "path": "/work/r1/repo", "glob": "{a,~/.ssh}/*"})),
    ] {
        match decide(&p, &req(tool, input.clone())) {
            GateAnswer::Deny { message } => assert!(!message.is_empty(), "{tool} {input}: a refusal says why"),
            other => panic!("{tool} {input}: expected a refusal, got {other:?}"),
        }
    }
}

fn message(a: &GateAnswer) -> &str {
    match a {
        GateAnswer::Deny { message } => message,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

/// Phase 5 Task 3 (row 30): each path of the agent's file tools is decided where it really leads, following every
/// link, never as written. A read through a link gets exactly the answer a read of its real place gets, with a note
/// saying where it leads; a write through a link out of the copy, or into `.git`, is refused as such a write is. A
/// link that stays inside the copy is allowed, and the allow hands back the request as written. A path whose real
/// place Steplet cannot look up is refused.
#[test]
fn a_path_is_decided_where_it_really_leads() {
    let links = Checker::new(
        policy(),
        Arc::new(MadeUpLinks(vec![
            ("/work/r1/repo/notes/hostname.txt", "/etc/hostname"),
            ("/work/r1/repo/notes/tmp.txt", "/tmp/steplet-run-r1/note.txt"),
            ("/work/r1/repo/notes/run", "/var/lib/steplet/runs/r1"),
            ("/work/r1/repo/notes/readme", "/work/r1/repo/README.md"),
            ("/work/r1/repo/notes/git", "/work/r1/repo/.git"),
        ])),
    );
    let direct = decide(&links, &req("Read", json!({"file_path": "/etc/hostname"})));
    let through = decide(&links, &req("Read", json!({"file_path": "/work/r1/repo/notes/hostname.txt"})));
    assert_eq!(
        message(&direct),
        "this needs the receiver's yes, and no one could answer: this path is outside the repository"
    );
    assert_eq!(
        message(&through),
        format!("{} (/work/r1/repo/notes/hostname.txt leads to /etc/hostname)", message(&direct)),
        "the same answer as the file itself, saying where the path leads"
    );
    let outside = "this needs the receiver's yes, and no one could answer: this path is outside the repository";
    let j1 = "the agent's file tools may change files only inside the working copy";
    for (tool, input, why) in [
        ("Read", json!({"file_path": "/work/r1/repo/notes/tmp.txt"}), outside),
        ("Grep", json!({"pattern": "x", "path": "/work/r1/repo/notes/tmp.txt"}), outside),
        ("Glob", json!({"pattern": "**/*", "path": "/work/r1/repo/notes/run"}), outside),
        ("Read", json!({"file_path": "/work/r1/repo/notes/run/home/note.txt"}), outside),
        ("Write", json!({"file_path": "/work/r1/repo/notes/tmp.txt", "content": "x"}), j1),
        (
            "Edit",
            json!({"file_path": "/work/r1/repo/notes/run/config/a.txt", "old_string": "a", "new_string": "b"}),
            j1,
        ),
        (
            "Write",
            json!({"file_path": "/work/r1/repo/notes/git/config", "content": "x"}),
            "no task may change .git or the agents' settings",
        ),
    ] {
        let answer = decide(&links, &req(tool, input.clone()));
        assert!(message(&answer).starts_with(why), "{tool} {input}: {answer:?}");
        assert!(message(&answer).contains(" leads to "), "{tool} {input}: says where it leads: {answer:?}");
    }
    let inside = json!({"file_path": "/work/r1/repo/notes/readme"});
    assert_eq!(decide(&links, &req("Read", inside.clone())), GateAnswer::Allow { updated_input: inside });
    assert_eq!(
        message(&decide(&links, &req("Read", json!({"file_path": CANNOT_LOOK})))),
        "Steplet could not find where this path leads, so it refused it"
    );
}
