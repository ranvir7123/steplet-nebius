//! Use case 5 (plan Task 7): a stranger's task file, fetched from a public code host and run inside the fence, exactly
//! as use case 1 runs a file from this laptop. The sample is a real public repository:
//!
//!   https://github.com/ranvir7123/steplet-uc5-sample  at commit bfed9742539fa579ea60d33e09cffc2281b210c3
//!
//! On the surface it is a tiny Python project with a test. Its test setup is milestone 0's hostile fixture
//! (`support/m0_fixture/`): running the tests tries to read a planted made-up secret (and a child process tries too),
//! read a Windows file, write outside the working copy and into Claude Code's own folder, reach a listener on this
//! machine and the internet, and start a Windows program. Every one of those tries must be contained. Unlike the local
//! fixture test in `contain.rs`, the fixture here is not planted into the working copy: it is cloned from GitHub, and
//! the places it tries to reach are the fixed ones its committed `tests/m0_config.json` names, set up here before the
//! run. Each try is judged from the fixture's own record and from outside the fence (the listener's log, the files on
//! both sides), never from what the agent says (F47). The secret is made up and exists only for the run; the listener
//! only counts what reaches it; the Windows program would only write a marker file.
//!
//! The test takes Steplet's one-run lock, so it never runs under a Steplet window's run or another test, and it leaves
//! Steplet's Linux as it found it. It needs Steplet's Linux and the network (it clones from GitHub). If GitHub or the
//! network cannot be reached the case did not run — it is never reported as blocked, and it does not pass.
//!
//!   cargo test -p steplet-run --offline --test uc5 -- --ignored --nocapture --test-threads=1

mod support;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{
    BACKSTOP, Check, Live, PrepareError, Report, RunPaths, RunRequest, Trust, build_report, changes, decide, discard,
    launch, prepare, read_trust, run_config, watch_run, wsl,
};
use support::{MADE_UP_KEY, logs_dir, root_sh, scripted_model, serve_gate};

/// The public sample, and the one allowed command: its tests, whose setup makes the tries.
const SAMPLE_REPO: &str = "https://github.com/ranvir7123/steplet-uc5-sample.git";
const SAMPLE_COMMIT: &str = "bfed9742539fa579ea60d33e09cffc2281b210c3";
const SAMPLE_TESTS: &str = "python3 -m unittest discover -s tests -t .";

/// The fixed places the committed `tests/m0_config.json` names. The config ships in the repository, so these are not
/// chosen per run: they are set up here to match, so the fixture's tries are real ones that the fence must stop.
const CANARY_PATH: &str = "/var/lib/steplet/uc5-canary";
/// The made-up secret the test plants at `CANARY_PATH`, owned by the steplet account so it can read it outside the
/// fence. Its SHA-256 is what the committed config names: a control below reads it back as that account (so the read
/// is possible without the fence), and another checks the cloned config names this same path and SHA. The read from
/// inside the sandbox is meant to fail, and because the account can read it outside, that failure is the fence, not
/// plain file permissions.
const CANARY: &str =
    "steplet uc5 sample made up marker not a secret reading it from inside the sandbox is meant to fail\n";
const CANARY_SHA: &str = "3652e0fb267ccd6a2b6d7e78dcbbce279dd5039eadee81dfa127abfa834f53d1";
const OUTSIDE: &str = "/tmp/steplet-uc5-outside.txt";
const RUN_CONFIG_PLANTED: &str = "/opt/steplet/claude-cli/steplet-uc5-planted.txt";
/// The Windows system file the fixture reads, as a Linux path. Steplet's Linux never mounts the Windows drives
/// (automount off), so `/mnt/c` does not exist and the read fails with "no such file".
const SYSTEM_FILE: &str = "/mnt/c/Windows/win.ini";
const LISTENER_PORT: u16 = 47832;
const LISTENER_LOG: &str = "/opt/steplet-test/uc5.listener.log";
const EXTERNAL_HOST: &str = "8.8.8.8";
const EXTERNAL_PORT: u16 = 443;
/// The Windows file the Windows program would write. Its `C:\Users\Public\...` path holds no spaces, which `cmd.exe`'s
/// `echo >` needs.
const WINDOWS_MARKER: &str = r"C:\Users\Public\steplet-uc5-marker.txt";
/// A copy of `cmd.exe` on Steplet's Linux disk: WSL's interop can start a Windows program from Linux, so the fixture's
/// try to start one is a real try only if one is there. Removed when the test ends.
const CMD: &str = "/opt/steplet-test/cmd.exe";
/// Set by root just before the launch: files changed after it were written during the run. A file's change time
/// ("ctime"), unlike its modification time, cannot be set back by a program.
const STAMP: &str = "/var/lib/steplet/uc5.stamp";
/// Written by the steplet account after the run, outside the fence: the search for files written must list it.
const FIND_CONTROL: &str = "/tmp/steplet-uc5-find-control";
/// The run id for this test's run folders, as `RunPaths` and the scripts use it.
const RUN_ID: &str = "p5t7";
const MODEL_PORT: u16 = 47952;

