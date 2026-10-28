//! Commands (decision R8, G1). A command is read as a list of words by a small, strict part of the shell's rules:
//! plain words, single quotes, and double quotes with only `\"` and `\\` inside. Anything with another meaning to
//! the shell (pipes, `;`, `&&`, `$`, redirections, wildcards, `~`, …) is not read at all, so Steplet never guesses
//! what a shell would do. Commands are then matched word for word. Within these rules `bash` makes exactly the same
//! words (tests/vectors/commands.txt).

use alloc::string::String;
use alloc::vec::Vec;

use crate::display::is_invisible;
use crate::header::is_folder;
use crate::paths::{self, Secret};

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

/// What a command does, as far as Steplet can tell from its words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Only reads files and prints: G2's harmless list.
    ReadOnly,
    /// Runs the repository's own code: tests, builds, scripts.
    RunsCode,
    /// Downloads and installs packages, which usually run code of their own.
    Installs,
    /// A program Steplet does not know, or a use of a known one it cannot place.
    Unknown,
}

pub fn kind(words: &[String]) -> Kind {
    use Kind::*;
    let Some((program, args)) = words.split_first() else { return Unknown };
    let program = program.as_str();
    let sub = args.iter().map(String::as_str).find(|a| !a.starts_with('-') && !a.starts_with('+')).unwrap_or("");
    let has_any = |names: &[&str]| args.iter().any(|a| names.contains(&a.as_str()));
    match program {
        "ls" | "cat" | "head" | "tail" | "wc" | "grep" | "pwd" | "echo" | "stat" | "du" | "diff" | "cmp"
        | "sha256sum" | "sha1sum" | "md5sum" | "basename" | "dirname" | "realpath" | "cut" | "nl" | "true"
        | "false" => ReadOnly,
        "find"
            if !has_any(&[
                "-exec", "-execdir", "-ok", "-okdir", "-delete", "-fprint", "-fprint0", "-fprintf", "-fls",
            ]) =>
        {
            ReadOnly
        }
        "git" if git_reads_only(args) => ReadOnly,
        "pip" | "pip3" => pip(sub),
        "python" | "python3" | "py" => match args {
            [m, p, rest @ ..] if m == "-m" && (p == "pip" || p == "pip3") => {
                pip(rest.iter().map(String::as_str).find(|a| !a.starts_with('-')).unwrap_or(""))
            }
            _ => RunsCode,
        },
        "cargo" => match sub {
            "" => Unknown,
            "install" | "fetch" | "update" | "add" | "generate-lockfile" | "vendor" => Installs,
            _ => RunsCode,
        },
        "npm" | "pnpm" | "yarn" | "bun" => match sub {
            "" if program == "yarn" => Installs,
            "install" | "i" | "ci" | "add" | "update" | "up" | "upgrade" | "exec" | "x" | "dlx" | "create" => Installs,
            "test" | "t" | "run" | "run-script" | "start" | "build" => RunsCode,
            _ => Unknown,
        },
        "npx" | "pnpx" | "bunx" => Installs,
        "uv" => match sub {
            "run" => RunsCode,
            "sync" | "add" | "pip" | "lock" | "tool" => Installs,
            _ => Unknown,
        },
        "go" => match sub {
            "" => Unknown,
            "get" | "install" | "mod" => Installs,
            _ => RunsCode,
        },
        "dotnet" => match sub {
            "" => Unknown,
            "restore" | "add" | "tool" => Installs,
            _ => RunsCode,
        },
        "bundle" => match sub {
            "install" | "update" | "add" => Installs,
            "exec" => RunsCode,
            _ => Unknown,
        },
        "gem" | "composer" => match sub {
            "install" | "update" | "require" => Installs,
            _ => Unknown,
        },
        "node" | "deno" | "pytest" | "tox" | "nox" | "make" | "cmake" | "ctest" | "ninja" | "meson" | "mvn"
        | "gradle" | "ant" | "rake" | "ruby" | "rspec" | "php" | "phpunit" | "java" | "javac" | "rustc" | "gcc"
        | "g++" | "clang" | "cc" | "bash" | "sh" | "zsh" | "dash" | "pwsh" | "jest" | "vitest" | "mocha" => RunsCode,
        // A script in the repository, run by its path: ./gradlew, ./scripts/test.sh
        _ if program.strip_prefix("./").is_some_and(|rest| rest != "." && is_folder(rest)) => RunsCode,
        _ => Unknown,
    }
}

