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

use std::cmp::Reverse;
use std::collections::HashMap;

use serde_json::{Map, Value};
use steplet_core::display::{is_invisible, is_space_like};

use crate::ask::{Answer, Reply};
use crate::gate::Asked;
use crate::run::RunOutcome;
use crate::types::{Event, GateAnswer};
use crate::watch::{Tripwire, described, refused_as_not_offered, what};

/// What a secret shows as in the report.
const HIDDEN: &str = "[hidden by Steplet]";

/// Phase 0 S6: the fence stops what it blocks without telling Steplet.
const FENCE_NOTE: &str = "Blocked fence attempts are not visible to Steplet: the fence stops a program without telling \
                          Steplet, so this report never says the fence stopped a particular thing.";

/// What the receiver is shown after a run.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    /// The run ended on its own and without an error, by Steplet's records: exactly one result, with `success`, no
    /// error and `end_turn`; no tripwire; not stopped at its time limit; no fault during the run; Claude Code's exit
    /// code 0. Faults in Steplet's own looks after the run and in removing it are listed in `faults` but do not change
    /// it. It says nothing about whether the task's own tests passed: their exit codes are in `commands`.
    pub completed: bool,
    /// Why Steplet stopped the run, if a tripwire did (D3, D8, row 38).
    pub stopped_by: Option<Tripwire>,
    /// The receiver pressed Stop (phase 4): the run was stopped as at a tripwire, and its folders were removed.
    pub stopped_by_you: bool,
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
    /// Every question the receiver was asked, and every yes reused, in order, with the reply (R7).
    pub answers: Vec<Answer>,
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
    /// Whether Claude Code marked what came back as an error; `None` when nothing came back.
    pub ended_in_error: Option<bool>,
    /// What came back, as Claude Code gave it.
    pub output: String,
    /// Steplet was already stopping the run before this call's result came (or none came): what it shows was cut
    /// short or refused by the stop, and it is not a breach of its own. Never set for a call that ran outside the
    /// checker (refused but it ran, or not asked and no harmless read, with no error): that is a breach, stop or not.
    pub after_stop: bool,
}

/// What Steplet did with one tool call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    /// The gate asked Steplet, and Steplet allowed it.
    Allowed,
    /// The gate asked Steplet, the checker asked the receiver, and the receiver said yes to this exact request;
    /// `reused`: they had said yes to it earlier in the run, so it was not asked again (J2 B).
    AllowedByYou { reused: bool },
    /// The gate asked Steplet, and Steplet refused it.
    Refused,
    /// The gate asked Steplet, the checker asked the receiver, and there was no yes: `how` the question ended.
    RefusedWhenAsked { how: NotYes },
    /// Steplet refused it at the gate, yet its result came back with no error: it ran anyway. That stops the run
    /// (Task 9), and it is a breach of its own even while the run is being stopped.
    RefusedButRan,
    /// It did not ask the gate. `harmless`: it was a harmless read (G2); otherwise it stopped the run.
    NotAsked { harmless: bool },
    /// It did not ask the gate, and its tool is not one the run offers: Claude Code answered it with its own words for
    /// such a tool, so it did not run. It stopped the run (phase 5 Task 5).
    NotOffered,
    /// No result came, so there is no evidence it ran (Task 9).
    NotJudged,
    /// The gate turned the call's connection away before deciding (the wrong run token, a request it could not
    /// read), and the call did not run (Task 13).
    TurnedAway,
}

impl Check {
    /// The check of a call that did not ask the gate, from Steplet's record of it (`Event::SkippedGate`).
    pub fn of_skipped(harmless: bool, not_offered: bool) -> Check {
        if not_offered { Check::NotOffered } else { Check::NotAsked { harmless } }
    }
}

/// How a question ended without a yes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotYes {
    /// The receiver said no (or the answer did not match the request on screen, which counts as a no).
    SaidNo,
    /// No answer came within the ask limit.
    NoAnswer,
    /// The run was stopping, so Steplet took the question back.
    TakenBack,
}

