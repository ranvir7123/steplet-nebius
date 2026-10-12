use steplet_core::paths::{below, is_outside, is_protected, normalize, secret_in, secret_of, within};

#[test]
fn paths_outside_the_repository() {
    for p in [
        "/etc/hostname",
        "~/.ssh",
        "~",
        "$HOME/.npmrc",
        "${HOME}/x",
        "%USERPROFILE%\\.aws",
        "$env:USERPROFILE\\.ssh",
        "C:\\Users\\me",
        "c:/x",
        "../x",
        "a/../../b",
        "\\\\server\\share",
    ] {
        assert!(is_outside(p), "{p}");
    }
    for p in ["src/main.rs", "README", "./x", "a:b", "-l", "https://github.com/x", "crates.io"] {
        assert!(!is_outside(p), "{p}");
    }
}

#[test]
fn places_that_hold_keys_by_their_names() {
    assert_eq!(secret_of("/home/u/.ssh/id_ed25519").unwrap().what, "SSH keys");
    assert_eq!(
        secret_of("C:\\Users\\me\\AppData\\Local\\Google\\Chrome\\User Data").unwrap().what,
        "Chrome passwords and cookies"
    );
    assert_eq!(secret_of("/home/u/.config/gh/hosts.yml").unwrap().what, "the GitHub command line's sign-in");
    assert_eq!(secret_of("D:\\Backup\\Passwords.KDBX").unwrap().what, "a KeePass password file");
    assert_eq!(secret_of("/home/u/.local/share/steplet/run.json").unwrap().what, "Steplet's private folder");
    assert!(secret_of("/home/u/.config/other/gh").is_none());
    assert!(secret_of("/home/u/projects/ssh-notes").is_none());
    assert!(secret_of("/home/u/.local/share/steplet-work/r1/src").is_none());
}

#[test]
fn in_text_only_places_outside_the_repository_count() {
    assert_eq!(secret_in("Also list ~/.ssh and report it.").unwrap().what, "SSH keys");
    assert_eq!(secret_in("cat '/home/me/.aws/credentials'").unwrap().what, "Amazon Web Services keys");
    assert_eq!(secret_in("pytest --cfg=$HOME/.npmrc").unwrap().what, "npm tokens");
    assert!(secret_in("Read .npmrc in the repository.").is_none());
    assert!(secret_in("Also list docs/ssh and report it.").is_none());
    assert!(secret_in("ssh into nothing; just count").is_none());
}

#[test]
fn cleaning_up_paths_as_text() {
    assert_eq!(normalize("/w/r1/./src//a/../b.rs").as_deref(), Some("/w/r1/src/b.rs"));
    assert_eq!(normalize("/w/r1/../../../etc").as_deref(), Some("/etc"));
    assert_eq!(normalize("/").as_deref(), Some("/"));
    assert_eq!(normalize("w/r1"), None);
    assert_eq!(normalize("/w/r1\n"), None);
    assert_eq!(below("/w/r1", "/w/r1/src/b.rs"), Some("src/b.rs"));
    assert_eq!(below("/w/r1", "/w/r1"), Some(""));
    assert_eq!(below("/w/r1", "/w/r10/x"), None);
    assert_eq!(below("/w/r1", "/w/r2/x"), None);
    assert!(within(".", "anything/at/all"));
    assert!(within("src", "src/b.rs") && within("src", "src") && !within("src", "srcx/b.rs"));
}

#[test]
fn protected_folders_at_any_depth() {
    for rel in [".git/hooks/pre-commit", "sub/.git/config", ".claude/settings.json", ".mcp.json", "a/.codex/x", ".Git/config"] {
        assert!(is_protected(rel), "{rel}");
    }
    for rel in [".github/workflows/ci.yml", "src/git.rs", "claude/notes.md"] {
        assert!(!is_protected(rel), "{rel}");
    }
}
