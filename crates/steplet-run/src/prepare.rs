//! The working copy (decision D6): a fresh hardened clone, the declared commit checked out inside the fence with no
//! agent, the agent settings files removed, and the run's private folders and settings files written. It all runs as
//! root inside Steplet's own Linux, from the fixed script `prepare.sh`; `discard` removes the run when it ends.

use std::time::Duration;

use steplet_core::hosts;

use crate::report::changed_files;
use crate::settings::{mcp_json, settings_json};
use crate::types::{BadValue, GateLink, RunPaths, RunRequest};
use crate::wsl;

const PREPARE: &str = include_str!("prepare.sh");
const DISCARD: &str = include_str!("discard.sh");
const AFTER: &str = include_str!("after.sh");
/// The longest Steplet waits for `prepare.sh`: its clone may take 10 minutes and its checkout 5 (their own limits).
const PREPARE_LIMIT: Duration = Duration::from_secs(20 * 60);
/// The longest Steplet waits for `discard.sh`, and for each look of `after.sh`.
const DISCARD_LIMIT: Duration = Duration::from_secs(2 * 60);

/// P3: milestone 1 clones public repositories only, so only from these public code hosts (the user's choice, 14 Oct
/// 2026). Any other name could be a home or company server that Steplet's own git would reach before any fence.
pub const REPOSITORY_HOSTS: &[&str] = &["github.com", "gitlab.com", "codeberg.org", "bitbucket.org"];

/// Whether Steplet would fetch the repository at `url`: a plain `https://` address on one of `REPOSITORY_HOSTS`, as
/// `prepare` checks it. The app asks this before offering Run.
pub fn fetchable(url: &str) -> bool {
    is_plain_https(url) && hosts::https_host(url).is_some_and(|h| REPOSITORY_HOSTS.contains(&h.as_str()))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrepareError {
    /// The request holds a value Steplet will not hand to its Linux.
    Refused(BadValue),
    /// `wsl.exe` could not be started.
    Wsl(String),
    /// A step inside Steplet's Linux failed. `step` names it; `detail` is the end of what it printed about it.
    Step { step: &'static str, detail: String },
}

/// In Steplet's words, as one sentence, for a report's faults (Task 13): never Rust's debugging form.
impl std::fmt::Display for PrepareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let said = match self {
            PrepareError::Refused(what) => format!("Steplet will not hand this {} to its Linux", what.0),
            PrepareError::Wsl(why) => format!("WSL could not be started: {}", why.trim()),
            PrepareError::Step { step, detail } => match detail.lines().rev().map(str::trim).find(|l| !l.is_empty()) {
                Some(last) => format!("it stopped while trying to {step}, and printed: {last}"),
                None => format!("it stopped while trying to {step}"),
            },
        };
        let said = said.trim_end_matches('.');
        write!(f, "{said}.")
    }
}

/// What preparing a run did, for the prepare screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prepared {
    /// The commit the working copy is at, as the script checked it.
    pub commit: String,
    /// The agent settings files and folders removed from the working copy (D3), in the script's fixed order.
    pub removed: Vec<String>,
}

/// The agent settings files and folders `prepare.sh` knows, in the order it removes them.
const AGENT_FILES: &[&str] = &[".claude", ".codex", ".cursor", ".gemini", ".mcp.json"];

/// Prepares run `paths` for `req`. The repository address and the commit go into a script run as root, so they are
/// checked here first: the address must be a plain `https://` address on one of `REPOSITORY_HOSTS`, and the commit
/// 40 or 64 lower-case hexadecimal digits.
pub fn prepare(req: &RunRequest, paths: &RunPaths, gate: &GateLink) -> Result<Prepared, PrepareError> {
    if !is_plain_https(&req.repo_url) {
        return Err(PrepareError::Refused(BadValue("repository address")));
    }
    if !fetchable(&req.repo_url) {
        return Err(PrepareError::Refused(BadValue("repository host")));
    }
    if !is_commit(&req.commit) {
        return Err(PrepareError::Refused(BadValue("commit")));
    }
    let settings = base64(settings_json(&req.steplet, paths, gate, req.allow_model_switch).as_bytes());
    let mcp = base64(mcp_json(gate).as_bytes());
    let args = ["prepare", paths.run_id(), &req.repo_url, &req.commit, &settings, &mcp];
    let out = as_root(paths, PREPARE, &args, PREPARE_LIMIT)?;
    let ready = format!("steplet-prepare: ready at {}", req.commit);
    if !out.stdout.lines().any(|line| line == ready) {
        let detail = "the script ended without saying the copy was ready at the declared commit".into();
        return Err(PrepareError::Step { step: "confirm the working copy is ready", detail });
    }
    Ok(Prepared { commit: req.commit.clone(), removed: removed_files(&out.stdout) })
}

