//! The runner's shared types. Each is added by the task that first needs it (plan section 6 gives the full list).

use std::fmt;
use std::time::Duration;

use serde_json::Value;
use steplet_core::header::Steplet;

/// Where Steplet's Linux keeps things (plan section 3). `/work` is kept apart from `/var/lib/steplet` so the fence
/// can hide the private folder without also hiding the working copy (C9).
pub(crate) const WORK_ROOT: &str = "/work";
pub(crate) const STATE_ROOT: &str = "/var/lib/steplet";
/// The start of each run's temporary folder, `/tmp/steplet-run-<id>` (the owner's choice A, 18 Oct 2026). It is not
/// inside `STATE_ROOT`, which the fence hides, because the fence's network bridge for a run is a socket in it: hidden,
/// a program the task runs could reach no site, not even one the file declares. The task's programs could already
/// read `/tmp`.
pub(crate) const RUN_TMP: &str = "/tmp/steplet-run-";

/// A value Steplet will not write into a run's files; the text names which value it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BadValue(pub &'static str);

/// The Linux paths of one run, in the layout of plan section 3. Made only by `new`, which checks its values, and
/// read-only after that.
#[derive(Clone, Debug, PartialEq)]
pub struct RunPaths {
    distro_id: String,
    run_id: String,
    work_repo: String,
    run_home: String,
    run_config: String,
    run_tmp: String,
    settings: String,
    mcp: String,
}

impl RunPaths {
    /// The paths of run `run_id` in the distribution `distro_id`. The run id becomes part of every path and of
    /// commands run as root, so it is 1 to 64 lower-case letters, digits and `-`, and does not start with `-`. The
    /// distribution id goes on `wsl.exe`'s command line, so it must be the braced GUID WSL's registry names it by.
    pub fn new(distro_id: &str, run_id: &str) -> Result<RunPaths, BadValue> {
        if !is_guid(distro_id) {
            return Err(BadValue("distribution id"));
        }
        let ok = (1..=64).contains(&run_id.len())
            && !run_id.starts_with('-')
            && run_id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !ok {
            return Err(BadValue("run id"));
        }
        let run = format!("{STATE_ROOT}/runs/{run_id}");
        Ok(RunPaths {
            distro_id: distro_id.into(),
            run_id: run_id.into(),
            work_repo: format!("{WORK_ROOT}/{run_id}/repo"),
            run_home: format!("{run}/home"),
            run_config: format!("{run}/config"),
            run_tmp: format!("{RUN_TMP}{run_id}"),
            settings: format!("{run}/settings.json"),
            mcp: format!("{run}/mcp.json"),
        })
    }

    pub fn distro_id(&self) -> &str {
        &self.distro_id
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// The working copy, `/work/<id>/repo`.
    pub fn work_repo(&self) -> &str {
        &self.work_repo
    }

    /// The run's `HOME`. It, `run_config` (`CLAUDE_CONFIG_DIR`) and `run_tmp` (`TMPDIR`, under `/tmp`) are fresh
    /// each run and deleted after it.
    pub fn run_home(&self) -> &str {
        &self.run_home
    }

    pub fn run_config(&self) -> &str {
        &self.run_config
    }

    pub fn run_tmp(&self) -> &str {
        &self.run_tmp
    }

    /// The run's settings file. It and the MCP config are owned by root, so the agent cannot rewrite them.
    pub fn settings(&self) -> &str {
        &self.settings
    }

    pub fn mcp(&self) -> &str {
        &self.mcp
    }
}

/// How Claude Code reaches the gate in one run: the gate program on Steplet's Linux disk, the pipe it relays to, the
/// run's token, the gate's log (a Windows path: the gate is a Windows program), and Steplet's limit on each decision
/// (F36). Made only by `new`, which checks its values, and read-only after that. Its `Debug` hides the token.
#[derive(Clone, PartialEq)]
pub struct GateLink {
    exe: String,
    pipe: String,
    token: String,
    log: String,
    deadline: Duration,
}

impl GateLink {
    /// The program, pipe name and token also go into `apiKeyHelper`, a line Claude Code hands to a shell, so each is
    /// made of plain words (the program is an absolute path of them). The log is passed as one argument, not through
    /// a shell, so only an empty log, a leading `-` (the gate would read it as an option) or a control character is
    /// refused. The deadline is from a tenth of a second to an hour.
    pub fn new(exe: &str, pipe: &str, token: &str, log: &str, deadline: Duration) -> Result<GateLink, BadValue> {
        let path_ok = exe
            .strip_prefix('/')
            .is_some_and(|rest| rest.split('/').all(|part| plain_word(part) && part != "." && part != ".."));
        if !path_ok {
            return Err(BadValue("gate program"));
        }
        if !plain_word(pipe) {
            return Err(BadValue("pipe name"));
        }
        if !plain_word(token) {
            return Err(BadValue("run token"));
        }
        if log.is_empty() || log.starts_with('-') || log.chars().any(char::is_control) {
            return Err(BadValue("gate log"));
        }
        if !(Duration::from_millis(100)..=Duration::from_secs(3600)).contains(&deadline) {
            return Err(BadValue("gate deadline"));
        }
        Ok(GateLink { exe: exe.into(), pipe: pipe.into(), token: token.into(), log: log.into(), deadline })
    }

