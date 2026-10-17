//! The run's report (plan Task 10): what the receiver reads after a run, built only from Steplet's own records. The
//! task came from a stranger, so the report must stay true and safe to read whatever the task printed:
//! - whether the run finished comes from the run itself, never from what the agent says or from `subtype` alone
//!   (F35, E1);
//! - every model used and every refusal is listed (D7, E1);
//! - each tool call shows Steplet's check, and each command its exit code as Claude Code marked it;
//! - the changed files come from Steplet's own `git status` after the run, and the trust check from the run's own
//!   config (D3);
//! - the agent's words are shown only as `agent_says` (F47);
//! - every text field keeps only characters that show (T5b), with the receiver's key and the run's token hidden (F48).

use std::collections::HashMap;

use serde_json::Value;
use steplet_core::display::{is_invisible, is_space_like};

use crate::gate::Asked;
use crate::run::RunOutcome;
use crate::types::{Event, GateAnswer};
use crate::watch::{Tripwire, described, what};

/// What a secret shows as in the report.
const HIDDEN: &str = "[hidden by Steplet]";

/// Phase 0 S6: the fence stops what it blocks without telling Steplet.
const FENCE_NOTE: &str = "Blocked fence attempts are not visible to Steplet: the fence stops a program without telling \
                          Steplet, so this report never says the fence stopped a particular thing.";

/// What the receiver is shown after a run.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    /// The run ended on its own and without an error, by Steplet's records: exactly one result, with `success`, no
    /// error and `end_turn`; no tripwire; not stopped at its time limit; no fault; Claude Code's exit code 0. It says
    /// nothing about whether the task's own tests passed: their exit codes are in `commands`.
    pub completed: bool,
    /// Why Steplet stopped the run, if a tripwire did (D3, D8, row 38).
    pub stopped_by: Option<Tripwire>,
    /// Every tool call the agent made, in order.
    pub commands: Vec<ReportCommand>,
    /// Everything the gate refused, with Steplet's reason.
    pub refused: Vec<String>,
    /// Every model used, each once, in the order first seen (D7).
    pub models: Vec<String>,
    /// Each time a model refused (E1).
    pub refusals: Vec<Refusal>,
    /// The working copy's changes, from Steplet's own `git status` (two status letters, then the path). `None` when
    /// Steplet could not look; `faults` then says why.
    pub changed_files: Option<Vec<String>>,
    /// Whether the run's Claude Code config recorded trust for a folder (D3).
    pub trust: Trust,
    /// The agent's own summary, shown last and only as "the agent says" (F47).
    pub agent_says: String,
    /// What went wrong around the run, in Steplet's words.
    pub faults: Vec<String>,
    /// Lines every report carries.
    pub notes: Vec<&'static str>,
}

/// One tool call, as the report shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportCommand {
    pub tool: String,
    /// The command, path, address or search.
    pub command: String,
    pub check: Check,
    /// For a command: the exit code of its whole command line as Claude Code marked it (a line such as
    /// `make test; true` ends in 0 whatever the tests did); `None` when it did not run, has not ended, or Claude Code
    /// did not say or gave the exit code a meaning of its own. Other tools have none.
    pub exit_code: Option<i32>,
    /// What came back, as Claude Code gave it.
    pub output: String,
    /// Steplet was already stopping the run before this call's result came (or none came): what it shows was cut
    /// short or refused by the stop, and it is not a breach of its own.
    pub after_stop: bool,
}

/// What Steplet did with one tool call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    /// The gate asked Steplet, and Steplet allowed it.
    Allowed,
    /// The gate asked Steplet, and Steplet refused it.
    Refused,
    /// Steplet refused it at the gate, yet its result came back with no error: it ran anyway. That stops the run
    /// (Task 9), and it is a breach of its own even while the run is being stopped.
    RefusedButRan,
    /// It did not ask the gate. `harmless`: it was a harmless read (G2); otherwise it stopped the run.
    NotAsked { harmless: bool },
    /// No result came, so there is no evidence it ran (Task 9).
    NotJudged,
}

/// A model refused (E1). `switched_to` is the model Claude Code then used, if it switched (D7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub model: String,
    pub switched_to: Option<String>,
    pub category: String,
    pub explanation: String,
}

/// The trust check after a run (D3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trust {
    NotRecorded,
    /// The run's config marks a folder as trusted: something got past Steplet's settings.
    Recorded,
    /// Steplet could not read the run's config.
    Unknown,
}

