//! "Open .steplet files with Steplet" (J4 A): a switch in Setup, off until the receiver turns it on. Its entries are
//! for this Windows user only (`HKEY_CURRENT_USER\Software\Classes`, no administrator rights): the file type names
//! Steplet's entry, whose open command starts this program with the file, and lists it among the entries it offers
//! (`OpenWithProgids`); the program gets an entry of its own under `Applications`, offered for that type only, since
//! Explorer's "Select an app" box lists programs from those entries (Task 15). If another program's entry had the
//! file type, Steplet notes which and gives the file type back to it when the switch goes off; otherwise off removes
//! the file type's own value, and its key once nothing else is in it. Nothing another program put there is removed.

use std::io;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, RRF_RT_ANY, RRF_RT_REG_SZ,
    RegCloseKey, RegCreateKeyExW, RegDeleteKeyValueW, RegDeleteKeyW, RegDeleteTreeW, RegGetValueW, RegOpenKeyExW,
    RegQueryInfoKeyW, RegSetValueExW,
};
use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};

use crate::views::Switch;

/// The file type, and Steplet's entry for it.
pub const EXT: &str = ".steplet";
pub const PROG_ID: &str = "Steplet.File";
const CLASSES: &str = r"Software\Classes";
/// The value, in Steplet's own entry, that names the entry that had the file type before Steplet took it.
const HAD_IT: &str = "Steplet.HadTheFileType";
/// The file type's list of the entries it offers, one value per entry, named after it (Microsoft's "Open With" list).
const OPEN_WITH: &str = "OpenWithProgids";
/// Where each program has its own entry, named after its file (Microsoft's application registration).
const APPLICATIONS: &str = r"Software\Classes\Applications";
/// The value, in Steplet's own entry, that names the program entry Steplet made under `APPLICATIONS`.
const MADE_APP: &str = "Steplet.Application";

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

/// The name of the program entry for `exe`: its file's name. A path with no file name has no entry.
fn app_name(exe: &Path) -> io::Result<String> {
    exe.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the program's path names no file"))
}

/// Turns the switch on: files of type `ext` open with the program `exe`. The program entry under `APPLICATIONS` is
/// Steplet's only when it is not there yet or already opens this program; another program's entry of the same name
/// is left as it is.
pub fn turn_on(ext: &str, prog_id: &str, exe: &Path) -> io::Result<()> {
    // What can fail without writing is worked out first, so that such a failure changes nothing.
    let app = app_name(exe)?;
    let (ext_key, prog_key) = (format!(r"{CLASSES}\{ext}"), format!(r"{CLASSES}\{prog_id}"));
    let (app_key, command) = (format!(r"{APPLICATIONS}\{app}"), command_for(exe));
    let before = default_value(&ext_key);
    set_default(&prog_key, "Steplet task file")?;
    if let Some(other) = before.filter(|b| b != prog_id && !b.is_empty()) {
        set_value(&prog_key, Some(HAD_IT), &other)?;
    }
    set_default(&format!(r"{prog_key}\shell\open\command"), &command)?;
    set_default(&ext_key, prog_id)?;
    set_value(&format!(r"{ext_key}\{OPEN_WITH}"), Some(prog_id), "")?;
    let app_command = format!(r"{app_key}\shell\open\command");
    if !exists(&app_key) || default_value(&app_command).as_deref() == Some(command.as_str()) {
        // Named first, so that turning off finds the entry even if a write below fails.
        set_value(&prog_key, Some(MADE_APP), &app)?;
        set_default(&app_command, &command)?;
        set_default(&format!(r"{app_key}\DefaultIcon"), &format!("\"{}\",0", exe.display()))?;
        set_value(&format!(r"{app_key}\SupportedTypes"), Some(ext), "")?;
    }
    told_windows();
    Ok(())
}

/// Turns the switch off: the file type goes back to the entry that had it, or loses Steplet's value; then Steplet's
/// own entry is removed, with the program entry it made, as long as that still opens what Steplet's entry opens.
pub fn turn_off(ext: &str, prog_id: &str) -> io::Result<()> {
    let (ext_key, prog_key) = (format!(r"{CLASSES}\{ext}"), format!(r"{CLASSES}\{prog_id}"));
    let open_with = format!(r"{ext_key}\{OPEN_WITH}");
    delete_value(&open_with, Some(prog_id))?;
    if is_empty(&open_with) {
        delete_tree(&open_with)?;
    }
    if default_value(&ext_key).as_deref() == Some(prog_id) {
        match value(&prog_key, Some(HAD_IT)) {
            Some(other) => set_default(&ext_key, &other)?,
            None => {
                delete_default(&ext_key)?;
                if is_empty(&ext_key) {
                    delete_tree(&ext_key)?;
                }
            }
        }
    }
    // Only the program entry Steplet's own entry names, never the whole list, and only the parts Steplet writes.
    let ours = default_value(&format!(r"{prog_key}\shell\open\command"));
    if let Some(app) = value(&prog_key, Some(MADE_APP)).filter(|a| !a.is_empty() && !a.contains('\\')) {
        let app_key = format!(r"{APPLICATIONS}\{app}");
        if ours.is_some() && default_value(&format!(r"{app_key}\shell\open\command")) == ours {
            let types = format!(r"{app_key}\SupportedTypes");
            delete_value(&types, Some(ext))?;
            delete_tree(&format!(r"{app_key}\shell\open\command"))?;
            delete_tree(&format!(r"{app_key}\DefaultIcon"))?;
            for part in [types, format!(r"{app_key}\shell\open"), format!(r"{app_key}\shell"), app_key] {
                if is_empty(&part) {
                    delete_tree(&part)?;
                }
            }
        }
    }
    delete_tree(&prog_key)?;
    told_windows();
    Ok(())
}

