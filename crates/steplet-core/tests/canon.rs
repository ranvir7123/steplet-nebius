use steplet_core::canon::{CanonError, Canonical, canonicalize};

fn ok(raw: &str) -> Canonical {
    canonicalize(raw.as_bytes()).expect("should clean up")
}

#[test]
fn windows_line_endings_become_unix_ones() {
    assert_eq!(ok("---\r\nsteplet: 1\r\n---\r\nbody\r\n").text(), "---\nsteplet: 1\n---\nbody\n");
}

#[test]
fn spaces_and_tabs_at_line_ends_go() {
    assert_eq!(ok("---  \nsteplet: 1\t \n---\nbody \t\n").text(), "---\nsteplet: 1\n---\nbody\n");
}

#[test]
fn exactly_one_final_newline() {
    assert_eq!(ok("---\na: b\n---\nbody").text(), "---\na: b\n---\nbody\n");
    assert_eq!(ok("---\na: b\n---\nbody\n\n\n \n").text(), "---\na: b\n---\nbody\n");
}

#[test]
fn one_byte_order_mark_is_dropped_and_only_one() {
    assert_eq!(ok("\u{feff}---\na: b\n---\n").text(), "---\na: b\n---\n");
    assert_eq!(canonicalize("\u{feff}\u{feff}---\na: b\n---\n".as_bytes()), Err(CanonError::NoHeaderStart));
}

#[test]
fn no_unicode_normalisation() {
    let c = ok("---\na: e\u{301}\n---\n\u{e9}\n");
    assert!(c.text().contains("e\u{301}"));
    assert!(c.text().contains('\u{e9}'));
}

#[test]
fn leading_and_inner_spaces_stay() {
    assert_eq!(ok("---\na:  b  c\n---\n  x\n").text(), "---\na:  b  c\n---\n  x\n");
}

#[test]
fn bad_utf8_names_its_line() {
    assert_eq!(canonicalize(b"---\na: b\n\xff\n---\n"), Err(CanonError::NotUtf8 { line: 3 }));
}

#[test]
fn a_lone_carriage_return_is_refused() {
    assert_eq!(canonicalize(b"---\na: b\rc\n---\n"), Err(CanonError::LoneCarriageReturn { line: 2 }));
}

#[test]
fn the_header_opens_on_line_one_and_closes() {
    for raw in ["", "\n---\na: b\n---\n", "title: x\n", " ---\na: b\n---\n"] {
        assert_eq!(canonicalize(raw.as_bytes()), Err(CanonError::NoHeaderStart), "{raw:?}");
    }
    assert_eq!(canonicalize(b"---\na: b\n"), Err(CanonError::NoHeaderEnd));
}

#[test]
fn header_and_body_are_split_at_the_first_closing_line() {
    let c = ok("---\nsteplet: 1\ntitle: x\n---\n# Hi\n---\nmore\n");
    assert_eq!(c.header(), "steplet: 1\ntitle: x\n");
    assert_eq!(c.body(), "# Hi\n---\nmore\n");
    assert_eq!(c.signed(), c.text());
    assert!(c.signature().is_none());
    assert_eq!(ok("---\na: b\n---\n").body(), "");
}

#[test]
fn size_limits() {
    let mut at_limit = b"---\na: b\n---\n".to_vec();
    at_limit.resize(256 * 1024, b'x');
    assert!(canonicalize(&at_limit).is_ok());
    at_limit.push(b'x');
    assert_eq!(canonicalize(&at_limit), Err(CanonError::FileTooLarge { bytes: 262_145 }));

    let mut big_header = String::from("---\n");
    while big_header.len() < 32 * 1024 + 4 {
        big_header.push_str("never: x\n");
    }
    big_header.push_str("---\n");
    assert!(matches!(canonicalize(big_header.as_bytes()), Err(CanonError::HeaderTooLarge { .. })));
}

#[test]
fn every_error_has_words_and_help() {
    let all = [
        CanonError::FileTooLarge { bytes: 1 },
        CanonError::NotUtf8 { line: 1 },
        CanonError::LoneCarriageReturn { line: 1 },
        CanonError::NoHeaderStart,
        CanonError::NoHeaderEnd,
        CanonError::HeaderTooLarge { bytes: 1 },
        CanonError::SignatureMisplaced { line: 1 },
        CanonError::SignatureTwice { line: 1 },
        CanonError::SignatureBroken { line: 1 },
    ];
    for e in all {
        assert!(!e.to_string().is_empty());
        assert!(!e.help().allowed.is_empty(), "{e:?}");
    }
}
