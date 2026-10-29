//! Steplet's runner: runs one `.steplet` file's task with Claude Code inside Steplet's own Linux (decision E3,
//! option D), with every action the agent proposes decided by the one checker, `steplet_core::policy::Policy::check`.

mod ask;
mod client;
mod decide;
mod gate;
mod launch;
mod live;
mod lock;
mod model;
mod places;
mod prepare;
mod price;
mod promises;
mod report;
mod run;
mod settings;
pub mod setup;
mod sites;
mod stream;
mod task;
mod tokenfactory;
mod types;
mod watch;
mod winhttp;
pub mod wsl;

pub use ask::{Answer, Answers, Asker, Question, Reply, fingerprint};
pub use client::Client;
pub use decide::{Checker, decide};
pub use gate::{
    ApiKey, Asked, GateError, GateService, gate_file_name, gate_for_run, gate_on_windows, install_gate, serve,
};
pub use launch::{LaunchError, RunHandle, launch};
pub use live::{Live, Progress, StopSignal};
pub use lock::RunLock;
pub use model::{
    Finish, Message, Model, ModelError, ModelReply, Role, Scripted, Seen, Step, ToolCall, ToolSpec, Usage, chat_body,
    read_reply,
};
pub use places::{LinuxPaths, RealPaths};
pub use prepare::{
    PrepareError, Prepared, REPOSITORY_HOSTS, changes, discard, fetchable, prepare, removed_files, run_config,
};
pub use price::{Nanodollars, Price, cost, price, worst_case};
pub use promises::{NetworkMode, Promise, network_mode, promises, read_network_mode};
pub use report::{
    Check, NotYes, Refusal, Report, ReportCommand, Trust, build_report, changed_files, exit_code_of, raw_log_line,
    read_trust,
};
pub use run::{
    BACKSTOP, RunError, RunOutcome, RunSetup, RunningAgent, run, run_live, run_live_locked, run_live_on_pipe, watch_run,
};
pub use settings::{mcp_json, settings_json};
pub use sites::{
    ListedSite, check_sites, leads_to_this_computer_or_private, pin_sites, resolve_all_in_linux, resolve_in_linux,
    unpin_sites,
};
pub use stream::parse_line;
pub use task::task_text;
pub use tokenfactory::{BASE, TokenFactory};
pub use types::{BadValue, Event, GateAnswer, GateLink, GateRequest, RunPaths, RunRequest};
pub use watch::{Tripwire, Verdict, Watcher, call_what};
pub use winhttp::HttpLimits;
