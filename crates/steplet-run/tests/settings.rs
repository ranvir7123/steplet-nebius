use std::time::Duration;

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::{Steplet, parse};
use steplet_run::{GateLink, RunPaths, fence_json, mcp_json, settings_json};

fn steplet(allow: &str) -> Steplet {
    let f = format!("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n{allow}---\n");
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap()
}

fn paths(run_id: &str) -> RunPaths {
    RunPaths::new("{44f36236-99b1-42c0-84bd-7ff8392d51ba}", run_id).unwrap()
}

const LOG: &str = r"C:\Users\Test User\AppData\Local\Steplet\logs\r1.gate.log";

/// Steplet's limit on one gate decision, in these tests.
const DEADLINE: Duration = Duration::from_secs(30);

fn gate(exe: &str, pipe: &str, token: &str) -> GateLink {
    GateLink::new(exe, pipe, token, LOG, DEADLINE).unwrap()
}

fn read(text: &str) -> Value {
    serde_json::from_str(text).expect("the generated file is JSON")
}

#[test]
fn the_run_paths_follow_the_layout() {
    let p = paths("r1");
    assert_eq!(p.distro_id(), "{44f36236-99b1-42c0-84bd-7ff8392d51ba}");
    assert_eq!(p.run_id(), "r1");
    assert_eq!(p.work_repo(), "/work/r1/repo");
    assert_eq!(p.run_home(), "/var/lib/steplet/runs/r1/home");
    assert_eq!(p.run_config(), "/var/lib/steplet/runs/r1/config");
    // The owner's choice A of 18 Oct 2026: under /tmp, where the fence's network bridge can be reached.
    assert_eq!(p.run_tmp(), "/tmp/steplet-run-r1");
    assert_eq!(p.settings(), "/var/lib/steplet/runs/r1/settings.json");
    assert_eq!(p.mcp(), "/var/lib/steplet/runs/r1/mcp.json");
}

#[test]
fn the_gate_link_keeps_its_values() {
    let g = gate("/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd");
    assert_eq!(
        [g.exe(), g.pipe(), g.token(), g.log()],
        ["/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd", LOG]
    );
    assert_eq!(g.deadline(), DEADLINE);
    assert_eq!(g.steplet_pid(), std::process::id(), "the program that makes the link serves its pipe");
}

#[test]
fn the_settings_are_exactly_the_planned_shape() {
    let s = steplet("allow:\n  domains:\n    - pypi.org\n    - files.pythonhosted.org\n");
    let g = gate("/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd");
    assert_eq!(
        read(&settings_json(&s, &paths("r1"), &g, false)),
        json!({
            "sandbox": {
                "enabled": true,
                "failIfUnavailable": true,
                "allowUnsandboxedCommands": false,
                "autoAllowBashIfSandboxed": false,
                "filesystem": {
                    "denyRead": ["/mnt", "/var/lib/steplet", "/work"],
                    "allowRead": ["/work/r1/repo"],
                    "denyWrite": ["/work/r1/repo/.git"]
                },
                "network": {"allowedDomains": ["pypi.org", "files.pythonhosted.org"], "strictAllowlist": true}
            },
            "permissions": {
                "deny": [
                    "Read(//mnt/**)",
                    "Read(//var/lib/steplet/**)",
                    "Edit(//work/r1/repo/.git/**)",
                    "Edit(//var/lib/steplet/**)"
                ]
            },
            "switchModelsOnFlag": false,
            "apiKeyHelper": format!(
                "/opt/steplet/gate/steplet-gate.exe --key --pipe steplet-r1 --token 0123abcd --deadline-ms 30000 \
                 --steplet-pid {}",
                std::process::id()
            )
        })
    );
}

#[test]
fn the_settings_match_the_file_the_task_2_runs_used() {
    // The file Task 2's scripted and real-model runs used, with its token placeholder (spikes/p3/p2.settings.json).
    // Since Task 8 the key mode also gets Steplet's deadline, and since phase 4 Task 1 Claude Code itself refuses
    // writes into Steplet's own folder, and since phase 5 Task 2 the gate is told which program Steplet is; the rest
    // is unchanged.
    let g = gate("/opt/steplet/gate/spike-gate.exe", "steplet-p3-p2", "RUN_TOKEN");
    let mut used = read(include_str!("support/task2.settings.json"));
    let helper =
        format!("{} --deadline-ms 30000 --steplet-pid {}", used["apiKeyHelper"].as_str().unwrap(), std::process::id());
    used["apiKeyHelper"] = helper.into();
    used["permissions"]["deny"].as_array_mut().unwrap().push("Edit(//var/lib/steplet/**)".into());
    assert_eq!(read(&settings_json(&steplet(""), &paths("p2"), &g, false)), used);
}

