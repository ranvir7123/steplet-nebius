//! Size limits (decision row 12). Starting values, to tune on the test corpus in phase 2. Over a limit is red.

use alloc::vec::Vec;
use core::fmt;

use crate::Help;
use crate::header::{Scalar, Steplet};

pub const MAX_FILE_BYTES: usize = 256 * 1024;
pub const MAX_HEADER_BYTES: usize = 32 * 1024;
pub const MAX_COMMANDS: usize = 25;
pub const MAX_DOMAINS: usize = 15;
pub const MAX_FOLDERS_PER_LIST: usize = 10;
pub const MAX_STEPS: usize = 30;
/// "Any command under 300 characters": 299 is the longest allowed. Characters are Unicode scalar values.
pub const MAX_COMMAND_CHARS: usize = 299;
/// Not in row 12. A starting value for the `never` list, equal to the command limit.
pub const MAX_NEVER: usize = 25;
/// The furthest an `expires` date may be from the day of signing (row 19).
pub const MAX_EXPIRY_DAYS: i64 = 365;
/// The expiry `steplet-sign` suggests (row 19).
pub const DEFAULT_EXPIRY_DAYS: i64 = 14;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limited {
    Steps,
    Commands,
    Domains,
    ReadFolders,
    WriteFolders,
    Never,
    CommandLength,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LimitBreach {
    pub what: Limited,
    /// For a list: the line of the first item over the limit. For a command: its line.
    pub line: u32,
    pub found: usize,
    pub max: usize,
}

pub fn check(s: &Steplet) -> Vec<LimitBreach> {
    let mut out = Vec::new();
    let mut count = |what, items: &[Scalar], max: usize| {
        if items.len() > max {
            out.push(LimitBreach { what, line: items[max].line, found: items.len(), max });
        }
    };
    count(Limited::Steps, &s.steps, MAX_STEPS);
    count(Limited::Commands, &s.allow.commands, MAX_COMMANDS);
    count(Limited::Domains, &s.allow.domains, MAX_DOMAINS);
    count(Limited::ReadFolders, &s.allow.read, MAX_FOLDERS_PER_LIST);
    count(Limited::WriteFolders, &s.allow.write, MAX_FOLDERS_PER_LIST);
    count(Limited::Never, &s.never, MAX_NEVER);
    for c in &s.allow.commands {
        let n = c.text.chars().count();
        if n > MAX_COMMAND_CHARS {
            out.push(LimitBreach { what: Limited::CommandLength, line: c.line, found: n, max: MAX_COMMAND_CHARS });
        }
    }
    out
}

impl fmt::Display for LimitBreach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, found, max) = (self.line, self.found, self.max);
        let what = match self.what {
            Limited::Steps => "steps",
            Limited::Commands => "commands",
            Limited::Domains => "domains",
            Limited::ReadFolders => "read folders",
            Limited::WriteFolders => "write folders",
            Limited::Never => "never lines",
            Limited::CommandLength => {
                return write!(f, "line {line}: this command is {found} characters long; it must be under {}", max + 1);
            }
        };
        write!(f, "line {line}: {found} {what}; at most {max} are allowed")
    }
}

impl LimitBreach {
    pub fn help(&self) -> Help {
        let allowed = match self.what {
            Limited::CommandLength => "a shorter command; put long scripts in the repository and run them by name",
            _ => "a shorter list; everything must fit on the receiver's screen, so split a bigger task into several files",
        };
        Help { allowed, example: "" }
    }
}
