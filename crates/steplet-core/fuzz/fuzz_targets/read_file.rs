//! Any bytes: no crash anywhere in the file core; cleaning up twice changes nothing; display pieces always join
//! back into the text they came from; the report is plain ASCII; the rating's outcome is its worst finding; and the
//! checker answers for any command, path or address.
#![no_main]

use libfuzzer_sys::fuzz_target;
use steplet_core::canon::canonicalize;
use steplet_core::display::{Segment, segments};
use steplet_core::policy::{Action, Policy};
use steplet_core::rules::{self, Outcome};
use steplet_core::{NAMESPACE, command, header, limits, report, sig};

/// 2026-10-11 12:00 UTC, as in the corpus.
const NOW: i64 = 1_791_720_000;

fn source(segs: &[Segment]) -> String {
    segs.iter()
        .map(|s| match s {
            Segment::Text(x) => x.clone(),
            Segment::InvisibleBadge { codepoint } => char::from_u32(*codepoint).unwrap().to_string(),
            Segment::Link { shown: Some(shown), real } => format!("[{}]({})", source(shown), source(real)),
            Segment::Link { shown: None, real } => source(real),
        })
        .collect()
}

fuzz_target!(|raw: &[u8]| {
    assert!(report::check(raw, NOW).to_json().is_ascii());
    let Ok(c) = canonicalize(raw) else { return };
    assert_eq!(canonicalize(c.text().as_bytes()).as_ref(), Ok(&c));
    let status = sig::verify(&c, NAMESPACE);
    assert_eq!(source(&segments(c.body())), c.body());
    for line in c.body().lines() {
        let _ = command::assess(line);
    }
    if let Ok(s) = header::parse(&c) {
        let _ = limits::check(&s);
        let rating = rules::rate(&s, &status, NOW);
        let worst = rating.findings.iter().map(|f| f.rule.severity).max();
        assert_eq!(rating.outcome, worst.map_or(Outcome::Green, Outcome::of));
        let policy = Policy::from_steplet(&s, "/w/r1").unwrap();
        let items = s.steps.iter().chain(&s.never).chain(&s.allow.commands).chain(&s.allow.domains);
        for item in items.chain(&s.allow.read).chain(&s.allow.write) {
            assert_eq!(source(&segments(&item.text)), item.text);
            // As given, inside the working copy, and climbing out of it: each reaches different checker code.
            let inside = format!("/w/r1/{}", item.text);
            let climbing = format!("/w/r1/../{}", item.text);
            for text in [item.text.as_str(), inside.as_str(), climbing.as_str()] {
                for action in [Action::Command(text), Action::Read(text), Action::Write(text), Action::Fetch(text)] {
                    let _ = policy.check(&action);
                }
            }
        }
    }
});
