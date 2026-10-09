//! Spike: does the Svelte page run inside WebView2 (milestone 1, phase 0)? Throwaway.
//! The page reports what it rendered through one command; the shell writes it to a file and exits.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn report(app: tauri::AppHandle, text: String) {
    let path = std::env::var("S9_REPORT").unwrap_or_else(|_| r"C:\dev\m1\logs\S9-webview.txt".into());
    std::fs::write(&path, text).expect("write the report");
    app.exit(0);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![report])
        .run(tauri::generate_context!())
        .expect("the S9 shell failed to start");
}
