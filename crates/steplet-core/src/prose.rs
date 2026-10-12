//! The words a person reads (G7, G3's phrase rules): words and phrases, code between backticks, fenced code,
//! long encoded runs, and characters that hide text.

use alloc::string::String;
use alloc::vec::Vec;

use crate::display;

/// A phrase: slots in order; each slot lists the words that may stand there; an empty word makes it optional.
pub type Phrase = &'static [&'static [&'static str]];

/// The words of `text` in lower case: letters, digits and apostrophes inside a word (a curly ’ counts as ').
/// Quotes around a word are dropped, so `'ignore' previous` reads like `ignore previous`.
pub fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '\u{2019}'))
        .map(|w| w.trim_matches(['\'', '\u{2019}']))
        .filter(|w| !w.is_empty())
        .map(|w| w.chars().map(|c| if c == '\u{2019}' { '\'' } else { c }).flat_map(char::to_lowercase).collect())
        .collect()
}

fn matches_at(words: &[String], phrase: &[&[&str]]) -> bool {
    match phrase.split_first() {
        None => true,
        Some((slot, rest)) => slot.iter().any(|w| {
            if w.is_empty() {
                matches_at(words, rest)
            } else {
                words.first().is_some_and(|x| x == w) && matches_at(&words[1..], rest)
            }
        }),
    }
}

const NEGATIONS: &[&str] = &["not", "never", "don't", "dont", "no", "without", "nobody"];

/// True when one of `phrases` appears inside one sentence of `text`. With `unless_negated`, a phrase with a
/// negation among the three words before it in the same sentence does not count ("never paste your key"), but a
/// negation in the sentence before does ("No problem. Paste your key.").
pub fn has_phrase(text: &str, phrases: &[Phrase], unless_negated: bool) -> bool {
    text.split(['.', '!', '?', ';', ':']).any(|sentence| {
        let words = words(sentence);
        (0..words.len()).any(|i| {
            phrases.iter().any(|p| matches_at(&words[i..], p))
                && !(unless_negated && words[i.saturating_sub(3)..i].iter().any(|w| NEGATIONS.contains(&w.as_str())))
        })
    })
}

/// True when `text` holds a hiding character. The joiner U+200D and the presentation selector U+FE0F right after a
/// character outside ASCII are part of ordinary emoji (a family, a tick with colour) and do not count.
pub fn hides(text: &str) -> bool {
    let mut prev = ' ';
    for c in text.chars() {
        if is_hiding(c) && !(matches!(c, '\u{200d}' | '\u{fe0f}') && !prev.is_ascii()) {
            return true;
        }
        prev = c;
    }
    false
}

/// The code between pairs of backticks in `line` (`like this`). Nothing when the backticks do not pair up.
pub fn code_spans(line: &str) -> Vec<&str> {
    if !line.matches('`').count().is_multiple_of(2) {
        return Vec::new();
    }
    line.split('`').skip(1).step_by(2).collect()
}

/// The lines of `body` inside fenced code blocks (between ``` or ~~~ lines), with a leading `$ ` prompt removed,
/// each with its index among the body's lines.
pub fn fenced_lines(body: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut inside = false;
    for (i, line) in body.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            inside = !inside;
        } else if inside {
            out.push((i, t.strip_prefix("$ ").unwrap_or(t)));
        }
    }
    out
}

/// The length of the longest run of characters used by base64 and hex (letters, digits and `+ / = - _`).
pub fn longest_encoded_run(text: &str) -> usize {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || "+/=-_".contains(c))).map(str::len).max().unwrap_or(0)
}

/// G7: an invisible character that is not simply space-like: zero-width characters, direction controls, tags,
/// control characters and the like.
pub fn is_hiding(c: char) -> bool {
    display::is_invisible(c) && !display::is_space_like(c)
}
