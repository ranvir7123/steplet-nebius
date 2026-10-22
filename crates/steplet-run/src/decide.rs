//! The one place a request from the agent becomes the checker's answer (row 29): each path of a file-tool request is
//! looked up where it really leads (row 30), the request is read as one of the checker's actions, and
//! `Policy::check` decides. Anything Steplet cannot read, or whose real place it cannot find, is refused, with a
//! reason.

use std::sync::Arc;

use serde_json::Value;
use steplet_core::policy::{Action, Decision, Policy};

use crate::places::RealPaths;
use crate::types::{GateAnswer, GateRequest};

const NOT_FOUND: &str = "Steplet could not find where this path leads, so it refused it";

/// The checker as a run uses it: the file's policy, deciding each path of the agent's file tools where it really
/// leads (`paths`), never as written.
#[derive(Clone)]
pub struct Checker {
    policy: Policy,
    paths: Arc<dyn RealPaths>,
}

impl Checker {
    pub fn new(policy: Policy, paths: Arc<dyn RealPaths>) -> Checker {
        Checker { policy, paths }
    }
}

/// The gate's answer to `req` when no receiver can be asked (an unattended run, the tests): an `Ask` from the checker
/// is a refusal that carries the checker's reason. A live run asks the receiver instead (`Asker::decide`). An allow
/// hands back the request as written.
pub fn decide(checker: &Checker, req: &GateRequest) -> GateAnswer {
    match check(checker, req) {
        Err(message) => deny(&message),
        Ok(c) => match c.decision {
            Decision::Allow => GateAnswer::Allow { updated_input: req.input.clone() },
            Decision::Deny { reason, .. } => deny(&c.noted(reason)),
            Decision::Ask { reason } => no_one_to_ask(&c.noted(reason)),
        },
    }
}

/// The checker's decision on a request, taken where its paths really lead.
pub(crate) struct Checked {
    pub(crate) decision: Decision,
    /// The request with each path replaced by where it really leads.
    pub(crate) real: GateRequest,
    /// "<path> leads to <place>", when the real place is not the path as written.
    leads: Option<String>,
}

impl Checked {
    /// `reason`, followed by where the path really leads when that is not where it is written.
    pub(crate) fn noted(&self, reason: &str) -> String {
        noted(reason, self.leads.as_deref())
    }
}

fn noted(reason: &str, leads: Option<&str>) -> String {
    match leads {
        Some(leads) => format!("{reason} ({leads})"),
        None => reason.into(),
    }
}

/// The checker's decision on `req` where its paths really lead, or why Steplet refuses it before asking the checker:
/// a tool it does not offer, a request it cannot read, or a path whose real place it cannot find.
pub(crate) fn check(checker: &Checker, req: &GateRequest) -> Result<Checked, String> {
    let (real, leads) = where_it_leads(&*checker.paths, req)?;
    match check_as_written(&checker.policy, &real) {
        Ok(decision) => Ok(Checked { decision, real, leads }),
        Err(message) => Err(noted(message, leads.as_deref())),
    }
}

/// `req` with the path of a file-tool request replaced by where it really leads, and "<path> leads to <place>" when
/// that differs. A path that is not absolute stays as written: the checker reads it as text, and refuses or asks.
fn where_it_leads(paths: &dyn RealPaths, req: &GateRequest) -> Result<(GateRequest, Option<String>), String> {
    let field = match req.tool_name.as_str() {
        "Read" | "Write" | "Edit" => "file_path",
        "Glob" | "Grep" => "path",
        _ => return Ok((req.clone(), None)),
    };
    let Some(written) = req.input.get(field).and_then(Value::as_str).filter(|p| p.starts_with('/')) else {
        return Ok((req.clone(), None));
    };
    let place = paths.real(written).map_err(|_| NOT_FOUND.to_owned())?;
    if place == written {
        return Ok((req.clone(), None));
    }
    let leads = format!("{written} leads to {place}");
    let mut real = req.clone();
    real.input[field] = Value::String(place);
    Ok((real, Some(leads)))
}

/// The checker's decision on `req`, its paths read as written, or why Steplet refuses it before asking the checker: a
/// tool it does not offer, or a request it cannot read.
fn check_as_written(policy: &Policy, req: &GateRequest) -> Result<Decision, &'static str> {
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