/// The report of `outcome`. `changed` is Steplet's own list of the working copy's changes (`None` if it could not
/// look), `trust` the trust check, and `secrets` the texts no field may show (the key, the run's token).
pub fn build_report(outcome: &RunOutcome, changed: Option<&[String]>, trust: Trust, secrets: &[&str]) -> Report {
    let scrub = Scrubber::new(secrets);
    let results: Vec<(&str, bool, &str)> = outcome
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Result { subtype, is_error, stop_reason, .. } => {
                Some((subtype.as_str(), *is_error, stop_reason.as_str()))
            }
            _ => None,
        })
        .collect();
    let completed = matches!(results[..], [("success", false, "end_turn")])
        && outcome.stopped_by.is_none()
        && outcome.stopped_at.is_none()
        && outcome.faults.is_empty()
        && outcome.exit_code == Some(0);
    Report {
        completed,
        stopped_by: outcome.stopped_by.as_ref().map(|t| scrub.tripwire(t)),
        commands: commands(outcome, &scrub),
        refused: refused(outcome, &scrub),
        models: models(&outcome.events, &scrub),
        refusals: outcome
            .events
            .iter()
            .filter_map(|e| match e {
                Event::ModelSwitch { from, to, category, explanation } => Some(Refusal {
                    model: scrub.text(from),
                    switched_to: to.as_deref().map(|t| scrub.text(t)),
                    category: scrub.text(category),
                    explanation: scrub.text(explanation),
                }),
                _ => None,
            })
            .collect(),
        changed_files: changed.map(|files| files.iter().map(|f| scrub.text(f)).collect()),
        trust,
        agent_says: scrub.text(agent_says(&outcome.events)),
        faults: outcome.faults.iter().map(|f| scrub.text(f)).collect(),
        notes: vec![FENCE_NOTE],
    }
}

/// The changes listed by `git status --porcelain=v1 -z --no-renames`: one entry per file, each its two status letters,
/// a space and the path, separated by NUL characters (the paths are not quoted).
pub fn changed_files(porcelain_z: &str) -> Vec<String> {
    porcelain_z.split('\0').filter(|entry| !entry.is_empty()).map(str::to_owned).collect()
}

/// The trust check (D3) of a run's Claude Code config file, `.claude.json`: `Recorded` if any folder in its
/// `projects` is marked `hasTrustDialogAccepted: true`. A file that is missing, not JSON or of an unknown shape gives
/// `Unknown`.
pub fn read_trust(claude_json: Option<&str>) -> Trust {
    let Some(Ok(Value::Object(config))) = claude_json.map(serde_json::from_str::<Value>) else { return Trust::Unknown };
    match config.get("projects") {
        None => Trust::NotRecorded,
        Some(Value::Object(projects)) => {
            if projects.values().any(|p| p.get("hasTrustDialogAccepted") == Some(&Value::Bool(true))) {
                Trust::Recorded
            } else {
                Trust::NotRecorded
            }
        }
        Some(_) => Trust::Unknown,
    }
}

/// Makes a text safe to show: only characters that show are kept, and the secrets are hidden.
pub(crate) struct Scrubber {
    /// Cleaned, longest first, none empty.
    secrets: Vec<String>,
}

impl Scrubber {
    pub(crate) fn new(secrets: &[&str]) -> Scrubber {
        let mut secrets: Vec<String> = secrets.iter().map(|s| shown(s)).filter(|s| !s.is_empty()).collect();
        secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
        Scrubber { secrets }
    }

    /// `text` with invisible and control characters removed (newline and tab stay), characters that look like a
    /// space made plain spaces, and then each secret hidden. Cleaning comes first, so a secret with an invisible
    /// character inside it is still found.
    pub(crate) fn text(&self, text: &str) -> String {
        self.secrets.iter().fold(shown(text), |text, secret| text.replace(secret.as_str(), HIDDEN))
    }

    fn tripwire(&self, t: &Tripwire) -> Tripwire {
        match t {
            Tripwire::Hook(name) => Tripwire::Hook(self.text(name)),
            Tripwire::SkippedCommand(command) => Tripwire::SkippedCommand(self.text(command)),
            Tripwire::SkippedGate(what) => Tripwire::SkippedGate(self.text(what)),
        }
    }
}

/// Only the characters of `text` that show (T5b).
fn shown(text: &str) -> String {
    text.chars()
        .filter_map(|c| match c {
            c if is_space_like(c) => Some(' '),
            c if is_invisible(c) => None,
            c => Some(c),
        })
        .collect()
}

