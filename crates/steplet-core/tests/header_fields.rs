use steplet_core::canon::canonicalize;
use steplet_core::date::Date;
use steplet_core::header::{ALLOW_KEYS, ParseError, ParseErrorKind as K, Steplet, TOP_KEYS, parse};

const GOOD: &str = "---
steplet: 1
title: Run the tests
sender: Ana Lee
expires: 2026-10-25
repository:
  url: https://github.com/octocat/Hello-World.git
  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
steps:
  - Build it.
  - Test it.
allow:
  commands:
    - cargo test
  domains:
    - crates.io
  read:
    - .
  write:
    - target/debug
never:
  - Do not push.
---
# Body
";

fn read(f: &str) -> Result<Steplet, ParseError> {
    parse(&canonicalize(f.as_bytes()).unwrap())
}
fn kind(from: &str, to: &str) -> K {
    let f = GOOD.replacen(from, to, 1);
    assert_ne!(f, GOOD, "the replacement did not apply: {from:?}");
    read(&f).unwrap_err().kind
}
fn texts(v: &[steplet_core::header::Scalar]) -> Vec<&str> {
    v.iter().map(|s| s.text.as_str()).collect()
}

#[test]
fn reads_every_field_with_its_line() {
    let s = read(GOOD).unwrap();
    assert_eq!((s.title.text.as_str(), s.title.line), ("Run the tests", 3));
    assert_eq!(s.sender.text, "Ana Lee");
    assert_eq!((s.expires, s.expires_line), (Date::parse("2026-10-25").unwrap(), 5));
    let r = s.repository.unwrap();
    assert_eq!(r.url.text, "https://github.com/octocat/Hello-World.git");
    assert_eq!((r.commit.text.as_str(), r.commit.line), ("7fd1a60b01f91b314f59955a4e4d4e80d8edf11d", 8));
    assert_eq!(texts(&s.steps), ["Build it.", "Test it."]);
    assert_eq!(s.steps[1].line, 11);
    assert_eq!(texts(&s.allow.commands), ["cargo test"]);
    assert_eq!(texts(&s.allow.domains), ["crates.io"]);
    assert_eq!(texts(&s.allow.read), ["."]);
    assert_eq!(texts(&s.allow.write), ["target/debug"]);
    assert_eq!(texts(&s.never), ["Do not push."]);
    assert_eq!(s.body, "# Body\n");
}

#[test]
fn optional_parts_may_be_left_out() {
    let s = read("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n  - a\n---\n").unwrap();
    assert!(s.repository.is_none());
    assert!(s.allow.commands.is_empty() && s.allow.domains.is_empty() && s.allow.read.is_empty() && s.allow.write.is_empty());
    assert!(s.never.is_empty());
    assert_eq!(s.body, "");
}

#[test]
fn the_version_comes_first_and_is_1() {
    assert_eq!(kind("steplet: 1\n", "steplet: 2\n"), K::UnsupportedVersion { found: "2".into() });
    assert_eq!(kind("steplet: 1\ntitle: Run the tests\n", "title: Run the tests\nsteplet: 1\n"), K::FirstKeyNotSteplet);
    assert_eq!(read("---\n---\n").unwrap_err(), ParseError { line: 1, kind: K::MissingKey { key: "steplet" } });
    assert_eq!(kind("steplet: 1\n", "steplet: 2\ncolour: green\n"), K::UnsupportedVersion { found: "2".into() });
}

#[test]
fn unknown_and_missing_fields() {
    assert_eq!(kind("steps:\n", "colour: green\nsteps:\n"), K::UnknownKey { key: "colour".into(), allowed: TOP_KEYS });
    assert_eq!(kind("  read:\n", "  network:\n    - x\n  read:\n"), K::UnknownKey { key: "network".into(), allowed: ALLOW_KEYS });
    assert_eq!(kind("sender: Ana Lee\n", ""), K::MissingKey { key: "sender" });
    assert_eq!(kind("expires: 2026-10-25\n", ""), K::MissingKey { key: "expires" });
    assert_eq!(kind("steps:\n  - Build it.\n  - Test it.\n", ""), K::MissingKey { key: "steps" });
    assert_eq!(kind("  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d\n", ""), K::MissingKey { key: "commit" });
}

