//! The senders this computer knows (rows 16 and 22, smaller choice S5): a pin for each sender name it has run a
//! signed file from, with that file's signing key, in `%LOCALAPPDATA%\Steplet\senders.json`. Read when a file is
//! opened; written only when Run is pressed (Task 12), through a temporary file and a rename, so the file is always
//! whole. One pin per sender name (compared as the core compares them), never an empty name, and a key only in the
//! form Steplet's signature check gives it. A file that breaks any of that is kept as it is and said, never
//! overwritten: Steplet then cannot warn about a changed key, and the preview says so.

use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use steplet_core::identity::{Pin, same_sender};
use steplet_core::limits::MAX_SENDER_CHARS;

/// The pins file is small; a bigger one is not Steplet's.
const MAX_PINS_BYTES: u64 = 1 << 20;
const VERSION: u32 = 1;

/// What the pins file holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pins {
    /// The senders known here (none when there is no file yet).
    Known(Vec<Pin>),
    /// The file is there but Steplet cannot read it as its list; why, in Steplet's words.
    Unreadable(String),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Saved {
    version: u32,
    senders: Vec<SavedPin>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedPin {
    sender: String,
    key: String,
    runs: u32,
}

/// Where Steplet keeps the pins: `%LOCALAPPDATA%\Steplet\senders.json`.
pub fn location() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("Steplet").join("senders.json"))
}

/// A key as `steplet_core::sig::verify` gives it: `SHA256:` and 43 characters of unpadded base64.
fn is_key(key: &str) -> bool {
    key.strip_prefix("SHA256:")
        .is_some_and(|b| b.len() == 43 && b.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'+' || c == b'/'))
}

fn is_name(name: &str) -> bool {
    !name.trim().is_empty() && name.chars().count() <= MAX_SENDER_CHARS && !name.chars().any(char::is_control)
}

/// The pins in `path`. Reading changes nothing.
pub fn load(path: &Path) -> Pins {
    let bytes = match std::fs::metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Pins::Known(Vec::new()),
        Err(e) => return Pins::Unreadable(format!("Windows would not show the file: {e}")),
        Ok(m) if m.len() > MAX_PINS_BYTES => {
            return Pins::Unreadable("the file is far bigger than Steplet's list".into());
        }
        Ok(_) => match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => return Pins::Unreadable(format!("Windows would not read the file: {e}")),
        },
    };
    let saved: Saved = match serde_json::from_slice(&bytes) {
        Ok(s) => s,
        Err(_) => return Pins::Unreadable("it is not Steplet's list of senders".into()),
    };
    if saved.version != VERSION {
        return Pins::Unreadable("it is a version of the list this Steplet does not know".into());
    }
    let mut pins: Vec<Pin> = Vec::new();
    for p in saved.senders {
        if !is_name(&p.sender) || !is_key(&p.key) || p.runs == 0 {
            return Pins::Unreadable("an entry in it is not one Steplet writes".into());
        }
        if pins.iter().any(|q| same_sender(&q.sender, &p.sender)) {
            return Pins::Unreadable("it holds one sender name twice".into());
        }
        pins.push(Pin { sender: p.sender, key: p.key, runs: p.runs });
    }
    Pins::Known(pins)
}

/// Records that a file from `sender`, signed with `key`, is being run: the sender's pin gets this key (a changed key
/// replaces the old one, S5), and its count of runs goes up by one when the key is the same. Refused for an empty
/// name, a key not in Steplet's form, and a pins file Steplet cannot read (it is kept as it is).
pub fn record(path: &Path, sender: &str, key: &str) -> io::Result<()> {
    if !is_name(sender) || !is_key(key) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Steplet does not pin this sender name or key"));
    }
    let mut pins = match load(path) {
        Pins::Known(pins) => pins,
        Pins::Unreadable(why) => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Steplet kept its list of senders: {why}")));
        }
    };
    match pins.iter_mut().find(|p| same_sender(&p.sender, sender)) {
        // The name keeps the spelling it was first pinned with.
        Some(p) if p.key == key => p.runs = p.runs.saturating_add(1),
        Some(p) => *p = Pin { sender: sender.into(), key: key.into(), runs: 1 },
        None => pins.push(Pin { sender: sender.into(), key: key.into(), runs: 1 }),
    }
    let saved = Saved {
        version: VERSION,
        senders: pins.into_iter().map(|p| SavedPin { sender: p.sender, key: p.key, runs: p.runs }).collect(),
    };
    let text = serde_json::to_string_pretty(&saved).map_err(io::Error::other)? + "\n";
    let dir = path.parent().ok_or_else(|| io::Error::other("the pins file has no folder"))?;
    std::fs::create_dir_all(dir)?;
    let part = path.with_extension("json.part");
    std::fs::write(&part, text)?;
    std::fs::rename(&part, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&part);
    })
}