/// Every tool call, with Steplet's check, its result and whether the stop shaped it.
fn commands(outcome: &RunOutcome, scrub: &Scrubber) -> Vec<ReportCommand> {
    let mut answered: HashMap<&str, bool> = HashMap::new();
    for asked in &outcome.requests {
        if let Asked::Decision { request, answer, .. } = asked {
            let allowed = answered.entry(request.tool_use_id.as_str()).or_insert(false);
            *allowed |= matches!(answer, GateAnswer::Allow { .. });
        }
    }
    let results = results(&outcome.events);
    let mut skipped: HashMap<&str, bool> = HashMap::new();
    for e in &outcome.events {
        if let Event::SkippedGate { id, harmless, .. } = e {
            skipped.insert(id.as_str(), *harmless);
        }
    }
    let stop = outcome.stopped_at;
    outcome
        .events
        .iter()
        .filter_map(|e| {
            let Event::ToolUse { id, name, input } = e else { return None };
            let result = results.get(id.as_str());
            let check = match (answered.get(id.as_str()), skipped.get(id.as_str())) {
                (Some(true), _) => Check::Allowed,
                (Some(false), _) if result.is_some_and(|r| !r.is_error) => Check::RefusedButRan,
                (Some(false), _) => Check::Refused,
                (None, Some(&harmless)) => Check::NotAsked { harmless },
                (None, None) => Check::NotJudged,
            };
            let exit_code = match result {
                Some(r) if name == "Bash" && input["run_in_background"] != Value::Bool(true) => exit_code(r),
                _ => None,
            };
            // A call that ran although Steplet refused it is a breach of its own, stop or not.
            let after_stop =
                check != Check::RefusedButRan && stop.is_some_and(|stop| result.is_none_or(|r| r.event >= stop));
            Some(ReportCommand {
                tool: scrub.text(name),
                command: scrub.text(&what(name, input)),
                check,
                exit_code,
                output: result.map_or(String::new(), |r| scrub.text(r.text)),
                after_stop,
            })
        })
        .collect()
}

/// One tool call's result: where it is among the events, and what Claude Code marked on it.
struct Answer<'a> {
    event: usize,
    is_error: bool,
    plain_end: bool,
    text: &'a str,
}

/// The first result for each tool call id.
fn results(events: &[Event]) -> HashMap<&str, Answer<'_>> {
    let mut results = HashMap::new();
    for (event, e) in events.iter().enumerate() {
        if let Event::ToolResult { id, is_error, text, plain_end } = e {
            results.entry(id.as_str()).or_insert(Answer {
                event,
                is_error: *is_error,
                plain_end: *plain_end,
                text: text.as_str(),
            });
        }
    }
    results
}

/// A command's exit code from Claude Code's own marks, never from what the program printed: no error with a plain end
/// in Claude Code's record means 0; with an error, Claude Code's first line `Exit code N` gives N. Anything else (a
/// refusal, an interruption, an exit code Claude Code gave a meaning of its own) has none.
fn exit_code(r: &Answer) -> Option<i32> {
    if !r.is_error {
        return r.plain_end.then_some(0);
    }
    r.text.lines().next()?.strip_prefix("Exit code ")?.parse().ok()
}

/// Everything the gate refused: each refused decision, marked if the call ran anyway, and each connection refused
/// before any decision.
fn refused(outcome: &RunOutcome, scrub: &Scrubber) -> Vec<String> {
    let results = results(&outcome.events);
    outcome
        .requests
        .iter()
        .filter_map(|asked| match asked {
            Asked::Decision { request, answer: GateAnswer::Deny { message }, .. } => {
                let call = described(&request.tool_name, &what(&request.tool_name, &request.input));
                let ran = results.get(request.tool_use_id.as_str()).is_some_and(|r| !r.is_error);
                let ran = if ran { " (it ran anyway)" } else { "" };
                Some(scrub.text(&format!("{call}: {message}{ran}")))
            }
            Asked::Refused { reason, .. } => Some(scrub.text(&format!("a connection to Steplet's gate: {reason}"))),
            _ => None,
        })
        .collect()
}

/// Every model the run used: the one it started with, those in the result's `modelUsage`, and both ends of every
/// switch; each once, in the order first seen.
fn models(events: &[Event], scrub: &Scrubber) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut add = |model: &str| {
        let model = scrub.text(model);
        if !model.is_empty() && !seen.contains(&model) {
            seen.push(model);
        }
    };
    for e in events {
        match e {
            Event::Started { model, .. } => add(model),
            Event::ModelSwitch { from, to, .. } => {
                add(from);
                to.iter().for_each(|t| add(t));
            }
            Event::Result { models, .. } => models.iter().for_each(|m| add(m)),
            _ => {}
        }
    }
    seen
}

/// The agent's own summary: the result's text when there is one, else the agent's last text.
fn agent_says(events: &[Event]) -> &str {
    let last_result = events.iter().rev().find_map(|e| match e {
        Event::Result { text, .. } => Some(text.as_str()),
        _ => None,
    });
    last_result
        .or_else(|| {
            events.iter().rev().find_map(|e| match e {
                Event::AgentText(text) => Some(text.as_str()),
                _ => None,
            })
        })
        .unwrap_or("")
}
