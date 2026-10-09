//! Spike: check an SSH signature (SSHSIG, armoured) without the operating system (milestone 1, phase 0). Throwaway.
#![no_std]
extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use ssh_key::{Algorithm, HashAlg, PublicKey, SshSig};

/// Checks `sig_pem` over `msg` against the key embedded in the signature. Ok holds that key's SHA256 fingerprint.
pub fn verify(msg: &[u8], sig_pem: &str, namespace: &str) -> Result<String, String> {
    let sig = SshSig::from_pem(sig_pem).map_err(|e| format!("not a signature: {e}"))?;
    if sig.version() != 1 {
        return Err("signature version is not 1".into());
    }
    if sig.hash_alg() != HashAlg::Sha512 {
        return Err("hash is not sha512".into());
    }
    if sig.algorithm() != Algorithm::Ed25519 {
        return Err("key is not Ed25519".into());
    }
    let key = PublicKey::from(sig.public_key().clone());
    key.verify(namespace, msg, &sig).map_err(|e| format!("does not verify: {e}"))?;
    Ok(key.fingerprint(HashAlg::Sha256).to_string())
}
