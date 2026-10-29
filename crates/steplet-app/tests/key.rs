//! The receiver's keys (Task 11, smaller choice S3; the Nebius key since the Nebius copy's Task 3): each kept in
//! Windows' own Credential Manager under an entry of its own, locked to this Windows account, and never in the data
//! the screens get. These tests use made-up entries of their own, never Steplet's real ones, and remove them when they
//! end, passed or failed.

use steplet_app::key::{self, Entries};
use steplet_app::setup;
use steplet_app::views::{KeyFor, Switch};
use steplet_run::setup::{State, Wsl};

/// Made-up text in a key's shape; never a real key.
const MADE_UP_KEY: &str = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";
const MADE_UP_NEBIUS_KEY: &str = "nebius-STEPLET-TEST-MADE-UP-NOT-A-KEY";

/// Removes a test's entries when the test ends, passed or failed.
struct Forget(&'static [&'static str]);

impl Drop for Forget {
    fn drop(&mut self) {
        for target in self.0 {
            let _ = key::forget(target);
        }
    }
}

fn all_ready() -> State {
    State {
        wsl: Wsl::Ready { version: "2.6.3.0".into() },
        linux: Some("{made-up-id}".into()),
        missing: Vec::new(),
        gate: true,
        trouble: None,
    }
}

#[test]
fn key_is_stored_read_and_forgotten() {
    const TARGET: &str = "Steplet-test/made-up-key";
    let _cleanup = Forget(&[TARGET]);
    key::forget(TARGET).unwrap();
    assert!(key::load(TARGET).unwrap().is_none(), "no entry yet");
    // A key pasted with a line break or spaces around it is saved without them.
    key::save(TARGET, &format!("  {MADE_UP_KEY}\r\n")).unwrap();
    let saved = key::load(TARGET).unwrap().expect("the key was saved");
    assert!(saved.is(MADE_UP_KEY), "read back as saved");
    assert_eq!(format!("{saved:?}"), "ApiKey(hidden)", "never printed");
    key::forget(TARGET).unwrap();
    assert!(key::load(TARGET).unwrap().is_none(), "forgotten");
    key::forget(TARGET).unwrap();
}

#[test]
fn what_is_not_a_key_is_refused_and_nothing_is_saved() {
    const TARGET: &str = "Steplet-test/made-up-key-refused";
    let _cleanup = Forget(&[TARGET]);
    key::forget(TARGET).unwrap();
    let long = "k".repeat(2049);
    for not_a_key in ["", "   ", "two words", "a\u{1b}[2Kkey", "tab\tinside", long.as_str()] {
        let refused = key::save(TARGET, not_a_key).expect_err(not_a_key);
        assert!(refused.starts_with("Steplet "), "Steplet's own words: {refused}");
        assert!(!refused.contains(not_a_key.trim()) || not_a_key.trim().is_empty(), "never repeats what was typed");
        assert!(key::load(TARGET).unwrap().is_none(), "{not_a_key:?} saved nothing");
    }
}

#[test]
fn a_pasted_key_is_trimmed() {
    assert_eq!(key::check(&format!("  {MADE_UP_NEBIUS_KEY}\r\n")), Ok(MADE_UP_NEBIUS_KEY));
    assert_eq!(key::check(&format!("\t{MADE_UP_KEY} ")), Ok(MADE_UP_KEY));
    // Nebius keys are long signed tokens: one of 2,048 characters, as long as Steplet takes, is a key.
    let long = format!("eyJ{}", "a".repeat(2045));
    assert_eq!(key::check(&long), Ok(long.as_str()));
}

#[test]
fn an_empty_key_is_refused() {
    for empty in ["", "   ", "\r\n", " \t "] {
        assert_eq!(key::check(empty), Err("Steplet needs a key: the box is empty.".to_string()), "{empty:?}");
    }
}

#[test]
fn the_nebius_key_has_its_own_entry() {
    // Steplet's real entries: the Anthropic one keeps its name, so a key saved before the Nebius copy is still found.
    assert_eq!(key::ENTRIES.of(KeyFor::Anthropic), "Steplet/anthropic-api-key");
    assert_eq!(key::ENTRIES.of(KeyFor::Nebius), "Steplet/nebius-api-key");
    const NEBIUS: &str = "Steplet-test/made-up-nebius-key";
    const ANTHROPIC: &str = "Steplet-test/made-up-anthropic-key";
    let _cleanup = Forget(&[NEBIUS, ANTHROPIC]);
    let entries = Entries { nebius: NEBIUS, anthropic: ANTHROPIC };
    for target in [NEBIUS, ANTHROPIC] {
        key::forget(target).unwrap();
    }
    let keys = || setup::view(&all_ready(), entries, Switch::Off).keys;

    // Saving one key leaves the other alone.
    key::save(entries.of(KeyFor::Nebius), MADE_UP_NEBIUS_KEY).unwrap();
    assert!(key::load(ANTHROPIC).unwrap().is_none(), "the Anthropic entry is untouched");
    assert!((keys().nebius, keys().anthropic) == (true, false));
    key::save(entries.of(KeyFor::Anthropic), MADE_UP_KEY).unwrap();
    assert!(key::load(NEBIUS).unwrap().expect("still saved").is(MADE_UP_NEBIUS_KEY), "the Nebius key is unchanged");
    assert!((keys().nebius, keys().anthropic) == (true, true));

    // Forgetting one key leaves the other alone.
    key::forget(entries.of(KeyFor::Nebius)).unwrap();
    assert!(key::load(ANTHROPIC).unwrap().expect("still saved").is(MADE_UP_KEY), "the Anthropic key is unchanged");
    assert!((keys().nebius, keys().anthropic) == (false, true));
    key::forget(entries.of(KeyFor::Anthropic)).unwrap();
    assert!((keys().nebius, keys().anthropic) == (false, false));
}

#[test]
fn the_setup_view_never_holds_the_key() {
    const NEBIUS: &str = "Steplet-test/made-up-key-view-nebius";
    const ANTHROPIC: &str = "Steplet-test/made-up-key-view";
    let _cleanup = Forget(&[NEBIUS, ANTHROPIC]);
    let entries = Entries { nebius: NEBIUS, anthropic: ANTHROPIC };
    key::save(NEBIUS, MADE_UP_NEBIUS_KEY).unwrap();
    key::save(ANTHROPIC, MADE_UP_KEY).unwrap();
    let view = setup::view(&all_ready(), entries, Switch::Off);
    assert!(view.keys.nebius && view.keys.anthropic);
    assert!(view.ready, "everything is there and a key is saved");
    let json = serde_json::to_string(&view).unwrap();
    assert!(!json.contains("STEPLET-TEST"), "the keys are not in the screen's data: {json}");
    key::forget(ANTHROPIC).unwrap();
    let view = setup::view(&all_ready(), entries, Switch::Off);
    assert!(!view.keys.anthropic);
    assert!(!view.ready, "no Anthropic key, so Claude Code cannot run yet");
}