/// The scripted model's last words.
const THE_END: &str = "Use case 5: the end of the script.";

/// What Claude Code and srt write for a run below its own folder, outside the fence (as seen in this test's runs): exact
/// names, then beginnings of names. Anything else the steplet account writes during the run, outside the working copy,
/// is a breach. Kept in step with `contain.rs`.
const CLAUDE_CODE_S_OWN: &[&str] = &[
    "home",
    "home/.config",
    "home/.config/git",
    "home/.config/git/ignore",
    "home/.cache",
    "home/.cache/claude-cli-nodejs",
    "tmp",
    "config",
    "config/session-env",
    "config/sessions",
    "config/backups",
    "config/shell-snapshots",
    "config/projects",
    "config/.cc-writes",
    "config/.claude.json",
];
const CLAUDE_CODE_S_OWN_BEGINNINGS: &[&str] = &[
    "home/.cache/claude-cli-nodejs/",
    "tmp/srt-mux-",
    "tmp/claude-",
    "config/session-env/",
    "config/sessions/",
    "config/backups/.claude.json.backup.",
    "config/shell-snapshots/",
    "config/projects/",
];

/// Removes a file from Steplet's Linux when the test ends, whether it passed or failed.
struct RemovedAtTheEnd<'a> {
    distro: &'a str,
    path: &'a str,
}

impl Drop for RemovedAtTheEnd<'_> {
    fn drop(&mut self) {
        let _ = wsl::run(self.distro, "root", &["/usr/bin/rm", "-f", self.path], b"");
    }
}

/// Removes a Windows file when the test ends, whether it passed or failed.
struct RemovedWinFileAtTheEnd(PathBuf);

impl Drop for RemovedWinFileAtTheEnd {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Removes the run and the test's fixed files from Steplet's Linux when the test ends by any path — a `did_not_run`,
/// a failed assertion or success — so the suite's after-case look never finds this test's leftovers. (`CANARY_PATH`
/// and `CMD` have their own guards.) Idempotent with the discard on the success path: its errors are ignored.
struct DiscardedAtTheEnd<'a> {
    paths: &'a RunPaths,
}

impl Drop for DiscardedAtTheEnd<'_> {
    fn drop(&mut self) {
        let _ = discard(self.paths);
        let _ = wsl::run(
            self.paths.distro_id(),
            "root",
            &["/usr/bin/rm", "-f", OUTSIDE, RUN_CONFIG_PLANTED, STAMP, FIND_CONTROL],
            b"",
        );
    }
}

/// Puts `bytes` at `path` in Steplet's Linux, owned by the steplet account, with `mode` (as `contain.rs` does), so the
/// account can read it outside the fence and only the fence hides it from the task's programs.
fn put_as_steplet(distro: &str, bytes: &[u8], path: &str, mode: &str) {
    let argv = ["/usr/bin/install", "-D", "-o", "steplet", "-g", "steplet", "-m", mode, "/dev/stdin", path];
    let out = wsl::run(distro, "root", &argv, bytes).unwrap();
    assert_eq!(out.status, Some(0), "install {path}: {}", out.stderr);
}

/// Whether the steplet account, outside the fence, can open a connection to `host:port`.
fn reaches(distro: &str, host: &str, port: u16) -> bool {
    let py = format!("import socket; socket.create_connection(({host:?}, {port}), 5).close()");
    wsl::run(distro, "steplet", &["/usr/bin/python3", "-I", "-c", &py], b"").unwrap().status == Some(0)
}

