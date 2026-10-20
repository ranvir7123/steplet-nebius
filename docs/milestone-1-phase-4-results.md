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

### The code review, and what changed

A code-reviewer agent read the diff (read-only). No critical or important findings; it checked that nothing the repository controls can make the root script remove something outside the copy (links are removed as links, never followed) or add a name to the removed list. Four small points, fixed:
- In `prepare.sh`, the strip step's command had lost its line break (it worked, but read as one long line); it is two lines again, with a comment on why it needs only a PATH.
- The strip entry now refuses an empty or missing folder, or `/`, on its own, instead of relying on its caller.
- A missing "ready" line is now said as "could not confirm the working copy is ready", not as a failed checkout.
- `task_text`'s comment says the sender's body may look like Steplet's headings by design, and that nothing of Steplet's own comes after it.

```
> cargo test -p steplet-run --offline --test prepare -- --ignored --test-threads=1
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... ok
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ok
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... ok
test prepare_reports_the_commit_and_no_removed_files_for_hello_world ... ok
test steplet_s_own_listing_of_the_copy_runs_inside_the_fence ... ok
test the_strip_step_lists_what_it_removed ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 18.75s
```

### The automatic security check, and what changed (after commit 75f653e)

An automatic security check read Task 2's commits and raised four points. Three are fixed; one is kept, with the reason.

1. **Fixed: a program of the task's account could put an agent settings file back.** When Steplet prepares a run, the strip step ran only after every program of the `steplet` account had been stopped. Run on its own, it stopped nothing, so a program of that account could make `.claude` again after its removal was said. The strip step now stops every program of the `steplet` account first, and checks all five names again at the end.
2. **Fixed: the strip step took any folder.** It now takes a run id, as Steplet's other scripts do, and works only on `/work/<run id>/repo`. Anything else is refused, and nothing is removed.
3. **Fixed: git's and the fence's own output could reach what Steplet reads.** The clone and the checkout now write only to the error output, so the only lines on standard output are the script's own `removed:` and `ready at` lines. No test shows this one, because git's quiet clone printed nothing there before either; it closes a way in, it does not change what happens today.
4. **Kept: the sender's notes go to the agent exactly as written,** even though they could look like Steplet's own headings. They are the sender's own task, which the agent is meant to follow; Steplet's safety never rests on what the agent is told, but on the checker deciding every action and on the fence. Nothing of Steplet's own comes after the notes.

The tests first. The listing test changed to the run-id form; two new tests show points 1 and 2:

```
> cargo test -p steplet-run --offline --test prepare strip -- --ignored --nocapture --test-threads=1
(the lines naming each test, the failures and the summary; unchanged)
test the_strip_step_lists_what_it_removed ... --- the strip step said:
thread 'the_strip_step_lists_what_it_removed' (3308) panicked at crates\steplet-run\tests\prepare.rs:308:5:
  left: Some(14)
 right: Some(0)
test the_strip_step_refuses_anything_but_a_run_s_working_copy ... --- the strip step said: [
--- left: 
thread 'the_strip_step_refuses_anything_but_a_run_s_working_copy' (45008) panicked at crates\steplet-run\tests\prepare.rs:334:9:
  left: (Some(0), "removed: .claude\nremoved: .codex\nremoved: .mcp.json\n")
 right: (Some(14), "")
test the_strip_step_stops_the_steplet_account_first ... --- the strip step said:
--- .claude afterwards: present; steplet programs: running
thread 'the_strip_step_stops_the_steplet_account_first' (40900) panicked at crates\steplet-run\tests\prepare.rs:366:5:
  left: Some(14)
 right: Some(0)
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 8 filtered out; finished in 4.04s
```

The first and third also fail because the step did not yet take a run id. The third's printed line shows the hole itself: `.claude` was back, and the looping program was still running. After the fix:

```
> cargo test -p steplet-run --offline --test prepare -- --ignored --nocapture --test-threads=1
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... --- https://github.com/octocat/steplet-test-no-such-repository.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d: could not clone the repository:
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ok
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... /var/lib/steplet/runs/t6a:
test prepare_reports_the_commit_and_no_removed_files_for_hello_world ... ok
test steplet_s_own_listing_of_the_copy_runs_inside_the_fence ... --- listed: Ok([" M README", "?? .gitattributes", "?? .steplet-test-probe.sh"])
test the_strip_step_lists_what_it_removed ... --- the strip step said:
test the_strip_step_refuses_anything_but_a_run_s_working_copy ... --- the strip step said: [
test the_strip_step_stops_the_steplet_account_first ... --- the strip step said:
--- .claude afterwards: absent; steplet programs: none
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 24.26s
```

What the refusal test's five bad arguments got, each one (unchanged):

```
        "steplet-prepare: not a run id: /opt/steplet-test/strip-outside",
        "steplet-prepare: not a run id: ../../opt/steplet-test/strip-outside",
        "steplet-prepare: not a run id:",
        "steplet-prepare: not a run id: -x",
        "steplet-prepare: not a run id: p4t2x/../..",
```

Everything planted in the outside folder was still there afterwards (`.claude`, `.codex`, `.mcp.json`). And one whole run through the changed script:

```
> cargo test -p steplet-run --offline --test contain use_case_1 -- --ignored
test use_case_1_runs_with_none_of_the_receiver_s_files_present ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 10.70s
> cargo clippy -p steplet-run --lib --test prepare --test contain --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
> cargo fmt --all -- --check
fmt exit 0
```

