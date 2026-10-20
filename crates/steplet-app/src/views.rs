//! Everything the screens get (plan section 6, Task 9). File, repository, agent and command text reaches the screens
//! only as display pieces (`Shown`), which only `web/shared/FileText.svelte` draws, as plain text (row 28). Every other
//! text here is Steplet's own words. The screens decide no colour and judge nothing: the outcome, the checks and the
//! answers come from the core and the runner, already decided.
//!
//! The TypeScript types in `web/shared/types/` are written from these types (ts-rs, J7 A); a test fails when they are
//! out of date.

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use steplet_core::display::{self, Segment};
use steplet_core::header::{Scalar, Steplet};
use steplet_core::identity::Identity;
use steplet_core::report::Report as FileReport;
use steplet_core::rules::{self, Finding, Outcome, Severity};
use steplet_core::sig::SigStatus;
use steplet_run::{
    Answer, Check, Event, Progress, Promise, Question, Refusal, Reply, Report, ReportCommand, Tripwire, Trust,
    call_what, exit_code_of,
};
use ts_rs::TS;

/// Text from the file, the repository, the agent or a command, cut into display pieces.
pub type Shown = Vec<Seg>;

/// One display piece.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Seg {
    /// Plain text, drawn as it is.
    Text { text: String },
    /// A character that shows nothing or looks like a space (or a line break inside one entry), drawn as a badge.
    Invisible { codepoint: u32 },
    /// A link in the text, drawn as its shown text followed by its real address, never as a link. `shown` is `None`
    /// for a bare address.
    Link { shown: Option<Vec<Part>>, real: Vec<Part> },
}

/// A piece inside a link: plain text or a badge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    Invisible { codepoint: u32 },
}

/// Text of several lines (a body, a command's output, what the agent says): its line breaks stay line breaks.
pub fn shown(text: &str) -> Shown {
    display::segments(text).into_iter().map(seg).collect()
}

/// One entry of a list (a step, a command, a changed file): a line break inside it is a badge, so one entry never
/// looks like two.
pub fn line(text: &str) -> Shown {
    let mut out = Vec::new();
    for (i, piece) in text.split('\n').enumerate() {
        if i > 0 {
            out.push(Seg::Invisible { codepoint: 0x0a });
        }
        out.extend(shown(piece));
    }
    out
}

fn seg(s: Segment) -> Seg {
    match s {
        Segment::Text(text) => Seg::Text { text },
        Segment::InvisibleBadge { codepoint } => Seg::Invisible { codepoint },
        Segment::Link { shown, real } => Seg::Link {
            shown: shown.map(|s| s.into_iter().map(part).collect()),
            real: real.into_iter().map(part).collect(),
        },
    }
}

fn part(s: Segment) -> Part {
    match s {
        Segment::Text(text) => Part::Text { text },
        Segment::InvisibleBadge { codepoint } => Part::Invisible { codepoint },
        // The core puts no link inside a link; were it ever to, its pieces are kept as plain text and badges.
        Segment::Link { real, .. } => Part::Text { text: real.into_iter().map(plain).collect() },
    }
}

