//! The attack suite (phase 5, R20): a test-only crate, never shipped. It lists every known attack on a Steplet run
//! (the catalogue), replays each one by running exactly the tests that already exist for it, shows each one blocked or
//! not, and keeps the whole output as a record in `docs/suite/`. `cargo suite list | scripted | live`.

pub mod catalogue;
pub mod live;
pub mod record;
pub mod runner;
pub mod verdict;

/// The runner's helpers shared with steplet-run's own tests: the look after a case, the made-up key.
#[path = "../../steplet-run/tests/support/mod.rs"]
mod support;
