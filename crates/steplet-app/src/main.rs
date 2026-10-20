//! Steplet's desktop app. Everything is in the library, so its tests can reach it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    steplet_app::run();
}
