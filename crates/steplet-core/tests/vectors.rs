use steplet_core::NAMESPACE;
use steplet_core::canon::{CanonError, canonicalize};
use steplet_core::header::parse;
use steplet_core::sig::{InvalidReason, SigStatus, verify};

const FINGERPRINT: &str = include_str!("vectors/fingerprint.txt");

fn status(raw: &[u8]) -> Result<SigStatus, CanonError> {
    canonicalize(raw).map(|c| verify(&c, NAMESPACE))
}
fn invalid(reason: InvalidReason) -> Result<SigStatus, CanonError> {
    Ok(SigStatus::Invalid { reason })
}

#[test]
fn ssh_keygen_signatures_verify_with_the_same_fingerprint() {
    let valid = Ok(SigStatus::Valid { fingerprint: FINGERPRINT.trim().into() });
    assert_eq!(status(include_bytes!("vectors/valid.steplet")), valid);
    assert_eq!(status(include_bytes!("vectors/crlf.steplet")), valid);
}

#[test]
fn windows_line_endings_clean_up_to_the_same_signed_text() {
    let a = canonicalize(include_bytes!("vectors/valid.steplet")).unwrap();
    let b = canonicalize(include_bytes!("vectors/crlf.steplet")).unwrap();
    assert!(include_bytes!("vectors/crlf.steplet").windows(2).any(|w| w == b"\r\n"));
    assert_eq!(a.signed(), b.signed());
    assert_eq!(a.signed().as_bytes(), include_bytes!("vectors/unsigned.steplet"));
}

#[test]
fn refused_signatures() {
    assert_eq!(status(include_bytes!("vectors/tampered.steplet")), invalid(InvalidReason::DoesNotVerify));
    assert_eq!(
        status(include_bytes!("vectors/wrong-namespace.steplet")),
        invalid(InvalidReason::WrongNamespace { found: "runfile-v1".into() })
    );
    assert_eq!(status(include_bytes!("vectors/sha256.steplet")), invalid(InvalidReason::WrongHash));
    assert_eq!(status(include_bytes!("vectors/rsa.steplet")), invalid(InvalidReason::WrongKeyType));
}

#[test]
fn missing_cut_doubled_and_moved_blocks() {
    assert_eq!(status(include_bytes!("vectors/unsigned.steplet")), Ok(SigStatus::Unsigned));
    assert!(matches!(status(include_bytes!("vectors/cut.steplet")), Err(CanonError::SignatureBroken { .. })));
    assert!(matches!(status(include_bytes!("vectors/doubled.steplet")), Err(CanonError::SignatureTwice { .. })));
    assert_eq!(
        status(include_bytes!("vectors/moved-in-header.steplet")),
        Err(CanonError::SignatureMisplaced { line: 9 })
    );
    assert!(matches!(
        status(include_bytes!("vectors/moved-to-body.steplet")),
        Err(CanonError::SignatureMisplaced { .. })
    ));
}

#[test]
fn the_signed_header_reads() {
    let c = canonicalize(include_bytes!("vectors/valid.steplet")).unwrap();
    assert_eq!(parse(&c).unwrap().steps.len(), 2);
}