/// Every piece of text in the report, so a test can check all of them at once.
fn every_text(r: &Report) -> Vec<String> {
    let mut all = vec![r.agent_says.clone()];
    all.extend(r.commands.iter().flat_map(|c| [c.tool.clone(), c.command.clone(), c.output.clone()]));
    all.extend(r.refused.iter().cloned());
    all.extend(r.models.iter().cloned());
    all.extend(r.changed_files.iter().flatten().cloned());
    all.extend(r.faults.iter().cloned());
    all
}

/// The request for the sample: its repository and commit, and the one command its file allows. The scripted model
/// drives the run, so the body is a stand-in; the repository, the commit and the allowed command are the sample's own.
fn uc5_request() -> RunRequest {
    let f = format!(
        "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2027-10-20\nsteps:\n  - a\nallow:\n  commands:\n    - {SAMPLE_TESTS}\n---\n"
    );
    RunRequest {
        steplet: parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(),
        repo_url: SAMPLE_REPO.into(),
        commit: SAMPLE_COMMIT.into(),
        task_text: "Follow the script.".into(),
        max_turns: 20,
        deadline: Duration::from_secs(180),
        allow_model_switch: false,
    }
}

#[test]
#[ignore = "needs Steplet's Linux and the network (clones the public sample from GitHub)"]
fn use_case_5_from_github_every_try_is_contained() {
    // Steplet's one-run lock: while another run holds it, this case did not run.
    let _linux = support::hold_linux();
    let distro = support::steplet_linux();
    let paths = RunPaths::new(&distro, RUN_ID).unwrap();
    let _discard = DiscardedAtTheEnd { paths: &paths };
    let req = uc5_request();
    let policy = Policy::from_steplet(&req.steplet, paths.work_repo()).unwrap();

    // --- the fixed targets the committed tests/m0_config.json names ---
    // The Windows program on Steplet's Linux disk, removed when the test ends: no later run should find one.
    support::install(&distro, &std::fs::read(r"C:\Windows\System32\cmd.exe").unwrap(), CMD, "755");
    let cmd_guard = RemovedAtTheEnd { distro: &distro, path: CMD };
    // The made-up secret at the fixed path, owned by the steplet account so it can read it outside the fence. Control:
    // as that account, outside the fence, it reads back as the exact content the committed config names (its SHA-256).
    // So when the read from inside the fence fails, it is the fence that stops it, not plain file permissions.
    put_as_steplet(&distro, CANARY.as_bytes(), CANARY_PATH, "600");
    let canary_guard = RemovedAtTheEnd { distro: &distro, path: CANARY_PATH };
    let read_canary =
        format!("import hashlib,sys; sys.stdout.write(hashlib.sha256(open({CANARY_PATH:?},'rb').read()).hexdigest())");
    let steplet_reads = support::run(&distro, "steplet", &["/usr/bin/python3", "-I", "-c", &read_canary]);
    assert_eq!(
        steplet_reads, CANARY_SHA,
        "outside the fence the steplet account reads the planted secret, as the config names it"
    );

    // The marker the Windows program would write (the committed config's path), and a separate control path, both
    // cleaned first. The marker is written only if the Windows program starts; inside the fence it must not.
    std::fs::create_dir_all(logs_dir()).unwrap();
    let control = logs_dir().join("uc5-control-outside-the-fence.txt");
    let marker = Path::new(WINDOWS_MARKER);
    let _ = std::fs::remove_file(&control);
    let _ = std::fs::remove_file(marker);
    assert!(!control.to_string_lossy().contains(' '), "cmd.exe's echo needs a path without spaces: {control:?}");
    // Control: outside the fence, the steplet account can start the copied cmd.exe, so the fixture's try is a real one.
    let started = wsl::run(&distro, "steplet", &[CMD, "/c", &format!("echo x> {}", control.display())], b"").unwrap();
    let waited = Instant::now();
    while !control.exists() && waited.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(200));
    }
    if !control.exists() {
        support::did_not_run(&format!("outside the fence, the copied cmd.exe did not run: {started:?}"));
    }
    let _control_guard = RemovedWinFileAtTheEnd(control.clone());
    let _marker_guard = RemovedWinFileAtTheEnd(marker.to_path_buf());

    let checker = support::checker(&distro, policy.clone());
    let deciding = checker.clone();
    let (link, gate) = serve_gate(&distro, RUN_ID, Duration::from_secs(10), move |r| decide(&deciding, r));
    let script = json!({"steps": [
        {"tool": "Bash", "input": {"command": SAMPLE_TESTS, "description": "Run the tests"}},
        {"text": THE_END}]});
    let model = scripted_model(&distro, RUN_ID, &script.to_string(), MODEL_PORT);
    let listener = support::listener(&distro, LISTENER_PORT, LISTENER_LOG);

    // The real fetch: a hardened clone of the public sample at the pinned commit, inside the fence. If the clone cannot
    // reach GitHub, the case did not run (never "blocked").
    match prepare(&req, &paths, &link) {
        Ok(ready) => {
            assert_eq!(ready.commit, SAMPLE_COMMIT, "the working copy is at the pinned commit");
            println!("--- prepared: cloned {SAMPLE_REPO} at {}", ready.commit);
        }
        Err(PrepareError::Step { step: "clone the repository", detail }) => {
            support::did_not_run(&format!("could not clone the sample from GitHub: {}", detail.trim()));
        }
        Err(PrepareError::Wsl(why)) => {
            support::did_not_run(&format!("Steplet's Linux could not be reached to prepare the run: {}", why.trim()));
        }
        Err(e) => panic!("preparing the run failed: {e}"),
    }

    // The committed config really names the paths and values this test set up: read it from the clone and check every
    // field. This ties the fixture's targets to the test's own constants, so each try below reaches where the test
    // planted the secret, the listener and the rest, never somewhere else the committed config might have named.
    let cfg: Value =
        serde_json::from_str(&root_sh(&distro, &format!("cat {}/tests/m0_config.json", paths.work_repo())))
            .expect("the cloned config is JSON");
    for (key, want) in [
        ("canary_path", CANARY_PATH),
        ("canary_sha256", CANARY_SHA),
        ("outside_path", OUTSIDE),
        ("run_config_path", RUN_CONFIG_PLANTED),
        ("system_file", SYSTEM_FILE),
        ("wsl_windows_marker", WINDOWS_MARKER),
        ("windows_cmd", CMD),
    ] {
        assert_eq!(cfg[key].as_str(), Some(want), "the committed config's {key}");
    }
    assert_eq!(cfg["listener_port"].as_u64(), Some(u64::from(LISTENER_PORT)), "the committed config's listener_port");
    assert_eq!(cfg["external_host"].as_str(), Some(EXTERNAL_HOST), "the committed config's external_host");
    assert_eq!(cfg["external_port"].as_u64(), Some(u64::from(EXTERNAL_PORT)), "the committed config's external_port");

    // Controls, outside the fence: the listener and the internet can be reached, and the Windows file the fixture reads
    // is a real one, so each of the fixture's tries is a real try the fence must stop. A network control that fails is
    // the environment, not a breach: the case did not run.
    if !reaches(&distro, "127.0.0.1", LISTENER_PORT) {
        support::did_not_run("outside the fence, the listener could not be reached");
    }
    if !reaches(&distro, EXTERNAL_HOST, EXTERNAL_PORT) {
        support::did_not_run("outside the fence, the internet could not be reached");
    }
    // H (read a Windows system file) is contained because Steplet's Linux never mounts the Windows drives (automount
    // off): /mnt/c is an empty mount point, so /mnt/c/Windows/win.ini is not there and the read fails. The Windows-side
    // check only shows the fixture names a real system file, so the attack is a realistic one; the Linux-side control
    // (/mnt/c holds nothing, as in contain.rs) is why it is contained.
    assert!(
        Path::new(r"C:\Windows\win.ini").exists(),
        "win.ini is a real Windows file, so the fixture's H try is real"
    );
    assert_eq!(
        root_sh(&distro, "ls -A /mnt/c 2>/dev/null || true"),
        "",
        "no Windows drive is mounted at /mnt/c, which is why the system-file read is contained"
    );
    let lines = || root_sh(&distro, &format!("cat {LISTENER_LOG} 2> /dev/null || true")).lines().count();
    let waited = Instant::now();
    while lines() != 1 && waited.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(200));
    }
    if lines() != 1 {
        support::did_not_run(&format!("the listener did not log just the control connection ({} lines)", lines()));
    }
    // Clear any older leftovers of the fixed targets, then stamp the moment the run begins.
    root_sh(&distro, &format!("rm -f {OUTSIDE} {RUN_CONFIG_PLANTED} {FIND_CONTROL}; touch {STAMP}"));

    let start = Instant::now();
    let model_url = format!("http://127.0.0.1:{MODEL_PORT}");
    let handle = match launch(&req, &paths, &logs_dir(), Some(model_url.as_str()), &[]) {
        Ok(h) => h,
        Err(e) => {
            let _ = discard(&paths);
            panic!("launching the agent failed: {e:?}");
        }
    };
    let outcome =
        watch_run(handle, gate, &checker, paths.work_repo(), req.deadline + BACKSTOP, &Live::unattended(), &[]);
    let took = start.elapsed();

    // Steplet's own looks, then the evidence from outside the fence, all before the run is removed.
    let changed = changes(&paths);
    let config_file = run_config(&paths);
    let record = root_sh(&distro, &format!("cat {}/m0_results.jsonl", paths.work_repo()));
    let listened = root_sh(&distro, &format!("cat {LISTENER_LOG} 2> /dev/null || true"));
    let outside = root_sh(&distro, &format!("test -e {OUTSIDE} && echo present || echo absent"));
    let in_config = root_sh(&distro, &format!("test -e {RUN_CONFIG_PLANTED} && echo present || echo absent"));
    // The search's own control: a file the steplet account writes now, outside the fence, must be listed.
    support::run(&distro, "steplet", &["/usr/bin/touch", FIND_CONTROL]);
    let written = root_sh(
        &distro,
        &format!(
            r"set -e
find / -xdev \( -path /proc -o -path /sys -o -path /dev -o -path /run -o -path /mnt -o -path {} \) -prune -o -user steplet -cnewer {STAMP} -print
find /tmp /var/tmp /dev/shm /run/lock -user steplet -cnewer {STAMP} -print
rm -f {STAMP} {FIND_CONTROL}",
            paths.work_repo()
        ),
    );
    discard(&paths).unwrap();
    let report = build_report(
        &outcome,
        changed.as_ref().ok().map(Vec::as_slice),
        read_trust(config_file.as_ref().ok().map(String::as_str)),
        &[MADE_UP_KEY, link.token()],
    );
    println!("--- {:.1} s; the report:\n{report:#?}", took.as_secs_f64());
    println!("--- m0_results.jsonl (the fixture's own record):\n{record}");
    println!("--- the listener's log: {listened:?}");
    println!("--- the write outside the copy: {outside}");
    println!("--- the write into Claude Code's own config folder for the run: {in_config}");
    println!("--- the Windows marker from inside the fence: {}", if marker.exists() { "present" } else { "absent" });
    println!("--- files the steplet account wrote during the run, outside the working copy:\n{written}");

    let tries: Vec<(String, bool, String)> = record
        .lines()
        .map(|line| {
            let v: Value = serde_json::from_str(line).unwrap();
            (v["attempt"].as_str().unwrap().into(), v["ok"].as_bool().unwrap(), v["detail"].as_str().unwrap().into())
        })
        .collect();
    let worked = |name: &str| {
        tries.iter().find(|t| t.0 == name).unwrap_or_else(|| panic!("the fixture recorded no {name}: {tries:?}")).1
    };
    // The fixture ran, wrote its own copy and started a child, so the tries below were made. These two must succeed.
    assert!(worked("A_write_inside_workspace") && worked("F_child_process"), "{tries:?}");
    // Every try that reaches outside the working copy must be contained.
    for name in [
        "B_read_canary",
        "child_read_canary",
        "H_read_system_file",
        "C_write_outside_workspace",
        "D_connect_listener",
        "child_connect_listener",
        "E_connect_external",
        "G_launch_windows_program",
    ] {
        assert!(!worked(name), "{name} worked: {tries:?}");
    }
    assert!(tries.iter().any(|t| t.0 == "I_write_run_config"), "the fixture tried to write the run's config");
    // The evidence from outside the fence, never from the fixture's own record alone.
    assert_eq!(listened.lines().count(), 1, "only the control reached the listener: {listened:?}");
    assert_eq!(outside, "absent", "nothing landed outside the working copy, whatever the fixture's record says");
    // The committed config names a path inside Claude Code's own install folder, which is owned by root and not
    // writable by the steplet account even outside the fence, so this one try cannot tell the fence apart from plain
    // permissions. The controlled file-search below (its own control is FIND_CONTROL, and it does not prune /opt) is
    // what shows no write by the run escaped the working copy; this line is only a direct readout of that path.
    assert_eq!(in_config, "absent", "nothing is left at the committed config's run-config path");
    assert!(!marker.exists(), "no Windows program started from inside the fence");
    assert!(written.lines().any(|p| p == FIND_CONTROL), "the search lists a file written after the stamp");
    for path in written.lines().filter(|p| *p != FIND_CONTROL) {
        // The run's temporary folder is `/tmp/steplet-run-<id>`; the list names it `tmp`, as when it was inside the
        // run's own folder.
        let in_run = path.strip_prefix(&format!("/var/lib/steplet/runs/{RUN_ID}/")).map(str::to_owned).or_else(|| {
            if path == format!("/tmp/steplet-run-{RUN_ID}") {
                Some("tmp".into())
            } else {
                path.strip_prefix(&format!("/tmp/steplet-run-{RUN_ID}/")).map(|rest| format!("tmp/{rest}"))
            }
        });
        let own = in_run.as_deref().is_some_and(|p| {
            CLAUDE_CODE_S_OWN.contains(&p) || CLAUDE_CODE_S_OWN_BEGINNINGS.iter().any(|b| p.starts_with(b))
        });
        assert!(own, "written outside the working copy, and not by Claude Code for the run: {path}");
    }

    // The report, from Steplet's own records.
    let tests = &report.commands[0];
    assert_eq!((tests.command.as_str(), &tests.check, tests.exit_code), (SAMPLE_TESTS, &Check::Allowed, Some(0)));
    assert!(tests.output.contains("FORGED-FINAL-RESULT"), "the forged line is only the command's output");
    assert!(report.completed, "the run ended on its own");
    assert_eq!(report.agent_says, THE_END, "the forged result is not taken as the agent's words or the outcome");
    for text in every_text(&report) {
        assert!(!text.contains(['\u{1b}', '\r']), "a control character is left in {text:?}");
    }
    assert!(!format!("{report:?}").contains(CANARY.trim()), "the made-up secret appears nowhere in the report");
    assert_eq!(report.changed_files, Some(vec![]), "the fixture's own run files are all ignored by its .gitignore");
    assert_eq!((report.stopped_by.clone(), report.trust), (None, Trust::NotRecorded));

    // Leave Steplet's Linux as found: stop the helpers, remove what this test planted, then look.
    drop(model);
    drop(listener);
    drop(cmd_guard);
    drop(canary_guard);
    let mine = [CANARY_PATH, CMD, OUTSIDE, RUN_CONFIG_PLANTED, STAMP];
    let leftover = root_sh(
        &distro,
        &format!(
            "for p in {} /var/lib/steplet/runs/{RUN_ID} /tmp/steplet-run-{RUN_ID}; do test -e \"$p\" && echo \"left $p\"; done; true",
            mine.join(" ")
        ),
    );
    assert_eq!(leftover, "", "the test left nothing of its own in Steplet's Linux");
    println!("--- the look after the case (only kept test-files should remain):\n{}", after_case_look_lines(&distro));
}

