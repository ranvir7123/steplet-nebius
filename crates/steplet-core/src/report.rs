//! One call for a whole file: clean it up, check its signature, read its header and rate it. The app's preview
//! and the check page both call this (row 29), and both show `to_json`'s text (G11).

use alloc::string::{String, ToString};
use alloc::vec;
use core::fmt::Write;

use crate::NAMESPACE;
use crate::canon::{self, CanonError};
use crate::header::{self, ParseErrorKind, Steplet};
use crate::rules::{self, Finding, Rating, Rule};
use crate::sig::{self, SigStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub rating: Rating,
    /// `None` when the file could not be cleaned up, so its signature could not be checked.
    pub signature: Option<SigStatus>,
    /// `None` when the header could not be read.
    pub steplet: Option<Steplet>,
}

/// The report for `raw`, the bytes of a file, at `now_unix` (seconds since 1970-01-01 UTC).
pub fn check(raw: &[u8], now_unix: i64) -> Report {
    let c = match canon::canonicalize(raw) {
        Ok(c) => c,
        Err(e) => {
            let (rule, line) = match e {
                CanonError::FileTooLarge { .. } | CanonError::HeaderTooLarge { .. } => (rules::READ_TOO_LARGE, None),
                CanonError::NoHeaderStart => (rules::READ_REFUSED, Some(1)),
                CanonError::NoHeaderEnd => (rules::READ_REFUSED, None),
                CanonError::NotUtf8 { line }
                | CanonError::LoneCarriageReturn { line }
                | CanonError::SignatureMisplaced { line }
                | CanonError::SignatureTwice { line }
                | CanonError::SignatureBroken { line } => (rules::READ_REFUSED, Some(line)),
            };
            let found = refused(rule, line, rules::without_line(e.to_string()));
            return Report { rating: Rating::from_findings(vec![found]), signature: None, steplet: None };
        }
    };
    let signature = sig::verify(&c, NAMESPACE);
    match header::parse(&c) {
        Ok(s) => Report { rating: rules::rate(&s, &signature, now_unix), signature: Some(signature), steplet: Some(s) },
        Err(e) => {
            // Two of the reader's messages repeat free text from the file; the rating uses its own words (row 28). The
            // others repeat at most a key name (lowercase letters, digits and hyphens only), which passes through.
            let detail = match e.kind {
                ParseErrorKind::UnsupportedVersion { .. } => {
                    String::from("a format version other than 1; this Steplet reads version 1 only")
                }
                ParseErrorKind::BadEscape { .. } => String::from(
                    "a backslash inside double quotes before something other than a quote or a backslash",
                ),
                _ => rules::without_line(e.to_string()),
            };
            let found = refused(rules::READ_REFUSED, Some(e.line), detail);
            // The signature's status is still reported. No `sig.invalid` finding is added: there is no file to rate,
            // and `read.refused` already blocks it.
            Report { rating: Rating::from_findings(vec![found]), signature: Some(signature), steplet: None }
        }
    }
}

fn refused(rule: &'static Rule, line: Option<u32>, detail: String) -> Finding {
    Finding { rule, line, detail: Some(detail), quote: None }
}

impl Report {
    /// The report as JSON, one finding per line, in plain ASCII (G11).
    pub fn to_json(&self) -> String {
        let mut o = String::from("{\n  \"outcome\": ");
        string(&mut o, self.rating.outcome.name());
        o.push_str(",\n  \"signature\": ");
        match &self.signature {
            None => o.push_str("{\"status\": \"unchecked\"}"),
            Some(SigStatus::Unsigned) => o.push_str("{\"status\": \"unsigned\"}"),
            Some(SigStatus::Invalid { .. }) => o.push_str("{\"status\": \"invalid\"}"),
            Some(SigStatus::Valid { fingerprint }) => {
                o.push_str("{\"status\": \"valid\", \"key\": ");
                string(&mut o, fingerprint);
                o.push('}');
            }
        }
        o.push_str(",\n  \"findings\": [");
        for (i, f) in self.rating.findings.iter().enumerate() {
            o.push_str(if i == 0 { "\n    " } else { ",\n    " });
            o.push_str("{\"rule\": ");
            string(&mut o, f.rule.id);
            o.push_str(", \"severity\": ");
            string(&mut o, f.rule.severity.name());
            o.push_str(", \"line\": ");
            match f.line {
                Some(n) => {
                    let _ = write!(o, "{n}");
                }
                None => o.push_str("null"),
            }
            o.push_str(", \"title\": ");
            string(&mut o, f.rule.title);
            o.push_str(", \"detail\": ");
            optional(&mut o, f.detail.as_deref());
            o.push_str(", \"advice\": ");
            string(&mut o, f.rule.advice);
            o.push_str(", \"quote\": ");
            optional(&mut o, f.quote.as_deref());
            o.push('}');
        }
        if !self.rating.findings.is_empty() {
            o.push_str("\n  ");
        }
        o.push_str("]\n}\n");
        o
    }
}

fn optional(o: &mut String, s: Option<&str>) {
    match s {
        Some(s) => string(o, s),
        None => o.push_str("null"),
    }
}

fn string(o: &mut String, s: &str) {
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            ' '..='~' => o.push(c),
            _ => {
                let mut units = [0u16; 2];
                for u in c.encode_utf16(&mut units).iter() {
                    let _ = write!(o, "\\u{u:04x}");
                }
            }
        }
    }
    o.push('"');
}
