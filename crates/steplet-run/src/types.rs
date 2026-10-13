//! The runner's shared types. Each is added by the task that first needs it (plan section 6 gives the full list).

use serde_json::Value;

/// One permission request from the agent, as `steplet-gate.exe` relays it over the pipe.
#[derive(Clone, Debug, PartialEq)]
pub struct GateRequest {
    pub run_token: String,
    pub tool_name: String,
    pub input: Value,
    pub tool_use_id: String,
}

/// The answer the gate hands back to Claude Code.
#[derive(Clone, Debug, PartialEq)]
pub enum GateAnswer {
    /// Run exactly this input (R7: the approval is bound to the request it answers).
    Allow { updated_input: Value },
    /// Do not run it; `message` says why, in Steplet's own words.
    Deny { message: String },
}
