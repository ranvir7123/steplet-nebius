//! Opening a file (Task 10): read at most 256 KiB and one byte of it, rate it with the core (`report::check`), and
//! work out what is true on this computer and not in the file: who signed it as far as the pins know, a damaged pins
//! file, no repository, or a repository Steplet does not fetch from (H9). Opening changes nothing on disk: pins are
//! written only when Run is pressed (S5, Task 12).

use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use steplet_core::identity::identity;
use steplet_core::limits::MAX_FILE_BYTES;
use steplet_core::report::{Report, check};
use steplet_core::rules::{Finding, Rule, Severity};

use crate::pins::Pins;
use crate::views::{self, Here, PreviewView};

/// The most bytes Steplet reads of a file: one more than the core takes, so a bigger file is known to be too big.
pub const MAX_READ: u64 = MAX_FILE_BYTES as u64 + 1;

/// A finding about this computer, not the file: Steplet cannot read the senders it remembers (Task 10). The app's own
/// fact, like the runner's host list; amber, since the key-changed warning cannot be given.
pub static PINS_UNREADABLE: Rule = Rule {
    id: "here.pins-unreadable",
    severity: Severity::Amber,
    title: "Steplet could not read the senders it remembers, so it cannot warn about a changed key",
    advice: "Steplet has left that list (senders.json in Steplet's folder) as it is. Check with the sender another way \
             that this file is theirs before you run it.",
};

/// The first `MAX_READ` bytes of the file at `path`. A refusal is in Steplet's words with Windows' reason, and never
/// names the file (its name could be made to look like Steplet's own words).
pub fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Steplet could not open this file. {}", crate::views::windows_says(e)))?;
    let mut raw = Vec::new();
    file.take(MAX_READ)
        .read_to_end(&mut raw)
        .map_err(|e| format!("Steplet could not read this file. {}", crate::views::windows_says(e)))?;
    Ok(raw)
}

/// The preview of the file whose first bytes are `raw`, at `now_unix` (seconds since 1970-01-01 UTC), with the pins
/// this computer holds.
pub fn preview_file(raw: &[u8], now_unix: i64, pins: &Pins) -> PreviewView {
    let raw = &raw[..raw.len().min(MAX_READ as usize)];
    let report = check(raw, now_unix);
    views::preview(raw, &report, &here(&report, pins))
}

/// What is true on this computer and not in the file.
pub fn here(report: &Report, pins: &Pins) -> Here {
    let (known, mut findings) = match pins {
        Pins::Known(pins) => (pins.as_slice(), Vec::new()),
        Pins::Unreadable(why) => {
            (&[][..], vec![Finding { rule: &PINS_UNREADABLE, line: None, detail: Some(why.clone()), quote: None }])
        }
    };
    let identity = match (&report.steplet, &report.signature) {
        (Some(s), Some(sig)) => {
            let (line, finding) = identity(&s.sender.text, sig, known);
            findings.extend(finding);
            Some(line)
        }
        _ => None,
    };
    let not_runnable = report.steplet.as_ref().and_then(|s| match &s.repository {
        None => Some(views::NO_REPOSITORY.to_string()),
        Some(r) if !steplet_run::fetchable(&r.url.text) => Some(views::not_fetched_from_here()),
        Some(_) => None,
    });
    Here { identity, findings, not_runnable }
}

/// The time now, for expiry (row 30: checked again when Run is pressed).
pub fn now_unix() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}
