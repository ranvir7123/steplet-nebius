//! `steplet-sign check`: everything the file core finds, in words, with line numbers and examples.

use steplet_core::NAMESPACE;
use steplet_core::canon::{Canonical, canonicalize};
use steplet_core::date::Date;
use steplet_core::header::{self, Steplet};
use steplet_core::limits::{self, DEFAULT_EXPIRY_DAYS, MAX_EXPIRY_DAYS};
use steplet_core::sig::{self, SigStatus};

pub struct Report {
    pub text: String,
    pub ok: bool,
}

pub fn check(name: &str, raw: &[u8], today: Date) -> Report {
    let mut info = Vec::new();
    let mut problems = Vec::new();
    match canonicalize(raw) {
        Err(e) => problems.push(problem(&e.to_string(), e.help().allowed, e.help().example)),
        Ok(c) => {
            let (steplet, found) = content(&c, today);
            if let Some(s) = steplet {
                info.push(summary(&s));
                info.push(match &s.repository {
                    Some(r) => format!("repository: {} at {}", r.url.text, r.commit.text),
                    None => "repository: none".into(),
                });
                info.push(format!("expires: {} ({})", s.expires, days_text(s.expires, today)));
            }
            problems.extend(found);
            match sig::verify(&c, NAMESPACE) {
                SigStatus::Unsigned => info.push("signature: not signed".into()),
                SigStatus::Valid { fingerprint } => info.push(format!("signature: valid, key {fingerprint}")),
                SigStatus::Invalid { reason } => problems.push(problem(
                    &format!("signature: {reason}"),
                    "a signature made by steplet-sign over the file exactly as it is now",
                    "steplet-sign sign my-task.steplet --key C:\\Users\\you\\.ssh\\id_ed25519",
                )),
            }
        }
    }
    let ok = problems.is_empty();
    let mut text = if ok { format!("{name}: ok\n") } else { format!("{name}: {}\n", count(problems.len(), "problem")) };
    for line in info {
        text.push_str("  ");
        text.push_str(&line);
        text.push('\n');
    }
    for p in problems {
        text.push_str(&p);
    }
    Report { text, ok }
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