/// The agent settings files `prepare.sh` says it removed (`removed: <name>` lines). Only names it knows count, each
/// once and in its order, so a line from anything else it ran cannot add a name of its own.
pub fn removed_files(stdout: &str) -> Vec<String> {
    let said: Vec<&str> = stdout.lines().filter_map(|line| line.strip_prefix("removed: ")).collect();
    AGENT_FILES.iter().filter(|f| said.contains(f)).map(|f| (*f).to_owned()).collect()
}

/// Ends whatever the run left running and deletes its working copy and private folders.
pub fn discard(paths: &RunPaths) -> Result<(), PrepareError> {
    as_root(paths, DISCARD, &[paths.run_id()], DISCARD_LIMIT).map(drop)
}

/// What the run changed in its working copy, from Steplet's own `git status` after the run and before `discard`
/// (Task 10): one entry per file, its two status letters and its path. Whatever the run left running is ended first.
pub fn changes(paths: &RunPaths) -> Result<Vec<String>, PrepareError> {
    let out = as_root(paths, AFTER, &[paths.run_id(), "status"], DISCARD_LIMIT)?;
    Ok(changed_files(&String::from_utf8_lossy(&out.stdout_bytes)))
}

/// The run's Claude Code config file (`.claude.json`, at most 4 MiB), for the trust check (D3), read after the run
/// and before `discard`.
pub fn run_config(paths: &RunPaths) -> Result<String, PrepareError> {
    Ok(as_root(paths, AFTER, &[paths.run_id(), "config"], DISCARD_LIMIT)?.stdout)
}

fn as_root(paths: &RunPaths, script: &str, args: &[&str], limit: Duration) -> Result<wsl::Output, PrepareError> {
    let argv = [&["/bin/bash", "-s", "--"][..], args].concat();
    let out = wsl::run_within(paths.distro_id(), "root", &argv, script.as_bytes(), limit)
        .map_err(|e| PrepareError::Wsl(e.to_string()))?;
    let step = match out.status {
        Some(0) => return Ok(out),
        Some(10) => "clear the last run",
        Some(11) => "clone the repository",
        Some(12) => "find the declared commit",
        Some(13) => "check out the commit inside the fence",
        Some(14) => "remove the agent settings files",
        Some(15) => "write the run's settings",
        Some(20) => "remove the run",
        Some(30) => "end what the run left running",
        Some(31) => "list the files the run changed",
        Some(32) => "read the run's config",
        _ => "reach Steplet's Linux",
    };
    // The script writes to its error output; `wsl.exe` writes its own failures to standard output.
    let said = if out.stderr.trim().is_empty() { &out.stdout } else { &out.stderr };
    let lines: Vec<&str> = said.lines().collect();
    Err(PrepareError::Step { step, detail: lines[lines.len().saturating_sub(15)..].join("\n") })
}

fn is_plain_https(url: &str) -> bool {
    hosts::https_host(url).is_some()
        && url
            .strip_prefix("https://")
            .is_some_and(|rest| rest.bytes().all(|b| b.is_ascii_alphanumeric() || b"._~/-".contains(&b)))
}

fn is_commit(s: &str) -> bool {
    (s.len() == 40 || s.len() == 64) && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Standard base64 with `=` padding (RFC 4648), so a file's text crosses the Windows command line unchanged.
fn base64(bytes: &[u8]) -> String {
    const LETTERS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, &b)| n | (u32::from(b) << (16 - 8 * i)));
        for i in 0..4 {
            out.push(if i <= chunk.len() { char::from(LETTERS[((n >> (18 - 6 * i)) & 63) as usize]) } else { '=' });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_gives_the_rfc_4648_test_vectors() {
        for (plain, coded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(super::base64(plain.as_bytes()), coded, "{plain:?}");
        }
        assert_eq!(super::base64(&[0xfb, 0xff, 0xfe]), "+//+", "the last two letters of the alphabet");
    }
}
