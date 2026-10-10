//! The header reader (decision rows 15 and R12). A small, strict part of YAML, read once from the signed bytes:
//! maps and lists of one-line values, plain or quoted. Anything else is refused with its line, what is
//! allowed, and an example. Field types come from the field list (`parse`), never from YAML's guessing.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use crate::Help;

/// One value and the line it is on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scalar {
    pub text: String,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Map(Vec<Entry>),
    List(Vec<Scalar>),
    Scalar(Scalar),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub line: u32,
    pub value: Node,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: u32,
    pub kind: ParseErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    Tab,
    ControlCharacter { codepoint: u32 },
    NotAKeyLine,
    BadIndent { expected: usize },
    EmptyValue { key: String },
    ListNotIndented { key: String },
    NotAListItem,
    EmptyListItem,
    DuplicateKey { key: String, first_line: u32 },
    StartsWithSymbol { symbol: char },
    ColonInPlainText,
    HashInPlainText,
    UnclosedQuote,
    BadEscape { found: char },
    TextAfterQuote,
}

use ParseErrorKind as K;

fn err(line: u32, kind: ParseErrorKind) -> ParseError {
    ParseError { line, kind }
}

struct Line<'a> {
    no: u32,
    indent: usize,
    text: &'a str,
}

/// Characters YAML allows in a document, minus U+0085, U+2028 and U+2029 (line breaks in older YAML)
/// and U+FEFF (byte-order mark), so no YAML reader can see a different line structure. Tab is refused separately.
fn allowed_char(c: char) -> bool {
    matches!(c, '\u{20}'..='\u{7e}' | '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
        && !matches!(c, '\u{2028}' | '\u{2029}' | '\u{feff}')
}

/// Reads `header` (the lines between the two `---` lines; its first line is line `first_line` of the file)
/// into a tree, without knowing any field names.
pub fn tree(header: &str, first_line: u32) -> Result<Vec<Entry>, ParseError> {
    let mut lines = Vec::new();
    for (i, raw) in header.lines().enumerate() {
        let no = first_line + i as u32;
        let raw = raw.trim_end_matches(' ');
        if raw.contains('\t') {
            return Err(err(no, K::Tab));
        }
        if let Some(c) = raw.chars().find(|&c| !allowed_char(c)) {
            return Err(err(no, K::ControlCharacter { codepoint: c as u32 }));
        }
        let text = raw.trim_start_matches(' ');
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        lines.push(Line { no, indent: raw.len() - text.len(), text });
    }
    let mut pos = 0;
    map(&lines, &mut pos, 0)
}

fn is_item(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

fn map(lines: &[Line], pos: &mut usize, indent: usize) -> Result<Vec<Entry>, ParseError> {
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(line) = lines.get(*pos).filter(|l| l.indent == indent) {
        let (key, rest) = split_key(line)?;
        if let Some(first) = entries.iter().find(|e| e.key == key) {
            return Err(err(line.no, K::DuplicateKey { key: key.to_string(), first_line: first.line }));
        }
        *pos += 1;
        let value = if !rest.is_empty() {
            Node::Scalar(scalar(rest, line.no)?)
        } else {
            match lines.get(*pos) {
                Some(next) if next.indent > indent => {
                    if next.indent != indent + 2 {
                        return Err(err(next.no, K::BadIndent { expected: indent + 2 }));
                    }
                    if is_item(next.text) {
                        Node::List(list(lines, pos, indent + 2)?)
                    } else {
                        Node::Map(map(lines, pos, indent + 2)?)
                    }
                }
                Some(next) if next.indent == indent && is_item(next.text) => {
                    return Err(err(line.no, K::ListNotIndented { key: key.to_string() }));
                }
                _ => return Err(err(line.no, K::EmptyValue { key: key.to_string() })),
            }
        };
        entries.push(Entry { key: key.to_string(), line: line.no, value });
    }
    if let Some(next) = lines.get(*pos).filter(|l| l.indent > indent) {
        return Err(err(next.no, K::BadIndent { expected: indent }));
    }
    Ok(entries)
}

fn list(lines: &[Line], pos: &mut usize, indent: usize) -> Result<Vec<Scalar>, ParseError> {
    let mut items = Vec::new();
    while let Some(line) = lines.get(*pos).filter(|l| l.indent == indent) {
        if !is_item(line.text) {
            return Err(err(line.no, K::NotAListItem));
        }
        let rest = line.text[1..].trim_start_matches(' ');
        if rest.is_empty() {
            return Err(err(line.no, K::EmptyListItem));
        }
        items.push(scalar(rest, line.no)?);
        *pos += 1;
    }
    if let Some(next) = lines.get(*pos).filter(|l| l.indent > indent) {
        return Err(err(next.no, K::BadIndent { expected: indent }));
    }
    Ok(items)
}

fn split_key<'a>(line: &Line<'a>) -> Result<(&'a str, &'a str), ParseError> {
    let bad = || err(line.no, K::NotAKeyLine);
    let colon = line.text.find(':').ok_or_else(bad)?;
    let key = &line.text[..colon];
    let mut bytes = key.bytes();
    let first_ok = bytes.next().is_some_and(|c| c.is_ascii_lowercase());
    if !first_ok || !bytes.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-') {
        return Err(bad());
    }
    let after = &line.text[colon + 1..];
    if after.is_empty() {
        return Ok((key, ""));
    }
    let value = after.strip_prefix(' ').ok_or_else(bad)?.trim_start_matches(' ');
    Ok((key, value))
}