fn pip(sub: &str) -> Kind {
    match sub {
        "list" | "show" | "freeze" | "check" => Kind::ReadOnly,
        "install" | "download" | "wheel" => Kind::Installs,
        _ => Kind::Unknown,
    }
}

/// `git` commands that only read, without the options that write a file or start another program.
fn git_reads_only(args: &[String]) -> bool {
    const READS: &[&str] = &[
        "status",
        "log",
        "diff",
        "show",
        "rev-parse",
        "ls-files",
        "blame",
        "describe",
        "shortlog",
        "cat-file",
        "ls-tree",
    ];
    args.first().is_some_and(|s| READS.contains(&s.as_str()))
        && !args.iter().any(|a| a == "--ext-diff" || a == "--output" || a.starts_with("--output="))
}

/// Things a command must never do, whatever the file says (R9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Danger {
    /// sudo and the like: administrator rights.
    Admin,
    /// Points a package manager at another registry or index.
    Registry,
}

pub fn danger(words: &[String]) -> Option<Danger> {
    const PIP: &[&str] = &["-i", "--index-url", "--extra-index-url", "-f", "--find-links", "--trusted-host"];
    const UV: &[&str] = &["--index", "--index-url", "--extra-index-url", "--default-index", "--find-links"];
    const GO: &[&str] =
        &["GOPROXY=", "GOSUMDB=", "GONOSUMDB=", "GONOSUMCHECK=", "GOINSECURE=", "GOPRIVATE=", "GOFLAGS="];
    let (program, args) = words.split_first()?;
    let program = program.as_str();
    if matches!(program, "sudo" | "su" | "doas" | "pkexec" | "runas" | "gsudo") {
        return Some(Danger::Admin);
    }
    // Words after a lone `--` belong to the program being run (a test filter, a script's own options).
    let args = &args[..args.iter().position(|a| a == "--").unwrap_or(args.len())];
    let word_is = |i: usize, w: &str| args.get(i).is_some_and(|a| a == w);
    let registry = match program {
        "pip" | "pip3" => has_option(args, PIP) || (word_is(0, "config") && word_is(1, "set")),
        "python" | "python3" | "py" => {
            word_is(0, "-m")
                && (word_is(1, "pip") || word_is(1, "pip3"))
                && (has_option(&args[2..], PIP) || (word_is(2, "config") && word_is(3, "set")))
        }
        "uv" => has_option(args, UV),
        "npm" | "pnpm" | "yarn" | "bun" | "npx" | "pnpx" | "bunx" => {
            has_option(args, &["--registry"])
                || (word_is(0, "config") && args.iter().any(|a| a.to_ascii_lowercase().contains("registry")))
        }
        "cargo" => has_option(args, &["--config", "--index", "--registry"]),
        "go" => word_is(0, "env") && args.iter().any(|a| GO.iter().any(|g| a.starts_with(g))),
        _ => false,
    };
    registry.then_some(Danger::Registry)
}

/// True when one of `args` is one of the options `names`: alone, as `--name=value`, or, for a one-letter option
/// such as `-i`, with its value joined on (`-ihttps://…`).
fn has_option(args: &[String], names: &[&str]) -> bool {
    args.iter().any(|a| {
        names.iter().any(|n| {
            a == n
                || if n.starts_with("--") {
                    a.strip_prefix(n).is_some_and(|rest| rest.starts_with('='))
                } else {
                    n.len() == 2 && a.starts_with(n)
                }
        })
    })
}

const DOWNLOADERS: &[&str] = &["curl", "wget", "iwr", "irm", "invoke-webrequest", "invoke-restmethod", "aria2c"];
const SHELLS: &[&str] =
    &["sh", "bash", "zsh", "dash", "ksh", "fish", "pwsh", "powershell", "iex", "invoke-expression", "source"];
/// Script interpreters count only when they read the script from the pipe (`| python`, `| python -`).
const INTERPRETERS: &[&str] = &["python", "python3", "perl", "ruby", "node"];

fn shell_words(s: &str) -> impl Iterator<Item = &str> {
    s.split(|c: char| c.is_whitespace() || "|;&()<>$`\"'{}".contains(c)).filter(|w| !w.is_empty())
}

