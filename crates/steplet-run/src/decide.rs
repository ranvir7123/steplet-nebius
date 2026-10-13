//! The one place a request from the agent becomes the checker's answer (row 29): the request is read as one of the
//! checker's actions, and `Policy::check` decides. Anything Steplet cannot read is refused, with a reason.

use serde_json::Value;
use steplet_core::policy::{Action, Decision, Policy};

use crate::types::{GateAnswer, GateRequest};

/// The gate's answer to `req`. With no receiver to ask (the test harness now, an unattended run later), an `Ask` from
/// the checker is a refusal that carries the checker's reason; phase 4's ask screen turns it into a question.
pub fn decide(policy: &Policy, req: &GateRequest) -> GateAnswer {
    let text = |field: &str| req.input.get(field).and_then(Value::as_str);
    let action = match req.tool_name.as_str() {
        "Bash" => text("command").map(Action::Command),
        "Read" => text("file_path").map(Action::Read),
        // A search names a folder, and a pattern that could reach outside it.
        "Glob" if text("pattern").is_some_and(climbs_out) => None,
        "Grep" if text("glob").is_some_and(climbs_out) => None,
        "Glob" | "Grep" => text("path").map(Action::Read),
        "Write" | "Edit" => text("file_path").map(Action::Write),
        "WebFetch" => text("url").map(Action::Fetch),
        _ => return deny("Steplet does not offer this tool to the agent"),
    };
    let Some(action) = action else { return deny("Steplet could not read this request, so it refused it") };
    match policy.check(&action) {
        Decision::Allow => GateAnswer::Allow { updated_input: req.input.clone() },
        Decision::Deny { reason, .. } => deny(reason),
        Decision::Ask { reason } => deny(&format!("this needs the receiver's yes, and no one could answer: {reason}")),
    }
}

/// True for a search pattern that starts at a root or a home folder, or climbs out with `..`.
fn climbs_out(pattern: &str) -> bool {
    pattern.starts_with(['/', '\\', '~']) || pattern.split(['/', '\\']).any(|part| part == "..")
}

fn deny(message: &str) -> GateAnswer {
    GateAnswer::Deny { message: message.into() }
}
