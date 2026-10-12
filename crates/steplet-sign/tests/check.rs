use steplet_core::date::Date;
use steplet_core::rules::{BLOCK_ADMIN, CAP_RUNS_CODE};
use steplet_sign::check::check;

fn today() -> Date {
    Date::parse("2026-10-11").unwrap()
}

const GOOD: &str = include_str!("../../../examples/run-tests.steplet");

#[test]
fn a_good_unsigned_file() {
    let r = check("run-tests.steplet", GOOD.as_bytes(), today());
    assert!(r.ok, "{}", r.text);
    assert_eq!(
        r.text,
        "run-tests.steplet: ok
  3 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 2 never lines
  repository: https://github.com/octocat/Hello-World.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
  expires: 2027-09-30 (in 354 days)
  signature: not signed
  rating: green
"
    );
}

#[test]
fn a_tab_names_its_line_and_shows_an_example() {
    let r = check("t.steplet", GOOD.replacen("  - Report", "\t- Report", 1).as_bytes(), today());
    assert!(!r.ok);
    assert_eq!(
        r.text,
        "t.steplet: 1 problem
  signature: not signed
  problem: line 12: a tab character
    allowed: indent with spaces, two per level; no tab characters anywhere in the header
    for example:
      allow:
        commands:
          - cargo test
"
    );
}

#[test]
fn expiry_in_the_past_or_too_far_ahead() {
    let r = check("t", GOOD.as_bytes(), Date::parse("2027-10-01").unwrap());
    assert!(!r.ok && r.text.contains("problem: line 5: this file expired on 2027-09-30"), "{}", r.text);
    assert!(r.text.contains("expires: 2027-10-15"), "the example shows today + 14 days:\n{}", r.text);
    let r = check("t", GOOD.as_bytes(), Date::parse("2026-09-29").unwrap());
    assert!(!r.ok && r.text.contains("more than 365 days"), "{}", r.text);
    assert!(check("t", GOOD.as_bytes(), Date::parse("2027-09-30").unwrap()).text.contains("(today)"));
}

#[test]
fn a_changed_signed_file_is_a_problem() {
    let signed = include_str!("../../steplet-core/tests/vectors/valid.steplet");
    let r = check("v", signed.as_bytes(), today());
    assert!(r.ok && r.text.contains("signature: valid, key SHA256:"), "{}", r.text);
    let r = check("v", signed.replace("Report the count.", "Report it.").as_bytes(), today());
    assert!(!r.ok && r.text.contains("problem: signature: the file was changed after it was signed"), "{}", r.text);
}

#[test]
fn limits_and_clean_up_errors() {
    let many: String = (0..31).map(|i| format!("  - step {i}\n")).collect();
    let f = GOOD.replacen(
        "  - Check that the README file is there.\n  - Count the lines in the README file.\n  - Report the count.\n",
        &many,
        1,
    );
    let r = check("t", f.as_bytes(), today());
    assert!(!r.ok && r.text.contains("problem: line 40: 31 steps; at most 30 are allowed"), "{}", r.text);
    let r = check("t", b"title: x\n", today());
    assert!(
        !r.ok && r.text.contains("problem: line 1: the file must start with a line holding only `---`"),
        "{}",
        r.text
    );
}

#[test]
fn the_rating_shows_what_to_do() {
    let f = GOOD.replace("    - wc -l README\n", "    - wc -l README\n    - cargo test\n");
    let r = check("t.steplet", f.as_bytes(), today());
    assert!(r.ok, "{}", r.text);
    let want = format!(
        "  rating: amber\n    amber, line 16: {}\n      cargo test\n      {}\n",
        CAP_RUNS_CODE.title, CAP_RUNS_CODE.advice
    );
    assert!(r.text.contains(&want), "{}", r.text);

    let f = GOOD.replace("    - wc -l README\n", "    - sudo wc -l README\n");
    let r = check("t.steplet", f.as_bytes(), today());
    assert!(!r.ok && r.text.starts_with("t.steplet: 1 problem\n"), "{}", r.text);
    let want = format!("  rating: blocked\n    blocked, line 15: {}\n      sudo wc -l README\n", BLOCK_ADMIN.title);
    assert!(r.text.contains(&want), "{}", r.text);
}
