//! Commands (decision R8, G1). A command is read as a list of words by a small, strict part of the shell's rules:
//! plain words, single quotes, and double quotes with only `\"` and `\\` inside. Anything with another meaning to
//! the shell (pipes, `;`, `&&`, `$`, redirections, wildcards, `~`, …) is not read at all, so Steplet never guesses
//! what a shell would do. Commands are then matched word for word. Within these rules `bash` makes exactly the same
//! words (tests/vectors/commands.txt).

use alloc::string::String;
use alloc::vec::Vec;

/// Why Steplet cannot read a command as a list of words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotUnderstood {
    Empty,
    /// A character with a special meaning to the shell, or one outside printable ASCII.
    Symbol(char),
    /// A tab, a line break or another control character.
    Control,
    UnclosedQuote,
    /// The first word sets a variable (`NAME=value program`).
    Assignment,
}

fn plain(c: char) -> bool {
    c.is_ascii_alphanumeric() || "_-./,:@%+=".contains(c)
}

fn refuse(c: char) -> NotUnderstood {
    if c.is_control() { NotUnderstood::Control } else { NotUnderstood::Symbol(c) }
}

pub fn split(line: &str) -> Result<Vec<String>, NotUnderstood> {
    use NotUnderstood::*;
    let mut words = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            ' ' => {
                if in_word {
                    words.push(core::mem::take(&mut word));
                    in_word = false;
                }
            }
            '\'' => {
                in_word = true;
                loop {
                    match chars.next() {
                        None => return Err(UnclosedQuote),
                        Some('\'') => break,
                        Some(q) if (' '..='~').contains(&q) => word.push(q),
                        Some(q) => return Err(refuse(q)),
                    }
                }
            }
            '"' => {
                in_word = true;
                loop {
                    match chars.next() {
                        None => return Err(UnclosedQuote),
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some(e @ ('"' | '\\')) => word.push(e),
                            Some(_) => return Err(Symbol('\\')),
                            None => return Err(UnclosedQuote),
                        },
                        Some(q @ ('$' | '`' | '!')) => return Err(Symbol(q)),
                        Some(q) if (' '..='~').contains(&q) => word.push(q),
                        Some(q) => return Err(refuse(q)),
                    }
                }
            }
            c if plain(c) => {
                in_word = true;
                word.push(c);
            }
            c => return Err(refuse(c)),
        }
    }
    if in_word {
        words.push(word);
    }
    match words.first() {
        None => Err(Empty),
        Some(first) if first.contains('=') => Err(Assignment),
        Some(_) => Ok(words),
    }
}
