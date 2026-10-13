//! Steplet's runner: runs one `.steplet` file's task with Claude Code inside Steplet's own Linux (decision E3,
//! option D), with every action the agent proposes decided by the one checker, `steplet_core::policy::Policy::check`.

mod decide;
mod types;

pub use decide::decide;
pub use types::{GateAnswer, GateRequest};
