//! The run chooser's lines (phase 4 Task 5, R6, R20): each line Steplet may promise for a file names the tests that
//! back it, and is worded no wider than those tests show.

use steplet_core::canon::canonicalize;
use steplet_core::header::{Steplet, parse};
use steplet_run::{Promise, promises};

fn steplet(allow: &str) -> Steplet {
    let f = format!("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n{allow}---\n");
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap()
}

fn texts(ps: &[Promise]) -> Vec<&str> {
    ps.iter().map(|p| p.text).collect()
}

/// Every test a line names is a test function in this crate's tests: renaming or deleting one breaks this test, so a
/// line cannot outlive the evidence for it.
#[test]
fn every_promise_names_tests_that_exist() {
    let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");
    let mut sources = String::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "rs") {
            sources.push_str(&std::fs::read_to_string(&path).unwrap());
        }
    }
    for s in [steplet(""), steplet("allow:\n  domains:\n    - example.org\n")] {
        for p in promises(&s) {
            assert!(!p.tests.is_empty(), "{:?} names no test", p.text);
            for test in p.tests {
                assert!(
                    sources.contains(&format!("fn {test}(")),
                    "{:?} names {test}, which is not a test here",
                    p.text
                );
            }
        }
    }
}

#[test]
fn the_lines_follow_the_files_sites() {
    let none = promises(&steplet(""));
    assert_eq!(
        texts(&none),
        [
            "Steplet checks the agent's commands that change something, its file writes and its web fetches before \
             they happen. If Steplet's checker fails, the action is refused.",
            "The agent cannot fetch any web address, unless you say yes to that exact address.",
            "The agent and the programs this task runs can change files only inside the working copy.",
            "Your own files are not inside Steplet's Linux, where this task runs.",
            "Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.",
            "Programs this task runs cannot start Windows programs.",
        ]
    );
    assert!(none.iter().all(|p| p.items.is_empty()));

    let one = promises(&steplet("allow:\n  domains:\n    - example.org\n"));
    assert_eq!(
        (one[1].text, one[1].items.clone()),
        (
            "The agent can fetch only these web addresses, and another one only if you say yes to that exact address:",
            vec!["example.org".to_owned()]
        )
    );
    assert_eq!(
        (one[4].text, one[4].items.clone()),
        (
            "Programs this task runs can connect only to these sites: not to the rest of the internet, or to other \
             programs in Steplet's Linux.",
            vec!["example.org".to_owned()]
        )
    );
    assert!(one[4].tests.contains(&"a_declared_site_is_the_only_one_a_program_can_reach"));
    assert!(!none[4].tests.contains(&"a_declared_site_is_the_only_one_a_program_can_reach"));
}

/// Milestone plan section 7 and the phase 4 plan's 3.7 point 3: "on this computer" may be said only after a test with
/// a program listening on the Windows side, which has not been run.
#[test]
fn no_line_says_on_this_computer() {
    for s in [steplet(""), steplet("allow:\n  domains:\n    - example.org\n")] {
        for p in promises(&s) {
            assert!(!p.text.to_lowercase().contains("on this computer"), "{:?}", p.text);
        }
    }
}
