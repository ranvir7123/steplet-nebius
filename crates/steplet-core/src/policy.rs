//! The one checker (decision row 29): every action an agent proposes during a run is decided here, with the same
//! reading of commands as the rating (`command::assess`; during a run `command::assess_in`, which also knows a full
//! path into the working copy is inside it, and reads a chain piece by piece). Anything the file did not ask for goes
//! to the receiver
//! (R7); the always-blocked list and protected folders are refused without a question (row 24, G8, G14).

use alloc::string::String;
use alloc::vec::Vec;

use crate::command::{self, Assessment, Blocked, Proposed};
use crate::header::{Scalar, Steplet};
use crate::hosts;
use crate::paths;
use crate::rules;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action<'a> {
    /// A shell command, exactly as the agent proposed it.
    Command(&'a str),
    /// Reading a file. The path is absolute, written with `/`, after the runner resolved links (row 30, R19).
    Read(&'a str),
    /// Writing a file, path as for `Read`.
    Write(&'a str),
    /// Fetching a web address.
    Fetch(&'a str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Allow,
    /// The receiver decides, on the ask screen (R7). `reason` is in Steplet's own words.
    Ask {
        reason: &'static str,
    },
    /// Refused without a question. `rule` is the id of the rating rule with the same meaning.
    Deny {
        rule: &'static str,
        reason: &'static str,
    },
}

/// The working copy's path was not a clean absolute path (G13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadWorkFolder;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    work: String,
    commands: Vec<Vec<String>>,
    domains: Vec<String>,
    read: Vec<String>,
    write: Vec<String>,
}

impl Policy {
    /// The checker for one run of `s` in the working copy `work`, an absolute Linux path with no `.`, `..`,
    /// repeated or trailing `/` (G13).
    pub fn from_steplet(s: &Steplet, work: &str) -> Result<Policy, BadWorkFolder> {
        if work == "/" || paths::normalize(work).as_deref() != Some(work) {
            return Err(BadWorkFolder);
        }
        let commands = s
            .allow
            .commands
            .iter()
            .filter_map(|c| match command::assess(&c.text) {
                Assessment::Words { words, .. } => Some(words),
                _ => None,
            })
            .collect();
        let texts = |items: &[Scalar]| -> Vec<String> { items.iter().map(|x| x.text.clone()).collect() };
        Ok(Policy {
            work: work.into(),
            commands,
            domains: texts(&s.allow.domains),
            read: texts(&s.allow.read),
            write: texts(&s.allow.write),
        })
    }

    pub fn check(&self, action: &Action) -> Decision {
        match *action {
            Action::Command(line) => self.command(line),
            Action::Read(path) => self.path(path, false),
            Action::Write(path) => self.path(path, true),
            Action::Fetch(url) => match hosts::https_host(url) {
                Some(host) if self.domains.contains(&host) => Decision::Allow,
                Some(_) => ask("the file does not ask for this site"),
                None => ask("Steplet allows only plain https addresses, without a user name or a port"),
            },
        }
    }

    /// A chain is allowed when each of its commands would be alone, except that the file's declared command never
    /// reads what another piece prints.
    fn command(&self, line: &str) -> Decision {
        match command::assess_in(line, &self.work) {
            Proposed::Blocked(why) => refuse(why),
            Proposed::NotUnderstood(_) => {
                ask("Steplet cannot read this command exactly, so it cannot match it with the file")
            }
            Proposed::Pieces(pieces) => {
                if pieces.iter().all(|p| (!p.wildcard && !p.piped && self.commands.contains(&p.words)) || p.harmless())
                {
                    Decision::Allow
                } else {
                    ask("the file does not ask for this command")
                }
            }
        }
    }

    /// A file the agent's own tools read or write. Those tools run outside the fence, so a write outside the working
    /// copy is refused without a question (J1): nothing of the receiver's is out there, and a file left there could
    /// change the next run (a settings file in the run's own folder could load hooks). Reads outside still ask. The
    /// path is read as text: the runner gives the place a link leads to, not the link (row 30; `steplet_run::Checker`,
    /// since milestone 1 phase 5 Task 3).
    fn path(&self, path: &str, write: bool) -> Decision {
        let outside =
            || deny(rules::CAP_OUTSIDE_PATH, "the agent's file tools may change files only inside the working copy");
        let Some(p) = paths::normalize(path) else {
            return if write { outside() } else { ask("this is not a plain absolute path") };
        };
        match paths::below(&self.work, &p) {
            Some(rel) if write && paths::is_protected(rel) => {
                deny(rules::BLOCK_WRITE_PROTECTED, "no task may change .git or the agents' settings")
            }
            Some(rel) => {
                let folders = if write { &self.write } else { &self.read };
                if (!write && folders.is_empty()) || folders.iter().any(|f| paths::within(f, rel)) {
                    Decision::Allow
                } else if write {
                    ask("the file does not ask to change files here")
                } else {
                    ask("the file does not ask to read files here")
                }
            }
            None => match paths::secret_of(&p) {
                Some(_) => deny(
                    rules::BLOCK_SECRET_PATH,
                    "this place holds keys, passwords or sign-ins, which no task may touch",
                ),
                None if write => outside(),
                None => ask("this path is outside the repository"),
            },
        }
    }
}

fn ask(reason: &'static str) -> Decision {
    Decision::Ask { reason }
}

fn deny(rule: &'static rules::Rule, reason: &'static str) -> Decision {
    Decision::Deny { rule: rule.id, reason }
}

fn refuse(why: Blocked) -> Decision {
    match why {
        Blocked::Invisible => deny(rules::BLOCK_INVISIBLE_IN_FIELD, "the command has invisible characters"),
        Blocked::DownloadIntoShell => {
            deny(rules::BLOCK_DOWNLOAD_INTO_SHELL, "downloading a script and running it straight away is never allowed")
        }
        Blocked::Secret(_) => {
            deny(rules::BLOCK_SECRET_PATH, "the command names a place that holds keys, passwords or sign-ins")
        }
        Blocked::Admin => deny(rules::BLOCK_ADMIN, "a task never gets administrator rights"),
        Blocked::Registry => deny(rules::BLOCK_REGISTRY, "a task may not change where packages come from"),
    }
}
