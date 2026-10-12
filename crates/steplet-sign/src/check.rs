//! `steplet-sign check`: everything the file core finds, in words, with line numbers and examples.

use steplet_core::NAMESPACE;
use steplet_core::canon::{Canonical, canonicalize};
use steplet_core::date::Date;
use steplet_core::display::is_invisible;
use steplet_core::header::{self, Steplet};
use steplet_core::limits::{self, DEFAULT_EXPIRY_DAYS, MAX_EXPIRY_DAYS};
use steplet_core::rules::{self, Finding, Severity};
use steplet_core::sig::{self, SigStatus};

pub struct Report {
    pub text: String,
    pub ok: bool,
}

/// Rules whose findings `check` already reports as problems in its own words, so the rating does not repeat them.
const ALREADY_REPORTED: &[&str] =
    &["read.refused", "read.too-large", "read.line-too-long", "sig.invalid", "time.expired", "time.too-far"];

pub fn check(name: &str, raw: &[u8], today: Date) -> Report {
    let mut info = Vec::new();
    let mut problems = Vec::new();
    let mut rating = String::new();
    let mut blocked = 0;
    match canonicalize(raw) {
        Err(e) => problems.push(problem(&e.to_string(), e.help().allowed, e.help().example)),
        Ok(c) => {
            let (steplet, found) = content(&c, today);
            problems.extend(found);
            let status = sig::verify(&c, NAMESPACE);
            if let Some(s) = &steplet {
                info.push(summary(s));
                info.push(match &s.repository {
                    Some(r) => format!("repository: {} at {}", r.url.text, r.commit.text),
                    None => "repository: none".into(),
                });
                info.push(format!("expires: {} ({})", s.expires, days_text(s.expires, today)));
            }
            match &status {
                SigStatus::Unsigned => info.push("signature: not signed".into()),
                SigStatus::Valid { fingerprint } => info.push(format!("signature: valid, key {fingerprint}")),
                SigStatus::Invalid { reason } => problems.push(problem(
                    &format!("signature: {reason}"),
                    "a signature made by steplet-sign over the file exactly as it is now",
                    "steplet-sign sign my-task.steplet --key C:\\Users\\you\\.ssh\\id_ed25519",
                )),
            }
            // G12: what the receiver will see. The time is the start of today, as for the expiry checks above.
            if let Some(s) = &steplet {
                let r = rules::rate(s, &status, today.days_since_epoch() * 86_400);
                info.push(format!("rating: {}", r.outcome.name()));
                for f in r.findings.iter().filter(|f| !ALREADY_REPORTED.contains(&f.rule.id)) {
                    blocked += usize::from(f.rule.severity == Severity::Blocked);
                    rating.push_str(&finding_text(f));
                }
            }
        }
    }
    let ok = problems.is_empty() && blocked == 0;
    let mut text =
        if ok { format!("{name}: ok\n") } else { format!("{name}: {}\n", count(problems.len() + blocked, "problem")) };
    for line in info {
        text.push_str("  ");
        text.push_str(&line);
        text.push('\n');
    }
    text.push_str(&rating);
    for p in problems {
        text.push_str(&p);
    }
    Report { text, ok }
}

/// One finding of the rating: severity, line and title; the file's own text, with invisible characters shown;
/// and what to do.
fn finding_text(f: &Finding) -> String {
    let mut s = format!("    {}", f.rule.severity.name());
    if let Some(line) = f.line {
        s.push_str(&format!(", line {line}"));
    }
    s.push_str(&format!(": {}", f.rule.title));
    if let Some(detail) = &f.detail {
        s.push_str(&format!(" ({detail})"));
    }
    s.push('\n');
    if let Some(quote) = &f.quote {
        s.push_str(&format!("      {}\n", visible(quote)));
    }
    s.push_str(&format!("      {}\n", f.rule.advice));
    s
}

/// `text` with every invisible character written as <U+XXXX>, so a terminal shows it.
fn visible(text: &str) -> String {
    text.chars().map(|c| if is_invisible(c) { format!("<U+{:04X}>", c as u32) } else { c.to_string() }).collect()
}

/// Header, limit and expiry problems of a cleaned-up file: everything except the signature.
pub fn content(c: &Canonical, today: Date) -> (Option<Steplet>, Vec<String>) {
    let s = match header::parse(c) {
        Ok(s) => s,
        Err(e) => return (None, vec![problem(&e.to_string(), e.help().allowed, e.help().example)]),
    };
    let mut out: Vec<String> =
        limits::check(&s).iter().map(|b| problem(&b.to_string(), b.help().allowed, b.help().example)).collect();
    let days = s.expires.days_since_epoch() - today.days_since_epoch();
    let suggested = today.add_days(DEFAULT_EXPIRY_DAYS).map(|d| format!("expires: {d}")).unwrap_or_default();
    if days < 0 {
        out.push(problem(
            &format!("line {}: this file expired on {}", s.expires_line, s.expires),
            "a date from today to 365 days ahead; 14 days is usual",
            &suggested,
        ));
    } else if days > MAX_EXPIRY_DAYS {
        out.push(problem(
            &format!("line {}: expires {}, more than 365 days from today ({today})", s.expires_line, s.expires),
            "a date from today to 365 days ahead; 14 days is usual",
            &suggested,
        ));
    }
    (Some(s), out)
}

pub fn problem(message: &str, allowed: &str, example: &str) -> String {
    let mut p = format!("  problem: {message}\n    allowed: {allowed}\n");
    if !example.is_empty() {
        p.push_str("    for example:\n");
        for line in example.lines() {
            p.push_str("      ");
            p.push_str(line);
            p.push('\n');
        }
    }
    p
}

fn summary(s: &Steplet) -> String {
    let a = &s.allow;
    format!(
        "{}; allowed: {}, {}, {}, {}; {}",
        count(s.steps.len(), "step"),
        count(a.commands.len(), "command"),
        count(a.domains.len(), "domain"),
        count(a.read.len(), "read folder"),
        count(a.write.len(), "write folder"),
        count(s.never.len(), "never line"),
    )
}

fn count(n: usize, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
}

fn days_text(expires: Date, today: Date) -> String {
    match expires.days_since_epoch() - today.days_since_epoch() {
        0 => "today".into(),
        1 => "in 1 day".into(),
        n if n > 1 => format!("in {n} days"),
        -1 => "1 day ago".into(),
        n => format!("{} days ago", -n),
    }
}
