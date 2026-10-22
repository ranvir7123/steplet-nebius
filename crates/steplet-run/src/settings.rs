//! The run's settings file and MCP config (plan section 3), written from the file and the run's paths. Deny rules
//! only: the gate decides every allow (R4). The layout of Steplet's Linux is the real guard; these are the backup.

use serde_json::{Value, json};
use steplet_core::header::Steplet;

use crate::types::{GateLink, RunPaths, STATE_ROOT, WORK_ROOT};

/// The gate's name as an MCP server; Claude Code calls its tool `mcp__gate__decide`.
pub(crate) const GATE_SERVER: &str = "gate";

/// Where WSL puts Windows drives and the folders it shares between distributions.
const MOUNTS: &str = "/mnt";

/// The settings file of one run of `s`.
pub fn settings_json(s: &Steplet, paths: &RunPaths, gate: &GateLink, allow_model_switch: bool) -> String {
    let repo = paths.work_repo();
    let git = format!("{repo}/.git");
    let domains: Vec<&str> = s.allow.domains.iter().map(|d| d.text.as_str()).collect();
    pretty(&json!({
        "sandbox": {
            "enabled": true,
            "failIfUnavailable": true,
            // Every command asks the gate, and none runs outside the fence.
            "allowUnsandboxedCommands": false,
            "autoAllowBashIfSandboxed": false,
            "filesystem": {
                "denyRead": [MOUNTS, STATE_ROOT, WORK_ROOT],
                // The narrower path wins, so this opens the working copy again inside the hidden `/work`.
                "allowRead": [repo],
                "denyWrite": [git],
            },
            // An undeclared site is refused, not asked about.
            "network": { "allowedDomains": domains, "strictAllowlist": true },
        },
        "permissions": {
            // For the agent's own file tools, which run outside the fence. No `Read` rule on `/work`: one refused the
            // agent's own copy (C9), and other runs' copies do not exist, since `/work` holds one run at a time. The
            // `Edit` rule on Steplet's own folder is a second lock behind the checker's J1 answer: Claude Code itself
            // refuses such a write, before any question.
            "deny": [rule("Read", MOUNTS), rule("Read", STATE_ROOT), rule("Edit", &git), rule("Edit", STATE_ROOT)],
        },
        // E1: the run stays on the model the receiver approved, and a refusal ends it, unless the receiver chose to
        // run again allowing another model.
        "switchModelsOnFlag": allow_model_switch,
        // H2: the gate asks the app for the key over the pipe, so the key is never written to the Linux disk.
        "apiKeyHelper": format!(
            "{} --key --pipe {} --token {} --deadline-ms {} --steplet-pid {}",
            gate.exe(),
            gate.pipe(),
            gate.token(),
            gate.deadline_ms(),
            gate.steplet_pid()
        ),
    }))
}

/// A permission rule for everything below the absolute path `folder`. Claude Code reads a rule's path as absolute
/// when it starts with `//`, so the rule is the folder with one more `/` in front.
fn rule(tool: &str, folder: &str) -> String {
    format!("{tool}(/{folder}/**)")
}

/// The MCP config of one run: the gate is its only server (Claude Code runs with `--strict-mcp-config`).
pub fn mcp_json(gate: &GateLink) -> String {
    pretty(&json!({
        "mcpServers": {
            GATE_SERVER: {
                "type": "stdio",
                "command": gate.exe(),
                "args": [
                    "--pipe", gate.pipe(), "--token", gate.token(), "--log", gate.log(),
                    "--deadline-ms", gate.deadline_ms(), "--steplet-pid", gate.steplet_pid().to_string(),
                ],
            }
        }
    }))
}

fn pretty(v: &Value) -> String {
    format!("{v:#}\n")
}
