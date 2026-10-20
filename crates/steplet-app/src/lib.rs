//! Steplet's desktop app: the Tauri shell around `steplet-core` and `steplet-run`.
//!
//! The window is made here from its entry in tauri.conf.json, so that it can only ever show the app's own page:
//! every navigation elsewhere and every new window is refused, and its title is never changed (it stays
//! "Steplet", whatever a file says).

use tauri::webview::NewWindowResponse;
use tauri::{Url, WebviewWindowBuilder};

include!("commands.rs");

pub mod open;
pub mod pins;
pub mod views;

/// WebView2's start arguments for every build.
/// - wry's own defaults, which any arguments of ours replace, so they are repeated: they turn off a few Edge
///   features, SmartScreen among them, which would send addresses to Microsoft.
/// - Every request the web view would send over the network goes to a closed port on this computer, local
///   addresses included (`<-loopback>`), so nothing leaves (row 28). The page rules already refuse requests, but
///   a navigation's request is sent before the app's navigation check can cancel it (measured in Task 8). The
///   app's own page and its channel to Rust never use the network, so they are not affected.
macro_rules! base_args {
    () => {
        concat!(
            "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
            " --proxy-server=http://127.0.0.1:47 --proxy-bypass-list=<-loopback>"
        )
    };
}

/// WebView2's extra start arguments. Only a test build (feature `e2e`) opens the debugging port, which Chromium
/// opens on 127.0.0.1 only (smaller choice S13).
pub fn browser_args() -> &'static str {
    if cfg!(feature = "e2e") { concat!(base_args!(), " --remote-debugging-port=9333") } else { base_args!() }
}

/// Whether `url` is the app's own page: WebView2 serves it at `http://tauri.localhost`, with no port and no user
/// name or password in the address.
pub fn is_own_page(url: &Url) -> bool {
    url.scheme() == "http"
        && url.host_str() == Some("tauri.localhost")
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
}

/// What the app keeps while its window is open.
pub struct AppState {
    /// Where the pins live (`None` when Windows does not say where this user's app data goes).
    pins: Option<std::path::PathBuf>,
    /// The file Steplet was started with (a double-click), until the page asks for it.
    initial: std::sync::Mutex<Option<std::path::PathBuf>>,
    /// The file on screen: its bytes as read, so Run can check they are the ones previewed (row 30, Task 12).
    opened: std::sync::Mutex<Option<Vec<u8>>>,
}

impl AppState {
    fn pins(&self) -> pins::Pins {
        match &self.pins {
            Some(p) => pins::load(p),
            None => pins::Pins::Unreadable("Windows did not say where this user's app data goes".into()),
        }
    }

    fn show(&self, mut raw: Vec<u8>) -> views::PreviewView {
        // The bytes kept for Run are exactly the bytes previewed, even if more were sent.
        raw.truncate(open::MAX_READ as usize);
        let view = open::preview_file(&raw, open::now_unix(), &self.pins());
        *self.opened.lock().unwrap_or_else(|e| e.into_inner()) = Some(raw);
        view
    }
}

/// The file Steplet was started with, if any. A file that cannot be read gives Steplet's words, never its name.
#[tauri::command]
fn initial_preview(state: tauri::State<AppState>) -> Result<Option<views::PreviewView>, String> {
    let path = state.initial.lock().unwrap_or_else(|e| e.into_inner()).take();
    match path {
        None => Ok(None),
        Some(p) => Ok(Some(state.show(open::read_file(&p)?))),
    }
}

/// A file dropped on the window or chosen with "Open a file…": the page sends at most its first `open::MAX_READ`
/// bytes (a browser page cannot learn a file's path, so the page never asks Rust to read a path).
#[tauri::command]
fn open_bytes(state: tauri::State<AppState>, bytes: Vec<u8>) -> views::PreviewView {
    state.show(bytes)
}

/// A note for the Open screen, in Steplet's words: a run left by a crash was cleared (Task 12 fills it in).
#[tauri::command]
fn start_note() -> Option<String> {
    None
}

/// Starts the app and returns when its window closes.
pub fn run() {
    let state = AppState {
        pins: pins::location(),
        initial: std::sync::Mutex::new(std::env::args_os().nth(1).map(std::path::PathBuf::from)),
        opened: std::sync::Mutex::new(None),
    };
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![initial_preview, open_bytes, start_note])
        .setup(|app| {
            let config = app
                .config()
                .app
                .windows
                .iter()
                .find(|w| w.label == "main")
                .cloned()
                .ok_or("tauri.conf.json has no window labelled main")?;
            WebviewWindowBuilder::from_config(app.handle(), &config)?
                .additional_browser_args(browser_args())
                .devtools(cfg!(feature = "e2e"))
                .on_navigation(is_own_page)
                .on_new_window(|_, _| NewWindowResponse::Deny)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Steplet's window could not start");
}