fn plain(s: Segment) -> String {
    match s {
        Segment::Text(text) => text,
        Segment::InvisibleBadge { codepoint } => char::from_u32(codepoint).map(String::from).unwrap_or_default(),
        Segment::Link { real, .. } => real.into_iter().map(plain).collect(),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The preview

/// Whether the Run button is offered, and how. The colour is the worst of the file's findings and the findings about
/// this computer (row 22: a changed sender key makes the preview red).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunStatus {
    /// No Run button; the first blocked finding's advice instead.
    Blocked,
    /// Run works only after the receiver types the sentence (J3 A, R9).
    Red,
    Amber,
    Green,
    /// The file cannot run on this computer (no repository, or one on a host Steplet does not fetch from, H9).
    NotRunnable {
        reason: String,
    },
}

/// The identity line (rows 16 and 22, R14).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IdentityView {
    /// The file could not be cleaned up, so its signature was not checked.
    Unchecked,
    Unsigned,
    Invalid,
    NewKey {
        key: String,
    },
    SameKey {
        key: String,
        runs: u32,
    },
    /// The key is known here only under other sender names; the names are text from earlier files.
    KeyKnownAs {
        key: String,
        names: Vec<Shown>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct FindingView {
    pub rule: String,
    /// `blocked`, `red`, `amber` or `note`.
    pub severity: String,
    pub line: Option<u32>,
    pub title: String,
    /// Steplet's own words (the core never puts file text here).
    pub detail: Option<String>,
    pub advice: String,
    /// The file's text the finding is about.
    pub quote: Option<Shown>,
}

/// One entry of the file's header, with its line in the file (findings name lines).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct EntryView {
    pub line: u32,
    pub text: Shown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct RepositoryView {
    pub url: EntryView,
    pub commit: EntryView,
}

/// Everything the file says, each part as file text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct FileView {
    pub title: EntryView,
    pub sender: EntryView,
    /// The expiry date, `YYYY-MM-DD`, as Steplet read it.
    pub expires: String,
    pub repository: Option<RepositoryView>,
    pub steps: Vec<EntryView>,
    pub commands: Vec<EntryView>,
    pub domains: Vec<EntryView>,
    pub read: Vec<EntryView>,
    pub write: Vec<EntryView>,
    pub never: Vec<EntryView>,
    /// The body, as plain text: Markdown is never turned into HTML.
    pub body: Shown,
    pub body_line: u32,
}

/// The counts the preview shows first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Counts {
    pub steps: u32,
    pub commands: u32,
    pub domains: u32,
    pub read: u32,
    pub write: u32,
    pub never: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PreviewView {
    /// The SHA-256 of the file's bytes as read, in hex: Run is refused if the bytes changed since (row 30).
    pub sha256: String,
    /// The file's own outcome, as the core rated it: `green`, `amber`, `red` or `blocked`.
    pub outcome: String,
    /// The colour the screen shows: the worst of the file's findings and those about this computer (row 22), in the
    /// same words. It differs from `outcome` only when something on this computer is worse than the file.
    pub colour: String,
    pub run_status: RunStatus,
    pub identity: IdentityView,
    /// The sender name is pinned here to another key, or the file is unsigned under a pinned name (`id.key-changed`,
    /// rows 16 and 22): the identity line starts with that warning, whose detail is in `here`.
    pub key_changed: bool,
    /// The file's findings, worst first, as the core ordered them.
    pub findings: Vec<FindingView>,
    /// Findings about this computer ("On this computer"): a changed sender key, a damaged list of known senders.
    pub here: Vec<FindingView>,
    /// `None` when the header could not be read.
    pub file: Option<FileView>,
    pub counts: Counts,
}

/// What is true on this computer and not in the file, worked out by the app (Task 10) from the pins and the runner's
/// host list.
#[derive(Clone, Debug, PartialEq)]
pub struct Here {
    /// `None` when the signature was not checked (the file could not be cleaned up).
    pub identity: Option<Identity>,
    /// Findings about this computer, each from the core's own rule (`id.key-changed`) or the app's own facts.
    pub findings: Vec<Finding>,
    /// Why the file cannot run here, if it cannot.
    pub not_runnable: Option<String>,
}

/// Why a file with no repository cannot run (in Steplet's words only).
pub const NO_REPOSITORY: &str = "This file names no repository, so Steplet has nothing to fetch and run.";

/// Why a file whose repository is on another host cannot run (H9). It names Steplet's own list and never the file's
/// host: that is file text, shown in the file's own panel.
pub fn not_fetched_from_here() -> String {
    format!(
        "Steplet fetches repositories only from plain https:// addresses on {}, and this file's repository is not one \
         of those.",
        and_list(steplet_run::REPOSITORY_HOSTS)
    )
}

fn and_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => one.to_string(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The preview of a file read as `raw`, with the core's report on it and what is true on this computer.
pub fn preview(raw: &[u8], report: &FileReport, here: &Here) -> PreviewView {
    let file_worst = report.rating.findings.first().map(|f| f.rule.severity);
    let here_worst = here.findings.iter().map(|f| f.rule.severity).max();
    let worst = file_worst.into_iter().chain(here_worst).max();
    let colour = worst.map(Outcome::of).unwrap_or(Outcome::Green);
    let run_status = match colour {
        Outcome::Blocked => RunStatus::Blocked,
        _ if here.not_runnable.is_some() => {
            RunStatus::NotRunnable { reason: here.not_runnable.clone().unwrap_or_default() }
        }
        Outcome::RedFindings => RunStatus::Red,
        Outcome::Amber => RunStatus::Amber,
        Outcome::Green => RunStatus::Green,
    };
    let identity = match (&here.identity, &report.signature) {
        (Some(i), _) => identity_view(i),
        (None, None) => IdentityView::Unchecked,
        (None, Some(SigStatus::Unsigned)) => IdentityView::Unsigned,
        (None, Some(SigStatus::Invalid { .. })) => IdentityView::Invalid,
        (None, Some(SigStatus::Valid { fingerprint })) => IdentityView::NewKey { key: fingerprint.clone() },
    };
    let s = report.steplet.as_ref();
    let count = |f: fn(&Steplet) -> usize| s.map_or(0, |s| f(s) as u32);
    PreviewView {
        sha256: Sha256::digest(raw).iter().map(|b| format!("{b:02x}")).collect(),
        outcome: report.rating.outcome.name().into(),
        colour: colour.name().into(),
        run_status,
        identity,
        key_changed: here.findings.iter().any(|f| f.rule.id == rules::KEY_CHANGED.id),
        findings: report.rating.findings.iter().map(finding_view).collect(),
        here: here.findings.iter().map(finding_view).collect(),
        file: s.map(file_view),
        counts: Counts {
            steps: count(|s| s.steps.len()),
            commands: count(|s| s.allow.commands.len()),
            domains: count(|s| s.allow.domains.len()),
            read: count(|s| s.allow.read.len()),
            write: count(|s| s.allow.write.len()),
            never: count(|s| s.never.len()),
        },
    }
}

fn identity_view(i: &Identity) -> IdentityView {
    match i {
        Identity::Unsigned => IdentityView::Unsigned,
        Identity::Invalid => IdentityView::Invalid,
        Identity::NewKey { key } => IdentityView::NewKey { key: key.clone() },
        Identity::SameKey { key, runs } => IdentityView::SameKey { key: key.clone(), runs: *runs },
        Identity::KeyKnownAs { key, names } => {
            IdentityView::KeyKnownAs { key: key.clone(), names: names.iter().map(|n| line(n)).collect() }
        }
    }
}

pub fn finding_view(f: &Finding) -> FindingView {
    FindingView {
        rule: f.rule.id.into(),
        severity: severity(f.rule.severity).into(),
        line: f.line,
        title: f.rule.title.into(),
        detail: f.detail.clone(),
        advice: f.rule.advice.into(),
        quote: f.quote.as_deref().map(line),
    }
}

fn severity(s: Severity) -> &'static str {
    match s {
        Severity::Blocked => "blocked",
        Severity::Red => "red",
        Severity::Amber => "amber",
        Severity::Note => "note",
    }
}

fn entry(s: &Scalar) -> EntryView {
    EntryView { line: s.line, text: line(&s.text) }
}

fn entries(list: &[Scalar]) -> Vec<EntryView> {
    list.iter().map(entry).collect()
}

fn file_view(s: &Steplet) -> FileView {
    FileView {
        title: entry(&s.title),
        sender: entry(&s.sender),
        expires: s.expires.to_string(),
        repository: s.repository.as_ref().map(|r| RepositoryView { url: entry(&r.url), commit: entry(&r.commit) }),
        steps: entries(&s.steps),
        commands: entries(&s.allow.commands),
        domains: entries(&s.allow.domains),
        read: entries(&s.allow.read),
        write: entries(&s.allow.write),
        never: entries(&s.never),
        body: shown(&s.body),
        body_line: s.body_line,
    }
}

// ---------------------------------------------------------------------------------------------------------------
// The run chooser

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PromiseView {
    pub text: String,
    /// The file's own words the line lists (its web sites).
    pub items: Vec<Shown>,
    /// The tests that back the line (R6).
    pub tests: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct RunnerView {
    /// Steplet's Linux and the key are ready; otherwise the button says "Set up first".
    pub ready: bool,
    pub promises: Vec<PromiseView>,
    /// What leaves the computer (R16).
    pub data_leaves: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct ChooserView {
    pub claude: RunnerView,
    /// Why Codex is not offered (D1).
    pub codex_reason: String,
}

const DATA_LEAVES: &str =
    "Claude Code sends what the agent reads, the commands' output and this task to Anthropic, using your API key.";
const CODEX_REASON: &str = "Not available for files that run code on this computer: Steplet has not shown that Codex \
                            keeps a task's programs away from the other programs on this computer.";

/// The chooser for a file, with the runner's own lines for it (`steplet_run::promises`).
pub fn chooser(promises: &[Promise], ready: bool) -> ChooserView {
    ChooserView {
        claude: RunnerView {
            ready,
            promises: promises
                .iter()
                .map(|p| PromiseView {
                    text: p.text.into(),
                    items: p.items.iter().map(|i| line(i)).collect(),
                    tests: p.tests.iter().map(|t| t.to_string()).collect(),
                })
                .collect(),
            data_leaves: DATA_LEAVES.into(),
        },
        codex_reason: CODEX_REASON.into(),
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Setup

/// How one line of the setup checklist stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum LineStatus {
    Ready,
    Missing,
    /// Steplet could not look (`setup::State.trouble`), which is not the same as missing.
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct LineView {
    pub status: LineStatus,
    /// Steplet's own words.
    pub text: String,
    /// What a program printed when Steplet could not look (`setup::State.trouble`): not Steplet's words, so file text.
    pub detail: Option<Shown>,
}

/// The "Open .steplet files with Steplet" switch (J4 A).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Switch {
    On,
    Off,
    /// On, but pointing at a program that is not this one any more.
    Broken,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct SetupView {
    pub wsl: LineView,
    pub linux: LineView,
    pub tools: LineView,
    pub gate: LineView,
    pub key_saved: bool,
    pub opens_steplet_files: Switch,
    /// Everything a run needs is there (the switch is not needed to run).
    pub ready: bool,
    /// When Windows' Linux feature is missing or too old: the command the receiver types in a terminal opened as
    /// administrator. Steplet never runs it.
    pub wsl_fix: Option<String>,
    /// Whether Setup offers to install Steplet's Linux, or to finish setting it up.
    pub install: Offer,
}

/// What the Setup screen's one button offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Offer {
    /// Nothing to install, or Steplet cannot tell yet (Windows' feature first, or it could not look).
    None,
    /// Steplet's Linux is not there.
    Install,
    /// Steplet's Linux is there but lacks something; setup is safe to repeat.
    Finish,
}

/// Plain words for each check `look.sh` makes (`steplet_run::setup::CHECKS`, in its order), for the line that says
/// what is missing.
pub const CHECK_WORDS: &[(&str, &str)] = &[
    ("git", "git"),
    ("python3", "Python"),
    ("python3-venv", "Python's venv module"),
    ("build-essential", "the tools that build programs from source"),
    ("ca-certificates", "the certificates that check web sites"),
    ("bubblewrap", "bubblewrap, part of the fence"),
    ("socat", "socat, part of the fence"),
    ("ripgrep", "ripgrep, which the fence needs"),
    ("nodejs", "Node.js"),
    ("npm", "npm"),
    ("claude-code", "Claude Code 2.1.220"),
    ("srt", "the fence program srt 0.0.77"),
    ("wsl.conf", "WSL's settings for Steplet's Linux"),
    ("steplet-account", "the steplet account the agent runs as"),
    ("folders", "Steplet's own folders"),
];

fn check_words(check: &str) -> &str {
    CHECK_WORDS.iter().find(|(c, _)| *c == check).map_or(check, |(_, words)| words)
}

/// The setup checklist from what `steplet_run::setup::look` saw, whether a key is saved, and the switch.
pub fn setup_view(state: &steplet_run::setup::State, key_saved: bool, switch: Switch) -> SetupView {
    use steplet_run::setup::Wsl;
    let said = |status: LineStatus, text: String| LineView { status, text, detail: None };
    let wsl = match &state.wsl {
        Wsl::Missing => said(LineStatus::Missing, "Windows' Linux feature (WSL 2) is not on this computer.".into()),
        Wsl::TooOld { version } => said(
            LineStatus::Missing,
            format!("Windows' Linux feature is version {version}; Steplet needs 2.6.3 or newer."),
        ),
        Wsl::Ready { version } => said(LineStatus::Ready, format!("Windows' Linux feature (WSL {version}).")),
    };
    let linux_there = state.linux.is_some();
    // A line Steplet cannot look at, and why, from the first cause in order: the feature, the trouble, its Linux.
    // What went wrong is said once, on the first line Steplet could not look at.
    let not_seen = |what: &str, first: bool| match (&state.wsl, state.trouble.as_deref()) {
        (Wsl::Missing, _) => {
            said(LineStatus::Unknown, format!("Steplet can look at {what} once Windows' Linux feature is there."))
        }
        (Wsl::TooOld { .. }, _) => {
            said(LineStatus::Unknown, format!("Steplet can look at {what} once Windows' Linux feature is updated."))
        }
        (_, Some(trouble)) if first => LineView {
            status: LineStatus::Unknown,
            text: format!("Steplet could not look at {what}. What went wrong:"),
            detail: Some(shown(trouble)),
        },
        (_, Some(_)) => said(LineStatus::Unknown, format!("Steplet could not look at {what} either.")),
        (_, None) => said(LineStatus::Unknown, format!("Steplet can look at {what} once its Linux is installed.")),
    };
    let seen = matches!(state.wsl, Wsl::Ready { .. }) && state.trouble.is_none();
    let linux = if !seen {
        not_seen("its own Linux", true)
    } else if linux_there {
        said(LineStatus::Ready, "Steplet's own Linux is installed.".into())
    } else {
        said(LineStatus::Missing, "Steplet's own Linux is not installed yet.".into())
    };
    let tools = if !seen || !linux_there {
        not_seen("the tools in its Linux", false)
    } else if state.missing.is_empty() {
        said(LineStatus::Ready, "Its tools are there: git, Python, Claude Code and the rest.".into())
    } else {
        let words: Vec<&str> = state.missing.iter().map(|m| check_words(m)).collect();
        said(LineStatus::Missing, format!("Missing in Steplet's Linux: {}.", and_list(&words)))
    };
    let gate = if !seen || !linux_there {
        not_seen("its gate program", false)
    } else if state.gate {
        said(LineStatus::Ready, "The gate program for this Steplet is in its Linux.".into())
    } else {
        said(LineStatus::Missing, "The gate program for this Steplet is not in its Linux yet.".into())
    };
    let ready = [&wsl, &linux, &tools, &gate].iter().all(|l| l.status == LineStatus::Ready) && key_saved;
    let wsl_fix = match state.wsl {
        Wsl::Missing => Some("wsl --install --no-distribution".to_string()),
        Wsl::TooOld { .. } => Some("wsl --update".to_string()),
        Wsl::Ready { .. } => None,
    };
    let install = if !seen {
        Offer::None
    } else if !linux_there {
        Offer::Install
    } else if !state.missing.is_empty() || !state.gate {
        Offer::Finish
    } else {
        Offer::None
    };
    SetupView { wsl, linux, tools, gate, key_saved, opens_steplet_files: switch, ready, wsl_fix, install }
}

// ---------------------------------------------------------------------------------------------------------------
// A run: its progress, its questions and its report

/// One field of a tool call's request, all of it (the ask screen shows the whole request, nothing cut).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct FieldView {
    pub name: Shown,
    pub value: Shown,
}

/// One row of the live log.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventView {
    /// The run started, with this model.
    Started { model: Shown },
    /// What the agent wrote: shown only as "the agent says".
    AgentSays { text: Shown },
    /// A tool call the agent made. `what` is its command, path or address; `fields` its whole request.
    Call { id: String, tool: Shown, what: Shown, fields: Vec<FieldView> },
    /// A tool call's result. `exit_code` from Claude Code's own marks (H13), never from the output.
    Result { id: String, is_error: bool, exit_code: Option<i32>, output: Shown, lines: u32 },
    /// Steplet's record of a call that did not ask the gate; `harmless` means a harmless read and the run went on.
    Skipped { id: String, tool: Shown, what: Shown, harmless: bool },
    /// A model refused (E1); `to` is the model Claude Code then used, if it switched.
    ModelRefused { from: Shown, to: Option<Shown>, category: Shown, explanation: Shown },
    /// A hook ran, which stops the run (D3).
    Hook { name: Shown },
    /// Claude Code says the run ended (not the outcome by itself, F35).
    Ended { models: Vec<Shown> },
    /// A line of Claude Code's error output, or a line Steplet could not read.
    Note { text: Shown },
}

/// The live log's row for `e`. `call` is the tool name and request of the call a result belongs to, when known.
pub fn event_view(e: &Event, call: Option<(&str, &Value)>) -> Option<EventView> {
    Some(match e {
        Event::Started { model, .. } => EventView::Started { model: line(model) },
        Event::AgentText(text) => EventView::AgentSays { text: shown(text) },
        Event::ToolUse { id, name, input } => EventView::Call {
            id: id.clone(),
            tool: line(name),
            what: shown(&call_what(name, input)),
            fields: fields(input),
        },
        Event::ToolResult { id, is_error, text, plain_end } => EventView::Result {
            id: id.clone(),
            is_error: *is_error,
            exit_code: call.and_then(|(tool, input)| exit_code_of(tool, input, *is_error, *plain_end, text)),
            output: shown(text),
            lines: text.lines().count() as u32,
        },
        Event::SkippedGate { id, tool, what, harmless } => {
            EventView::Skipped { id: id.clone(), tool: line(tool), what: shown(what), harmless: *harmless }
        }
        Event::ModelSwitch { from, to, category, explanation } => EventView::ModelRefused {
            from: line(from),
            to: to.as_deref().map(line),
            category: line(category),
            explanation: shown(explanation),
        },
        Event::HookEvent { name } => EventView::Hook { name: line(name) },
        Event::Result { models, .. } => EventView::Ended { models: models.iter().map(|m| line(m)).collect() },
        Event::Stderr(text) | Event::Unreadable(text) => EventView::Note { text: shown(text) },
        Event::Other(_) => return None,
    })
}

/// Every field of a request, in its own order: strings as they are, anything else as JSON.
fn fields(input: &Value) -> Vec<FieldView> {
    match input {
        Value::Object(map) => map
            .iter()
            .map(|(k, v)| FieldView {
                name: line(k),
                value: match v {
                    Value::String(s) => shown(s),
                    other => shown(&other.to_string()),
                },
            })
            .collect(),
        other => vec![FieldView { name: Vec::new(), value: shown(&other.to_string()) }],
    }
}

/// A question on the ask screen (R7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct QuestionView {
    pub id: u32,
    /// What exactly is asked, in Steplet's words ("Run this command", "Change this file", …).
    pub ask: String,
    pub tool: Shown,
    /// The command, path or address.
    pub what: Shown,
    /// The whole request.
    pub fields: Vec<FieldView>,
    /// Why Steplet asks: the checker's reason, in Steplet's words.
    pub reason: String,
    /// Bound to the exact request; the yes carries it back (R7).
    pub fingerprint: String,
    /// How long the question had already waited when this view was made, and how long it may wait in all.
    pub waited_ms: u32,
    pub limit_ms: u32,
}

/// What the ask screen asks for a tool.
pub fn ask_words(tool: &str) -> &'static str {
    match tool {
        "Bash" => "Run this command",
        "Write" | "Edit" => "Change this file",
        "Read" | "Glob" | "Grep" => "Read this file",
        "WebFetch" => "Open this web address",
        _ => "Use this tool",
    }
}

