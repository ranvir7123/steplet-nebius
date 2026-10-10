//! The header reader (decision rows 15 and R12). A small, strict part of YAML, read once from the signed bytes:
//! maps and lists of one-line values, plain or quoted. Anything else is refused with its line, what is
//! allowed, and an example. Field types come from the field list (`parse`), never from YAML's guessing.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use crate::Help;
use crate::canon::Canonical;
use crate::date::Date;

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
    UnknownKey { key: String, allowed: &'static [&'static str] },
    MissingKey { key: &'static str },
    FirstKeyNotSteplet,
    UnsupportedVersion { found: String },
    ExpectedText { key: String },
    ExpectedList { key: String },
    ExpectedMap { key: String },
    BadDate,
    BadUrl,
    BadCommit,
    BadDomain,
    BadFolder,
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
            K::UnknownKey { key, allowed } => {
                write!(f, "unknown name `{key}`; allowed here: ")?;
                for (i, a) in allowed.iter().enumerate() {
                    write!(f, "{}{a}", if i == 0 { "" } else { ", " })?;
                }
                Ok(())
            }
            K::MissingKey { key } => write!(f, "`{key}` is missing"),
            K::FirstKeyNotSteplet => write!(f, "the header must start with `steplet: 1`"),
            K::UnsupportedVersion { found } => write!(f, "format version {found}; this Steplet reads version 1 only"),
            K::ExpectedText { key } => write!(f, "`{key}` must be one line of text"),
            K::ExpectedList { key } => write!(f, "`{key}` must be a list"),
            K::ExpectedMap { key } => write!(f, "`{key}` must have named fields below it"),
            K::BadDate => write!(f, "not a date written YYYY-MM-DD"),
            K::BadUrl => write!(f, "not a public https address Steplet accepts"),
            K::BadCommit => write!(f, "not a full commit hash"),
            K::BadDomain => write!(f, "not a host name Steplet accepts"),
            K::BadFolder => write!(f, "not a folder inside the repository"),
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
            K::UnknownKey { .. } => ("only the names listed in this message", "title: Run the tests"),
            K::MissingKey { key } => ("every required field", match *key {
                "steplet" => "steplet: 1",
                "sender" => "sender: Ana Lee",
                "expires" => "expires: 2026-10-25",
                "steps" => "steps:\n  - Run the tests.",
                "url" => "  url: https://github.com/octocat/Hello-World.git",
                "commit" => "  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
                _ => "title: Run the tests",
            }),
            K::FirstKeyNotSteplet => ("`steplet: 1` as the first line of the header", "---\nsteplet: 1\ntitle: Run the tests"),
            K::UnsupportedVersion { .. } => ("format version 1", "steplet: 1"),
            K::ExpectedText { .. } => ("a single line of text", "title: Run the tests"),
            K::ExpectedList { .. } => ("a list, one `- ` item per line, indented two spaces", "steps:\n  - Run the tests."),
            K::ExpectedMap { .. } => ("named fields indented two spaces below", "repository:\n  url: https://github.com/octocat/Hello-World.git\n  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d"),
            K::BadDate => ("a real calendar date written YYYY-MM-DD", "expires: 2026-10-25"),
            K::BadUrl => ("https://, a lowercase host name, then a path of letters, digits and . _ ~ / -; no user name, port, ? or #", "  url: https://github.com/octocat/Hello-World.git"),
            K::BadCommit => ("the full commit hash: 40 (or 64) lowercase hexadecimal characters", "  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d"),
            K::BadDomain => ("an exact lowercase host name with at least one dot; no *, scheme, port or path", "    - crates.io"),
            K::BadFolder => ("a folder inside the repository written with /, no .. and no leading or trailing /; . means all of it", "    - target/debug"),
        };
        Help { allowed, example }
    }
}

pub const TOP_KEYS: &[&str] = &["steplet", "title", "sender", "expires", "repository", "steps", "allow", "never"];
pub const REPOSITORY_KEYS: &[&str] = &["url", "commit"];
pub const ALLOW_KEYS: &[&str] = &["commands", "domains", "read", "write"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Steplet {
    pub title: Scalar,
    pub sender: Scalar,
    pub expires: Date,
    pub expires_line: u32,
    pub repository: Option<Repository>,
    pub steps: Vec<Scalar>,
    pub allow: Allow,
    pub never: Vec<Scalar>,
    /// The cleaned-up, signed body: what the agent is given.
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    pub url: Scalar,
    pub commit: Scalar,
}

/// What the file asks to be allowed. Commands are kept as written; their meaning is phase 2's (R8).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Allow {
    pub commands: Vec<Scalar>,
    pub domains: Vec<Scalar>,
    pub read: Vec<Scalar>,
    pub write: Vec<Scalar>,
}

