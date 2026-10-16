//! Steplet's runner: runs one `.steplet` file's task with Claude Code inside Steplet's own Linux (decision E3,
//! option D), with every action the agent proposes decided by the one checker, `steplet_core::policy::Policy::check`.

mod decide;
mod launch;
mod prepare;
mod settings;
mod stream;
mod types;
pub mod wsl;

pub use decide::decide;
pub use launch::{LaunchError, RunHandle, launch};
pub use prepare::{PrepareError, REPOSITORY_HOSTS, discard, prepare};
pub use settings::{mcp_json, settings_json};
pub use stream::parse_line;
pub use types::{BadValue, Event, GateAnswer, GateLink, GateRequest, RunPaths, RunRequest};
