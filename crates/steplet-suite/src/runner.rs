//! The runner (phase 5, section 3.3): checks this computer first, builds the test programs once, runs exactly the
//! tests each case names, judges each case from their results, and looks after each case that used Steplet's Linux
//! for anything it left. What it needs from this computer is the `Machine`; the suite's own tests give it a stand-in.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use steplet_run::setup::{self, CLAUDE_VERSION, State, Wsl};
use steplet_run::{RunLock, wsl};

use crate::catalogue::{Case, Needs};
use crate::support;
use crate::verdict::{TestOutcome, Verdict, read_results, verdict};

/// The made-up keys the scripted layer's tests use (Anthropic's, and Nebius's for Steplet's agent); each record is
/// counted for them before it is kept.
pub use crate::support::{MADE_UP_KEY, MADE_UP_NEBIUS_KEY};

/// Why the suite does not start. Nothing is built or run, and nothing on Steplet's Linux is changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotReady {
    /// Steplet's Linux is not installed or not set up; `why` says what is wrong.
    LinuxMissing { why: String },
    /// Another run holds Steplet's one-run lock: a Steplet window, or another session's test.
    LinuxBusy,
    /// Steplet's Linux has another Claude Code than the one the suite backs.
    ClaudeVersion { found: String },
    /// Before the first case, Steplet's Linux held these, and the clean-up could not remove them.
    NotClean { left: Vec<String> },
}

impl NotReady {
    pub fn words(&self) -> String {
        match self {
            NotReady::LinuxMissing { why } => format!("Steplet's Linux is not ready: {why}. Nothing was changed."),
            NotReady::LinuxBusy => "Steplet's Linux is in use by another run (a Steplet window, or another session's \
                                    test). Nothing was changed."
                .to_owned(),
            NotReady::ClaudeVersion { found } => format!(
                "Claude Code {found} is installed in Steplet's Linux; this suite backs {CLAUDE_VERSION} only. Nothing \
                 was changed."
            ),
            NotReady::NotClean { left } => format!(
                "Before the first case, Steplet's Linux held what the suite could not remove: {}. No case was run.",
                left.join("; ")
            ),
        }
    }
}

/// Whether the cases on Steplet's Linux may run: no run holds the lock, Steplet's Linux is there and set up, and its
/// Claude Code is the pinned version. `claude_found` is the version the suite read itself: Steplet's own look only
/// says Claude Code is missing when its version is another.
pub fn ready_to_run(look: &State, claude_found: Option<&str>, lock_free: bool) -> Result<(), NotReady> {
    if !lock_free {
        return Err(NotReady::LinuxBusy);
    }
    let missing = |why: String| Err(NotReady::LinuxMissing { why });
    match &look.wsl {
        Wsl::Missing => return missing("Windows' Linux feature (WSL) is not installed".into()),
        Wsl::TooOld { version } => return missing(format!("WSL {version} is older than Steplet needs")),
        Wsl::Ready { .. } => {}
    }
    if let Some(trouble) = &look.trouble {
        return missing(trouble.clone());
    }
    if look.linux.is_none() {
        return missing("it is not installed".into());
    }
    match claude_found {
        None => return missing("Claude Code is not installed in it".into()),
        Some(found) if found != CLAUDE_VERSION => return Err(NotReady::ClaudeVersion { found: found.into() }),
        Some(_) => {}
    }
    if !look.missing.is_empty() {
        return missing(format!("it lacks {}", look.missing.join(", ")));
    }
    Ok(())
}

/// The files of `/opt/steplet-test` kept on purpose: the helpers each test puts there again, and their logs.
const KEPT_TEST_FILES: &[&str] =
    &["listener.py", "fake_model.py", "made-up-key.sha256", "count_key.py", "count_everywhere.py", "m0-fixture"];
const KEPT_TEST_ENDINGS: &[&str] = &[".script.json", ".model.log", ".listener.log"];