/// The Nebius plan, Task 5: Steplet's own agent runs every move inside a fence of its own, with the same read rules and
/// the same sites as Claude Code's fence; it may write the working copy and the run's temporary folder, but not the
/// copy's `.git` or any agent settings file.
#[test]
fn the_hands_fence_matches_claude_codes_fence() {
    let s = steplet(
        "allow:
  domains:
    - pypi.org
    - files.pythonhosted.org
",
    );
    let g = gate("/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd");
    let claude = read(&settings_json(&s, &paths("r1"), &g, false))["sandbox"].clone();
    let fence = read(&fence_json(&s, &paths("r1")));
    assert_eq!(fence["filesystem"]["denyRead"], claude["filesystem"]["denyRead"]);
    assert_eq!(fence["filesystem"]["allowRead"], claude["filesystem"]["allowRead"]);
    assert_eq!(fence["network"]["allowedDomains"], claude["network"]["allowedDomains"]);
    assert_eq!(
        fence,
        json!({
            "filesystem": {
                "denyRead": ["/mnt", "/var/lib/steplet", "/work"],
                "allowRead": ["/work/r1/repo"],
                "allowWrite": ["/work/r1/repo", "/tmp/steplet-run-r1"],
                "denyWrite": [
                    "/work/r1/repo/.git",
                    "/work/r1/repo/.claude",
                    "/work/r1/repo/.codex",
                    "/work/r1/repo/.cursor",
                    "/work/r1/repo/.gemini",
                    "/work/r1/repo/.mcp.json"
                ]
            },
            "network": {"allowedDomains": ["pypi.org", "files.pythonhosted.org"], "deniedDomains": []}
        })
    );
    let none = read(&fence_json(&steplet(""), &paths("r1")));
    assert_eq!(none["network"]["allowedDomains"], json!([]), "no declared site, no network");
}

#[test]
fn no_allow_rules_and_no_read_rule_on_the_work_folder() {
    // R4 and C9 on their own, so that a later change to the shape cannot quietly add either.
    let s = steplet("allow:\n  commands:\n    - cargo test\n  read:\n    - src\n  write:\n    - target\n");
    let g = gate("/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd");
    let v = read(&settings_json(&s, &paths("r1"), &g, false));
    let permissions = v["permissions"].as_object().unwrap();
    assert_eq!(permissions.keys().collect::<Vec<_>>(), ["deny"], "deny rules only (R4)");
    for rule in permissions["deny"].as_array().unwrap() {
        assert!(!rule.as_str().unwrap().starts_with("Read(//work"), "no Read rule on the work folder (C9): {rule}");
    }
    assert_eq!(v["sandbox"]["network"]["allowedDomains"], json!([]), "no declared site, no network");
    assert_eq!(
        v,
        read(&settings_json(&steplet(""), &paths("r1"), &g, false)),
        "the file's commands and folders reach the gate's checker only, never the settings"
    );
}

#[test]
fn the_mcp_config_names_the_gate_as_its_only_server() {
    let g = gate("/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd");
    assert_eq!(
        read(&mcp_json(&g)),
        json!({
            "mcpServers": {
                "gate": {
                    "type": "stdio",
                    "command": "/opt/steplet/gate/steplet-gate.exe",
                    "args": [
                        "--pipe", "steplet-r1", "--token", "0123abcd", "--log", LOG, "--deadline-ms", "30000",
                        "--steplet-pid", std::process::id().to_string()
                    ]
                }
            }
        })
    );
}

#[test]
fn a_run_id_that_could_reach_another_place_is_refused() {
    let long = "a".repeat(65);
    for bad in ["", "..", "a/b", "a b", "a;b", "R1", "-r1", "r1\n", long.as_str()] {
        assert!(RunPaths::new("{44f36236-99b1-42c0-84bd-7ff8392d51ba}", bad).is_err(), "run id {bad:?}");
    }
}

