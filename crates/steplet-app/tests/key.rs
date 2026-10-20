//! The receiver's Anthropic key (Task 11, smaller choice S3): kept in Windows' own Credential Manager, locked to this
//! Windows account, and never in the data the screens get. These tests use made-up entries of their own, never
//! Steplet's real one, and remove them when they end, passed or failed.

use steplet_app::key;
use steplet_app::setup;
use steplet_app::views::Switch;
use steplet_run::setup::{State, Wsl};

/// Made-up text in a key's shape; never a real key.
const MADE_UP_KEY: &str = "sk-ant-api03-STEPLET-TEST-MADE-UP-NOT-A-KEY";

/// Removes a test's entry when the test ends, passed or failed.
struct Forget(&'static str);

impl Drop for Forget {
    fn drop(&mut self) {
        let _ = key::forget(self.0);
    }
}

#[test]
fn key_is_stored_read_and_forgotten() {
    const TARGET: &str = "Steplet-test/made-up-key";
    let _cleanup = Forget(TARGET);
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
    let _cleanup = Forget(TARGET);
    key::forget(TARGET).unwrap();
    let long = "k".repeat(513);
    for not_a_key in ["", "   ", "two words", "a\u{1b}[2Kkey", "tab\tinside", long.as_str()] {
        let refused = key::save(TARGET, not_a_key).expect_err(not_a_key);
        assert!(refused.starts_with("Steplet "), "Steplet's own words: {refused}");
        assert!(!refused.contains(not_a_key.trim()) || not_a_key.trim().is_empty(), "never repeats what was typed");
        assert!(key::load(TARGET).unwrap().is_none(), "{not_a_key:?} saved nothing");
    }
}

#[test]
fn the_setup_view_never_holds_the_key() {
    const TARGET: &str = "Steplet-test/made-up-key-view";
    let _cleanup = Forget(TARGET);
    let state = State {
        wsl: Wsl::Ready { version: "2.6.3.0".into() },
        linux: Some("{made-up-id}".into()),
        missing: Vec::new(),
        gate: true,
        trouble: None,
    };
    key::save(TARGET, MADE_UP_KEY).unwrap();
    let view = setup::view(&state, TARGET, Switch::Off);
    assert!(view.key_saved);
    assert!(view.ready, "everything is there and a key is saved");
    let json = serde_json::to_string(&view).unwrap();
    assert!(!json.contains("STEPLET-TEST"), "the key is not in the screen's data: {json}");
    key::forget(TARGET).unwrap();
    let view = setup::view(&state, TARGET, Switch::Off);
    assert!(!view.key_saved);
    assert!(!view.ready, "no key, so not ready to run");
}
