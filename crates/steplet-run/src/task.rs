//! What the agent is told (phase 4 Task 2): one fixed form, built from the file. It lists the allowed commands word
//! for word, so the agent's commands match them more often (G1), and gives the sender's body exactly as signed.

use steplet_core::header::{Scalar, Steplet};

/// The task text Claude Code is given for a run of `s` in a working copy at `commit`. The sender's body comes last and
/// exactly as signed, so text in it may look like the headings above (by design: it is the sender's own task), but
/// nothing of Steplet's own comes after it.
pub fn task_text(s: &Steplet, commit: &str) -> String {
    let mut t = String::from(
        "You are doing a task that someone sent as a .steplet file. Steplet, the app running you, checks every action \
         before it happens.\n\n",
    );
    t.push_str(&format!("The repository is checked out in the current folder at commit {commit}.\n\nSteps:\n"));
    for (n, step) in s.steps.iter().enumerate() {
        t.push_str(&format!("{}. {}\n", n + 1, step.text));
    }
    t.push('\n');
    if s.allow.commands.is_empty() {
        t.push_str("The file allows no commands. Any command that changes something needs the receiver's yes.\n");
    } else {
        t.push_str(
            "Commands the file allows, exactly as written (any other command that changes something needs the \
             receiver's yes):\n",
        );
        t.push_str(&list(&s.allow.commands));
    }
    t.push('\n');
    if s.allow.domains.is_empty() {
        t.push_str("Web sites the file allows: none.\n");
    } else {
        t.push_str("Web sites the file allows:\n");
        t.push_str(&list(&s.allow.domains));
    }
    t.push('\n');
    if s.never.is_empty() {
        t.push_str("Never: nothing listed.\n");
    } else {
        t.push_str("Never:\n");
        t.push_str(&list(&s.never));
    }
    t.push('\n');
    if s.body.is_empty() {
        t.push_str("The sender's notes: none.\n");
    } else {
        t.push_str("The sender's notes:\n");
        t.push_str(&s.body);
    }
    t
}

fn list(items: &[Scalar]) -> String {
    items.iter().map(|i| format!("- {}\n", i.text)).collect()
}
