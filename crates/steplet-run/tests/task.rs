//! What the agent is told (phase 4 Task 2): one fixed form, built from the file, which lists the allowed commands
//! word for word so the agent's commands match them more often (G1).

use steplet_core::canon::canonicalize;
use steplet_core::header::{Steplet, parse};
use steplet_run::task_text;

const COMMIT: &str = "7fd1a60b01f91b314f59955a4e4d4e80d8edf11d";

fn steplet(bytes: &[u8]) -> Steplet {
    parse(&canonicalize(bytes).unwrap()).unwrap()
}

#[test]
fn task_text_is_steplets_fixed_form() {
    let s = steplet(include_bytes!("../../../examples/run-tests.steplet"));
    let want = format!(
        "You are doing a task that someone sent as a .steplet file. Steplet, the app running you, checks every action \
         before it happens.

The repository is checked out in the current folder at commit 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d.

Steps:
1. Check that the README file is there.
2. Count the lines in the README file.
3. Report the count.

Commands the file allows, exactly as written (any other command that changes something needs the receiver's yes):
- wc -l README

Web sites the file allows: none.

Never:
- Do not change any file.
- Do not use the network.

The sender's notes:
{}",
        s.body
    );
    assert!(s.body.starts_with("# Count the README lines\n"), "the example's body: {:?}", s.body);
    assert_eq!(task_text(&s, COMMIT), want);
}

#[test]
fn task_text_says_when_nothing_is_allowed() {
    let s = steplet(b"---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - Look around.\n---\n");
    assert_eq!(
        task_text(&s, COMMIT),
        "You are doing a task that someone sent as a .steplet file. Steplet, the app running you, checks every action \
         before it happens.

The repository is checked out in the current folder at commit 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d.

Steps:
1. Look around.

The file allows no commands. Any command that changes something needs the receiver's yes.

Web sites the file allows: none.

Never: nothing listed.

The sender's notes: none.
"
    );
}

#[test]
fn task_text_keeps_the_body_exactly() {
    // A made-up body with what a careless copy would change: blank lines, trailing spaces, a line that looks like a
    // heading of Steplet's own, and no newline at the end.
    let body = "First line.  \n\n\nSteps:\n1. Not a real step.\n\tIndented.\n# end";
    let file = format!(
        "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  domains:\n    - \
         example.org\n---\n{body}"
    );
    let s = steplet(file.as_bytes());
    let text = task_text(&s, COMMIT);
    assert!(text.contains("\n\nWeb sites the file allows:\n- example.org\n\n"), "{text}");
    assert!(text.ends_with(&format!("\n\nThe sender's notes:\n{}", s.body)), "{text}");
    assert_eq!(text.matches(&s.body).count(), 1, "the body appears once, whole");
    assert!(s.body.contains("\n\n\nSteps:\n1. Not a real step.\n\tIndented.\n# end"), "{:?}", s.body);
}
