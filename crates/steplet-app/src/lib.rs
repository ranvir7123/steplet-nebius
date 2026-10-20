//! Steplet's desktop app: the Tauri shell around `steplet-core` and `steplet-run`.
//!
//! The window is made here from its entry in tauri.conf.json, so that it can only ever show the app's own page:
//! every navigation elsewhere and every new window is refused, and its title is never changed (it stays
//! "Steplet", whatever a file says).

use tauri::webview::NewWindowResponse;
use tauri::{Url, WebviewWindowBuilder};

include!("commands.rs");

pub mod help;
pub mod key;
pub mod open;
pub mod pins;
pub mod register;
pub mod session;
pub mod setup;
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
    /// What the last look at this computer's setup saw, so saving the key or turning the switch does not look again.
    looked: std::sync::Mutex<Option<steplet_run::setup::State>>,
    /// An install or a removal of Steplet's Linux is going on.
    busy: setup::OneAtATime,
    /// The run, if one goes on in this window.
    session: session::Session,
    /// The Open screen's note after a crash's run was cleared: `None` until the look at start is done.
    note: std::sync::Arc<(std::sync::Mutex<Option<Option<String>>>, std::sync::Condvar)>,
}

impl AppState {
    fn pins(&self) -> pins::Pins {
        pins::load_at(self.pins.as_deref())
    }

    fn show(&self, mut raw: Vec<u8>) -> views::PreviewView {
        // The bytes kept for Run are exactly the bytes previewed, even if more were sent.
        raw.truncate(open::MAX_READ as usize);
        let view = open::preview_file(&raw, open::now_unix(), &self.pins());
        *self.opened.lock().unwrap_or_else(|e| e.into_inner()) = Some(raw);
        view
    }

    /// Looks at this computer's setup again and keeps what it saw.
    fn look(&self) -> steplet_run::setup::State {
        let state = setup::look();
        *self.looked.lock().unwrap_or_else(|e| e.into_inner()) = Some(state.clone());
        state
    }

