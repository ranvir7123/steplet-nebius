include!("src/commands.rs");

fn main() {
    // Tauri checks each call of a listed command against capabilities/main.json (see src/commands.rs).
    let app = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(app)).expect("tauri-build failed");
}
