//! The corpus (milestone plan, phase 2; G3, G4): every file's report at a fixed time must equal its expected JSON,
//! and every rule has a hit and a near-miss. After reading a change, write the expected files again with the
//! environment variable STEPLET_BLESS=1 (corpus/README.md).

use std::fs;
use std::path::{Path, PathBuf};

use steplet_core::report::{Report, check};
use steplet_core::rules::{Outcome, RULES};

/// 2026-10-11 12:00 UTC.
const NOW: i64 = 1_791_720_000;

fn corpus() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus"))
}

fn files(dir: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(corpus().join(dir))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "steplet"))
        .collect();
    v.sort();
    v
}

fn report(path: &Path) -> Report {
    check(&fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())), NOW)
}

#[test]
fn every_file_gives_its_expected_json() {
    let bless = std::env::var("STEPLET_BLESS").as_deref() == Ok("1");
    let all: Vec<PathBuf> = files("ordinary").into_iter().chain(files("hostile")).collect();
    assert_eq!(all.len(), 134);
    let mut different = Vec::new();
    for p in &all {
        let got = report(p).to_json();
        let expected = p.with_extension("json");
        if bless {
            fs::write(&expected, &got).unwrap();
        } else if fs::read_to_string(&expected).ok().as_deref() != Some(got.as_str()) {
            different.push(p.display().to_string());
        }
    }
    assert!(different.is_empty(), "different from the expected JSON: {different:#?}");
    for dir in ["ordinary", "hostile"] {
        for entry in fs::read_dir(corpus().join(dir)).unwrap() {
            let p = entry.unwrap().path();
            if p.extension().is_some_and(|e| e == "json") {
                assert!(p.with_extension("steplet").exists(), "an expected answer with no file: {}", p.display());
            }
        }
    }
}

#[test]
fn every_rule_has_a_hit_and_a_near_miss() {
    for rule in RULES {
        let hit = corpus().join("hostile").join(format!("{}.hit.steplet", rule.id));
        let r = report(&hit).rating;
        assert!(r.findings.iter().any(|f| f.rule.id == rule.id), "{} does not trigger {}", hit.display(), rule.id);
        assert_eq!(r.outcome, Outcome::of(rule.severity), "{}: something worse than {} fired too", hit.display(), rule.id);
        let near = corpus().join("hostile").join(format!("{}.near.steplet", rule.id));
        let r = report(&near).rating;
        assert!(!r.findings.iter().any(|f| f.rule.id == rule.id), "{} triggers {}", near.display(), rule.id);
    }
}

#[test]
fn every_hostile_file_belongs_to_a_rule() {
    for p in files("hostile") {
        let name = p.file_name().unwrap().to_str().unwrap();
        let id = name.strip_suffix(".hit.steplet").or_else(|| name.strip_suffix(".near.steplet"));
        assert!(id.is_some_and(|id| RULES.iter().any(|r| r.id == id)), "{name}: not <rule id>.hit or <rule id>.near");
    }
}

#[test]
fn counts_of_the_ordinary_files() {
    let mut counts = [0usize; 4];
    for p in files("ordinary") {
        let r = report(&p).rating;
        counts[r.outcome as usize] += 1;
        assert_ne!(r.outcome, Outcome::Blocked, "{}", p.display());
        if p.to_string_lossy().ends_with("-french.steplet") {
            assert!(r.findings.iter().all(|f| f.rule.id != "hide.invisible-in-text"), "French spacing: {}", p.display());
        }
    }
    println!("ordinary files: {} green, {} amber, {} red, {} blocked", counts[0], counts[1], counts[2], counts[3]);
    assert_eq!(counts.iter().sum::<usize>(), 62);
    let outcome = |name: &str| report(&corpus().join("ordinary").join(name)).rating.outcome;
    assert_eq!(outcome("uc5-lookalike.steplet"), outcome("pytest-minimal.steplet"), "use case 5 looks like use case 1");
}

/// Prints one line per file: its name, outcome and rule ids, for reading the corpus by eye (`-- --nocapture`).
#[test]
fn list_every_file_with_its_findings() {
    for p in files("ordinary").into_iter().chain(files("hostile")) {
        let r = report(&p).rating;
        let ids: Vec<&str> = r.findings.iter().map(|f| f.rule.id).collect();
        println!("{}: {}: {}", p.file_name().unwrap().to_string_lossy(), r.outcome.name(), ids.join(" "));
    }
}
