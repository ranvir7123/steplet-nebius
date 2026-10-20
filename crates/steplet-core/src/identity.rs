//! Who signed a file, as far as this computer knows (rows 16 and 22, R14, smaller choices S5 and S6; phase 4 Task 7).
//! Trust on first use: the app keeps a pin for each sender name it has run a file from, with that file's signing key.
//! The finding `id.key-changed` is a rule like every other, but only the app applies it, since it needs this
//! computer's pins (R10).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::rules::{self, Finding};
use crate::sig::SigStatus;

/// The identity line, shown beside the colour (row 22).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Identity {
    /// "Not signed".
    Unsigned,
    /// The signature does not match the file; `sig.invalid` already blocks it.
    Invalid,
    /// "Signature valid, sender unknown" (R14): this key is not known here.
    NewKey { key: String },
    /// Signed with the same key as the files from this sender that were run here before (`runs` of them).
    SameKey { key: String, runs: u32 },
    /// The key is known here, but only under other sender names (`names`, each as it was written).
    KeyKnownAs { key: String, names: Vec<String> },
}

/// Steplet's note that the sender name `sender` was seen with the signing key `key` (its fingerprint, as `sig::verify`
/// gives it), and how many files from them were run here. An unsigned file leaves no pin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    pub sender: String,
    pub key: String,
    pub runs: u32,
}

/// The identity line for a file from `sender` with the signature `sig`, and the red finding `id.key-changed` when
/// `sender` is pinned to another key, or the file is unsigned under a pinned sender name (S6). Sender names are
/// compared ignoring capitals and spaces, so writing a known name differently does not dodge the finding.
pub fn identity(sender: &str, sig: &SigStatus, pins: &[Pin]) -> (Identity, Option<Finding>) {
    let name = comparable(sender);
    let known: Vec<&Pin> = pins.iter().filter(|p| comparable(&p.sender) == name).collect();
    match sig {
        SigStatus::Invalid { .. } => (Identity::Invalid, None),
        SigStatus::Unsigned => (Identity::Unsigned, key_changed(sender, &known, None)),
        SigStatus::Valid { fingerprint } => {
            if let Some(p) = known.iter().find(|p| p.key == *fingerprint) {
                return (Identity::SameKey { key: fingerprint.clone(), runs: p.runs }, None);
            }
            let mut names: Vec<String> = Vec::new();
            for p in pins.iter().filter(|p| p.key == *fingerprint) {
                if !names.contains(&p.sender) {
                    names.push(p.sender.clone());
                }
            }
            let line = if names.is_empty() {
                Identity::NewKey { key: fingerprint.clone() }
            } else {
                Identity::KeyKnownAs { key: fingerprint.clone(), names }
            };
            (line, key_changed(sender, &known, Some(fingerprint)))
        }
    }
}

/// The finding for a file from `sender`, whose name is pinned to the keys of `known` (none: no finding), signed with
/// `key` or not signed. The keys are Steplet's own fingerprints; the name is file text, so it goes in the quote.
fn key_changed(sender: &str, known: &[&Pin], key: Option<&String>) -> Option<Finding> {
    let mut keys: Vec<&str> = Vec::new();
    for p in known {
        if !keys.contains(&p.key.as_str()) {
            keys.push(&p.key);
        }
    }
    if keys.is_empty() {
        return None;
    }
    let this = match key {
        Some(k) => format!("This file is signed with {k}."),
        None => String::from("This file is not signed."),
    };
    Some(Finding {
        rule: rules::KEY_CHANGED,
        line: None,
        detail: Some(format!("Steplet knows this sender name with the key {}. {this}", keys.join(", "))),
        quote: Some(sender.into()),
    })
}

/// A sender name as it is compared: lower case, with each run of spaces as one space and none at either end.
fn comparable(name: &str) -> String {
    let words: Vec<String> = name.split_whitespace().map(str::to_lowercase).collect();
    words.join(" ")
}