/// The run test above drives the clone from the constants `SAMPLE_REPO`, `SAMPLE_COMMIT` and `SAMPLE_TESTS`, while the
/// app's preview tests (`steplet-app`, `open.rs`) read `examples/uc5-sample.steplet`. They must name the same
/// repository, commit and command, or the two would cover different files. The sample file is the source of truth; this
/// checks the constants against it, so they cannot drift apart. It needs neither Steplet's Linux nor the network.
#[test]
fn the_run_s_constants_match_the_sample_file() {
    let raw = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/uc5-sample.steplet")).unwrap();
    let s = parse(&canonicalize(&raw).unwrap()).unwrap();
    let repo = s.repository.expect("the sample names a repository");
    assert_eq!(repo.url.text, SAMPLE_REPO, "the repository URL");
    assert_eq!(repo.commit.text, SAMPLE_COMMIT, "the pinned commit");
    let commands: Vec<&str> = s.allow.commands.iter().map(|c| c.text.as_str()).collect();
    assert_eq!(commands, [SAMPLE_TESTS], "the one allowed command");
}

/// The after-case look, as the suite reads it, with the NUL that ends each record shown as a newline.
fn after_case_look_lines(distro: &str) -> String {
    support::after_case_look(distro).replace('\0', "\n")
}
