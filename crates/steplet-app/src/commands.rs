// The app's own commands, the only ones the page may call. build.rs reads this file too and hands the list to
// tauri-build, which writes the app's permission list from it; Tauri then checks every call of a listed command
// against capabilities/main.json, where each needs its `allow-<name>` line (with `-` for `_`). Tauri checks app
// commands only once this list holds one, so every command the app registers must be listed here:
// tests/shell.rs fails otherwise.
pub const COMMANDS: &[&str] = &[
    "initial_preview",
    "open_bytes",
    "start_note",
    "chooser",
    "setup_status",
    "setup_install",
    "setup_remove",
    "save_key",
    "forget_key",
    "set_opens_steplet_files",
    "open_help",
];
