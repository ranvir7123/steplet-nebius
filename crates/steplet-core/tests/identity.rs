//! Who signed a file, as far as this computer knows (rows 16 and 22, R14, smaller choice S6; phase 4 Task 7). The keys
//! below are made-up test data, not real key fingerprints.

use steplet_core::identity::{Identity, Pin, identity};
use steplet_core::rules::{self, Finding, MACHINE_RULES, RULES, Severity};
use steplet_core::sig::{InvalidReason, SigStatus};

const ANA: &str = "SHA256:made-up-key-of-ana-lee";
const OTHER: &str = "SHA256:made-up-other-key";

fn pin(sender: &str, key: &str, runs: u32) -> Pin {
    Pin { sender: sender.into(), key: key.into(), runs }
}

fn signed(key: &str) -> SigStatus {
    SigStatus::Valid { fingerprint: key.into() }
}

fn key_changed(f: Option<Finding>) -> Finding {
    let f = f.expect("a finding");
    assert_eq!(f.rule.id, "id.key-changed");
    assert_eq!(f.rule.severity, Severity::Red);
    assert_eq!(f.line, None, "about the whole file");
    f
}

#[test]
fn a_known_sender_with_another_key_is_red() {
    let pins = [pin("Ana Lee", ANA, 1)];
    let (id, f) = identity("Ana Lee", &signed(OTHER), &pins);
    assert_eq!(id, Identity::NewKey { key: OTHER.into() });
    let f = key_changed(f);
    assert_eq!(f.quote.as_deref(), Some("Ana Lee"), "the name is file text, so it travels as the quote");
    assert_eq!(
        f.detail.as_deref(),
        Some(
            "Steplet knows this sender name with the key SHA256:made-up-key-of-ana-lee. This file is signed with \
             SHA256:made-up-other-key."
        )
    );
    // The same name written with other capitals or spaces is the same name: the warning cannot be dodged that way.
    for written in ["ana lee", "Ana  Lee", "ANA LEE", "Ana\tLee"] {
        let (id, f) = identity(written, &signed(OTHER), &pins);
        assert_eq!(id, Identity::NewKey { key: OTHER.into() }, "{written:?}");
        assert_eq!(key_changed(f).quote.as_deref(), Some(written));
    }
}

#[test]
fn a_known_sender_unsigned_is_red() {
    let (id, f) = identity("Ana Lee", &SigStatus::Unsigned, &[pin("Ana Lee", ANA, 4)]);
    assert_eq!(id, Identity::Unsigned);
    assert_eq!(
        key_changed(f).detail.as_deref(),
        Some("Steplet knows this sender name with the key SHA256:made-up-key-of-ana-lee. This file is not signed.")
    );
    // Task 7's review: other capitals or spaces are the same name here too.
    for written in ["ana lee", "ANA\u{a0}LEE"] {
        let (id, f) = identity(written, &SigStatus::Unsigned, &[pin("Ana Lee", ANA, 4)]);
        assert_eq!(id, Identity::Unsigned, "{written:?}");
        key_changed(f);
    }
}

/// Task 7's review: what the comparison does not join. Such a name is another name, so it gets "sender unknown",
/// never "the same key as before".
#[test]
fn only_capitals_and_spaces_are_ignored() {
    let pins = [pin("Straße", ANA, 1), pin("José", ANA, 1)];
    for other in ["STRASSE", "Jose\u{301}"] {
        assert_eq!(identity(other, &signed(OTHER), &pins), (Identity::NewKey { key: OTHER.into() }, None), "{other:?}");
    }
}

/// Task 7's review: an empty name matches no pin. No file can have one (the header reader refuses it), so only a
/// damaged pins file could hold one; it must not make every nameless call match.
#[test]
fn an_empty_name_matches_no_pin() {
    let pins = [pin("", ANA, 1), pin("   ", ANA, 1)];
    assert_eq!(identity("", &SigStatus::Unsigned, &pins), (Identity::Unsigned, None));
    assert_eq!(identity(" ", &signed(ANA), &pins).1, None);
    assert_ne!(identity(" ", &signed(ANA), &pins).0, Identity::SameKey { key: ANA.into(), runs: 1 });
}

#[test]
fn the_same_key_gives_the_same_key_line_and_no_finding() {
    let pins = [pin("Bob Ray", OTHER, 1), pin("Ana Lee", ANA, 2)];
    assert_eq!(identity("Ana Lee", &signed(ANA), &pins), (Identity::SameKey { key: ANA.into(), runs: 2 }, None));
    assert_eq!(identity("ana  lee", &signed(ANA), &pins), (Identity::SameKey { key: ANA.into(), runs: 2 }, None));
}

#[test]
fn a_new_sender_is_new_key() {
    assert_eq!(identity("Ana Lee", &signed(ANA), &[]), (Identity::NewKey { key: ANA.into() }, None));
    let pins = [pin("Bob Ray", OTHER, 1)];
    assert_eq!(identity("Ana Lee", &signed(ANA), &pins), (Identity::NewKey { key: ANA.into() }, None));
    assert_eq!(identity("Ana Lee", &SigStatus::Unsigned, &pins), (Identity::Unsigned, None), "unsigned, name unknown");
}

#[test]
fn a_known_key_under_another_name_is_said_but_not_a_finding() {
    let pins = [pin("Ana Lee", ANA, 3), pin("A. Lee", ANA, 1), pin("Bob Ray", OTHER, 1)];
    assert_eq!(
        identity("Ana at work", &signed(ANA), &pins),
        (Identity::KeyKnownAs { key: ANA.into(), names: vec!["Ana Lee".into(), "A. Lee".into()] }, None)
    );
}

#[test]
fn an_invalid_signature_gives_no_second_finding() {
    let pins = [pin("Ana Lee", ANA, 1)];
    let bad = SigStatus::Invalid { reason: InvalidReason::DoesNotVerify };
    assert_eq!(identity("Ana Lee", &bad, &pins), (Identity::Invalid, None), "sig.invalid already blocks the file");
}

#[test]
fn the_machine_rule_is_not_in_rules() {
    let r = rules::KEY_CHANGED;
    assert_eq!((r.id, r.severity), ("id.key-changed", Severity::Red));
    assert_eq!(r.title, "This sender name was used before with another key");
    assert_eq!(
        r.advice,
        "Someone may be using this name. Ask the sender, another way, whether they changed their key. If they did, \
         type the sentence to run it; Steplet then remembers the new key."
    );
    assert_eq!(MACHINE_RULES, [rules::KEY_CHANGED]);
    // No file alone can hit it, so the corpus's "every rule has a hit" stays true.
    assert!(RULES.iter().all(|x| x.id != r.id));
    // The same plain-text limits as every rule (docs/rules.md is a table).
    for text in [r.title, r.advice] {
        assert!(text.bytes().all(|b| (b' '..=b'~').contains(&b) && !b"\"\\|".contains(&b)), "{text}");
    }
}
