//! Whatever our header reader accepts, saphyr-parser (a full YAML 1.2 reader) must read the same way.

use proptest::prelude::*;
use saphyr_parser::{Event, Parser};
use steplet_core::header::{Entry, Node, tree};

#[derive(Debug, PartialEq)]
enum Yaml {
    Map(Vec<(String, Yaml)>),
    List(Vec<Yaml>),
    Text(String),
}

fn ours(entries: &[Entry]) -> Yaml {
    Yaml::Map(entries.iter().map(|e| (e.key.clone(), node(&e.value))).collect())
}

fn node(n: &Node) -> Yaml {
    match n {
        Node::Map(es) => ours(es),
        Node::List(items) => Yaml::List(items.iter().map(|s| Yaml::Text(s.text.clone())).collect()),
        Node::Scalar(s) => Yaml::Text(s.text.clone()),
    }
}

/// What saphyr-parser makes of the same text. Tags, aliases and a second document count as a disagreement.
fn theirs(src: &str) -> Result<Yaml, String> {
    let mut events = Vec::new();
    for item in Parser::new_from_str(src) {
        let (ev, _span) = item.map_err(|e| format!("{e:?}"))?;
        if !matches!(ev, Event::StreamStart | Event::StreamEnd | Event::DocumentStart(_) | Event::DocumentEnd | Event::Nothing) {
            events.push(ev);
        }
    }
    let mut i = 0;
    let y = build(&events, &mut i)?;
    if i != events.len() {
        return Err(format!("{} events left over", events.len() - i));
    }
    Ok(y)
}

fn build(ev: &[Event], i: &mut usize) -> Result<Yaml, String> {
    let e = ev.get(*i).ok_or("ran out of events")?;
    *i += 1;
    match e {
        Event::Scalar(v, _, _, None) => Ok(Yaml::Text(v.to_string())),
        Event::SequenceStart(_, None) => {
            let mut items = Vec::new();
            while !matches!(ev.get(*i), Some(Event::SequenceEnd)) {
                items.push(build(ev, i)?);
            }
            *i += 1;
            Ok(Yaml::List(items))
        }
        Event::MappingStart(_, None) => {
            let mut pairs = Vec::new();
            while !matches!(ev.get(*i), Some(Event::MappingEnd)) {
                let Yaml::Text(k) = build(ev, i)? else { return Err("a key that is not text".into()) };
                pairs.push((k, build(ev, i)?));
            }
            *i += 1;
            Ok(Yaml::Map(pairs))
        }
        other => Err(format!("not allowed here: {other:?}")),
    }
}

fn agree(h: &str) -> bool {
    match tree(h, 2) {
        Ok(es) => {
            assert_eq!(theirs(h), Ok(ours(&es)), "header:\n{h}");
            true
        }
        Err(_) => false,
    }
}

const CORPUS: &[&str] = &[
    "steplet: 1\ntitle: plain text\n",
    "title: a#b\n",
    "title:    spaced\n",
    "title: https://x.org/a:b\n",
    "title: 'it''s'\n",
    "title: \"say \\\"hi\\\" \\\\\"\n",
    "title: a, b [c] {d} e|f g>h\n",
    "title: yes\nother: no\nnum: 012\nnul: null\ntilde: ~\nfloat: 1e3\n",
    "title: \"\"\n",
    "allow:\n  commands:\n    - cargo test\n    - 'echo \"#1\"'\n  domains:\n    - crates.io\n",
    "# comment\n\nsteps:\n  - one\n  # inner comment\n  - two\n",
    "title: x\u{200b}y\u{202e}z\n",
    "title: ...\n",
    "title: a -b\n",
    // The rest are refused by ours; they are here so the test shows they stay refused.
    "title: a # b\n",
    "title: key: v\n",
    "steps:\n- a\n",
    "title: &a x\nother: *a\n",
    "title: !!str x\n",
    "title: |\n  x\n",
    "title: [a, b]\n",
    "title: a\ntitle: b\n",
    "title: a\n  continued\n",
];

#[test]
fn every_accepted_corpus_header_reads_the_same() {
    let accepted = CORPUS.iter().filter(|h| agree(h)).count();
    assert_eq!(accepted, 14, "the first 14 corpus entries are accepted, the last 9 refused");
    for file in [include_str!("../../../examples/run-tests.steplet")] {
        let c = steplet_core::canon::canonicalize(file.as_bytes()).unwrap();
        assert!(agree(c.header()));
    }
}

#[derive(Clone, Debug)]
enum Gen {
    Text(String, u8),
    List(Vec<(String, u8)>),
    Map(Vec<Gen>),
}

fn scalar_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(vec![
            "a", "Z", "0", " ", ":", "#", "-", "'", "\"", "\\", "[", "]", "{", "}", ",", "&", "*", "!", "|", ">",
            "%", "@", "`", "?", "é", "\u{200b}", "...", "~", "null",
        ]),
        1..8,
    )
    .prop_map(|v| v.concat())
}

fn value() -> impl Strategy<Value = Gen> {
    let leaf = prop_oneof![
        (scalar_text(), 0u8..3).prop_map(|(t, s)| Gen::Text(t, s)),
        prop::collection::vec((scalar_text(), 0u8..3), 1..4).prop_map(Gen::List),
    ];
    leaf.prop_recursive(3, 24, 4, |inner| prop::collection::vec(inner, 1..4).prop_map(Gen::Map))
}

/// Style 0 is plain, 1 single-quoted, 2 double-quoted. `force_quotes` turns plain into double-quoted.
fn quote(t: &str, style: u8, force_quotes: bool) -> String {
    match if force_quotes && style == 0 { 2 } else { style } {
        0 => t.to_string(),
        1 => format!("'{}'", t.replace('\'', "''")),
        _ => format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\"")),
    }
}

/// Keys are k0, k1, … so they never repeat (duplicates have their own unit tests).
fn render(items: &[Gen], indent: usize, force_quotes: bool, out: &mut String) {
    let pad = " ".repeat(indent);
    for (i, g) in items.iter().enumerate() {
        match g {
            Gen::Text(t, s) => out.push_str(&format!("{pad}k{i}: {}\n", quote(t, *s, force_quotes))),
            Gen::List(list) => {
                out.push_str(&format!("{pad}k{i}:\n"));
                for (t, s) in list {
                    out.push_str(&format!("{pad}  - {}\n", quote(t, *s, force_quotes)));
                }
            }
            Gen::Map(children) => {
                out.push_str(&format!("{pad}k{i}:\n"));
                render(children, indent + 2, force_quotes, out);
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    #[test]
    fn whatever_we_accept_saphyr_reads_the_same(items in prop::collection::vec(value(), 1..5)) {
        let mut h = String::new();
        render(&items, 0, false, &mut h);
        agree(&h);
    }

    #[test]
    fn anything_quoted_is_accepted_and_agrees(items in prop::collection::vec(value(), 1..5)) {
        let mut h = String::new();
        render(&items, 0, true, &mut h);
        prop_assert!(agree(&h), "refused although every value is quoted:\n{}", h);
    }
}
