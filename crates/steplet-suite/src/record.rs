//! The suite's records (phase 5): the whole output of a run, kept as plain text in `docs/suite/`, one file per layer,
//! agent (Claude Code by its version, Steplet's own agent) and day, and never written over. Each record is counted for
//! a key's text before it is kept.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::catalogue::{Layer, Runner};

/// A day in this computer's time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Day {
    pub year: u16,
    pub month: u16,
    pub day: u16,
}

/// Now, in this computer's time: the day, and the time as `HH:MM`.
pub fn now() -> (Day, String) {
    use windows_sys::Win32::System::SystemInformation::GetLocalTime;
    // SAFETY: GetLocalTime only fills the structure it is given.
    let t = unsafe {
        let mut t = std::mem::zeroed();
        GetLocalTime(&mut t);
        t
    };
    (Day { year: t.wYear, month: t.wMonth, day: t.wDay }, format!("{:02}:{:02}", t.wHour, t.wMinute))
}

/// `<date>-<agents>-<layer>.txt`, where `<agents>` names the agents the cases ran: `claude-<version>`,
/// `steplet-agent`, or both joined by `-and-`. A run of some cases only says which (`…-scripted-only-G-W1.txt`).
pub fn record_name(day: &Day, version: &str, layer: Layer, runners: &[Runner], only: Option<&str>) -> String {
    let agents: Vec<String> = runners
        .iter()
        .map(|r| match r {
            Runner::Claude => format!("claude-{version}"),
            Runner::Own => "steplet-agent".to_owned(),
        })
        .collect();
    let only = only
        .map(|o| format!("-only-{}", o.split(',').map(str::trim).collect::<Vec<_>>().join("-")))
        .unwrap_or_default();
    format!("{:04}-{:02}-{:02}-{}-{}{only}.txt", day.year, day.month, day.day, agents.join("-and-"), layer.name())
}

/// A kept record, and how many times each key's text was in it before it was kept.
#[derive(Debug)]
pub struct Kept {
    pub path: PathBuf,
    /// Each key's label (never its text) and its count.
    pub counts: Vec<(String, usize)>,
}

impl Kept {
    pub fn clean(&self) -> bool {
        self.counts.iter().all(|(_, n)| *n == 0)
    }
}

/// Keeps `text` as the record `name` in `dir`, or as `name-2`, `name-3`, … when that name is taken: an existing record
/// is never written over. Each key's text is counted and replaced by `[hidden by Steplet]` before anything is written,
/// and the record's last line says what was counted.
pub fn keep(dir: &Path, name: &str, text: &str, keys: &[(&str, &str)]) -> io::Result<Kept> {
    let counts: Vec<(String, usize)> = keys
        .iter()
        .map(|(label, key)| ((*label).to_owned(), if key.is_empty() { 0 } else { text.matches(key).count() }))
        .collect();
    let mut kept = text.to_owned();
    if !keys.is_empty() {
        for (_, key) in keys.iter().filter(|(_, k)| !k.is_empty()) {
            kept = kept.replace(key, "[hidden by Steplet]");
        }
        let said: Vec<String> = counts.iter().map(|(label, n)| format!("{label} {n} times")).collect();
        kept += &format!("Counted before this record was kept: {}.", said.join(", "));
        if counts.iter().any(|(_, n)| *n > 0) {
            kept += " The suite does not pass while a key's text is in its record; each one above was replaced by \
                     [hidden by Steplet].";
        }
        kept += "\n";
    }
    fs::create_dir_all(dir)?;
    let (stem, ending) = name.rsplit_once('.').map(|(s, e)| (s, format!(".{e}"))).unwrap_or((name, String::new()));
    for n in 1.. {
        let path = if n == 1 { dir.join(name) } else { dir.join(format!("{stem}-{n}{ending}")) };
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        };
        if let Err(e) = file.write_all(kept.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(e);
        }
        return Ok(Kept { path, counts });
    }
    unreachable!("an endless count of names")
}
