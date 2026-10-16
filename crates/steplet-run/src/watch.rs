//! The tripwire watcher (plan Task 9): it judges each event of a run and says when the run must stop (D3, D8, row
//! 38). The rule comes from what Claude Code 2.1.220 was measured to do with Steplet's settings (`tests/tripwires.rs`,
//! `which_actions_ask_the_gate`): harmless reads run without asking the gate; whatever changes something asks it.
//!
//! So a tool call is judged when its result arrives. By then the gate's record of it exists, because each record is
//! written before its answer is sent.
//! - A call the gate allowed is fine. One it refused must come back as an error; otherwise it ran although Steplet
//!   refused it.
//! - A call that skipped the gate is fine only if it was a harmless read: a command the command reader reads as a
//!   read-only one inside the working copy (G2, D8, E2), or a Read, Glob or Grep the checker would allow. Anything
//!   else stops the run. That includes chained commands (`;`, `|`), which the command reader does not read at all
//!   (R8, G1), and calls Claude Code refused by Steplet's own deny rules: Steplet cannot tell from the stream whether
//!   a skipped command ran, since its output could imitate a refusal.
//! - Any hook event stops the run (D3).

use std::collections::HashMap;

use serde_json::Value;
use steplet_core::command::{self, Assessment, Kind};
use steplet_core::policy::Policy;

use crate::decide::decide;
use crate::types::{Event, GateAnswer, GateRequest};

/// Why Steplet stopped a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tripwire {
    /// A hook ran (D3); the hook's name as Claude Code gave it.
    Hook(String),
    /// A command ran without asking the gate, and is not a harmless read (D8, E2); the command.
    SkippedCommand(String),
    /// Another action did not go through the gate, or ran although the gate refused it (row 38); what it was.
    SkippedGate(String),
}

/// What the watcher made of one event.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Verdict {
    /// The record of a tool call that skipped the gate (`Event::SkippedGate`), for the run's events.
    pub skipped: Option<Event>,
    /// Set when the run must stop.
    pub stop: Option<Tripwire>,
}

/// Judges the events of one run, in order.
pub struct Watcher<'a> {
    policy: &'a Policy,
    work: String,
    /// The tool calls seen whose results have not come yet: id, then tool and input.
    calls: HashMap<String, (String, Value)>,
}

impl<'a> Watcher<'a> {
    /// A watcher for a run that `policy` checks, in the working copy `work`.
    pub fn new(policy: &'a Policy, work: &str) -> Watcher<'a> {
        Watcher { policy, work: work.into(), calls: HashMap::new() }
    }

    /// Judges `event`. `gate` says what Steplet answered the gate about a tool call id: `None` if the gate never
    /// asked about it, `Some(true)` if any answer was "allow", `Some(false)` if every one was "deny".
    pub fn see(&mut self, event: &Event, gate: &dyn Fn(&str) -> Option<bool>) -> Verdict {
        match event {
            Event::HookEvent { name } => stop(Tripwire::Hook(name.clone())),
            Event::ToolUse { id, name, input } => {
                self.calls.insert(id.clone(), (name.clone(), input.clone()));
                Verdict::default()
            }
            Event::ToolResult { id, is_error, .. } => self.judge(id, *is_error, gate(id)),
            _ => Verdict::default(),
        }
    }

    fn judge(&mut self, id: &str, is_error: bool, answered: Option<bool>) -> Verdict {
        let Some((tool, mut input)) = self.calls.remove(id) else {
            return stop(Tripwire::SkippedGate("a result for a tool call Steplet did not see".into()));
        };
        // A search with no folder searches Claude Code's working folder, which is the working copy.
        if matches!(tool.as_str(), "Glob" | "Grep")
            && let Some(fields) = input.as_object_mut()
            && fields.get("path").is_none_or(Value::is_null)
        {
            fields.insert("path".into(), Value::String(self.work.clone()));
        }
        let what = what(&tool, &input);
        match answered {
            Some(true) => Verdict::default(),
            Some(false) if is_error => Verdict::default(),
            Some(false) => stop(Tripwire::SkippedGate(described(&tool, &what))),
            None => {
                let harmless = self.harmless(&tool, &input);
                let tripwire = match tool.as_str() {
                    _ if harmless => None,
                    "Bash" => Some(Tripwire::SkippedCommand(what.clone())),
                    _ => Some(Tripwire::SkippedGate(described(&tool, &what))),
                };
                Verdict { skipped: Some(Event::SkippedGate { tool, what, harmless }), stop: tripwire }
            }
        }
    }

    /// True for a call that may run without asking the gate: a harmless read inside the working copy (G2).
    fn harmless(&self, tool: &str, input: &Value) -> bool {
        match tool {
            "Bash" => matches!(
                command::assess(text(input, "command")),
                Assessment::Words { kind: Kind::ReadOnly, outside: false, .. }
            ),
            // The checker's own answer, as the gate would have given it (row 29).
            "Read" | "Glob" | "Grep" => {
                let request = GateRequest {
                    run_token: String::new(),
                    tool_name: tool.into(),
                    input: input.clone(),
                    tool_use_id: String::new(),
                };
                matches!(decide(self.policy, &request), GateAnswer::Allow { .. })
            }
            _ => false,
        }
    }
}

fn stop(why: Tripwire) -> Verdict {
    Verdict { skipped: None, stop: Some(why) }
}

/// The part of a call a person needs to see: its command, path or address, or a search's pattern and folder.
fn what(tool: &str, input: &Value) -> String {
    match tool {
        "Bash" => text(input, "command").into(),
        "Read" | "Write" | "Edit" => text(input, "file_path").into(),
        "WebFetch" => text(input, "url").into(),
        "Glob" | "Grep" => format!("{} in {}", text(input, "pattern"), text(input, "path")),
        _ => String::new(),
    }
}

/// A call in a few words: the tool, then what it was.
fn described(tool: &str, what: &str) -> String {
    format!("{tool} {what}").trim_end().into()
}

fn text<'a>(input: &'a Value, field: &str) -> &'a str {
    input.get(field).and_then(Value::as_str).unwrap_or("")
}
