const MSG: &[u8] = include_bytes!("vectors/msg.txt");
const SIG: &str = include_str!("vectors/msg.txt.sig");
const FINGERPRINT: &str = include_str!("vectors/fingerprint.txt");

#[test]
fn ssh_keygen_signature_verifies_with_the_same_fingerprint() {
    assert_eq!(s9_core::verify(MSG, SIG, "steplet-v1"), Ok(FINGERPRINT.trim().to_string()));
}

#[test]
fn changed_message_fails() {
    assert!(s9_core::verify(b"Steplet spike S9 test message!", SIG, "steplet-v1").is_err());
}

#[test]
fn wrong_namespace_fails() {
    assert!(s9_core::verify(MSG, SIG, "runfile-v1").is_err());
}