/// What a case left, from the look after it (`support::after_case_look`, records ended by a NUL): each path, or
/// `program <pid> <command>`, or `could not look: <why>`, or `could not read: <record>` for a record the look should
/// not have printed, a path that does not start at `/`, or one holding a control character (a line break in a name a
/// task's program chose). Shown text has every control character replaced by `?`. Empty when the case left nothing.
pub fn leftovers(listing: &str) -> Vec<String> {
    let shown = |text: &str| text.chars().map(|c| if c.is_control() { '?' } else { c }).collect::<String>();
    let mut left = Vec::new();
    for record in listing.split('\0').filter(|r| !r.trim().is_empty()) {
        let unreadable = format!("could not read: {}", shown(record));
        let Some((kind, what)) = record.split_once('\t') else {
            left.push(unreadable);
            continue;
        };
        let path = what.starts_with('/') && !what.contains(|c: char| c.is_control());
        match kind {
            "left" | "marker" if path => left.push(what.to_owned()),
            "test-file" if path => {
                let name = what.rsplit('/').next().unwrap_or(what);
                if !(KEPT_TEST_FILES.contains(&name) || KEPT_TEST_ENDINGS.iter().any(|e| name.ends_with(e))) {
                    left.push(what.to_owned());
                }
            }
            "program" => match what.split_once('\t') {
                Some((pid, command)) if !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit()) => {
                    left.push(format!("program {pid} {}", shown(command)));
                }
                _ => left.push(unreadable),
            },
            "could-not-look" => left.push(format!("could not look: {}", shown(what))),
            _ => left.push(unreadable),
        }
    }
    left
}

/// The folders the look lists the contents of; the clean-up removes only what sits directly inside one of them.
const LOOKED_IN: &[&str] = &[
    "/work",
    "/var/lib/steplet/runs",
    "/opt/steplet/gate/runs",
    "/var/lib/steplet",
    "/tmp",
    "/var/tmp",
    "/opt/steplet-test",
];

/// The script root runs to remove what `leftovers` named. First it ends each program named, by its number; then it
/// removes each path that sits directly inside a folder the look lists (never Steplet's runs folder, nothing deeper,
/// where a link on the way could lead out, and nothing with `..`, a quote or a control character). Everything else is
/// left as it is: it has been named, and the suite does not pass.
pub fn removal_script(left: &[String]) -> String {
    let mut ends = String::new();
    let mut removals = String::new();
    for what in left {
        if let Some(rest) = what.strip_prefix("program ") {
            let pid = rest.split(' ').next().unwrap_or("");
            if !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit()) {
                ends += &format!("kill -KILL {pid}\n");
            }
            continue;
        }
        let Some((folder, name)) = what.rsplit_once('/') else { continue };
        let inside = LOOKED_IN.contains(&folder) && !name.is_empty() && name != "." && name != "..";
        let safe = !what.contains(|c: char| c == '\'' || c.is_control()) && what != "/var/lib/steplet/runs";
        if inside && safe {
            removals += &format!("rm -rf -- '{what}'\n");
        }
    }
    ends + &removals
}

/// What the runner needs from this computer.
pub trait Machine {
    /// Takes Steplet's one-run lock and holds it until `let_go`; false when another run holds it. Never held while a
    /// case's tests run: the tests that start runs take it themselves.
    fn take_lock(&mut self) -> bool;
    fn let_go(&mut self);
    /// What is set up in Steplet's Linux, and the Claude Code version installed there. Changes nothing.
    fn look(&mut self) -> (State, Option<String>);
    /// Builds the named test programs of `package`; on failure, the end of what the build said.
    fn build(&mut self, package: &str, programs: &[&str]) -> Result<(), String>;
    /// Runs exactly the named tests of one test program, set aside or not, one at a time; everything it printed.
    fn run_tests(&mut self, package: &str, program: &str, names: &[&str]) -> String;
    /// What is left in Steplet's Linux after a case (`support::after_case_look`).
    fn after_case_look(&mut self) -> String;
    /// Removes what `leftovers` named.
    fn remove(&mut self, left: &[String]);
}

