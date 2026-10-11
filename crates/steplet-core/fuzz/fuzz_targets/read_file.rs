//! Any bytes: no crash anywhere in the file core, cleaning up twice changes nothing,
//! and display pieces always join back into the text they came from.
#![no_main]

use libfuzzer_sys::fuzz_target;
use steplet_core::canon::canonicalize;
use steplet_core::display::{Segment, segments};
use steplet_core::{NAMESPACE, header, limits, sig};

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
    let Ok(c) = canonicalize(raw) else { return };
    assert_eq!(canonicalize(c.text().as_bytes()).as_ref(), Ok(&c));
    let _ = sig::verify(&c, NAMESPACE);
    assert_eq!(source(&segments(c.body())), c.body());
    if let Ok(s) = header::parse(&c) {
        let _ = limits::check(&s);
        for item in s.steps.iter().chain(&s.never).chain(&s.allow.commands) {
            assert_eq!(source(&segments(&item.text)), item.text);
        }
    }
});