/// The ask screen's view of `q`, asked with `limit` to answer.
pub fn question_view(q: &Question, limit: Duration) -> QuestionView {
    QuestionView {
        id: q.id as u32,
        ask: ask_words(&q.tool).into(),
        tool: line(&q.tool),
        what: shown(&q.what),
        fields: fields(&q.input),
        reason: q.reason.clone(),
        fingerprint: q.fingerprint.clone(),
        waited_ms: q.asked_at.elapsed().as_millis().min(u32::MAX as u128) as u32,
        limit_ms: limit.as_millis().min(u32::MAX as u128) as u32,
    }
}

/// How a closed question ended, in Steplet's words.
pub fn reply_words(r: &Reply) -> &'static str {
    match r {
        Reply::Yes { .. } => "You said yes to this exact request.",
        Reply::No => "You said no.",
        Reply::NoAnswer => "No answer in time, so Steplet refused it.",
        Reply::Withdrawn => "Taken back: the run was stopping.",
        Reply::Reused => "Allowed by your earlier yes to this exact request.",
    }
}

/// The setup screen after each change the receiver can make on it, for the stand-in backend (it decides nothing).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct SetupChanges {
    pub key_saved: SetupView,
    pub key_forgotten: SetupView,
    pub switch_on: SetupView,
    pub switch_off: SetupView,
    /// After "Start over": Steplet's Linux removed.
    pub removed: SetupView,
}