#[test]
fn a_distribution_id_must_be_a_windows_guid() {
    assert!(RunPaths::new("{44F36236-99B1-42C0-84BD-7FF8392D51BA}", "r1").is_ok(), "upper case is the same id");
    for bad in [
        "",
        "steplet",
        "-d",
        "44f36236-99b1-42c0-84bd-7ff8392d51ba",
        "{44f36236-99b1-42c0-84bd-7ff8392d51ba",
        "{44f36236-99b1-42c0-84bd-7ff8392d51bz}",
        "{44f3623699b1-42c0-84bd-7ff8392d51ba-}",
        "{44f36236-99b1-42c0-84bd-7ff8392d51ba} x",
    ] {
        assert!(RunPaths::new(bad, "r1").is_err(), "distribution id {bad:?}");
    }
}

#[test]
fn a_gate_value_that_a_shell_could_misread_is_refused() {
    let exe = "/opt/steplet/gate/steplet-gate.exe";
    let long = "t".repeat(129);
    assert!(GateLink::new(exe, "p", &"t".repeat(128), LOG, DEADLINE).is_ok(), "128 characters is the longest word");
    for (exe, pipe, token, log) in [
        (exe, "p", long.as_str(), LOG),
        ("steplet-gate.exe", "p", "t", LOG),
        ("/opt/steplet/../gate.exe", "p", "t", LOG),
        ("/opt/steplet gate/gate.exe", "p", "t", LOG),
        (exe, "", "t", LOG),
        (exe, "p q", "t", LOG),
        (exe, r"p\q", "t", LOG),
        (exe, "p", "", LOG),
        (exe, "p", "t;echo", LOG),
        (exe, "p", "$(t)", LOG),
        (exe, "p", "t", ""),
        (exe, "p", "t", "--key"),
        (exe, "p", "t", "C:\\logs\nx"),
    ] {
        assert!(GateLink::new(exe, pipe, token, log, DEADLINE).is_err(), "{exe:?} {pipe:?} {token:?} {log:?}");
    }
}

#[test]
fn a_gate_deadline_outside_a_tenth_of_a_second_to_an_hour_is_refused() {
    let exe = "/opt/steplet/gate/steplet-gate.exe";
    for ok in [Duration::from_millis(100), Duration::from_secs(3600)] {
        assert!(GateLink::new(exe, "p", "t", LOG, ok).is_ok(), "{ok:?}");
    }
    for bad in [Duration::ZERO, Duration::from_millis(99), Duration::from_millis(3_600_001)] {
        assert!(GateLink::new(exe, "p", "t", LOG, bad).is_err(), "{bad:?}");
    }
}

/// Task 5's review: the run's temporary folder is named in three of Steplet's scripts as well as in `RunPaths`; all
/// four must agree.
#[test]
fn the_scripts_and_the_paths_name_the_same_temporary_folder() {
    let tmp = paths("r1").run_tmp().to_owned();
    let prefix = tmp.strip_suffix("r1").unwrap();
    assert_eq!(prefix, "/tmp/steplet-run-");
    for (name, script) in [
        ("prepare.sh", include_str!("../src/prepare.sh")),
        ("start-run.sh", include_str!("../src/start-run.sh")),
        ("discard.sh", include_str!("../src/discard.sh")),
        ("root-common.sh", include_str!("../src/root-common.sh")),
        ("start-hands.sh", include_str!("../src/start-hands.sh")),
    ] {
        assert!(script.contains(&format!("{prefix}$")), "{name} names the run's temporary folder as {prefix}<id>");
    }
}

/// Task 13 (E1): the run stays on the model the receiver approved, unless they pressed "Run again, allowing another
/// model" after a refusal; only then may Claude Code switch models.
#[test]
fn switch_models_only_when_the_receiver_allowed_it() {
    let g = gate("/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd");
    let switch = |allowed| read(&settings_json(&steplet(""), &paths("r1"), &g, allowed))["switchModelsOnFlag"].clone();
    assert_eq!(switch(false), json!(false));
    assert_eq!(switch(true), json!(true));
}
