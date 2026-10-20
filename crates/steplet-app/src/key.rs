//! The receiver's Anthropic key (smaller choice S3): one "generic" entry in Windows' own Credential Manager, locked to
//! this Windows account and kept on this computer only, never in a file. Steplet's entry is `TARGET`; "Forget key"
//! removes it. The key never goes back to the screens: they learn only whether one is saved.

use std::io;
use std::ptr;

use steplet_run::ApiKey;
use windows_sys::Win32::Foundation::ERROR_NOT_FOUND;
use windows_sys::Win32::Security::Credentials::{
    CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree, CredReadW, CredWriteW,
};

/// Steplet's entry in Credential Manager.
pub const TARGET: &str = "Steplet/anthropic-api-key";
/// Far longer than any Anthropic key; anything longer is something else pasted by mistake.
const MAX_KEY_CHARS: usize = 512;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn not_found(e: &io::Error) -> bool {
    e.raw_os_error() == Some(ERROR_NOT_FOUND as i32)
}

/// Saves `key` as the entry `target`, without the spaces and line breaks a paste brings around it. A refusal is in
/// Steplet's words and never repeats what was typed.
pub fn save(target: &str, key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Steplet needs a key: the box is empty.".into());
    }
    if key.chars().count() > MAX_KEY_CHARS {
        return Err("Steplet did not save this: it is far longer than an Anthropic key.".into());
    }
    if key.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(
            "Steplet did not save this: a key has no spaces, line breaks or hidden characters inside it.".into()
        );
    }
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
        Err(format!("Steplet could not save the key in Windows' Credential Manager. Windows says: {why}"))
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
