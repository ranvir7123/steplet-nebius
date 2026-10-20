// The app's own commands, the only ones the page may call. Each needs its `allow-<name>` line (with `-` for
// `_`) in capabilities/main.json. build.rs reads this file too, so Tauri refuses a command not listed here.
pub const COMMANDS: &[&str] = &[];
