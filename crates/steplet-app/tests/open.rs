//! Opening a file (Task 10): at most 256 KiB and one byte are read, opening changes nothing on disk, and what is
//! true on this computer and not in the file (no repository, a host Steplet does not fetch from, a damaged list of
//! known senders) is said before Run.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use steplet_app::open::{MAX_READ, preview_file, read_file};
use steplet_app::pins::{self, Pins};
use steplet_app::views::{RunStatus, not_fetched_from_here};
use steplet_core::identity::Pin;

/// The corpus's own time of viewing (crates/steplet-core/tests/corpus.rs).
const NOW: i64 = 1_791_720_000;

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("open-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn example() -> Vec<u8> {
    std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/run-tests.steplet")).unwrap()
}

fn written(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, bytes).unwrap();
    p
}

#[test]
fn a_file_over_256_kib_is_not_read_whole() {
    let dir = scratch("large");
    let mut big = example();
    big.resize(10 << 20, b'a');
    let p = written(&dir, "big.steplet", &big);
    let raw = read_file(&p).unwrap();
    assert_eq!(MAX_READ, 262_145);
    assert_eq!(raw.len() as u64, MAX_READ, "at most 256 KiB and one byte are read");
    let view = preview_file(&raw, NOW, &Pins::Known(Vec::new()));
    assert_eq!(view.run_status, RunStatus::Blocked);
    assert_eq!(view.findings[0].rule, "read.too-large");
}

#[test]
fn opening_changes_nothing_on_disk() {
    let dir = scratch("nothing");
    let pins_path = written(&dir, "senders.json", br#"{"version": 1, "senders": []}"#);
    let file = written(&dir, "run-tests.steplet", &example());
    let before = |p: &Path| (std::fs::read(p).unwrap(), std::fs::metadata(p).unwrap().modified().unwrap());
    let listing = || {
        let mut names: Vec<String> =
            std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        names.sort();
        names
    };
    let (pins_then, file_then, files_then) = (before(&pins_path), before(&file), listing());
    let start = SystemTime::now();
    let view = preview_file(&read_file(&file).unwrap(), NOW, &pins::load(&pins_path));
    assert!(view.file.is_some());
    assert_eq!(before(&pins_path), pins_then, "the pins file's bytes and time");
    assert_eq!(before(&file), file_then, "the file's bytes and time");
    assert_eq!(listing(), files_then, "no file was made");
    assert!(start.elapsed().unwrap().as_secs() < 5);
}

#[test]
fn a_host_steplet_does_not_fetch_from_is_said_before_run() {
    let file = String::from_utf8(example())
        .unwrap()
        .replace("https://github.com/octocat/Hello-World.git", "https://gitlab.example.com/octocat/Hello-World.git");
    let view = preview_file(file.as_bytes(), NOW, &Pins::Known(Vec::new()));
    let RunStatus::NotRunnable { reason } = &view.run_status else { panic!("{:?}", view.run_status) };
    assert_eq!(*reason, not_fetched_from_here());
    assert!(reason.contains("github.com, gitlab.com, codeberg.org and bitbucket.org"), "H9's hosts: {reason}");
    assert!(!reason.contains("gitlab.example.com"), "the file's own host is file text, shown in its panel only");
    assert!(!reason.contains("  "), "one space between words: {reason:?}");
}

#[test]
fn a_file_with_no_repository_cannot_run_here() {
    let raw =
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/hostile/note.no-repository.hit.steplet"))
            .unwrap();
    let view = preview_file(&raw, NOW, &Pins::Known(Vec::new()));
    assert_eq!(view.run_status, RunStatus::NotRunnable { reason: steplet_app::views::NO_REPOSITORY.into() });
}

/// A key's fingerprint that is not the corpus's signing key.
const OTHER_KEY: &str = "SHA256:Q2hhbmdlZEtleUNoYW5nZWRLZXlDaGFuZ2VkS2V5Q2g";

#[test]
fn a_changed_key_leads_the_identity_line() {
    let signed =
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/hostile/sig.invalid.near.steplet")).unwrap();
    let report = steplet_core::report::check(&signed, NOW);
    let Some(steplet_core::sig::SigStatus::Valid { fingerprint }) = &report.signature else {
        panic!("the corpus file is signed: {:?}", report.signature)
    };
    let sender_of = |raw: &[u8]| steplet_core::report::check(raw, NOW).steplet.unwrap().sender.text;
    let pinned = |sender: String, key: &str| Pins::Known(vec![Pin { sender, key: key.into(), runs: 1 }]);
    let rules = |v: &steplet_app::views::PreviewView| v.here.iter().map(|f| f.rule.clone()).collect::<Vec<_>>();

    let changed = preview_file(&signed, NOW, &pinned(sender_of(&signed), OTHER_KEY));
    assert!(changed.key_changed, "the sender name is pinned to another key");
    assert_eq!(rules(&changed), ["id.key-changed"]);
    assert_eq!(changed.run_status, RunStatus::Red, "row 22");

    let same = preview_file(&signed, NOW, &pinned(sender_of(&signed), fingerprint));
    assert!(!same.key_changed, "the same key");
    assert!(rules(&same).is_empty());

    let unsigned = preview_file(&example(), NOW, &pinned(sender_of(&example()), OTHER_KEY));
    assert!(unsigned.key_changed, "unsigned under a pinned sender name (S6)");
    assert_eq!(rules(&unsigned), ["id.key-changed"]);
}

#[test]
fn a_file_that_cannot_run_here_keeps_this_computers_colour() {
    // A file with no repository, from a sender name this computer knows with a signing key, now unsigned (S6).
    let raw =
        std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/hostile/note.no-repository.hit.steplet"))
            .unwrap();
    let sender = steplet_core::report::check(&raw, NOW).steplet.unwrap().sender.text;
    let view = preview_file(&raw, NOW, &Pins::Known(vec![Pin { sender, key: OTHER_KEY.into(), runs: 1 }]));
    assert_eq!(view.run_status, RunStatus::NotRunnable { reason: steplet_app::views::NO_REPOSITORY.into() });
    assert!(view.key_changed);
    assert_eq!(view.outcome, "green", "the file's own outcome");
    assert_eq!(view.colour, "red", "the banner: the worst of the file's findings and this computer's (row 22)");
    let plain = preview_file(&raw, NOW, &Pins::Known(Vec::new()));
    assert_eq!(plain.colour, plain.outcome, "nothing about this computer: the file's own colour");
}

#[test]
fn an_unreadable_pins_file_is_kept_and_said() {
    let dir = scratch("junk");
    let junk = b"this is not the senders list \x00\xff";
    let pins_path = written(&dir, "senders.json", junk);
    let pins = pins::load(&pins_path);
    assert!(matches!(pins, Pins::Unreadable(_)), "junk is not a pins list");
    let view = preview_file(&example(), NOW, &pins);
    let here: Vec<(&str, &str, &str)> =
        view.here.iter().map(|f| (f.rule.as_str(), f.severity.as_str(), f.title.as_str())).collect();
    assert_eq!(
        here,
        [(
            "here.pins-unreadable",
            "amber",
            "Steplet could not read the senders it remembers, so it cannot warn about a changed key"
        )]
    );
    assert_eq!(view.run_status, RunStatus::Amber, "the warning raises a green file to amber");
    assert!(pins::record(&pins_path, "Ana Lee", "SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk").is_err());
    assert_eq!(std::fs::read(&pins_path).unwrap(), junk, "the file is not overwritten");
}
