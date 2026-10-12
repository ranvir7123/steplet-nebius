//! Runs the real steplet-sign program with Windows' own ssh-keygen and throwaway keys.
#![cfg(windows)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ssh_key::private::Ed25519Keypair;
use ssh_key::{HashAlg, LineEnding, PrivateKey};
use steplet_core::canon::canonicalize;
use steplet_core::date::Date;

const KEYGEN: &str = r"C:\Windows\System32\OpenSSH\ssh-keygen.exe";

struct Scratch(PathBuf);
impl Scratch {
    fn new(name: &str) -> Scratch {
        let d = std::env::temp_dir().join(format!("steplet-sign-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        Scratch(d)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn file_text(expires: Option<Date>) -> String {
    let expires = expires.unwrap_or_else(|| steplet_sign::today().add_days(14).unwrap());
    format!(
        "---\nsteplet: 1\ntitle: Count the README lines\nsender: Test\nexpires: {expires}\nsteps:\n  - Count the lines.\n---\nCount them.\n"
    )
}

fn keygen(dir: &Path, name: &str, kind: &[&str]) -> PathBuf {
    let k = dir.join(name);
    let ok = Command::new(KEYGEN)
        .arg("-t")
        .args(kind)
        .args(["-N", "", "-q", "-C", "steplet-sign-test", "-f"])
        .arg(&k)
        .status()
        .unwrap();
    assert!(ok.success());
    k
}

fn run(args: &[&std::ffi::OsStr]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_steplet-sign")).args(args).stdin(Stdio::null()).output().unwrap();
    (o.status.code().unwrap(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn sign(file: &Path, key: &Path) -> (i32, String) {
    run(&["sign".as_ref(), file.as_os_str(), "--key".as_ref(), key.as_os_str()])
}

fn check(file: &Path) -> (i32, String) {
    run(&["check".as_ref(), file.as_os_str()])
}

fn fingerprint(public_key: &Path) -> String {
    let o = Command::new(KEYGEN).arg("-l").arg("-f").arg(public_key).output().unwrap();
    String::from_utf8_lossy(&o.stdout).split_whitespace().nth(1).unwrap().to_string()
}

/// ssh-keygen's own check of `pem` over `payload`, with no key list: exit code 0 means the signature is good.
fn keygen_accepts(dir: &Path, payload: &str, pem: &str) -> bool {
    let (p, s) = (dir.join("payload"), dir.join("payload.sig"));
    fs::write(&p, payload).unwrap();
    fs::write(&s, pem).unwrap();
    Command::new(KEYGEN)
        .args(["-Y", "check-novalidate", "-n", "steplet-v1", "-s"])
        .arg(&s)
        .stdin(fs::File::open(&p).unwrap())
        .status()
        .unwrap()
        .success()
}

#[test]
fn sign_then_check_shows_ssh_keygens_fingerprint() {
    let d = Scratch::new("sign");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None)).unwrap();
    let (code, out) = sign(&file, &key);
    assert_eq!(code, 0, "{out}");
    let fp = fingerprint(&key.with_extension("pub"));
    assert!(out.contains(&fp), "{out}");
    let (code, out) = check(&file);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains(&format!("signature: valid, key {fp}")), "{out}");
}

#[test]
fn ssh_keygen_accepts_what_we_signed_and_what_we_cut_out() {
    let d = Scratch::new("agree");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None)).unwrap();
    assert_eq!(sign(&file, &key).0, 0);
    let c = canonicalize(&fs::read(&file).unwrap()).unwrap();
    assert!(keygen_accepts(&d.0, c.signed(), &c.signature().unwrap().pem));
    assert!(!keygen_accepts(&d.0, &c.signed().replace("Count them.", "Count them!"), &c.signature().unwrap().pem));
}

#[test]
fn ssh_keygen_accepts_a_signature_made_in_rust() {
    let d = Scratch::new("rust");
    let c = canonicalize(file_text(None).as_bytes()).unwrap();
    let key = PrivateKey::from(Ed25519Keypair::from_seed(&[3; 32]));
    let pem = key.sign("steplet-v1", HashAlg::Sha512, c.signed().as_bytes()).unwrap().to_pem(LineEnding::LF).unwrap();
    assert!(keygen_accepts(&d.0, c.signed(), &pem));
}

#[test]
fn an_rsa_key_is_refused_and_the_file_is_left_alone() {
    let d = Scratch::new("rsa");
    let key = keygen(&d.0, "k", &["rsa", "-b", "3072"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None)).unwrap();
    let (code, out) = sign(&file, &key);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("not an Ed25519 key") && out.contains("ssh-keygen -t ed25519"), "{out}");
    assert_eq!(fs::read_to_string(&file).unwrap(), file_text(None));
}

#[test]
fn expiry_must_be_from_today_to_365_days_ahead() {
    let d = Scratch::new("expiry");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    for days in [-1, 366] {
        let text = file_text(steplet_sign::today().add_days(days));
        fs::write(&file, &text).unwrap();
        let (code, out) = sign(&file, &key);
        assert_eq!(code, 1, "{days}: {out}");
        assert_eq!(fs::read_to_string(&file).unwrap(), text, "{days}");
    }
}

#[test]
fn signing_again_after_an_edit_replaces_the_signature() {
    let d = Scratch::new("again");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None)).unwrap();
    assert_eq!(sign(&file, &key).0, 0);
    let edited = fs::read_to_string(&file).unwrap().replace("Count them.", "Count them, then stop.");
    fs::write(&file, edited).unwrap();
    assert_eq!(check(&file).0, 1);
    let (code, out) = sign(&file, &key);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("replaced the signature that was there before"), "{out}");
    assert_eq!(check(&file).0, 0);
    assert_eq!(fs::read_to_string(&file).unwrap().matches("BEGIN SSH SIGNATURE").count(), 1);
}

#[test]
fn windows_line_endings_are_written_back_as_unix_ones_and_still_verify() {
    let d = Scratch::new("crlf");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None).replace('\n', "\r\n")).unwrap();
    assert_eq!(sign(&file, &key).0, 0);
    let written = fs::read(&file).unwrap();
    assert!(!written.contains(&b'\r'));
    assert_eq!(check(&file).0, 0);
}

#[test]
fn header_problems_stop_signing() {
    let d = Scratch::new("bad");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    let text = file_text(None).replace("  - Count", "\t- Count");
    fs::write(&file, &text).unwrap();
    let (code, out) = sign(&file, &key);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("line 7: a tab character"), "{out}");
    assert_eq!(fs::read_to_string(&file).unwrap(), text);
}

/// Decision F10: a file that cannot be read is exit code 2, for `sign` as for `check`.
#[test]
fn a_file_that_cannot_be_read_is_exit_code_2() {
    let d = Scratch::new("missing");
    let missing = d.0.join("no-such-file.steplet");
    let (code, out) = sign(&missing, &d.0.join("no-such-key"));
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("cannot read"), "{out}");
    let (code, out) = check(&missing);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("cannot read"), "{out}");
}

#[test]
fn wrong_use_prints_how_to_use_it() {
    let (code, out) = run(&["sign".as_ref(), "x.steplet".as_ref()]);
    assert_eq!(code, 2);
    assert!(out.contains("usage:"), "{out}");
}