impl SetupChanges {
    pub fn of(state: &steplet_run::setup::State, key_saved: bool, switch: Switch) -> SetupChanges {
        SetupChanges {
            key_saved: setup_view(state, true, switch),
            key_forgotten: setup_view(state, false, switch),
            switch_on: setup_view(state, key_saved, Switch::On),
            switch_off: setup_view(state, key_saved, Switch::Off),
            removed: setup_view(
                &steplet_run::setup::State {
                    linux: None,
                    missing: Vec::new(),
                    gate: false,
                    trouble: None,
                    ..state.clone()
                },
                key_saved,
                switch,
            ),
        }
    }
}

/// Steplet's words for how a question ended, for the stand-in backend's `QuestionClosed`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Replies {
    pub yes: String,
    pub no: String,
}

impl Default for Replies {
    fn default() -> Self {
        Replies {
            yes: reply_words(&Reply::Yes { fingerprint: String::new() }).into(),
            no: reply_words(&Reply::No).into(),
        }
    }
}

/// One step of a run, for the screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProgressView {
    /// The working copy is ready at `commit`; these agent settings files were removed from it (D3, S7).
    Prepared {
        commit: String,
        removed: Vec<Shown>,
    },
    Event {
        event: EventView,
    },
    Question {
        question: QuestionView,
    },
    QuestionClosed {
        id: u32,
        how: String,
    },
    Finished {
        report: ReportView,
    },
    /// The run could not start or prepare: the stage, in Steplet's words, and why.
    Failed {
        stage: String,
        failure: FailureView,
    },
}

