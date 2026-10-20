//! Steplet's own help pages (smaller choice S11): a fixed list, opened in the receiver's browser by Windows. Links in
//! a file's text are never opened: they are shown with their real address and are not clickable.

use std::ptr;

use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// The address of the help page `topic`, if Steplet has one by that name.
pub fn address(topic: &str) -> Option<&'static str> {
    match topic {
        "wsl" => Some("https://learn.microsoft.com/windows/wsl/install"),
        "anthropic-key" => Some("https://console.anthropic.com/settings/keys"),
        _ => None,
    }
}

/// Opens the help page `topic` in the receiver's browser.
pub fn open(topic: &str) -> Result<(), String> {
    let Some(url) = address(topic) else { return Err("Steplet has no help page by that name.".into()) };
    let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    let file: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();
    // SAFETY: both strings are whole wide strings; Windows asks the browser to open the address.
    let r = unsafe {
        ShellExecuteW(ptr::null_mut(), verb.as_ptr(), file.as_ptr(), ptr::null(), ptr::null(), SW_SHOWNORMAL)
    };
    // Windows' own rule: a value above 32 means it worked.
    if r as usize > 32 { Ok(()) } else { Err("Windows could not open your browser.".into()) }
}