const SYMBOLS: &str = "-?:,[]{}#&*!|>'\"%@`";

fn scalar(s: &str, line: u32) -> Result<Scalar, ParseError> {
    let text = match s.chars().next() {
        Some('"') => double_quoted(&s[1..], line)?,
        Some('\'') => single_quoted(&s[1..], line)?,
        Some(c) if SYMBOLS.contains(c) => return Err(err(line, K::StartsWithSymbol { symbol: c })),
        _ if s.contains(": ") || s.ends_with(':') => return Err(err(line, K::ColonInPlainText)),
        _ if s.contains(" #") => return Err(err(line, K::HashInPlainText)),
        _ => s.to_string(),
    };
    Ok(Scalar { text, line })
}

fn double_quoted(s: &str, line: u32) -> Result<String, ParseError> {
    let mut out = String::new();
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some((_, q @ ('"' | '\\'))) => out.push(q),
                Some((_, found)) => return Err(err(line, K::BadEscape { found })),
                None => return Err(err(line, K::UnclosedQuote)),
            },
            '"' if s[i + 1..].is_empty() => return Ok(out),
            '"' => return Err(err(line, K::TextAfterQuote)),
            _ => out.push(c),
        }
    }
    Err(err(line, K::UnclosedQuote))
}

fn single_quoted(s: &str, line: u32) -> Result<String, ParseError> {
    let mut out = String::new();
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '\'' {
            out.push(c);
        } else if chars.peek().is_some_and(|&(_, n)| n == '\'') {
            chars.next();
            out.push('\'');
        } else if s[i + 1..].is_empty() {
            return Ok(out);
        } else {
            return Err(err(line, K::TextAfterQuote));
        }
    }
    Err(err(line, K::UnclosedQuote))
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: ", self.line)?;
        match &self.kind {
            K::Tab => write!(f, "a tab character"),
            K::ControlCharacter { codepoint } => write!(f, "an invisible or control character U+{codepoint:04X}"),
            K::NotAKeyLine => write!(f, "this line is not `name: value`"),
            K::BadIndent { expected } => write!(f, "this line should start {expected} spaces in"),
            K::EmptyValue { key } => write!(f, "`{key}` has no value"),
            K::ListNotIndented { key } => write!(f, "the items of `{key}` are not indented"),
            K::NotAListItem => write!(f, "this line is inside a list but does not start with `- `"),
            K::EmptyListItem => write!(f, "an empty list item"),
            K::DuplicateKey { key, first_line } => write!(f, "`{key}` again; it is already on line {first_line}"),
            K::StartsWithSymbol { symbol } => write!(f, "a value starting with `{symbol}` must be in double quotes"),
            K::ColonInPlainText => write!(f, "a value with a colon then a space, or ending in a colon, must be in double quotes"),
            K::HashInPlainText => write!(f, "a value with a space then `#` must be in double quotes"),
            K::UnclosedQuote => write!(f, "a quote that is not closed on the same line"),
            K::BadEscape { found } => write!(f, "`\\{found}` inside double quotes"),
            K::TextAfterQuote => write!(f, "text after the closing quote"),
        }
    }
}

impl ParseError {
    pub fn help(&self) -> Help {
        let (allowed, example) = match &self.kind {
            K::Tab => ("indent with spaces, two per level; no tab characters anywhere in the header", "allow:\n  commands:\n    - cargo test"),
            K::ControlCharacter { .. } => ("ordinary visible text; control characters, byte-order marks and Unicode line separators are not allowed in the header", "title: Run the tests"),
            K::NotAKeyLine => ("one `name: value` per line; names are lowercase letters, digits and hyphens, with a space after the colon", "title: Run the tests"),
            K::BadIndent { .. } => ("each level indented exactly two spaces more than the line it belongs to; every value on one line", "repository:\n  url: https://github.com/octocat/Hello-World.git"),
            K::EmptyValue { .. } => ("a value after the colon, or lines indented two spaces below it; leave out a field you do not need", "title: Run the tests"),
            K::ListNotIndented { .. } => ("list items indented two spaces more than the list's name", "steps:\n  - Build the project."),
            K::NotAListItem => ("every line of a list starts with `- `", "steps:\n  - Build the project.\n  - Run the tests."),
            K::EmptyListItem => ("text after every `- `", "steps:\n  - Run the tests."),
            K::DuplicateKey { .. } => ("each name once at each level", "title: Run the tests"),
            K::StartsWithSymbol { .. } => ("text starting with a letter, digit or other ordinary character; otherwise put the value in double quotes", "title: \"[draft] Run the tests\""),
            K::ColonInPlainText => ("double quotes around text that contains a colon followed by a space, or ends with a colon", "title: \"Note: run the tests\""),
            K::HashInPlainText => ("double quotes around text that contains a space followed by #", "title: \"Fix issue #12\""),
            K::UnclosedQuote => ("the closing quote on the same line", "title: \"Run the tests\""),
            K::BadEscape { .. } => ("inside double quotes, a backslash only before a quote (\\\") or another backslash (\\\\)", "title: \"Say \\\"hi\\\"\""),
            K::TextAfterQuote => ("nothing after the closing quote, not even a comment", "title: \"Run the tests\""),
        };
        Help { allowed, example }
    }
}
