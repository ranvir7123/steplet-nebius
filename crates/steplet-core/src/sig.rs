//! Signature checks (decision rows 13 and 14, P4). Ed25519 keys, sha512 and SSHSIG version 1 only.

use alloc::string::{String, ToString};
use core::fmt;

use ssh_key::{Algorithm, HashAlg, LineEnding, PublicKey, SshSig};

use crate::canon::Canonical;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SigStatus {
    Unsigned,
    Valid { fingerprint: String },
    Invalid { reason: InvalidReason },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidReason {
    NotASignature,
    /// `ssh-key` 0.6.7 already refuses versions above 1 while reading (so this shows as `NotASignature`);
    /// this check stays as a second lock.
    WrongVersion,
    WrongNamespace {
        found: String,
    },
    WrongHash,
    WrongKeyType,
    DoesNotVerify,
    /// The block reads as a signature, but is not exactly the text its signer wrote (for example, a length
    /// inside it was edited). One signature has one text.
    NotCanonical,
}

pub fn verify(c: &Canonical, namespace: &str) -> SigStatus {
    let Some(block) = c.signature() else { return SigStatus::Unsigned };
    let bad = |reason| SigStatus::Invalid { reason };
    let Ok(sig) = SshSig::from_pem(&block.pem) else { return bad(InvalidReason::NotASignature) };
    if sig.version() != 1 {
        return bad(InvalidReason::WrongVersion);
    }
    if sig.namespace() != namespace {
        return bad(InvalidReason::WrongNamespace { found: sig.namespace().to_string() });
    }
    if sig.hash_alg() != HashAlg::Sha512 {
        return bad(InvalidReason::WrongHash);
    }
    if sig.algorithm() != Algorithm::Ed25519 {
        return bad(InvalidReason::WrongKeyType);
    }
    // One text per signature. `ssh-key` 0.6.7 does not check that a length written inside the signature
    // matches what it then reads, so a block with an edited length still verified, although OpenSSH's
    // ssh-keygen refuses it (found by the property tests, 11 Oct 2026). Writing the signature out again
    // must give back exactly the block that is in the file.
    if sig.to_pem(LineEnding::LF).ok().as_deref() != Some(block.pem.as_str()) {
        return bad(InvalidReason::NotCanonical);
    }
    let key = PublicKey::from(sig.public_key().clone());
    if key.verify(namespace, c.signed().as_bytes(), &sig).is_err() {
        return bad(InvalidReason::DoesNotVerify);
    }
    SigStatus::Valid { fingerprint: key.fingerprint(HashAlg::Sha256).to_string() }
}

impl fmt::Display for InvalidReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidReason::NotASignature => write!(f, "the signature block is not a readable SSH signature"),
            InvalidReason::WrongVersion => write!(f, "the signature uses a format version other than 1"),
            InvalidReason::WrongNamespace { found } => {
                write!(f, "the signature was made for \"{found}\", not for Steplet files (steplet-v1)")
            }
            InvalidReason::WrongHash => write!(f, "the signature uses a hash other than sha512"),
            InvalidReason::WrongKeyType => write!(f, "the signing key is not an Ed25519 key"),
            InvalidReason::DoesNotVerify => {
                write!(f, "the file was changed after it was signed, or the signature belongs to another file")
            }
            InvalidReason::NotCanonical => {
                write!(f, "the signature block was edited after signing; it must stay exactly as it was written")
            }
        }
    }
}
