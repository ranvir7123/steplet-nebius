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

### The code review, and what changed

A code-reviewer agent read the diff (read-only). No critical findings.
- **Important, fixed as text:** the checker reads the path as text, so a link inside the working copy that points out of it is not caught here; resolving links is the runner's job (row 30, R19), which phase 3's Task 3 was to test and is still deferred. The comment on `Policy::path` now says so instead of claiming more.
- **Important, kept as the plan set it:** the refusal reuses the rule id `cap.outside-path`, whose text speaks of commands. The id is not shown anywhere today (the gate passes on the reason only), and the plan named this id. If the app ever shows a refusal's rule, it should get its own words.
- **Minor, fixed:** the containment test now checks that the temporary file is in the run's own temporary folder, not only that the output holds a `/`. Re-run:

```
> cargo test -p steplet-run --offline --test contain a_write_into_steplets_own_folder -- --ignored --nocapture
            output: "/var/lib/steplet/runs/p4t1/tmp/claude-999/tmp4y934sv1",
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 8.87s
```

- **Minor, left:** a relative path with a secret name (such as `.ssh/x`) is now refused as `cap.outside-path`, not `block.secret-path`; it is refused either way.

## Task 2: what the agent is told, and what preparing removed

### Changes from the plan, recorded here

- **Both entries of `prepare.sh` are named.** The plan added a second entry, `strip <copy>`. But `strip` is also a valid run id, so `prepare.sh strip` could have meant either. Now the first argument always names the entry: `prepare <run id> …` or `strip <copy>`.
- **`removed_files` is public**, as `changed_files` already is: it reads the script's `removed: <name>` lines, and the strip test checks it on the script's real output. It counts only the five names the script knows, each once, in the script's order, so a line printed by anything else the script ran cannot add a name. Its own small test (`removed_files_counts_only_names_it_knows`) was written together with it, not before it.
- **`prepare` also checks the script said the copy is ready** at the declared commit (`steplet-prepare: ready at <commit>`), and fails with "check out the commit inside the fence" otherwise.
- **The existing test on a real repository that carries a `.claude` folder** (`agent_settings_files_the_commit_carries_are_checked_out_then_removed`, anthropics/claude-code-action) now also checks that `prepare` says it removed `.claude`.
- The plan did not fix the words for an empty "never" list or an empty body; they are "Never: nothing listed." and "The sender's notes: none.".

### The tests first

```
> cargo test -p steplet-run --offline --test task
error[E0432]: unresolved import `steplet_run::task_text`
 --> crates\steplet-run\tests\task.rs:6:5
error: could not compile `steplet-run` (test "task") due to 1 previous error
> cargo test -p steplet-run --offline --test prepare --no-run
error[E0432]: unresolved import `steplet_run::Prepared`
  --> crates\steplet-run\tests\prepare.rs:11:29
error[E0425]: cannot find function `removed_files` in crate `steplet_run`
   --> crates\steplet-run\tests\prepare.rs:298:29
error[E0609]: no field `removed` on type `()`
   --> crates\steplet-run\tests\prepare.rs:159:25
error: could not compile `steplet-run` (test "prepare") due to 3 previous errors
```

(The lines starting with `error` or `-->`, as printed.)

### The code

- `crates/steplet-run/src/task.rs`: `task_text(s, commit)`, the fixed form of the plan (the steps numbered, the allowed commands and sites word for word, the never list, then the sender's body exactly as signed).
- `crates/steplet-run/src/prepare.sh`: the `strip` entry removes `.claude .codex .cursor .gemini .mcp.json` in that order, as the `steplet` account, and prints `removed: <name>` for each one it removed; the `prepare` entry runs it as its step 5.
- `crates/steplet-run/src/prepare.rs`: `prepare` returns `Prepared { commit, removed }`; `run` passes it on (Task 3 shows it on screen).

### The tests pass

```
> cargo test -p steplet-run --offline --test task
test task_text_keeps_the_body_exactly ... ok
test task_text_is_steplets_fixed_form ... ok
test task_text_says_when_nothing_is_allowed ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```
> cargo test -p steplet-run --offline --test prepare -- --ignored --nocapture --test-threads=1
(the lines that name each test, the strip step's output, and the summary; unchanged)
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... --- https://github.com/octocat/steplet-test-no-such-repository.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d: could not clone the repository:
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ok
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... /var/lib/steplet/runs/t6a:
test prepare_reports_the_commit_and_no_removed_files_for_hello_world ... ok
test steplet_s_own_listing_of_the_copy_runs_inside_the_fence ... --- listed: Ok([" M README", "?? .gitattributes", "?? .steplet-test-probe.sh"])
test the_strip_step_lists_what_it_removed ... --- the strip step said:
removed: .claude
removed: .codex
removed: .mcp.json

--- left in the copy:
/opt/steplet-test/strip-copy/src
/opt/steplet-test/strip-copy/src/keep.txt
ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 18.86s
```

The strip step removed exactly the three planted agent files and left `src/keep.txt`. Hello-World's copy reports its commit and an empty removed list.

```
> cargo test -p steplet-run --offline
(14 summary lines, every one `ok`, `0 failed`; 80 passed, 26 ignored: the Linux-only ones; counted from the summary lines)
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.59s
clippy exit 0
```

### What this task did not verify

- That the other Linux-only tests (launch, gate, tripwires, contain) still pass with the renamed script entry: Task 3 Step 4 runs all of them.
- Whether a real agent follows the task text better than phase 3's: only the real-model run of Task 15 sees a real agent.