impl NotYes {
    /// How `reply` ended its question, if it was no yes.
    pub fn of(reply: &Reply) -> Option<NotYes> {
        match reply {
            Reply::Yes { .. } | Reply::Reused => None,
            Reply::No => Some(NotYes::SaidNo),
            Reply::NoAnswer => Some(NotYes::NoAnswer),
            Reply::Withdrawn => Some(NotYes::TakenBack),
        }
    }
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
        stopped_by_you: outcome.stopped_by_you,
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
        faults: outcome.faults.iter().map(|f| scrub.text(f)).chain(service_refused(&outcome.events)).collect(),
        answers: outcome
            .answers
            .iter()
            .map(|a| Answer {
                call: scrub.text(&a.call),
                tool: scrub.text(&a.tool),
                what: scrub.text(&a.what),
                reply: a.reply.clone(),
            })
            .collect(),
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

    /// `e` with every text in it cleaned as `text` cleans it, for the live log (phase 4 Task 3): the screen sees
    /// what the report would show.
    pub(crate) fn event(&self, e: &Event) -> Event {
        let t = |text: &str| self.text(text);
        match e {
            Event::Started { model, tools, gate_connected } => Event::Started {
                model: t(model),
                tools: tools.iter().map(|x| t(x)).collect(),
                gate_connected: *gate_connected,
            },
            Event::AgentText(text) => Event::AgentText(t(text)),
            Event::ToolUse { id, name, input } => Event::ToolUse { id: t(id), name: t(name), input: self.value(input) },
            Event::ToolResult { id, is_error, text, plain_end } => {
                Event::ToolResult { id: t(id), is_error: *is_error, text: t(text), plain_end: *plain_end }
            }
            Event::HookEvent { name } => Event::HookEvent { name: t(name) },
            Event::ModelSwitch { from, to, category, explanation } => Event::ModelSwitch {
                from: t(from),
                to: to.as_deref().map(t),
                category: t(category),
                explanation: t(explanation),
            },
            Event::Result { subtype, is_error, stop_reason, models, text } => Event::Result {
                subtype: t(subtype),
                is_error: *is_error,
                stop_reason: t(stop_reason),
                models: models.iter().map(|m| t(m)).collect(),
                text: t(text),
            },
            Event::SkippedGate { id, tool, what, harmless, not_offered } => Event::SkippedGate {
                id: t(id),
                tool: t(tool),
                what: t(what),
                harmless: *harmless,
                not_offered: *not_offered,
            },
            Event::ApiRetry { status, error, attempt, max } => {
                Event::ApiRetry { status: *status, error: t(error), attempt: *attempt, max: *max }
            }
            Event::ApiError { status } => Event::ApiError { status: *status },
            Event::Stderr(text) => Event::Stderr(t(text)),
            Event::Unreadable(text) => Event::Unreadable(t(text)),
            Event::Other(text) => Event::Other(t(text)),
        }
    }

    /// A JSON value with every string in it, keys included, cleaned.
    pub(crate) fn value(&self, v: &Value) -> Value {
        match v {
            Value::String(s) => Value::String(self.text(s)),
            Value::Array(items) => Value::Array(items.iter().map(|x| self.value(x)).collect()),
            Value::Object(fields) => {
                Value::Object(fields.iter().map(|(k, x)| (self.text(k), self.value(x))).collect::<Map<_, _>>())
            }
            other => other.clone(),
        }
    }

    fn tripwire(&self, t: &Tripwire) -> Tripwire {
        match t {
            Tripwire::Hook(name) => Tripwire::Hook(self.text(name)),
            Tripwire::SkippedCommand(command) => Tripwire::SkippedCommand(self.text(command)),
            Tripwire::SkippedGate(what) => Tripwire::SkippedGate(self.text(what)),
            Tripwire::NotOffered(tool) => Tripwire::NotOffered(self.text(tool)),
        }
    }
}