/// Reads the header of a cleaned-up file, once, from the signed bytes (row 15).
pub fn parse(c: &Canonical) -> Result<Steplet, ParseError> {
    let entries = tree(c.header(), Canonical::HEADER_FIRST_LINE)?;
    let first = entries.first().ok_or(err(1, K::MissingKey { key: "steplet" }))?;
    if first.key != "steplet" {
        return Err(err(first.line, K::FirstKeyNotSteplet));
    }
    let version = text(first)?;
    if version.text != "1" {
        return Err(err(version.line, K::UnsupportedVersion { found: version.text }));
    }
    let top = Fields::new(&entries, TOP_KEYS)?;
    let expires = text(top.required("expires", 1)?)?;
    let repository = match top.optional("repository") {
        None => None,
        Some(e) => {
            let f = Fields::new(map_of(e)?, REPOSITORY_KEYS)?;
            let url = text(f.required("url", e.line)?)?;
            if !is_https_url(&url.text) {
                return Err(err(url.line, K::BadUrl));
            }
            let commit = text(f.required("commit", e.line)?)?;
            if !is_commit(&commit.text) {
                return Err(err(commit.line, K::BadCommit));
            }
            Some(Repository { url, commit })
        }
    };
    let allow = match top.optional("allow") {
        None => Allow::default(),
        Some(e) => {
            let f = Fields::new(map_of(e)?, ALLOW_KEYS)?;
            Allow {
                commands: f.list("commands")?,
                domains: checked(f.list("domains")?, is_domain, K::BadDomain)?,
                read: checked(f.list("read")?, is_folder, K::BadFolder)?,
                write: checked(f.list("write")?, is_folder, K::BadFolder)?,
            }
        }
    };
    Ok(Steplet {
        title: text(top.required("title", 1)?)?,
        sender: text(top.required("sender", 1)?)?,
        expires: Date::parse(&expires.text).ok_or(err(expires.line, K::BadDate))?,
        expires_line: expires.line,
        repository,
        steps: list_of(top.required("steps", 1)?)?,
        allow,
        never: top.list("never")?,
        body: c.body().to_string(),
    })
}

struct Fields<'a> {
    entries: &'a [Entry],
}

impl<'a> Fields<'a> {
    fn new(entries: &'a [Entry], allowed: &'static [&'static str]) -> Result<Self, ParseError> {
        if let Some(e) = entries.iter().find(|e| !allowed.contains(&e.key.as_str())) {
            return Err(err(e.line, K::UnknownKey { key: e.key.clone(), allowed }));
        }
        Ok(Fields { entries })
    }
    fn optional(&self, key: &str) -> Option<&'a Entry> {
        self.entries.iter().find(|e| e.key == key)
    }
    fn required(&self, key: &'static str, parent_line: u32) -> Result<&'a Entry, ParseError> {
        self.optional(key).ok_or(err(parent_line, K::MissingKey { key }))
    }
    fn list(&self, key: &str) -> Result<Vec<Scalar>, ParseError> {
        self.optional(key).map_or(Ok(Vec::new()), list_of)
    }
}

fn text(e: &Entry) -> Result<Scalar, ParseError> {
    match &e.value {
        Node::Scalar(s) if s.text.is_empty() => Err(err(e.line, K::EmptyValue { key: e.key.clone() })),
        Node::Scalar(s) => Ok(s.clone()),
        _ => Err(err(e.line, K::ExpectedText { key: e.key.clone() })),
    }
}

fn list_of(e: &Entry) -> Result<Vec<Scalar>, ParseError> {
    match &e.value {
        Node::List(items) => match items.iter().find(|s| s.text.is_empty()) {
            Some(empty) => Err(err(empty.line, K::EmptyListItem)),
            None => Ok(items.clone()),
        },
        _ => Err(err(e.line, K::ExpectedList { key: e.key.clone() })),
    }
}

fn map_of(e: &Entry) -> Result<&[Entry], ParseError> {
    match &e.value {
        Node::Map(m) => Ok(m),
        _ => Err(err(e.line, K::ExpectedMap { key: e.key.clone() })),
    }
}

fn checked(items: Vec<Scalar>, ok: fn(&str) -> bool, kind: ParseErrorKind) -> Result<Vec<Scalar>, ParseError> {
    match items.iter().find(|s| !ok(&s.text)) {
        Some(bad) => Err(err(bad.line, kind)),
        None => Ok(items),
    }
}

fn is_domain(s: &str) -> bool {
    let label_ok = |l: &str| {
        !l.is_empty()
            && l.len() <= 63
            && !l.starts_with('-')
            && !l.ends_with('-')
            && l.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    };
    s.len() <= 253
        && s.split('.').count() >= 2
        && s.split('.').all(label_ok)
        && s.rsplit('.').next().is_some_and(|tld| tld.bytes().any(|b| b.is_ascii_lowercase()))
}

fn is_https_url(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("https://") else { return false };
    let (host, path) = rest.find('/').map_or((rest, ""), |i| (&rest[..i], &rest[i..]));
    is_domain(host) && path.bytes().all(|b| b.is_ascii_alphanumeric() || b"._~/-".contains(&b))
}

fn is_commit(s: &str) -> bool {
    (s.len() == 40 || s.len() == 64) && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn is_folder(s: &str) -> bool {
    s == "."
        || s.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