#[derive(Debug)]
pub struct CaseResult {
    pub case: &'static Case,
    pub verdict: Verdict,
    /// How long this case's own test runs took (none when an earlier case already ran its tests).
    pub took: Duration,
    /// Everything its test programs printed, for the record.
    pub output: String,
    /// Every one of its tests had already run for an earlier case in this run.
    pub reused: bool,
    /// What the look after it found in Steplet's Linux, each said in words (`left behind: …`, then removed; `still
    /// there after the clean-up: …`; or why it could not look). Empty when it left nothing.
    pub left: Vec<String>,
}

/// What one run of the suite found.
#[derive(Debug)]
pub struct Suite {
    /// What Steplet's Linux held before the first case, each said in words; already removed, and counted against no
    /// case.
    pub before: Vec<String>,
    pub results: Vec<CaseResult>,
}

/// Runs `cases` in order, calling `row` as each one finishes. Stops before anything when a case needs Steplet's Linux
/// and it is busy, not set up, has another Claude Code, or holds from earlier work what the clean-up cannot remove. A
/// test that several cases name runs once.
pub fn run_cases(
    cases: &[&'static Case],
    machine: &mut dyn Machine,
    row: &mut dyn FnMut(&CaseResult),
) -> Result<Suite, NotReady> {
    let mut before = Vec::new();
    if cases.iter().any(|c| c.needs.contains(&Needs::StepletLinux)) {
        if !machine.take_lock() {
            return Err(NotReady::LinuxBusy);
        }
        let (state, claude) = machine.look();
        machine.let_go();
        ready_to_run(&state, claude.as_deref(), true)?;
        let start = look_and_clean(machine).ok_or(NotReady::LinuxBusy)?;
        if !start.still_there.is_empty() {
            return Err(NotReady::NotClean { left: start.still_there });
        }
        before = start.left.iter().map(|l| format!("there before the suite started, removed: {l}")).collect();
    }

    let mut programs: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for t in cases.iter().flat_map(|c| c.tests) {
        let list = programs.entry(t.package).or_default();
        if !list.contains(&t.program) {
            list.push(t.program);
        }
    }
    let mut not_built: HashMap<&str, String> = HashMap::new();
    for (package, list) in &programs {
        if let Err(why) = machine.build(package, list) {
            not_built.insert(package, why);
        }
    }

    type Key = (&'static str, &'static str, &'static str);
    let mut ran: HashSet<Key> = HashSet::new();
    let mut known: HashMap<Key, TestOutcome> = HashMap::new();
    let mut results = Vec::new();
    for &case in cases {
        let start = Instant::now();
        let mut output = String::new();
        let mut groups: Vec<((&'static str, &'static str), Vec<&'static str>)> = Vec::new();
        let mut earlier: Vec<&'static str> = Vec::new();
        for t in case.tests {
            if ran.contains(&(t.package, t.program, t.name)) {
                earlier.push(t.name);
                continue;
            }
            match groups.iter_mut().find(|(p, _)| *p == (t.package, t.program)) {
                Some((_, names)) => names.push(t.name),
                None => groups.push(((t.package, t.program), vec![t.name])),
            }
        }
        for ((package, program), names) in groups {
            ran.extend(names.iter().map(|n| (package, program, *n)));
            if let Some(why) = not_built.get(package) {
                output += &format!("--- {package}, tests/{program}.rs: its test programs did not build:\n{why}\n");
                for n in names {
                    let why = format!("the test programs of {package} did not build");
                    known.insert((package, program, n), TestOutcome::DidNotRun(why));
                }
                continue;
            }
            output += &format!("--- {package}, tests/{program}.rs: {}\n", names.join(", "));
            let printed = machine.run_tests(package, program, &names);
            let read = read_results(&printed);
            output += &printed;
            for n in names {
                if let Some(outcome) = read.get(n) {
                    known.insert((package, program, n), outcome.clone());
                }
            }
        }
        if !earlier.is_empty() {
            output += &format!("--- already run for an earlier case in this run: {}\n", earlier.join(", "));
        }
        let took = start.elapsed();
        let mine: HashMap<String, TestOutcome> = case
            .tests
            .iter()
            .filter_map(|t| known.get(&(t.package, t.program, t.name)).map(|o| (t.name.to_owned(), o.clone())))
            .collect();
        let left = if case.needs.contains(&Needs::StepletLinux) {
            match look_and_clean(machine) {
                Some(looked) => looked.said(),
                None => vec![
                    "could not look after this case: Steplet's Linux is in use by another run, so nothing was looked \
                     at or removed"
                        .to_owned(),
                ],
            }
        } else {
            Vec::new()
        };
        let reused = earlier.len() == case.tests.len();
        let result = CaseResult { case, verdict: verdict(case, &mine), took, output, reused, left };
        row(&result);
        results.push(result);
    }
    Ok(Suite { before, results })
}

/// What one look and clean-up found: what was left (then removed), and what was still there after the clean-up.
struct Looked {
    left: Vec<String>,
    still_there: Vec<String>,
}

impl Looked {
    fn said(&self) -> Vec<String> {
        let left = self.left.iter().map(|l| format!("left behind: {l}"));
        left.chain(self.still_there.iter().map(|l| format!("still there after the clean-up: {l}"))).collect()
    }
}

/// A look in Steplet's Linux for anything left, its removal, and a second look, all while holding Steplet's one-run
/// lock, so a run a Steplet window started meanwhile is never taken for leftovers. `None` when another run holds the
/// lock: then nothing was looked at or removed.
fn look_and_clean(machine: &mut dyn Machine) -> Option<Looked> {
    if !machine.take_lock() {
        return None;
    }
    let left = leftovers(&machine.after_case_look());
    let mut still_there = Vec::new();
    if !left.is_empty() {
        machine.remove(&left);
        still_there = leftovers(&machine.after_case_look());
    }
    machine.let_go();
    Some(Looked { left, still_there })
}

/// The suite passes only when every case was blocked and none left anything behind.
pub fn passed(results: &[CaseResult]) -> bool {
    results.iter().all(|r| r.verdict == Verdict::Blocked && r.left.is_empty())
}

/// Runs `command` with its output and its errors in one file, in the order they were written, and returns the file's
/// text. The exit code is not needed: every result is read from the text.
pub fn output_of(command: &mut Command, log: &Path) -> io::Result<String> {
    if let Some(dir) = log.parent() {
        fs::create_dir_all(dir)?;
    }
    let file = File::create(log)?;
    command.stdin(Stdio::null()).stdout(file.try_clone()?).stderr(file).status()?;
    Ok(String::from_utf8_lossy(&fs::read(log)?).into_owned())
}

/// The Windows variables a test program gets from the suite: where Windows, this user's folders, cargo and Rust are,
/// and what this computer is. Nothing else of whoever started the suite reaches the tests, or their logs in the record.
const TEST_VARIABLES: &[&str] = &[
    "PATH",
    "PATHEXT",
    "SystemRoot",
    "windir",
    "SystemDrive",
    "ComSpec",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
    "LOCALAPPDATA",
    "APPDATA",
    "ProgramData",
    "ALLUSERSPROFILE",
    "PUBLIC",
    "ProgramFiles",
    "ProgramFiles(x86)",
    "ProgramW6432",
    "CommonProgramFiles",
    "CommonProgramFiles(x86)",
    "CommonProgramW6432",
    "USERNAME",
    "USERDOMAIN",
    "COMPUTERNAME",
    "NUMBER_OF_PROCESSORS",
    "PROCESSOR_ARCHITECTURE",
    "PROCESSOR_IDENTIFIER",
    "PROCESSOR_LEVEL",
    "PROCESSOR_REVISION",
    "OS",
    "CARGO_HOME",
    "RUSTUP_HOME",
    "RUSTUP_TOOLCHAIN",
    // The live layer passes each live case its spending cap through this variable; without it a live case spends
    // nothing. The suite's `live --yes-spend` command sets it before it runs the cases.
    crate::live::CAP_VAR,
];

/// The variables of `vars` a test program gets, by name without regard to case (as Windows reads them).
pub fn test_environment(vars: impl Iterator<Item = (OsString, OsString)>) -> Vec<(OsString, OsString)> {
    vars.filter(|(name, _)| name.to_str().is_some_and(|n| TEST_VARIABLES.iter().any(|t| t.eq_ignore_ascii_case(n))))
        .collect()
}

/// This repository's root folder.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// This computer: cargo for the test programs, Steplet's one-run lock, and Steplet's own Linux.
pub struct ThisComputer {
    /// Where each test program's output is kept while it runs (in `target`, never under `%TEMP%`).
    logs: PathBuf,
    /// The id of Steplet's Linux, once looked at.
    distro: Option<String>,
    /// Steplet's one-run lock, while the suite holds it.
    held: Option<RunLock>,
}

impl ThisComputer {
    pub fn new() -> ThisComputer {
        ThisComputer { logs: repo_root().join("target").join("steplet-suite"), distro: None, held: None }
    }

    /// cargo, in this repository, with only the variables of `test_environment`.
    fn cargo() -> Command {
        let mut c = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
        c.current_dir(repo_root()).stdin(Stdio::null()).env_clear().envs(test_environment(std::env::vars_os()));
        c
    }
}

impl Default for ThisComputer {
    fn default() -> ThisComputer {
        ThisComputer::new()
    }
}

impl Machine for ThisComputer {
    fn take_lock(&mut self) -> bool {
        self.held = RunLock::take().ok();
        self.held.is_some()
    }

    fn let_go(&mut self) {
        self.held = None;
    }

    fn look(&mut self) -> (State, Option<String>) {
        // The gate on the disk is not looked at: every test that needs it puts this worktree's gate there itself.
        let state = setup::look(setup::LINUX, &[]);
        self.distro = state.linux.clone();
        let package =
            "require('/opt/steplet/claude-cli/lib/node_modules/@anthropic-ai/claude-code/package.json').version";
        let claude = self.distro.as_deref().and_then(|d| {
            let out = wsl::run(d, "root", &["/usr/bin/node", "-p", package], b"").ok()?;
            (out.status == Some(0)).then(|| out.stdout.trim().to_owned()).filter(|v| !v.is_empty())
        });
        (state, claude)
    }

    fn build(&mut self, package: &str, programs: &[&str]) -> Result<(), String> {
        // McAfee sometimes holds a freshly built program for a moment (LNK1104): up to three tries.
        let mut said = String::new();
        for _ in 0..3 {
            let mut c = ThisComputer::cargo();
            c.args(["test", "--no-run", "--offline", "-p", package]);
            for p in programs {
                c.args(["--test", p]);
            }
            let out = c.output().map_err(|e| format!("cargo could not be started: {e}"))?;
            if out.status.success() {
                return Ok(());
            }
            let text = String::from_utf8_lossy(&out.stderr).into_owned();
            said = text.lines().rev().take(20).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
            std::thread::sleep(Duration::from_secs(2));
        }
        Err(said)
    }

    fn run_tests(&mut self, package: &str, program: &str, names: &[&str]) -> String {
        let mut c = ThisComputer::cargo();
        c.args(["test", "--offline", "-p", package, "--test", program, "--"]);
        c.args(["--include-ignored", "--exact", "--test-threads=1", "--nocapture"]).args(names);
        let log = self.logs.join(format!("{package}-{program}.log"));
        output_of(&mut c, &log).unwrap_or_else(|e| format!("the test program could not be started: {e}\n"))
    }

    fn after_case_look(&mut self) -> String {
        let Some(distro) = self.distro.clone() else { return "could-not-look\tSteplet's Linux was not found\0".into() };
        if self.held.is_none() {
            return "could-not-look\tthe suite does not hold Steplet's one-run lock\0".into();
        }
        std::panic::catch_unwind(|| support::after_case_look(&distro))
            .unwrap_or_else(|_| "could-not-look\tthe look inside Steplet's Linux failed\0".into())
    }

    fn remove(&mut self, left: &[String]) {
        let (Some(distro), Some(_)) = (&self.distro, &self.held) else { return };
        let script = format!("main() {{\n{}true\n}}\nmain < /dev/null\n", removal_script(left));
        let _ = wsl::run(distro, "root", &["/bin/bash", "-s"], script.as_bytes());
    }
}