/// Why something failed: Steplet's own sentence, and what the program that failed printed, if anything. That output is
/// not Steplet's words (git prints what a repository's host sends; setup's programs print their own), so it is file
/// text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct FailureView {
    pub reason: String,
    pub detail: Option<Shown>,
}

pub fn failure(reason: &str, detail: Option<&str>) -> FailureView {
    FailureView { reason: reason.into(), detail: detail.map(shown) }
}

/// Why setup stopped, in Steplet's words, with the end of what the failed step printed as file text.
pub fn setup_failure(e: &steplet_run::setup::SetupError) -> FailureView {
    use steplet_run::setup::SetupError;
    match e {
        SetupError::Refused(what) => failure(&format!("Steplet will not hand this {what} to WSL."), None),
        SetupError::Step { step, detail } => {
            let detail = without_terminal_codes(detail);
            let said = if detail.trim().is_empty() { "" } else { " What it printed last:" };
            FailureView {
                reason: format!(
                    "Setup stopped while trying to {step}. Trying again is safe: Steplet picks up where it stopped.{said}"
                ),
                detail: (!said.is_empty()).then(|| shown(&detail)),
            }
        }
    }
}

/// One line a setup program printed, as the Setup log shows it (file text): a line drawn again in place (`\r`)
/// shows its last drawing, and the terminal's colour and cursor codes (`ESC [`, numbers, a letter) are left out.
/// Anything else that shows nothing is a badge, as in all file text.
pub fn setup_line(raw: &str) -> Shown {
    line(&without_terminal_codes(raw.rsplit('\r').next().unwrap_or(raw)))
}

