//! Steplet's file core: the clean-up, the header reader, signature checks, size limits and display pieces.
//! It uses no operating system at all (`no_std`), so the desktop app and the browser check page run the same code.
#![no_std]
extern crate alloc;

pub mod canon;
pub mod date;
pub mod header;
pub mod limits;
pub mod sig;

/// The signature name space (decision P4).
pub const NAMESPACE: &str = "steplet-v1";

/// What a refused file may contain instead, and a corrected example. Shown by `steplet-sign check`
/// and, from phase 4, by the preview. `example` may be empty when no example helps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Help {
    pub allowed: &'static str,
    pub example: &'static str,
}
