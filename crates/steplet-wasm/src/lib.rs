//! The file core for web pages (the public check page, phase 6): one function, the same answer as the app.

use wasm_bindgen::prelude::*;

/// Reads a `.steplet` file and returns its report as JSON (`steplet_core::report::Report::to_json`). `now_unix` is
/// the page's current time in seconds since 1970-01-01 UTC; JavaScript passes it as a BigInt.
#[wasm_bindgen]
pub fn check(bytes: &[u8], now_unix: i64) -> String {
    steplet_core::report::check(bytes, now_unix).to_json()
}