/// `text` without the terminal's control sequences: `ESC [`, then its numbers and signs, then one letter or sign.
fn without_terminal_codes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for n in chars.by_ref() {
                if ('@'..='~').contains(&n) {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// The screen's view of one piece of live progress. `call` is the tool name and request of the call a result belongs
/// to, when known; `ask_limit` is how long a question may wait.
pub fn progress_view(p: &Progress, call: Option<(&str, &Value)>, ask_limit: Duration) -> Option<ProgressView> {
    Some(match p {
        Progress::Prepared(prepared) => ProgressView::Prepared {
            commit: prepared.commit.clone(),
            removed: prepared.removed.iter().map(|r| line(r)).collect(),
        },
        Progress::Event(e) => ProgressView::Event { event: event_view(e, call)? },
        Progress::Asking(q) => ProgressView::Question { question: question_view(q, ask_limit) },
        Progress::Answered { question, reply } => {
            ProgressView::QuestionClosed { id: *question as u32, how: reply_words(reply).into() }
        }
    })
}

/// The report's first line: how the run ended.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EndView {
    Finished,
    /// Steplet stopped it: `why` in Steplet's words, and what it was about (file, agent or command text).
    StoppedBySteplet {
        why: String,
        about: Shown,
    },
    /// The receiver pressed Stop (D4).
    StoppedByYou,
    /// A model refused, and the run did not finish (E1).
    ModelRefused,
    DidNotFinish,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct CheckView {
    /// `allowed`, `allowed_by_you`, `refused`, `refused_but_ran`, `not_asked` or `not_judged`.
    pub kind: String,
    /// Steplet's words for it.
    pub words: String,
}

pub fn check_view(c: &Check) -> CheckView {
    let (kind, words) = match c {
        Check::Allowed => ("allowed", "Allowed by Steplet's checker"),
        Check::AllowedByYou { reused: false } => ("allowed_by_you", "You allowed this exact request"),
        Check::AllowedByYou { reused: true } => ("allowed_by_you", "Allowed by your earlier yes to this exact request"),
        Check::Refused => ("refused", "Refused by Steplet's checker"),
        Check::RefusedButRan => ("refused_but_ran", "Refused by Steplet, yet it ran: Steplet stopped the run"),
        Check::NotAsked { harmless: true } => ("not_asked", "Ran without asking Steplet: a harmless read"),
        Check::NotAsked { harmless: false } => ("not_asked", "Ran without asking Steplet: Steplet stopped the run"),
        Check::NotJudged => ("not_judged", "No result came, so there is no sign it ran"),
    };
    CheckView { kind: kind.into(), words: words.into() }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct CommandView {
    pub tool: Shown,
    pub command: Shown,
    pub check: CheckView,
    /// From Claude Code's own marks (H13); `None` when there is none to show.
    pub exit_code: Option<i32>,
    pub output: Shown,
    pub lines: u32,
    /// Steplet was already stopping the run: the result was cut short or refused by the stop.
    pub after_stop: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct RefusalView {
    pub model: Shown,
    pub switched_to: Option<Shown>,
    pub category: Shown,
    pub explanation: Shown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct AnswerView {
    pub tool: Shown,
    pub what: Shown,
    /// How the question ended, in Steplet's words.
    pub reply: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct ReportView {
    pub end: EndView,
    pub completed: bool,
    pub stopped_by_you: bool,
    pub commands: Vec<CommandView>,
    pub refused: Vec<Shown>,
    pub answers: Vec<AnswerView>,
    pub models: Vec<Shown>,
    pub refusals: Vec<RefusalView>,
    /// `None` when Steplet could not look; `faults` says why.
    pub changed_files: Option<Vec<Shown>>,
    /// The trust check (D3), in Steplet's words.
    pub trust: String,
    pub faults: Vec<Shown>,
    pub notes: Vec<String>,
    /// Shown last, labelled (F47).
    pub agent_says: Shown,
}

pub fn report_view(r: &Report) -> ReportView {
    let end = if r.stopped_by_you {
        EndView::StoppedByYou
    } else if let Some(t) = &r.stopped_by {
        let (why, about) = match t {
            Tripwire::Hook(name) => ("A hook ran, so Steplet stopped the run.", name),
            Tripwire::SkippedCommand(command) => (
                "A command ran without asking Steplet, and it was not a harmless read, so Steplet stopped the run.",
                command,
            ),
            Tripwire::SkippedGate(what) => ("An action went past Steplet's checker, so Steplet stopped the run.", what),
        };
        EndView::StoppedBySteplet { why: why.into(), about: line(about) }
    } else if r.completed {
        EndView::Finished
    } else if !r.refusals.is_empty() {
        EndView::ModelRefused
    } else {
        EndView::DidNotFinish
    };
    ReportView {
        end,
        completed: r.completed,
        stopped_by_you: r.stopped_by_you,
        commands: r.commands.iter().map(command_view).collect(),
        refused: r.refused.iter().map(|x| line(x)).collect(),
        answers: r.answers.iter().map(answer_view).collect(),
        models: r.models.iter().map(|m| line(m)).collect(),
        refusals: r.refusals.iter().map(refusal_view).collect(),
        changed_files: r.changed_files.as_ref().map(|files| files.iter().map(|f| line(f)).collect()),
        trust: match r.trust {
            Trust::NotRecorded => "The run's settings mark no folder as trusted.",
            Trust::Recorded => "The run's settings mark a folder as trusted: something got past Steplet's settings.",
            Trust::Unknown => "Steplet could not read the run's settings to check them.",
        }
        .into(),
        faults: r.faults.iter().map(|f| shown(f)).collect(),
        notes: r.notes.iter().map(|n| n.to_string()).collect(),
        agent_says: shown(&r.agent_says),
    }
}

fn command_view(c: &ReportCommand) -> CommandView {
    CommandView {
        tool: line(&c.tool),
        command: shown(&c.command),
        check: check_view(&c.check),
        exit_code: c.exit_code,
        output: shown(&c.output),
        lines: c.output.lines().count() as u32,
        after_stop: c.after_stop,
    }
}

fn refusal_view(r: &Refusal) -> RefusalView {
    RefusalView {
        model: line(&r.model),
        switched_to: r.switched_to.as_deref().map(line),
        category: line(&r.category),
        explanation: shown(&r.explanation),
    }
}

fn answer_view(a: &Answer) -> AnswerView {
    AnswerView { tool: line(&a.tool), what: shown(&a.what), reply: reply_words(&a.reply).into() }
}

// ---------------------------------------------------------------------------------------------------------------
// Screen-test fixtures

/// One saved scenario for the stand-in backend (`web/app/lib/fake.ts`): what each command answers, and the progress a
/// run sends. Made only by `tests/fixtures.rs`, from real corpus files and real runner output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct Fixture {
    /// The screen the scenario opens on: `open`, `preview`, `chooser`, `setup`, `run` or `report`.
    pub screen: String,
    /// What `initial_preview` and `open_path` answer.
    pub preview: Option<PreviewView>,
    /// `open_path`'s refusal instead, in Steplet's words (the file could not be read). It never names the file.
    pub open_error: Option<String>,
    /// A note on the Open screen, in Steplet's words (a crash's run was cleared).
    pub note: Option<String>,
    pub chooser: Option<ChooserView>,
    pub setup: Option<SetupView>,
    /// The setup's progress lines (raw lines from Linux programs, so file text).
    pub setup_lines: Vec<Shown>,
    /// How the install ended, if it failed.
    pub setup_error: Option<FailureView>,
    /// The setup screen after each change the receiver can make there, as the Rust side works it out.
    pub setup_changes: Option<SetupChanges>,
    /// The run's progress, in order.
    pub progress: Vec<ProgressView>,
    /// What the run sends after the receiver presses Stop.
    pub after_stop: Vec<ProgressView>,
    /// Steplet's words for a question the receiver answered, as `QuestionClosed` carries them.
    pub replies: Replies,
    /// Where `save_report` saved the report, or why it could not.
    pub saved_to: Option<String>,
    pub save_error: Option<String>,
}
