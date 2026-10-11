use steplet_core::canon::canonicalize;
use steplet_core::header::{Steplet, parse};
use steplet_core::limits::{LimitBreach, Limited, check};

fn file(steps: usize, commands: &[String], never: usize) -> Steplet {
    let mut f = String::from("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n");
    for i in 0..steps {
        f.push_str(&format!("  - step {i}\n"));
    }
    if !commands.is_empty() {
        f.push_str("allow:\n  commands:\n");
        for c in commands {
            f.push_str(&format!("    - \"{c}\"\n"));
        }
    }
    if never > 0 {
        f.push_str("never:\n");
        for i in 0..never {
            f.push_str(&format!("  - never {i}\n"));
        }
    }
    f.push_str("---\n");
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap()
}

#[test]
fn at_the_limits_nothing_is_reported() {
    let commands: Vec<String> = (0..25).map(|i| format!("{i:0>299}")).collect();
    assert_eq!(check(&file(30, &commands, 25)), vec![]);
}

#[test]
fn one_over_each_limit_names_the_first_line_over() {
    assert_eq!(check(&file(31, &[], 0)), vec![LimitBreach { what: Limited::Steps, line: 37, found: 31, max: 30 }]);
    let commands: Vec<String> = (0..26).map(|i| format!("c{i}")).collect();
    let b = check(&file(1, &commands, 0));
    assert_eq!(b, vec![LimitBreach { what: Limited::Commands, line: 35, found: 26, max: 25 }]);
    let b = check(&file(1, &[], 26));
    assert_eq!(b, vec![LimitBreach { what: Limited::Never, line: 34, found: 26, max: 25 }]);
}

#[test]
fn a_command_of_300_characters_is_too_long_counted_in_characters() {
    let b = check(&file(1, &["é".repeat(300)], 0));
    assert_eq!(b, vec![LimitBreach { what: Limited::CommandLength, line: 10, found: 300, max: 299 }]);
    assert_eq!(check(&file(1, &["é".repeat(299)], 0)), vec![]);
}

#[test]
fn domains_and_folders() {
    let mut f = String::from("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n  - a\nallow:\n  domains:\n");
    for i in 0..16 {
        f.push_str(&format!("    - d{i}.example.org\n"));
    }
    f.push_str("  read:\n");
    for i in 0..11 {
        f.push_str(&format!("    - r{i}\n"));
    }
    f.push_str("  write:\n");
    for i in 0..11 {
        f.push_str(&format!("    - w{i}\n"));
    }
    f.push_str("---\n");
    let s = parse(&canonicalize(f.as_bytes()).unwrap()).unwrap();
    let what: Vec<Limited> = check(&s).into_iter().map(|b| b.what).collect();
    assert_eq!(what, [Limited::Domains, Limited::ReadFolders, Limited::WriteFolders]);
}

#[test]
fn words_and_help() {
    let b = LimitBreach { what: Limited::Steps, line: 37, found: 31, max: 30 };
    assert_eq!(b.to_string(), "line 37: 31 steps; at most 30 are allowed");
    let b = LimitBreach { what: Limited::CommandLength, line: 10, found: 300, max: 299 };
    assert_eq!(b.to_string(), "line 10: this command is 300 characters long; it must be under 300");
    assert!(!b.help().allowed.is_empty());
}

#[test]
fn long_title_sender_and_lines_name_their_line() {
    let f = format!(
        "---\nsteplet: 1\ntitle: {}\nsender: {}\nexpires: 2026-10-25\nsteps:\n  - {}\nnever:\n  - {}\n---\n",
        "t".repeat(121),
        "s".repeat(81),
        "a".repeat(300),
        "n".repeat(300)
    );
    let s = parse(&canonicalize(f.as_bytes()).unwrap()).unwrap();
    assert_eq!(
        check(&s),
        vec![
            LimitBreach { what: Limited::TitleLength, line: 3, found: 121, max: 120 },
            LimitBreach { what: Limited::SenderLength, line: 4, found: 81, max: 80 },
            LimitBreach { what: Limited::ValueLength, line: 7, found: 300, max: 299 },
            LimitBreach { what: Limited::ValueLength, line: 9, found: 300, max: 299 },
        ]
    );
}

#[test]
fn at_the_line_limits_nothing_is_reported() {
    let f = format!(
        "---\nsteplet: 1\ntitle: {}\nsender: {}\nexpires: 2026-10-25\nrepository:\n  url: https://github.com/{}\n  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d\nsteps:\n  - {}\nallow:\n  read:\n    - {}\n---\n",
        "t".repeat(120),
        "s".repeat(80),
        "p".repeat(299 - "https://github.com/".len()),
        "é".repeat(299),
        "r".repeat(299)
    );
    let s = parse(&canonicalize(f.as_bytes()).unwrap()).unwrap();
    assert_eq!(check(&s), vec![]);
}

#[test]
fn words_for_line_limits() {
    let b = LimitBreach { what: Limited::TitleLength, line: 3, found: 121, max: 120 };
    assert_eq!(b.to_string(), "line 3: the title is 121 characters long; at most 120 are allowed");
    let b = LimitBreach { what: Limited::ValueLength, line: 7, found: 300, max: 299 };
    assert_eq!(b.to_string(), "line 7: this line is 300 characters long; it must be under 300");
    assert!(b.help().allowed.contains("body"));
}