/// How the switch stands for the program `exe`: on; off; or broken, when it points at a program that is not this one
/// (it moved, or the entry was changed) or is not offered in Explorer's "Select an app" box (an older Steplet's).
pub fn state(ext: &str, prog_id: &str, exe: &Path) -> Switch {
    if default_value(&format!(r"{CLASSES}\{ext}")).as_deref() != Some(prog_id) {
        return Switch::Off;
    }
    let command = command_for(exe);
    let offered = has_value(&format!(r"{CLASSES}\{ext}\{OPEN_WITH}"), prog_id)
        && app_name(exe).is_ok_and(|app| {
            let app_key = format!(r"{APPLICATIONS}\{app}");
            match default_value(&format!(r"{app_key}\shell\open\command")) {
                // Steplet's own program entry offers this type.
                Some(c) if c == command => has_value(&format!(r"{app_key}\SupportedTypes"), ext),
                // Another program's entry of that name is left alone; none at all is missing.
                _ => exists(&app_key),
            }
        });
    match default_value(&format!(r"{CLASSES}\{prog_id}\shell\open\command")) {
        Some(c) if c == command && exe.is_file() && offered => Switch::On,
        _ => Switch::Broken,
    }
}

/// The default value of the key `subkey` of `HKEY_CURRENT_USER`, if it is there and is text.
pub fn default_value(subkey: &str) -> Option<String> {
    value(subkey, None)
}

/// The value `name` (`None`: the default value) of the key `subkey` of `HKEY_CURRENT_USER`, if it is there and is
/// text.
fn value(subkey: &str, name: Option<&str>) -> Option<String> {
    let key = wide(subkey);
    let name = name.map(wide);
    let name_ptr = name.as_ref().map_or(ptr::null(), |n| n.as_ptr());
    let mut size: u32 = 0;
    // SAFETY: a size query: no buffer, Windows writes the size it needs in bytes.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name_ptr,
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
            key.as_ptr(),
            name_ptr,
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

/// Whether the key `subkey` of `HKEY_CURRENT_USER` holds the value `name`, of any type: Microsoft documents the
/// values of the "Open With" lists as REG_NONE, and Windows may write them so.
pub fn has_value(subkey: &str, name: &str) -> bool {
    let (key, name) = (wide(subkey), wide(name));
    let mut size: u32 = 0;
    // SAFETY: a size query of any type: no buffer, Windows writes the size it needs in bytes.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_ANY,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut size,
        )
    };
    r == 0
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

/// Sets the default value of the key `subkey` of `HKEY_CURRENT_USER`, making the key if it is not there. Only this
/// user's own part of the registry; the switch's tests use it to stand for another program's entry.
pub fn set_default(subkey: &str, value: &str) -> io::Result<()> {
    set_value(subkey, None, value)
}

fn set_value(subkey: &str, name: Option<&str>, value: &str) -> io::Result<()> {
    let key_name = wide(subkey);
    let name = name.map(wide);
    let mut key: HKEY = ptr::null_mut();
    // SAFETY: on success `key` is an open key, closed below.
    checked(unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            key_name.as_ptr(),
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
    let name_ptr = name.as_ref().map_or(ptr::null(), |n| n.as_ptr());
    // SAFETY: `data` is a whole wide string of `data.len() * 2` bytes; `key` is open.
    let r = unsafe { RegSetValueExW(key, name_ptr, 0, REG_SZ, data.as_ptr().cast(), (data.len() * 2) as u32) };
    // SAFETY: `key` was opened above.
    unsafe { RegCloseKey(key) };
    checked(r)
}

/// Removes the default value of the key `subkey`. Nothing there is not a failure.
fn delete_default(subkey: &str) -> io::Result<()> {
    delete_value(subkey, None)
}

/// Removes the value `name` (`None`: the default value) of the key `subkey`. Nothing there is not a failure.
fn delete_value(subkey: &str, name: Option<&str>) -> io::Result<()> {
    let key = wide(subkey);
    let name = name.map(wide);
    let name_ptr = name.as_ref().map_or(ptr::null(), |n| n.as_ptr());
    // SAFETY: `key` and `name` are whole wide strings; a null value name is the default value.
    let r = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, key.as_ptr(), name_ptr) };
    if r == ERROR_FILE_NOT_FOUND { Ok(()) } else { checked(r) }
}

/// Whether the key `subkey` holds no value and no key of its own (a key Windows cannot read counts as not empty).
fn is_empty(subkey: &str) -> bool {
    let name = wide(subkey);
    let mut key: HKEY = ptr::null_mut();
    // SAFETY: on success `key` is an open key, closed below.
    if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, name.as_ptr(), 0, KEY_READ, &mut key) } != 0 {
        return false;
    }
    let (mut keys, mut values) = (0u32, 0u32);
    // SAFETY: `key` is open; only the two counts are asked for.
    let r = unsafe {
        RegQueryInfoKeyW(
            key,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null(),
            &mut keys,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut values,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };
    // SAFETY: `key` was opened above.
    unsafe { RegCloseKey(key) };
    r == 0 && keys == 0 && values == 0
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
