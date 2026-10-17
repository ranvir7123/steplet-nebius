# Milestone 1, phase 4: results

The plan: `docs/plans/2026-10-18-milestone-1-phase-4.md`, approved on 18 Oct 2026 with the recommended option for every choice (J1 A, J2 B, J3 A to J9 A; S1 to S13 kept). Every command below was run on this laptop (Windows 11 Home, WSL 2.6.3, Claude Code 2.1.220 inside Steplet's own Linux), and its output is pasted unchanged. Where only part of a long output is shown, the text says so; the parts shown are not edited.

## Task 1: the agent's file tools cannot write outside the working copy (J1), and Steplet's own folder is denied

### Changes from the plan, recorded here

- **The order of the three calls in the new containment test.** The plan had the write into Steplet's own folder first. But Steplet's watcher stops a run as soon as Claude Code refuses a call by Steplet's own deny rules (decision H11), so the other two calls would never have been made. That write now comes last, and the test also checks that the run stops there, as H11 says it must.
- **`docs/rules.md` is not changed.** The plan asked for a reason line under `cap.outside-path`. But that page is written by a test from the rule list (`tests/rules_doc.rs`, "do not edit it by hand"), and it holds each rule's meaning and advice, not the checker's reasons. The checker's new reason is in the code and in this file; decision J1 goes into `docs/decisions.md` at the wrap-up.
- **The Task 2 settings test** (`the_settings_match_the_file_the_task_2_runs_used`) compares the settings with the file phase 3's Task 2 runs used. It now adds the new deny rule to that file's list before comparing, with a comment saying why; the rest is unchanged.

### Step 1: where the `steplet` account can write, outside a run

A new script, `crates/steplet-run/tests/support/writable.sh`, run as `steplet`, lists every folder that account can write on every mounted file system except `/proc`, `/sys` and `/dev/pts`. It changes nothing.

```
> cargo test -p steplet-run --offline --test contain where_steplet_can_write -- --ignored --nocapture
     Running tests\contain.rs (target\debug\deps\contain-e9ca5c3e4707e558.exe)

running 1 test
--- folders the steplet account can write:
account: steplet
/dev/mqueue
/dev/shm
/mnt/c
/run/lock
/run/screen
/run/shm
/tmp
/var/crash
/var/tmp
end of list

test where_steplet_can_write ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 8.62s
```

The four the plan expected are there (`/tmp`, `/var/tmp`, `/dev/shm`, `/run/lock`), plus `/dev/mqueue`, `/run/screen`, `/run/shm`, `/var/crash` and **`/mnt/c`**. `/mnt/c` is an empty folder in Steplet's own Linux, not your Windows drive (phase 3's tests check that it is empty and that no Windows drive is mounted), but anyone can write into it. Nothing under `/var/lib/steplet` or `/work` is listed: outside a run, `steplet` can write neither. Each of these places outlives a run, which is why a yes to an agent's file-tool write outside the working copy would have been a risk.

### Step 2 and 3: the tests first, and seeing them fail