#[test]
fn wrong_shapes() {
    assert_eq!(kind("title: Run the tests\n", "title:\n  - a\n"), K::ExpectedText { key: "title".into() });
    assert_eq!(kind("title: Run the tests\n", "title: \"\"\n"), K::EmptyValue { key: "title".into() });
    assert_eq!(kind("steps:\n  - Build it.\n  - Test it.\n", "steps: Build it.\n"), K::ExpectedList { key: "steps".into() });
    assert_eq!(kind("  - Build it.\n", "  - ''\n"), K::EmptyListItem);
    assert_eq!(
        kind("repository:\n  url: https://github.com/octocat/Hello-World.git\n  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d\n", "repository: x\n"),
        K::ExpectedMap { key: "repository".into() }
    );
}

#[test]
fn typed_values() {
    assert_eq!(kind("2026-10-25", "2026-02-30"), K::BadDate);
    assert_eq!(kind("2026-10-25", "25 Oct 2026"), K::BadDate);
    for url in ["http://github.com/x", "https://user@github.com/x", "https://GitHub.com/x", "https://127.0.0.1/x",
                "https://github.com:443/x", "https://github.com/x?y=1", "https://localhost/x", "https://github.com/a b"] {
        assert_eq!(kind("https://github.com/octocat/Hello-World.git", url), K::BadUrl, "{url}");
    }
    for commit in ["7fd1a60", "7FD1A60B01F91B314F59955A4E4D4E80D8EDF11D", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11g"] {
        assert_eq!(kind("7fd1a60b01f91b314f59955a4e4d4e80d8edf11d", commit), K::BadCommit, "{commit}");
    }
    for domain in ["Crates.io", "localhost", "\"*.crates.io\"", "crates.io:443", "https://crates.io", "\"-a.io\"", "1.2.3.4"] {
        assert_eq!(kind("    - crates.io\n", &format!("    - {domain}\n")), K::BadDomain, "{domain}");
    }
    for folder in ["../x", "/etc", "a\\b", "a//b", "target/", "./target", "a/../b", "my folder"] {
        // Single quotes, so the backslash in `a\b` is not read as an escape.
        assert_eq!(kind("    - target/debug\n", &format!("    - '{folder}'\n")), K::BadFolder, "{folder}");
    }
}

#[test]
fn every_field_error_has_words_and_help() {
    let kinds = [
        kind("steps:\n", "colour: green\nsteps:\n"), kind("sender: Ana Lee\n", ""),
        kind("steplet: 1\ntitle: Run the tests\n", "title: Run the tests\nsteplet: 1\n"),
        kind("steplet: 1\n", "steplet: 2\n"), kind("title: Run the tests\n", "title:\n  - a\n"),
        kind("steps:\n  - Build it.\n  - Test it.\n", "steps: Build it.\n"), kind("2026-10-25", "2026-02-30"),
        kind("https://github.com/octocat/Hello-World.git", "http://x.org"), kind("7fd1a60b01f91b314f59955a4e4d4e80d8edf11d", "7fd"),
        kind("    - crates.io\n", "    - Crates.io\n"), kind("    - target/debug\n", "    - \"..\"\n"),
    ];
    for k in kinds {
        let e = ParseError { line: 4, kind: k };
        assert!(e.to_string().starts_with("line 4: "), "{e:?}");
        assert!(!e.help().allowed.is_empty() && !e.help().example.is_empty(), "{e:?}");
    }
}

#[test]
fn the_example_file_reads() {
    let s = read(include_str!("../../../examples/run-tests.steplet")).unwrap();
    assert_eq!(s.steps.len(), 3);
    assert!(s.repository.is_some());
}
