//! Commands (decision R8, G1). A command is read as a list of words by a small, strict part of the shell's rules:
//! plain words, single quotes, and double quotes with only `\"` and `\\` inside. Anything with another meaning to
//! the shell (pipes, `;`, `&&`, `$`, redirections, wildcards, `~`, …) is not read at all, so Steplet never guesses
//! what a shell would do. Commands are then matched word for word. Within these rules `bash` makes exactly the same
//! words (tests/vectors/commands.txt). During a run only, a chain of commands is also read, piece by piece, with a
//! few more rules (`assess_in`, phase 5 wrap-up); a file's own commands are still read one at a time.

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
    let mut pieces = read(line, false)?;
    debug_assert_eq!(pieces.len(), 1);
    Ok(pieces.swap_remove(0).words)
}

/// One command of a chain, as read: its words, and whether one of them has a wildcard the shell will expand.
#[derive(Default)]
struct Read {
    words: Vec<String>,
    wildcard: bool,
    piped: bool,
}

/// The only redirections a chain may have, each as a word of its own: they throw away or merge error messages.
const REDIRECTIONS: &[&str] = &["2>/dev/null", "2>&1"];

/// Reads `line` by `split`'s rules. During a run (`chain`) it also reads, outside quotes: commands joined by `;`,
/// `&&`, `||` or `|`; the `REDIRECTIONS`, which are dropped; and the wildcards `*` and `?`, in a word that is not
/// the program, does not start with `-`, `*` or `?`, and has no part starting with `.` other than `.` itself. So a
/// wildcard can neither name the program nor become an option, whatever files the repository holds, and it never
/// reaches a hidden name such as `..`.
fn read(line: &str, chain: bool) -> Result<Vec<Read>, NotUnderstood> {
    use NotUnderstood::*;
    let mut pieces = Vec::new();
    let mut piece = Read::default();
    let mut word = String::new();
    let mut in_word = false;
    let mut wildcard = None;
    let mut chars = line.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if chain && !in_word && c == '2' {
            let rest = &line[i..];
            let redirection = REDIRECTIONS.iter().find(|r| {
                rest.starts_with(**r) && matches!(rest[r.len()..].chars().next(), None | Some(' ' | ';' | '|' | '&'))
            });
            if let Some(r) = redirection {
                for _ in 1..r.len() {
                    chars.next();
                }
                continue;
            }
        }
        match c {
            ' ' => {
                if in_word {
                    end_word(&mut piece, core::mem::take(&mut word), wildcard.take())?;
                    in_word = false;
                }
            }
            '\'' => {
                in_word = true;
                loop {
                    match chars.next().map(|(_, q)| q) {
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
                    match chars.next().map(|(_, q)| q) {
                        None => return Err(UnclosedQuote),
                        Some('"') => break,
                        Some('\\') => match chars.next().map(|(_, e)| e) {
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
            '*' | '?' if chain => {
                in_word = true;
                wildcard.get_or_insert(c);
                word.push(c);
            }
            ';' | '|' | '&' if chain => {
                if in_word {
                    end_word(&mut piece, core::mem::take(&mut word), wildcard.take())?;
                    in_word = false;
                }
                let pipe = match (c, chars.peek().map(|&(_, n)| n)) {
                    ('|', Some('|')) | ('&', Some('&')) => {
                        chars.next();
                        false
                    }
                    // `|&` also pipes error messages; a lone `&` runs a command in the background.
                    ('|', Some('&')) | ('&', _) => return Err(Symbol('&')),
                    (c, _) => c == '|',
                };
                end_piece(&mut pieces, core::mem::take(&mut piece))?;
                piece.piped = pipe;
            }
            c if plain(c) => {
                in_word = true;
                word.push(c);
            }
            c => return Err(refuse(c)),
        }
    }
    if in_word {
        end_word(&mut piece, word, wildcard)?;
    }
    end_piece(&mut pieces, piece)?;
    Ok(pieces)
}

/// Adds `word` to `piece`. `wildcard` is the first wildcard written outside quotes in it, if any.
fn end_word(piece: &mut Read, word: String, wildcard: Option<char>) -> Result<(), NotUnderstood> {
    if let Some(w) = wildcard {
        if piece.words.is_empty()
            || word.starts_with(['-', '*', '?'])
            || word.split('/').any(|part| part.starts_with('.') && part != ".")
        {
            return Err(NotUnderstood::Symbol(w));
        }
        piece.wildcard = true;
    }
    piece.words.push(word);
    Ok(())
}

fn end_piece(pieces: &mut Vec<Read>, piece: Read) -> Result<(), NotUnderstood> {
    match piece.words.first() {
        None => Err(NotUnderstood::Empty),
        Some(first) if first.contains('=') => Err(NotUnderstood::Assignment),
        Some(_) => {
            pieces.push(piece);
            Ok(())
        }
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
        "pip" | "pip3" => pip(args),
        "python" | "python3" | "py" => match args {
            [m, p, rest @ ..] if m == "-m" && (p == "pip" || p == "pip3") => pip(rest),
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

/// `args` are pip's own. Its command is the first word that is not an option; but an option before it can take
/// that word as its value (`pip --log list install x` writes a log named `list` and installs), so pip only reads
/// when its command comes first (code review, phase 5 wrap-up).
fn pip(args: &[String]) -> Kind {
    let sub = args.iter().map(String::as_str).find(|a| !a.starts_with('-')).unwrap_or("");
    match sub {
        "list" | "show" | "freeze" | "check" if args[0] == sub => Kind::ReadOnly,
        "list" | "show" | "freeze" | "check" => Kind::Unknown,
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
    if let Some(why) = blocked(text) {
        return Assessment::Blocked(why);
    }
    let words = match split(text) {
        Ok(words) => words,
        Err(why) => return Assessment::NotUnderstood(why),
    };
    match judge(&words, None) {
        Ok((kind, outside)) => Assessment::Words { words, kind, outside },
        Err(why) => Assessment::Blocked(why),
    }
}

/// What Steplet makes of a command an agent proposes during a run (`assess_in`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proposed {
    /// The whole text, or one of its pieces, can never be allowed.
    Blocked(Blocked),
    NotUnderstood(NotUnderstood),
    /// One command, or each command of a chain in order.
    Pieces(Vec<Piece>),
}

/// One command of what an agent proposed, judged as `assess` judges a file's command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    pub words: Vec<String>,
    pub kind: Kind,
    pub outside: bool,
    /// A word has a wildcard the shell will expand, so the words that run are not these: the piece is never the
    /// file's declared command, and only counts as a harmless read.
    pub wildcard: bool,
    /// It reads what the piece before it prints (it follows a single `|`). The agent chooses that input, so the
    /// piece is never the file's declared command either: `echo … | python3` would run the agent's own code.
    pub piped: bool,
}

impl Piece {
    /// A harmless read inside the working copy (G2).
    pub fn harmless(&self) -> bool {
        self.kind == Kind::ReadOnly && !self.outside
    }
}

/// As `assess`, for a command proposed during a run in the working copy `work` (an absolute path, as the checker's).
/// Two things differ. A full path to the working copy or into it is inside, as its short form is: a model writes
/// the copy's full path (phase 5 wrap-up: `ls -la /work/p5l1/repo` was its first move in both paid live runs). And
/// a chain is read piece by piece (`read`): its second move in both was a chain of reads. The checks on the whole
/// text still come first.
pub fn assess_in(text: &str, work: &str) -> Proposed {
    debug_assert!(work != "/" && paths::normalize(work).as_deref() == Some(work), "not a working copy: {work}");
    if let Some(why) = blocked(text) {
        return Proposed::Blocked(why);
    }
    let read = match read(text, true) {
        Ok(read) => read,
        Err(why) => return Proposed::NotUnderstood(why),
    };
    let mut pieces = Vec::with_capacity(read.len());
    for Read { words, wildcard, piped } in read {
        match judge(&words, Some(work)) {
            Ok((kind, outside)) => pieces.push(Piece { words, kind, outside, wildcard, piped }),
            Err(why) => return Proposed::Blocked(why),
        }
    }
    Proposed::Pieces(pieces)
}

/// Why the whole text of a command can never be allowed, before it is read.
fn blocked(text: &str) -> Option<Blocked> {
    if text.chars().any(is_invisible) {
        return Some(Blocked::Invisible);
    }
    if downloads_into_shell(text) {
        return Some(Blocked::DownloadIntoShell);
    }
    paths::secret_in(text).map(Blocked::Secret)
}

/// The kind of one command's `words` and whether it reaches outside, or why it can never be allowed.
fn judge(words: &[String], work: Option<&str>) -> Result<(Kind, bool), Blocked> {
    match danger(words) {
        Some(Danger::Admin) => return Err(Blocked::Admin),
        Some(Danger::Registry) => return Err(Blocked::Registry),
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
    Ok((kind(words), outside))
}