New tests: `a_file_tool_write_outside_the_working_copy_is_refused_without_a_question` (`crates/steplet-core/tests/policy.rs`), the fourth deny rule in the two exact settings tests (`crates/steplet-run/tests/settings.rs`), and `a_write_into_steplets_own_folder_is_refused_before_the_gate` (`crates/steplet-run/tests/contain.rs`, needs Steplet's Linux).

```
> cargo test -p steplet-core --offline --test policy
test sites_and_bad_work_folders ... ok
test commands ... ok
test files ... ok
test a_file_tool_write_outside_the_working_copy_is_refused_without_a_question ... FAILED

failures:

---- a_file_tool_write_outside_the_working_copy_is_refused_without_a_question stdout ----

thread 'a_file_tool_write_outside_the_working_copy_is_refused_without_a_question' (36464) panicked at crates\steplet-core\tests\policy.rs:98:5:
assertion `left == right` failed
  left: Ask { reason: "this path is outside the repository" }
 right: Deny { rule: "cap.outside-path", reason: "the agent's file tools may change files only inside the working copy" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_file_tool_write_outside_the_working_copy_is_refused_without_a_question

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```
> cargo test -p steplet-run --offline --test settings
(the two failures, each line as printed)
---- the_settings_match_the_file_the_task_2_runs_used stdout ----
thread 'the_settings_match_the_file_the_task_2_runs_used' (39552) panicked at crates\steplet-run\tests\settings.rs:96:5:
assertion `left == right` failed
  left: Object {"apiKeyHelper": String("/opt/steplet/gate/spike-gate.exe --key --pipe steplet-p3-p2 --token RUN_TOKEN --deadline-ms 30000"), "permissions": Object {"deny": Array [String("Read(//mnt/**)"), String("Read(//var/lib/steplet/**)"), String("Edit(//work/p2/repo/.git/**)")]}, ...
 right: Object {"apiKeyHelper": String("/opt/steplet/gate/spike-gate.exe --key --pipe steplet-p3-p2 --token RUN_TOKEN --deadline-ms 30000"), "permissions": Object {"deny": Array [String("Read(//mnt/**)"), String("Read(//var/lib/steplet/**)"), String("Edit(//work/p2/repo/.git/**)"), String("Edit(//var/lib/steplet/**)")]}, ...
---- the_settings_are_exactly_the_planned_shape stdout ----
thread 'the_settings_are_exactly_the_planned_shape' (41496) panicked at crates\steplet-run\tests\settings.rs:57:5:
assertion `left == right` failed
  left: Object {... "permissions": Object {"deny": Array [String("Read(//mnt/**)"), String("Read(//var/lib/steplet/**)"), String("Edit(//work/r1/repo/.git/**)")]}, ...
 right: Object {... "permissions": Object {"deny": Array [String("Read(//mnt/**)"), String("Read(//var/lib/steplet/**)"), String("Edit(//work/r1/repo/.git/**)"), String("Edit(//var/lib/steplet/**)")]}, ...
test result: FAILED. 8 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(The two `left`/`right` lines are long; the `...` marks where I cut each one. Everything cut is the same on both sides.)

The containment test was also run before the change, which the plan did not ask for. It fails, and it is a control: without the new rule the fence's temporary folder works, and it is **inside Steplet's own folder**, which the new rule covers.

```
> cargo test -p steplet-run --offline --test contain a_write_into_steplets_own_folder -- --ignored --nocapture
(the report's three calls, then the failure; unchanged)
        ReportCommand {
            tool: "Write",
            command: "/tmp/steplet-probe.txt",
            check: Refused,
            exit_code: None,
            output: "this needs the receiver's yes, and no one could answer: this path is outside the repository",
            after_stop: false,
        },
        ReportCommand {
            tool: "Bash",
            command: "python3 -c \"import tempfile; print(tempfile.mkstemp()[1])\"",
            check: Allowed,
            exit_code: Some(
                0,
            ),
            output: "/var/lib/steplet/runs/p4t1/tmp/claude-999/tmpzkobw9_6",
            after_stop: false,
        },
        ReportCommand {
            tool: "Write",
            command: "/var/lib/steplet/runs/p4t1/config/settings.json",
            check: Refused,
            exit_code: None,
            output: "this needs the receiver's yes, and no one could answer: this path is outside the repository",
            after_stop: false,
        },
...
thread 'a_write_into_steplets_own_folder_is_refused_before_the_gate' (38708) panicked at crates\steplet-run\tests\contain.rs:468:5:
the write to /tmp reached the gate and was refused with J1's reason: [("/tmp/steplet-probe.txt", "Deny { message: \"this needs the receiver's yes, and no one could answer: this path is outside the repository\" }"), ("", "Allow { updated_input: Object {\"command\": String(\"python3 -c \\\"import tempfile; print(tempfile.mkstemp()[1])\\\"\"), \"description\": String(\"Make a temporary file\")} }"), ("/var/lib/steplet/runs/p4t1/config/settings.json", "Deny { message: \"this needs the receiver's yes, and no one could answer: this path is outside the repository\" }")]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 15.94s
```

### Step 4: the change

- `crates/steplet-core/src/policy.rs`: a write by the agent's file tools that is not a plain absolute path, or is outside the working copy and not in an always-blocked place, is now `Deny { rule: "cap.outside-path", reason: "the agent's file tools may change files only inside the working copy" }`. A write into an always-blocked place keeps `block.secret-path`. Reads are unchanged: outside the copy they still ask.
- `crates/steplet-run/src/settings.rs`: a fourth deny rule, `Edit(//var/lib/steplet/**)`, so Claude Code itself refuses a write into Steplet's own folder before any question.

### Step 5: the tests pass, and nothing else moved

```
> cargo test -p steplet-core --offline
(every summary line, as printed)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests\corpus.rs (target\debug\deps\corpus-8220497a53785d57.exe)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\policy.rs (target\debug\deps\policy-51ac68b37eecea73.exe)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.74s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The corpus test (`every_file_gives_its_expected_json` among its 5) is unchanged and passes: the rating does not use this path. The runner's tests that need no Linux pass too (`cargo test -p steplet-run --offline`: 13 summary lines, every one `ok`, `0 failed`; 76 passed, 24 ignored: the Linux-only ones).

```
> cargo test -p steplet-run --offline --test contain -- --ignored --nocapture --test-threads=1
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 40.43s
```

The new containment test's records, unchanged (the gate's records first, without the key request; then the report):

```
        request: GateRequest {
            run_token: "(hidden)",
            tool_name: "Write",
            input: Object {
                "content": String("made-up test text\n"),
                "file_path": String("/tmp/steplet-probe.txt"),
            },
            tool_use_id: "toolu_test_01bb619b99144f9fb6eb",
        },
        answer: Deny {
            message: "the agent's file tools may change files only inside the working copy",
        },
...
        request: GateRequest {
            run_token: "(hidden)",
            tool_name: "Bash",
            input: Object {
                "command": String("python3 -c \"import tempfile; print(tempfile.mkstemp()[1])\""),
                "description": String("Make a temporary file"),
            },
            tool_use_id: "toolu_test_e84e227b8ae54bec9c2b",
        },
        answer: Allow {
...
--- the report:
Report {
    completed: false,
    stopped_by: Some(
        SkippedGate(
            "Write /var/lib/steplet/runs/p4t1/config/settings.json",
        ),
    ),
    commands: [
        ReportCommand {
            tool: "Write",
            command: "/tmp/steplet-probe.txt",
            check: Refused,
            exit_code: None,
            output: "the agent's file tools may change files only inside the working copy",
            after_stop: false,
        },
        ReportCommand {
            tool: "Bash",
            command: "python3 -c \"import tempfile; print(tempfile.mkstemp()[1])\"",
            check: Allowed,
            exit_code: Some(
                0,
            ),
            output: "/var/lib/steplet/runs/p4t1/tmp/claude-999/tmpy48gtend",
            after_stop: false,
        },
        ReportCommand {
            tool: "Write",
            command: "/var/lib/steplet/runs/p4t1/config/settings.json",
            check: NotAsked {
                harmless: false,
            },
            exit_code: None,
            output: "<tool_use_error>File is in a directory that is denied by your permission settings.</tool_use_error>",
            after_stop: false,
        },
    ],
    refused: [
        "Write /tmp/steplet-probe.txt: the agent's file tools may change files only inside the working copy",
    ],
...
--- afterwards, /tmp/steplet-probe.txt: absent; /var/lib/steplet/runs/p4t1/config/settings.json: absent
ok
```

What this shows:
- The write to `/tmp` reached the gate, and the checker refused it with J1's reason, without a question.
- **The fence's own temporary folder still works with the new rule** on Claude Code 2.1.220: the declared command made its file in `/var/lib/steplet/runs/p4t1/tmp/claude-999/` and ended with exit code 0. So the plan's "if the tempfile command fails" stop did not happen.
- The write into the run's Claude Code config folder never reached the gate: Claude Code refused it itself ("File is in a directory that is denied by your permission settings"), and Steplet stopped the run there, as H11 says.
- Neither file exists afterwards (checked as root before the run's folders were removed).

```
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.20s
clippy exit 0
```

### What this task did not verify

- Whether the merged rule takes the fence's temporary folder away on Claude Code versions after 2.1.220 (the documentation describes the newest one).
- Writes by the agent's file tools through a link inside the working copy that points out of it: that is phase 3's Task 3, still deferred on your word.