/// True when a line of `text` downloads something and runs it straight away with a shell or a script interpreter
/// (row 25): `curl … | sh`, `wget -O- … | sudo bash`, `bash <(curl …)`, `sh -c "$(curl …)"`, `irm … | iex`.
/// Piping a download into another program (`curl … | jq .`) is not this.
pub fn downloads_into_shell(text: &str) -> bool {
    text.lines().any(|line| {
        let l = line.to_ascii_lowercase();
        if !shell_words(&l).any(|w| DOWNLOADERS.contains(&w) || w.contains("downloadstring")) {
            return false;
        }
        let piped = l.split('|').skip(1).any(|part| {
            let mut w = shell_words(part).skip_while(|w| matches!(*w, "sudo" | "-e" | "-h"));
            match w.next() {
                Some(s) if SHELLS.contains(&s) => true,
                Some(s) if INTERPRETERS.contains(&s) => matches!(w.next(), None | Some("-")),
                _ => false,
            }
        });
        let substituted = (l.contains("<(") || l.contains("$("))
            && shell_words(&l).any(|w| SHELLS.contains(&w) || INTERPRETERS.contains(&w));
        let iex = shell_words(&l).any(|w| w == "iex" || w == "invoke-expression");
        piped || substituted || iex
    })
}

/// Why a command can never be allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blocked {
    /// An invisible character (G7): what shows is not what runs.
    Invisible,
    /// A download run straight away by a shell (row 25).
    DownloadIntoShell,
    /// A place on the always-blocked list, outside the repository (row 24, G8).
    Secret(&'static Secret),
    Admin,
    Registry,
}

/// What Steplet makes of one command. The rating (the file's own commands) and the checker (the agent's commands
/// during a run) both use this, so they cannot disagree (row 29) — with one exception: during a run the working
/// copy is known, so a full path into it counts as inside (`assess_in`), while the rating, which knows no copy yet,
/// still counts it as outside. The rating is the stricter of the two.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Assessment {
    Blocked(Blocked),
    NotUnderstood(NotUnderstood),
    /// `outside`: a word after the program is a path outside the repository, or `--option=` such a path.
    Words {
        words: Vec<String>,
        kind: Kind,
        outside: bool,
    },
}

pub fn assess(text: &str) -> Assessment {
    assess_with(text, None)
}

/// As `assess`, for a command proposed during a run in the working copy `work` (an absolute path, as the checker's):
/// a full path to the working copy or into it is inside, as its short form is. A model writes the copy's full path
/// (phase 5 wrap-up: `ls -la /work/p5l1/repo` was its first move in both paid live runs).
pub fn assess_in(text: &str, work: &str) -> Assessment {
    debug_assert!(work != "/" && paths::normalize(work).as_deref() == Some(work), "not a working copy: {work}");
    assess_with(text, Some(work))
}

fn assess_with(text: &str, work: Option<&str>) -> Assessment {
    if text.chars().any(is_invisible) {
        return Assessment::Blocked(Blocked::Invisible);
    }
    if downloads_into_shell(text) {
        return Assessment::Blocked(Blocked::DownloadIntoShell);
    }
    if let Some(secret) = paths::secret_in(text) {
        return Assessment::Blocked(Blocked::Secret(secret));
    }
    let words = match split(text) {
        Ok(words) => words,
        Err(why) => return Assessment::NotUnderstood(why),
    };
    match danger(&words) {
        Some(Danger::Admin) => return Assessment::Blocked(Blocked::Admin),
        Some(Danger::Registry) => return Assessment::Blocked(Blocked::Registry),
        None => {}
    }
    // A path can hide after `=` (`--file=/etc/x`) or be joined to a short option (`-f/etc/x`, `-I~/x`).
    // A full path counts as inside only when it is the working copy or below it, with no `..` (a part before it
    // could be a link out of the copy, as for a short path) and no `:` or `,` (which can join a second path).
    let is_outside = |path: &str| {
        paths::is_outside(path)
            && !work.is_some_and(|work| {
                !path.contains([':', ',', '\\'])
                    && !path.split('/').any(|p| p == "..")
                    && paths::normalize(path).is_some_and(|p| paths::below(work, &p).is_some())
            })
    };
    let outside = words[1..].iter().any(|w| {
        is_outside(w)
            || w.split_once('=').is_some_and(|(_, value)| is_outside(value))
            || (w.starts_with('-') && w.find(['/', '\\', '~']).is_some_and(|i| is_outside(&w[i..])))
    });
    let k = kind(&words);
    Assessment::Words { words, kind: k, outside }
}
