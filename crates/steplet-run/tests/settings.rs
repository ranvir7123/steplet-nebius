use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::{Steplet, parse};
use steplet_run::{GateLink, RunPaths, mcp_json, settings_json};

fn steplet(allow: &str) -> Steplet {
    let f = format!("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\n{allow}---\n");
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap()
}

fn paths(run_id: &str) -> RunPaths {
    RunPaths::new("{44f36236-99b1-42c0-84bd-7ff8392d51ba}", run_id).unwrap()
}

const LOG: &str = r"C:\Users\Test User\AppData\Local\Steplet\logs\r1.gate.log";

fn gate(exe: &str, pipe: &str, token: &str) -> GateLink {
    GateLink::new(exe, pipe, token, LOG).unwrap()
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
    assert_eq!(p.run_tmp(), "/var/lib/steplet/runs/r1/tmp");
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
}

#[test]
fn the_settings_are_exactly_the_planned_shape() {
    let s = steplet("allow:\n  domains:\n    - pypi.org\n    - files.pythonhosted.org\n");
    let g = gate("/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd");
    assert_eq!(
        read(&settings_json(&s, &paths("r1"), &g)),
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
                "deny": ["Read(//mnt/**)", "Read(//var/lib/steplet/**)", "Edit(//work/r1/repo/.git/**)"]
            },
            "switchModelsOnFlag": false,
            "apiKeyHelper": "/opt/steplet/gate/steplet-gate.exe --key --pipe steplet-r1 --token 0123abcd"
        })
    );
}

#[test]
fn the_settings_match_the_file_the_task_2_runs_used() {
    // The file Task 2's scripted and real-model runs used, with its token placeholder (spikes/p3/p2.settings.json).
    let g = gate("/opt/steplet/gate/spike-gate.exe", "steplet-p3-p2", "RUN_TOKEN");
    assert_eq!(read(&settings_json(&steplet(""), &paths("p2"), &g)), read(include_str!("support/task2.settings.json")));
}

#[test]
fn no_allow_rules_and_no_read_rule_on_the_work_folder() {
    // R4 and C9 on their own, so that a later change to the shape cannot quietly add either.
    let s = steplet("allow:\n  commands:\n    - cargo test\n  read:\n    - src\n  write:\n    - target\n");
    let g = gate("/opt/steplet/gate/steplet-gate.exe", "steplet-r1", "0123abcd");
    let v = read(&settings_json(&s, &paths("r1"), &g));
    let permissions = v["permissions"].as_object().unwrap();
    assert_eq!(permissions.keys().collect::<Vec<_>>(), ["deny"], "deny rules only (R4)");
    for rule in permissions["deny"].as_array().unwrap() {
        assert!(!rule.as_str().unwrap().starts_with("Read(//work"), "no Read rule on the work folder (C9): {rule}");
    }
    assert_eq!(v["sandbox"]["network"]["allowedDomains"], json!([]), "no declared site, no network");
    assert_eq!(
        v,
        read(&settings_json(&steplet(""), &paths("r1"), &g)),
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
                    "args": ["--pipe", "steplet-r1", "--token", "0123abcd", "--log", LOG]
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
    assert!(GateLink::new(exe, "p", &"t".repeat(128), LOG).is_ok(), "128 characters is the longest word");
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
        assert!(GateLink::new(exe, pipe, token, log).is_err(), "{exe:?} {pipe:?} {token:?} {log:?}");
    }
}