    /// The Setup screen, from the last look (or a new one if there was none).
    fn setup_view(&self) -> views::SetupView {
        let last = self.looked.lock().unwrap_or_else(|e| e.into_inner()).clone();
        setup::view(&last.unwrap_or_else(|| self.look()), key::TARGET, setup::switch())
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

/// A note for the Open screen, in Steplet's words: a run left by a crash was cleared. It waits for the look at start
/// (at most the time Steplet takes to remove that run).
#[tauri::command(async)]
fn start_note(state: tauri::State<AppState>) -> Option<String> {
    let (note, ready) = &*state.note;
    let mut note = note.lock().unwrap_or_else(|e| e.into_inner());
    while note.is_none() {
        note = ready.wait(note).unwrap_or_else(|e| e.into_inner());
    }
    note.clone().flatten()
}

/// The window, as a run needs it: progress as `run-progress` events, the taskbar's flash, and closing.
struct WindowScreen(tauri::WebviewWindow);

impl session::Screen for WindowScreen {
    fn send(&self, p: views::ProgressView) {
        use tauri::Emitter;
        let _ = self.0.emit("run-progress", p);
    }
    fn attention(&self) {
        let _ = self.0.request_user_attention(Some(tauri::UserAttentionType::Critical));
    }
    fn close(&self) {
        let _ = self.0.close();
    }
}

/// Starts the run of the file on screen, previewed with the SHA-256 `sha256`; `typed` is a red file's sentence. Rust
/// checks the file again now (row 30). A refusal is Steplet's own sentence.
#[tauri::command(async)]
fn start_run(
    state: tauri::State<AppState>,
    window: tauri::WebviewWindow,
    sha256: String,
    typed: Option<String>,
) -> Result<(), String> {
    let raw = state.opened.lock().unwrap_or_else(|e| e.into_inner()).clone().ok_or("Steplet has no file open.")?;
    let ready = setup::ready_for_run(&state.look());
    state.session.start(&raw, &sha256, typed.as_deref(), ready, std::sync::Arc::new(WindowScreen(window)))
}

/// The receiver's answer to question `question`, bound to the request's fingerprint (R7).
#[tauri::command]
fn answer(state: tauri::State<AppState>, question: u32, yes: bool, fingerprint: String) -> bool {
    state.session.answer(question, yes, &fingerprint)
}

#[tauri::command]
fn stop_run(state: tauri::State<AppState>) {
    state.session.stop();
}

/// The receiver closes the window while a file runs: Stop, then close once the run is over.
#[tauri::command]
fn stop_and_close(state: tauri::State<AppState>) {
    state.session.stop_and_close();
}

/// The run chooser for the file on screen: the runner's own lines for it (R6), and whether a run can start now.
/// Commands that may wait on WSL run off the window's own thread (`async`), so the window never freezes.
#[tauri::command(async)]
fn chooser(state: tauri::State<AppState>) -> Result<views::ChooserView, String> {
    let raw = state.opened.lock().unwrap_or_else(|e| e.into_inner()).clone().ok_or("Steplet has no file open.")?;
    let file = steplet_core::report::check(&raw, open::now_unix())
        .steplet
        .ok_or("Steplet could not read this file, so it cannot run.")?;
    let ready = setup::view(&state.look(), key::TARGET, setup::switch()).ready;
    Ok(views::chooser(&steplet_run::promises(&file), ready))
}

/// The Setup screen, from a new look at this computer ("Check again" too).
#[tauri::command(async)]
fn setup_status(state: tauri::State<AppState>) -> views::SetupView {
    setup::view(&state.look(), key::TARGET, setup::switch())
}

/// Installs Steplet's Linux, or finishes an install that stopped; each line it prints is a `setup-line` event.
#[tauri::command(async)]
fn setup_install(app: tauri::AppHandle, state: tauri::State<AppState>) -> Result<(), views::FailureView> {
    use tauri::Emitter;
    state.busy.run(|| {
        setup::install(&|line| {
            let _ = app.emit("setup-line", line);
        })
    })
}

/// "Start over": removes Steplet's Linux and everything in it (nothing of the receiver's is in it).
#[tauri::command(async)]
fn setup_remove(state: tauri::State<AppState>) -> Result<views::SetupView, views::FailureView> {
    state.busy.run(setup::remove)?;
    Ok(setup::view(&state.look(), key::TARGET, setup::switch()))
}

/// Saves the receiver's key in Credential Manager. The answer says only that a key is saved.
#[tauri::command(async)]
fn save_key(state: tauri::State<AppState>, key: String) -> Result<views::SetupView, String> {
    key::save(key::TARGET, &key)?;
    Ok(state.setup_view())
}

#[tauri::command(async)]
fn forget_key(state: tauri::State<AppState>) -> Result<views::SetupView, String> {
    key::forget(key::TARGET)
        .map_err(|e| format!("Steplet could not remove the key from Windows' Credential Manager. Windows says: {e}"))?;
    Ok(state.setup_view())
}

/// Turns "Open .steplet files with Steplet" on or off, for this Windows user only (J4 A).
#[tauri::command(async)]
fn set_opens_steplet_files(state: tauri::State<AppState>, on: bool) -> Result<views::SetupView, String> {
    let exe = setup::this_program().ok_or("Windows did not say where Steplet's own program is.")?;
    let changed = if on {
        register::turn_on(register::EXT, register::PROG_ID, &exe)
    } else {
        register::turn_off(register::EXT, register::PROG_ID)
    };
    changed.map_err(|e| format!("Steplet could not change what opens .steplet files. Windows says: {e}"))?;
    Ok(state.setup_view())
}

/// Opens one of Steplet's own help pages in the browser, from a fixed list (S11).
#[tauri::command]
fn open_help(topic: String) -> Result<(), String> {
    help::open(&topic)
}

/// Starts the app and returns when its window closes.
pub fn run() {
    let places = session::Places::here().unwrap_or_else(|| {
        // Windows did not say where this user's app data goes: Steplet keeps its run notes with its program.
        let here = std::env::current_exe().ok().and_then(|e| e.parent().map(|d| d.join("Steplet"))).unwrap_or_default();
        session::Places { marker: here.join("run-in-progress.json"), pins: None, logs: here.join("logs") }
    });
    let note: std::sync::Arc<(std::sync::Mutex<Option<Option<String>>>, std::sync::Condvar)> = Default::default();
    let state = AppState {
        pins: pins::location(),
        // Windows starts Steplet with the file to open (a double-click); anything else is not a file.
        initial: std::sync::Mutex::new(
            std::env::args_os().skip(1).find(|a| !a.to_string_lossy().starts_with('-')).map(std::path::PathBuf::from),
        ),
        opened: std::sync::Mutex::new(None),
        looked: std::sync::Mutex::new(None),
        busy: setup::OneAtATime::default(),
        session: session::Session::new(
            std::sync::Arc::new(steplet_run::run_live),
            std::sync::Arc::new(open::now_unix),
            places.clone(),
        ),
        note: note.clone(),
    };
    // A run left by a crash is stopped and removed now, off the window's thread; the Open screen then says so.
    std::thread::spawn(move || {
        // Even if that look fails badly, the Open screen's note stops waiting.
        let said = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            session::Session::clear_leftover(&places, &|paths| steplet_run::discard(paths))
        }))
        .unwrap_or(None);
        let (slot, ready) = &*note;
        *slot.lock().unwrap_or_else(|e| e.into_inner()) = Some(said);
        ready.notify_all();
    });
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            initial_preview,
            open_bytes,
            start_note,
            chooser,
            setup_status,
            setup_install,
            setup_remove,
            save_key,
            forget_key,
            set_opens_steplet_files,
            open_help,
            start_run,
            answer,
            stop_run,
            stop_and_close
        ])
        .on_window_event(|window, event| {
            // Closing while a file runs asks first: the page shows the question (`close-asked`).
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                use tauri::{Emitter, Manager};
                if !window.state::<AppState>().session.may_close() {
                    api.prevent_close();
                    let _ = window.emit("close-asked", ());
                }
            }
        })
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