/// One line of a raw log Steplet keeps on Windows (`<run>.jsonl`, `<run>.stderr.txt`), as it is written: each secret
/// (the key, the run's token) hidden, every other byte exactly as Claude Code wrote it (J6 A).
pub fn raw_log_line(line: &[u8], secrets: &[&str]) -> Vec<u8> {
    let mut secrets: Vec<&[u8]> = secrets.iter().map(|s| s.as_bytes()).filter(|s| !s.is_empty()).collect();
    secrets.sort_by_key(|s| Reverse(s.len()));
    let mut out = Vec::with_capacity(line.len());
    let mut rest = line;
    'bytes: while let Some((&first, after)) = rest.split_first() {
        for secret in &secrets {
            if let Some(after_secret) = rest.strip_prefix(*secret) {
                out.extend_from_slice(HIDDEN.as_bytes());
                rest = after_secret;
                continue 'bytes;
            }
        }
        out.push(first);
        rest = after;
    }
    out
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

/// The report's fault when the run ended on Anthropic's HTTP 401: a key it does not accept (Task 12).
pub const KEY_REFUSED: &str = "Anthropic did not accept the key. Change it in Setup.";

/// The report's fault when the run ended on an error from Anthropic, in Steplet's own words. 401 is a key it does
/// not accept (Task 12); any other status says only its number, since Anthropic's reason is text from the run
/// (Task 15: a key that belongs to no workspace ended a real run on 400). Only a 4xx is a refusal of the request; any
/// other status is the service failing, and may pass. The fault points to "The agent says" only when it holds
/// something. An error status on a result that is no error is no fault.
fn service_refused(events: &[Event]) -> Option<String> {
    events.windows(2).find_map(|pair| match pair {
        [Event::ApiError { status: 401 }, Event::Result { is_error: true, .. }] => Some(KEY_REFUSED.to_string()),
        [Event::ApiError { status }, Event::Result { is_error: true, text, .. }] => {
            let what = if (400..500).contains(status) {
                format!("Anthropic refused the run's request with error {status}, so the agent could not go on.")
            } else {
                format!(
                    "Anthropic's service answered with error {status}, so the agent could not go on. It may work if you \
                     run it again later."
                )
            };
            let pointer = if text.trim().is_empty() {
                ""
            } else {
                " Claude Code passed on what Anthropic said under \"The agent says\"."
            };
            Some(format!("{what}{pointer}"))
        }
        _ => None,
    })
}