    pub fn exe(&self) -> &str {
        &self.exe
    }

    pub fn pipe(&self) -> &str {
        &self.pipe
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn log(&self) -> &str {
        &self.log
    }

    /// How long Steplet takes at most to answer one request; a decision not made by then is a refusal.
    pub fn deadline(&self) -> Duration {
        self.deadline
    }

    /// The deadline as the gate's `--deadline-ms` option.
    pub(crate) fn deadline_ms(&self) -> String {
        self.deadline.as_millis().to_string()
    }
}

impl fmt::Debug for GateLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GateLink")
            .field("exe", &self.exe)
            .field("pipe", &self.pipe)
            .field("token", &HIDDEN)
            .field("log", &self.log)
            .field("deadline", &self.deadline)
            .finish()
    }
}

/// True for a word a shell reads as itself and a program does not read as an option: 1 to 128 ASCII letters, digits,
/// `.`, `_` and `-`, not starting with `-`.
fn plain_word(s: &str) -> bool {
    (1..=128).contains(&s.len())
        && !s.starts_with('-')
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// True for a GUID in braces, `{8-4-4-4-12}` hexadecimal digits, as WSL's registry names each distribution.
pub(crate) fn is_guid(s: &str) -> bool {
    let Some(inner) = s.strip_prefix('{').and_then(|rest| rest.strip_suffix('}')) else { return false };
    let groups: Vec<&str> = inner.split('-').collect();
    groups.len() == 5
        && groups.iter().zip([8, 4, 4, 4, 12]).all(|(g, n)| g.len() == n && g.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// What one run is asked to do. `prepare` checks the repository address and the commit before it uses them.
#[derive(Clone, Debug, PartialEq)]
pub struct RunRequest {
    pub steplet: Steplet,
    pub repo_url: String,
    pub commit: String,
    pub task_text: String,
    pub max_turns: u32,
    pub deadline: Duration,
}

/// One thing that happened in a run, as read from Claude Code's event stream or its error output. Nothing on the
/// stream is dropped without a trace except the model's thinking: a kind Steplet does not read becomes `Other`.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The run started: the model, the tools offered, and whether the gate's MCP server connected.
    Started {
        model: String,
        tools: Vec<String>,
        gate_connected: bool,
    },
    /// Text the agent wrote. Shown only as "the agent says", never read for the run's outcome (F47).
    AgentText(String),
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    /// A tool call's result. `plain_end`: Claude Code's own record of the call (the line's `tool_use_result`, one
    /// result on the line) says it was not interrupted, not moved to the background, and its exit code was given no
    /// meaning of Claude Code's own (Claude Code marks exit code 1 of grep, find, diff or test as no error). Only then
    /// does "no error" mean exit code 0.
    ToolResult {
        id: String,
        is_error: bool,
        text: String,
        plain_end: bool,
    },
    /// A hook started, ran or answered. Any of them stops the run (D3).
    HookEvent {
        name: String,
    },
    /// The model refused. `to` is the model Claude Code then switched to, if it did (E1, D7).
    ModelSwitch {
        from: String,
        to: Option<String>,
        category: String,
        explanation: String,
    },
    /// The run's end as Claude Code reports it; `models` are the keys of its `modelUsage`. Not the outcome by
    /// itself (F35): a refused run says `success`.
    Result {
        subtype: String,
        is_error: bool,
        stop_reason: String,
        models: Vec<String>,
        text: String,
    },
    /// A tool call that did not ask the gate, as the tripwire watcher judged it (Task 9). Steplet's own record, put
    /// right after the call's result. `id` is the call's; `what` is its command, path or address; `harmless` means it
    /// was a harmless read (G2) and the run went on.
    SkippedGate {
        id: String,
        tool: String,
        what: String,
        harmless: bool,
    },
    /// One line of Claude Code's error output.
    Stderr(String),
    /// A line on the event stream that is not a JSON object.
    Unreadable(String),
    /// A JSON event of a kind Steplet does not read, named `type/subtype`.
    Other(String),
}

/// One permission request from the agent, as `steplet-gate.exe` relays it over the pipe. Its `Debug` hides the run
/// token.
#[derive(Clone, PartialEq)]
pub struct GateRequest {
    pub run_token: String,
    pub tool_name: String,
    pub input: Value,
    pub tool_use_id: String,
}

impl fmt::Debug for GateRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GateRequest")
            .field("run_token", &HIDDEN)
            .field("tool_name", &self.tool_name)
            .field("input", &self.input)
            .field("tool_use_id", &self.tool_use_id)
            .finish()
    }
}

/// What a hidden value shows as in `Debug`.
const HIDDEN: &str = "(hidden)";

/// The answer the gate hands back to Claude Code.
#[derive(Clone, Debug, PartialEq)]
pub enum GateAnswer {
    /// Run exactly this input (R7: the approval is bound to the request it answers).
    Allow { updated_input: Value },
    /// Do not run it; `message` says why, in Steplet's own words.
    Deny { message: String },
}
