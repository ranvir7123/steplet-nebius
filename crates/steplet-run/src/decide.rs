//! The one place a request from the agent becomes the checker's answer (row 29): the request is read as one of the
//! checker's actions, and `Policy::check` decides. Anything Steplet cannot read is refused, with a reason.

use serde_json::Value;
use steplet_core::policy::{Action, Decision, Policy};

use crate::types::{GateAnswer, GateRequest};

/// The gate's answer to `req` when no receiver can be asked (an unattended run, the tests): an `Ask` from the checker
/// is a refusal that carries the checker's reason. A live run asks the receiver instead (`Asker::decide`).
pub fn decide(policy: &Policy, req: &GateRequest) -> GateAnswer {
    match check(policy, req) {
        Err(message) => deny(message),
        Ok(Decision::Allow) => GateAnswer::Allow { updated_input: req.input.clone() },
        Ok(Decision::Deny { reason, .. }) => deny(reason),
        Ok(Decision::Ask { reason }) => no_one_to_ask(reason),
    }
}

/// The checker's decision on `req`, or why Steplet refuses it before asking the checker: a tool it does not offer, or
/// a request it cannot read.
pub(crate) fn check(policy: &Policy, req: &GateRequest) -> Result<Decision, &'static str> {
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
        _ => return Err("Steplet does not offer this tool to the agent"),
    };
    let Some(action) = action else { return Err("Steplet could not read this request, so it refused it") };
    Ok(policy.check(&action))
}

/// The refusal of a request that needs the receiver's yes when no one can answer.
pub(crate) fn no_one_to_ask(reason: &str) -> GateAnswer {
    deny(&format!("this needs the receiver's yes, and no one could answer: {reason}"))
}

/// True for a search pattern that starts at a root or a home folder, or climbs out with `..`. A choice such as
/// `{a,b}` counts too: a search may try every alternative, so each is checked as if it began the pattern. (A comma
/// outside a choice is read the same way, which can only refuse more.)
fn climbs_out(pattern: &str) -> bool {
    let starts_out = |p: &str| p.starts_with(['/', '\\', '~']);
    starts_out(pattern)
        || pattern.split(['{', ',']).skip(1).any(starts_out)
        || pattern.split(['/', '\\', '{', '}', ',']).any(|part| part == "..")
}

fn deny(message: &str) -> GateAnswer {
    GateAnswer::Deny { message: message.into() }
}
