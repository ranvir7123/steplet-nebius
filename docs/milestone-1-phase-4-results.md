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
