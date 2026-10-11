use icu_properties::props::{DefaultIgnorableCodePoint, GeneralCategory};
use icu_properties::{CodePointMapData, CodePointSetData};
use proptest::prelude::*;
use steplet_core::display::{Segment, is_invisible, segments};

fn t(s: &str) -> Segment {
    Segment::Text(s.into())
}
fn badge(cp: u32) -> Segment {
    Segment::InvisibleBadge { codepoint: cp }
}

/// Joins the pieces back into the text they came from.
fn source(segs: &[Segment]) -> String {
    segs.iter()
        .map(|s| match s {
            Segment::Text(x) => x.clone(),
            Segment::InvisibleBadge { codepoint } => char::from_u32(*codepoint).unwrap().to_string(),
            Segment::Link { shown: Some(shown), real } => format!("[{}]({})", source(shown), source(real)),
            Segment::Link { shown: None, real } => source(real),
        })
        .collect()
}

#[test]
fn invisible_characters_become_badges() {
    assert_eq!(segments("a\u{200b}b"), [t("a"), badge(0x200b), t("b")]);
    assert_eq!(segments("\u{202e}abc"), [badge(0x202e), t("abc")]);
    assert_eq!(segments("a\u{a0}b\u{feff}"), [t("a"), badge(0xa0), t("b"), badge(0xfeff)]);
    assert_eq!(segments("a\tb\nc d"), [t("a\tb\nc d")]);
    assert_eq!(segments(""), []);
}

#[test]
fn markdown_links_show_their_real_address() {
    assert_eq!(
        segments("see [docs](https://evil.example/x) now"),
        [t("see "), Segment::Link { shown: Some(vec![t("docs")]), real: vec![t("https://evil.example/x")] }, t(" now")]
    );
    assert_eq!(
        segments("[a](https://x\u{200b}.org)"),
        [Segment::Link { shown: Some(vec![t("a")]), real: vec![t("https://x"), badge(0x200b), t(".org")] }]
    );
    assert_eq!(segments("[a](javascript:alert(1))"), [t("[a](javascript:alert(1))")]);
    for not_a_link in ["[a] (b)", "[](x)", "[a]()", "[a](b c)", "[a\nb](c)"] {
        assert_eq!(source(&segments(not_a_link)), not_a_link);
        assert!(!segments(not_a_link).iter().any(|s| matches!(s, Segment::Link { shown: Some(_), .. })), "{not_a_link:?}");
    }
}

#[test]
fn bare_addresses() {
    assert_eq!(
        segments("go to https://a.org/x."),
        [t("go to "), Segment::Link { shown: None, real: vec![t("https://a.org/x")] }, t(".")]
    );
    assert_eq!(segments("HTTPS://A.ORG"), [Segment::Link { shown: None, real: vec![t("HTTPS://A.ORG")] }]);
    assert_eq!(segments("(http://a.org)"), [t("("), Segment::Link { shown: None, real: vec![t("http://a.org")] }, t(")")]);
    assert_eq!(segments("https://"), [t("https://")]);
}

#[test]
fn the_invisible_table_matches_unicode() {
    let di = CodePointSetData::new::<DefaultIgnorableCodePoint>();
    let gc = CodePointMapData::<GeneralCategory>::new();
    let mut wrong = Vec::new();
    for cp in 0..=0x10_ffff_u32 {
        let Some(c) = char::from_u32(cp) else { continue };
        let expected = !matches!(c, '\t' | '\n' | ' ')
            && (di.contains(c)
                || matches!(
                    gc.get(c),
                    GeneralCategory::Control
                        | GeneralCategory::Format
                        | GeneralCategory::SpaceSeparator
                        | GeneralCategory::LineSeparator
                        | GeneralCategory::ParagraphSeparator
                        | GeneralCategory::PrivateUse
                ));
        if is_invisible(c) != expected {
            wrong.push(format!("U+{cp:04X} (should be {expected})"));
        }
    }
    assert!(wrong.is_empty(), "{} code points differ, first: {:?}", wrong.len(), &wrong[..wrong.len().min(40)]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    #[test]
    fn nothing_is_ever_dropped(parts in prop::collection::vec(prop::sample::select(vec![
        "a", " ", "\n", "[", "]", "(", ")", "x", "https://", "http://", "e.org", ".", "\u{200b}", "\u{202e}",
        "é", "<", ">", "\"", "`", "!", "?",
    ]), 0..30)) {
        let text = parts.concat();
        prop_assert_eq!(source(&segments(&text)), text);
    }
}
