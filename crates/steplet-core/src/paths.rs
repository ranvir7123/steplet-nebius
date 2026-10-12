//! Paths (decision row 24, G8, G14): the always-blocked list, paths outside the repository, and protected
//! folders. These functions read path text only; no file system is touched. Following links, junctions and
//! short names on a real disk is the runner's job (row 30, R19).

use alloc::string::String;
use alloc::vec::Vec;

/// A place where keys, passwords or sign-ins are kept. `parts` are folder or file names that appear one after
/// another in a path, compared without regard to upper and lower case; a part starting with `*` matches any name
/// ending with the rest (`*.kdbx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Secret {
    pub parts: &'static [&'static str],
    /// What is kept there, in Steplet's own words.
    pub what: &'static str,
}

const fn s(parts: &'static [&'static str], what: &'static str) -> Secret {
    Secret { parts, what }
}

/// Row 24, D2 and E3: places no task may read or write, whatever the file says.
pub const ALWAYS_BLOCKED: &[Secret] = &[
    s(&[".ssh"], "SSH keys"),
    s(&[".gnupg"], "GnuPG keys"),
    s(&[".aws"], "Amazon Web Services keys"),
    s(&[".azure"], "Azure sign-ins"),
    s(&[".kube"], "Kubernetes keys"),
    s(&[".docker"], "Docker sign-ins"),
    s(&[".config", "gcloud"], "Google Cloud sign-ins"),
    s(&[".config", "gh"], "the GitHub command line's sign-in"),
    s(&[".npmrc"], "npm tokens"),
    s(&[".pypirc"], "PyPI tokens"),
    s(&[".netrc"], "saved server passwords"),
    s(&[".git-credentials"], "saved git passwords"),
    s(&[".cargo", "credentials.toml"], "the crates.io token"),
    s(&[".cargo", "credentials"], "the crates.io token"),
    s(&[".mozilla"], "Firefox passwords and cookies"),
    s(&["mozilla", "firefox"], "Firefox passwords and cookies"),
    s(&[".config", "google-chrome"], "Chrome passwords and cookies"),
    s(&["google", "chrome"], "Chrome passwords and cookies"),
    s(&[".config", "chromium"], "Chromium passwords and cookies"),
    s(&[".config", "microsoft-edge"], "Edge passwords and cookies"),
    s(&["microsoft", "edge"], "Edge passwords and cookies"),
    s(&[".password-store"], "a password store"),
    s(&["*.kdbx"], "a KeePass password file"),
    s(&["1password"], "1Password"),
    s(&["bitwarden"], "Bitwarden"),
    s(&[".bitcoin"], "a Bitcoin wallet"),
    s(&[".ethereum"], "an Ethereum wallet"),
    s(&[".electrum"], "an Electrum wallet"),
    s(&["wallet.dat"], "a wallet"),
    s(&[".config", "solana"], "a Solana wallet"),
    s(&[".claude", ".credentials.json"], "Claude Code's sign-in"),
    s(&[".codex", "auth.json"], "Codex's sign-in"),
    s(&[".gemini", "oauth_creds.json"], "Gemini's sign-in"),
    s(&[".config", "github-copilot"], "GitHub Copilot's sign-in"),
    s(&[".local", "share", "steplet"], "Steplet's private folder"),
];

fn parts(path: &str) -> Vec<&str> {
    path.split(['/', '\\']).filter(|p| !p.is_empty() && *p != ".").collect()
}

fn part_matches(pattern: &str, part: &str) -> bool {
    match pattern.strip_prefix('*') {
        Some(end) => {
            let (p, e) = (part.as_bytes(), end.as_bytes());
            p.len() >= e.len() && p[p.len() - e.len()..].eq_ignore_ascii_case(e)
        }
        None => part.eq_ignore_ascii_case(pattern),
    }
}

/// The first entry of the always-blocked list that `path` points into.
pub fn secret_of(path: &str) -> Option<&'static Secret> {
    let ps = parts(path);
    ALWAYS_BLOCKED.iter().find(|sec| {
        ps.windows(sec.parts.len()).any(|w| w.iter().zip(sec.parts).all(|(p, pattern)| part_matches(pattern, p)))
    })
}

/// True for a path that starts outside the repository: at a root (`/`, `\`, `C:\`, `C:/`), at a home folder (`~`,
/// `$HOME`, `${HOME}`, `%USERPROFILE%`, `$env:USERPROFILE`, …), or one that climbs out with `..`.
pub fn is_outside(path: &str) -> bool {
    const HOMES: &[&str] =
        &["$home", "${home}", "%userprofile%", "$env:userprofile", "%homepath%", "%appdata%", "%localappdata%"];
    let b = path.as_bytes();
    let lower = path.to_ascii_lowercase();
    matches!(b.first(), Some(b'/' | b'\\' | b'~'))
        || (b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b.len() == 2 || matches!(b[2], b'/' | b'\\')))
        || HOMES.iter().any(|h| lower.starts_with(h))
        || parts(path).contains(&"..")
}

/// The first always-blocked place that a path outside the repository in `text` points into (G8). Paths are found
/// by splitting the text at spaces, quotes, brackets and similar marks; a path inside the repository never counts.
pub fn secret_in(text: &str) -> Option<&'static Secret> {
    text.split(|c: char| c.is_whitespace() || "\"'`()[]{}<>,;|&=".contains(c))
        .map(|t| t.trim_end_matches(['.', ':', '!', '?']))
        .filter(|t| is_outside(t))
        .find_map(secret_of)
}

/// `path` (absolute, written with `/`) with empty, `.` and `..` parts resolved, as text only. `None` when the
/// path is not absolute or has a control character.
pub fn normalize(path: &str) -> Option<String> {
    if !path.starts_with('/') || path.chars().any(char::is_control) {
        return None;
    }
    let mut kept: Vec<&str> = Vec::new();
    for p in path.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                kept.pop();
            }
            p => kept.push(p),
        }
    }
    let mut out = String::new();
    for p in &kept {
        out.push('/');
        out.push_str(p);
    }
    if out.is_empty() {
        out.push('/');
    }
    Some(out)
}

/// The part of `path` below the folder `root` (both normalised); `Some("")` for the folder itself.
pub fn below<'a>(root: &str, path: &'a str) -> Option<&'a str> {
    if path == root {
        return Some("");
    }
    path.strip_prefix(root)?.strip_prefix('/')
}

/// True when the relative path `rel` is the folder `folder` or inside it. The folder `.` holds everything.
pub fn within(folder: &str, rel: &str) -> bool {
    folder == "." || rel == folder || rel.strip_prefix(folder).is_some_and(|r| r.starts_with('/'))
}

/// G14: folders and files no task may change, even when the file allows writing everywhere: `.git` (D6) and the
/// agents' own settings (D2, D3).
pub const PROTECTED: &[&str] = &[".git", ".claude", ".codex", ".gemini", ".cursor", ".mcp.json"];

/// True when any part of the relative path `rel` is a protected name, whatever its letter case (the working copy may
/// sit on a file system that ignores case).
pub fn is_protected(rel: &str) -> bool {
    rel.split('/').any(|p| PROTECTED.iter().any(|q| p.eq_ignore_ascii_case(q)))
}
