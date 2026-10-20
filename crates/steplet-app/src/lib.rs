//! Steplet's desktop app: the Tauri shell around `steplet-core` and `steplet-run`.
//!
//! The window is made here from its entry in tauri.conf.json, so that it can only ever show the app's own page:
//! every navigation elsewhere and every new window is refused, and its title is never changed (it stays
//! "Steplet", whatever a file says).

use tauri::webview::NewWindowResponse;
use tauri::{Url, WebviewWindowBuilder};

include!("commands.rs");

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

/// Starts the app and returns when its window closes.
pub fn run() {
    tauri::Builder::default()
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