/// Every tool call, with Steplet's check, its result and whether the stop shaped it.
fn commands(outcome: &RunOutcome, scrub: &Scrubber) -> Vec<ReportCommand> {
    let mut answered: HashMap<&str, bool> = HashMap::new();
    for asked in &outcome.requests {
        if let Asked::Decision { request, answer, .. } | Asked::InApp { request, answer, .. } = asked {
            let allowed = answered.entry(request.tool_use_id.as_str()).or_insert(false);
            *allowed |= matches!(answer, GateAnswer::Allow { .. });
        }
    }
    // The calls the receiver said yes to: `true` when the yes was one given earlier in the run.
    let by_you: HashMap<&str, bool> = outcome
        .answers
        .iter()
        .filter_map(|a| match a.reply {
            Reply::Yes { .. } => Some((a.call.as_str(), false)),
            Reply::Reused => Some((a.call.as_str(), true)),
            _ => None,
        })
        .collect();
    // The calls whose connection the gate turned away before deciding, as the gate program named them.
    let turned_away: std::collections::HashSet<&str> = outcome
        .requests
        .iter()
        .filter_map(|a| match a {
            Asked::Refused { call: Some(call), .. } => Some(call.as_str()),
            _ => None,
        })
        .collect();
    // The calls asked about that got no yes, and how each question ended.
    let not_yes: HashMap<&str, NotYes> =
        outcome.answers.iter().filter_map(|a| Some((a.call.as_str(), NotYes::of(&a.reply)?))).collect();
    let results = results(&outcome.events);
    let mut skipped: HashMap<&str, (bool, bool)> = HashMap::new();
    for e in &outcome.events {
        if let Event::SkippedGate { id, harmless, not_offered, .. } = e {
            skipped.insert(id.as_str(), (*harmless, *not_offered));
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
                (Some(true), _) => match by_you.get(id.as_str()) {
                    Some(&reused) => Check::AllowedByYou { reused },
                    None => Check::Allowed,
                },
                (Some(false), _) if result.is_some_and(|r| !r.is_error) => Check::RefusedButRan,
                (Some(false), _) => match not_yes.get(id.as_str()) {
                    Some(&how) => Check::RefusedWhenAsked { how },
                    None => Check::Refused,
                },
                // Turned away, and it did not run: a call that ran without a decision, or went past the gate, stays
                // what it is.
                (None, None) if turned_away.contains(id.as_str()) && result.is_none_or(|r| r.is_error) => {
                    Check::TurnedAway
                }
                // Marked not offered only with Claude Code's own words for such a tool: a call that came back with no
                // error, or with other words, may have run, whatever else.
                (None, Some(&(harmless, not_offered))) => Check::of_skipped(
                    harmless,
                    not_offered && result.is_some_and(|r| r.is_error && refused_as_not_offered(name, r.text)),
                ),
                (None, None) => Check::NotJudged,
            };
            let exit_code = result.and_then(|r| exit_code_of(name, input, r.is_error, r.plain_end, r.text));
            // A call that ran although Steplet refused it, or ran without asking and was no harmless read, is a breach
            // of its own, stop or not.
            let ran = result.is_some_and(|r| !r.is_error);
            let breach = check == Check::RefusedButRan || (check == Check::NotAsked { harmless: false } && ran);
            let after_stop = !breach && stop.is_some_and(|stop| result.is_none_or(|r| r.event >= stop));
            Some(ReportCommand {
                tool: scrub.text(name),
                command: scrub.text(&what(name, input)),
                check,
                exit_code,
                ended_in_error: result.map(|r| r.is_error),
                output: result.map_or(String::new(), |r| scrub.text(r.text)),
                after_stop,
            })
        })
        .collect()
}

/// One tool call's result: where it is among the events, and what Claude Code marked on it.
struct CallResult<'a> {
    event: usize,
    is_error: bool,
    plain_end: bool,
    text: &'a str,
}

/// The first result for each tool call id.
fn results(events: &[Event]) -> HashMap<&str, CallResult<'_>> {
    let mut results = HashMap::new();
    for (event, e) in events.iter().enumerate() {
        if let Event::ToolResult { id, is_error, text, plain_end } = e {
            results.entry(id.as_str()).or_insert(CallResult {
                event,
                is_error: *is_error,
                plain_end: *plain_end,
                text: text.as_str(),
            });
        }
    }
    results
}

/// A tool call's exit code from Claude Code's own marks on its result (H13), never from what the program printed. Only
/// a `Bash` call not moved to the background has one: no error with a plain end in Claude Code's record means 0; with
/// an error, Claude Code's first line `Exit code N` gives N. Anything else (a refusal, an interruption, an exit code
/// Claude Code gave a meaning of its own) has none. The report and the app's live log both read exit codes here.
pub fn exit_code_of(tool: &str, input: &Value, is_error: bool, plain_end: bool, text: &str) -> Option<i32> {
    if tool != "Bash" || input["run_in_background"] == Value::Bool(true) {
        return None;
    }
    if !is_error {
        return plain_end.then_some(0);
    }
    text.lines().next()?.strip_prefix("Exit code ")?.parse().ok()
}

/// Everything the gate refused: each refused decision, marked if the call ran anyway, and each connection refused
/// before any decision.
fn refused(outcome: &RunOutcome, scrub: &Scrubber) -> Vec<String> {
    let results = results(&outcome.events);
    outcome
        .requests
        .iter()
        .filter_map(|asked| match asked {
            Asked::Decision { request, answer: GateAnswer::Deny { message }, .. }
            | Asked::InApp { request, answer: GateAnswer::Deny { message }, .. } => {
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
