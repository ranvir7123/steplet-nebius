//! The rating (decision rows 20 to 27, R9, R11, G3, G7 to G10): code rules only, each with a fixed id, a severity
//! and plain words. The outcome is the worst single finding (row 21); notes never change it. A finding never puts
//! text from the file into Steplet's own words (row 28): that text travels separately, as `quote`.
//! The phrase rules follow ideas from Cisco's skill-scanner (Apache-2.0,
//! https://github.com/cisco-ai-defense/skill-scanner, read at commit f18074a5), rewritten as word lists.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Reverse;

use crate::command::{self, Assessment, Blocked, Kind};
use crate::display::{self, Segment};
use crate::header::{Scalar, Steplet};
use crate::hosts;
use crate::limits::{self, Limited};
use crate::paths;
use crate::prose;
use crate::sig::{InvalidReason, SigStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Note,
    Amber,
    Red,
    Blocked,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Severity::Note => "note",
            Severity::Amber => "amber",
            Severity::Red => "red",
            Severity::Blocked => "blocked",
        }
    }
}

/// R9's outcomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Green,
    Amber,
    RedFindings,
    Blocked,
}

impl Outcome {
    pub fn of(worst: Severity) -> Outcome {
        match worst {
            Severity::Note => Outcome::Green,
            Severity::Amber => Outcome::Amber,
            Severity::Red => Outcome::RedFindings,
            Severity::Blocked => Outcome::Blocked,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Outcome::Green => "green",
            Outcome::Amber => "amber",
            Outcome::RedFindings => "red",
            Outcome::Blocked => "blocked",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Rule {
    /// Fixed for good: screens, corpus file names and docs/rules.md use it.
    pub id: &'static str,
    pub severity: Severity,
    /// What was found, in Steplet's own words.
    pub title: &'static str,
    /// What the receiver, and the sender, can do about it.
    pub advice: &'static str,
}

const fn rule(id: &'static str, severity: Severity, title: &'static str, advice: &'static str) -> Rule {
    Rule { id, severity, title, advice }
}

use self::Severity::{Amber as A, Blocked as B, Note as N, Red as R};

pub const READ_REFUSED: &Rule = &rule("read.refused", B, "Steplet cannot read this file",
    "Ask the sender to fix the line named here; steplet-sign check shows them what is allowed.");
pub const READ_TOO_LARGE: &Rule = &rule("read.too-large", B, "The file is bigger than Steplet allows",
    "Everything must fit on your screen. The sender can split the task into several files.");
pub const READ_LINE_TOO_LONG: &Rule = &rule("read.line-too-long", B, "A line of the header is longer than Steplet allows",
    "The sender can shorten it and put the details in the body.");
pub const SIG_INVALID: &Rule = &rule("sig.invalid", B, "The signature does not match this file",
    "The file was changed after it was signed, or the signature was damaged. Ask the sender for a fresh copy.");
pub const TIME_EXPIRED: &Rule = &rule("time.expired", B, "The file has expired",
    "Ask the sender for a new file with a later expiry date.");
pub const TIME_TOO_FAR: &Rule = &rule("time.too-far", B, "The file expires more than 365 days from today",
    "Files may be valid for at most 365 days. Ask the sender for a file with an earlier expiry date.");
pub const BLOCK_SECRET_PATH: &Rule = &rule("block.secret-path", B, "The file names a place where keys or passwords are kept",
    "No task may touch keys, passwords, wallets or sign-ins. Senders: if this is a warning for the agent, put it in the never list.");
pub const BLOCK_ADMIN: &Rule = &rule("block.admin", B, "A command needs administrator rights",
    "Steplet never gives a task administrator rights. Senders: remove sudo or similar from the command.");
pub const BLOCK_DOWNLOAD_INTO_SHELL: &Rule = &rule("block.download-into-shell", B,
    "The file downloads a script and runs it straight away",
    "This is never allowed, even from an official site. Senders: use a package manager, or put the script in the repository.");
pub const BLOCK_REGISTRY: &Rule = &rule("block.registry", B, "A command points a package manager at another source",
    "Steplet does not let a task change where packages come from. Senders: use the normal registry.");
pub const BLOCK_INVISIBLE_IN_FIELD: &Rule = &rule("block.invisible-in-field", B,
    "An invisible character in the title, the sender or a command",
    "Invisible characters can make a name or a command look different from what it is. Senders: remove them.");
pub const BLOCK_WRITE_PROTECTED: &Rule = &rule("block.write-protected", B, "The file asks to change .git or an agent's settings",
    "No task may change .git, .claude, .codex, .gemini, .cursor or .mcp.json. Senders: remove it from allow: write.");
pub const HIDE_INVISIBLE_IN_TEXT: &Rule = &rule("hide.invisible-in-text", R, "Invisible characters in the text",
    "They can hide words from you that the agent still reads. Each one is shown as a badge; check them before running.");
pub const HIDE_HTML_COMMENT: &Rule = &rule("hide.html-comment", R, "An HTML comment in the text",
    "Many Markdown viewers hide HTML comments, but the agent reads them. Steplet shows them; read them before running.");
pub const HIDE_ENCODED_BLOCK: &Rule = &rule("hide.encoded-block", R, "A long block of encoded characters",
    "Encoded text (base64 or hex) can hide instructions or a program. Ask the sender what it is.");
pub const HIDE_BLANK_RUN: &Rule = &rule("hide.blank-run", R, "Many empty lines in a row",
    "A long run of empty lines can push text out of view. Scroll through the whole body before running.");
pub const HIDE_LINK_MISMATCH: &Rule = &rule("hide.link-mismatch", R, "A link shows one address but opens another",
    "The text of the link names a different site from the one it opens. Steplet shows the real address.");
pub const HIDE_PUNYCODE: &Rule = &rule("hide.punycode", R, "A web address written in another alphabet",
    "A name with xn-- stands for letters of another alphabet, which can imitate a well-known site.");
pub const SOCIAL_OVERRIDE: &Rule = &rule("social.override", R, "The text tells the agent to ignore its earlier instructions",
    "This is a common way to take over an agent. Run it only if you trust the sender completely.");
pub const SOCIAL_CONCEAL: &Rule = &rule("social.conceal", R, "The text tells the agent to hide something from you",
    "An honest task has nothing to hide from the person running it.");
pub const SOCIAL_DISABLE_SAFETY: &Rule = &rule("social.disable-safety", R, "The text tells the agent to switch off its safety rules",
    "An honest task does not need the agent's safety rules switched off.");
pub const SOCIAL_CLAIMS_APPROVAL: &Rule = &rule("social.claims-approval", R, "The text says the run was already approved",
    "Only you approve a run, on Steplet's own screens. Text in a file cannot approve anything.");
pub const SOCIAL_ASKS_SECRETS: &Rule = &rule("social.asks-secrets", R, "The text asks for a password, a key or a token",
    "A task like this never needs your passwords, keys or tokens.");
pub const SOCIAL_ASKS_ADMIN: &Rule = &rule("social.asks-admin", R, "The text asks for administrator rights",
    "Steplet never gives a task administrator rights, so a step that needs them will fail.");
pub const SOCIAL_DISABLE_PROTECTION: &Rule = &rule("social.disable-protection", R,
    "The text asks to switch off antivirus, the firewall or other protection",
    "An honest task does not need your computer's protection switched off.");
pub const SOCIAL_LOOKALIKE_HOST: &Rule = &rule("social.lookalike-host", R, "A web address that imitates a well-known site",
    "The name is one letter away from a well-known site, or has a different ending. Check it carefully.");
pub const NET_EXFIL_DOMAIN: &Rule = &rule("net.exfil-domain", R, "The file lets data go to a paste, tunnel or webhook site",
    "These sites are often used to collect stolen data. A test task rarely needs them.");
pub const CAP_RUNS_CODE: &Rule = &rule("cap.runs-code", A, "A command runs the repository's own code",
    "Tests, builds and scripts run code from the repository. Steplet runs them only inside the fence.");
pub const CAP_INSTALLS: &Rule = &rule("cap.installs", A, "A command downloads and installs packages",
    "Packages are other people's code and can run while they install. Steplet runs this only inside the fence.");
pub const CAP_UNKNOWN_PROGRAM: &Rule = &rule("cap.unknown-program", A, "A command uses a program Steplet does not know",
    "Steplet treats it as able to run any code, so it runs only inside the fence.");
pub const CAP_NOT_UNDERSTOOD: &Rule = &rule("cap.not-understood", A, "Steplet cannot read a command exactly",
    "Commands are matched word for word, so the agent will have to ask you each time. Senders: write one plain command, without pipes, semicolons, &&, $, redirections or wildcards.");
pub const CAP_OUTSIDE_PATH: &Rule = &rule("cap.outside-path", A, "A command names a path outside the repository",
    "Programs inside the fence can still read some places outside the repository. Check that this one is needed.");
pub const NET_OTHER_DOMAIN: &Rule = &rule("net.other-domain", A, "The file lets the agent and its programs reach a web site",
    "Data can be sent to every site on this list. Package registries Steplet knows are not shown here.");
pub const CONSIST_UNDECLARED_COMMAND: &Rule = &rule("consist.undeclared-command", A,
    "The instructions mention a command the file does not ask for",
    "The agent will have to ask you before running it. Senders: add it to allow: commands.");
pub const CONSIST_UNDECLARED_HOST: &Rule = &rule("consist.undeclared-host", A, "A step mentions a web site the file does not ask for",
    "The agent will have to ask you before opening it. Senders: add it to allow: domains.");
pub const NOTE_NO_REPOSITORY: &Rule = &rule("note.no-repository", N, "The file names no repository",
    "Steplet 1 runs only tasks on a public repository, so this file can be read but not run.");

/// Every rule, in the order findings of the same severity and line are listed, and docs/rules.md lists them.
/// Every rule `const` above must be in this list: nothing checks it, and a rule left out would sort first, be
/// missing from docs/rules.md, and have no corpus pair.
pub const RULES: &[&Rule] = &[
    READ_REFUSED, READ_TOO_LARGE, READ_LINE_TOO_LONG, SIG_INVALID, TIME_EXPIRED, TIME_TOO_FAR,
    BLOCK_SECRET_PATH, BLOCK_ADMIN, BLOCK_DOWNLOAD_INTO_SHELL, BLOCK_REGISTRY, BLOCK_INVISIBLE_IN_FIELD, BLOCK_WRITE_PROTECTED,
    HIDE_INVISIBLE_IN_TEXT, HIDE_HTML_COMMENT, HIDE_ENCODED_BLOCK, HIDE_BLANK_RUN, HIDE_LINK_MISMATCH, HIDE_PUNYCODE,
    SOCIAL_OVERRIDE, SOCIAL_CONCEAL, SOCIAL_DISABLE_SAFETY, SOCIAL_CLAIMS_APPROVAL, SOCIAL_ASKS_SECRETS, SOCIAL_ASKS_ADMIN,
    SOCIAL_DISABLE_PROTECTION, SOCIAL_LOOKALIKE_HOST, NET_EXFIL_DOMAIN,
    CAP_RUNS_CODE, CAP_INSTALLS, CAP_UNKNOWN_PROGRAM, CAP_NOT_UNDERSTOOD, CAP_OUTSIDE_PATH, NET_OTHER_DOMAIN,
    CONSIST_UNDECLARED_COMMAND, CONSIST_UNDECLARED_HOST,
    NOTE_NO_REPOSITORY,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static Rule,
    /// The line of the file the finding is about; `None` when it is about the whole file.
    pub line: Option<u32>,
    /// More detail in Steplet's own words, when the title is not enough. Never text from the file (row 28).
    pub detail: Option<String>,
    /// The text from the file this finding is about. Screens draw it as file text (display segments).
    pub quote: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rating {
    pub outcome: Outcome,
    /// Worst first; then by line (findings about the whole file first); then in the order of `RULES`.
    pub findings: Vec<Finding>,
}

impl Rating {
    pub fn from_findings(mut findings: Vec<Finding>) -> Rating {
        findings.sort_by_key(|f| (Reverse(f.rule.severity), f.line, RULES.iter().position(|r| r.id == f.rule.id)));
        let outcome = findings.first().map_or(Outcome::Green, |f| Outcome::of(f.rule.severity));
        Rating { outcome, findings }
    }
}

/// The most characters a finding quotes. A body line has no length limit, and one long line can be quoted by
/// several findings, so a longer quote is cut on a character boundary and marked.
const QUOTE_CHARS: usize = 300;

struct Out(Vec<Finding>);

impl Out {
    fn add(&mut self, rule: &'static Rule, line: Option<u32>, detail: Option<String>, quote: Option<&str>) {
        let quote = quote.map(|q| match q.char_indices().nth(QUOTE_CHARS) {
            Some((end, _)) => format!("{} (cut)", &q[..end]),
            None => String::from(q),
        });
        self.0.push(Finding { rule, line, detail, quote });
    }
}

/// Rates a parsed file (R9). `sig` is its signature check, `now_unix` the time of viewing (seconds since
/// 1970-01-01 UTC). Findings that depend on the receiver's computer are the app's (R10, phase 4).
pub fn rate(s: &Steplet, sig: &SigStatus, now_unix: i64) -> Rating {
    let mut out = Out(Vec::new());
    signature_and_time(&mut out, s, sig, now_unix);
    for b in limits::check(s) {
        let rule = match b.what {
            Limited::CommandLength | Limited::TitleLength | Limited::SenderLength | Limited::ValueLength => {
                READ_LINE_TOO_LONG
            }
            _ => READ_TOO_LARGE,
        };
        out.add(rule, Some(b.line), Some(without_line(b.to_string())), None);
    }
    for c in &s.allow.commands {
        command_finding(&mut out, c);
    }
    for field in [&s.title, &s.sender] {
        if field.text.chars().any(prose::is_hiding) {
            out.add(BLOCK_INVISIBLE_IN_FIELD, Some(field.line), None, Some(&field.text));
        }
    }
    for w in &s.allow.write {
        if paths::is_protected(&w.text) {
            out.add(BLOCK_WRITE_PROTECTED, Some(w.line), None, Some(&w.text));
        }
    }
    host_rules(&mut out, s);
    text_rules(&mut out, s);
    consistency(&mut out, s);
    if s.repository.is_none() {
        out.add(NOTE_NO_REPOSITORY, None, None, None);
    }
    Rating::from_findings(out.0)
}

/// `line 12: 31 steps; …` without its `line 12: ` (screens show the line on its own).
pub(crate) fn without_line(message: String) -> String {
    match message.strip_prefix("line ").and_then(|m| m.split_once(": ")) {
        Some((_, rest)) => rest.to_string(),
        None => message,
    }
}

fn signature_and_time(out: &mut Out, s: &Steplet, sig: &SigStatus, now_unix: i64) {
    if let SigStatus::Invalid { reason } = sig {
        let detail = match reason {
            // The name space is text the signer chose, so it is not repeated in Steplet's own words (row 28).
            InvalidReason::WrongNamespace { .. } => {
                String::from("the signature was made for something other than Steplet files")
            }
            other => other.to_string(),
        };
        out.add(SIG_INVALID, None, Some(detail), None);
    }
    if now_unix >= s.expires.end_unix() {
        let detail = format!("it expired at the end of {} (UTC)", s.expires);
        out.add(TIME_EXPIRED, Some(s.expires_line), Some(detail), None);
    } else if s.expires.days_since_epoch() - now_unix.div_euclid(86_400) > limits::MAX_EXPIRY_DAYS {
        let detail = format!("it expires on {}, more than 365 days from today", s.expires);
        out.add(TIME_TOO_FAR, Some(s.expires_line), Some(detail), None);
    }
}

/// At most one finding per command: the worst that `command::assess` gives.
fn command_finding(out: &mut Out, c: &Scalar) {
    let (rule, detail) = match command::assess(&c.text) {
        Assessment::Blocked(Blocked::Invisible) => (BLOCK_INVISIBLE_IN_FIELD, None),
        Assessment::Blocked(Blocked::DownloadIntoShell) => (BLOCK_DOWNLOAD_INTO_SHELL, None),
        Assessment::Blocked(Blocked::Secret(secret)) => {
            (BLOCK_SECRET_PATH, Some(format!("it names a place that holds {}", secret.what)))
        }
        Assessment::Blocked(Blocked::Admin) => (BLOCK_ADMIN, None),
        Assessment::Blocked(Blocked::Registry) => (BLOCK_REGISTRY, None),
        Assessment::NotUnderstood(_) => (CAP_NOT_UNDERSTOOD, None),
        Assessment::Words { outside: true, .. } => (CAP_OUTSIDE_PATH, None),
        Assessment::Words { kind: Kind::ReadOnly, .. } => return,
        Assessment::Words { kind: Kind::RunsCode, .. } => (CAP_RUNS_CODE, None),
        Assessment::Words { kind: Kind::Installs, .. } => (CAP_INSTALLS, None),
        Assessment::Words { kind: Kind::Unknown, .. } => (CAP_UNKNOWN_PROGRAM, None),
    };
    out.add(rule, Some(c.line), detail, Some(&c.text));
}

/// The allowed sites (G9), and every host the file names: in `allow: domains`, the repository, and the links in
/// the title, steps, never lines and body.
fn host_rules(out: &mut Out, s: &Steplet) {
    for d in &s.allow.domains {
        if hosts::is_exfil_site(&d.text) {
            out.add(NET_EXFIL_DOMAIN, Some(d.line), None, Some(&d.text));
        } else if !hosts::is_known_registry(&d.text) {
            out.add(NET_OTHER_DOMAIN, Some(d.line), None, Some(&d.text));
        }
    }
    let mut named: Vec<(u32, String)> = s.allow.domains.iter().map(|d| (d.line, d.text.clone())).collect();
    if let Some(r) = &s.repository {
        named.extend(hosts::host_of(&r.url.text).map(|h| (r.url.line, h)));
    }
    for (_, line, text) in readable(s) {
        named.extend(links(text).into_iter().filter_map(|(_, real)| hosts::host_of(&real)).map(|h| (line, h)));
    }
    named.sort();
    named.dedup();
    for (line, host) in &named {
        if hosts::is_punycode(host) {
            out.add(HIDE_PUNYCODE, Some(*line), None, Some(host.as_str()));
        } else if let Some(famous) = hosts::imitates(host) {
            out.add(SOCIAL_LOOKALIKE_HOST, Some(*line), Some(format!("it looks like {famous}")), Some(host.as_str()));
        }
    }
}

/// Where a piece of text a person reads comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Title,
    Step,
    Never,
    Body,
}

/// The title, steps, never lines and body lines, each with its line in the file.
fn readable(s: &Steplet) -> Vec<(Part, u32, &str)> {
    core::iter::once((Part::Title, s.title.line, s.title.text.as_str()))
        .chain(s.steps.iter().map(|x| (Part::Step, x.line, x.text.as_str())))
        .chain(s.never.iter().map(|x| (Part::Never, x.line, x.text.as_str())))
        .chain(body_lines(s).map(|(line, text)| (Part::Body, line, text)))
        .collect()
}

fn body_lines(s: &Steplet) -> impl Iterator<Item = (u32, &str)> {
    s.body.lines().enumerate().map(move |(i, text)| (s.body_line + i as u32, text))
}

/// The links in `text`: the text shown (for a Markdown link) and the real address.
fn links(text: &str) -> Vec<(Option<String>, String)> {
    display::segments(text)
        .iter()
        .filter_map(|seg| match seg {
            Segment::Link { shown, real } => Some((shown.as_deref().map(joined), joined(real))),
            _ => None,
        })
        .collect()
}

/// Display pieces joined back into their text.
fn joined(segments: &[Segment]) -> String {
    let mut s = String::new();
    for seg in segments {
        match seg {
            Segment::Text(t) => s.push_str(t),
            Segment::InvisibleBadge { codepoint } => s.extend(char::from_u32(*codepoint)),
            Segment::Link { .. } => {}
        }
    }
    s
}

/// Rules on the words a person reads: filled in by Task 7.
fn text_rules(_out: &mut Out, _s: &Steplet) {}

/// The steps and body against what the file allows: filled in by Task 7.
fn consistency(_out: &mut Out, _s: &Steplet) {}
