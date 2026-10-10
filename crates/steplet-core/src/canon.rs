//! The fixed clean-up (decision row 14) and the split into header, signature block and body (R13).
//! Signed bytes are the cleaned-up text minus the signature block lines.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

use crate::Help;
use crate::limits::{MAX_FILE_BYTES, MAX_HEADER_BYTES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canonical {
    text: String,
    signed: String,
    /// Into `signed`: the lines between the two `---` lines.
    header: Range<usize>,
    /// Into `signed`: everything after the closing `---` line.
    body: Range<usize>,
    signature: Option<SignatureBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureBlock {
    /// The armoured signature with the two-space indent removed, a `\n` after each line.
    pub pem: String,
    /// Line numbers in the cleaned-up text of the `signature: |` line and of the END line.
    pub first_line: u32,
    pub last_line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonError {
    FileTooLarge { bytes: usize },
    NotUtf8 { line: u32 },
    LoneCarriageReturn { line: u32 },
    NoHeaderStart,
    NoHeaderEnd,
    HeaderTooLarge { bytes: usize },
    SignatureMisplaced { line: u32 },
    SignatureTwice { line: u32 },
    SignatureBroken { line: u32 },
}

impl Canonical {
    /// Line number of the header's first line (the line after the opening `---`).
    pub const HEADER_FIRST_LINE: u32 = 2;

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn signed(&self) -> &str {
        &self.signed
    }
    pub fn header(&self) -> &str {
        &self.signed[self.header.clone()]
    }
    pub fn body(&self) -> &str {
        &self.signed[self.body.clone()]
    }
    pub fn signature(&self) -> Option<&SignatureBlock> {
        self.signature.as_ref()
    }
}

pub fn canonicalize(raw: &[u8]) -> Result<Canonical, CanonError> {
    if raw.len() > MAX_FILE_BYTES {
        return Err(CanonError::FileTooLarge { bytes: raw.len() });
    }
    let s = core::str::from_utf8(raw)
        .map_err(|e| CanonError::NotUtf8 { line: line_of(&raw[..e.valid_up_to()]) })?;
    let s = s.strip_prefix('\u{feff}').unwrap_or(s);
    let mut lines: Vec<&str> = Vec::new();
    for (i, piece) in s.split('\n').enumerate() {
        let piece = piece.strip_suffix('\r').unwrap_or(piece);
        if piece.contains('\r') {
            return Err(CanonError::LoneCarriageReturn { line: i as u32 + 1 });
        }
        lines.push(piece.trim_end_matches([' ', '\t']));
    }
    while lines.last() == Some(&"") {
        lines.pop();
    }
    split(&lines)
}

fn split(lines: &[&str]) -> Result<Canonical, CanonError> {
    if lines.first() != Some(&"---") {
        return Err(CanonError::NoHeaderStart);
    }
    let close = lines.iter().skip(1).position(|l| *l == "---").ok_or(CanonError::NoHeaderEnd)? + 1;
    let block = signature_block(lines, close)?;

    let mut text = String::new();
    let mut signed = String::new();
    let (mut header, mut body) = (0..0, 0..0);
    for (i, line) in lines.iter().enumerate() {
        text.push_str(line);
        text.push('\n');
        if block.as_ref().is_some_and(|(k, e, _)| (*k..=*e).contains(&i)) {
            continue;
        }
        if i == close {
            header.end = signed.len();
        }
        signed.push_str(line);
        signed.push('\n');
        if i == 0 {
            header.start = signed.len();
        }
        if i == close {
            body.start = signed.len();
        }
    }
    body.end = signed.len();
    if header.len() > MAX_HEADER_BYTES {
        return Err(CanonError::HeaderTooLarge { bytes: header.len() });
    }
    let signature = block.map(|(k, e, pem)| SignatureBlock { pem, first_line: k as u32 + 1, last_line: e as u32 + 1 });
    Ok(Canonical { text, signed, header, body, signature })
}

/// Task 3 replaces this with the real search. For now: no signature block.
fn signature_block(_lines: &[&str], _close: usize) -> Result<Option<(usize, usize, String)>, CanonError> {
    Ok(None)
}

fn line_of(before: &[u8]) -> u32 {
    before.iter().filter(|b| **b == b'\n').count() as u32 + 1
}

impl fmt::Display for CanonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonError::FileTooLarge { bytes } => {
                write!(f, "the file is {bytes} bytes; at most {MAX_FILE_BYTES} bytes (256 KiB) are allowed")
            }
            CanonError::NotUtf8 { line } => write!(f, "line {line}: the text is not valid UTF-8"),
            CanonError::LoneCarriageReturn { line } => {
                write!(f, "line {line}: a carriage return (\\r) that is not part of a Windows line ending")
            }
            CanonError::NoHeaderStart => write!(f, "line 1: the file must start with a line holding only `---`"),
            CanonError::NoHeaderEnd => write!(f, "the header has no closing `---` line"),
            CanonError::HeaderTooLarge { bytes } => {
                write!(f, "the header is {bytes} bytes; at most {MAX_HEADER_BYTES} bytes (32 KiB) are allowed")
            }
            CanonError::SignatureMisplaced { line } => write!(
                f,
                "line {line}: a signature block in the wrong place; it must be the last thing in the header"
            ),
            CanonError::SignatureTwice { line } => write!(f, "line {line}: a second signature block"),
            CanonError::SignatureBroken { line } => write!(f, "line {line}: the signature block is damaged"),
        }
    }
}

const RESIGN: &str = "steplet-sign sign my-task.steplet --key C:\\Users\\you\\.ssh\\id_ed25519";

impl CanonError {
    pub fn help(&self) -> Help {
        let (allowed, example) = match self {
            CanonError::FileTooLarge { .. } => ("a file of at most 256 KiB; split a bigger task into several files", ""),
            CanonError::NotUtf8 { .. } => ("text saved as UTF-8 (most editors call it \"UTF-8\")", ""),
            CanonError::LoneCarriageReturn { .. } => ("Windows (\\r\\n) or Unix (\\n) line endings only", ""),
            CanonError::NoHeaderStart => (
                "`---` alone on the first line, then the header",
                "---\nsteplet: 1\ntitle: Run the tests",
            ),
            CanonError::NoHeaderEnd => (
                "a line holding only `---` after the header, then the body",
                "never:\n  - Do not push.\n---\n# Instructions",
            ),
            CanonError::HeaderTooLarge { .. } => ("a header of at most 32 KiB; move long text into the body", ""),
            CanonError::SignatureMisplaced { .. }
            | CanonError::SignatureTwice { .. }
            | CanonError::SignatureBroken { .. } => (
                "one signature block, made by steplet-sign, as the last thing in the header; delete the damaged lines and sign again",
                RESIGN,
            ),
        };
        Help { allowed, example }
    }
}
