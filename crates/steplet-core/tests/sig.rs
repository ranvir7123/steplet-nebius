mod common;

use ssh_key::HashAlg;
use steplet_core::NAMESPACE;
use steplet_core::canon::canonicalize;
use steplet_core::sig::{InvalidReason, SigStatus, verify};

const FILE: &str = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n  - a\n---\nbody\n";

fn status(file: &str) -> SigStatus {
    verify(&canonicalize(file.as_bytes()).unwrap(), NAMESPACE)
}

#[test]
fn unsigned() {
    assert_eq!(status(FILE), SigStatus::Unsigned);
}

#[test]
fn valid_with_the_keys_fingerprint() {
    let fp = common::key(1).public_key().fingerprint(HashAlg::Sha256).to_string();
    assert!(fp.starts_with("SHA256:"));
    assert_eq!(status(&common::sign(FILE, 1)), SigStatus::Valid { fingerprint: fp });
}

#[test]
fn changed_body() {
    let signed = common::sign(FILE, 1).replace("body", "bodY");
    assert_eq!(status(&signed), SigStatus::Invalid { reason: InvalidReason::DoesNotVerify });
}

#[test]
fn wrong_name_space_and_wrong_hash() {
    let c = canonicalize(FILE.as_bytes()).unwrap();
    let other = c.with_signature(&common::pem_for(c.signed(), &common::key(1), "runfile-v1", HashAlg::Sha512));
    assert_eq!(
        status(&other),
        SigStatus::Invalid { reason: InvalidReason::WrongNamespace { found: "runfile-v1".into() } }
    );
    let sha256 = c.with_signature(&common::pem_for(c.signed(), &common::key(1), NAMESPACE, HashAlg::Sha256));
    assert_eq!(status(&sha256), SigStatus::Invalid { reason: InvalidReason::WrongHash });
}

#[test]
fn not_a_signature() {
    let c = canonicalize(FILE.as_bytes()).unwrap();
    let junk = c.with_signature("-----BEGIN SSH SIGNATURE-----\nU1NIU0lHAAAAAQ==\n-----END SSH SIGNATURE-----\n");
    assert_eq!(status(&junk), SigStatus::Invalid { reason: InvalidReason::NotASignature });
}

/// The signature's base64 text, all lines joined.
fn base64_of(signed: &str) -> String {
    let c = canonicalize(signed.as_bytes()).unwrap();
    c.signature().unwrap().pem.lines().filter(|l| !l.starts_with("-----")).collect()
}

/// `signed` with its signature block rebuilt from `b64`, cut into lines of `width` characters.
fn with_base64(signed: &str, b64: &str, width: usize) -> String {
    let mut pem = String::from("-----BEGIN SSH SIGNATURE-----\n");
    for line in b64.as_bytes().chunks(width) {
        pem.push_str(std::str::from_utf8(line).unwrap());
        pem.push('\n');
    }
    pem.push_str("-----END SSH SIGNATURE-----\n");
    canonicalize(signed.as_bytes()).unwrap().with_signature(&pem)
}

const NOT_CANONICAL: SigStatus = SigStatus::Invalid { reason: InvalidReason::NotCanonical };

/// Found by `any_edit_to_non_whitespace_breaks_the_signature` on 11 Oct 2026. Every Ed25519 signature starts
/// with the same bytes; the 17th base64 character is part of the length written before the public key.
/// `ssh-key` 0.6.7 does not check that a length matches what it then reads, so `A` -> `x` (length 51 ->
/// 50227) used to leave the signature valid. The signed text is unchanged; only the signature block is.
#[test]
fn an_edited_length_before_the_public_key_is_refused() {
    let signed = common::sign(FILE, 1);
    let header = "  U1NIU0lHAAAAAQAAADMAAAAL";
    assert!(signed.contains(header), "every Ed25519 signature starts with these characters");
    let edited = signed.replacen(header, "  U1NIU0lHAAAAAQAAxDMAAAAL", 1);
    assert_eq!(status(&edited), NOT_CANONICAL);
}

/// The same hole in the other length: the one written before the signature bytes. With an Ed25519 key,
/// name space `steplet-v1` and sha512, that length (83) is in bytes 93 to 96; base64 characters 124 to 127
/// encode bytes 93 to 95. Changing character 127 from `A` to `B` makes the length 339.
#[test]
fn an_edited_length_before_the_signature_bytes_is_refused() {
    let signed = common::sign(FILE, 1);
    let mut b64 = base64_of(&signed);
    assert_eq!(&b64[124..128], "AAAA");
    b64.replace_range(127..128, "B");
    assert_eq!(status(&with_base64(&signed, &b64, 70)), NOT_CANONICAL);
}

/// One text per signature: lines of 70 characters, as OpenSSH writes them. `ssh-key` already refuses
/// other widths while reading, so the answer is `NotASignature`; the re-encoding check is a second lock.
#[test]
fn re_wrapped_signature_lines_are_refused() {
    let signed = common::sign(FILE, 1);
    let b64 = base64_of(&signed);
    assert_eq!(with_base64(&signed, &b64, 70), signed, "rebuilding at 70 characters gives back the same file");
    for width in [64, 69, 71, 76] {
        let rewrapped = with_base64(&signed, &b64, width);
        assert_eq!(status(&rewrapped), SigStatus::Invalid { reason: InvalidReason::NotASignature }, "{width}");
    }
}

#[test]
fn reasons_in_words() {
    for r in [
        InvalidReason::NotASignature,
        InvalidReason::WrongVersion,
        InvalidReason::WrongNamespace { found: "x".into() },
        InvalidReason::WrongHash,
        InvalidReason::WrongKeyType,
        InvalidReason::DoesNotVerify,
        InvalidReason::NotCanonical,
    ] {
        assert!(!r.to_string().is_empty());
    }
}