(The lint check covers only these parts: Task 3's tests, already written but not yet backed by code, are in the working folder and do not build yet. Task 3 runs the full check.)

## Task 3: live runs: progress as it happens, cleaned; Stop; one run at a time; raw logs

### Changes from the plan, recorded here

- **`watch_run` also takes the secrets to hide** (the key and the run's token), besides `live`: the live log is cleaned there, and the token is only made inside the run. Phase 3's tests pass `&Live::unattended(), &[]`.
- **`launch` takes the secrets too** (`hide`), for the raw logs (J6 A). The function that hides them, `raw_log_line`, sits in `report.rs` next to the report's own "[hidden by Steplet]".
- **`Live` has only `on_progress` and `stop` for now.** The answers and the ask limit come with Task 4, the task that uses them; `Progress` likewise has only `Prepared` and `Event` until then.
- **`RunError` gains `Lock`** besides `Busy`: if Windows would not make the lock at all, the message says so instead of claiming that another run holds it.
- **A Stop pressed while the working copy is being prepared:** the agent is then never started, and the report says the receiver stopped it. The plan did not say what happens in that case.
- **A tripwire after the receiver's Stop** is still recorded, but the run is not stopped a second time.
- The new tests count four live events, not three: after a call that skipped the gate, Steplet's own record of it is sent too, as the plan's Step 3 says.

### The tests first

```
> cargo test -p steplet-run --offline --test watch_run --test lock --test report
error[E0432]: unresolved import `steplet_run::RunLock`
 --> crates\steplet-run\tests\lock.rs:6:29
error[E0432]: unresolved import `steplet_run::raw_log_line`
  --> crates\steplet-run\tests\report.rs:24:85
error[E0432]: unresolved imports `steplet_run::Live`, `steplet_run::Progress`, `steplet_run::StopSignal`
  --> crates\steplet-run\tests\watch_run.rs:23:47
error[E0560]: struct `RunOutcome` has no field named `stopped_by_you`
  --> crates\steplet-run\tests\report.rs:46:9
error[E0599]: no variant, associated function, or constant named `Busy` found for enum `RunError` in the current scope
  --> crates\steplet-run\tests\lock.rs:11:53
error[E0599]: no variant, associated function, or constant named `Busy` found for enum `RunError` in the current scope
  --> crates\steplet-run\tests\lock.rs:12:78
error[E0061]: this function takes 5 arguments but 7 arguments were supplied
   --> crates\steplet-run\tests\watch_run.rs:175:25
   --> crates\steplet-run\src\run.rs:160:8
error[E0061]: this function takes 5 arguments but 7 arguments were supplied
   --> crates\steplet-run\tests\watch_run.rs:265:25
   --> crates\steplet-run\src\run.rs:160:8
error: could not compile `steplet-run` (test "lock") due to 3 previous errors
error[E0609]: no field `stopped_by_you` on type `steplet_run::Report`
```

(The lines starting with `error` or `-->`, as printed; this run was before the usage-limit stop.)

### The code

- `crates/steplet-run/src/live.rs`: `StopSignal`, `Progress`, `Live` and `Live::unattended()`.
- `crates/steplet-run/src/lock.rs`: `RunLock::take()`, the named Windows object `Local\Steplet-run`. Only that the object exists counts, so the lock can be let go on any thread, and Windows lets it go when Steplet ends or crashes.
- `crates/steplet-run/src/report.rs`: `Scrubber::event` (every text of an event cleaned as the report cleans it, JSON keys included), `raw_log_line`, `Report.stopped_by_you`.
- `crates/steplet-run/src/run.rs`: `run_live` takes the lock first, sends `Progress::Prepared` after preparing, then each event as it is judged; `run` is `run_live` with no one watching. `watch_run` looks at the Stop button on every turn and stops the run as at a tripwire, recording `stopped_by_you`.
- `crates/steplet-run/src/launch.rs`: each raw log line is written with the key and the token hidden.

### The tests pass

```
> cargo test -p steplet-run --offline --test watch_run --test lock --test report
test a_second_run_cannot_start_while_one_is_running ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test raw_log_lines_hide_the_key_and_the_token ... ok
test result: ok. 20 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
test stop_signal_stops_the_run_and_the_report_says_you_stopped_it ... ok
test each_event_reaches_the_screen_cleaned_and_in_order ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
```

(The lines of the four new tests and the summaries.)

```
> cargo test -p steplet-run --offline
plain exit 0
(15 summary lines, every one ok; 84 passed, 0 failed, 28 ignored: counted from the summary lines)
> cargo test -p steplet-run --offline -- --ignored --nocapture --test-threads=1
(each test program's line and summary, unchanged)
     Running unittests src\lib.rs (target\debug\deps\steplet_run-edb81ff285450e2f.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
     Running tests\contain.rs (target\debug\deps\contain-e9ca5c3e4707e558.exe)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.19s
     Running tests\decide.rs (target\debug\deps\decide-f984e75a8222c3ff.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
     Running tests\gate.rs (target\debug\deps\gate-ae3f0e2f1d920802.exe)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 25.18s
     Running tests\launch.rs (target\debug\deps\launch-2c821a776c9bbf7e.exe)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 57.72s
     Running tests\lock.rs (target\debug\deps\lock-208f96d8593d1013.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
     Running tests\prepare.rs (target\debug\deps\prepare-ca0ff602a7cf43dc.exe)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 21.75s
     Running tests\report.rs (target\debug\deps\report-ff7dc5d64e57faad.exe)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 20.40s
     Running tests\settings.rs (target\debug\deps\settings-8f98eebba7e02420.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
     Running tests\stream.rs (target\debug\deps\stream-14a26201034b512e.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
     Running tests\task.rs (target\debug\deps\task-4ed8a12bb5f61ffb.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
     Running tests\tripwires.rs (target\debug\deps\tripwires-8535515b103c05f7.exe)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.69s
     Running tests\watch.rs (target\debug\deps\watch-3da87fd5f880fdb0.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s
     Running tests\watch_run.rs (target\debug\deps\watch_run-76ecdcace2e75d9b.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
ignored exit 0
```

Every Linux-only test passes: 28 of 28 (phase 3's 22, Task 1's two and Task 2's four), with the new live-run code and the renamed script entry.

```
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Checking steplet-run v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase4-build-88af2b\crates\steplet-run)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.08s
clippy exit 0
```

### What this task did not verify

- A live run whose raw log really held the key or the token: no run prints them (H2), so hiding them is shown by the unit test only.
- Two Steplet windows in two different Windows sign-in sessions: the lock is per session (`Local\`), as the plan chose; Steplet's Linux is shared by them.
- A program of the same Windows account that makes `Local\Steplet-run` itself would keep every run from starting. It could not harm a run, only block one.

### The code review, and what changed

A code-reviewer agent read the diff (read-only). No critical findings; three important ones, all fixed, each with a test that failed first:

1. **A tripwire that came after your Stop took the stop from you.** When the receiver pressed Stop and an event already on its way then set off a tripwire, the report said Steplet's tripwire stopped the run. Now a tripwire after the receiver's Stop does not count as the reason; the call that set it off still shows as what it was (here a write that skipped the checker, which is not marked as cut short by the stop). Test: `a_tripwire_after_your_stop_does_not_take_the_stop_from_you`. To show this, the stand-in agent of `tests/watch_run.rs` now hands over the events it had not yet given out when it is finished, as the real one does.
2. **Stop did nothing while the working copy was being prepared,** which can take minutes for a large repository. Now a second thread looks at the Stop button while preparing runs and, once it is pressed, ends every program of the `steplet` account (the clone and the checkout run as that account). Preparing then fails, which counts as the receiver's Stop, not as a fault; the agent never starts, and Steplet does not try to list changes in a copy the agent never touched. Test: `a_stop_while_preparing_ends_the_preparing` (anthropics/claude-code-action, Stop pressed one second in).
3. **The live feed's documentation** now says that `on_progress` must only hand the progress on and return: it runs on the thread that watches the agent, so a slow screen would delay the Stop button and the time limit.

Also added: `run_live_is_refused_while_another_run_holds_the_lock` (no Linux). It passed at once, as it should: it guards what Task 3 already did.

The tests first:

```
> cargo test -p steplet-run --offline --test watch_run --test lock
test run_live_is_refused_while_another_run_holds_the_lock ... ok
test a_tripwire_after_your_stop_does_not_take_the_stop_from_you ... FAILED
thread 'a_tripwire_after_your_stop_does_not_take_the_stop_from_you' (49976) panicked at crates\steplet-run\tests\watch_run.rs:358:5:
  left: Some(SkippedGate("Write /work/r1/repo/x.txt"))
 right: None
test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
> cargo test -p steplet-run --offline --test contain a_stop_while_preparing -- --ignored --nocapture
--- 6.2 s; sent to the screen: [Prepared(Prepared { commit: "34968b5981d5d170904281aa3df6f3c0d4a28855", removed: [".claude"] })]
--- left: "/var/lib/steplet/runs:\n\n/work:"
thread 'a_stop_while_preparing_ends_the_preparing' (48592) panicked at crates\steplet-run\tests\contain.rs:561:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 6.46s
```

After the fixes:

```
> cargo test -p steplet-run --offline --test watch_run --test lock
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
> cargo test -p steplet-run --offline --test contain a_stop_while_preparing -- --ignored --nocapture
--- 1.8 s; sent to the screen: []
--- left: "/var/lib/steplet/runs:\n\n/work:"
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 2.04s
```

Minor points, left as they are and named here:
- The raw logs hide the key and the token only where they appear as the same bytes. A task that wrote the key in a disguised form (with an escape or a zero byte inside it) would get past that. The screen and the report are not affected: they clean the text before they look for secrets.
- If another Windows account in the same sign-in session held `Local\Steplet-run`, the message would be "Windows would not make the lock", not "another run". Two sign-in sessions of the same account could each run one file at the same time; both use the one Steplet Linux.
- A Stop pressed in the same moment the agent ends on its own is reported as the receiver's stop.

The whole runner after the fixes. The first plain run had one failure, in a test, not in Steplet: the two lock tests ran at the same time in one test program, so each found the other holding the computer-wide lock (`no run holds the lock: Busy`). They now take turns, as every test file that runs Steplet does; three runs in a row passed.

```
> cargo test -p steplet-run --offline
(16 summary lines, every one ok: 86 passed, 0 failed, 30 ignored; counted from the summary lines)
> cargo test -p steplet-run --offline -- --ignored --test-threads=1
(16 summary lines, every one ok: 30 passed, 0 failed)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.10s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.46s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 28.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 56.32s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 21.77s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 21.94s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.27s
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit 0
> cargo fmt --all -- --check
fmt exit 0
```

(The Linux-only run's non-empty summary lines, in order: ask_live (Task 4's measurement, already in the working folder), contain, gate, launch, prepare, report, tripwires.)

## Task 4: the ask path (R7)

### Changes from the plan, recorded here

- **`Asker::new` also takes the secrets** (the key, the run's token): the question shows the request cleaned, and the token is made inside the run.
- **`Answer` also holds the tool call's id** (`call`): the report needs it to mark a call "allowed by you", since a request carries no fingerprint of its own.
- **Steplet's own stops take back open questions too.** When Steplet stops a run itself (a tripwire, the time limit, an end it cannot see), it now also sets the Stop signal, so a question still on screen is closed at once instead of waiting out its 5 minutes. The report still says what stopped the run: a stop Steplet makes after it has begun stopping does not count as the receiver's. Test: `a_tripwire_takes_back_every_open_question`, written together with the change; it was then shown to fail with the one line it guards removed (below).
- **`decide.rs` is split in two,** so the unattended answer and the asker read requests the same way: `check` (the request read as the checker's action) and `no_one_to_ask`. The one checker is still `Policy::check` (row 29).
- **The fingerprint sorts the JSON keys itself,** instead of relying on the JSON library's order (the desktop app's libraries could switch that library to keep the order keys came in).
- **Two pieces of my own test data were wrong,** and the tests said so: a command holding a control character is refused by the checker outright (so the escape went into the request's description, which the question shows but the checker does not read), and `echo` is a harmless read the checker allows without asking (so the test uses `touch`).
- The private `Answer` inside `report.rs` (one call's result) is renamed `CallResult`, so it does not clash with the new public `Answer`.
- A yes given with another fingerprint, or a "reused" reply from the screen (which only Steplet can give), is recorded as a no.
- The words the agent gets back: "the receiver said no"; "no answer in time, so Steplet refused it"; "the answer did not match the request on screen, so Steplet refused it"; "Steplet is stopping this run, so it took the question back and refused it".

### Step 1: does Claude Code ask again after a yes? (a record)

The gate allowed everything, as a receiver who says yes to every question would (Claude Code cannot tell the two apart). Each kind of request had a run of its own:

```
> cargo test -p steplet-run --offline --test ask_live does_claude_code -- --ignored --nocapture
--- Bash: calls 2; first reached the gate: true; second reached the gate: true; stopped by: None
--- Write: calls 2; first reached the gate: true; second reached the gate: true; stopped by: None
--- Read: calls 2; first reached the gate: true; second reached the gate: true; stopped by: None
--- WebFetch: calls 2; first reached the gate: true; second reached the gate: true; stopped by: None
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.46s
```

So Claude Code 2.1.220 asks Steplet again every time, for all four kinds, and the watcher needs no change; the plan's extra test for a skipped repeat is not needed.

### Step 2 and 3: the tests first

```
> cargo test -p steplet-run --offline --test ask
error[E0432]: unresolved imports `steplet_run::Answers`, `steplet_run::Asker`, `steplet_run::Question`, `steplet_run::Reply`, `steplet_run::fingerprint`
  --> crates\steplet-run\tests\ask.rs:19:5
error[E0599]: no variant, associated function, or constant named `Asking` found for enum `Progress` in the current scope
  --> crates\steplet-run\tests\ask.rs:58:30
error[E0560]: struct `Live` has no field named `answers`
  --> crates\steplet-run\tests\ask.rs:71:9
error[E0560]: struct `Live` has no field named `ask_limit`
  --> crates\steplet-run\tests\ask.rs:72:9
error[E0599]: no variant, associated function, or constant named `Asking` found for enum `Progress` in the current scope
  --> crates\steplet-run\tests\ask.rs:85:42
error[E0599]: no variant named `Answered` found for enum `Progress`
  --> crates\steplet-run\tests\ask.rs:94:30
```

(The first six `error` lines with their places, as printed.)

### Step 4: the code

`crates/steplet-run/src/ask.rs` (plan section 3.3): `Question`, `Reply`, `Answer`, `Answers` (where the screen puts the receiver's answers), `Asker` (one run's question path: its approvals, kept in memory only, its question numbers and its record), and `fingerprint` (SHA-256 of the tool's name, a zero byte and the request's JSON with sorted keys; the call's id is not part of it). `run_live` refuses an ask limit the gate's deadline cannot cover (`RunError::Limits`: the deadline must be at least the ask limit plus 15 s), and uses the asker as the gate's decision function when someone can answer. The report gains `answers` and `Check::AllowedByYou { reused }`.

### Step 5: the tests pass

```
> cargo test -p steplet-run --offline --test ask
test the_checkers_allow_and_deny_never_ask ... ok
test run_live_refuses_an_ask_limit_the_gate_deadline_cannot_cover ... ok
test two_questions_at_once_each_get_their_own_answer ... ok
test a_yes_with_another_fingerprint_is_a_no ... ok
test the_question_shows_the_request_cleaned ... ok
test a_yes_allows_exactly_the_request_shown ... ok
test the_same_exact_request_later_in_the_run_is_allowed_without_asking_again ... ok
test a_request_that_differs_by_one_character_asks_again ... ok
test a_stop_withdraws_the_open_question_at_once ... ok
test no_answer_within_the_limit_is_a_no ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
> cargo test -p steplet-run --offline --test report answers_and
test answers_and_allowed_by_you_show_in_the_report ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s
> cargo test -p steplet-run --offline --test watch_run a_tripwire_takes
test a_tripwire_takes_back_every_open_question ... ok
```

With the one line it guards (`stop.stop();` in `stop_now`) taken out for a moment, and then put back:

```
test a_tripwire_takes_back_every_open_question ... FAILED
thread 'a_tripwire_takes_back_every_open_question' (47372) panicked at crates\steplet-run\tests\watch_run.rs:379:5:
open questions are taken back
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
```

The real runs, with the screen played by the test:

```
> cargo test -p steplet-run --offline --test ask_live -- --ignored --nocapture --test-threads=1
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 40.03s
```

What the screen was sent in `a_yes_on_the_ask_screen_lets_exactly_that_command_run`, and the report (unchanged, the client details of the gate's records left out):

```
    Asking(
        Question {
            id: 1,
            tool: "Bash",
            what: "touch asked.txt",
            input: Object {
                "command": String("touch asked.txt"),
                "description": String("Make a file"),
            },
            reason: "the file does not ask for this command",
            fingerprint: "f2d02cf1aac4cb12ee6b4c03da6331c1a5a67732d195f25ace19aa0353044b8d",
            asked_at: Instant {
                t: 4611815197.958626403s,
            },
        },
    ),
    Answered {
        question: 1,
        reply: Yes {
            fingerprint: "f2d02cf1aac4cb12ee6b4c03da6331c1a5a67732d195f25ace19aa0353044b8d",
        },
    },
    Event(
--- the report:
Report {
    completed: true,
    stopped_by: None,
    stopped_by_you: false,
    commands: [
        ReportCommand {
            tool: "Bash",
            command: "touch asked.txt",
            check: AllowedByYou {
                reused: false,
            },
            exit_code: Some(
                0,
            ),
            output: "(Bash completed with no output)",
            after_stop: false,
        },
    ],
    refused: [],
    models: [
        "claude-opus-5[1m]",
    ],
    refusals: [],
    changed_files: Some(
        [
            "?? asked.txt",
        ],
    ),
    trust: NotRecorded,
    agent_says: "Task 4: the end of the script.",
    faults: [],
    answers: [
        Answer {
            call: "toolu_test_ac89998be82e42338bdc",
            tool: "Bash",
            what: "touch asked.txt",
            reply: Yes {
                fingerprint: "f2d02cf1aac4cb12ee6b4c03da6331c1a5a67732d195f25ace19aa0353044b8d",
            },
        },
    ],
    notes: [
        "Blocked fence attempts are not visible to Steplet: the fence stops a program without telling Steplet, so this report never says the fence stopped a particular thing.",
    ],
}
```

The question carried the request whole, its reason and its fingerprint; the screen's yes carried the same fingerprint; the command ran inside the fence with exit code 0; and Steplet's own `git status` after the run saw the file it made. In `no_answer_refuses_it` (a 2-second limit, the screen never answers) the call is refused with "no answer in time", nothing is made, and the report lists `NoAnswer`.

The whole runner, and the checks:

```
> cargo test -p steplet-run --offline
(17 summary lines, every one ok: 98 passed, 0 failed, 32 ignored; counted from the summary lines)
plain exit 0
> cargo test -p steplet-run --offline -- --ignored --test-threads=1
(17 summary lines, every one ok: 32 passed, 0 failed; counted from the summary lines)
ignored exit 0
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.48s
clippy exit 0
```

### What this task did not verify

- A real person answering: the screen is played by the tests here; the ask screen itself is Task 12.
- Whether a yes reused for "the same exact request" ever differs in effect the second time (J2 B accepts this; the button will say so).
- Later Claude Code versions asking again after a yes: measured on 2.1.220 only.

### The code review, and what changed

A code-reviewer agent read the diff (read-only). It found no way for a yes to allow anything other than the request it answered. Three important findings, and one bug its test found besides:

1. **Fixed: the question showed the request cleaned, so a yes could cover something other than what the receiver saw.** For example, a file write whose content holds a no-break space would have been shown with a plain space, or a hidden character dropped. Now a request that Steplet cannot show exactly (cleaning would change it: a character that does not show, a terminal escape, or the receiver's key) is refused without a question; the agent is told why. A command holding such characters was already refused by the checker. Test: `a_request_steplet_cannot_show_exactly_is_refused_without_asking` (replaces the plan's `the_question_shows_the_request_cleaned`, which expected such a request to be asked about with the cleaned text).
2. **Fixed: a question still open when the run ended stayed open** for up to its 5 minutes (for example when the run reached its time limit while the screen showed a question). Now every open question is taken back when a run is over, and recorded as taken back before the report is made. Test: `a_question_open_when_the_run_ends_is_taken_back` (a 15-second run, a 120-second ask limit, a screen that never answers).
3. **Found by that test, and fixed: a run cut short while a question was open was blamed on the agent.** At the time limit, Claude Code gives back a "rejected" result for the call that waited for the answer. Steplet's watcher had no record of a gate answer for it yet (a decision being made was not on record), so it judged the call as one that skipped the checker, stopped the run for it, and the report said "a command ran without asking". The command had never run. Now the gate service also knows which calls it is still deciding, and counts each as not allowed: an error result for it is fine; a result showing it ran would still be a breach. Test: `a_request_still_being_decided_counts_as_not_allowed` (no Linux).
4. **Kept as it is, as a known limit: a late answer after the gate gave up.** The gate gives Steplet its deadline from the moment the gate program connects; the asker's 5 minutes start when the request has been read. Only if reading the request took longer than the 15-second margin could the gate refuse while the question is still on screen. The request comes from Steplet's own gate program, which sends it at once.

The tests first:

```
> cargo test -p steplet-run --offline --test ask a_request_steplet_cannot
test a_request_steplet_cannot_show_exactly_is_refused_without_asking ... FAILED
thread 'a_request_steplet_cannot_show_exactly_is_refused_without_asking' (47156) panicked at crates\steplet-run\tests\ask.rs:207:9:
Object {"command": String("touch made-up.txt"), "description": String("made up \u{1b}[31mtest")}: Allow { updated_input: Object {"command": String("touch made-up.txt"), "description": String("made up \u{1b}[31mtest")} }
> cargo test -p steplet-run --offline --test ask_live a_question_open_when -- --ignored --nocapture
(the first try passed, for the wrong reason: the report showed why)
Report {
    completed: false,
    stopped_by: Some(
        SkippedCommand(
            "touch asked.txt",
        ),
    ),
...
            command: "touch asked.txt",
            check: Refused,
...
            output: "The user doesn't want to proceed with this tool use. The tool use was rejected (eg. if it was a file edit, the new_string was NOT written to the file). STOP what you are doing and wait for the user to tell you how to proceed.",
> cargo test -p steplet-run --offline --test gate a_request_still_being
test a_request_still_being_decided_counts_as_not_allowed ... FAILED
thread 'a_request_still_being_decided_counts_as_not_allowed' (4852) panicked at crates\steplet-run\tests\gate.rs:534:5:
  left: None
 right: Some(false)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 17 filtered out; finished in 5.02s
```

After the gate fix, the run-end test failed for the right reason (no false stop any more, but the question was not taken back):

```
--- 18.7 s; closed: []
thread 'a_question_open_when_the_run_ends_is_taken_back' (10496) panicked at crates\steplet-run\tests\ask_live.rs:216:5:
  left: []
 right: [Withdrawn]
Report {
    completed: false,
    stopped_by: None,
```

After all the fixes:

```
> cargo test -p steplet-run --offline --test ask
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
> cargo test -p steplet-run --offline --test gate
test result: ok. 15 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 9.14s
> cargo test -p steplet-run --offline --test ask_live -- --ignored --nocapture --test-threads=1
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 58.43s
```

(One test's own wording was then corrected and re-run: its third case, a command with a no-break space, is already refused by the checker; it is now a file write whose content holds one, which the checker does not read.)

Minor points, left as they are: a run-time ask limit too long to add to a clock now counts as reached at once (and `run_live` refuses it before starting); the report matches the receiver's answers to calls by the call's id alone, which only a model that reused an id could confuse.

The whole runner after the fixes (Task 5's unfinished files were in the working folder; its declared-site test, which waits on a decision, was left out of this run):

```
> cargo test -p steplet-run --offline
(18 summary lines, every one ok: 102 passed, 0 failed, 34 ignored; counted from the summary lines)
> cargo test -p steplet-run --offline -- --ignored --test-threads=1 --skip a_declared_site_is_the_only_one
(18 summary lines, every one ok: 33 passed, 0 failed; counted from the summary lines)
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit 0
```

## Task 5: the chooser's lines, each tied to its test; and a phase 3 gap it found

### What the new test found: a file's declared web sites never worked for the programs a task runs

The new test `a_declared_site_is_the_only_one_a_program_can_reach` runs a small probe inside the fence (made-up test data: it tries `https://example.org/`, which the file declares, and `https://example.net/`, which it does not), with a control from outside the fence. Its first run:

```
--- control, outside the fence: {"https://example.org/": "failed: HTTPError", "https://example.net/": "failed: HTTPError"}
--- inside the fence, the probe's own record: {"https://example.org/": "failed: URLError", "https://example.net/": "failed: URLError"}
```

The control's "HTTPError" means the sites did answer (with an error status, to Python's default client), so the probe now counts any answer from the site as reached. Inside the fence, both were cut off, the declared one too. Three different clients from inside the fence (the run's proxy password replaced by `(token)` here):

```
--- inside the fence, the probe's own record {
 "proxy": "http://srt:(token)@localhost:3128",
 "https://example.org/": "failed: URLError: <urlopen error Remote end closed connection without response>",
 "curl https://example.org/": "000 curl: (56) Proxy CONNECT aborted",
 "raw11 https://example.org/": "",
 "raw10 https://example.org/": "(closed)",
 "https://example.net/": "failed: URLError: <urlopen error Remote end closed connection without response>",
 "curl https://example.net/": "000 curl: (56) Proxy CONNECT aborted",
 "raw11 https://example.net/": "",
 "raw10 https://example.net/": "(closed)"
}
```

The fence program (`srt` 0.0.77) on its own, outside Claude Code, with example.org allowed, works as promised (the lines about the decision and curl's result, from its `--debug` output):

```
--- https://example.org/
[SandboxDebug] Allowed by config rule: example.org:443
200
--- https://example.net/
[SandboxDebug] No matching config rule, denying: example.net:443
[SandboxDebug] Connection blocked to example.net:443
curl: (7) CONNECT tunnel failed, response 403
000
```

**The cause:** in a Claude Code run, the fence's network bridge is a socket file in the run's temporary folder, which phase 3 put inside Steplet's private folder (`/var/lib/steplet/runs/<id>/tmp`). Steplet's settings hide that whole folder from the programs a task runs, so the bridge inside the fence could not reach the socket, and every connection was cut. Phase 3's tests checked only that undeclared sites are refused, which a fence that refuses everything also passes. Nothing unsafe happened: programs could reach nothing at all. But the chooser's line for a file that declares sites would have been false, and a task that needs, for example, a package registry could not run.

Two fixes were measured (and undone after each):
- Letting the fence read the run's temporary folder in its old place: the sites then behave as promised, but the folder becomes read-only for the task's commands, and Claude Code's own small file for each command fails (`/var/lib/steplet/runs/p4t5a/tmp/claude-999/cwd-e7c3: Read-only file system`, exit code 1). Not usable.
- Giving each run its own temporary folder under `/tmp`: example.org reached, example.net refused with 403, and the test passes.

### Your decision (18 Oct 2026): A

**A: each run gets its own fresh temporary folder under `/tmp` (`/tmp/steplet-run-<id>`), deleted after the run.** It changes phase 3's folder layout (the plan had said the only settings change in this phase is Task 1's rule); Task 16 records it in `docs/decisions.md`. Your condition: a test that, during a run, the Anthropic key and the run's token appear 0 times in that folder (counted, never printed, with phase 3's method), that the folder is gone after the run, and the containment battery run again.

What changed:
- `RunPaths::run_tmp()` is `/tmp/steplet-run-<id>`.
- `prepare.sh` removes every `/tmp/steplet-run-*` left over (with nothing of the `steplet` account running, so nothing can make one again in between), then makes the run's folder fresh, owned by `steplet`, for it alone (mode 700).
- `start-run.sh` checks that it is that folder (not a link, the `steplet` account's, mode 700) before it starts Claude Code with it as `TMPDIR`.
- `discard.sh` removes it with the rest of the run, and fails if it is still there.
- The test harness's own gate token is now at least 20 characters, so phase 3's counter can count it.
- The hostile fixture test's list of what Claude Code itself writes for a run now finds the temporary folder in its new place.

Your condition, as a test (`the_run_s_temporary_folder_holds_no_key_or_token_and_is_gone_after`). Its declared command waits 8 seconds inside the fence, and the counts are taken while it runs. (Its first try used `sleep 8`, which Claude Code ran without asking the gate; Steplet's command reader does not count `sleep` as a harmless read, so the run was stopped, as H11 says. The test now uses a Python wait, which Claude Code asks about.)

```
> cargo test -p steplet-run --offline --test contain the_run_s_temporary_folder -- --ignored --nocapture
--- during the run, /tmp/steplet-run-p4t5t held (type and name):
d claude-999
d claude-999/-work-p4t5t-repo
d claude-999/-work-p4t5t-repo/ffa4b06d-e41b-4fe9-b68b-9b3bcb4a3805
d claude-999/-work-p4t5t-repo/ffa4b06d-e41b-4fe9-b68b-9b3bcb4a3805/tasks
f claude-999/-work-p4t5t-repo/ffa4b06d-e41b-4fe9-b68b-9b3bcb4a3805/tasks/bchlp516y.output
s claude-http-79e425107eb4797c.sock
s srt-mux-85398-0.sock
--- the made-up key in it:
key: 0  (files read: 1)
key, last 20 characters: 0  (files read: 1)
--- the run's token in it:
key: 0  (files read: 1)
key, last 20 characters: 0  (files read: 1)
--- after the run: absent
test the_run_s_temporary_folder_holds_no_key_or_token_and_is_gone_after ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 15.13s
```

(`d` is a folder, `f` a file, `s` a socket. The counter says "key" for whichever secret it is given; it reads every regular file, here the one file, and sockets hold no text.) This test checks a safety property, so it would also pass with the old layout. What shows the move is the layout test, which failed first (`left: "/var/lib/steplet/runs/r1/tmp"`, `right: "/tmp/steplet-run-r1"`), and the declared-site test, which passes only with it.

### Changes from the plan, recorded here

- **Line 2 is worded for the ask screen.** The plan's "The agent cannot fetch any web address" (and "can fetch only these web addresses") was written before the ask screen. Now a site the file does not declare is asked about, and the receiver can say yes. So the lines say: "The agent cannot fetch any web address, unless you say yes to that exact address." and "The agent can fetch only these web addresses, and another one only if you say yes to that exact address:". They also name `a_yes_allows_exactly_the_request_shown`.
- The sites a line lists are its `items`, shown as file text, not set into the line's text.

### The tests

```
> cargo test -p steplet-run --offline --test promises
error[E0432]: unresolved imports `steplet_run::Promise`, `steplet_run::promises`
 --> crates\steplet-run\tests\promises.rs:6:19
error: could not compile `steplet-run` (test "promises") due to 1 previous error
```

After the code:

```
> cargo test -p steplet-run --offline --test promises
test no_line_says_on_this_computer ... ok
test the_lines_follow_the_files_sites ... ok
test every_promise_names_tests_that_exist ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The containment battery again, with the new layout (the owner's condition), and the whole runner:

```
> cargo test -p steplet-run --offline -- --ignored --nocapture --test-threads=1
     Running tests\contain.rs (target\debug\deps\contain-e9ca5c3e4707e558.exe)
--- inside the fence, the probe's own record: {"proxy": "http://srt:(token)@localhost:3128", "https://example.org/": "reached", "https://example.net/": "failed: URLError: <urlopen error Tunnel connection failed: 403 Forbidden>"}
--- the listener's log: "2026-10-18T10:18:40.508262 from=('127.0.0.1', 44734) bytes=0"
--- the write outside the copy: absent
--- the Windows marker from inside the fence: absent
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 66.11s
(18 summary lines, every one ok: 35 passed, 0 failed; counted from the summary lines)
ignored exit 0
> cargo test -p steplet-run --offline
(18 summary lines, every one ok: 102 passed, 0 failed, 35 ignored; counted from the summary lines)
plain exit 0
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit 0
```

(From the battery: the declared-site probe's record, and three lines of the hostile fixture's test: only the control's connection reached the listener, nothing landed outside the copy, and no Windows program started. The proxy password is replaced by `(token)`.) The containment battery's eight tests all pass with the new layout: use case 1, the hostile fixture, the web fetch, the writable folders, J1's write, the stop while preparing, the declared site, and the temporary folder.

### What this task did not verify

- Sites with more than one name behind them (a site that sends the program on to another site): the probe reaches one page of each.
- A program listening on the Windows side (3.7 point 3): still not tried, so no line says "on this computer".
- The lines on screen: the run chooser screen is Task 11.

### The code review, and what changed

A code-reviewer agent read Task 5's diff (read-only). Its own text was not kept when the last session stopped at its usage limit, so the points below are read back from the fixes it led to (written then, built and tested now, on 20 Oct 2026). It found nothing unsafe; it found two lines worded wider than their tests, and three tests weaker than they looked:

1. **Line 1 said more than its tests show.** "If Steplet's checker fails, the action is refused" covers any failure; the tests show two: no decision in time, and a gate program that stops working. Now: "If Steplet's checker does not decide in time, or its gate stops working, the action is refused." The line also speaks of file writes and web fetches, so it now names their tests too (`a_write_into_steplets_own_folder_is_refused_before_the_gate`, `a_web_fetch_is_allowed_only_to_a_site_the_file_declares`).
2. **Line 3 left out the run's temporary folder.** Programs a task runs can also write in `/tmp/steplet-run-<id>` (your decision A). Now: "The agent and the programs this task runs can change files only inside the working copy (the programs also in the run's own temporary folder, which Steplet deletes after the run)." It names `the_run_s_temporary_folder_holds_no_key_or_token_and_is_gone_after`.
3. **The check that each named test exists also took helpers and commented-out code.** `every_promise_names_tests_that_exist` looked for the text `fn <name>(` anywhere. Now a name counts only for a line starting `fn <name>(` with `#[test]` on one of the three lines above it; the test also checks itself on a helper that is not a test.
4. **The hostile-fixture test's list of Claude Code's own writes matched by the start of a name,** so `/tmp/steplet-run-t11b` would also have taken a folder such as `/tmp/steplet-run-t11bX`. Now the folder itself or a path inside it, exactly.
5. **The temporary-folder test counted the key and the token only while the agent ran.** It now counts again once the agent has ended, before the run is removed, so anything written late is counted too. And the stop-while-preparing test now also checks that no `/tmp/steplet-run-*` folder is left.
6. **The folder's name is written in four places** (`RunPaths` and three scripts). A new test, `the_scripts_and_the_paths_name_the_same_temporary_folder`, fails when any of them names another folder.

Each changed test, seen failing first. The tests were written with their fixes, so each was run against the old code or a planted mistake, one at a time, and the file restored after (the script is in the scratchpad: `t5r-red.sh`):

```
=== 1. the old promises.rs (the lines before the review), the new tests
test no_line_says_on_this_computer ... ok
test the_lines_follow_the_files_sites ... FAILED
test every_promise_names_tests_that_exist ... ok
thread 'the_lines_follow_the_files_sites' (40472) panicked at crates\steplet-run\tests\promises.rs:49:5:
  left: ["Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen. If Steplet's checker fails, the action is refused.", "The agent cannot fetch any web address, unless you say yes to that exact address.", "The agent and the programs this task runs can change files only inside the working copy.", "Your own files are not inside Steplet's Linux, where this task runs.", "Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.", "Programs this task runs cannot start Windows programs."]
 right: ["Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen. If Steplet's checker does not decide in time, or its gate stops working, the action is refused.", "The agent cannot fetch any web address, unless you say yes to that exact address.", "The agent and the programs this task runs can change files only inside the working copy (the programs also in the run's own temporary folder, which Steplet deletes after the run).", "Your own files are not inside Steplet's Linux, where this task runs.", "Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.", "Programs this task runs cannot start Windows programs."]
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
=== 2. a line that names a helper, not a test (fn steplet in tests/promises.rs)
1
test every_promise_names_tests_that_exist ... FAILED
thread 'every_promise_names_tests_that_exist' (55128) panicked at crates\steplet-run\tests\promises.rs:39:17:
"Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen. If Steplet's checker does not decide in time, or its gate stops working, the action is refused." names steplet, which is not a test here
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s
=== 3. prepare.sh names another temporary folder
33:    local W=/work/$ID R=/var/lib/steplet/runs/$ID T=/tmp/steplet-tmp-$ID
test the_scripts_and_the_paths_name_the_same_temporary_folder ... FAILED
thread 'the_scripts_and_the_paths_name_the_same_temporary_folder' (43556) panicked at crates\steplet-run\tests\settings.rs:208:9:
prepare.sh names the run's temporary folder as /tmp/steplet-run-<id>
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
=== restored:
promises.rs as written
```

(With the old `every_promise_names_tests_that_exist`, case 2 would have passed: `fn steplet(` is in `tests/promises.rs`, as a helper.)

With the fixes:

```
> cargo test -p steplet-run --offline --test promises --test settings
     Running tests\promises.rs (target\debug\deps\promises-5d855bf6aff586d8.exe)
test the_lines_follow_the_files_sites ... ok
test no_line_says_on_this_computer ... ok
test every_promise_names_tests_that_exist ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests\settings.rs (target\debug\deps\settings-8f98eebba7e02420.exe)
test the_gate_link_keeps_its_values ... ok
test a_gate_deadline_outside_a_tenth_of_a_second_to_an_hour_is_refused ... ok
test a_distribution_id_must_be_a_windows_guid ... ok
test a_gate_value_that_a_shell_could_misread_is_refused ... ok
test the_run_paths_follow_the_layout ... ok
test the_scripts_and_the_paths_name_the_same_temporary_folder ... ok
test a_run_id_that_could_reach_another_place_is_refused ... ok
test the_mcp_config_names_the_gate_as_its_only_server ... ok
test the_settings_are_exactly_the_planned_shape ... ok
test the_settings_match_the_file_the_task_2_runs_used ... ok
test no_allow_rules_and_no_read_rule_on_the_work_folder ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
plain exit 0
```

The containment battery again (lines picked from its output, unchanged except the proxy password, replaced by `(token)`):

```
> cargo test -p steplet-run --offline --test contain -- --ignored --nocapture --test-threads=1
--- inside the fence, the probe's own record: {"proxy": "http://srt:(token)@localhost:3128", "https://example.org/": "reached", "https://example.net/": "failed: URLError: <urlopen error Tunnel connection failed: 403 Forbidden>"}
--- 1.8 s; sent to the screen: []
--- left: "/var/lib/steplet/runs:\n\n/work:"
--- the listener's log: "2026-10-20T11:06:05.771973 from=('127.0.0.1', 40886) bytes=0"
--- the write outside the copy: absent
--- the Windows marker from inside the fence: absent
--- the made-up key in it:
key: 0  (files read: 1)
key, last 20 characters: 0  (files read: 1)
--- the run's token in it:
key: 0  (files read: 1)
key, last 20 characters: 0  (files read: 1)
--- once the agent had ended, the key:
key: 0  (files read: 0)
key, last 20 characters: 0  (files read: 0)
--- and the token:
key: 0  (files read: 0)
key, last 20 characters: 0  (files read: 0)
--- after the run: absent
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 80.20s
battery exit 0
```

(The second count read no files: once the agent had ended, Claude Code had already removed its one file from the folder, and sockets hold no text.)

## Task 6: setup as product code

`steplet_run::setup` now holds what was phase 3's test script: `look` (reads what is set up, changes nothing), `install` (makes Steplet's Linux from nothing) and `remove`. `src/setup.sh` is the old `tests/support/setup_steplet.sh`, which is deleted; `src/look.sh` is new. Words used: **WSL** is Windows' Linux feature; a **distribution** is one Linux installed in it.

### Changes from the plan, recorded here

- **`wsl.exe --install` needs more of Windows' variables than `SystemRoot`.** Steplet starts Windows programs with `SystemRoot` as their only variable (C8). The first install try (18 Oct 2026, by the last session) stopped with `Wsl/InstallDistro/E_ACCESSDENIED` ("Access is denied"), until `wsl --install` was also given this Windows user's `TEMP`, `TMP`, `USERPROFILE` and `LOCALAPPDATA`. That session's notes keep the error code; its output of the failure was not kept.
- **Found this session, and fixed: the install also left a new folder named `%SystemDrive%`** (four Windows cache files, 1 MB) inside the folder the test was started from, `crates/steplet-run`. With no `SystemDrive` variable, a part of Windows could not turn `%SystemDrive%\ProgramData` into `C:\ProgramData`, and wrote to a folder with that literal name, wherever `wsl.exe` was started. In the app, that could be any folder of the receiver's, for example the Downloads folder a `.steplet` file was double-clicked in. Now every call setup makes to `wsl.exe` gets Windows' folder variables (`SystemDrive`, `ProgramData`, `ALLUSERSPROFILE`, `APPDATA`, `LOCALAPPDATA`, `USERPROFILE`, `TEMP`, `TMP`; none is a secret, and these calls start nothing inside Linux), and starts in Windows' System32 folder, which a normal user cannot write. A place for the install that is not a full path is refused, since `wsl.exe` would read it from the folder it starts in. Two new tests, inside `src/setup.rs`.
- **Found this session, for the Setup screen (Task 11): WSL changes the receiver's Windows when it installs a Linux.** It adds a profile named after the Linux to Windows Terminal, and a Start-menu folder (empty here). Both are on this laptop for `steplet` since 13 Oct (phase 3's install). After `steplet-setup-test` was removed, no Terminal profile for it was left, but its empty Start-menu folder was (seen only afterwards: its profile was not looked at while it was installed). Ruled: Setup says so before the install; whether Steplet can stop both is not tried (it would need Steplet's own pinned image, H3's launch item).
- `look.sh` also checks `python3-venv`: `setup.sh` installs it, so a Linux without it is not ready. A new test, `look_checks_every_package_setup_installs`, keeps the two lists the same.
- The plan's Step 4 expected `1 passed; 2 ignored`; there are 3 plain tests (`the_versions_are_the_tested_ones` and the list test besides the version reader).
- `tests/support/mod.rs` needed no change: it never named the old script, which phase 3 ran by hand.

### Measured: which call left the `%SystemDrive%` folder

The four files were all written at the same moment, during Task 6's first runs:

```
2026-10-18 15:54:05.1316121000 16384 crates/steplet-run/%SystemDrive%/ProgramData/Microsoft/Windows/Caches/cversions.2.db
2026-10-18 15:54:05.1430941000 309072 crates/steplet-run/%SystemDrive%/ProgramData/Microsoft/Windows/Caches/{6AF0698E-D558-4F6E-9B3C-3716689AF493}.2.ver0x0000000000000001.db
2026-10-18 15:54:05.1375366000 1240 crates/steplet-run/%SystemDrive%/ProgramData/Microsoft/Windows/Caches/{AF1F47E6-3FA6-41DD-A56A-2358AA2A366C}.2.ver0x0000000000000001.db
2026-10-18 15:54:05.1326080000 662488 crates/steplet-run/%SystemDrive%/ProgramData/Microsoft/Windows/Caches/{DDF571F2-BE98-426D-8288-1A9A39C3FDA2}.2.ver0x0000000000000001.db
```

The other calls setup makes to `wsl.exe` (`--version`, `--terminate`, `--unregister`; it finds its Linux in the registry, not with `--list`), and three it does not make (`--list --verbose`, `--status`, and `--list --online`, which uses the install's download path for a small list), each started with only `SystemRoot` in its own empty folder, left nothing. The `--terminate` and `--unregister` tries name a Linux that does not exist, so they only fail:

```
> wsl.exe --version  (only SystemRoot, in an empty folder)
WSL version: 2.6.3.0
Kernel version: 6.6.87.2-1
WSLg version: 1.0.71
MSRDC version: 1.2.6353
exit 0; left in the folder: 
> wsl.exe --list --verbose  (only SystemRoot, in an empty folder)
  NAME            STATE           VERSION
* Ubuntu          Stopped         2
  steplet         Running         2
  steplet-test    Stopped         2
exit 0; left in the folder: 
> wsl.exe --status  (only SystemRoot, in an empty folder)
Default Distribution: Ubuntu
Default Version: 2
exit 0; left in the folder: 
> env -i SystemRoot=C:\Windows WSL_UTF8=1 /c/Windows/System32/wsl.exe --terminate steplet-no-such-linux
There is no distribution with the supplied name.
Error code: Wsl/Service/WSL_E_DISTRO_NOT_FOUND
exit 127; left in the folder: 
> env -i SystemRoot=C:\Windows WSL_UTF8=1 /c/Windows/System32/wsl.exe --unregister steplet-no-such-linux
There is no distribution with the supplied name.
Error code: Wsl/Service/WSL_E_DISTRO_NOT_FOUND
exit 127; left in the folder: 
> env -i SystemRoot=C:\Windows WSL_UTF8=1 /c/Windows/System32/wsl.exe --list --online
The following is a list of valid distributions that can be installed.
Install using 'wsl.exe --install <Distro>'.

exit 0; left in the folder: 
```

(Shown through `head`; "exit 127" is how Git Bash's `env` passes on `wsl.exe`'s failure.) So the folder came from `wsl.exe --install` itself. Running another install to watch it happen again would be a second 1 to 2 GB download, so the fix is checked with a Windows program that prints what it sees, started exactly as setup starts `wsl.exe` (below).

What else the install left on this Windows, looked at on 20 Oct 2026:

```
== steplet  (made 2026-10-13 17:56:22)
== steplet-setup-test  (made 2026-10-18 15:56:08)
== steplet-test  (made 2026-10-08 16:37:26)
   Alacritty (steplet-test).lnk  2026-10-15 00:43:39
== Ubuntu  (made 2026-08-01 21:25:43)
== fragment {44a496b7-005e-53fd-8552-019f88be0dc0}.json  2026-08-01 21:25:39
   "name": "Ubuntu",
   "name": "Ubuntu",
== fragment {4cdf5ea7-63fc-5d26-9eab-04d9fe1af47a}.json  2026-10-08 16:37:23
   "name": "steplet-test",
   "name": "Ubuntu",
   "name": "Ubuntu Light",
== fragment {ce802844-b61e-5612-a894-8012ec41a7d5}.json  2026-10-13 17:56:10
   "name": "steplet",
   "name": "Ubuntu",
   "name": "Ubuntu Light",
```

(The first five lines are the folders in the Start menu's Programs folder; the "fragment" lines are Windows Terminal's profiles that WSL wrote.) Both leftovers of the 18 Oct test were then removed: the `%SystemDrive%` folder in this worktree, and the empty `steplet-setup-test` Start-menu folder (`rmdir`, which removes only an empty folder):

```
worktree cache folder removed
empty Start-menu folder removed
```

### The tests first

`tests/setup.rs` before the `setup` module existed, and the list test against `look.sh` as it was (each change undone after its run; the script is in the scratchpad: `t6-red.sh`):

```
=== 1. tests/setup.rs before the setup module exists
0
error[E0432]: unresolved import `steplet_run::setup`
  --> crates\steplet-run\tests\setup.rs:12:18
error: could not compile `steplet-run` (test "setup") due to 1 previous error
=== 2. look.sh without python3-venv (as it was), the new list test
20:    for p in git python3 build-essential ca-certificates bubblewrap socat ripgrep nodejs npm; do
test look_checks_every_package_setup_installs ... FAILED
thread 'look_checks_every_package_setup_installs' (12736) panicked at crates\steplet-run\tests\setup.rs:52:5:
  left: ["git", "python3", "build-essential", "ca-certificates", "bubblewrap", "socat", "ripgrep", "nodejs", "npm"]
 right: ["git", "python3", "python3-venv", "build-essential", "ca-certificates", "bubblewrap", "socat", "ripgrep", "nodejs", "npm"]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
=== restored:
lib.rs as written
look.sh as written
```

The two new tests in `src/setup.rs`, run first against helpers that did exactly what the code did then (the four variables, no fixed start folder, no check of the place):

```
> cargo test -p steplet-run --offline --lib setup::
test setup::tests::the_install_command_is_exact_and_refuses_what_wsl_could_misread ... FAILED
test setup::tests::setup_s_windows_programs_know_windows_folders_and_start_in_system32 ... FAILED
thread 'setup::tests::the_install_command_is_exact_and_refuses_what_wsl_could_misread' (5296) panicked at crates\steplet-run\src\setup.rs:296:13:
thread 'setup::tests::setup_s_windows_programs_know_windows_folders_and_start_in_system32' (32396) panicked at crates\steplet-run\src\setup.rs:272:9:
  left: "[%SystemDrive%] [%ProgramData%] [C:\\Users\\RANVIR\\AppData\\Local] [C:\\Users\\RANVIR\\AppData\\Local\\Temp] [C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase4-build-88af2b\\crates\\steplet-run] [%USERNAME%]"
 right: "[C:] [C:\\ProgramData] [C:\\Users\\RANVIR\\AppData\\Local] [C:\\Users\\RANVIR\\AppData\\Local\\Temp] [C:\\WINDOWS\\System32] [%USERNAME%]"
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.05s
```

(The program, `cmd.exe`, printed `%SystemDrive%` and `%ProgramData%` unexpanded, and it started in the crate's own folder. The first test failed on a relative place for the install, which nothing refused.) After the fix, the folder test failed once more, on its own expectation: Windows names its folder `C:\Windows`, while `SystemRoot` says `C:\WINDOWS`; Windows' folder names ignore case, so the test now compares without case:

```
  left: "[C:] [C:\\ProgramData] [C:\\Users\\RANVIR\\AppData\\Local] [C:\\Users\\RANVIR\\AppData\\Local\\Temp] [C:\\Windows\\System32] [%USERNAME%]"
 right: "[C:] [C:\\ProgramData] [C:\\Users\\RANVIR\\AppData\\Local] [C:\\Users\\RANVIR\\AppData\\Local\\Temp] [C:\\WINDOWS\\System32] [%USERNAME%]"
```

### The tests pass

```
> cargo test -p steplet-run --offline --lib setup::
test setup::tests::the_install_command_is_exact_and_refuses_what_wsl_could_misread ... ok
test setup::tests::setup_s_windows_programs_know_windows_folders_and_start_in_system32 ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.05s
> cargo test -p steplet-run --offline --test setup
running 5 tests
test install_makes_a_ready_linux_from_nothing ... ignored, installs a second Linux from the internet: run only with the owner's yes (Z3)
test look_finds_steplets_linux_ready ... ignored, needs Steplet's Linux
test the_versions_are_the_tested_ones ... ok
test look_checks_every_package_setup_installs ... ok
test reads_wsl_version_output ... ok
test result: ok. 3 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
> cargo test -p steplet-run --offline --test setup look_finds_steplets_linux_ready -- --ignored --nocapture
running 1 test
--- State {
    wsl: Ready {
        version: "2.6.3.0",
    },
    linux: Some(
        "{44f36236-99b1-42c0-84bd-7ff8392d51ba}",
    ),
    missing: [],
    gate: true,
}
test look_finds_steplets_linux_ready ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.76s
```

**Z3, the install from nothing** (your yes of 18 Oct 2026). It ran on 18 Oct, after the first try's fix and **before this session's fix**: it installed `steplet-setup-test` from the internet in 172 s, found it ready, and removed it. Its log as it came, except two terminal escape characters, written here as `\e`:

```
> cargo test -p steplet-run --offline --test setup install_makes_a_ready_linux_from_nothing -- --ignored --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running tests\setup.rs (target\debug\deps\setup-e14c5120c2aa1fe9.exe)

running 1 test
setup: Downloading: Ubuntu 26.04 LTS
setup: Installing: Ubuntu 26.04 LTS
setup: Distribution successfully installed. It can be launched via 'wsl.exe -d steplet-setup-test'
setup: --- /etc/wsl.conf: Windows drives not mounted, Windows PATH not added, default user steplet
setup: [boot]
setup: systemd=true
setup: 
setup: [automount]
setup: enabled=false
setup: mountFsTab=true
setup: 
setup: [interop]
setup: enabled=true
setup: appendWindowsPath=false
setup: 
setup: [user]
setup: default=steplet
setup: --- /opt/steplet/gate: the gate program's folder on the Linux disk, owned by root
setup: --- the steplet account: no password, no sudo, no login shell
setup: passwd: password changed.
setup: uid=999(steplet) gid=986(steplet) groups=986(steplet)
setup: --- base tools (choice H4-A; ripgrep because srt needs it, approved 14 Oct 2026)
test install_makes_a_ready_linux_from_nothing has been running for over 60 seconds
setup: \e[0;1;31mFailed to connect to system scope bus via local transport: Connection refused\e[0m
setup: Processing triggers for man-db (2.13.1-1build1) ...
setup: git: 1:2.53.0-1ubuntu1
setup: python3: 3.14.3-0ubuntu2
setup: build-essential: 12.12ubuntu2.26.04.2
setup: bubblewrap: 0.11.1-1ubuntu0.3
setup: socat: 1.8.1.1-1ubuntu0.1
setup: ripgrep: 15.1.0-1ubuntu1
setup: nodejs: 22.22.1+dfsg+~cs22.19.15-1ubuntu1
setup: npm: 9.2.0~ds3-1
setup: --- Claude Code 2.1.220 into /opt/steplet/claude-cli
setup: 
setup: added 3 packages in 8s
setup: --- srt 0.0.77 into /opt/steplet/srt
setup: 
setup: added 5 packages in 3s
setup: --- Steplet's folders, owned by root; nothing under /opt/steplet writable by steplet
setup: --- what was installed
setup: claude runs: /opt/steplet/claude-cli/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe
setup: claude sha256: 674f61f20ff306f3100cf9200e4c36c4b70278b5bef2884549819b942a89c863
setup: claude version: 2.1.220 (Claude Code)
setup: srt package: 0.0.77
setup: drwxr-xr-x 5 root root 4096 Oct 18 10:28 /opt/steplet
setup: drwxr-xr-x 4 root root 4096 Oct 18 10:28 /opt/steplet/claude-cli
setup: drwxr-xr-x 2 root root 4096 Oct 18 10:26 /opt/steplet/gate
setup: drwxr-xr-x 3 root root 4096 Oct 18 10:28 /opt/steplet/srt
setup: drwxr-xr-x 3 root root 4096 Oct 18 10:28 /var/lib/steplet
setup: drwxr-xr-x 2 root root 4096 Oct 18 10:28 /var/lib/steplet/runs
setup: drwxr-xr-x 2 root root 4096 Oct 18 10:28 /work
setup: --- setup finished
setup: The operation completed successfully. 
setup: --- the gate program of this build
setup: gate: /opt/steplet/gate/steplet-gate-ba1474290a83aa52.exe
--- install took 172 s: Ok("{10bac215-f631-4157-9704-aa2d7e523890}")
--- State {
    wsl: Ready {
        version: "2.6.3.0",
    },
    linux: Some(
        "{10bac215-f631-4157-9704-aa2d7e523890}",
    ),
    missing: [],
    gate: true,
}
--- removed: Ok(())
test install_makes_a_ready_linux_from_nothing ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 183.34s

install test exit 0
```

(The red line is `systemd`'s, while the packages install; the install went on and finished.) The install test also got a check that nothing appears in the folder it was started from; the review below removed it, since it looked in the wrong folder.

The whole runner, and the checks:

```
> cargo test -p steplet-run --offline
(19 summary lines, every one ok: 108 passed, 0 failed, 37 ignored; counted from the summary lines)
plain exit 0
> cargo test -p steplet-run --offline -- --ignored --test-threads=1 --skip install_makes_a_ready_linux_from_nothing
(19 summary lines, every one ok: 36 passed, 0 failed; counted from the summary lines)
ignored exit 0
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit 0
```

(The Linux-only run was made before `look.sh` gained `python3-venv`; `look` was run again after, above. Plain: 102 before this task, plus Task 5's review test and Task 6's five new plain tests. Linux-only: 35 before, plus `look`. For `fmt` and `clippy`, Task 7's first test file, already written, was set aside, since the code it tests did not exist yet.)

### What this task did not verify

- **The fix with a real install.** Z3's run was before it; the folder test shows what a program started the new way sees, not what `wsl.exe --install` then writes. A second install (1 to 2 GB) would show it.
- **Install failures:** no network, a full disk, a setup that stops half way and then "Try again". Every step of `setup.sh` is written to be repeated; a repeat was not run.
- **`remove` on the real `steplet`:** run only on the test Linux.
- **A Windows without WSL, or with an older WSL:** only through the version reader's test, on text.
- **A setup that hangs** (for example a package download that never ends): `install` has no time limit and would wait.
- **The setup log's lines come from Linux programs** as they print them, escape characters included (one line of the log above has two). The Setup screen must show them as file text, cleaned (Task 11).
- **What setup downloads** (Ubuntu's packages, Claude Code and `srt` from npm) is fixed by version, not by content; pinning by content is H3's launch item.

### The code review, and what changed

A code-reviewer agent read Task 6's diff (read-only; it ran nothing). It found no way for setup's extra Windows variables to reach a program inside Linux: every command for inside Linux still starts with `SystemRoot` only. Its points, and what changed:

1. **Important, fixed: setup took any plain name, so it could reach one of the receiver's own Linuxes.** `remove("debian")` would have deleted a Linux called `debian`, and `install("debian", …)` would have run Steplet's setup as root inside it. Now `install`, `remove` and `look` take only Steplet's own names: `steplet`, or `steplet-` and a plain name (the tests' `steplet-setup-test`). Test: `only_steplets_own_linux_names_are_taken` (it calls `remove` only with a made-up name no Linux has, so even the old code would only have failed).
2. **Important, fixed: a failed package step showed no reason.** The package tools write to log files, and a failure printed nothing, so the receiver could not tell "no network" from "disk full". Now each of those commands runs through `logged` in `setup.sh`: on a failure, the end of its log goes to the setup's output and the setup stops. Test: `a_failed_setup_step_shows_the_end_of_its_log` (in Steplet's Linux, the helper alone on a made-up command that fails).
3. **Important, fixed: a step could wait for ever** (a stalled download), and a read error was not counted. Now each step has a limit and is ended at it: the download 90 minutes, the setup inside 30, the restart 2, a removal 10. A read error ends the step too, and an ended step's program is stopped. A failure now starts with its exit code. Tests: `a_step_that_takes_too_long_is_ended_at_its_limit` and `a_failed_step_says_its_exit_code_and_the_end_of_what_it_printed`.
4. **Important, fixed: `look` could say "ready" when it saw nothing.** A check now counts only when `look.sh` said `ok` for it; a look that printed nothing finds everything missing. Tests: `look_counts_only_what_it_saw`, and `the_checks_are_the_ones_look_sh_makes` (the list of checks is the one `look.sh` makes).
5. **Important, fixed: "could not look" was reported as a missing part.** `State` gains `trouble` (one field more than the plan's section 6): why Steplet could not look. It is also set when WSL's list of Linuxes cannot be read, which before looked like "not installed"; `install` now stops on that instead of trying a second install. Covered by `look_counts_only_what_it_saw`.
6. **Important, fixed in words: `look` starts Steplet's Linux if it is stopped.** Its description now says so (it changes nothing inside it). Not starting it would leave `look` unable to look most of the time.
7. **Important, fixed: the install test's new check looked in the wrong folder.** Since setup now starts `wsl.exe` in System32, the test's own folder could never get the stray folder. The check is removed; the unit test on the start folder covers the fix.

Minor points done because they were cheap: a test that the commands for inside Linux still get `SystemRoot` only (C8), `programs_for_inside_linux_get_systemroot_only`; a place for the install that is not text is refused; a comment says System32 is closed to normal accounts, not to administrators. Minor points left: the escape characters in setup lines (the Setup screen's job, Task 11); a network path is accepted as a place for the install; the `/etc/fstab` line of `setup.sh` keeps `|| true`, so a read error there would leave that file empty (in Steplet's own Linux only). The results text above was corrected on three points the reviewer named (what was seen of the Terminal profile, which calls setup makes, and the removed check).

The tests first:

```
> cargo test -p steplet-run --offline --lib setup::
1 x error: could not compile `steplet-run` (lib test) due to 9 previous errors
2 x error[E0061]: this function takes 4 arguments but 5 arguments were supplied
2 x error[E0425]: cannot find function `own_name` in this scope
2 x error[E0425]: cannot find function `read_look` in this scope
3 x error[E0425]: cannot find value `CHECKS` in this scope
> cargo test -p steplet-run --offline --test setup a_failed_setup_step -- --ignored
test a_failed_setup_step_shows_the_end_of_its_log ... FAILED
thread 'a_failed_setup_step_shows_the_end_of_its_log' (36696) panicked at crates\steplet-run\tests\setup.rs:84:46:
setup.sh has its helper `logged`
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
```

(The unit tests' errors are shown each once, with how many times it came. `programs_for_inside_linux_get_systemroot_only` is a guard: it would have passed before too.) The time-limit test, with the two limit checks taken out (then put back):

```
test setup::tests::a_step_that_takes_too_long_is_ended_at_its_limit ... FAILED
thread 'setup::tests::a_step_that_takes_too_long_is_ended_at_its_limit' (43960) panicked at crates\steplet-run\src\setup.rs:483:9:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 29.39s
```

(It failed after 29 s on `took < 8 s`: the stalled program ran its whole 30 seconds.)

With the fixes:

```
> cargo test -p steplet-run --offline --lib setup::
running 8 tests
test setup::tests::only_steplets_own_linux_names_are_taken ... ok
test setup::tests::the_install_command_is_exact_and_refuses_what_wsl_could_misread ... ok
test setup::tests::the_checks_are_the_ones_look_sh_makes ... ok
test setup::tests::look_counts_only_what_it_saw ... ok
test setup::tests::programs_for_inside_linux_get_systemroot_only ... ok
test setup::tests::setup_s_windows_programs_know_windows_folders_and_start_in_system32 ... ok
test setup::tests::a_failed_step_says_its_exit_code_and_the_end_of_what_it_printed ... ok
test setup::tests::a_step_that_takes_too_long_is_ended_at_its_limit ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 3.39s
> cargo test -p steplet-run --offline --test setup -- --ignored --nocapture --test-threads=1 --skip install_makes_a_ready_linux_from_nothing
running 2 tests
test a_failed_setup_step_shows_the_end_of_its_log ... --- Output {
    status: Some(
        1,
    ),
    stdout: "",
    stderr: "--- failed with exit code 3: sh -c echo first; echo the made-up reason it failed; exit 3\nfirst\nthe made-up reason it failed\n",
    stdout_bytes: [],
}
ok
test look_finds_steplets_linux_ready ... --- State {
    wsl: Ready {
        version: "2.6.3.0",
    },
    linux: Some(
        "{44f36236-99b1-42c0-84bd-7ff8392d51ba}",
    ),
    missing: [],
    gate: true,
    trouble: None,
}
ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 3.38s
```

**"Try again", measured once** (new test `setup_runs_again_on_a_linux_that_is_there`): the whole setup run again on Steplet's own Linux, which is there, so nothing is downloaded. It changes only Steplet's Linux: Ubuntu upgraded nothing, Claude Code was put in again at the same version (the same SHA-256 as the 18 Oct install), and `look` finds it ready after:

```
> cargo test -p steplet-run --offline --test setup setup_runs_again_on_a_linux_that_is_there -- --ignored --nocapture
running 1 test
setup: --- /etc/wsl.conf: Windows drives not mounted, Windows PATH not added, default user steplet
setup: [boot]
setup: systemd=true
setup: 
setup: [automount]
setup: enabled=false
setup: mountFsTab=true
setup: 
setup: [interop]
setup: enabled=true
setup: appendWindowsPath=false
setup: 
setup: [user]
setup: default=steplet
setup: --- /opt/steplet/gate: the gate program's folder on the Linux disk, owned by root
setup: --- the steplet account: no password, no sudo, no login shell
setup: passwd: password changed.
setup: uid=999(steplet) gid=986(steplet) groups=986(steplet)
setup: --- base tools (choice H4-A; ripgrep because srt needs it, approved 14 Oct 2026)
setup: Solving dependencies...
setup: 0 upgraded, 0 newly installed, 0 to remove and 25 not upgraded.
setup: git: 1:2.53.0-1ubuntu1.1
setup: python3: 3.14.3-0ubuntu2
setup: build-essential: 12.12ubuntu2.26.04.2
setup: bubblewrap: 0.11.1-1ubuntu0.3
setup: socat: 1.8.1.1-1ubuntu0.1
setup: ripgrep: 15.1.0-1ubuntu1
setup: nodejs: 22.22.1+dfsg+~cs22.19.15-1ubuntu1
setup: npm: 9.2.0~ds3-1
setup: --- Claude Code 2.1.220 into /opt/steplet/claude-cli
setup: 
setup: changed 3 packages in 8s
setup: --- srt 0.0.77 into /opt/steplet/srt
setup: 
setup: up to date in 1s
setup: --- Steplet's folders, owned by root; nothing under /opt/steplet writable by steplet
setup: --- what was installed
setup: claude runs: /opt/steplet/claude-cli/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe
setup: claude sha256: 674f61f20ff306f3100cf9200e4c36c4b70278b5bef2884549819b942a89c863
setup: claude version: 2.1.220 (Claude Code)
setup: srt package: 0.0.77
setup: drwxr-xr-x 6 root root 4096 Oct 14 19:20 /opt/steplet
setup: drwxr-xr-x 2 root root 4096 Oct 20 11:18 /opt/steplet/bin
setup: drwxr-xr-x 4 root root 4096 Oct 13 12:33 /opt/steplet/claude-cli
setup: drwxr-xr-x 2 root root 4096 Oct 17 22:40 /opt/steplet/gate
setup: drwxr-xr-x 3 root root 4096 Oct 13 12:33 /opt/steplet/srt
setup: drwxr-xr-x 3 root root 4096 Oct 20 11:15 /var/lib/steplet
setup: drwxr-xr-x 2 root root 4096 Oct 20 11:18 /var/lib/steplet/runs
setup: drwxr-xr-x 2 root root 4096 Oct 20 11:18 /work
setup: --- setup finished
setup: The operation completed successfully. 
setup: --- the gate program of this build
setup: gate: /opt/steplet/gate/steplet-gate-ba1474290a83aa52.exe
--- took 28 s: Ok("{44f36236-99b1-42c0-84bd-7ff8392d51ba}")
--- State {
    wsl: Ready {
        version: "2.6.3.0",
    },
    linux: Some(
        "{44f36236-99b1-42c0-84bd-7ff8392d51ba}",
    ),
    missing: [],
    gate: true,
    trouble: None,
}
test setup_runs_again_on_a_linux_that_is_there ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 28.70s
```

The whole runner, and the checks (`cargo fmt` first only moved some lines of the new tests):

```
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit 0
> cargo test -p steplet-run --offline --no-run
build ok on try 1
> cargo test -p steplet-run --offline
(19 summary lines, every one ok: 114 passed, 0 failed, 39 ignored; counted from the summary lines)
plain exit 0
```

(The first plain run stopped on McAfee's hold on three freshly built test programs, `LNK1104`, so it was built first and run again, as the plan says. Plain: 108 before, plus the six new unit tests. The rest of the Linux-only tests do not use setup's code and were not run again; the three setup ones were.)

What the review's points still leave unverified: the limits were seen on a program that stalls (`ping`), not on a stalled download; `logged` was seen on a made-up failure, not on a real `apt-get` failure.

## Task 7: sender pins in the core: the identity line and the key-changed rule

`steplet_core::identity` says who signed a file, as far as this computer knows: not signed; signature does not match; "signature valid, sender unknown" (a new key); the same key as the files from this sender run here before; or a key known here under another name. A **pin** is Steplet's note of a sender name and the key it came with. The new red rule `id.key-changed` ("This sender name was used before with another key") is found when a known sender name comes with another key, or with no signature at all (S6). It lives with the other rules, but in a list of its own, `MACHINE_RULES`, since no file alone can hit it: only the app applies it, with this computer's pins (R10).

### Changes from the plan, recorded here

- **Sender names are compared ignoring capitals and spaces** ("ana  lee" is "Ana Lee"). The plan does not say how names are compared; compared letter for letter, writing a known name in other capitals would dodge the red finding and show only "sender unknown". The cost: two different senders whose names differ only in capitals or spacing get the red warning. Names that only look alike (another alphabet's letters) still count as different names; see below.
- The rule is written `pub const KEY_CHANGED: &Rule`, like every rule in `rules.rs` (the plan's section 6 said `pub static`).
- The finding is about the whole file (no line number). Its detail names both keys in Steplet's words ("Steplet knows this sender name with the key …. This file is signed with …." or "This file is not signed."); the sender name is file text, so it travels as the finding's quote.
- `docs/rules.md` is written by its test from the code; the new section comes from that generator (`tests/rules_doc.rs` now writes `MACHINE_RULES` too).

### The tests first

```
> cargo test -p steplet-core --offline --test identity
error[E0432]: unresolved import `steplet_core::identity`
 --> crates\steplet-core\tests\identity.rs:4:19
error[E0432]: unresolved import `steplet_core::rules::MACHINE_RULES`
 --> crates\steplet-core\tests\identity.rs:5:42
error[E0425]: cannot find value `KEY_CHANGED` in module `rules`
  --> crates\steplet-core\tests\identity.rs:92:20
error[E0425]: cannot find value `KEY_CHANGED` in module `rules`
   --> crates\steplet-core\tests\identity.rs:100:39
```

(The first eight lines of the errors.) With the code written, the doc test failed until `docs/rules.md` was written again from the code:

```
> cargo test -p steplet-core --offline --test identity --test rules_doc
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test docs_rules_md_lists_every_rule ... FAILED
thread 'docs_rules_md_lists_every_rule' (53712) panicked at crates\steplet-core\tests\rules_doc.rs:42:5:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> STEPLET_BLESS=1 cargo test -p steplet-core --offline --test rules_doc     (in Git Bash)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> git diff docs/rules.md
+
+## Rules about this computer, applied by the app only
+
+These rules need what this computer knows, such as the senders it remembers, so only the desktop app applies them; the public check page does not (decision R10). No file alone can hit them.
+
+| Rule | Severity | What it means | What to do |
+|---|---|---|---|
+| `id.key-changed` | red | This sender name was used before with another key | Someone may be using this name. Ask the sender, another way, whether they changed their key. If they did, type the sentence to run it; Steplet then remembers the new key. |
```

The ruling on names, checked: with `comparable` changed to return the name as it is (planted, then undone), `cargo test -p steplet-core --offline --test identity` fails two tests:

```
test a_known_sender_with_another_key_is_red ... FAILED
test the_same_key_gives_the_same_key_line_and_no_finding ... FAILED
thread 'a_known_sender_with_another_key_is_red' (36592) panicked at crates\steplet-core\tests\identity.rs:20:15:
thread 'the_same_key_gives_the_same_key_line_and_no_finding' (7988) panicked at crates\steplet-core\tests\identity.rs:63:5:
  left: (KeyKnownAs { key: "SHA256:made-up-key-of-ana-lee", names: ["Ana Lee"] }, None)
 right: (SameKey { key: "SHA256:made-up-key-of-ana-lee", runs: 2 }, None)
test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### The tests pass

```
> cargo test -p steplet-core --offline
     Running tests\identity.rs (target\debug\deps\identity-c0037e436ec81595.exe)
test a_known_key_under_another_name_is_said_but_not_a_finding ... ok
test a_known_sender_unsigned_is_red ... ok
test a_known_sender_with_another_key_is_red ... ok
test a_new_sender_is_new_key ... ok
test an_invalid_signature_gives_no_second_finding ... ok
test the_machine_rule_is_not_in_rules ... ok
test the_same_key_gives_the_same_key_line_and_no_finding ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\rules_doc.rs (target\debug\deps\rules_doc-658b2949100a0f98.exe)
test docs_rules_md_lists_every_rule ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\corpus.rs (target\debug\deps\corpus-8220497a53785d57.exe)
test every_hostile_file_belongs_to_a_rule ... ok
test counts_of_the_ordinary_files ... ok
test every_rule_has_a_hit_and_a_near_miss ... ok
test list_every_file_with_its_findings ... ok
test every_file_gives_its_expected_json ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
(26 summary lines, every one ok: 128 passed, 0 failed, 0 ignored; counted from the summary lines)
core exit 0
> cargo build -p steplet-wasm --target wasm32-unknown-unknown --release --offline
   Compiling steplet-wasm v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase4-build-88af2b\crates\steplet-wasm)
    Finished `release` profile [optimized] target(s) in 18.47s
wasm exit 0
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit 0
```

(Three test files' lines picked from the core's 26; the corpus is unchanged, so "every rule has a hit" still holds.)

### What this task did not verify

- **Names that only look alike,** such as "Ana Lee" written with a Cyrillic "а", and names written alike in other ways (see the review below: "STRASSE" for "Straße", an accent typed as a second character): a different name to Steplet, so the file shows "signature valid, sender unknown" and no red finding. Trust on first use cannot tell such names apart; the identity line is the honest status.
- **The pins themselves:** where they are kept, when they are written (only when Run is pressed, S5), and a damaged pins file are Task 10.
- **The colour on screen:** the key-changed finding is about this computer ("On this computer" on the preview), and it must still make the preview red (row 22). That is the app's job, Task 10.

### The code review, and what changed

A code-reviewer agent read Task 7's diff (read-only) and ran the identity, rules and rules-doc tests (7, 12 and 1 passed). It found nothing critical, and found that the key-changed detail never holds file text. Its points:

1. **Important, fixed in words and tests: the comment claimed more than the code does.** Only capitals and runs of spaces are ignored (Rust's lower case, not full Unicode case folding, and no joining of an accent typed as a second character). "STRASSE" against "Straße", or "José" written with a separate accent, is another name: "sender unknown", no red finding, never "the same key as before". The comment and "What this task did not verify" now say so. (An invisible character in a sender name already blocks the file, `block.invisible-in-field`.) Test: `only_capitals_and_spaces_are_ignored`. A full Unicode comparison would need a library in the core, which is not in the cache; not done.
2. **Important, fixed in words: the names in `KeyKnownAs` are file text** (from earlier files), so the screens must show them as file text. The comment says so; Task 10 draws them.
3. **Important, fixed: an empty name matched every empty-named pin.** No file can have an empty sender (the header reader refuses it), but a damaged pins file could hold one. Now an empty name matches no pin. Test: `an_empty_name_matches_no_pin`.
4. Fixed with point 1: the unsigned case is now also checked with other capitals and a no-break space (`a_known_sender_unsigned_is_red`).

Minor points left, for Task 10, which writes the pins: two spellings of one name with the same key are both listed in `KeyKnownAs`; two pins for one name with different keys make either key "the same key" (S5 keeps one pin per name, so the pins file must never hold two); the shared rule checks (unique ids, plain text) run over `RULES` only, and `MACHINE_RULES` is checked by its own test.

```
> cargo test -p steplet-core --offline --test identity     (before the fix)
test an_empty_name_matches_no_pin ... FAILED
thread 'an_empty_name_matches_no_pin' (4112) panicked at crates\steplet-core\tests\identity.rs:80:5:
  left: (Unsigned, Some(Finding { rule: Rule { id: "id.key-changed", severity: Red, title: "This sender name was used before with another key", advice: "Someone may be using this name. Ask the sender, another way, whether they changed their key. If they did, type the sentence to run it; Steplet then remembers the new key." }, line: None, detail: Some("Steplet knows this sender name with the key SHA256:made-up-key-of-ana-lee. This file is not signed."), quote: Some("") }))
 right: (Unsigned, None)
test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> cargo test -p steplet-core --offline --test identity     (after)
test a_new_sender_is_new_key ... ok
test a_known_key_under_another_name_is_said_but_not_a_finding ... ok
test a_known_sender_unsigned_is_red ... ok
test an_invalid_signature_gives_no_second_finding ... ok
test the_machine_rule_is_not_in_rules ... ok
test only_capitals_and_spaces_are_ignored ... ok
test a_known_sender_with_another_key_is_red ... ok
test an_empty_name_matches_no_pin ... ok
test the_same_key_gives_the_same_key_line_and_no_finding ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
identity exit 0
> cargo fmt --all -- --check
fmt exit 0
> cargo test -p steplet-core --offline
26 summary lines: 130 passed, 0 failed, 0 ignored
core exit 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit 0
> cargo build -p steplet-wasm --target wasm32-unknown-unknown --release --offline
wasm exit 0
```

(`only_capitals_and_spaces_are_ignored` and the new unsigned cases passed before the fix too: they pin down what the code already did, now said in its comment. The "26 summary lines" line is counted from the run's own summary lines.)

## Task 8: the app's shell

Steplet now has a desktop window. It is a Tauri app (`crates/steplet-app`, the program `steplet.exe`) that shows the screens from `web/`, written in Svelte. So far it shows only the empty Open screen; opening files comes with Task 10. What is fixed from the start: the window is always titled `Steplet`, the page may load nothing from outside the app (its **page rules**, the content security policy), it may only listen for Steplet's own events and call Steplet's own commands, it can never go to another address or open a second window, and it sends nothing to the network. Two check scripts keep the screens honest: one refuses every way of drawing text as HTML, one checks that every pair of colours is readable.

### Changes from the plan, recorded here

- **Found and closed: trying to leave the app's page sent a request to the internet.** The plan relies on Tauri's navigation check to keep the window on the app's own page. Measured on the real window: the page stays where it is, but WebView2 had already sent the request (example.com answered `200`) before the check cancelled it. Every request the web view makes now goes to a closed port on this computer instead (`--proxy-server=http://127.0.0.1:47`, with no exception for local addresses), so nothing can leave even where the page rules and the navigation check come too late. The app's own page and its channel to Rust never use the network, and the test shows both still work. See "Found while measuring" below.
- **Z2, the download for writing the screens' data types (J7 A):** your yes was recorded by the plan session on 20 Oct and confirmed again here on 21 Oct before the download. Exactly three packages were downloaded (`ts-rs` 12.0.1, `ts-rs-macros` 12.0.1, `termcolor` 1.4.1); everything else came from the caches, and cargo went back to `--offline` straight after.
- **Six packages that came with Tauri were under 14 days old today** (`toml` and five of its parts, published 8 and 9 Oct). Each was moved to an older version already in the cache (`cargo update --offline --precise`), as the plan says. The age script counts age from its own fixed date, 11 Oct 2026, so its "under 14 days" lines are counted again from today's date by a small reader (`ages_today.py`, in the session's scratch folder); both outputs are below.
- **The page must be built before Rust.** The app carries its page inside the program, so `cargo build`, `cargo test` and `cargo clippy` of `steplet-app` need `web/dist` first: `npm --prefix web run build`. Without it, Tauri stops with "The `frontendDist` configuration is set to `"../../web/dist"` but this path doesn't exist". Every later check runs the page build first.
- **The browser's developer tools are off** in normal builds and on only in the test build (`e2e`), which already opens the debugging port. Not in the plan; it removes a way into the page that no receiver needs.
- **The app's commands are listed once,** in `src/commands.rs`. `build.rs` hands the same list to Tauri, so Tauri checks every call from the page against `capabilities/main.json`; `tests/shell.rs` checks that the file grants exactly the two event permissions and one line per listed command. The list is empty until Task 10.
- **The app's identifier is `dev.steplet.desktop`.** WebView2 keeps its own data (its cache, for example) in `%LOCALAPPDATA%\dev.steplet.desktop`. The plan names no identifier.
- **`web/package-lock.json` is phase 0's S9 lock file** with only its name changed (`s9-check` to `steplet-web`; checked with `diff`), so `npm ci --offline` installs exactly what S9 tested.
- **`@types/node` is not in the npm cache,** so instead of a download (Z6) the screen tests declare the one Node function they use, in `web/app/tests/node.d.ts`.
- **The contrast check also checks focus rings and control borders at 3 : 1** (WCAG 2.2, rule 1.4.11), and lists each pair's ratio with `--list`. **The raw-HTML check refuses four more ways** of turning text into HTML besides the plan's five: `createContextualFragment`, `setHTMLUnsafe`, `parseHTMLUnsafe` and `srcdoc`.
- **`web/shared/base.css`** (fonts, the 4-pixel spacing scale, the focus ring, buttons and links) is a file the plan did not list.
- **A rule for later screens:** the page rules (`style-src 'self'`) also block a `style="…"` written into a screen's markup, so screens set styles only through classes.
- Real-window tests are the files named `real-*.spec.ts`, in their own Playwright project; Playwright still starts the screens' preview server, so `npm --prefix web run build:screens` comes first. `svelte-check` runs with `--fail-on-warnings`, so a warning fails `npm run check`.

### Step 1: installing from the caches

```
> npm --prefix web ci --offline --no-audit --no-fund
added 53 packages in 3s
> diff <(tr -d '\r' < spikes/s9-check/package-lock.json) <(tr -d '\r' < web/package-lock.json)
2c2
<   "name": "s9-check",
---
>   "name": "steplet-web",
7c7
<       "name": "s9-check",
---
>       "name": "steplet-web",
> cargo build -p steplet-app --offline        (empty App.svelte; the page built first)
     Locking 331 packages to latest Rust 1.98 compatible versions
      Adding toml_edit v0.20.2 (available: v0.20.7)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 40s
build exit: 0
```

The lock file gained 332 packages (Tauri's 331 and `steplet-app` itself) and lost none (compared by a small script over the old and new `[[package]]` lists: `old 115 new 447 removed [] added 332`).

### Z2: the download, with your yes

```
> cargo add ts-rs@=12.0.1 --offline -p steplet-app
error: the crate `ts-rs@=12.0.1` could not be found in registry index.
exit: 101
> cargo add ts-rs@=12.0.1 -p steplet-app          (online, after your yes)
    Updating crates.io index
      Adding ts-rs =12.0.1 to dependencies
             Features:
             + serde-compat
             33 deactivated features
    Updating crates.io index
     Locking 3 packages to latest Rust 1.98 compatible versions
      Adding termcolor v1.4.1
      Adding ts-rs v12.0.1
      Adding ts-rs-macros v12.0.1
exit: 0
lock: removed [] added [('termcolor', '1.4.1'), ('ts-rs', '12.0.1'), ('ts-rs-macros', '12.0.1')]
> cargo build -p steplet-app --offline
  attempting to make an HTTP request, but --offline was specified
offline exit: 101
> cargo build -p steplet-app                       (online, the one build that downloads)
 Downloading crates ...
  Downloaded ts-rs v12.0.1
  Downloaded termcolor v1.4.1
  Downloaded ts-rs-macros v12.0.1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 25.28s
online exit: 0
```

Every cargo command after this one ran with `--offline`.

### Step 2 and 3: the tests first, and seeing them fail

`crates/steplet-app/tests/shell.rs` reads `tauri.conf.json` and `capabilities/main.json` and checks the fixed values of the plan (title, sizes, `dragDropEnabled` false, `withGlobalTauri` false, the exact page rules, no plugins, only the two event permissions and the app's own commands), and that a normal build opens no debugging port. Before the app had any of it:

```
> cargo test -p steplet-app --offline --test shell
error[E0425]: cannot find value `COMMANDS` in crate `steplet_app`
  --> crates\steplet-app\tests\shell.rs:63:61
error[E0425]: cannot find value `COMMANDS` in crate `steplet_app`
  --> crates\steplet-app\tests\shell.rs:67:27
error[E0425]: cannot find function `browser_args` in crate `steplet_app`
  --> crates\steplet-app\tests\shell.rs:75:29
error[E0425]: cannot find function `browser_args` in crate `steplet_app`
  --> crates\steplet-app\tests\shell.rs:92:22
error[E0425]: cannot find function `browser_args` in crate `steplet_app`
  --> crates\steplet-app\tests\shell.rs:94:22
error: could not compile `steplet-app` (test "shell") due to 5 previous errors
exit: 101
```

With those two names added (an empty command list, and wry's own browser arguments), the test ran and failed on the first setting the config did not have yet:

```
test no_debug_port_without_the_e2e_feature ... ok
test the_browser_keeps_wrys_default_arguments ... ok
test the_page_is_titled_steplet ... ok
test the_window_and_security_settings_are_fixed ... FAILED
thread 'the_window_and_security_settings_are_fixed' (52388) panicked at crates\steplet-app\tests\shell.rs:25:5:
  left: Null
 right: "main"
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> node web\scripts\no-raw-html.mjs --self-test
self-test passed
```

Both check scripts refuse a planted mistake, in their self-tests and on a file planted from outside:

```
> node web/scripts/no-raw-html.mjs <scratch folder holding Planted.svelte, whose one line is <p>{@html x}</p>>
..\..\..\..\..\..\Users\RANVIR\AppData\Local\Temp\claude\C--dev-runFile--claude-worktrees-steplet-phase4-build-88af2b\ac0cdc0b-e668-433d-856a-95a60957c963\scratchpad\planted\Planted.svelte:1: {@html
raw HTML refused: 1 place(s)
no-raw-html exit: 1
> node web/scripts/contrast.mjs <a copy of tokens.css with --text-muted: #eeeeee>
light --text-muted on --bg: 1.05 : 1 (needs 4.5)  <-- too low
light --text-muted on --surface: 1.16 : 1 (needs 4.5)  <-- too low
light --text-muted on --surface-sunken: 1.07 : 1 (needs 4.5)  <-- too low
light --text-muted on --file-bg: 1.11 : 1 (needs 4.5)  <-- too low
contrast exit: 1
> node web/scripts/contrast.mjs --self-test
self-test passed
> node web/scripts/no-raw-html.mjs --self-test
self-test passed
```

### Step 4: the shell

`src/lib.rs` makes the one window from its config entry (`"create": false`, then `WebviewWindowBuilder::from_config`) and never changes its title. It refuses every navigation that is not the app's own page (`http://tauri.localhost`, checked by `is_own_page`) and every new window, and gives WebView2 its start arguments (`browser_args`): wry's own defaults (which turn off SmartScreen, among others, so no addresses go to Microsoft), the closed-port proxy (below), and, only in the `e2e` build, the debugging port. `web/app/App.svelte` shows the empty Open screen: the plan's sentence split into a heading and a line, "Open a file…" (not wired yet; Task 10), "or drop it on this window", and "Setup" at the top right. The colours are tokens in `web/shared/tokens.css`, one set for light and one for dark.

Looked at in the screens build, in both themes and at both sizes (1100 × 800 and 560 × 720; those screenshots stayed in the scratch folder, since Task 14 takes the kept set): the first version stretched the dashed panel over the whole window, which read like a placeholder; it is now a centred panel, with Setup above it. After that, right as it is: the text is centred, the panel's right edge lines up with Setup's at both widths, and the dark theme reads well.

### Step 5: the real window, driven through its debugging port (S13)

```
> cargo build -p steplet-app --offline --features e2e
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.36s
> Start-Process target\debug\steplet.exe -WorkingDirectory C:\Windows\System32
pid: 51888; running: True
> (Invoke-WebRequest -UseBasicParsing http://127.0.0.1:9333/json/version).Content
{
   "Browser": "Edg/155.0.4283.62",
   "Protocol-Version": "1.3",
   "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/155.0.0.0 Safari/537.36 Edg/155.0.0.0",
   "V8-Version": "15.5.9.7",
   "WebKit-Version": "537.36 (@aa3a7406ba8c53a83536df539f07124d73c899bd)",
   "webSocketDebuggerUrl": "ws://127.0.0.1:9333/devtools/browser/89ecc6d8-f3d8-4f34-8699-97ff917631d3"
}
> netstat -ano | grep ":9333" | grep LISTEN
  TCP    127.0.0.1:9333         0.0.0.0:0              LISTENING       38248
> (Get-Process -Name steplet).MainWindowTitle
title: 'Steplet'
```

Playwright attaches (`chromium.connectOverCDP`), so every later "real" screenshot can go through it; the PowerShell window capture is not needed. The port listens on 127.0.0.1 only. The real window's screenshot is `docs/screens/phase-4/real-open-empty-light.png` (this laptop is in light mode).

### Found while measuring: trying to leave the app's page sent a request to the internet

The second real-window test tries three ways out of the app: a `fetch` of a web address, `window.open`, and setting `location.href`. A probe (a throwaway test, since deleted) logged each one:

```
1. fetch
  console error: Connecting to 'https://example.com/' violates the following Content Security Policy directive: "connect-src ipc: http://ipc.localhost". The action has been blocked.
  console error: Fetch API cannot load https://example.com/. Refused to connect because it violates the document's Content Security Policy.
  -> refused: TypeError: Failed to fetch
2. window.open
  -> null
3. location.href
  request https://example.com/ type=document nav=true
  cdp requestWillBeSent https://example.com/
  response https://example.com/ 200
  cdp responseReceived https://example.com/ 200
  failed https://example.com/: net::ERR_ABORTED
  cdp loadingFailed net::ERR_ABORTED blocked=- canceled=true
  -> url now http://tauri.localhost/, pages 1
```

The page rules refuse the `fetch` before any request, the new window is refused, and the page stays on the app's own page; but the navigation's request reached example.com, which answered `200`, before Tauri's navigation check cancelled it. Nothing in a file can start a navigation today (file text is drawn as plain text, and links are never clickable), but the promise is that opening and previewing a file make no network request (row 28), so the window must not be able to send one at all.

The fix: WebView2 starts with `--proxy-server=http://127.0.0.1:47 --proxy-bypass-list=<-loopback>`. Every request it would send goes to a closed port on this computer, local addresses included (nothing listens on port 47 here, and it is not one of the ports Chromium refuses to use). The tests, first failing:

```
> cargo test -p steplet-app --offline --features e2e --test shell        (before the fix)
test the_web_view_sends_nothing_to_the_network ... FAILED
thread 'the_web_view_sends_nothing_to_the_network' (49264) panicked at crates\steplet-app\tests\shell.rs:104:5:
test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> npm --prefix web test -- real-open                                      (before the fix)
  ok 1 [real] › app\tests\real-open.spec.ts:6:1 › the real window opens on the empty Open screen (160ms)
  x  2 [real] › app\tests\real-open.spec.ts:25:1 › the real window reaches nothing outside the app (3.0s)
    Error: expect(received).toEqual(expected) // deep equality
    + Received  + 3
    +   "https://example.com/",
  1 failed
  1 passed (8.5s)
```

With the fix in, the real test still saw an answer. A second probe showed where it came from: the web view's own cache, holding the page from the first probe, not the internet. A never-visited address failed at the closed port:

```
navigate to https://example.com/
  answer https://example.com/ 200 fromDiskCache=true ip=[2606:4700:10::6814:179a]
  failed net::ERR_ABORTED canceled=true
  -> url now http://tauri.localhost/
navigate to https://example.com/?steplet-probe=1792522537208
  failed net::ERR_PROXY_CONNECTION_FAILED canceled=false
  failed net::ERR_ABORTED canceled=true
  -> url now http://tauri.localhost/
```

So each try in the test now uses an address never asked for before, and the test fails on any answer from outside the app. To show the test catches a missing fix, the proxy line was taken out of a build, then put back:

```
> npm --prefix web test -- real-open        (a build without the closed-port proxy)
  ok 1 [real] › app\tests\real-open.spec.ts:6:1 › the real window opens on the empty Open screen (148ms)
  x  2 [real] › app\tests\real-open.spec.ts:25:1 › the real window reaches nothing outside the app (3.0s)
    + Received  + 3
    +   "https://example.com/?steplet-test=navigate-1792522624728",
  1 failed
  1 passed (7.1s)
> the web view's browser process and its start arguments, as Windows lists them (the fixed build)
pid 15020 parent 13644 started 10/21/2026 00:24:19
  --user-data-dir="C:\Users\RANVIR\AppData\Local\dev.steplet.desktop\EBWebView"
  --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection
  --proxy-bypass-list=<-loopback>
  --proxy-server=http://127.0.0.1:47
  --remote-debugging-port=9333
> npm --prefix web test -- real-open        (the fixed build)
Running 2 tests using 1 worker
saved docs/screens/phase-4/real-open-empty-light.png
  ok 1 [real] › app\tests\real-open.spec.ts:6:1 › the real window opens on the empty Open screen (148ms)
  ok 2 [real] › app\tests\real-open.spec.ts:25:1 › the real window reaches nothing outside the app (3.1s)
  2 passed (7.1s)
```

The same test also calls a made-up command through the app's channel to Rust and gets Tauri's own refusal, so the closed-port proxy does not cut the page off from Rust.

### The 14-day rule (F12, S10)

The first count, after Tauri and `ts-rs` came in. `lock_ages.py` counts age from its fixed date, 11 Oct 2026, so its own summary line over-counts; `ages_today.py` reads the same output and counts from today:

```
> python -I C:\Users\RANVIR\.claude\handoffs\2026-10-11-1606-steplet-tools\lock_ages.py Cargo.lock
termcolor          1.4.1                  published 2024-01-10  1005 days old
ts-rs              12.0.1                 published 2026-01-31   253 days old
ts-rs-macros       12.0.1                 published 2026-01-31   253 days old
444 crates.io packages in the lock file; 23 under 14 days old; 0 yanked
lock_ages exit: 0
> python -I ages_today.py t8-lock-ages.log
serde_spanned 1.1.2 published 2026-10-08, 13 days old on 2026-10-21
toml 1.1.8+spec-1.1.0 published 2026-10-09, 12 days old on 2026-10-21
toml_datetime 1.1.2+spec-1.1.0 published 2026-10-08, 13 days old on 2026-10-21
toml_edit 0.25.17+spec-1.1.0 published 2026-10-09, 12 days old on 2026-10-21
toml_parser 1.1.5+spec-1.1.0 published 2026-10-09, 12 days old on 2026-10-21
toml_writer 1.1.3+spec-1.1.0 published 2026-10-08, 13 days old on 2026-10-21
444 packages read; on 2026-10-21: 6 under 14 days old; 0 yanked
```

(The three lines of the new download, picked from 444; the other 441 are in the run's log.) Each of the six was moved to an older version, the cached one where Windows builds need it (`toml_edit` 0.25.x is used only when building for other systems, so its version comes from cargo's saved index):

```
> cargo update --offline -p toml@1.1.8+spec-1.1.0 --precise 1.1.4+spec-1.1.0
 Downgrading toml v1.1.8+spec-1.1.0 -> v1.1.4+spec-1.1.0
> cargo update --offline -p toml_parser@1.1.5+spec-1.1.0 --precise 1.1.3+spec-1.1.0
error: failed to select a version for the requirement `toml_parser = "^1.1.5"`
candidate versions found which didn't match: 1.1.3+spec-1.1.0
required by package `toml_edit v0.25.17+spec-1.1.0`
> cargo update --offline -p toml_writer@1.1.3+spec-1.1.0 --precise 1.1.2+spec-1.1.0
 Downgrading toml_writer v1.1.3+spec-1.1.0 -> v1.1.2+spec-1.1.0
> cargo update --offline -p toml_datetime@1.1.2+spec-1.1.0 --precise 1.1.1+spec-1.1.0
error: failed to select a version for the requirement `toml_datetime = "^1.1.2"`
candidate versions found which didn't match: 1.1.1+spec-1.1.0
required by package `toml_edit v0.25.17+spec-1.1.0`
> cargo update --offline -p serde_spanned@1.1.2 --precise 1.1.1
 Downgrading serde_spanned v1.1.2 -> v1.1.1
> cargo update --offline -p toml_edit@0.25.17+spec-1.1.0 --precise 0.25.15+spec-1.1.0
 Downgrading toml_edit v0.25.17+spec-1.1.0 -> v0.25.15+spec-1.1.0
> cargo update --offline -p toml_parser@1.1.5+spec-1.1.0 --precise 1.1.3+spec-1.1.0      (again, now toml_edit allows it)
 Downgrading toml_parser v1.1.5+spec-1.1.0 -> v1.1.3+spec-1.1.0
> cargo update --offline -p toml_datetime@1.1.2+spec-1.1.0 --precise 1.1.1+spec-1.1.0
 Downgrading toml_datetime v1.1.2+spec-1.1.0 -> v1.1.1+spec-1.1.0
left: [('serde_spanned', '1.1.2'), ('toml', '1.1.8+spec-1.1.0'), ('toml_datetime', '1.1.2+spec-1.1.0'), ('toml_edit', '0.25.17+spec-1.1.0'), ('toml_parser', '1.1.5+spec-1.1.0'), ('toml_writer', '1.1.3+spec-1.1.0')]
came in: [('serde_spanned', '1.1.1'), ('toml', '1.1.4+spec-1.1.0'), ('toml_datetime', '1.1.1+spec-1.1.0'), ('toml_edit', '0.25.15+spec-1.1.0'), ('toml_parser', '1.1.3+spec-1.1.0'), ('toml_writer', '1.1.2+spec-1.1.0')]
```

(The "note: pass `--verbose`" and "location searched" lines are left out.) The count again, on the changed lock file:

```
> python -I C:\Users\RANVIR\.claude\handoffs\2026-10-11-1606-steplet-tools\lock_ages.py Cargo.lock
serde_spanned      1.1.1                  published 2026-03-31   194 days old
toml               1.1.4+spec-1.1.0       published 2026-07-28    75 days old
toml_datetime      1.1.1+spec-1.1.0       published 2026-03-31   194 days old
toml_edit          0.25.15+spec-1.1.0     published 2026-09-11    30 days old
toml_parser        1.1.3+spec-1.1.0       published 2026-07-27    76 days old
toml_writer        1.1.2+spec-1.1.0       published 2026-07-14    89 days old
444 crates.io packages in the lock file; 17 under 14 days old; 0 yanked
lock_ages exit: 0
> python -I ages_today.py t8-lock-ages2.log
444 packages read; on 2026-10-21: 0 under 14 days old; 0 yanked
> python -I web\scripts\npm_ages.py web\package-lock.json
@rolldown/binding-android-arm-eabi       1.2.13       published 2026-10-07    14 days old
88 npm packages in the lock file; 0 under 14 days old; 0 withdrawn or deprecated
npm_ages exit: 0
```

(The npm lock lists 88 packages, the 53 installed here plus other systems' optional parts; the youngest, `rolldown` 1.2.13, is exactly 14 days old today.)

### Step 6: the tests pass

The page is built first (`npm --prefix web run build`), then:

```
> cargo test -p steplet-app --offline
     Running tests\shell.rs (target\debug\deps\shell-a76efebd90a8ec1b.exe)
test no_debug_port_without_the_e2e_feature ... ok
test the_browser_keeps_wrys_default_arguments ... ok
test the_web_view_sends_nothing_to_the_network ... ok
test the_page_is_titled_steplet ... ok
test the_window_goes_nowhere_but_the_apps_own_page ... ok
test the_window_and_security_settings_are_fixed ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
app tests exit: 0
> cargo test -p steplet-app --offline --features e2e --test shell
test the_browser_keeps_wrys_default_arguments ... ok
test the_e2e_build_opens_its_debug_port_on_this_computer_only ... ok
test the_web_view_sends_nothing_to_the_network ... ok
test the_page_is_titled_steplet ... ok
test the_window_goes_nowhere_but_the_apps_own_page ... ok
test the_window_and_security_settings_are_fixed ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> npm --prefix web run check
> svelte-check --tsconfig ./tsconfig.json --fail-on-warnings && node scripts/no-raw-html.mjs && node scripts/contrast.mjs
1792523179047 COMPLETED 97 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 62 pairs
check exit: 0
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.02s
clippy exit: 0
> cargo test --workspace --offline
workspace tests exit: 0
57 summary lines: 274 passed, 0 failed, 39 ignored
```

(The "57 summary lines" line is counted from the run's own summary lines; the 39 ignored are the Linux-only tests, which this task does not touch. The app's two empty unit-test lines are left out.) The first time `npm run check` ran it found four errors, all in the setup, fixed before the run above: the build-mode guard in `vite.config.ts` also refused the type checker, the CSS imports needed Vite's own types, and the test's Node import had no types (`node.d.ts`).

### What this task did not verify

- **Other ways out of the window** than the three the real test tries (`fetch`, `window.open`, `location.href`): for example a form, an image or a frame. The page rules refuse each of those by name (`form-action`, `img-src`, `default-src`), and the closed-port proxy stops any request that slips past, but neither was tried on these.
- **The window's smallest size** (560 × 720) was not tried by dragging the real window's edge; it is checked in the config only. The screens at that size were looked at in the screens build.
- **The dark theme in the real window:** this laptop is in light mode, so the real screenshot is light; the dark tokens were looked at in the screens build only.
- **Whether WebView2 itself sends anything outside the page,** for example its own update checks or its crash reports. The proxy covers the web view's network traffic; WebView2's own updater runs as a separate Windows service and was not watched.
- **That port 47 stays closed.** If a program on this computer ever listened on port 127.0.0.1:47, the web view's requests would reach that program (still nothing leaves the computer).
- **The app's commands under Tauri's check:** the list is still empty, so a command refused for lack of its `allow-` line is first seen in Task 10.

### The code review, and what changed

A code-reviewer agent read Task 8's diff (read-only) and ran the app's tests and clippy (6 passed, clean). It found nothing critical, and checked as fine: the proxy setting cannot fall back to a direct connection; the page rules in `tauri.conf.json` are exactly the ones the test pins, and Tauri adds nothing to them (the built page has no inline script or style); the permission file grants only the two event permissions; the debugging port exists only in the `e2e` build; and the Open screen has one heading, real buttons and a visible focus ring. Its Important points, each fixed test-first:

1. **The comments claimed more than Tauri does.** Listing the app's commands makes Tauri check each call against `capabilities/main.json`, but only once the list holds at least one command: with an empty list tauri-build writes no permission list for the app, and Tauri then lets the page call any command the app registers. So a command registered but left off the list would have no check at all. The comments now say so, and a new test reads the app's source and fails unless the commands it registers (`generate_handler![…]`) are exactly the ones listed. Shown failing with a planted command:

```
> cargo test -p steplet-app --offline --test shell every_command      (a command `planted` registered, not listed)
test every_command_the_app_registers_is_listed_for_tauri_to_check ... FAILED
assertion `left == right` failed: src/commands.rs must list exactly the commands the app registers
  left: ["planted"]
 right: []
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

2. **The real test's check of the channel to Rust was weak.** If Tauri's own channel (`http://ipc.localhost`) fails, Tauri quietly sends the call another way and only warns on the console, so "Tauri refused the made-up command" passed either way. The test now also requires the channel's own answer and no such warning. It passed on the fixed build at once, which confirms the closed-port proxy leaves the channel working. To show the new check is not empty, the channel's address was taken out of the page rules in one build (then put back): the old check still passed, the new one failed:

```
> npm --prefix web test -- real-open        (a build whose page rules leave out http://ipc.localhost)
  ok 1 [real] › app\tests\real-open.spec.ts:6:1 › the real window opens on the empty Open screen (148ms)
  x  2 [real] › app\tests\real-open.spec.ts:25:1 › the real window reaches nothing outside the app (3.1s)
    Error: expect(received).toContain(expected) // indexOf
    Expected value: "http://ipc.localhost/no_such_command"
    Received array: []
      75 |     expect(ipc).toMatch(/^refused by Tauri: .*no_such_command/);
    > 76 |     expect(answered).toContain("http://ipc.localhost/no_such_command");
      77 |     expect(warnings.filter((w) => w.includes("IPC custom protocol failed"))).toEqual([]);
  1 failed
  1 passed (7.2s)
> npm --prefix web test -- real-open        (the fixed build)
Running 2 tests using 1 worker
saved docs/screens/phase-4/real-open-empty-light.png
  ok 1 [real] › app\tests\real-open.spec.ts:6:1 › the real window opens on the empty Open screen (162ms)
  ok 2 [real] › app\tests\real-open.spec.ts:25:1 › the real window reaches nothing outside the app (3.0s)
  2 passed (7.2s)
```

3. **The own-page check took an address with a user name in it** (`http://user@tauri.localhost/`). Harmless today (it is the same page), but the test did not try it. It now refuses a user name or a password:

```
> cargo test -p steplet-app --offline --test shell        (before the fix)
test the_window_goes_nowhere_but_the_apps_own_page ... FAILED
thread 'the_window_goes_nowhere_but_the_apps_own_page' (48656) panicked at crates\steplet-app\tests\shell.rs:165:9:
http://user@tauri.localhost/ is not the app's own page
test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> cargo test -p steplet-app --offline         (after)
test no_debug_port_without_the_e2e_feature ... ok
test the_browser_keeps_wrys_default_arguments ... ok
test the_page_is_titled_steplet ... ok
test the_web_view_sends_nothing_to_the_network ... ok
test the_window_goes_nowhere_but_the_apps_own_page ... ok
test the_window_and_security_settings_are_fixed ... ok
test every_command_the_app_registers_is_listed_for_tauri_to_check ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit: 0
> npm --prefix web run check
1792524127548 COMPLETED 97 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 62 pairs
```

(The app's four empty test lines are left out.)

Minor points left for later, each noted for the task it belongs to:
- The crate names libraries nothing uses yet (`windows-sys`, `ts-rs`, `serde`, the core and the runner); Tasks 9 to 11 use them.
- In a release build, a start-up failure (for example WebView2 missing) shows nothing at all; phase 4 runs only from the build folder, where a console shows the message. Recorded for "not verified" at Task 16.
- The test pins the page rules as written in the config, not as the window receives them. The reviewer read Tauri's code and found nothing added for this page; a real test could read the page's own header.
- The raw-HTML check compares letter case exactly and line by line, scans six kinds of file, and does not list `<iframe`, `<svelte:element>` or `DOMParser`. The page rules still block inline scripts and frames from outside.
- The contrast check's pairs are written by hand: a screen that puts a colour on a background not in the list, or misspells a colour's name, is not noticed. Tasks 10 to 14 add each pair they use.
- `npm_ages.py` takes a package's name from its folder, which would be wrong for a renamed (aliased) package; there is none in this lock file.
- "Open a file…" and "Setup" do nothing yet (Task 10 and Task 11 wire them).
- Nothing stops a release build with the `e2e` feature on; it is off unless asked for.

## Task 9: what the screens get: view types, the one backend interface, and fixtures from real Rust output

The screens now have their data. `crates/steplet-app/src/views.rs` turns the core's report on a file, the runner's live progress and the runner's report into plain data for the screens, in which every piece of text from the file, the repository, the agent or a command is already cut into **display pieces** (plain text, a badge for each character that cannot be seen, and links with their real address). The screens decide no colour and judge nothing: the outcome, each check and each answer arrive already decided. The TypeScript types the screens use (`web/shared/types/`, 28 files) are written from these Rust types by ts-rs, and a test fails when they fall behind. The screens talk to Rust through one interface, `web/app/lib/backend.ts`, with two versions: `tauri.ts` (the app's own commands) and `fake.ts`, a stand-in that replays one of 57 saved scenarios (`web/app/tests/fixtures/`), each made from real corpus files, the real checker and the runner's own report builder. Only the screen-test build holds the stand-in; a test checks that the app's build does not.

### Changes from the plan, recorded here

- **A link's two parts are display pieces too,** not plain strings as the plan's section 6 sketched: the core's link pieces can hold invisible characters, which plain strings would hide.
- **A finding's `detail` stays plain text** (it is always Steplet's own words: the core's rule, and its details are dates, fixed place names and famous site names), while its `quote` (the file's words) is display pieces.
- **Two ways of cutting text:** one entry of a list (a step, a command the file allows, a changed file) shows a line break inside it as a badge (`U+000A`), so one entry never looks like two; a block (the body, a command's output, what the agent says, a whole command) keeps its line breaks.
- **Each header entry keeps its line number,** since findings name lines; the repository is a small record (address and commit), not a pair.
- **"Cannot run here" never repeats the file's host:** that is file text, and the reason is Steplet's own sentence ("Steplet fetches repositories only from github.com, gitlab.com, codeberg.org and bitbucket.org, and this file's repository is somewhere else."). The file's address is in its own panel.
- **The preview's colour for the Run button is the worst of the file's findings and the findings about this computer** (row 22: a changed sender key makes it red); its `outcome` stays the core's own, which the test compares.
- **The screen types are compared, not written on every test run:** `the_screen_types_are_current` fails when they are out of date and writes them with `STEPLET_BLESS=1`, like the fixtures. That is stricter than the plan's "write, then `git diff`", and works before the files are in git.
- **Two small public functions in the runner,** `exit_code_of` and `call_what`: the report already read exit codes this way (H13); the live log now reads them in the same one place. Nothing else in the runner changed (its plain tests: 114 passed, as before).
- **The live log's rows do not carry Steplet's check yet** ("Allowed…", "Refused…"): the gate's decisions are not part of the live progress today. Task 12, which builds the run screen, adds them.
- The saved scenarios hold a few things beyond the plan's list, each needed by a state of section 8: an open error, the Open screen's note, the install's error, what Stop sends, Steplet's words for a yes and a no, and where a report was saved (or why it could not be). The interface also has `startNote` (the note after a crash, Task 12's command `start_note`).

### The tests first

```
> cargo test -p steplet-app --offline --test views
error[E0432]: unresolved import `steplet_app::views`
 --> crates\steplet-app\tests\views.rs:7:18
  |
7 | use steplet_app::views::{Here, Seg, line, preview, report_view};
error[E0433]: cannot find `views` in `steplet_app`
   --> crates\steplet-app\tests\views.rs:117:29
error: could not compile `steplet-app` (test "views") due to 2 previous errors
exit: 101
> cargo test -p steplet-app --offline --test fixtures          (nothing saved yet)
test fixtures_are_current ... FAILED
test the_screen_types_are_current ... FAILED
28 in C:\dev\runFile\.claude\worktrees\steplet-phase4-build-88af2b\crates\steplet-app/../..\web/shared/types: ["AnswerView.ts is missing", "CheckView.ts is missing", "ChooserView.ts is missing", "CommandView.ts is missing", "Counts.ts is missing", "EndView.ts is missing", "EntryView.ts is missing",
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

(The second run's first failure was in the test data itself: one of the made-up files did not read, because a step held a colon and a space, which this format allows only in quotes; the step was reworded.) `file_text_reaches_the_screen_only_as_segments` gives every part of a made-up file the word "Qzx" and then looks through the whole preview for that word outside a display piece. It passed at once; to show it can fail, a finding's file quote was planted into its plain `detail` field (then undone):

```
> cargo test -p steplet-app --offline --test views file_text      (detail planted with the quote)
test file_text_reaches_the_screen_only_as_segments ... FAILED
assertion `left == right` failed: file text outside display pieces
  left: ["preview.findings[0].detail", "preview.findings[1].detail"]
 right: []
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

`the_app_build_holds_no_stand_in_backend` (in `tests/shell.rs`) reads every file of `web/dist`. With the screens build planted there (then the app build put back):

```
test the_app_build_holds_no_stand_in_backend ... FAILED
thread 'the_app_build_holds_no_stand_in_backend' (51364) panicked at crates\steplet-app\tests\shell.rs:185:13:
C:\dev\runFile\.claude\worktrees\steplet-phase4-build-88af2b\crates\steplet-app/../../web/dist\assets\index-NZc1u4Ct.js holds the stand-in backend
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.01s
```

### Writing the saved data, and looking at it

```
> STEPLET_BLESS=1 cargo test -p steplet-app --offline --test fixtures      (in Git Bash)
test the_screen_types_are_current ... ok
test fixtures_are_current ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
> ls web/app/tests/fixtures | wc -l; du -sh web/app/tests/fixtures
57
1.8M	web/app/tests/fixtures
```

The largest is `run-long-output.json` (1.3 MB: a command output of 50,000 lines, for the plan's responsiveness test). Each scenario was read back with a small script (`fixture_summary.py`, in the session's scratch folder) that prints what a person would check; its output, one line per scenario (cut at 400 characters):

```
ask-10-seconds-left            run  prepared > started > agent_says > question(Run this command: touch asked.txt | the file does not ask for this command | waited 290s)
ask-2-more-waiting             run  prepared > started > agent_says > question(Run this command: touch asked.txt | the file does not ask for this command | waited 4s) > question(Run this command: touch second.txt | the file does not ask for this command | waited 2s) > question(Open this web address: [link https://example.org/thir | the file does not ask for this site | waited 0s)
ask-command                    run  prepared > started > agent_says > question(Run this command: touch asked.txt | the file does not ask for this command | waited 0s)
ask-file-write                 run  prepared > started > agent_says > question(Change this file: /work/r1/repo/NOTES.md | the file does not ask to change files here | waited 0s)
ask-web-address                run  prepared > started > agent_says > question(Open this web address: [link https://example.org/read | the file does not ask for this site | waited 0s)
ask-withdrawn                  run  prepared > started > agent_says > question(Run this command: touch asked.txt | the file does not ask for this command | waited 0s) > question_closed
chooser-needs-setup            chooser  ready=False lines=6 items=0
chooser-ready-15-sites         chooser  ready=True lines=6 items=30
chooser-ready-no-sites         chooser  ready=True lines=6 items=0
chooser-wsl-missing            chooser  ready=False lines=6 items=0
open-could-not-read            open  open_error=Steplet could not read this file: Windows says access is den
open-crash-cleared             open  note=Steplet was closed while a file was running. It stopped what
open-empty                     open
open-reading                   open  outcome=green run=green id=unsigned  findings=0 here=[]  title=Count the lines of the README in Hello-World
prepare-commit-missing         run  failed
prepare-fetch-failed           run  failed
prepare-fetching               run
prepare-ready-removed          run  prepared
preview-amber                  preview  outcome=amber run=amber id=unsigned  findings=1 here=[]  title=Run the tests of tiny-py-lib
preview-blocked                preview  outcome=blocked run=blocked id=unsigned  findings=1 here=[]  title=Count the README lines
preview-green                  preview  outcome=green run=green id=unsigned  findings=0 here=[]  title=Count the lines of the README in Hello-World
preview-header-unreadable      preview  outcome=blocked run=blocked id=unsigned  findings=1 here=[]
preview-id-key-changed         preview  outcome=green run=red id=new_key  findings=0 here=['id.key-changed']  title=Count the lines of the README in Hello-World
preview-id-known-as            preview  outcome=green run=green id=key_known_as  findings=0 here=[]  title=Count the lines of the README in Hello-World
preview-id-new-key             preview  outcome=green run=green id=new_key  findings=0 here=[]  title=Count the lines of the README in Hello-World
preview-id-same-key            preview  outcome=green run=green id=same_key  findings=0 here=[]  title=Count the lines of the README in Hello-World
preview-id-unsigned            preview  outcome=green run=green id=unsigned  findings=0 here=[]  title=Count the lines of the README in Hello-World
preview-longest                preview  outcome=red run=red id=unsigned  findings=41 here=[]  title=Run the whole test suite of the payments service a
preview-not-runnable-host      preview  outcome=green run=not_runnable id=unsigned  findings=0 here=[]  reason=Steplet fetches repositories only from github.com, gitlab.com, codeberg.org and bitbucket.org, and this file's repository is somewhere else.  title=Count the lines of the README in Hello-World
preview-not-runnable-no-repo   preview  outcome=green run=not_runnable id=unsigned  findings=1 here=[]  reason=This file names no repository, so Steplet has nothing to fetch and run.  title=Count the README lines
preview-red                    preview  outcome=red run=red id=unsigned  findings=1 here=[]  title=Count the README lines
report-40-commands             report  finished(end=finished completed=True cmds=[('allowed', 0), ('allowed', 0), ('allowed', 0), ('allowed', 0)] answers=[] faults=0 changed=[])
report-faults                  report  finished(end=did_not_finish completed=False cmds=[('allowed', 0)] answers=[] faults=2 changed=[])
report-finished                report  finished(end=finished completed=True cmds=[('allowed', 0)] answers=[] faults=0 changed=[])
report-forged-result           report  finished(end=finished completed=True cmds=[('allowed', 1)] answers=[] faults=0 changed=['?? README.bak'])
report-interrupted             report  finished(end=stopped_by_you completed=False cmds=[('allowed', 0), ('allowed', 0), ('refused', None), ('allowed', 0)] answers=['You said no.'] faults=0 changed=['?? build/'])
report-save-failed             report  save_error=Steplet could not save the report: Windows says the disk is   finished(end=finished completed=True cmds=[('allowed', 0)] answers=[] faults=0 changed=[])
report-saved                   report  saved_to=C:\Users\Ana\Documents\steplet-report-2026-10-11-1405.txt  finished(end=finished completed=True cmds=[('allowed', 0)] answers=[] faults=0 changed=[])
report-test-failed             report  finished(end=finished completed=True cmds=[('allowed', 1)] answers=[] faults=0 changed=[])
report-tripwire                report  finished(end=stopped_by_steplet completed=False cmds=[('allowed', 0), ('not_asked', 0)] answers=[] faults=0 changed=[])
run-busy                       run  prepared > started > agent_says > call(wc -l README) > result(exit=0,lines=1) > call(cat README) > result(exit=0,lines=1) > agent_says > call(rm -rf build) > result(exit=None,lines=1) > call(wc -l README) > result(exit=0,lines=1)
run-long-command               run  prepared > started > call(wc -l README docs/very/long/pa) > result(exit=0,lines=0)
run-long-output                run  prepared > started > call(wc -l README) > result(exit=0,lines=50000)
run-model-refusal              run  prepared > started > model_refused > ended > finished(end=model_refused completed=False cmds=[] answers=[] faults=0 changed=[])
run-started                    run  prepared > started
run-stopping                   run  prepared > started > agent_says > call(wc -l README) > result(exit=0,lines=1) > call(cat README) > result(exit=0,lines=1) > agent_says > call(rm -rf build) > result(exit=None,lines=1) > call(wc -l README) > result(exit=0,lines=1)
run-tripwire                   run  prepared > started > call(wc -l README) > result(exit=0,lines=1) > call(curl [link https://example.net) > result(exit=0,lines=0) > skipped > finished(end=stopped_by_steplet completed=False cmds=[('allowed', 0), ('not_asked', 0)] answers=[] faults=0 changed=[])
run-wrong-key                  run  prepared > started > ended > finished(end=did_not_finish completed=False cmds=[] answers=[] faults=1 changed=[])
setup-all-ready                setup  wsl:ready | linux:ready | tools:ready | gate:ready  key=True switch=on ready=True
setup-could-not-look           setup  wsl:ready | linux:unknown | tools:unknown | gate:unknown  key=True switch=off ready=False
setup-install-failed           setup  wsl:ready | linux:missing | tools:unknown | gate:unknown  key=False switch=off ready=False  setup_lines=4  setup_error=Setting up Steplet's Linux stopped at "install the tools": t
setup-installing               setup  wsl:ready | linux:missing | tools:unknown | gate:unknown  key=False switch=off ready=False  setup_lines=8
setup-key-saved                setup  wsl:ready | linux:ready | tools:missing | gate:ready  key=True switch=off ready=False
setup-nothing-done             setup  wsl:ready | linux:missing | tools:unknown | gate:unknown  key=False switch=off ready=False
setup-switch-broken            setup  wsl:ready | linux:ready | tools:ready | gate:ready  key=True switch=broken ready=True
setup-wsl-missing              setup  wsl:missing | linux:unknown | tools:unknown | gate:unknown  key=False switch=off ready=False
setup-wsl-too-old              setup  wsl:missing | linux:unknown | tools:unknown | gate:unknown  key=False switch=off ready=False
```

(This listing was taken before Stop's saved data and the reply words were added; those changed no line of it.) What was checked in it: the key-changed file is red only through its "On this computer" finding, as row 22 asks, while its own outcome stays green; the forged output ("exit 0", "Steplet: every test passed") still shows exit code 1; a refused command has no exit code; each question carries the checker's real reason; "could not look" is told apart from "missing". The preview's identity line for a changed key is "signature valid, sender unknown" plus the red finding, which is what the core gives (Task 7). The run fixtures carry no per-call check yet (see above).

### The tests pass

```
> cargo test -p steplet-app --offline
     Running tests\fixtures.rs (target\debug\deps\fixtures-69f45a4a78dd73f9.exe)
test the_screen_types_are_current ... ok
test fixtures_are_current ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
     Running tests\shell.rs (target\debug\deps\shell-52d52ba15b88d342.exe)
test no_debug_port_without_the_e2e_feature ... ok
test the_browser_keeps_wrys_default_arguments ... ok
test the_web_view_sends_nothing_to_the_network ... ok
test the_page_is_titled_steplet ... ok
test the_window_goes_nowhere_but_the_apps_own_page ... ok
test the_window_and_security_settings_are_fixed ... ok
test the_app_build_holds_no_stand_in_backend ... ok
test every_command_the_app_registers_is_listed_for_tauri_to_check ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\views.rs (target\debug\deps\views-5ef4c53d6ec120ac.exe)
test a_newline_inside_one_entry_shows_as_a_badge ... ok
test file_text_reaches_the_screen_only_as_segments ... ok
test no_view_holds_the_key ... ok
test the_preview_agrees_with_the_core ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
app exit: 0
> cargo test -p steplet-run --offline
19 summary lines: 114 passed, 0 failed, 39 ignored
run exit: 0
> npm --prefix web run check
1792525139448 COMPLETED 132 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 62 pairs
> npm --prefix web run build ; npm --prefix web run build:screens
dist/assets/index-CT59eRkx.js   24.85 kB │ gzip: 10.21 kB
dist-screens/assets/index-NZc1u4Ct.js                            33.17 kB │ gzip:  12.05 kB
> grep -l "steplet-stand-in-backend" -r web/dist web/dist-screens
web/dist-screens/assets/index-NZc1u4Ct.js
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit: 0
```

(The app's empty test lines are left out; the runner's line is counted from its 19 summary lines. `npm run check` type-checks both backends against the one interface.)

### What this task did not verify

- **The runner's Linux-only report tests,** which also read exit codes: they could not run. Both failed before any exit code was read, because Steplet's gate program did not start inside Steplet's Linux (`/opt/steplet/gate/steplet-gate-ba1474290a83aa52.exe: Invalid argument`), so Claude Code ran with no gate and every tool call failed closed. The cause is on Windows' side and not in this task's change; it is measured, and fixed, in "Found while running Task 9's Linux checks" below. The runner's plain tests, which cover the exit-code rule on made-up events, pass (114).
- **The stand-in backend in a browser:** no screen uses it yet; Task 10's screen tests are its first users. Only the type checker has seen it against the interface.
- **The real backend:** none of its commands exists in Rust yet (Tasks 10 to 13).
- **Look-alike sender names and unusual text in a fixture's live rows** beyond what the core's display pieces already cover (phase 1's display tests).

### Found while running Task 9's Linux checks: Windows refused to start the gate from its usual place

**What happened.** The runner's Linux-only report tests failed: Claude Code ran with no gate, so every tool call failed closed (nothing ran unchecked). Claude Code's own error output said why:

```
> type %LOCALAPPDATA%\Steplet\logs\t10a.stderr.txt
apiKeyHelper failed: exited 1: /opt/steplet/gate/steplet-gate-ba1474290a83aa52.exe: Invalid argument
Error: MCP tool mcp__gate__decide (passed via --permission-prompt-tool) not found. Available MCP tools: none
```

The gate program had not changed: `steplet-gate-ba1474290a83aa52.exe` is the copy installed on 17 Oct, and its last good run was on 20 Oct at 16:48. Phase 3 met the same failure once (its Task 7, on another path) and did not find the cause; its remedy was a new file name for each build, which this shows is not enough. What was measured this time, step by step:

```
> wsl -d steplet -u root -- /opt/steplet/gate/steplet-gate-ba1474290a83aa52.exe --version
/opt/steplet/gate/steplet-gate-ba1474290a83aa52.exe: Invalid argument
> wsl -d steplet -u root -- /opt/steplet/gate/steplet-gate-14b2ba5f90b138dd.exe --version      (an older copy)
/opt/steplet/gate/steplet-gate-14b2ba5f90b138dd.exe: Invalid argument
> (WSL's handler for Windows programs, in Steplet's Linux)
enabled
interpreter /init
flags: P
> wsl --terminate steplet, then the same start
/opt/steplet/gate/steplet-gate-ba1474290a83aa52.exe: Invalid argument
> wsl -d Ubuntu -- /mnt/c/Windows/System32/whoami.exe ; and a copy of it on Ubuntu's own disk, /tmp/w.exe
laptop-r1rg22o7\ranvir
laptop-r1rg22o7\ranvir
> gate_probe.sh, in Steplet's Linux as root (each start with an empty input)
the installed file: exit 1 /opt/steplet/gate/steplet-gate-ba1474290a83aa52.exe: Invalid argument
same name, in /tmp/p: exit 2 steplet-gate: unknown option "--version"
other name, gate folder: exit 2 steplet-gate: unknown option "--version"
same pattern, gate folder: exit 2 steplet-gate: unknown option "--version"
> touch_probe.sh: a new timestamp on the refused file, then a new file (new inode) with the same bytes at the same path
before touch: exit 1 /opt/steplet/gate/steplet-gate-14b2ba5f90b138dd.exe: Invalid argument
after touch: exit 1 /opt/steplet/gate/steplet-gate-14b2ba5f90b138dd.exe: Invalid argument
before: exit 1 /opt/steplet/gate/steplet-gate-14b2ba5f90b138dd.exe: Invalid argument
inode 14394
inode 1665
fresh file, same path: exit 1 /opt/steplet/gate/steplet-gate-14b2ba5f90b138dd.exe: Invalid argument
> from Windows (PowerShell), the refused path and a fresh copy of the same bytes, both through \\wsl.localhost\steplet
zz-control.exe -> exit 2 : steplet-gate: unknown option "--version"
steplet-gate-ba1474290a83aa52.exe -> start failed: Program 'steplet-gate-ba1474290a83aa52.exe' failed to run: Error performing inpage operation
```

(Exit code 2 with "unknown option" is the gate itself answering: it started.) So Windows refuses to start a program from that one exact path ("Error performing inpage operation": Windows could not load the program's pages), whatever file is put there, while the same bytes start from any new path. It is not this session's own restrictions either: the same PowerShell started the fresh copy. The phase 3 path that failed then (`spike-gate.exe`) starts again today, so such a refusal can also pass. **Why Windows does this is still not known.**

**The fix.** A gate path that Windows refuses stops every later run, on this laptop or a receiver's. So each run now starts the gate from a fresh copy of its own: `gate_for_run` copies the installed gate to `/opt/steplet/gate/runs/<run id>.<16 random hex digits>.exe` as root (mode 755, so the agent cannot change the program Windows starts), checks that the copy has the same SHA-256, and `run_live` uses it for the run's settings and for the gate service's check of who connects. `discard.sh` removes the run's copies with its folders (a run id holds no `.`, so no other run's copies match), and says so if one is left. The test harness's own gate (`support::serve_gate`) does the same. One test that ended the gate by its old file name now ends the run's own copy (`taskkill /IM <run id>.*`).

```
> cargo test -p steplet-run --offline --test gate --no-run      (the test written first)
error[E0432]: unresolved import `steplet_run::gate_for_run`
  --> crates\steplet-run\tests\gate.rs:23:14
> cargo test -p steplet-run --offline --test gate each_run_starts -- --ignored --test-threads=1
test each_run_starts_the_gate_from_a_path_of_its_own ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 3.31s
> cargo test -p steplet-run --offline --test report -- --ignored --test-threads=1      (the two that failed)
test no_error_from_claude_code_does_not_always_mean_exit_code_0 ... ok
test use_case_1_report_shows_the_test_command_s_real_exit_code_and_no_trust ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 26.20s
> ls -la /opt/steplet/gate/runs/      (after those runs)
total 8
drwxr-xr-x 2 root root 4096 Oct 20 19:54 .
drwxr-xr-x 3 root root 4096 Oct 20 19:53 ..
```

The first full Linux run after the fix stopped at `a_gate_that_dies_mid_request_refuses_it_and_the_restarted_gate_answers_the_next`: the test ended the gate by its old file name, so nothing was ended (`--- taskkill /F /IM steplet-gate-ba1474290a83aa52.exe:` printed nothing) and both writes were allowed (`left: [true, true]`, `right: [false, true]`). The test now ends the run's own copy. The whole Linux suite, run again with that change:

```
> cargo test -p steplet-run --offline --no-fail-fast -- --ignored --test-threads=1 --skip install_makes_a_ready_linux_from_nothing
Running tests\ask_live.rs (target\debug\deps\ask_live-c45a1842be6b2015.exe)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.76s
Running tests\contain.rs (target\debug\deps\contain-e9ca5c3e4707e558.exe)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 71.72s
Running tests\gate.rs (target\debug\deps\gate-ae3f0e2f1d920802.exe)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 29.77s
Running tests\launch.rs (target\debug\deps\launch-2c821a776c9bbf7e.exe)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 60.95s
Running tests\prepare.rs (target\debug\deps\prepare-ca0ff602a7cf43dc.exe)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 23.70s
Running tests\report.rs (target\debug\deps\report-ff7dc5d64e57faad.exe)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 23.50s
Running tests\setup.rs (target\debug\deps\setup-e14c5120c2aa1fe9.exe)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 35.01s
Running tests\tripwires.rs (target\debug\deps\tripwires-8535515b103c05f7.exe)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.55s
exit: 0
19 summary lines: 39 passed, 0 failed, 0 ignored
> cargo test -p steplet-run --offline
19 summary lines: 114 passed, 0 failed, 40 ignored
plain exit: 0
> cargo clippy -p steplet-run --all-targets --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.95s
> cargo fmt --all -- --check
fmt exit: 0
```

(The test files with no Linux-only test, and their empty summary lines, are left out; the totals are counted from all 19 summary lines. The install test, which downloads 1 to 2 GB, was not run again, as the rules say. The 40 ignored in the plain run are the Linux-only tests, one more than before: the new one.)

**What this did not verify:** why Windows refuses a path (unknown); whether it could happen in the middle of a run, to the run's own fresh copy (never seen: each refused path had been started many times over days); a receiver's computer. Each run now writes one 395 KB copy and removes it after; the older content-named copies in `/opt/steplet/gate` stay where they are (the installed one is the source of every run's copy). Decision H8 changes from "a new name for each build" to "a fresh copy for each run"; Task 16 records it in `docs/decisions.md`.

### The code review, and what changed

A code-reviewer agent read Task 9's diff (read-only) and ran the app's tests, clippy and the screen checks (all clean). It found nothing critical, and checked as fine: every other plain-text field holds only Steplet's own words (the not-runnable reason, the checker's reason in a question, the check and end words, the trust line, the notes, a finding's detail, the promises' text); `line()` and `shown()` are used in the right places and never break a link or drop text; the preview's colour logic; `exit_code_of` is exactly the old rule; the app build cannot hold the stand-in. Its Important points, each fixed test-first:

1. **A failed preparation's message could carry git's output as plain text.** Git prints what the repository's host sends, so it is file text. A failure is now Steplet's own sentence plus a separate `detail` in display pieces (`FailureView`), for a run that could not prepare and for a failed install. Test: `what_a_failed_program_printed_is_file_text` (before the fix there was no such type: Task 9's saved `prepare-fetch-failed` put git's words into the plain `reason`).
2. **Setup's "could not look" line carried other programs' error output as plain text** (`setup::State.trouble` ends with what programs inside the Linux and `wsl.exe` printed). It now goes to the line's `detail`, in display pieces. Test: `what_a_program_printed_when_setup_could_not_look_is_file_text`.
3. **Setup gave the wrong reason** when Windows' Linux feature is there but Steplet's Linux is not installed yet ("…until Windows' Linux feature is there"). Each line now names the first real cause: the feature missing, the feature too old, Steplet could not look, or its Linux not installed yet. Test: `setup_says_why_it_cannot_look_yet`.
4. **The stand-in worked out the setup screen itself** after a key was saved or the switch changed (and so left "ready" stale). The saved scenarios now hold the setup screen after each change, worked out by the Rust side (`SetupChanges`), and the stand-in only replays them.

Shown failing by putting the old behaviour back into the new code (then undone), and as Task 9 had saved it:

```
> cargo test -p steplet-app --offline --test views setup      (old wording and old trouble text planted)
test setup_says_why_it_cannot_look_yet ... FAILED
test what_a_program_printed_when_setup_could_not_look_is_file_text ... FAILED
assertion `left == right` failed
  left: "Steplet cannot look at the tools in its Linux until Windows' Linux feature is there."
 right: "Steplet can look at the tools in its Linux once its Linux is installed."
assertion `left == right` failed: program output outside display pieces
  left: ["view.gate.text", "view.linux.text", "view.tools.text"]
 right: []
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
> git show ec1102a:web/app/tests/fixtures/prepare-fetch-failed.json      (Task 9 as committed)
      "kind": "failed",
      "stage": "Fetching the repository",
      "reason": "Steplet could not fetch https://github.com/octocat/Hello-World.git. Git said: could not resolve host github.com. Nothing ran."
```

The reviewer also asked for wider tests of the rule itself (its points 8 and 9), and one was added: `live_and_report_views_keep_outside_text_in_pieces` puts a marker word in every text the agent, a command or the model fills, and checks it is found only inside display pieces in the live rows, a question and a report. It passed at once: it pins down what the code already did.

```
> cargo test -p steplet-app --offline --test views
test setup_says_why_it_cannot_look_yet ... ok
test a_newline_inside_one_entry_shows_as_a_badge ... ok
test what_a_failed_program_printed_is_file_text ... ok
test what_a_program_printed_when_setup_could_not_look_is_file_text ... ok
test no_view_holds_the_key ... ok
test live_and_report_views_keep_outside_text_in_pieces ... ok
test file_text_reaches_the_screen_only_as_segments ... ok
test the_preview_agrees_with_the_core ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
> STEPLET_BLESS=1 cargo test -p steplet-app --offline --test fixtures      (the scenarios and types written again)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
> cargo test -p steplet-app --offline
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
app exit: 0
> npm --prefix web run check
1792526410423 COMPLETED 134 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 62 pairs
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit: 0
```

(The app's empty test lines are left out.) Minor points left for later, each noted for the task it belongs to:
- The live rows' call ids come from Claude Code's stream; they are used only to match a result to its call, and no screen may draw them (Task 12).
- When a file's header cannot be read but its signature is valid, the identity line says "new key" without comparing it to the pins (the file is blocked anyway).
- Two small tidy-ups in `preview()` (the not-runnable branch, and taking the worst finding from the core's outcome).
- A question's number is cut to 32 bits without a check; bless removes every `.ts` in `web/shared/types`, so nothing hand-written may go there.
- A dropped file goes to Rust as a list of numbers, several times its size (Task 10 cuts it to 262,145 bytes first).
- `call_what` could be `what` made public; setup's missing items show Steplet's short check names ("wsl.conf"), which a receiver may not understand (Task 11 words them).

## Task 10: the Open and Preview screens, and pins

A receiver can now open a file and read everything it asks for before anything runs. The Open screen takes a file through "Open a file…" (Windows' own Open dialog, through the page's file picker) or a drop anywhere on the window; the Preview screen shows, from top to bottom, the colour as a word, an icon and its one line of meaning; who signed the file; the counts and the expiry date; what Steplet found; the file's own words in their marked "From the file" panel; what is true on this computer and not in the file; and the Run button last. A red file's Run works only once the sentence is typed exactly; a blocked file has no Run, only the advice of its first blocked finding. `crates/steplet-app/src/open.rs` reads at most 262,145 bytes of a file and rates it with the core; `src/pins.rs` keeps the senders this computer knows in `%LOCALAPPDATA%\Steplet\senders.json` (read when a file is opened, written only through a temporary file and a rename, and only when Run starts, which comes with Task 12). Text from the file is drawn only by `web/shared/FileText.svelte`, as plain text with a badge for each character that shows nothing, and links as their shown text plus their real address, never clickable.

### Changes from the plan, recorded here

- **No `open_path` command.** A web page cannot learn the path of a file dropped on it or chosen in its file picker, so the page never asks Rust to read a path: it sends the first 262,145 bytes (`openBytes`). A double-clicked file (Task 11) reaches Rust as the program's own start argument and is read there (`initial_preview`).
- **A damaged list of known senders is a finding about this computer** (`here.pins-unreadable`, amber), the app's own fact like the runner's host list. The list is then left exactly as it is: Steplet never overwrites a file it cannot read.
- **Two small public functions,** each so the app and the runner use one rule: `same_sender` in the core (how two sender names count as the same name; the core's own `identity` now uses it) and `fetchable` in the runner (whether Steplet would fetch a repository address; `prepare` now uses it). Behaviour unchanged.
- **One pin per sender name:** the same key again keeps the name as first written and counts one more run; a new key replaces the pin (S5).
- **The "cannot run here" reason was reworded** to say what Steplet checks: "Steplet fetches repositories only from plain https:// addresses on github.com, gitlab.com, codeberg.org and bitbucket.org, and this file's repository is not one of those."
- **Ten screen tests, not the plan's seven.** Looking at the screenshots found three faults the seven did not catch; each got its test first (below).
- **The saved preview scenarios are now made through the app's own `open::preview_file`,** not a copy of its steps in the test, so they cannot drift from what the app does.
- **The bytes kept for Run are exactly the bytes previewed,** even if the page sent more (Rust cuts them to 262,145 bytes too).
- **`crates/steplet-app/permissions/` is ignored by git,** like `gen/`: Tauri's build tool writes it again on every build.

### The tests first

The Rust tests, run against the last commit (2134cf4) with this task's two test files copied in (a plain copy of the commit, made with `git archive`, outside the repository):

```
> cargo test -p steplet-app --offline --test open --test pins
error[E0432]: unresolved import `steplet_app::open`
 --> crates\steplet-app\tests\open.rs:8:18
  |
8 | use steplet_app::open::{MAX_READ, preview_file, read_file};
  |                  ^^^^ could not find `open` in `steplet_app`
error[E0432]: unresolved import `steplet_app::pins`
 --> crates\steplet-app\tests\pins.rs:7:18
  |
7 | use steplet_app::pins::{self, Pins};
  |                  ^^^^ could not find `pins` in `steplet_app`
error[E0432]: unresolved import `steplet_app::pins`
 --> crates\steplet-app\tests\open.rs:9:18
  |
9 | use steplet_app::pins::{self, Pins};
  |                  ^^^^ could not find `pins` in `steplet_app`
error: could not compile `steplet-app` (test "pins") due to 1 previous error
exit: 101
```

The screen tests, with the old window (the empty Open screen only) still in place:

```
> npm --prefix web run build:screens; npm --prefix web test -- preview
  1) [screens] › app\tests\preview.spec.ts:11:1 › the_run_button_comes_after_the_full_list ─────────
  2) [screens] › app\tests\preview.spec.ts:22:1 › a_blocked_file_has_no_run_button ─────────────────
  3) [screens] › app\tests\preview.spec.ts:28:1 › a_red_file_needs_the_exact_sentence ──────────────
  4) [screens] › app\tests\preview.spec.ts:40:1 › text_that_imitates_steplet_stays_in_its_panel ────
  5) [screens] › app\tests\preview.spec.ts:57:1 › nothing_is_cut_off ───────────────────────────────
  6) [screens] › app\tests\preview.spec.ts:79:1 › the_document_title_stays_steplet ─────────────────
  7) [screens] › app\tests\preview.spec.ts:85:1 › opening_makes_no_network_request ─────────────────
    Error: expect(locator).toBeVisible() failed
    Error: element(s) not found
    Error: locator.setInputFiles: Test timeout of 30000ms exceeded.
  7 failed
```

With the new window put back, the same command: `7 passed (5.1s)`.

### Looking at the screens, and what that found

Every Open and Preview state of section 8 (4 Open states, 14 Preview states) was screenshotted in light and dark, at 1100 and 560 pixels wide (72 pictures, by a temporary test that was then deleted; Task 14 makes the lasting set), and looked at. What was wrong, and what changed:

1. **A web address in the file's text looked like a link you could click.** The file-text piece used the class name `link`, which is also Steplet's own style for its link buttons, so the file's address came out blue and underlined. Renamed (`file-link`); the address now reads as plain text with its real destination in a grey badge beside it. Test: `a_link_in_the_file_never_looks_like_a_link` (nothing inside the file's text is underlined, has a pointing cursor, or has the colour of Steplet's own links).
2. **"Steplet could not read this file" showed a general sentence instead of the reason.** The stand-in backend rejected with an error object, while Tauri rejects a failed command with Rust's sentence itself; the screen shows a sentence only. The stand-in now does what Tauri does. Test: `a_file_that_cannot_be_read_says_why`.
3. **A changed sender key began with "Signature valid, sender unknown".** That is what the core says about the key, but the warning that matters was only further down. The preview data now says when the red key-changed finding is there (`key_changed`, worked out in Rust), and the identity line then starts with "This sender name was used before with another key. See "On this computer", below." in red with its icon. Tests: Rust `a_changed_key_leads_the_identity_line` (a changed key, the same key, and an unsigned file under a known name), and screen `a_changed_key_leads_the_identity_line`.
4. **The count boxes wrapped unevenly** (five and two at 1100 wide; three, three and one at 560), because the expiry date was a seventh box. Now six even boxes (one row; three and three when narrow), the numbers lined up at the bottom, and "Expires 2026-12-31" on its own line under them.
5. **The line numbers beside steps looked like step numbers** ("10 Run the tests." for the first step). They now say "line 10", in small grey text.
6. **"From the file" was said twice,** as the section heading and again as a label inside the panel right under it. The inner label is gone; the heading, the sentence under it and the panel's coloured left edge mark the panel.

Right as it was: the outcome banners and their words and icons in both themes; the red sentence box and the disabled Run; the blocked screen with only the advice; the identity lines for not signed, new key, same key and known under another name; the imitation file (a title "✅ Verified by Steplet — safe to run") staying inside its panel in file style; the longest file (41 findings, 30 steps, 25 commands of 299 characters) with nothing cut and no sideways scroll at 560; the "On this computer" reason for a host Steplet does not fetch from; the four Open states.

The three new tests, before the fixes:

```
> cargo test -p steplet-app --offline --test open
error[E0609]: no field `key_changed` on type `PreviewView`
   --> crates\steplet-app\tests\open.rs:106:21
106 |     assert!(changed.key_changed, "the sender name is pinned to another key");
error[E0609]: no field `key_changed` on type `PreviewView`
   --> crates\steplet-app\tests\open.rs:111:19
111 |     assert!(!same.key_changed, "the same key");
error[E0609]: no field `key_changed` on type `PreviewView`
   --> crates\steplet-app\tests\open.rs:115:22
115 |     assert!(unsigned.key_changed, "unsigned under a pinned sender name (S6)");
error: could not compile `steplet-app` (test "open") due to 3 previous errors
> npm --prefix web test -- preview
  x   7 [screens] › app\tests\preview.spec.ts:86:1 › a_link_in_the_file_never_looks_like_a_link (208ms)
  x   8 [screens] › app\tests\preview.spec.ts:107:1 › a_file_that_cannot_be_read_says_why (5.2s)
  x   9 [screens] › app\tests\preview.spec.ts:113:1 › a_changed_key_leads_the_identity_line (5.1s)
    - Array []
    + Array [
    +   "SPAN.link",
    +   "SPAN.real svelte-1hd1j2t",
    +   "SPAN.real-label svelte-1hd1j2t",
    Expected: "Steplet could not read this file: Windows says access is denied."
    Received: "Steplet could not open this file."
    Expected pattern: /^This sender name was used before with another key\./
    Received string:  "Signature valid, sender unknown: this computer has not seen this key before."
  3 failed
  7 passed (19.8s)
```

The saved scenarios were then written again (`STEPLET_BLESS=1 cargo test -p steplet-app --offline --test fixtures`, 2 passed). What changed in them, counted line by line from `git diff`: 44 previews gained `"key_changed": false`, the key-changed preview gained `"key_changed": true`, the generated type `PreviewView.ts` gained the field, and nothing else.

### The real window

One more test drives the real app window (the `e2e` build, through its debugging port): it opens `examples/run-tests.steplet` through the window's own file picker and needs the preview. This is the first check of the new Rust commands through Tauri itself. To show it can fail, it ran first on a build with `allow-open-bytes` taken out of `capabilities/main.json` (then put back):

```
> cargo build -p steplet-app --offline --features e2e       (allow-open-bytes taken out)
try 1 e2e build exit: 0
> target\debug\steplet.exe started from C:\Windows\System32; npm --prefix web test -- real-open
  ok 1 [real] › app\tests\real-open.spec.ts:6:1 › the real window opens on the empty Open screen (148ms)
  x  2 [real] › app\tests\real-open.spec.ts:25:1 › the real window opens a file and shows what it asks for (5.0s)
  ok 3 [real] › app\tests\real-open.spec.ts:50:1 › the real window reaches nothing outside the app (3.1s)
    Error: expect(locator).toBeVisible() failed
  1 failed
  2 passed (12.7s)
```

The window showed Tauri's refusal (`open_bytes not allowed. Permissions associated with this command: allow-open-bytes`). With the line back:

```
> cargo build -p steplet-app --offline --features e2e
try 1 e2e build exit: 0
> target\debug\steplet.exe started from C:\Windows\System32; npm --prefix web test -- real-open
saved docs/screens/phase-4/real-open-empty-light.png
  ok 1 [real] › app\tests\real-open.spec.ts:6:1 › the real window opens on the empty Open screen (145ms)
saved docs/screens/phase-4/real-preview-light.png
  ok 2 [real] › app\tests\real-open.spec.ts:25:1 › the real window opens a file and shows what it asks for (325ms)
  ok 3 [real] › app\tests\real-open.spec.ts:50:1 › the real window reaches nothing outside the app (3.0s)
  3 passed (7.4s)
```

`docs/screens/phase-4/real-preview-light.png` is the real window's preview, looked at: it matches the screen build's. The normal app was built again afterwards (`cargo build -p steplet-app --offline`, exit 0).

### The tests pass

```
> cargo test -p steplet-app --offline --test open --test pins       (lines from the workspace run below)
test a_file_with_no_repository_cannot_run_here ... ok
test a_host_steplet_does_not_fetch_from_is_said_before_run ... ok
test an_unreadable_pins_file_is_kept_and_said ... ok
test opening_changes_nothing_on_disk ... ok
test a_file_over_256_kib_is_not_read_whole ... ok
test a_changed_key_leads_the_identity_line ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test an_empty_name_or_a_bad_key_is_never_pinned ... ok
test no_file_yet_means_no_senders_known ... ok
test a_pins_file_that_breaks_the_rules_is_unreadable ... ok
test one_pin_per_sender_name_and_a_new_key_replaces_the_old ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
> npm --prefix web run build:screens; npm --prefix web test -- preview
  ok  1 [screens] › app\tests\preview.spec.ts:12:1 › the_run_button_comes_after_the_full_list (407ms)
  ok  2 [screens] › app\tests\preview.spec.ts:23:1 › a_blocked_file_has_no_run_button (159ms)
  ok  3 [screens] › app\tests\preview.spec.ts:29:1 › a_red_file_needs_the_exact_sentence (253ms)
  ok  4 [screens] › app\tests\preview.spec.ts:41:1 › text_that_imitates_steplet_stays_in_its_panel (234ms)
  ok  5 [screens] › app\tests\preview.spec.ts:58:1 › nothing_is_cut_off (548ms)
  ok  6 [screens] › app\tests\preview.spec.ts:80:1 › the_document_title_stays_steplet (176ms)
  ok  7 [screens] › app\tests\preview.spec.ts:86:1 › a_link_in_the_file_never_looks_like_a_link (174ms)
  ok  8 [screens] › app\tests\preview.spec.ts:107:1 › a_file_that_cannot_be_read_says_why (146ms)
  ok  9 [screens] › app\tests\preview.spec.ts:113:1 › a_changed_key_leads_the_identity_line (159ms)
  ok 10 [screens] › app\tests\preview.spec.ts:119:1 › opening_makes_no_network_request (192ms)
  10 passed (5.8s)
> npm --prefix web run build; cargo test --workspace --offline --no-fail-fast
61 summary lines: 296 passed, 0 failed, 40 ignored
workspace exit: 0
> npm --prefix web run check
1792528902062 COMPLETED 141 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 64 pairs
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit: 0
```

(The workspace line is counted from its 61 summary lines; the 40 ignored are the Linux-only tests, which this task does not touch. The first try of the workspace build failed with `found invalid metadata files for crate steplet_app`, a half-written build file, the McAfee hold the global rules name; the build with `--no-run` then passed at the first try. No lock file changed, so the 14-day check was not needed.)

### What this task did not verify

- **A double-clicked file.** `initial_preview` reads the file Steplet was started with, but nothing starts Steplet with a file yet (Task 11's switch); it has not run.
- **Dropping a file on the real window.** The drop was not tried by hand or by a test; the file picker path, which uses the same `openBytes`, was tried in the real window.
- **The pins being written when Run starts** (Task 12 calls `pins::record`); `record` itself is tested.
- **Look-alike sender names** in other alphabets stay different names (Task 7); not tried on screen.
- **Windows' own Open dialog** opened by "Open a file…" in the real window: the test sets the file straight into the picker, so the dialog itself was not seen.

### The code review, and what changed

A read-only reviewer read the task's commit (60c18b0). It found nothing critical: no file text outside the display pieces, no raw HTML, nothing written to disk while opening, no network call. Its points, and what was done with each:

1. **A file that cannot run here showed the file's own colour,** even when something on this computer was worse. Example: a file with no repository, from a sender name this computer knows with a signing key, now unsigned: the banner said "Green" while "On this computer" held the red key warning. The preview data now carries `colour`, the worst of the file's findings and this computer's, worked out in Rust; the banner shows it (the screen no longer works it out). Tests: Rust `a_file_that_cannot_run_here_keeps_this_computers_colour`, screen `the_banner_shows_this_computers_warnings_too` (a new saved scenario, `preview-not-runnable-key-changed`).
2. **The changed-key line said "another key" for an unsigned file,** and the next line still said "sender unknown". Now an unsigned file under a known name says "This sender name was used before on a signed file, and this file is not signed."; a valid new key under a known name says "The signature is valid, but this computer has not seen this key before." Tests: screen `an_unsigned_file_under_a_known_name_says_so` (a new saved scenario, `preview-id-unsigned-known-name`), and `a_changed_key_leads_the_identity_line` now also requires that "sender unknown" is not there.
3. **The senders list was not flushed to disk before it replaced the old one,** so a power cut at that moment could leave an empty list, which Steplet would then refuse to touch for good. It is now written, flushed (`sync_all`), and only then renamed into place; a temporary file left by a failed write is removed. No test can make a power cut, so this change has none that failed first; the existing test that no temporary file is left still passes.
4. **Two windows recording a pin at the same moment** could lose one update. Pins are written only when Run starts, and Run takes Steplet's one-run lock (Task 3), so Task 12 must write the pin while it holds that lock. Noted for Task 12; nothing to change now.
5. **`nothing_is_cut_off` could only see text clipped by a hidden overflow,** and Steplet's file text never clips, so that half could not fail; text spilling out of its box but not past the window's edge would pass. The test now also checks that nothing is drawn outside the box it sits in (the file panel, a quote, a finding, the notes, the Run box, the banner, a count box). To show it can fail, a list's second column was made 850 pixels wide (then put back):

```
> npm --prefix web test -- preview -g nothing_is_cut_off        (the planted overflow)
  x  1 [screens] › app\tests\preview.spec.ts:58:1 › nothing_is_cut_off (257ms)
    Error: preview-longest at 1100: spills out of its box
  1 failed
```

6. **`opening_makes_no_network_request` checks the page only,** not the Rust side. True: Rust's opening code holds no network code at all, and the real window's test (`the real window reaches nothing outside the app`) covers the web view. Left as it is (deferred below).
7. **The cut to 262,145 bytes in `AppState::show` has no test of its own.** A file over that size is always blocked (`read.too-large`), so it can never be run, which makes a missed cut harmless. Deferred.

Also found by the reviewer, and fixed because the results above quote the sentence: the "cannot run here" reason held a run of ten spaces ("is not          one of those"), from a broken line join; the test `a_host_steplet_does_not_fetch_from_is_said_before_run` now requires single spaces. And while looking at the two new states: the red icon of a two-line warning sat between the lines; it now sits beside the first line.

The tests first:

```
> cargo test -p steplet-app --offline --test open
error[E0609]: no field `colour` on type `PreviewView`
   --> crates\steplet-app\tests\open.rs:131:21
error[E0609]: no field `colour` on type `PreviewView`
   --> crates\steplet-app\tests\open.rs:133:22
error: could not compile `steplet-app` (test "open") due to 2 previous errors
> STEPLET_BLESS=1 cargo test -p steplet-app --offline --test fixtures        (the two new scenarios)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
> npm --prefix web run build:screens; npm --prefix web test -- preview
  x   9 [screens] › app\tests\preview.spec.ts:126:1 › a_changed_key_leads_the_identity_line (5.1s)
  x  10 [screens] › app\tests\preview.spec.ts:133:1 › an_unsigned_file_under_a_known_name_says_so (5.2s)
  x  11 [screens] › app\tests\preview.spec.ts:141:1 › the_banner_shows_this_computers_warnings_too (5.2s)
    Expected substring: not "sender unknown"
    Received string: "This sender name was used before with another key. See \"On this computer\", below. Signature valid, sender unknown: this computer has not seen this key before. Key SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk"
    Expected pattern: /^This sender name was used before on a signed file, and this file is not signed\./
    Received string:  "This sender name was used before with another key. See \"On this computer\", below."
    Expected substring: "Red"
    Received string:    " Green Nothing in this file asks for more than it declares, and everything declared is low risk."
  3 failed
  9 passed (22.9s)
```

(The first run after the change also failed two of them on a space at the start of the warning, left by how the markup was laid out; the markup was tidied.) The saved scenarios were written again; each preview gained `colour` (checked by a script over all 49 previews: it equals the Run colour wherever the file can run, and is red for the three key-changed states, whose own outcome is green), and the host reason lost its extra spaces.

The tests pass:

```
> npm --prefix web run build:screens; npm --prefix web test -- preview
  12 passed (5.3s)
> npm --prefix web run build; cargo test -p steplet-app --offline
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
app exit: 0
> npm --prefix web run check
1792529841796 COMPLETED 141 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 64 pairs
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit: 0
```

(The app's empty test lines are left out.) Minor points left for later:
- The page shows any refusal string as it is, so in a broken build Tauri's own refusal text would reach the screen (seen above with the planted missing permission); Rust's own refusals are Steplet's sentences and never name the file.
- If a double-clicked file is still loading when another file is dropped, the two answers can arrive in either order (Run's check of the file's bytes, Task 12, refuses a mismatch).
- `steplet.exe --anything` treats the flag as a file to open (Task 11 handles the start argument).
- The "Setup" link in the top bar does nothing yet (Task 11).
- A red file that also cannot run here still shows red's line "you will type a sentence first" in its banner (Task 14 looks at every state's words).
- `opening_makes_no_network_request` checks the page only; the cut in `AppState::show` and the pins file's 1 MiB limit have no tests of their own.

## Task 11: the run chooser, setup, the key, and opening `.steplet` files

After Run on the preview, the receiver now sees the **run chooser**: Claude Code, in Steplet's own Linux, with the lines Steplet may promise for this file (each with the tests that back it, behind "The tests that show this"), what leaves the computer, and "Run with Claude Code", or "Set up first" when something is missing; Codex is listed as not available, with its reason. The **Setup** screen (from the "Setup" link at the top of every screen) has a checklist of what a run needs, each line with its state as a word and an icon; the command to type when Windows' Linux feature is missing or too old (Steplet never runs it); one button that installs Steplet's Linux, or finishes setting it up, with what setup prints; "Try again" and "Start over" (which asks first) when it fails; the key box; and the switch "Open .steplet files with Steplet". The Rust side: `src/key.rs` keeps the key in Windows' Credential Manager (entry `Steplet/anthropic-api-key`, generic, this computer only); `src/register.rs` is the switch, under `HKEY_CURRENT_USER\Software\Classes` only; `src/help.rs` opens Steplet's two help pages; `src/setup.rs` joins them to the runner's `setup::{look, install, remove}`. The real double-click was measured with your yes (Z4).

### Changes from the plan, recorded here

- **One more command, `setup_remove`,** for "Start over" (the plan's list had no command for it). It removes only Steplet's own Linux (`wsl --unregister steplet`), after the screen asks; the saved scenarios gained the screen after it (`SetupChanges.removed`).
- **The screens decide nothing on Setup either:** Rust says which command to type (`wsl_fix`: `wsl --install --no-distribution`, or `wsl --update` when too old) and whether the button offers Install, Finish or nothing (`install`). Test: `setup_says_what_to_do_next`.
- **What setup prints is cleaned of the terminal's colour codes** (`ESC [` … a letter, as systemd printed in Task 6's real install log) and a line drawn again in place shows its last drawing; every other character that shows nothing is still a badge. Test: `setup_lines_lose_terminal_codes_but_keep_other_hidden_characters`.
- **A failed setup says where it stopped in plain words** ("Setup stopped while trying to set up Steplet's Linux. Trying again is safe: Steplet picks up where it stopped. What it printed last:"), with the end of what it printed as file text. Test: `a_setup_failure_says_where_it_stopped_in_words`.
- **The missing items are named in plain words** (for example "Claude Code 2.1.220 and WSL's settings for Steplet's Linux", not `claude-code, wsl.conf`): one entry for each of the runner's checks, which became public (`steplet_run::setup::CHECKS`) so a test can hold the two lists together. Test: `every_setup_check_has_plain_words`.
- **When Steplet could not look, what went wrong is said once,** on the first line, not three times. Test: `setup_says_what_went_wrong_once` (and `setup_says_why_it_cannot_look_yet` now expects that).
- **Steplet's own sentences carry no decision codes:** Codex's reason ended in "(D1)". Test: `steplets_own_words_carry_no_decision_codes`.
- **The key:** a paste is trimmed of spaces and line breaks around it; an empty key, one with spaces or hidden characters inside, or one over 512 characters is refused in Steplet's words, which never repeat what was typed. `load` says when Windows could not read the entry, rather than calling that "no key". The runner's key type gained `is(text)`, so a test can compare a key without the key being handed out. Test: `what_is_not_a_key_is_refused_and_nothing_is_saved`.
- **The gate program sits next to Steplet's own program** (`steplet-gate.exe`), and Steplet's Linux keeps its disk in `%LOCALAPPDATA%\Steplet\wsl`.
- **The commands that wait on WSL run off the window's own thread** (Tauri's `async` commands), so the window never freezes while Steplet looks or installs. Only one install or removal runs at a time.
- **Help pages: a fixed list of two,** Microsoft's WSL page and Anthropic's key page, opened by Windows in the browser. One more feature of the `windows-sys` library is switched on for this (`Win32_UI_WindowsAndMessaging`); no download, the lock file did not change. Test: `only_steplets_own_help_pages_open`.
- **Steplet takes the first start argument that is not a flag** as the file to open (a double-click gives the file's path).
- **Six screen tests, not the plan's five:** `a_runner_that_is_not_set_up_sends_you_to_setup` was added. And one more real-window test: the real Setup screen looks at this laptop.

### The tests first

```
> cargo test -p steplet-app --offline --test key --test register
error[E0432]: unresolved import `steplet_app::register`
 --> crates\steplet-app\tests\register.rs:5:5
error[E0432]: unresolved import `steplet_app::key`
 --> crates\steplet-app\tests\key.rs:5:5
error[E0432]: unresolved import `steplet_app::setup`
 --> crates\steplet-app\tests\key.rs:6:5
error: could not compile `steplet-app` (test "key") due to 2 previous errors
error: could not compile `steplet-app` (test "register") due to 1 previous error
> npm --prefix web run build:screens; npm --prefix web test -- chooser setup
  x  1 [screens] › app\tests\chooser.spec.ts:20:1 › codex_is_shown_unavailable_with_its_reason (5.8s)
  x  2 [screens] › app\tests\setup.spec.ts:17:1 › wsl_missing_shows_the_command_to_type (5.8s)
  x  4 [screens] › app\tests\setup.spec.ts:25:1 › install_progress_then_failure_offers_try_again_and_start_over (5.5s)
  x  3 [screens] › app\tests\chooser.spec.ts:28:1 › only_the_lines_promises_returns_are_shown (5.7s)
  x  5 [screens] › app\tests\setup.spec.ts:42:1 › the_key_box_never_shows_a_saved_key (5.4s)
  x  6 [screens] › app\tests\chooser.spec.ts:46:1 › a_runner_that_is_not_set_up_sends_you_to_setup (5.3s)
    Error: expect(locator).toBeVisible() failed
  6 failed
```

The words tests, each before its change:

```
> cargo test -p steplet-app --offline --test views
error[E0432]: unresolved imports `steplet_app::views::setup_failure`, `steplet_app::views::setup_line`
error[E0425]: cannot find value `CHECK_WORDS` in module `steplet_app::views`
> cargo test -p steplet-app --offline --test views        (setup_says_what_to_do_next added)
error[E0432]: unresolved import `steplet_app::views::Offer`
error[E0609]: no field `wsl_fix` on type `SetupView`
error[E0609]: no field `install` on type `SetupView`
> cargo test -p steplet-app --offline --test views        (the last two added, after looking at the screens)
test setup_says_what_went_wrong_once ... FAILED
test steplets_own_words_carry_no_decision_codes ... FAILED
  left: (Some([Text { text: "WSL did not answer within 60 seconds" }]), Some([Text { text: "WSL did not answer within 60 seconds" }]))
 right: (None, None)
  left: ["D1"]
 right: []
test result: FAILED. 12 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

### Looking at the screens, and what that found

Every Chooser state (ready with no sites, ready with 15 sites, needs setup, WSL missing) and every Setup state (nothing done, WSL missing, WSL too old, installing, install failed, all ready, key saved, switch broken, could not look) was screenshotted in light and dark, at 1100 and 560 wide (52 pictures, by a temporary test, then deleted; Task 14 makes the lasting set), and looked at. What was wrong, and what changed:

1. **Each checklist line said its subject twice** (a title "Steplet's own Linux" above "Steplet's own Linux is not installed yet."). The titles are gone; Rust's sentence says it all.
2. **The install log squeezed runs of spaces,** so a progress bar lost its shape. Each line is now drawn as a block that keeps its spaces.
3. **"Check again" sat on a second row of buttons** under "Try again" and "Start over". It is now beside the panel's heading.
4. **When Steplet could not look, the same error was printed three times,** and **Codex's reason ended in "(D1)"** (both fixed in Rust, tests above).
5. **"Finish setting up" carried Install's note** ("downloads Ubuntu 26.04 … about 1 to 2 GB"), which is untrue once Steplet's Linux is there. It now says it adds what is missing and is safe to repeat.

Right as it was: the chooser's lines, ticks and the 15 sites as file-text chips; "Set up first"; the Codex panel; the WSL-missing box with the command and Microsoft's page; the installing state; the failure box with git's own words as file text; the key box ("Saved", empty password box, "Forget key"); the broken-switch warning; the narrow layout (each state chip above its sentence).

### The tests pass

```
> cargo test -p steplet-app --offline --no-fail-fast -- --skip file_type_switch_round_trip
     Running tests\fixtures.rs (target\debug\deps\fixtures-69f45a4a78dd73f9.exe)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
     Running tests\help.rs (target\debug\deps\help-8c6bdabcdb2c6b36.exe)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\key.rs (target\debug\deps\key-7998c8650d45633c.exe)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests\open.rs (target\debug\deps\open-1ff61e9983282f36.exe)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests\pins.rs (target\debug\deps\pins-ba69d7f5516c4ba8.exe)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests\register.rs (target\debug\deps\register-b7fd4615a3017ff0.exe)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
     Running tests\shell.rs (target\debug\deps\shell-52d52ba15b88d342.exe)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\views.rs (target\debug\deps\views-5ef4c53d6ec120ac.exe)
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
app exit: 0
> cmdkey /list        (after the key tests: no made-up entry left)
entries listed: 9
no Steplet entries
> npm --prefix web run build:screens; npm --prefix web test -- chooser setup preview
  18 passed (6.7s)
> cargo test -p steplet-run --offline
19 summary lines: 114 passed, 0 failed, 40 ignored
run exit: 0
> npm --prefix web run check
1792531284958 COMPLETED 146 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 66 pairs
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -j 4 -- -D warnings
clippy exit: 0
```

(The app's empty test lines are left out. The first build of this task failed with "The paging file is too small for this operation to complete" and a failed memory allocation: this laptop had about 55 of its 63 GB of memory spoken for (WSL, the browser, other windows), and cargo builds many test programs at once. Every build since uses four jobs at a time, `-j 4`, and passed.)

**The registry test, with your yes (Z4, 21 Oct 2026):**

```
> cargo test -p steplet-app --offline --test register
test file_type_switch_round_trip ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
> reg query HKCU\Software\Classes\.steplet-test ; reg query HKCU\Software\Classes\Steplet.TestFile ; reg query HKCU\Software\Classes\Steplet.OtherTestFile
ERROR: The system was unable to find the specified registry key or value.
ERROR: The system was unable to find the specified registry key or value.
ERROR: The system was unable to find the specified registry key or value.
```

### The real window

The real Setup screen, in the `e2e` build, looked at this laptop for real (it changes nothing; it may start Steplet's Linux to look inside):

```
> target\debug\steplet.exe started from C:\Windows\System32; npm --prefix web test -- real-open
saved docs/screens/phase-4/real-open-empty-light.png
  ok 1 [real] › app\tests\real-open.spec.ts:6:1 › the real window opens on the empty Open screen (163ms)
saved docs/screens/phase-4/real-preview-light.png
  ok 2 [real] › app\tests\real-open.spec.ts:25:1 › the real window opens a file and shows what it asks for (256ms)
saved docs/screens/phase-4/real-setup-light.png
setup line: Ready |  | Windows' Linux feature (WSL 2.6.3.0).
setup line: Ready |  | Steplet's own Linux is installed.
setup line: Ready |  | Its tools are there: git, Python, Claude Code and the rest.
setup line: Missing |  | The gate program for this Steplet is not in its Linux yet.
  ok 3 [real] › app\tests\real-open.spec.ts:50:1 › the real window's Setup looks at this computer (2.6s)
  ok 4 [real] › app\tests\real-open.spec.ts:73:1 › the real window reaches nothing outside the app (3.1s)
  4 passed (10.2s)
```

The gate line is right: this build's gate program (`target\debug\steplet-gate.exe`) has never been put on Steplet's Linux disk (the runner's tests use their own release build), so the screen offers "Finish setting up Steplet's Linux". Task 15 does that.

### Step 5: the real double-click, measured

Before the switch, nothing in Windows knew `.steplet` files:

```
HKCU .steplet:
  not there
HKCU Steplet.File:
  not there
Explorer's own choice for .steplet:
  nothing recorded
HKLM .steplet:
  not there
```

With the normal build running, you pressed Setup and ticked "Open .steplet files with Steplet". Then Windows' own "open" path, the one a double-click takes:

```
HKCU .steplet (default): Steplet.File
HKCU Steplet.File (default): Steplet task file
open command: "C:\dev\runFile\.claude\worktrees\steplet-phase4-build-88af2b\target\debug\steplet.exe" "%1"
Steplet processes before: 1
> Invoke-Item examples\run-tests.steplet
Steplet processes after Invoke-Item: 2
  pid 36168, started 03:12:19, window: ''
  pid 50932, started 03:14:38, window: 'Steplet'
OpenWith dialogs open: 0
```

**Steplet opened the file straight away, in a window of its own titled "Steplet", and Windows did not ask which app to use.** The new window showed the file's preview (`docs/screens/phase-4/real-double-click-dark.png`, captured from the screen and cropped to Steplet's window; Windows was in dark mode by then). In that capture the page was scrolled down by about a hundred pixels, so the colour banner was half hidden. It did not happen again: started with the file as Windows starts it, and again through `Invoke-Item` (the e2e build, read through its debugging port), the page stood at the top each time:

```
0 {"scrollY":0,"innerH":780,"docH":1625,"active":"BODY.","anchor":"auto"}
500 {"scrollY":0,"innerH":780,"docH":1625,"active":"BODY.","anchor":"auto"}
2000 {"scrollY":0,"innerH":780,"docH":1625,"active":"BODY.","anchor":"auto"}
```

(the same three lines both times). The cause of the one scrolled window is not known; a scroll of the mouse over the new window would explain it. Task 14 looks again. The switch is still on, pointing at this work folder's build; turn it off in Setup when you no longer want it.

### What this task did not verify

- **A real install from the Setup screen.** The install and "Start over" buttons were tried only against the stand-in backend; the runner's `install` itself was measured in Task 6 (Z3).
- **The real key box.** Saving a key through the real window was not tried (no key is saved now); `key.rs` was tested on made-up entries.
- **The help links in the real window:** `ShellExecuteW` was not called in a test (it would open your browser).
- **The scrolled window seen once after the double-click** (above).
- **A second Windows user, and a Windows where another app already owns `.steplet` files** (Explorer's own choice would win over Steplet's entry).

### The code review, and what changed

A read-only reviewer read the task's commit (e9829e4). It found nothing critical: the key never reaches the screens or an error, the unsafe Windows calls free and close what they open, and the registry test touches only its made-up type. Its points, and what was done with each:

1. **The "Open .steplet files with Steplet" box could show the wrong state.** The browser ticks the box on a click; if Windows then refused the change, the box stayed ticked while the switch was off. The box is now put back to how the switch really stands. Test: `a_switch_that_could_not_change_shows_how_it_stands` (a new saved scenario, `setup-switch-fails`, where Windows refuses).
2. **A crash inside an install would have blocked every later install** until Steplet was restarted ("already changing its Linux"), because the mark was cleared only after the work returned. The mark is now cleared however the work ends (`setup::OneAtATime`). Tests: `a_crashed_install_does_not_block_the_next_one`, `a_second_install_while_one_runs_is_refused`.
3. **Leaving Setup during an install lost it:** coming back showed "Install" again, without the log, and pressing it was refused. An install's progress now lives as long as the window (`web/app/lib/install.svelte.ts`), so coming back shows the same install, still going, with what it printed. Test: `leaving_setup_during_an_install_keeps_it_going`.
4. **Turning the switch on took the file type from another program without noting it, and turning it off then deleted that program's entry.** Steplet now notes which entry had the file type and gives it back when the switch goes off; with no such entry it removes only its own value, and the file type's key only once nothing else is in it (the reviewer's minor point 5, a part of the same fix). Test: `turning_off_gives_the_file_type_back_to_the_program_that_had_it`, on the made-up type `.steplet-test` like the round-trip test (the registry test you said yes to, now with this one more step; everything is removed at the end).

To show the new tests catch the real faults, the two Rust ones were also run against the old code (then the new code was put back):

```
> cargo test -p steplet-app --offline --test register        (the old switch code, its helper made public)
test file_type_switch_round_trip ... ok
test turning_off_gives_the_file_type_back_to_the_program_that_had_it ... FAILED
  left: None
 right: Some("SomeOther.TestFile")
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
> cargo test -p steplet-app --offline --test setup           (the mark cleared only after the work returns)
test a_crashed_install_does_not_block_the_next_one ... FAILED
test a_second_install_while_one_runs_is_refused ... ok
  left: Err(FailureView { reason: "Steplet is already changing its Linux. Wait for that to finish.", detail: None })
 right: Ok(7)
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> npm --prefix web test -- setup -g a_switch_that              (before the box fix)
  x  1 [screens] › app\tests\setup.spec.ts:55:1 › a_switch_that_could_not_change_shows_how_it_stands (5.4s)
    Error: expect(locator).not.toBeChecked() failed
    Expected: not checked
    Received: checked
  1 failed
> npm --prefix web test -- setup                                (before the install store)
  x  5 [screens] › app\tests\setup.spec.ts:65:1 › leaving_setup_during_an_install_keeps_it_going (5.3s)
    Error: expect(locator).toBeVisible() failed
```

The tests pass:

```
> npm --prefix web run build:screens; npm --prefix web test -- chooser setup preview
  20 passed (6.2s)
> npm --prefix web run build; cargo test -p steplet-app --offline --no-fail-fast
     Running tests\fixtures.rs 	test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
     Running tests\help.rs 	test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\key.rs 	test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests\open.rs 	test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests\pins.rs 	test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests\register.rs 	test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests\setup.rs 	test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\shell.rs 	test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\views.rs 	test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
app exit: 0
> reg query HKCU\Software\Classes\.steplet-test
ERROR: The system was unable to find the specified registry key or value.
> npm --prefix web run check
1792533568555 COMPLETED 147 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 66 pairs
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -j 4 -- -D warnings
clippy exit: 0
```

(The app's empty test lines are left out.) Minor points left for later:
- When Steplet's Linux is there but cannot be looked at, Setup offers only "Check again"; "Start over" shows only after a failed install (Task 14 looks at that state again).
- The key's text is wiped from one copy in memory, not from every copy (the one the page sent, and the one read back); a Credential Manager read that fails for another reason than "no entry" shows as "no key saved".
- The words for Claude Code and srt name their versions as fixed text instead of the runner's version constants, so they could drift.
- An unfinished terminal code (`ESC [` with no letter after) drops the rest of its line.

## Task 12: prepare, the run with its live log, and the question screen

After "Run with Claude Code", the receiver now sees the **run screen**. A bar stays at the top while the log scrolls. It shows where the run is (Preparing, Running, Waiting for your answer, Stopping…, or how it ended), the time against the run's 30-minute limit, and **Stop**, which asks first. Under it:

- **Getting the repository ready:** fetched from where, the commit checked out inside the fence, and which agent settings files were removed (Steplet tells the agent not to load them).
- **The live log:** what the agent says (quoted, as file text), and each command or file action with Steplet's check ("Allowed by Steplet's checker", "You allowed this exact request", "Refused: you said no", "Ran without asking Steplet: …"), its exit code, and the first 40 lines of what it printed, with "Show all N lines". The log follows the newest row unless the receiver scrolls up; "Show the newest" brings it back.

When the agent wants something the file did not ask for, the **question screen** covers the window and the taskbar flashes. It shows:

- what exactly is asked, with the whole request, nothing cut;
- why Steplet asks, and what the agent said just before;
- what still holds on a yes;
- "No answer in 4:59 means no".

**No** has the focus. **"Yes, allow this exact request for this run"** works only once the question has been on screen for a second. Questions come one at a time ("2 more questions are waiting"). Closing the window during a run asks first ("Stop and close" or "Keep running").

On the Rust side:

- `src/session.rs` runs a file. It checks the file again at Run, writes the crash marker (`%LOCALAPPDATA%\Steplet\run-in-progress.json`), starts `steplet_run::run_live` on its own thread, passes each piece of progress to the window through a `Relay` (one question on screen at a time), and takes the answers.
- At start, a marker left by a crash means that run is stopped and removed, and the Open screen says so.
- The window gained the commands `start_run`, `answer`, `stop_run` and `stop_and_close`, and asks before closing during a run.

### Changes from the plan, recorded here

- **The run function is a shared value (`Arc<dyn Fn …>`), not a plain function pointer,** so the tests can give a session a stand-in run and the live tests can point the real one at the scripted model.
- **Questions wait their turn in Rust, not on the page.** The `Relay` shows one question and holds the rest; the page gets "N more are waiting" (`ProgressView::Waiting`). Test: `a_question_flashes_the_taskbar_and_waits_its_turn`.
- **Each gate decision reaches the screen as it is made** (`Progress::Checked`, new in the runner), so a log row says "Allowed" or "Refused" before the report exists. Test: `each_decision_reaches_the_screen_as_its_check`. A call that never asked the gate gets its check the same way. Test: `a_call_that_skipped_the_gate_reaches_the_screen_with_its_check`.
- **A refusal after asking you has its own label.** Before, both the live log and the report called your "no" "Refused by Steplet's checker", with the runner's internal note "the receiver said no" as the reason. Now it is `Check::RefusedWhenAsked` (`NotYes::SaidNo`, `NoAnswer` or `TakenBack`): "Refused: you said no", "Refused: no answer in time", "Refused: Steplet took the question back as the run stopped". Tests: `each_decision_reaches_the_screen_as_its_check` and `answers_and_allowed_by_you_show_in_the_report` (runner), `a_refusal_after_asking_says_how_it_ended` (app), and the Linux test `no_answer_refuses_it`, whose expectation changed with it.
- **A command that ran without asking and was no harmless read has its own kind** (`ran_without_asking`), so the screen marks it red, not grey like a harmless read. Test: `a_call_that_ran_without_asking_is_not_marked_as_a_harmless_read`.
- **The red sentence comes from Rust** (`PreviewView.red_sentence`), and **the run's time limit comes with the first progress** (`Prepared.limit_ms`), so the page holds no copy of either.
- **The run's start checks the same colour the preview showed.** This was found by an automated security review during this task. Before, a run's start looked only at the file's own colour, so a known sender with another key (red on this computer, row 22) could be started without the sentence by a broken page. Now `views::run_status` is the one place both use. Test: `a_known_sender_with_another_key_needs_the_sentence_too`; it failed against the old start, which was planted back for the check and then removed.
- **The one-run lock is taken only for a moment at start,** to write the sender's pin, then let go; `run_live` takes it itself for the run.
- **If the run's own thread crashes, the marker stays,** so the next start of Steplet clears what is left. The look at start that does this cannot leave the Open screen's note waiting, even if it crashes.
- **A wrong key, measured first** (below). The runner now reads Claude Code's "trying again" lines (`Event::ApiRetry`) and the error status at the end (`Event::ApiError`). The report says "Anthropic did not accept the key. Change it in Setup." The live log shows one row that counts the tries, so the three minutes Claude Code spends trying again are not a silent screen. Tests: `a_key_anthropic_refuses_is_read_from_the_stream` (two of the lines exactly as Claude Code wrote them), `a_key_anthropic_refuses_is_a_fault_in_words`, `each_try_after_a_service_error_is_said_in_steplets_words`, `a_wrong_key_shows_each_try_and_says_so`, and the Linux test `a_wrong_key_says_so`. The scripted model now answers a key that is not the expected one with HTTP 401 and Anthropic's own error body.
- **Two more saved scenarios:** a question with a 346-character command (`ask-long-command`), and a run whose 40 commands keep coming for a few seconds (`run-40-commands`), for the "follows the end" test. The run scenarios now pass through the app's own `Relay` and carry the chooser.
- **Eight screen tests, not the plan's six:** `closing_the_window_during_a_run_asks_first` and `a_wrong_key_shows_each_try_and_says_so` were added.
- **The plan's test command `npm --prefix web test -- run` matches every test file,** because the folder path holds "runFile". The commands below use `run.spec`.

### The wrong key, measured

The scripted model answered every request with HTTP 401 and Anthropic's body for a wrong key (`authentication_error`, "invalid x-api-key"). Claude Code 2.1.220 tried 10 times over 186 seconds, its pauses doubling up to about 39 seconds. It wrote each try on its stream, then its own message, then a result carrying the status. Four of its 13 lines (the run's `app-1792536988-3160.jsonl`):

```
6:{"type":"system","subtype":"api_retry","attempt":5,"max_retries":10,"retry_delay_ms":8418,"error_status":401,"error":"authentication_failed","session_id":"31060bcf-4ad5-4a8e-b398-266bb4476edf","uuid":"51a8c2fd-8ad0-4e8e-a887-1a08aeee527b"}
11:{"type":"system","subtype":"api_retry","attempt":10,"max_retries":10,"retry_delay_ms":38974,"error_status":401,"error":"authentication_failed","session_id":"31060bcf-4ad5-4a8e-b398-266bb4476edf","uuid":"7036c6cf-5ba7-43f5-9436-8c2e119b20f9"}
12:{"type":"assistant","message":{"id":"ca03080b-bdd3-4e6b-942c-dcba5b5e08bb","container":null,"model":"<synthetic>","role":"assistant", … ,"content":[{"type":"text","text":"Invalid API key · Fix external API key"}],"context_management":null},"parent_tool_use_id":null,"session_id":"31060bcf-4ad5-4a8e-b398-266bb4476edf","uuid":"c65769d3-64b0-401f-bf42-d5b1a566aa24","timestamp":"2026-10-20T22:59:39.534Z","error":"authentication_failed","is_api_error_message":true}
13:{"is_error":true,"duration_api_ms":0,"num_turns":1,"stop_reason":"stop_sequence", … ,"terminal_reason":"api_error","fast_mode_state":"off","fast_mode_disabled_reason":"sdk_opt_in_required","subtype":"success","api_error_status":401,"result":"Invalid API key · Fix external API key","type":"result","duration_ms":186462,"uuid":"2b15c859-a6ae-4d0b-add2-f7a938802e42"}
```

Where "…" stands, the usage numbers are left out of this page; the stream test holds lines 6 and 13 whole. When each request reached the scripted model, in seconds after the first (read while the run was still going, after 9 of them):

```
[0.0, 0.8, 2.1, 4.7, 9.8, 18.4, 37.6, 76.9, 110.8]
```

Before the change, what the screen got ended like this, with no fault (the long lines shortened where "…" stands):

```
--- {"kind":"event","event":{"kind":"agent_says","text":[{"kind":"text","text":"Invalid API key · Fix external API key"}]}}
--- {"kind":"event","event":{"kind":"ended","models":[]}}
--- {"kind":"finished","report":{"end":{"kind":"did_not_finish"},"completed":false, … ,"faults":[], … }}

thread 'a_wrong_key_says_so' (28904) panicked at crates\steplet-app\tests\session_live.rs:161:5:
[]
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 211.75s
```

The model's log also shows Claude Code sends the key both as `x-api-key` and as a bearer token (`"bearer_sent": true`); both reach only the address Steplet's settings give it.

### The tests first

The session's own tests, and the runner's new progress, before the code existed:

```
> cargo test -p steplet-app --offline --test session
error[E0432]: unresolved import `steplet_app::session`
  --> crates\steplet-app\tests\session.rs:13:18
error[E0432]: unresolved import `steplet_app::views::RED_SENTENCE`
  --> crates\steplet-app\tests\session.rs:14:40
error: could not compile `steplet-app` (test "session") due to 2 previous errors
> cargo test -p steplet-run --offline --test ask
error[E0599]: no variant named `Checked` found for enum `Progress`
   --> crates\steplet-run\tests\ask.rs:299:27
error: could not compile `steplet-run` (test "ask") due to 1 previous error
```

The screen tests, before the run screen existed (`npm --prefix web run build:screens`, then the run screen's tests; the four "real" window tests in the same run failed only because no e2e window was open):

```
  x   2 [screens] › app\tests\run.spec.ts:32:1 › ask_starts_on_no_and_yes_waits_one_second (6.0s)
  x  26 [screens] › app\tests\run.spec.ts:52:1 › ask_shows_the_whole_request (5.4s)
  x  27 [screens] › app\tests\run.spec.ts:67:1 › a_withdrawn_question_closes_with_a_note (5.3s)
  x  28 [screens] › app\tests\run.spec.ts:79:1 › the_log_follows_the_end_unless_you_scrolled_up (5.2s)
  x  29 [screens] › app\tests\run.spec.ts:98:1 › stop_asks_first_and_then_says_interrupted (5.3s)
  x  30 [screens] › app\tests\run.spec.ts:116:1 › a_50000_line_output_keeps_the_log_responsive (5.3s)
  x  31 [screens] › app\tests\run.spec.ts:138:1 › closing_the_window_during_a_run_asks_first (5.5s)
```

Each change after that, before its code:

```
> cargo test -p steplet-app --offline --test session a_call_that
test a_call_that_skipped_the_gate_reaches_the_screen_with_its_check ... FAILED
  left: []
 right: [("toolu_s", "not_asked", "Ran without asking Steplet: a harmless read")]
> cargo test -p steplet-run --offline --test ask --test report        (the label for a refusal after asking)
error[E0432]: unresolved import `steplet_run::NotYes`
error[E0599]: no variant named `RefusedWhenAsked` found for enum `Check`
error: could not compile `steplet-run` (test "ask") due to 3 previous errors
error: could not compile `steplet-run` (test "report") due to 3 previous errors
> cargo build -p steplet-app --offline --tests                       (its words, a_refusal_after_asking_says_how_it_ended)
error[E0004]: non-exhaustive patterns: `&steplet_run::Check::RefusedWhenAsked { .. }` not covered
> cargo test -p steplet-app --offline --test views a_call_that_ran
test a_call_that_ran_without_asking_is_not_marked_as_a_harmless_read ... FAILED
  left: "not_asked"
 right: "ran_without_asking"
> cargo test -p steplet-run --offline --test stream                  (the wrong key, read from the stream)
error[E0599]: no variant named `ApiRetry` found for enum `steplet_run::Event`
error[E0599]: no variant named `ApiError` found for enum `steplet_run::Event`
> cargo test -p steplet-run --offline --test report a_key
error[E0599]: no variant named `ApiRetry` found for enum `steplet_run::Event`
error[E0599]: no variant named `ApiError` found for enum `steplet_run::Event`
> cargo test -p steplet-app --offline --test views each_try
error[E0004]: non-exhaustive patterns: `&steplet_run::Event::ApiRetry { .. }` and `&steplet_run::Event::ApiError { .. }` not covered
> npx playwright test --project=screens run.spec -g wrong_key        (before the screen drew the tries)
  x  1 [screens] › app\tests\run.spec.ts:151:1 › a_wrong_key_shows_each_try_and_says_so (5.5s)
    Expected: 10
    Received: 0
> (the same, once each try was a row: changed to one row that counts the tries, run against that build)
    Expected: 1
    Received: 10
```

Two tests were also run against a fault planted back on purpose, to show they catch it (each time the right code was put back and the file compared):

```
> (the log made to jump to the end always) npx playwright test --project=screens run.spec -g follows
  x  1 [screens] › app\tests\run.spec.ts:79:1 › the_log_follows_the_end_unless_you_scrolled_up (7.9s)
    Error: expect(received).toBe(expected) // Object.is equality
    Expected: false
    Received: true
> (a run's start blind to this computer's findings, as before) cargo test -p steplet-app --offline --test session a_known_sender
test a_known_sender_with_another_key_needs_the_sentence_too ... FAILED
  left: Ok(())
 right: Err("This file is red: type the sentence exactly before you run it.")
```

### Looking at the screens, and what that found

Every run and question state was screenshotted, in light and dark, at 1100 and 560 wide, both at the newest row and scrolled to the top (by a temporary test, then deleted; Task 14 makes the lasting set). The states were: fetching; ready with files removed; fetch failed; busy; long command; 50,000-line output; tripwire; model refusal; wrong key; a command question; a 200-line file question; a web address question; 2 more waiting. What was wrong, and what changed:

1. **The empty top strip during a run:** the header with "Open another file" and "Setup" is hidden while a file runs, but its space stayed. It is gone now, and the run's bar sits at the very top.
2. **The bar's line ran wider than the column.** It now lines up with the panels.
3. **A command you said no to read "Refused by Steplet's checker" and "the receiver said no",** and **"Ended with an error"** for a call that never ran. The label is now "Refused: you said no" (Rust, tests above), and a refused call shows no exit.
4. **A command that ran without asking got a broken badge:** a style name used twice, so the badge took the look of a "Claude Code printed" block. Renamed; and it is red now (its own kind, test above).
5. **A failed fetch said "Nothing ran" twice** and kept the half-done "Fetching…" steps on screen. The failure's heading is now its stage ("Fetching the repository"), and the steps are not shown when preparing failed.
6. **The bar and the panel both said "Getting the repository ready"** while fetching. The bar says "Preparing".
7. **On the 200-line file question, the reason and "what still holds" were below the fold.** Each long field now scrolls in its own box, so they stay in view; all 200 lines are still there.
8. **The model refusal read "Its reason: cyber."** It now reads "The kind of request it refused: cyber."
9. **Ten rows of "tries again"** for a wrong key: now one row that counts (test above).

Right as it was: the question's layout with No first and in colour, Yes greyed for its first second, the countdown, "2 more questions are waiting", the web address as a link chip with "address"; the 40-line output with "Show all 50,000 lines"; "Show the newest" while scrolled up; the end panels (Interrupted, Steplet stopped the run, The model refused, Did not finish) with their colours, words and icons; the narrow layout.

### The tests pass

The whole workspace (the two registry tests left out: they change Windows' settings, and ran with your yes in Task 11):

```
> npm --prefix web run build; cargo test --workspace --offline -j 2 --no-fail-fast -- --skip file_type_switch_round_trip --skip turning_off_gives_the_file_type_back
Running tests\fixtures.rs test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\help.rs test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\key.rs test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\open.rs test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\pins.rs test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\register.rs test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out;
Running tests\session.rs test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\session_live.rs test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out;
Running tests\setup.rs test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\shell.rs test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\views.rs test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
   (the core: 24 test files, every one ok)
Running tests\gate.rs test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;           (steplet-gate)
Running unittests src\lib.rs test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;   (the runner)
Running tests\ask.rs test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\ask_live.rs test result: ok. 0 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out;
Running tests\contain.rs test result: ok. 0 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out;
Running tests\decide.rs test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\gate.rs test result: ok. 15 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out;
Running tests\launch.rs test result: ok. 1 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out;
Running tests\lock.rs test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\prepare.rs test result: ok. 3 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out;
Running tests\promises.rs test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\report.rs test result: ok. 22 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out;
Running tests\settings.rs test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\setup.rs test result: ok. 3 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out;
Running tests\stream.rs test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\task.rs test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\tripwires.rs test result: ok. 0 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out;
Running tests\watch.rs test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
Running tests\watch_run.rs test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
exit 0
```

(Each line is cargo's "Running" line joined to its result line; the times and the empty lines of the programs with no tests are left out.)

The screens:

```
> npm --prefix web run build:screens; npx playwright test --project=screens
  ok  3 [screens] › app\tests\run.spec.ts:32:1 › ask_starts_on_no_and_yes_waits_one_second (3.5s)
  ok 16 [screens] › app\tests\run.spec.ts:52:1 › ask_shows_the_whole_request (847ms)
  ok 19 [screens] › app\tests\run.spec.ts:67:1 › a_withdrawn_question_closes_with_a_note (1.9s)
  ok 24 [screens] › app\tests\run.spec.ts:79:1 › the_log_follows_the_end_unless_you_scrolled_up (6.1s)
  ok 25 [screens] › app\tests\run.spec.ts:99:1 › stop_asks_first_and_then_says_interrupted (555ms)
  ok 26 [screens] › app\tests\run.spec.ts:117:1 › a_50000_line_output_keeps_the_log_responsive (677ms)
  ok 27 [screens] › app\tests\run.spec.ts:139:1 › closing_the_window_during_a_run_asks_first (428ms)
  ok 28 [screens] › app\tests\run.spec.ts:151:1 › a_wrong_key_shows_each_try_and_says_so (324ms)
  28 passed (17.5s)
> npm --prefix web run check
1792538237445 COMPLETED 153 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 70 pairs
> cargo fmt --all -- --check
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -j 4 -- -D warnings
clippy exit: 0
```

(The other 20 screen tests are the Open, Preview, Chooser and Setup ones of Tasks 10 and 11. These screen runs used a Playwright setting that reuses a preview server already running on the test port: a run whose output was cut short by a pipe left its server behind, and stopping that program was not allowed from here. It serves this work folder's own screens build, checked by its page's script name matching the build on disk. The setting is a scratch file, not committed.)

The live session tests, on Steplet's Linux with the scripted model:

```
> cargo test -p steplet-app --offline -j 2 --test session_live -- --ignored --nocapture --test-threads=1
test a_session_runs_use_case_1_with_the_scripted_model_and_asks_once ... ok
test a_wrong_key_says_so ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 230.72s
```

Picked out of what the two runs sent to the screen (their full output is printed by the tests): the first run's one question closed with `"how":"You said yes to this exact request."` and its report has `"completed":true`; in the second, 10 `"kind":"retry"` rows came, and its report has `"completed":false` and `"faults":[[{"kind":"text","text":"Anthropic did not accept the key. Change it in Setup."}]]`.

### What this task did not verify

- **The real window during a run.** `start_run`, `answer`, `stop_run`, `stop_and_close`, the `run-progress` events, the taskbar's flash and the close question were tested through the session with a stand-in screen, and through the screens with the stand-in backend, but not in the real window. That needs Steplet's Linux finished for this build (its gate program) and a key in Credential Manager: Task 15.
- **A run with the real model.** The live tests use the scripted model; Task 15 runs the real one, with your yes (Z5).
- **Stop in a real Linux run started from a session.** The runner's Stop was measured in Task 3; here Stop was tested with a stand-in run only.
- **A real crash's clean-up.** `clear_leftover` was tested with a planted marker and a stand-in for `discard`; the real `discard` was measured in Task 3.
- **Escape on the question screen and the Stop question** does nothing (on purpose: Escape never answers). No test presses it.
- **A wrong key still takes Claude Code about three minutes** of tries before the run ends. Steplet shows each try, but does not stop the run at the first 401. That would save the wait; it is noted for the review.
