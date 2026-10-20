include!("src/commands.rs");

fn main() {
    // Listing the app's commands makes Tauri check each call against capabilities/main.json.
    let app = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(app)).expect("tauri-build failed");
}
