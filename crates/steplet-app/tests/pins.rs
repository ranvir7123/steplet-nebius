//! The senders this computer knows (Task 10, rows 16 and 22, S5): one pin per sender name, never an empty name, a key
//! in the form Steplet's signature check gives it, and a file that breaks any of that is kept and said, never
//! overwritten.

use std::path::PathBuf;

use steplet_app::pins::{self, Pins};
use steplet_core::identity::Pin;

const KEY_1: &str = "SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk";
const KEY_2: &str = "SHA256:Q2hhbmdlZEtleUNoYW5nZWRLZXlDaGFuZ2VkS2V5Q2g";

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("pins-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("senders.json")
}

fn known(p: &std::path::Path) -> Vec<Pin> {
    match pins::load(p) {
        Pins::Known(list) => list,
        Pins::Unreadable(why) => panic!("unreadable: {why}"),
    }
}

#[test]
fn no_file_yet_means_no_senders_known() {
    assert_eq!(known(&scratch("none")), Vec::new());
}

#[test]
fn one_pin_per_sender_name_and_a_new_key_replaces_the_old() {
    let p = scratch("one");
    pins::record(&p, "Ana Lee", KEY_1).unwrap();
    pins::record(&p, "ana  lee", KEY_1).unwrap();
    assert_eq!(known(&p), [Pin { sender: "Ana Lee".into(), key: KEY_1.into(), runs: 2 }]);
    // S5: after a run with a changed key (the receiver typed the red sentence), the new key replaces the old one.
    pins::record(&p, "ANA LEE", KEY_2).unwrap();
    assert_eq!(known(&p), [Pin { sender: "ANA LEE".into(), key: KEY_2.into(), runs: 1 }]);
    pins::record(&p, "Bo", KEY_1).unwrap();
    assert_eq!(known(&p).len(), 2);
    let left: Vec<String> = std::fs::read_dir(p.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, ["senders.json"], "no temporary file is left");
}

#[test]
fn an_empty_name_or_a_bad_key_is_never_pinned() {
    let p = scratch("bad");
    assert!(pins::record(&p, "", KEY_1).is_err());
    assert!(pins::record(&p, "   ", KEY_1).is_err());
    assert!(pins::record(&p, "Ana Lee", "not a key").is_err());
    assert!(pins::record(&p, "Ana Lee", "SHA256:short").is_err());
    assert!(!p.exists(), "nothing was written");
}

#[test]
fn a_pins_file_that_breaks_the_rules_is_unreadable() {
    let p = scratch("rules");
    let cases = [
        r#"{"version": 1, "senders": [{"sender": "Ana Lee", "key": "K", "runs": 1}]}"#,
        r#"{"version": 2, "senders": []}"#,
        r#"{"version": 1, "senders": [{"sender": "", "key": "SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk", "runs": 1}]}"#,
        r#"{"version": 1, "senders": [{"sender": "Ana Lee", "key": "SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk", "runs": 1},
                                      {"sender": "ana lee", "key": "SHA256:Q2hhbmdlZEtleUNoYW5nZWRLZXlDaGFuZ2VkS2V5Q2g", "runs": 1}]}"#,
        r#"{"version": 1, "senders": [], "extra": true}"#,
        r#"{"version": 1, "senders": [{"sender": "Ana Lee", "key": "SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk", "runs": 0}]}"#,
    ];
    for text in cases {
        std::fs::write(&p, text).unwrap();
        assert!(matches!(pins::load(&p), Pins::Unreadable(_)), "{text}");
        assert!(pins::record(&p, "Bo", KEY_1).is_err(), "{text}: not overwritten");
        assert_eq!(std::fs::read_to_string(&p).unwrap(), text);
    }
}
