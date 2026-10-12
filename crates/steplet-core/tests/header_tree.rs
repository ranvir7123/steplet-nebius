use steplet_core::header::{Entry, Node, ParseErrorKind as K, Scalar, tree};

fn s(text: &str, line: u32) -> Scalar {
    Scalar { text: text.into(), line }
}
fn e(key: &str, line: u32, value: Node) -> Entry {
    Entry { key: key.into(), line, value }
}
fn refused(h: &str) -> (u32, K) {
    let err = tree(h, 2).unwrap_err();
    (err.line, err.kind)
}

#[test]
fn maps_lists_and_values() {
    let h = "steplet: 1\nallow:\n  commands:\n    - cargo test\n    - \"echo '#1'\"\n# a comment\n\ntitle: 'it''s'\n";
    assert_eq!(
        tree(h, 2).unwrap(),
        vec![
            e("steplet", 2, Node::Scalar(s("1", 2))),
            e("allow", 3, Node::Map(vec![e("commands", 4, Node::List(vec![s("cargo test", 5), s("echo '#1'", 6)]))])),
            e("title", 9, Node::Scalar(s("it's", 9))),
        ]
    );
}

#[test]
fn values_yaml_reads_as_written() {
    let one = |h: &str| match tree(h, 2).unwrap().remove(0).value {
        Node::Scalar(x) => x.text,
        other => panic!("{other:?}"),
    };
    assert_eq!(one("title: a#b\n"), "a#b");
    assert_eq!(one("title:    spaced out\n"), "spaced out");
    assert_eq!(one("title: https://x.org/a:b\n"), "https://x.org/a:b");
    assert_eq!(one("title: \"say \\\"hi\\\" \\\\ ok # :\"\n"), "say \"hi\" \\ ok # :");
    assert_eq!(one("title: a, b [c] {d} e|f g>h\n"), "a, b [c] {d} e|f g>h");
    assert_eq!(one("title: ''\n"), "");
    assert_eq!(one("title: x\u{200b}y\n"), "x\u{200b}y");
}

#[test]
fn refusals_name_the_line() {
    assert_eq!(refused("title:\tx\n"), (2, K::Tab));
    assert_eq!(refused("title: a\u{7}b\n"), (2, K::ControlCharacter { codepoint: 7 }));
    assert_eq!(refused("title: a\u{feff}\n"), (2, K::ControlCharacter { codepoint: 0xfeff }));
    assert_eq!(refused("title: a\u{85}\n"), (2, K::ControlCharacter { codepoint: 0x85 }));
    assert_eq!(refused("title: a\u{2028}\n"), (2, K::ControlCharacter { codepoint: 0x2028 }));
    assert_eq!(refused("Title: x\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("title:x\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("\"title\": x\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("? title\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("...\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("- a\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("title: x\n title2: y\n"), (3, K::BadIndent { expected: 0 }));
    assert_eq!(refused("  title: x\n"), (2, K::BadIndent { expected: 0 }));
    assert_eq!(refused("allow:\n   commands:\n"), (3, K::BadIndent { expected: 2 }));
    assert_eq!(refused("title: a\n  b\n"), (3, K::BadIndent { expected: 0 }));
    assert_eq!(refused("title:\n"), (2, K::EmptyValue { key: "title".into() }));
    assert_eq!(refused("steps:\n- a\n"), (2, K::ListNotIndented { key: "steps".into() }));
    assert_eq!(refused("steps:\n  - a\n  b: c\n"), (4, K::NotAListItem));
    assert_eq!(refused("steps:\n  -\n"), (3, K::EmptyListItem));
    assert_eq!(refused("title: a\ntitle: b\n"), (3, K::DuplicateKey { key: "title".into(), first_line: 2 }));
    assert_eq!(
        refused("allow:\n  read:\n    - a\n  read:\n    - b\n"),
        (5, K::DuplicateKey { key: "read".into(), first_line: 3 })
    );
    for symbol in "-?:,[]{}#&*!|>%@`".chars() {
        assert_eq!(refused(&format!("title: {symbol}x\n")), (2, K::StartsWithSymbol { symbol }), "{symbol}");
    }
    assert_eq!(refused("steps:\n  - - a\n"), (3, K::StartsWithSymbol { symbol: '-' }));
    assert_eq!(refused("title: a: b\n"), (2, K::ColonInPlainText));
    assert_eq!(refused("title: a:\n"), (2, K::ColonInPlainText));
    assert_eq!(refused("steps:\n  - key: v\n"), (3, K::ColonInPlainText));
    assert_eq!(refused("title: a #b\n"), (2, K::HashInPlainText));
    assert_eq!(refused("title: \"abc\n"), (2, K::UnclosedQuote));
    assert_eq!(refused("title: 'abc\n"), (2, K::UnclosedQuote));
    assert_eq!(refused("title: \"abc\\\"\n"), (2, K::UnclosedQuote));
    assert_eq!(refused("title: \"a\\nb\"\n"), (2, K::BadEscape { found: 'n' }));
    assert_eq!(refused("title: \"a\" b\n"), (2, K::TextAfterQuote));
    assert_eq!(refused("title: 'a' # note\n"), (2, K::TextAfterQuote));
}

#[test]
fn every_refusal_has_words_and_help() {
    for h in [
        "title:\tx\n",
        "title: a\u{7}\n",
        "Title: x\n",
        "  title: x\n",
        "title:\n",
        "steps:\n- a\n",
        "steps:\n  - a\n  b: c\n",
        "steps:\n  -\n",
        "title: a\ntitle: b\n",
        "title: [x]\n",
        "title: a: b\n",
        "title: a #b\n",
        "title: \"a\n",
        "title: \"\\n\"\n",
        "title: \"a\" b\n",
    ] {
        let err = tree(h, 2).unwrap_err();
        assert!(err.to_string().starts_with("line "), "{h:?}");
        assert!(!err.help().allowed.is_empty() && !err.help().example.is_empty(), "{h:?}");
    }
}
