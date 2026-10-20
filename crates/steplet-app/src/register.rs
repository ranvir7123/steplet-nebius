//! "Open .steplet files with Steplet" (J4 A): a switch in Setup, off until the receiver turns it on. Its entries are
//! for this Windows user only (`HKEY_CURRENT_USER\Software\Classes`, no administrator rights): the file type names
//! Steplet's entry, whose open command starts this program with the file. Turning it off removes them, and never
//! takes the file type from another program's entry.

use std::io;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, RRF_RT_REG_SZ, RegCloseKey,
    RegCreateKeyExW, RegDeleteKeyW, RegDeleteTreeW, RegGetValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};

use crate::views::Switch;

/// The file type, and Steplet's entry for it.
pub const EXT: &str = ".steplet";
pub const PROG_ID: &str = "Steplet.File";
const CLASSES: &str = r"Software\Classes";

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn checked(r: WIN32_ERROR) -> io::Result<()> {
    if r == 0 { Ok(()) } else { Err(io::Error::from_raw_os_error(r as i32)) }
}

/// The command Windows runs to open a file with the program `exe`.
fn command_for(exe: &Path) -> String {
    format!("\"{}\" \"%1\"", exe.display())
}

/// Turns the switch on: files of type `ext` open with the program `exe`.
pub fn turn_on(ext: &str, prog_id: &str, exe: &Path) -> io::Result<()> {
    set_default(&format!(r"{CLASSES}\{prog_id}"), "Steplet task file")?;
    set_default(&format!(r"{CLASSES}\{prog_id}\shell\open\command"), &command_for(exe))?;
    set_default(&format!(r"{CLASSES}\{ext}"), prog_id)?;
    told_windows();
    Ok(())
}

/// Turns the switch off: removes Steplet's entry, and the file type's own entry if it names Steplet's.
pub fn turn_off(ext: &str, prog_id: &str) -> io::Result<()> {
    let ext_key = format!(r"{CLASSES}\{ext}");
    if default_value(&ext_key).as_deref() == Some(prog_id) {
        delete_tree(&ext_key)?;
    }
    delete_tree(&format!(r"{CLASSES}\{prog_id}"))?;
    told_windows();
    Ok(())
}

/// How the switch stands for the program `exe`: on, off, or on but pointing at a program that is not this one (it
/// moved, or the entry was changed).
pub fn state(ext: &str, prog_id: &str, exe: &Path) -> Switch {
    if default_value(&format!(r"{CLASSES}\{ext}")).as_deref() != Some(prog_id) {
        return Switch::Off;
    }
    match default_value(&format!(r"{CLASSES}\{prog_id}\shell\open\command")) {
        Some(command) if command == command_for(exe) && exe.is_file() => Switch::On,
        _ => Switch::Broken,
    }
}

/// The default value of the key `subkey` of `HKEY_CURRENT_USER`, if it is there and is text.
pub fn default_value(subkey: &str) -> Option<String> {
    let name = wide(subkey);
    let mut size: u32 = 0;
    // SAFETY: a size query: no buffer, Windows writes the size it needs in bytes.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            name.as_ptr(),
            ptr::null(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut size,
        )
    };
    if r != 0 || size == 0 {
        return None;
    }
    let mut buf = vec![0u16; (size as usize).div_ceil(2)];
    // SAFETY: `buf` holds `size` bytes.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            name.as_ptr(),
            ptr::null(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if r != 0 {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16(&buf[..len]).ok()
}

/// Whether the key `subkey` of `HKEY_CURRENT_USER` is there.
pub fn exists(subkey: &str) -> bool {
    let name = wide(subkey);
    let mut key: HKEY = ptr::null_mut();
    // SAFETY: on success `key` is an open key, closed at once.
    let r = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, name.as_ptr(), 0, KEY_READ, &mut key) };
    if r == 0 {
        // SAFETY: `key` was opened above.
        unsafe { RegCloseKey(key) };
    }
    r == 0
}

fn set_default(subkey: &str, value: &str) -> io::Result<()> {
    let name = wide(subkey);
    let mut key: HKEY = ptr::null_mut();
    // SAFETY: on success `key` is an open key, closed below.
    checked(unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            name.as_ptr(),
            0,
            ptr::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            ptr::null(),
            &mut key,
            ptr::null_mut(),
        )
    })?;
    let data = wide(value);
    // SAFETY: `data` is a whole wide string of `data.len() * 2` bytes; `key` is open.
    let r = unsafe { RegSetValueExW(key, ptr::null(), 0, REG_SZ, data.as_ptr().cast(), (data.len() * 2) as u32) };
    // SAFETY: `key` was opened above.
    unsafe { RegCloseKey(key) };
    checked(r)
}

/// Removes the key `subkey` of `HKEY_CURRENT_USER` and everything under it. Nothing there is not a failure.
fn delete_tree(subkey: &str) -> io::Result<()> {
    let name = wide(subkey);
    let gone = |r: WIN32_ERROR| if r == ERROR_FILE_NOT_FOUND { Ok(()) } else { checked(r) };
    // SAFETY: `name` is a whole wide string.
    gone(unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, name.as_ptr()) })?;
    // SAFETY: as above; the tree's own key, if the call above left it.
    gone(unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, name.as_ptr()) })
}

/// Tells Windows that what opens which file type changed, so Explorer shows it at once.
fn told_windows() {
    // SAFETY: this event takes no items.
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED as i32, SHCNF_IDLIST, ptr::null(), ptr::null()) };
}
