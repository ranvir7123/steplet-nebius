//! Display pieces (decision row 28): file text broken into plain text, visible badges for invisible
//! characters, and links that carry their real address. The screens draw only these, never raw file text,
//! and joining the pieces back always gives the exact original text: nothing is dropped.

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    InvisibleBadge { codepoint: u32 },
    /// `shown` is `None` for a bare address. Inside a link there are only `Text` and `InvisibleBadge` pieces.
    Link { shown: Option<Vec<Segment>>, real: Vec<Segment> },
}

pub fn segments(text: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let (mut plain, mut i) = (0, 0);
    while i < text.len() {
        let rest = &text[i..];
        let found = markdown_link(rest).map(|(len, shown, real)| (len, Some(shown), real)).or_else(|| bare_url(rest).map(|len| (len, None, &rest[..len])));
        if let Some((len, shown, real)) = found {
            out.extend(pieces(&text[plain..i]));
            out.push(Segment::Link { shown: shown.map(pieces), real: pieces(real) });
            i += len;
            plain = i;
        } else {
            i += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    out.extend(pieces(&text[plain..]));
    out
}

fn pieces(s: &str) -> Vec<Segment> {
    let mut v = Vec::new();
    let mut start = 0;
    for (j, c) in s.char_indices() {
        if is_invisible(c) {
            if start < j {
                v.push(Segment::Text(s[start..j].into()));
            }
            v.push(Segment::InvisibleBadge { codepoint: c as u32 });
            start = j + c.len_utf8();
        }
    }
    if start < s.len() {
        v.push(Segment::Text(s[start..].into()));
    }
    v
}

/// `[shown](real)` at the start of `s`: (length, shown, real).
fn markdown_link(s: &str) -> Option<(usize, &str, &str)> {
    let inner = s.strip_prefix('[')?;
    let close = inner.find([']', '[', '\n'])?;
    if !inner[close..].starts_with(']') || close == 0 {
        return None;
    }
    let after = inner[close + 1..].strip_prefix('(')?;
    let end = after.find(|c: char| c == ')' || c == '(' || c.is_whitespace())?;
    if !after[end..].starts_with(')') || end == 0 {
        return None;
    }
    Some((close + end + 4, &inner[..close], &after[..end]))
}

/// A bare `http://` or `https://` address at the start of `s`: its length.
fn bare_url(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let scheme = if b.len() >= 8 && b[..8].eq_ignore_ascii_case(b"https://") {
        8
    } else if b.len() >= 7 && b[..7].eq_ignore_ascii_case(b"http://") {
        7
    } else {
        return None;
    };
    let mut end = s.find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\'' | '`')).unwrap_or(s.len());
    while end > scheme && s[..end].ends_with(['.', ',', ';', ':', '!', '?', ')', ']']) {
        end -= 1;
    }
    (end > scheme).then_some(end)
}

/// True for characters that show nothing, or look like a space, and so get a badge: Unicode's
/// Default_Ignorable_Code_Point, and the general categories Cc, Cf, Zs, Zl, Zp and Co, except tab, newline
/// and the ordinary space. Checked against icu_properties for every code point in tests/display.rs.
pub fn is_invisible(c: char) -> bool {
    let cp = c as u32;
    if matches!(cp, 0x09 | 0x0a | 0x20) {
        return false;
    }
    INVISIBLE
        .binary_search_by(|&(lo, hi)| {
            if hi < cp {
                core::cmp::Ordering::Less
            } else if lo > cp {
                core::cmp::Ordering::Greater
            } else {
                core::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// Sorted, non-overlapping ranges, both ends included.
const INVISIBLE: &[(u32, u32)] = &[
    (0x0000, 0x001f), (0x007f, 0x00a0), (0x00ad, 0x00ad), (0x034f, 0x034f), (0x0600, 0x0605),
    (0x061c, 0x061c), (0x06dd, 0x06dd), (0x070f, 0x070f), (0x0890, 0x0891), (0x08e2, 0x08e2),
    (0x115f, 0x1160), (0x1680, 0x1680), (0x17b4, 0x17b5), (0x180b, 0x180f), (0x2000, 0x200f),
    (0x2028, 0x202f), (0x205f, 0x206f), (0x3000, 0x3000), (0x3164, 0x3164), (0xe000, 0xf8ff),
    (0xfe00, 0xfe0f), (0xfeff, 0xfeff), (0xffa0, 0xffa0), (0xfff0, 0xfffb), (0x110bd, 0x110bd),
    (0x110cd, 0x110cd), (0x13430, 0x1343f), (0x1bca0, 0x1bca3), (0x1d173, 0x1d17a), (0xe0000, 0xe0fff),
    (0xf0000, 0xffffd), (0x100000, 0x10fffd),
];
