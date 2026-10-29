//! The receiver's keys (smaller choice S3): the Anthropic key for Claude Code and the Nebius Token Factory key for
//! Steplet's own agent, each one "generic" entry of its own in Windows' own Credential Manager, locked to this Windows
//! account and kept on this computer only, never in a file. "Forget key" removes one entry and leaves the other. A key
//! never goes back to the screens: they learn only whether each one is saved.

use std::io;
use std::ptr;

use steplet_run::ApiKey;
use windows_sys::Win32::Foundation::ERROR_NOT_FOUND;
use windows_sys::Win32::Security::Credentials::{
    CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree, CredReadW, CredWriteW,
};

use crate::views::KeyFor;

/// Steplet's entry for the Anthropic key, named as before the Nebius copy so a key saved then is still found.
pub const ANTHROPIC_TARGET: &str = "Steplet/anthropic-api-key";
/// Steplet's entry for the Nebius Token Factory key.
pub const NEBIUS_TARGET: &str = "Steplet/nebius-api-key";
/// Nebius keys are long signed tokens. Credential Manager keeps at most 2,560 bytes; anything longer than this is
/// something else pasted by mistake.
const MAX_KEY_BYTES: usize = 2048;

/// Where each key is kept: Steplet's own entries (`ENTRIES`), or a test's made-up ones.
#[derive(Clone, Copy, Debug)]
pub struct Entries {
    pub nebius: &'static str,
    pub anthropic: &'static str,
}

pub const ENTRIES: Entries = Entries { nebius: NEBIUS_TARGET, anthropic: ANTHROPIC_TARGET };

impl Entries {
    /// The entry of the key `which`.
    pub fn of(&self, which: KeyFor) -> &'static str {
        match which {
            KeyFor::Nebius => self.nebius,
            KeyFor::Anthropic => self.anthropic,
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn not_found(e: &io::Error) -> bool {
    e.raw_os_error() == Some(ERROR_NOT_FOUND as i32)
}

/// The key as typed, without the spaces and line breaks a paste brings around it, or why it is not a key. A refusal is
/// in Steplet's words and never repeats what was typed.
pub fn check(key: &str) -> Result<&str, String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Steplet needs a key: the box is empty.".into());
    }
    if key.len() > MAX_KEY_BYTES {
        return Err("Steplet did not save this: it is far longer than an API key.".into());
    }
    if key.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(
            "Steplet did not save this: a key has no spaces, line breaks or hidden characters inside it.".into()
        );
    }
    Ok(key)
}

/// Saves `key` as the entry `target`, once `check` takes it.
pub fn save(target: &str, key: &str) -> Result<(), String> {
    let key = check(key)?;
    let mut name = wide(target);
    let mut blob = key.as_bytes().to_vec();
    let entry = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: name.as_mut_ptr(),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        ..Default::default()
    };
    // SAFETY: `entry` points at `name` and `blob`, which outlive the call; Windows copies what it keeps.
    let saved = unsafe { CredWriteW(&entry, 0) } != 0;
    let why = io::Error::last_os_error();
    blob.fill(0);
    if saved {
        Ok(())
    } else {
        Err(format!(
            "Steplet could not save the key in Windows' Credential Manager. {}",
            crate::views::windows_says(why)
        ))
    }
}

/// The key saved as the entry `target`, if there is one.
pub fn load(target: &str) -> io::Result<Option<ApiKey>> {
    let name = wide(target);
    let mut entry: *mut CREDENTIALW = ptr::null_mut();
    // SAFETY: `name` is a whole wide string; on success Windows hands back one allocation, freed below.
    if unsafe { CredReadW(name.as_ptr(), CRED_TYPE_GENERIC, 0, &mut entry) } == 0 {
        let e = io::Error::last_os_error();
        return if not_found(&e) { Ok(None) } else { Err(e) };
    }
    // SAFETY: `entry` is Windows' answer, valid until `CredFree`; its blob is `CredentialBlobSize` bytes long.
    let text = unsafe {
        let e = &*entry;
        let bytes = if e.CredentialBlob.is_null() {
            Vec::new()
        } else {
            std::slice::from_raw_parts(e.CredentialBlob, e.CredentialBlobSize as usize).to_vec()
        };
        CredFree(entry as *const _);
        String::from_utf8(bytes)
    };
    match text {
        Ok(t) if t.is_empty() => Ok(None),
        Ok(t) => Ok(Some(ApiKey::new(t))),
        Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "the saved entry is not text")),
    }
}

/// Removes the entry `target`. Nothing to remove is not a failure.
pub fn forget(target: &str) -> io::Result<()> {
    let name = wide(target);
    // SAFETY: `name` is a whole wide string.
    if unsafe { CredDeleteW(name.as_ptr(), CRED_TYPE_GENERIC, 0) } == 0 {
        let e = io::Error::last_os_error();
        if !not_found(&e) {
            return Err(e);
        }
    }
    Ok(())
}
