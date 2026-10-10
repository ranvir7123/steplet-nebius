use steplet_core::canon::{CanonError, canonicalize};

const SIG: &str = "signature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n";
const PEM: &str = "-----BEGIN SSH SIGNATURE-----\nU1NIU0lHAAAAAQ==\n-----END SSH SIGNATURE-----\n";

fn read(raw: &str) -> Result<steplet_core::canon::Canonical, CanonError> {
    canonicalize(raw.as_bytes())
}

#[test]
fn the_block_is_split_off() {
    let c = read(&format!("---\nsteplet: 1\ntitle: x\n{SIG}---\nbody\n")).unwrap();
    assert_eq!(c.signed(), "---\nsteplet: 1\ntitle: x\n---\nbody\n");
    assert_eq!(c.header(), "steplet: 1\ntitle: x\n");
    assert_eq!(c.body(), "body\n");
    let s = c.signature().unwrap();
    assert_eq!(s.pem, PEM);
    assert_eq!((s.first_line, s.last_line), (4, 7));
}

#[test]
fn windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned() {
    let raw = format!("---\nsteplet: 1\n{SIG}---\nb\n").replace('\n', " \r\n");
    assert_eq!(read(&raw).unwrap().signature().unwrap().pem, PEM);
}

#[test]
fn two_blocks() {
    assert_eq!(read(&format!("---\nsteplet: 1\n{SIG}{SIG}---\n")), Err(CanonError::SignatureTwice { line: 7 }));
}

#[test]
fn block_not_last_in_the_header() {
    assert_eq!(read(&format!("---\nsteplet: 1\n{SIG}title: x\n---\n")), Err(CanonError::SignatureMisplaced { line: 3 }));
}

#[test]
fn block_in_the_body() {
    assert_eq!(read(&format!("---\nsteplet: 1\n---\nbody\n{SIG}")), Err(CanonError::SignatureMisplaced { line: 6 }));
}

#[test]
fn markers_without_the_key_line() {
    let raw = "---\nsteplet: 1\n  -----BEGIN SSH SIGNATURE-----\n---\n";
    assert_eq!(read(raw), Err(CanonError::SignatureMisplaced { line: 3 }));
    let raw = "---\nsteplet: 1\n---\nsee -----END SSH SIGNATURE-----\n-----END SSH SIGNATURE----- trailing\n";
    assert_eq!(read(raw), Err(CanonError::SignatureMisplaced { line: 5 }));
}

#[test]
fn end_marker_missing() {
    let sig = "signature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n";
    assert_eq!(read(&format!("---\nsteplet: 1\n{sig}---\n")), Err(CanonError::SignatureBroken { line: 6 }));
}

#[test]
fn the_key_line_must_be_exact() {
    for key in ["signature: >", "signature: |-", "signature: abc", "signature:"] {
        let raw = format!("---\nsteplet: 1\n{}---\n", SIG.replacen("signature: |", key, 1));
        assert_eq!(read(&raw), Err(CanonError::SignatureBroken { line: 3 }), "{key}");
    }
}

#[test]
fn block_lines_must_be_indented_base64() {
    for bad in ["  U1NI U0lH", "U1NIU0lH", "   U1NIU0lH", "  U1NI#U0lH", "  "] {
        let raw = format!("---\nsteplet: 1\n{}---\n", SIG.replacen("  U1NIU0lHAAAAAQ==", bad, 1));
        assert!(matches!(read(&raw), Err(CanonError::SignatureBroken { line: 5 })), "{bad:?}");
    }
}

#[test]
fn with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one() {
    let unsigned = read("---\nsteplet: 1\ntitle: x\n---\nbody\n").unwrap();
    let signed = unsigned.with_signature("-----BEGIN SSH SIGNATURE-----\r\nU1NIU0lHAAAAAQ==\r\n-----END SSH SIGNATURE-----\r\n\r\n");
    assert_eq!(signed, format!("---\nsteplet: 1\ntitle: x\n{SIG}---\nbody\n"));
    let again = read(&signed).unwrap();
    assert_eq!(again.signed(), unsigned.signed());
    assert_eq!(again.with_signature(PEM), signed);
}
