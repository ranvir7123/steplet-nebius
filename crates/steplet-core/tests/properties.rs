//! The three rules the milestone plan asks for: cleaning up twice changes nothing; Windows and Unix line
//! endings sign the same; any edit to a non-whitespace character breaks the signature.

mod common;

use proptest::prelude::*;
use steplet_core::NAMESPACE;
use steplet_core::canon::canonicalize;
use steplet_core::sig::{SigStatus, verify};

fn messy_text() -> impl Strategy<Value = String> {
    let piece = prop::sample::select(vec![
        "---",
        "\n",
        "\r\n",
        " ",
        "\t",
        "a",
        "é",
        "\u{feff}",
        "\u{200b}",
        "signature: |",
        "  -----BEGIN SSH SIGNATURE-----",
        "  -----END SSH SIGNATURE-----",
        "  QUJD",
        "steplet: 1",
        "\r",
    ]);
    (any::<bool>(), prop::collection::vec(piece, 0..40))
        .prop_map(|(head, v)| format!("{}{}", if head { "---\n" } else { "" }, v.concat()))
}

fn file_text() -> impl Strategy<Value = String> {
    let words = prop::sample::select(vec!["Run", "the", "tests", "café", "#1", "a:b", "x", "\u{200b}", "--", "→"]);
    let line = prop::collection::vec(words, 1..6).prop_map(|w| w.join(" "));
    (line.clone(), prop::collection::vec(line.clone(), 1..5), prop::collection::vec(line, 0..6)).prop_map(
        |(title, steps, body)| {
            let mut f = format!("---\nsteplet: 1\ntitle: \"{title}\"\nsender: Test\nexpires: 2026-12-31\nsteps:\n");
            for s in &steps {
                f.push_str(&format!("  - \"{s}\"\n"));
            }
            f.push_str("---\n");
            for b in &body {
                f.push_str(b);
                f.push('\n');
            }
            f
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn cleaning_up_twice_changes_nothing(raw in messy_text()) {
        if let Ok(c) = canonicalize(raw.as_bytes()) {
            prop_assert_eq!(canonicalize(c.text().as_bytes()), Ok(c));
        }
    }

    #[test]
    fn windows_and_unix_line_endings_sign_the_same(f in file_text(), spaces in 0usize..3) {
        let signed = common::sign(&f, 7);
        let windows = signed.replace('\n', &format!("{}\r\n", " ".repeat(spaces)));
        let unix = canonicalize(signed.as_bytes()).unwrap();
        let win = canonicalize(windows.as_bytes()).unwrap();
        prop_assert_eq!(win.signed(), unix.signed());
        // An explicit message: without one, prop_assert! puts this expression into a format string,
        // where `{ .. }` does not compile.
        prop_assert!(matches!(verify(&unix, NAMESPACE), SigStatus::Valid { .. }), "the Unix version does not verify");
        prop_assert_eq!(verify(&win, NAMESPACE), verify(&unix, NAMESPACE));
    }

    #[test]
    fn any_edit_to_non_whitespace_breaks_the_signature(
        f in file_text(),
        pick in any::<prop::sample::Index>(),
        op in 0u8..3,
        ch in prop::sample::select(vec!['x', 'Q', '7', '-', '#', 'é']),
    ) {
        let signed = common::sign(&f, 9);
        let spots: Vec<usize> = signed.char_indices().filter(|(_, c)| !c.is_whitespace()).map(|(i, _)| i).collect();
        let at = spots[pick.index(spots.len())];
        let old = signed[at..].chars().next().unwrap();
        let after = &signed[at + old.len_utf8()..];
        let edited = match op {
            0 if old != ch => format!("{}{ch}{after}", &signed[..at]),
            0 | 1 => format!("{}{after}", &signed[..at]),
            _ => format!("{}{ch}{}", &signed[..at], &signed[at..]),
        };
        if let Ok(c) = canonicalize(edited.as_bytes()) {
            prop_assert!(!matches!(verify(&c, NAMESPACE), SigStatus::Valid { .. }), "still valid after an edit at byte {}:\n{}", at, edited);
        }
    }
}
