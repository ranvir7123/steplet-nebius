//! Signs test files with a throwaway key made from a fixed seed. Only for tests.

use ssh_key::private::Ed25519Keypair;
use ssh_key::{HashAlg, LineEnding, PrivateKey};
use steplet_core::canon::canonicalize;

pub fn key(seed: u8) -> PrivateKey {
    PrivateKey::from(Ed25519Keypair::from_seed(&[seed; 32]))
}

pub fn pem_for(text: &str, key: &PrivateKey, namespace: &str, hash: HashAlg) -> String {
    key.sign(namespace, hash, text.as_bytes()).unwrap().to_pem(LineEnding::LF).unwrap()
}

/// `file` signed in the Steplet way: the signature covers the cleaned-up text.
pub fn sign(file: &str, seed: u8) -> String {
    let c = canonicalize(file.as_bytes()).unwrap();
    c.with_signature(&pem_for(c.signed(), &key(seed), steplet_core::NAMESPACE, HashAlg::Sha512))
}
