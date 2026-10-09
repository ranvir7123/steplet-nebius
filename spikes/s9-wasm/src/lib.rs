//! Spike: s9-core for the browser (milestone 1, phase 0). Throwaway.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn verify(msg: &[u8], sig_pem: &str, namespace: &str) -> String {
    match s9_core::verify(msg, sig_pem, namespace) {
        Ok(fingerprint) => format!("valid ({fingerprint})"),
        Err(e) => format!("invalid: {e}"),
    }
}
