//! `steplet-sign sign`: signs with the sender's own ssh-keygen, then re-checks with Steplet's own core
//! before anything is written. The file is replaced only when every step succeeded.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use steplet_core::NAMESPACE;
use steplet_core::canon::canonicalize;
use steplet_core::date::Date;
use steplet_core::limits::MAX_FILE_BYTES;
use steplet_core::sig::{self, SigStatus};

use crate::check::{content, problem};

/// Windows' own ssh-keygen. Called by full path: Git Bash's PATH finds a different one first.
pub const DEFAULT_SSH_KEYGEN: &str = if cfg!(windows) { r"C:\Windows\System32\OpenSSH\ssh-keygen.exe" } else { "ssh-keygen" };

pub struct SignRequest<'a> {
    pub file: &'a Path,
    pub key: &'a Path,
    pub ssh_keygen: &'a Path,
    pub today: Date,
}

pub fn sign(req: &SignRequest) -> Result<String, String> {
    let name = req.file.display();
    let raw = fs::read(req.file).map_err(|e| format!("cannot read {name}: {e}"))?;
    let c = canonicalize(&raw).map_err(|e| format!("{name}: not signed\n{}", problem(&e.to_string(), e.help().allowed, e.help().example)))?;
    let (steplet, problems) = content(&c, req.today);
    let steplet = match steplet {
        Some(s) if problems.is_empty() => s,
        _ => return Err(format!("{name}: not signed\n{}", problems.concat())),
    };
    let replacing = c.signature().is_some();

    let work = TempDir::new().map_err(|e| format!("cannot make a temporary folder: {e}"))?;
    let payload = work.0.join("payload.steplet");
    fs::write(&payload, c.signed()).map_err(|e| format!("cannot write {}: {e}", payload.display()))?;
    let status = Command::new(req.ssh_keygen)
        .args(["-Y", "sign", "-f"])
        .arg(req.key)
        .args(["-n", NAMESPACE, "-O", "hashalg=sha512"])
        .arg(&payload)
        .status()
        .map_err(|e| format!("cannot start {}: {e}", req.ssh_keygen.display()))?;
    if !status.success() {
        return Err(format!(
            "{name}: not signed; ssh-keygen stopped with exit code {} (its message is above)",
            status.code().map_or("none".into(), |c| c.to_string())
        ));
    }
    let pem = fs::read_to_string(work.0.join("payload.steplet.sig")).map_err(|e| format!("cannot read ssh-keygen's signature: {e}"))?;

    let signed = c.with_signature(&pem);
    if signed.len() > MAX_FILE_BYTES {
        return Err(format!(
            "{name}: not signed; with the signature block the file would be {} bytes, over the limit of {MAX_FILE_BYTES} bytes (256 KiB); shorten it first",
            signed.len()
        ));
    }
    let again = canonicalize(signed.as_bytes()).map_err(|e| format!("{name}: not signed; internal error: {e}"))?;
    let fingerprint = match sig::verify(&again, NAMESPACE) {
        SigStatus::Valid { fingerprint } => fingerprint,
        SigStatus::Invalid { reason } => {
            return Err(format!("{name}: not signed; {reason}. Steplet needs an Ed25519 key; make one with: ssh-keygen -t ed25519"));
        }
        SigStatus::Unsigned => return Err(format!("{name}: not signed; internal error: the signature block went missing")),
    };

    let tmp = req.file.with_extension("steplet-sign-tmp");
    fs::write(&tmp, &signed).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, req.file).map_err(|e| format!("cannot replace {name}: {e}"))?;
    let mut msg = format!("signed {name}\n  key {fingerprint}\n  expires {}", steplet.expires);
    if replacing {
        msg.push_str("\n  replaced the signature that was there before");
    }
    Ok(msg)
}

/// A new, empty folder under %TEMP%, deleted when dropped.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> std::io::Result<TempDir> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        let path = std::env::temp_dir().join(format!("steplet-sign-{}-{nanos}", std::process::id()));
        fs::create_dir(&path)?;
        Ok(TempDir(path))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
