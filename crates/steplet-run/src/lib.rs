//! Steplet's runner: runs one `.steplet` file's task with Claude Code inside Steplet's own Linux (decision E3,
//! option D), with every action the agent proposes decided by the one checker, `steplet_core::policy::Policy::check`.

mod client;
mod decide;
mod gate;
mod launch;
mod prepare;
mod report;
mod run;
mod settings;
mod stream;
mod task;
mod types;
mod watch;
pub mod wsl;

pub use client::Client;
pub use decide::decide;
pub use gate::{ApiKey, Asked, GateError, GateService, gate_file_name, gate_on_windows, install_gate, serve};
pub use launch::{LaunchError, RunHandle, launch};
pub use prepare::{PrepareError, Prepared, REPOSITORY_HOSTS, changes, discard, prepare, removed_files, run_config};
pub use report::{Check, Refusal, Report, ReportCommand, Trust, build_report, changed_files, read_trust};
pub use run::{BACKSTOP, RunError, RunOutcome, RunSetup, RunningAgent, run, watch_run};
pub use settings::{mcp_json, settings_json};
pub use stream::parse_line;
pub use task::task_text;
pub use types::{BadValue, Event, GateAnswer, GateLink, GateRequest, RunPaths, RunRequest};
pub use watch::{Tripwire, Verdict, Watcher};
