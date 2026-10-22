# Milestone 1, phase 5: results

The plan: `docs/plans/2026-10-22-milestone-1-phase-5.md`, approved on 22 Oct 2026 with the recommended option for every choice (K1 A to K6 A; the smaller choices kept). **What this phase is:** defensive testing of our own app. Steplet is an open-source Windows app that runs other people's task files inside a fenced-off Linux; this phase builds a regression test suite (tests that are run again after every change) proving that Steplet's own fence and its permission gate block every known kind of misuse. Every test runs only on this laptop, inside Steplet's own Linux or on Windows, against Steplet's own code. Test content is harmless: marker files holding made-up words, the made-up key `MADE_UP_KEY` (it works nowhere), and the milestone 0 test fixture already in the repository. Each case is described by what its test checks and what passing looks like.

Every command below was run on this laptop (Windows 11 Home, Rust 1.98.0), and its output is pasted unchanged. Where only part of a long output is shown, the text says which part; the lines shown are not edited. `echo "exit: $?"` after a command prints its exit code (0 means it succeeded).

## Task 1: the catalogue and the one command

Session "Steplet phase 5 Task 1: the attack suite's list and command", branch `claude/steplet-phase5-task1-5653f2`, cut from the approved plan's commit 34c965c. **Done: all 27 existing cases blocked on Steplet's Linux, and the suite passes** (code ac5f618, 89f89c7 and f68e62b; this file and the records fdb8295 and the Step 6 commit). Steps 1 to 4 ran before Steplet's Linux was free, with nothing run on it; Step 5 ran once you said it was free (Z9). No step started `steplet.exe` or made an end-to-end build. No AI safety check stopped any step of this task, and the session's model was not changed.

### What was built

- `crates/steplet-suite` (test-only, never shipped: `publish = false`, and nothing in the app depends on it):
  - `src/catalogue.rs`: the 27 cases whose tests already exist (section 8 of the plan, marked "exists"), each with the attack in plain words, the lines on screen it backs, its tests, its layer and what it needs (Windows only, or Steplet's Linux and the network).
  - `src/verdict.rs`: reads Rust's own test output and judges each case: blocked only when every one of its tests said `ok`; "did not run" (never blocked) when a test could not start its case, was set aside, gave no result, or its test program did not run to its end.
  - `src/runner.rs`: checks this computer first (Steplet's one-run lock free, Steplet's Linux set up, Claude Code 2.1.220) for any case that needs Steplet's Linux; builds the test programs once (with the McAfee retries); runs exactly the tests each case names, one at a time, with a short fixed list of Windows variables; runs a test that several cases name only once; after each case on Steplet's Linux, holds the one-run lock while it looks for anything left behind, removes it, looks again, and fails the suite.
  - `src/record.rs`: keeps the whole output as `docs/suite/<date>-claude-<version>-<layer>.txt`, never over an earlier record, with the made-up key's text counted and hidden before anything is written.
  - `src/main.rs`: `cargo suite list | scripted | live [--only <ids>]`; `live` says it comes with Task 8 and runs nothing.
  - Its own tests: `tests/catalogue.rs`, `tests/verdict.rs`, `tests/record.rs` (none needs Steplet's Linux).
- `.cargo/config.toml`: `[alias] suite = "run -p steplet-suite --offline --"`.
- `crates/steplet-run/tests/support/mod.rs` gains `did_not_run`, `hold_linux`, `after_case_look`, `Marker` and `marker`.
- `docs/suite/.gitkeep`, the first record (below), and this file.

### Changes from the plan, recorded here

- **No `Line::UseCase5` yet.** It comes with Task 7, together with its cases U1 and U2. Use case 5 is the milestone's done-means 3, not one of the milestone plan's lines on screen, and no existing test fetches it from GitHub; pointing it at the local fixture test would claim more than that test shows. With the line there and no case for it, `every_line_on_screen_has_a_scripted_case` could not pass.
- **`ready_to_run` takes the Claude Code version the suite reads itself:** `ready_to_run(look, claude_found, lock_free)`. Steplet's own look (`setup::look`) only says "claude-code missing" when the version differs, without naming the version found, so the stop could not say "Claude Code 2.1.231 is installed" from it.
- **`NotReady` has no `GateMissing`, and `LinuxMissing` says why.** Every Linux test puts this worktree's own gate program on Steplet's Linux disk itself (`support::install_gate`), so a gate missing there is no reason to stop; and the stop must say in words what is missing.
- **The runner's view of this computer (`Machine`) takes and lets go of the lock** (`take_lock`, `let_go`) instead of only asking whether it is free: the look after a case must hold it (see the code review below). The lock is never held while a case's tests run, since the tests that start runs take it themselves.
- **W1 also names `a_yes_allows_exactly_the_request_shown`.** The run chooser's fetch line names that test, and `the_chooser_and_the_suite_name_the_same_tests` needs every test the chooser names to be in a case of the same line.
- **Some cases back two lines.** The chooser puts some tests under two lines (the web-fetch test and the write-into-Steplet's-folder test also back "Steplet checks…"; the temporary-folder test also backs "…change files only inside the working copy"); section 8 lists each under one line only.
- **`src/lib.rs`, and a third test file `tests/record.rs`.** Tests in `tests/` can only reach a library, and the record writer needed tests of its own.
- **More tests than the plan named:** `what_a_case_needs_follows_its_tests` (a case needs Steplet's Linux exactly when one of its tests is set aside for it, so the runner never skips the check before such a case), `a_test_name_means_one_test`, `only_picks_cases_by_id_or_by_their_letter`, `the_reader_reads_rust_s_own_test_output`, `nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up`, `a_test_named_by_two_cases_runs_once`, `the_look_after_a_case_touches_nothing_while_another_run_holds_the_lock`, `the_clean_up_removes_only_what_the_look_named_below_its_places`, `test_programs_get_only_the_windows_variables_they_need`, and the three record tests.
- **The reader reads both shapes of Rust's test output.** Section 3.3 expected one `test <name> ... ok` line per test, but with `--nocapture` (which the runner uses, so a long case shows its lines as they come) a test's own lines sit between its name and its result. The reader handles both, and a test checks it against what this Rust really prints.
- **The suite starts each test program with a short fixed list of Windows variables** (where Windows, your folders, cargo and Rust are), not everything of whoever started it. Found on the first real run: a gate test writes the names (never the values) of every variable it was given into its log, so that record carried this session's variable names, among them the name of a GitHub token variable. That record was deleted before any commit, and the run was made again (below).
- **A record name already taken the same day gets `-2`, `-3`; a run of some cases says so in its name** (`…-scripted-only-G.txt`). Otherwise Task 2's `cargo suite scripted --only G` would write over that day's full record.
- **A key's text found in the output** is replaced by `[hidden by Steplet]` in the kept record, its count is written at the record's end, and the suite does not pass. The plan said the record is counted before it is kept, but not what happens when the count is not 0.
- **Two more helper files are kept on purpose in `/opt/steplet-test` (`count_key.py`, `m0-fixture`), and the suite looks once before the first case**, so what earlier work left there is named apart and blamed on no case (found by Step 5's first replay; see Step 5).
- **The web page was built before clippy** (`npm --prefix web ci --offline`, then `npm --prefix web run build`), as phase 4 recorded: `steplet-app` carries its page inside the program, and a fresh worktree has none.

### Steps 1 and 2: the tests first, and seeing them fail

The tests were written before any of the suite's code, with only an empty `src/lib.rs` and the crate's `Cargo.toml` in place.

```
> cargo test -p steplet-suite --offline; echo "exit: $?"
(the last 51 lines of 95; the 44 before them are cargo compiling the packages the suite uses)
   Compiling steplet-suite v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase5-task1-5653f2\crates\steplet-suite)
error[E0432]: unresolved import `steplet_suite::catalogue`
 --> crates\steplet-suite\tests\record.rs:6:20
  |
6 | use steplet_suite::catalogue::Layer;
  |                    ^^^^^^^^^ could not find `catalogue` in `steplet_suite`

error[E0432]: unresolved import `steplet_suite::record`
 --> crates\steplet-suite\tests\record.rs:7:20
  |
7 | use steplet_suite::record::{Day, keep, record_name};
  |                    ^^^^^^ could not find `record` in `steplet_suite`

error[E0432]: unresolved import `steplet_suite::catalogue`
 --> crates\steplet-suite\tests\catalogue.rs:9:20
  |
9 | use steplet_suite::catalogue::{CASES, Case, Layer, Line, Needs, TestRef, cases_for, select};
  |                    ^^^^^^^^^ could not find `catalogue` in `steplet_suite`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `steplet-suite` (test "record") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
error[E0432]: unresolved import `steplet_suite::catalogue`
  --> crates\steplet-suite\tests\verdict.rs:14:20
   |
14 | use steplet_suite::catalogue::{Case, Layer, Line, Needs, TestRef};
   |                    ^^^^^^^^^ could not find `catalogue` in `steplet_suite`

error[E0432]: unresolved import `steplet_suite::runner`
  --> crates\steplet-suite\tests\verdict.rs:15:20
   |
15 | use steplet_suite::runner::{CaseResult, Machine, NotReady, leftovers, output_of, passed, ready_to_run, run_cases};
   |                    ^^^^^^ could not find `runner` in `steplet_suite`

error[E0432]: unresolved import `steplet_suite::verdict`
  --> crates\steplet-suite\tests\verdict.rs:16:20
   |
16 | use steplet_suite::verdict::{TestOutcome, Verdict, line_backed, read_results, verdict};
   |                    ^^^^^^^ could not find `verdict` in `steplet_suite`

error[E0425]: cannot find function `did_not_run` in module `support`
   --> crates\steplet-suite\tests\verdict.rs:184:18
    |
184 |         support::did_not_run("the network was not reachable");
    |                  ^^^^^^^^^^^ not found in `support`

error: could not compile `steplet-suite` (test "catalogue") due to 1 previous error
Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `steplet-suite` (test "verdict") due to 4 previous errors
exit: 101
```

As the plan expected: it fails because the crate's modules (`catalogue`, `record`, `runner`, `verdict`) and the support helper `did_not_run` do not exist.

The lock file gained only the new crate:

```
> git diff 34c965c ac5f618 -- Cargo.lock
diff --git a/Cargo.lock b/Cargo.lock
index 662b426..1b9f132 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -3181,6 +3181,16 @@ dependencies = [
  "steplet-core",
 ]
 
+[[package]]
+name = "steplet-suite"
+version = "0.1.0"
+dependencies = [
+ "sha2",
+ "steplet-core",
+ "steplet-run",
+ "windows-sys 0.61.2",
+]
+
 [[package]]
 name = "steplet-wasm"
 version = "0.1.0"
```

### Step 3: the code

Written as listed above. Two tests added during this step were each seen failing first, against an empty stand-in, before their code was written. The clean-up's safety test:

```
> cargo test -p steplet-suite --offline; echo "exit: $?"
(from the verdict tests on; the catalogue and record tests before them all said ok)
     Running tests\verdict.rs (target\debug\deps\verdict-ba926aa185fd982f.exe)

running 13 tests
test sample_that_did_not_run ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_did_not_try ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_fails ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_passes ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test blocked_only_when_every_test_of_the_case_says_ok ... ok
test a_case_that_did_not_run_is_never_shown_as_blocked ... ok
test a_missing_or_ignored_test_is_not_blocked ... ok
test a_test_named_by_two_cases_runs_once ... ok
test leftovers_after_a_case_are_named_and_fail_the_suite ... ok
test the_suite_refuses_another_claude_code_version ... ok
test the_suite_starts_nothing_while_a_run_holds_the_lock ... ok
test the_clean_up_removes_only_what_the_look_named_below_its_places ... FAILED
test the_reader_reads_rust_s_own_test_output ... ok

failures:

---- the_clean_up_removes_only_what_the_look_named_below_its_places stdout ----

thread 'the_clean_up_removes_only_what_the_look_named_below_its_places' (29872) panicked at crates\steplet-suite\tests\verdict.rs:300:5:
assertion `left == right` failed
  left: ""
 right: "rm -rf -- '/work/t11b'\nrm -rf -- '/opt/steplet-test/cmd.exe'\nkill -KILL 812\n"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    the_clean_up_removes_only_what_the_look_named_below_its_places

test result: FAILED. 8 passed; 1 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.10s

error: test failed, to rerun pass `-p steplet-suite --test verdict`
exit: 101
```

And the short list of Windows variables:

```
> cargo test -p steplet-suite --offline --test verdict test_programs_get_only_the_windows_variables_they_need; echo "exit: $?"
     Running tests\verdict.rs (target\debug\deps\verdict-ba926aa185fd982f.exe)

running 1 test
test test_programs_get_only_the_windows_variables_they_need ... FAILED

failures:

---- test_programs_get_only_the_windows_variables_they_need stdout ----

thread 'test_programs_get_only_the_windows_variables_they_need' (50836) panicked at crates\steplet-suite\tests\verdict.rs:348:5:
assertion `left == right` failed: Windows reads variable names without regard to case
  left: []
 right: [("Path", "C:\\made-up"), ("SystemRoot", "C:\\Windows"), ("LOCALAPPDATA", "C:\\made-up\\Local"), ("RUSTUP_TOOLCHAIN", "1.98.0"), ("cargo_home", "C:\\made-up\\.cargo")]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_programs_get_only_the_windows_variables_they_need

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p steplet-suite --test verdict`
exit: 101
```

### The code review, and what changed

A reviewer with a fresh view (an AI code-review helper that had not seen the work) read the finished Task 1 code. What it found, graded by what you would get, and what was done:

- **Must fix: the look after a case ran without the one-run lock.** Between two cases a Steplet window could start a run; the look would then take that run's working copy for leftovers, remove it, and end its programs. Now the look, the removal and a second look all happen while the suite holds the lock, and when someone else holds it the suite looks at and removes nothing and does not pass (`the_look_after_a_case_touches_nothing_while_another_run_holds_the_lock`).
- **A test's own line reading `ok`, then a crash, could be read as passed.** Now nothing from a test program counts unless it printed Rust's last line (`test result: …`) once and the results read add up to its counts; while a test runs, a line it prints that looks like a test's line is its own output (`nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up`).
- **The clean-up was never checked.** Now the look runs again after a removal and names anything still there.
- **The record writer** wrote the text to disk before hiding the key, and could replace a record made at the same moment. Now the key is counted and hidden in memory before anything is written, and the record is created only if no file of that name exists. (No test makes a write fail half way; this is fixed by how the code is built.)
- **One name, one test:** results are read by test name, so `a_test_name_means_one_test` guards that no two tests share a name (true today).
- **Smaller:** the made-up key is counted before the "Suite:" line is written, so the record can never say PASSED and then NOT PASSED; `cargo suite live --only G` now says the live layer is not built yet instead of "no case has the id"; the record test uses the real `MADE_UP_KEY`.
- **Not changed (small, noted for later):** the clean-up's places include `/home/` and `/var/lib/steplet/`; a marker's word found in an unrelated file there would get that file removed. This is only inside Steplet's own Linux.

The updated tests, run against the code before these fixes, could not even be built, because they use the lock's new `take_lock` and `let_go`:

```
> cargo test -p steplet-suite --offline; echo "exit: $?"
(the error lines and the exit code only)
error[E0407]: method `take_lock` is not a member of trait `Machine`
error[E0407]: method `let_go` is not a member of trait `Machine`
error[E0046]: not all trait items implemented, missing: `lock_free`
error: could not compile `steplet-suite` (test "verdict") due to 3 previous errors
exit: 101
```

### Step 4: the tests pass

McAfee held a freshly built test program several times (`LNK1104: cannot open file '…\verdict-ba926aa185fd982f.exe'`); each time the build was made with `--no-run` first, which worked on its first or second try, and then the tests were run:

```
> cargo test -p steplet-suite --offline; echo "exit: $?"
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Running unittests src\lib.rs (target\debug\deps\steplet_suite-ca4bf3c18e827b16.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\main.rs (target\debug\deps\steplet_suite-310c39508ecb5c8c.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\catalogue.rs (target\debug\deps\catalogue-10e83d3e2aea4425.exe)

running 7 tests
test every_line_on_screen_has_a_scripted_case ... ok
test a_test_name_means_one_test ... ok
test only_picks_cases_by_id_or_by_their_letter ... ok
test ids_are_unique_and_follow_their_line ... ok
test the_chooser_and_the_suite_name_the_same_tests ... ok
test every_case_names_tests_that_exist ... ok
test what_a_case_needs_follows_its_tests ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests\record.rs (target\debug\deps\record-41bb88e1c96ea496.exe)

running 3 tests
test a_record_is_named_by_day_version_and_layer ... ok
test a_key_in_a_record_is_counted_and_hidden_before_it_is_kept ... ok
test a_second_record_the_same_day_does_not_replace_the_first ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\verdict.rs (target\debug\deps\verdict-ba926aa185fd982f.exe)

running 16 tests
test sample_that_did_not_run ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_did_not_try ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_fails ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_passes ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test a_missing_or_ignored_test_is_not_blocked ... ok
test a_case_that_did_not_run_is_never_shown_as_blocked ... ok
test nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up ... ok
test leftovers_after_a_case_are_named_and_fail_the_suite ... ok
test a_test_named_by_two_cases_runs_once ... ok
test test_programs_get_only_the_windows_variables_they_need ... ok
test the_clean_up_removes_only_what_the_look_named_below_its_places ... ok
test the_look_after_a_case_touches_nothing_while_another_run_holds_the_lock ... ok
test the_suite_refuses_another_claude_code_version ... ok
test the_suite_starts_nothing_while_a_run_holds_the_lock ... ok
test blocked_only_when_every_test_of_the_case_says_ok ... ok
test the_reader_reads_rust_s_own_test_output ... ok

test result: ok. 12 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.08s

   Doc-tests steplet_suite

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit: 0
```

`0 failed`, as the plan expected: 22 passed. The four `sample_…` tests are set aside on purpose: they are the samples `the_reader_reads_rust_s_own_test_output` runs, and they do something only when that test runs them.

**These checks can fail.** Three mistakes planted in copies (a test name changed in W1; G5 marked as needing Steplet's Linux; the reader's check of Rust's counts switched off), both test files run, and the files put back:

```
> diff catalogue.rs.keep crates/steplet-suite/src/catalogue.rs; diff verdict.rs.keep crates/steplet-suite/src/verdict.rs
197c197
<         needs: WINDOWS,
---
>         needs: LINUX_AND_NETWORK,
256c256
<             run("ask", "a_yes_allows_exactly_the_request_shown"),
---
>             run("ask", "a_yes_allows_exactly_the_request_shown_renamed"),
80c80
<     if counted != [Some(read)] {
---
>     if false && counted != [Some(read)] {
> cargo test -p steplet-suite --offline --no-fail-fast --test catalogue --test verdict
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
     Running tests\catalogue.rs (target\debug\deps\catalogue-10e83d3e2aea4425.exe)

running 7 tests
test a_test_name_means_one_test ... ok
test every_line_on_screen_has_a_scripted_case ... ok
test ids_are_unique_and_follow_their_line ... ok
test only_picks_cases_by_id_or_by_their_letter ... ok
test the_chooser_and_the_suite_name_the_same_tests ... FAILED
test what_a_case_needs_follows_its_tests ... FAILED
test every_case_names_tests_that_exist ... FAILED

failures:

---- the_chooser_and_the_suite_name_the_same_tests stdout ----

thread 'the_chooser_and_the_suite_name_the_same_tests' (28640) panicked at crates\steplet-suite\tests\catalogue.rs:90:17:
the chooser's Fetch line names a_yes_allows_exactly_the_request_shown, which no case of that line replays
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- what_a_case_needs_follows_its_tests stdout ----

thread 'what_a_case_needs_follows_its_tests' (40736) panicked at crates\steplet-suite\tests\catalogue.rs:121:9:
assertion `left == right` failed: G5: []
  left: true
 right: false

---- every_case_names_tests_that_exist stdout ----

thread 'every_case_names_tests_that_exist' (3900) panicked at crates\steplet-suite\tests\catalogue.rs:58:13:
W1 names steplet-run::ask::a_yes_allows_exactly_the_request_shown_renamed, which is not a test there


failures:
    every_case_names_tests_that_exist
    the_chooser_and_the_suite_name_the_same_tests
    what_a_case_needs_follows_its_tests

test result: FAILED. 4 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p steplet-suite --test catalogue`
     Running tests\verdict.rs (target\debug\deps\verdict-ba926aa185fd982f.exe)

running 16 tests
test sample_that_did_not_run ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_did_not_try ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_fails ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_passes ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test blocked_only_when_every_test_of_the_case_says_ok ... ok
test a_case_that_did_not_run_is_never_shown_as_blocked ... ok
test a_missing_or_ignored_test_is_not_blocked ... ok
test a_test_named_by_two_cases_runs_once ... ok
test leftovers_after_a_case_are_named_and_fail_the_suite ... ok
test the_clean_up_removes_only_what_the_look_named_below_its_places ... ok
test nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up ... FAILED
test the_look_after_a_case_touches_nothing_while_another_run_holds_the_lock ... ok
test the_suite_refuses_another_claude_code_version ... ok
test the_suite_starts_nothing_while_a_run_holds_the_lock ... ok
test test_programs_get_only_the_windows_variables_they_need ... ok
test the_reader_reads_rust_s_own_test_output ... ok

failures:

---- nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up stdout ----

thread 'nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up' (47316) panicked at crates\steplet-suite\tests\verdict.rs:205:5:
assertion `left == right` failed
  left: {"first_check": Ok}
 right: {}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up

test result: FAILED. 11 passed; 1 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.11s

error: test failed, to rerun pass `-p steplet-suite --test verdict`
error: 2 targets failed:
    `-p steplet-suite --test catalogue`
    `-p steplet-suite --test verdict`
> git diff --no-index --quiet (each copy against its file): both files put back exactly
```

Each planted mistake is caught, and the files were put back exactly.

**What the reader reads, as this Rust really prints it** (the four samples, run with `--nocapture` and with `--show-output`):

```
> cargo test -p steplet-suite --offline --test verdict the_reader_reads_rust_s_own_test_output -- --nocapture; echo "exit: $?"
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running tests\verdict.rs (target\debug\deps\verdict-ba926aa185fd982f.exe)

running 1 test
--- --nocapture:

running 4 tests
test sample_that_did_not_run ... suite: did not run: the network was not reachable

thread 'sample_that_did_not_run' (50076) panicked at crates\steplet-suite\tests\..\..\steplet-run\tests\support\mod.rs:163:5:
did not run: the network was not reachable
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test sample_that_did_not_try ... suite: did not try: the model refused
ok
test sample_that_fails ... 
thread 'sample_that_fails' (40212) panicked at crates\steplet-suite\tests\verdict.rs:240:9:
a made-up failure
FAILED
test sample_that_passes ... an ordinary line of output
ok

failures:

failures:
    sample_that_did_not_run
    sample_that_fails

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s


--- --show-output:

running 4 tests
test sample_that_did_not_run ... FAILED
test sample_that_did_not_try ... ok
test sample_that_fails ... FAILED
test sample_that_passes ... ok

successes:

---- sample_that_did_not_try stdout ----
suite: did not try: the model refused

---- sample_that_passes stdout ----
an ordinary line of output


successes:
    sample_that_did_not_try
    sample_that_passes

failures:

---- sample_that_did_not_run stdout ----
suite: did not run: the network was not reachable

thread 'sample_that_did_not_run' (38652) panicked at crates\steplet-suite\tests\..\..\steplet-run\tests\support\mod.rs:163:5:
did not run: the network was not reachable
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- sample_that_fails stdout ----

thread 'sample_that_fails' (49296) panicked at crates\steplet-suite\tests\verdict.rs:240:9:
a made-up failure


failures:
    sample_that_did_not_run
    sample_that_fails

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s


test the_reader_reads_rust_s_own_test_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.08s

exit: 0
```

**The list:**

```
> cargo suite list; echo "exit: $?"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running `target\debug\steplet-suite.exe list`
The attack suite: 27 cases (27 scripted, 0 live). This list runs nothing.

G1   The agent runs a command the file declares, and others it does not                 scripted, needs Steplet's Linux and the network   backs "Steplet checks…"
G2   The gate program stops in the middle of a request                                  scripted, needs Steplet's Linux and the network   backs "Steplet checks…"
G3   Steplet's decision never comes                                                     scripted, needs Steplet's Linux and the network   backs "Steplet checks…"
G4   The gate gets an answer of the wrong shape, a closed channel, or no answer         scripted, needs Windows only                      backs "Steplet checks…"
G5   Something says nothing, or nonsense, on the gate channel                           scripted, needs Windows only                      backs "Steplet checks…"
G6   A request on the gate channel carries the wrong run token                          scripted, needs Windows only                      backs "Steplet checks…"
G7   Another program takes the gate channel's name first                                scripted, needs Windows only                      backs "Steplet checks…"
G8   A program other than the gate connects to the channel                              scripted, needs Windows only                      backs "Steplet checks…"
G9   A restricted or low-integrity program connects to the channel                      scripted, needs Windows only                      backs "Steplet checks…"
G11  A command is made to skip Steplet's checker                                        scripted, needs Steplet's Linux and the network   backs "Steplet checks…"
G12  Requests keep coming while Steplet stops the run                                   scripted, needs Windows only                      backs "Steplet checks…"
W1   The agent fetches a web page the file did not declare                              scripted, needs Steplet's Linux and the network   backs "The agent can fetch only…", "Steplet checks…"
F1   The test fixture writes outside its working copy                                   scripted, needs Steplet's Linux and the network   backs "…change files only inside the working copy"
F2   The agent writes into Steplet's own folder                                         scripted, needs Steplet's Linux and the network   backs "…change files only inside the working copy", "Steplet checks…"
Y1   Use case 1 runs, with none of your own files present                               scripted, needs Steplet's Linux and the network   backs "Your own files are not inside…"
Y2   The run's temporary folder holds the key or the run's token, or outlives the run   scripted, needs Steplet's Linux and the network   backs "Your own files are not inside…", "…change files only inside the working copy"
Y4   The test fixture reads a marker in Steplet's run folder                            scripted, needs Steplet's Linux and the network   backs "Your own files are not inside…"
N1   The test fixture connects to the internet and to a listener in Steplet's Linux     scripted, needs Steplet's Linux and the network   backs "…cannot connect…"
N2   A program reaches a site other than the one the file declares                      scripted, needs Steplet's Linux and the network   backs "…cannot connect…"
P1   The test fixture starts a Windows program                                          scripted, needs Steplet's Linux and the network   backs "…cannot start Windows programs"
S1   The repository carries agent settings files                                        scripted, needs Steplet's Linux and the network   backs "…not to load agent settings"
S2   A hook sits in the run's settings                                                  scripted, needs Steplet's Linux and the network   backs "…not to load agent settings"
S3   The run's folder ends up recorded as trusted                                       scripted, needs Steplet's Linux and the network   backs "…not to load agent settings"
I1   A run is stopped, runs out of time, or leaves programs running                     scripted, needs Steplet's Linux and the network   backs "Interrupted…"
M1   A run uses several models, and a model refuses                                     scripted, needs Windows only                      backs "every model used, and refusals"
R1   A program prints look-alike lines and control characters                           scripted, needs Steplet's Linux and the network   backs "…safe to read"
R2   The key or the run's token reaches the raw logs                                    scripted, needs Windows only                      backs "…safe to read"

The lines on screen, and the cases that back them:
  Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen.
      G1 G2 G3 G4 G5 G6 G7 G8 G9 G11 G12 W1 F2
  The agent can fetch only the web addresses the file declares, and another one only if you say yes to that exact address.
      W1
  The agent and the programs this task runs can change files only inside the working copy.
      F1 F2 Y2
  Your own files are not inside Steplet's Linux, where this task runs.
      Y1 Y2 Y4
  Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.
      N1 N2
  Programs this task runs cannot start Windows programs.
      P1
  Prepare: Steplet tells the agent not to load agent settings.
      S1 S2 S3
  Interrupted; the folder was reset; run again?
      I1
  Report: every model used, and refusals.
      M1
  The report and the live log are safe to read.
      R1 R2

Every line on screen has a scripted case.
exit: 0
```

One line per case, and every line on screen of the milestone plan's section 7 is there with at least one scripted case (the Codex line is not a suite case: it stays until a phase 7 trial passes).

**The live layer is not there yet, and says so:**

```
> cargo suite live --only G; echo "exit: $?"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running `target\debug\steplet-suite.exe live --only G`
The live layer is not built yet: it comes with phase 5, Task 8. Nothing was run.
error: process didn't exit successfully: `target\debug\steplet-suite.exe live --only G` (exit code: 2)
exit: 2
```

**The runner, for real, on the cases that need Windows only.** These nine cases (the gate channel's checks, the report's model list and its raw-log hiding) touch neither Steplet's Linux nor the network, and none of their tests is set aside, so they could run now. This shows the whole path working before Step 5: the nested `cargo` builds and runs, the reader reads real output, the lines on screen are judged, and the record is kept. It was run on the committed code:

```
> cargo suite scripted --only G4,G5,G6,G7,G8,G9,G12,M1,R2; echo "exit: $?"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.33s
     Running `target\debug\steplet-suite.exe scripted --only G4,G5,G6,G7,G8,G9,G12,M1,R2`
The attack suite, scripted layer, on Claude Code 2.1.220: 9 cases (only G4,G5,G6,G7,G8,G9,G12,M1,R2).
Started 2026-10-22 at 03:33, on the code at commit ac5f618.
These cases need Windows only: Steplet's Linux is not used.
G4   The gate gets an answer of the wrong shape, a closed channel, or no answer .... blocked   (3.5 s)
G5   Something says nothing, or nonsense, on the gate channel ...................... blocked   (6.7 s)
G6   A request on the gate channel carries the wrong run token ..................... blocked   (0.3 s)
G7   Another program takes the gate channel's name first ........................... blocked   (0.3 s)
G8   A program other than the gate connects to the channel ......................... blocked   (0.3 s)
G9   A restricted or low-integrity program connects to the channel ................. blocked   (0.3 s)
G12  Requests keep coming while Steplet stops the run .............................. blocked   (0.3 s)
M1   A run uses several models, and a model refuses ................................ blocked   (0.3 s)
R2   The key or the run's token reaches the raw logs ............................... blocked   (0.3 s)

The lines on screen, on Claude Code 2.1.220:
  Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen.
      backed (7 of 7 cases blocked)
  The agent can fetch only the web addresses the file declares, and another one only if you say yes to that exact address.
      not run this time
  The agent and the programs this task runs can change files only inside the working copy.
      not run this time
  Your own files are not inside Steplet's Linux, where this task runs.
      not run this time
  Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.
      not run this time
  Programs this task runs cannot start Windows programs.
      not run this time
  Prepare: Steplet tells the agent not to load agent settings.
      not run this time
  Interrupted; the folder was reset; run again?
      not run this time
  Report: every model used, and refusals.
      backed (1 of 1 cases blocked)
  The report and the live log are safe to read.
      backed (1 of 1 cases blocked)
Suite: PASSED for the 9 cases run (only G4,G5,G6,G7,G8,G9,G12,M1,R2): each was blocked and left nothing behind. It backs only the lines marked backed above. Took 0 min 13 s.
Record: docs/suite/2026-10-22-claude-2.1.220-scripted-only-G4-G5-G6-G7-G8-G9-G12-M1-R2.txt (counted before it was kept: the made-up key's text 0 times).
exit: 0
```

The record is committed with this file. Its gate log now names only Windows' and cargo's own variables.

**Formatting, clippy, and the 14-day rule:**

```
> cargo fmt --all -- --check; echo "exit: $?"
exit: 0
```

```
> cargo clippy --workspace --all-targets --offline -j 2 -- -D warnings; echo "exit: $?"
    Checking steplet-suite v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase5-task1-5653f2\crates\steplet-suite)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.55s
exit: 0
```

Only `steplet-suite` is checked again in this last run; the rest of the workspace was checked by the same command earlier in this task and has not changed since. The first clippy run had stopped on `steplet-app` with "The `frontendDist` configuration is set to `"../../web/dist"` but this path doesn't exist" (a fresh worktree has no built page); after the page build below, it passed:

```
> npm --prefix web ci --offline; echo "exit: $?"; npm --prefix web run build; echo "exit: $?"

added 53 packages, and audited 54 packages in 2s

13 packages are looking for funding
  run `npm fund` for details

found 0 vulnerabilities
exit: 0

> build
> vite build --mode app

[36mvite v8.3.1 [32mbuilding client environment for app...[36m[39m
transforming...
✓ 142 modules transformed.
rendering chunks...
computing gzip size...
dist/index.html                   0.45 kB │ gzip:  0.29 kB
dist/assets/index-BJFQ5Kn6.css   29.67 kB │ gzip:  4.82 kB
dist/assets/index-C7X2rW84.js   102.24 kB │ gzip: 33.47 kB

[32m✓ built in 409ms[39m
exit: 0
```

```
> python -I C:\Users\RANVIR\.claude\handoffs\2026-10-11-1606-steplet-tools\lock_ages.py Cargo.lock
(the first 3 lines and the last 3 lines of 445)
adler2             2.0.1                  published 2025-06-09   489 days old
aho-corasick       1.1.5                  published 2026-08-03    69 days old
alloc-no-stdlib    3.0.0                  published 2026-06-14   119 days old
…
zlib-rs            0.6.8                  published 2026-09-15    26 days old
zmij               1.0.23                 published 2026-07-13    90 days old
444 crates.io packages in the lock file; 17 under 14 days old; 0 yanked
```

The script counts ages from 11 Oct 2026, a date written inside it, so its last line flags the 18 packages that were under 14 days old on that day:

```
> grep "under 14\|YANKED" lock-ages.log
cc                 1.6.0                  published 2026-10-03     8 days old  <-- under 14 days
embed-resource     3.0.12                 published 2026-10-06     5 days old  <-- under 14 days
hyper              1.12.0                 published 2026-10-06     5 days old  <-- under 14 days
jiff               0.2.38                 published 2026-10-06     5 days old  <-- under 14 days
jiff-static        0.2.38                 published 2026-10-06     5 days old  <-- under 14 days
jiff-tzdb          0.1.9                  published 2026-10-06     5 days old  <-- under 14 days
objc2              0.6.5                  published 2026-10-05     6 days old  <-- under 14 days
powerfmt           0.2.1                  published 2026-10-05     6 days old  <-- under 14 days
tauri              2.12.1                 published 2026-09-30    11 days old  <-- under 14 days
tauri-build        2.7.1                  published 2026-09-30    11 days old  <-- under 14 days
tauri-codegen      2.7.1                  published 2026-09-30    11 days old  <-- under 14 days
tauri-macros       2.7.1                  published 2026-09-30    11 days old  <-- under 14 days
tauri-runtime      2.12.1                 published 2026-09-30    11 days old  <-- under 14 days
tauri-runtime-wry  2.12.1                 published 2026-09-30    11 days old  <-- under 14 days
tauri-utils        2.10.1                 published 2026-09-30    11 days old  <-- under 14 days
uuid               1.27.0                 published 2026-10-02     9 days old  <-- under 14 days
want               0.3.2                  published 2026-10-05     6 days old  <-- under 14 days
444 crates.io packages in the lock file; 17 under 14 days old; 0 yanked
```

Counted again from today, 22 Oct 2026, from the same publish dates (a short Python count over that output): 444 packages, **0 under 14 days old**, 0 withdrawn. The youngest were published on 2026-10-06, 16 days ago: `embed-resource 3.0.12`, `hyper 1.12.0`, `jiff 0.2.38`, `jiff-static 0.2.38`, `jiff-tzdb 0.1.9`. The only change to the lock file in this task is the new crate itself, which comes from this repository.

```
> rustc --version; cargo --version
rustc 1.98.0 (88d9e12ae 2026-08-18)
cargo 1.98.0 (797e8a9bc 2026-08-05)
```

### The automatic security check, and what changed (after commit fdb8295)

An automatic security review of the committed code found a real problem in the look after a case. The look listed what a case left as plain lines of text, and the clean-up acted on those lines as root inside Steplet's Linux. A program the task runs works as the `steplet` account and can name a file in `/tmp` with a line break inside it; that one name would then read as two lines, and the second could pose as a program to end or a path to remove. The clean-up also removed deeper paths and files under `/home`, where a link on the way could lead out. Nothing of this had run yet (the look first runs in Step 5), but the suite exists to replay untrusted samples, so it was fixed at once:

- The look now ends each record with a NUL, the one character a file name can never hold (`find -printf '…\0'`, `grep -Z`, `printf '…\0'`), and reads Steplet's raw output (`stdout_bytes`), since the usual text drops NULs.
- A record whose path holds a control character, or does not start at `/`, is named as "could not read" (its control characters shown as `?`) and never acted on.
- Programs to end come only from their own records, by number, and are ended first; then only paths directly inside one of the folders the look lists are removed. Nothing deeper, nothing under `/home`, never Steplet's runs folder. This also settles the small point the code review had left for later.

The tests first, against the code before the fix (the verdict tests, with the three that fail):

```
> cargo test -p steplet-suite --offline --test verdict; echo "exit: $?"
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running tests\verdict.rs (target\debug\deps\verdict-ba926aa185fd982f.exe)

running 17 tests
test sample_that_did_not_run ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_did_not_try ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_fails ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_passes ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test blocked_only_when_every_test_of_the_case_says_ok ... ok
test a_case_that_did_not_run_is_never_shown_as_blocked ... ok
test a_missing_or_ignored_test_is_not_blocked ... ok
test a_test_named_by_two_cases_runs_once ... ok
test a_name_with_a_line_break_cannot_add_a_record_of_its_own ... FAILED
test nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up ... ok
test test_programs_get_only_the_windows_variables_they_need ... ok
test leftovers_after_a_case_are_named_and_fail_the_suite ... FAILED
test the_suite_refuses_another_claude_code_version ... ok
test the_clean_up_removes_only_what_the_look_named_below_its_places ... FAILED
test the_look_after_a_case_touches_nothing_while_another_run_holds_the_lock ... ok
test the_suite_starts_nothing_while_a_run_holds_the_lock ... ok
test the_reader_reads_rust_s_own_test_output ... ok

failures:

---- a_name_with_a_line_break_cannot_add_a_record_of_its_own stdout ----

thread 'a_name_with_a_line_break_cannot_add_a_record_of_its_own' (49552) panicked at crates\steplet-suite\tests\verdict.rs:425:5:
assertion `left == right` failed
  left: ["could not read: left\t/tmp/steplet-a", "could not read: program\t1\tx\0marker\t/tmp/steplet-b", "/etc\0left\t/work/t11b\0"]
 right: ["could not read: left?/tmp/steplet-a?program?1?x", "could not read: marker?/tmp/steplet-b?left /etc", "/work/t11b"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- leftovers_after_a_case_are_named_and_fail_the_suite stdout ----

thread 'leftovers_after_a_case_are_named_and_fail_the_suite' (37964) panicked at crates\steplet-suite\tests\verdict.rs:324:5:
assertion `left == right` failed
  left: ["could not read: test-file\t/opt/steplet-test/listener.py\0test-file\t/opt/steplet-test/cmd.exe\0test-file\t/opt/steplet-test/t11b.listener.log\0left\t/work/t11b\0"]
 right: ["/opt/steplet-test/cmd.exe", "/work/t11b"]

---- the_clean_up_removes_only_what_the_look_named_below_its_places stdout ----

thread 'the_clean_up_removes_only_what_the_look_named_below_its_places' (35304) panicked at crates\steplet-suite\tests\verdict.rs:411:5:
assertion `left == right` failed
  left: "rm -rf -- '/work/t11b'\nrm -rf -- '/opt/steplet-test/cmd.exe'\nkill -KILL 812\nrm -rf -- '/var/lib/steplet/t11b.stamp'\nrm -rf -- '/tmp/steplet-dir/inner/marker.txt'\nrm -rf -- '/home/steplet/notes.txt'\n"
 right: "kill -KILL 812\nrm -rf -- '/work/t11b'\nrm -rf -- '/opt/steplet-test/cmd.exe'\nrm -rf -- '/var/lib/steplet/t11b.stamp'\n"


failures:
    a_name_with_a_line_break_cannot_add_a_record_of_its_own
    leftovers_after_a_case_are_named_and_fail_the_suite
    the_clean_up_removes_only_what_the_look_named_below_its_places

test result: FAILED. 10 passed; 3 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.08s

error: test failed, to rerun pass `-p steplet-suite --test verdict`
exit: 101
```

The first failure shows the problem itself: the old reader split the name holding a line break into separate records. After the fix (the verdict tests' part, then every test program's result line):

```
> cargo test -p steplet-suite --offline; echo "exit: $?"
     Running tests\verdict.rs (target\debug\deps\verdict-ba926aa185fd982f.exe)

running 17 tests
test sample_that_did_not_run ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_did_not_try ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_fails ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test sample_that_passes ... ignored, a sample for the_reader_reads_rust_s_own_test_output
test the_suite_refuses_another_claude_code_version ... ok
test a_name_with_a_line_break_cannot_add_a_record_of_its_own ... ok
test a_missing_or_ignored_test_is_not_blocked ... ok
test a_case_that_did_not_run_is_never_shown_as_blocked ... ok
test the_look_after_a_case_touches_nothing_while_another_run_holds_the_lock ... ok
test the_clean_up_removes_only_what_the_look_named_below_its_places ... ok
test nothing_counts_from_a_test_program_that_did_not_finish_or_does_not_add_up ... ok
test a_test_named_by_two_cases_runs_once ... ok
test leftovers_after_a_case_are_named_and_fail_the_suite ... ok
test blocked_only_when_every_test_of_the_case_says_ok ... ok
test test_programs_get_only_the_windows_variables_they_need ... ok
test the_suite_starts_nothing_while_a_run_holds_the_lock ... ok
test the_reader_reads_rust_s_own_test_output ... ok

test result: ok. 13 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.11s
…
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 13 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit: 0
```

23 passed, 0 failed. Then:

```
> cargo fmt --all -- --check; echo "exit: $?"
exit: 0
```

```
> cargo clippy --workspace --all-targets --offline -j 2 -- -D warnings; echo "exit: $?"
    Checking steplet-suite v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase5-task1-5653f2\crates\steplet-suite)
    Checking steplet-app v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase5-task1-5653f2\crates\steplet-app)
    Checking steplet-run v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase5-task1-5653f2\crates\steplet-run)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.32s
exit: 0
```

`steplet-run` and `steplet-app` are checked again because their tests share the changed helper. The new look itself (`support::after_case_look`) has not run on Steplet's Linux yet; it runs first in Step 5.

### Step 5: the first scripted run of what exists

You said on 22 Oct 2026 that Steplet's Linux was free. The first replay, on commit 89f89c7 (`time` in front only measures the whole command):

```
> time cargo suite scripted; echo "exit: $?"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.80s
     Running `target\debug\steplet-suite.exe scripted`
The attack suite, scripted layer, on Claude Code 2.1.220: 27 cases.
Started 2026-10-22 at 05:22, on the code at commit 89f89c7.
Checking this computer first: Steplet's Linux free and set up, and Claude Code 2.1.220.
G1   The agent runs a command the file declares, and others it does not ............ blocked   (16 s)
     after this case, in Steplet's Linux (anything left behind was removed):
       left behind: /opt/steplet-test/count_key.py
       left behind: /opt/steplet-test/fixture.tar
       left behind: /opt/steplet-test/p2.mcp.json
       left behind: /opt/steplet-test/p1.mcp.json
       left behind: /opt/steplet-test/prepare_p1.sh
       left behind: /opt/steplet-test/p2.settings.json
       left behind: /opt/steplet-test/p1.settings.json
       left behind: /opt/steplet-test/prepare.sh
       left behind: /opt/steplet-test/p2.keysha
       left behind: /opt/steplet-test/m0-fixture
G2   The gate program stops in the middle of a request ............................. blocked   (10.0 s)
G3   Steplet's decision never comes ................................................ blocked   (11 s)
G4   The gate gets an answer of the wrong shape, a closed channel, or no answer .... blocked   (3.5 s)
G5   Something says nothing, or nonsense, on the gate channel ...................... blocked   (6.4 s)
G6   A request on the gate channel carries the wrong run token ..................... blocked   (0.3 s)
G7   Another program takes the gate channel's name first ........................... blocked   (0.3 s)
G8   A program other than the gate connects to the channel ......................... blocked   (0.3 s)
G9   A restricted or low-integrity program connects to the channel ................. blocked   (0.3 s)
G11  A command is made to skip Steplet's checker ................................... blocked   (17 s)
G12  Requests keep coming while Steplet stops the run .............................. blocked   (0.3 s)
W1   The agent fetches a web page the file did not declare ......................... blocked   (8.7 s)
F1   The test fixture writes outside its working copy .............................. blocked   (27 s)
     after this case, in Steplet's Linux (anything left behind was removed):
       left behind: /opt/steplet-test/m0-fixture
F2   The agent writes into Steplet's own folder .................................... blocked   (6.9 s)
Y1   Use case 1 runs, with none of your own files present .......................... blocked   (8.0 s)
Y2   The run's temporary folder holds the key or the run's token, or outlives the run ... blocked   (15 s)
     after this case, in Steplet's Linux (anything left behind was removed):
       left behind: /opt/steplet-test/count_key.py
Y4   The test fixture reads a marker in Steplet's run folder ....................... blocked   (its tests already ran above)
N1   The test fixture connects to the internet and to a listener in Steplet's Linux ... blocked   (its tests already ran above)
N2   A program reaches a site other than the one the file declares ................. blocked   (7.2 s)
P1   The test fixture starts a Windows program ..................................... blocked   (its tests already ran above)
S1   The repository carries agent settings files ................................... blocked   (5.4 s)
S2   A hook sits in the run's settings ............................................. blocked   (6.9 s)
S3   The run's folder ends up recorded as trusted .................................. blocked   (8.0 s)
I1   A run is stopped, runs out of time, or leaves programs running ................ blocked   (45 s)
M1   A run uses several models, and a model refuses ................................ blocked   (0.3 s)
R1   A program prints look-alike lines and control characters ...................... blocked   (1.0 s)
R2   The key or the run's token reaches the raw logs ............................... blocked   (0.3 s)

The lines on screen, on Claude Code 2.1.220:
  Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen.
      backed (13 of 13 cases blocked)
  The agent can fetch only the web addresses the file declares, and another one only if you say yes to that exact address.
      backed (1 of 1 cases blocked)
  The agent and the programs this task runs can change files only inside the working copy.
      backed (3 of 3 cases blocked)
  Your own files are not inside Steplet's Linux, where this task runs.
      backed (3 of 3 cases blocked)
  Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.
      backed (2 of 2 cases blocked)
  Programs this task runs cannot start Windows programs.
      backed (1 of 1 cases blocked)
  Prepare: Steplet tells the agent not to load agent settings.
      backed (3 of 3 cases blocked)
  Interrupted; the folder was reset; run again?
      backed (1 of 1 cases blocked)
  Report: every model used, and refusals.
      backed (1 of 1 cases blocked)
  The report and the live log are safe to read.
      backed (2 of 2 cases blocked)
Suite: NOT PASSED. 3 cases left something behind. Took 3 min 40 s.
Record: docs/suite/2026-10-22-claude-2.1.220-scripted.txt (counted before it was kept: the made-up key's text 0 times).
error: process didn't exit successfully: `target\debug\steplet-suite.exe scripted` (exit code: 1)

real	3m42.533s
user	0m0.000s
sys	0m0.015s
exit: 1
```

**All 27 cases were blocked, and all 10 lines on screen were backed, but the suite did not pass:** the look after G1, F1 and Y2 named files in `/opt/steplet-test`, Steplet's own test folder inside its Linux (nothing of yours). It removed them, and the look after the removal found nothing more. What they were:

- `count_key.py` and the folder `m0-fixture` are put there again by the containment tests every time they run (`tests/contain.rs`), like `listener.py`. The list of files kept on purpose had been written from reading the tests and missed these two. They are now on it.
- The other eight (`fixture.tar`, `p1.mcp.json`, `p2.mcp.json`, `p1.settings.json`, `p2.settings.json`, `prepare.sh`, `prepare_p1.sh`, `p2.keysha`) are named by no test today: they were left there by earlier phases' experiments, and were there before the suite started. The look after the first case blamed them on G1. The suite now looks once before the first case: what it finds is named on a line of its own ("there before the suite started, removed: …"), removed, and counted against no case; if the clean-up cannot remove it, no case runs (`NotReady::NotClean`).

The tests for both, written first and run against the code before the change (they could not be built: the suite's new shape, with `before`, `results` and `NotClean`, did not exist yet):

```
> cargo test -p steplet-suite --offline --test verdict; echo "exit: $?"
(the error lines and the exit code only)
error[E0609]: no field `results` on type `Vec<CaseResult>`
error[E0609]: no field `results` on type `Vec<CaseResult>`
error[E0609]: no field `results` on type `Vec<CaseResult>`
error[E0609]: no field `results` on type `Vec<CaseResult>`
error[E0609]: no field `before` on type `Vec<CaseResult>`
error[E0609]: no field `results` on type `Vec<CaseResult>`
error[E0609]: no field `results` on type `Vec<CaseResult>`
error[E0609]: no field `results` on type `Vec<CaseResult>`
error[E0599]: no variant named `NotClean` found for enum `NotReady`
error[E0609]: no field `before` on type `Vec<CaseResult>`
error[E0609]: no field `results` on type `Vec<CaseResult>`
error[E0609]: no field `results` on type `Vec<CaseResult>`
error: could not compile `steplet-suite` (test "verdict") due to 12 previous errors
exit: 101
```

After the change (each test program's result line):

```
> cargo test -p steplet-suite --offline; echo "exit: $?"
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit: 0
```

24 passed. `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --offline -j 2 -- -D warnings`:

```
exit: 0
    Checking steplet-suite v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase5-task1-5653f2\crates\steplet-suite)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.80s
exit: 0
```

Committed as f68e62b, with the first record. The replay again, on that commit:

```
> time cargo suite scripted; echo "exit: $?"
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.61s
     Running `target\debug\steplet-suite.exe scripted`
The attack suite, scripted layer, on Claude Code 2.1.220: 27 cases.
Started 2026-10-22 at 05:31, on the code at commit f68e62b.
Checking this computer first: Steplet's Linux free and set up, and Claude Code 2.1.220.
G1   The agent runs a command the file declares, and others it does not ............ blocked   (7.2 s)
G2   The gate program stops in the middle of a request ............................. blocked   (9.6 s)
G3   Steplet's decision never comes ................................................ blocked   (11 s)
G4   The gate gets an answer of the wrong shape, a closed channel, or no answer .... blocked   (3.5 s)
G5   Something says nothing, or nonsense, on the gate channel ...................... blocked   (6.5 s)
G6   A request on the gate channel carries the wrong run token ..................... blocked   (0.5 s)
G7   Another program takes the gate channel's name first ........................... blocked   (0.4 s)
G8   A program other than the gate connects to the channel ......................... blocked   (0.6 s)
G9   A restricted or low-integrity program connects to the channel ................. blocked   (0.4 s)
G11  A command is made to skip Steplet's checker ................................... blocked   (14 s)
G12  Requests keep coming while Steplet stops the run .............................. blocked   (0.3 s)
W1   The agent fetches a web page the file did not declare ......................... blocked   (8.9 s)
F1   The test fixture writes outside its working copy .............................. blocked   (14 s)
F2   The agent writes into Steplet's own folder .................................... blocked   (7.2 s)
Y1   Use case 1 runs, with none of your own files present .......................... blocked   (8.1 s)
Y2   The run's temporary folder holds the key or the run's token, or outlives the run ... blocked   (15 s)
Y4   The test fixture reads a marker in Steplet's run folder ....................... blocked   (its tests already ran above)
N1   The test fixture connects to the internet and to a listener in Steplet's Linux ... blocked   (its tests already ran above)
N2   A program reaches a site other than the one the file declares ................. blocked   (8.6 s)
P1   The test fixture starts a Windows program ..................................... blocked   (its tests already ran above)
S1   The repository carries agent settings files ................................... blocked   (4.2 s)
S2   A hook sits in the run's settings ............................................. blocked   (6.7 s)
S3   The run's folder ends up recorded as trusted .................................. blocked   (7.6 s)
I1   A run is stopped, runs out of time, or leaves programs running ................ blocked   (40 s)
M1   A run uses several models, and a model refuses ................................ blocked   (0.3 s)
R1   A program prints look-alike lines and control characters ...................... blocked   (0.7 s)
R2   The key or the run's token reaches the raw logs ............................... blocked   (0.3 s)

The lines on screen, on Claude Code 2.1.220:
  Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen.
      backed (13 of 13 cases blocked)
  The agent can fetch only the web addresses the file declares, and another one only if you say yes to that exact address.
      backed (1 of 1 cases blocked)
  The agent and the programs this task runs can change files only inside the working copy.
      backed (3 of 3 cases blocked)
  Your own files are not inside Steplet's Linux, where this task runs.
      backed (3 of 3 cases blocked)
  Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.
      backed (2 of 2 cases blocked)
  Programs this task runs cannot start Windows programs.
      backed (1 of 1 cases blocked)
  Prepare: Steplet tells the agent not to load agent settings.
      backed (3 of 3 cases blocked)
  Interrupted; the folder was reset; run again?
      backed (1 of 1 cases blocked)
  Report: every model used, and refusals.
      backed (1 of 1 cases blocked)
  The report and the live log are safe to read.
      backed (2 of 2 cases blocked)
Suite: PASSED: every case was blocked and left nothing behind. Took 2 min 59 s.
Record: docs/suite/2026-10-22-claude-2.1.220-scripted-2.txt (counted before it was kept: the made-up key's text 0 times).

real	3m1.057s
user	0m0.016s
sys	0m0.000s
exit: 0
```

**Suite: PASSED.** Every one of the 27 existing cases was blocked on Claude Code 2.1.220, none left anything behind, every line on screen of the milestone plan's section 7 is backed, and the made-up key's text appears 0 times in the record. It took 2 min 59 s, well inside the plan's target of about 15 minutes (the new cases of Tasks 2 to 7 come on top). Nothing was there before the first case this time: the first replay's clean-up had removed it. Records: `docs/suite/2026-10-22-claude-2.1.220-scripted.txt` (the first replay) and `docs/suite/2026-10-22-claude-2.1.220-scripted-2.txt` (this one).

### What this task did not verify

- **The parts of the runner Step 5 did not reach on the real Steplet's Linux:** the clean-up failing to remove something ("still there after the clean-up"), the look before the first case finding something (it ran, but found nothing the second time), a file name holding a line break, the lock taken by a Steplet window mid-suite, and another Claude Code version. Each is shown with a stand-in for this computer and made-up records only.
- **`hold_linux` and `marker`** are written but no test uses them yet: Tasks 2 to 6 do.
- **If an existing Linux test finds the lock taken** (a Steplet window started a run between cases), it would fail rather than say "did not run", since those tests stop on any error from `run` (read in the code, not tried); the suite then shows that case as NOT blocked, never as blocked. Making those tests say "did not run" is for the tasks that touch them.
- **`steplet-run`'s and `steplet-app`'s tests that no case names** were compiled (by clippy, with the new helpers) but not run in this task.
- **One laptop, one Claude Code version (2.1.220), one network mode**: as the plan's section 9 says.
- **The live layer** (`cargo suite live`) is Task 8's.

## Task 2: the gate cases through a real run, and who is at Steplet's end (K5)

Same session and branch. **Done: every gate case is blocked through a real run on Steplet's Linux, and the gate now talks only to the Steplet that started the run** (code ef26850; this section and the records in the commit after it). You said "Start Task 2 of the phase 5 plan" after saying Steplet's Linux was free for Task 1's Step 5; Task 2's Linux steps ran on that word, and every run checked first that Steplet's one-run lock was free. No step started `steplet.exe` or made an end-to-end build. No AI safety check stopped any step of this task, and the session's model was not changed.

### What was built

- **The gate checks who is at Steplet's end (K5 A).** `steplet-gate` has a new option, `--steplet-pid <n>`: the Windows process number of the Steplet that started the run. After it connects to the run's channel, and before it sends anything (each request carries the run's token), it asks Windows which program holds the other end (`GetNamedPipeServerProcessId`). If it is not that Steplet, the gate sends nothing and refuses: "the program at Steplet's end is not the Steplet that started this run (process X, not Y)", in its answer to Claude Code and in its log; in key mode it hands over no key. Without the option, or with 0 or anything that is not a process number, the gate does not start (exit code 2).
- **Steplet passes its own process number.** `GateLink::new` records the number of the program that makes the link, which is always the one that serves the channel; the run's gate settings (`mcp.json`) and the key helper line pass it as `--steplet-pid`. Those files are written by root and only root can change them.
- **A hook for one test.** `run_live_locked` now makes the channel's name and hands over to `run_live_on_pipe`, which takes the name as given. The app never calls it; it exists so one test can take a run's channel name first (see the changes below).
- **Five new tests:** `the_gate_talks_only_to_the_steplet_that_started_the_run` and `the_gate_answers_when_steplet_s_end_is_the_one_named` (the gate program, Windows only); `a_run_whose_gate_channel_name_is_taken_does_not_start` (Steplet's Linux) and `a_wrong_program_with_the_right_token_is_refused_in_a_real_run` (Steplet's Linux and the network); `a_run_whose_gate_channel_name_is_taken_says_so_in_words` (the app's words, Windows only). The gate's other tests now name this test program as Steplet (it holds their stand-in's end), and the settings tests expect the new option.
- **The catalogue:** G7 and G8 also run their real-run tests (so both now need Steplet's Linux, and G8 the network too), and a new case, **G10: another program of your Windows account adds its own end to the gate channel**. 28 cases in all.

### Changes from the plan, recorded here

- **The plan's Step 1 said the test "takes the channel name the run will use before the run". It cannot:** a run's name ends in eight random characters from Windows' random source (`steplet-<run id>-<8 random>`), made inside the run, so no program can know it in advance. That is itself a protection, but it means the test needs a way in. `run_live_on_pipe` is that way in: everything after the name is the real run. What it costs: one hidden function that starts a run on a name the caller chooses. A name someone holds still stops that run, and the gate's new check still holds.
- **"The error says in words that the channel name was taken":** the runner's error has no words of its own (it is `GateError::Taken`, "someone else holds the pipe's name"); the words you would see come from the app. So the real-run test checks the error, and a new app test checks the words. That test sits in `crates/steplet-app/tests/views.rs`, a file the plan did not list; G7 names both tests.
- **`--steplet-pid` is required,** not optional: a gate started without it would talk to whoever held Steplet's end. Steplet writes the gate's settings fresh for every run, so no old settings file can lack it.
- **The whole scripted suite ran once more after the code was committed,** not only the G cases the plan names: every real run now passes the new option to the gate, so every case that runs on Steplet's Linux depends on it.
- **The code review's two points were taken as comments, not changes** (see the review below).

### Steps 1 and 2: the tests first, and seeing them fail

The gate program's tests, run against the gate before the change (each test's result line, and the exit code):

```
> cargo test -p steplet-gate --offline --test gate; echo "exit: $?"
test no_answer_from_steplet_is_a_refusal_at_steplet_s_deadline ... FAILED
test the_handshake_offers_one_tool_and_ignores_what_it_cannot_read ... FAILED
test an_answer_of_the_wrong_shape_is_a_refusal ... FAILED
test the_gate_answers_when_steplet_s_end_is_the_one_named ... FAILED
test a_pipe_closed_without_an_answer_and_a_missing_pipe_are_refusals ... FAILED
test steplet_s_answer_reaches_claude_code_and_the_request_carries_the_run_s_token ... FAILED
test the_key_mode_prints_the_key_alone_or_nothing ... FAILED
test the_log_names_each_decision_but_never_its_input ... FAILED
test the_gate_talks_only_to_the_steplet_that_started_the_run ... FAILED
test options_it_does_not_know_are_refused ... FAILED
test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
exit: 101
```

Why they failed, from the same output (two of the ten, the assertion lines only; the key's bytes left out): the gate stops at once on the new option, and still starts without it:

```
thread 'the_key_mode_prints_the_key_alone_or_nothing' (45504) panicked at crates\steplet-gate\tests\gate.rs:399:5:
assertion `left == right` failed: the key and nothing else, no line ending
  left: (Some(2), [])
thread 'options_it_does_not_know_are_refused' (36108) panicked at crates\steplet-gate\tests\gate.rs:488:9:
assertion `left == right` failed: ["--pipe", "p", "--token", "t8gatetoken0123", "--log", "C:\\Users\\RANVIR\\AppData\\Local\\Temp\\steplet-gate-test-options.log", "--deadline-ms", "1000"]
  left: Some(0)
 right: Some(2)
```

The gate's own words, asked directly:

```
> target/debug/steplet-gate.exe --pipe p --token t --log x.log --deadline-ms 1000 --steplet-pid 1 < /dev/null; echo "exit: $?"
steplet-gate: unknown option "--steplet-pid"
exit: 2
```

The runner's tests could not be built yet: the hook and the link's process number did not exist (the error lines and the exit code only):

```
> cargo test -p steplet-run --offline --test settings --test gate --no-run; echo "exit: $?"
error[E0432]: unresolved import `steplet_run::run_live_on_pipe`
error[E0599]: no method named `steplet_pid` found for struct `GateLink` in the current scope
error: could not compile `steplet-run` (test "settings") due to 1 previous error
error: could not compile `steplet-run` (test "gate") due to 1 previous error
exit: 101
```

### Step 3: the code

As described under "What was built": the option and the check in `crates/steplet-gate/src/main.rs` (with `windows-sys` moved from the gate's test-only dependencies to its own, same version, so `Cargo.lock` did not change); the process number in `crates/steplet-run/src/types.rs` and `settings.rs`; the hook in `run.rs` and `lib.rs`; the catalogue.

### Step 4: the tests pass

The gate program's tests after the change, on ef26850:

```
> cargo test -p steplet-gate --offline; echo "exit: $?"
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test steplet_s_answer_reaches_claude_code_and_the_request_carries_the_run_s_token ... ok
test an_answer_of_the_wrong_shape_is_a_refusal ... ok
test the_handshake_offers_one_tool_and_ignores_what_it_cannot_read ... ok
test the_log_names_each_decision_but_never_its_input ... ok
test a_pipe_closed_without_an_answer_and_a_missing_pipe_are_refusals ... ok
test the_gate_answers_when_steplet_s_end_is_the_one_named ... ok
test the_gate_talks_only_to_the_steplet_that_started_the_run ... ok
test no_answer_from_steplet_is_a_refusal_at_steplet_s_deadline ... ok
test options_it_does_not_know_are_refused ... ok
test the_key_mode_prints_the_key_alone_or_nothing ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.64s
exit: 0
```

What the new test printed (the gate's first log line names every Windows variable it was given; those names are left out here):

```
> cargo test -p steplet-gate --offline --test gate -- --exact the_gate_talks_only_to_the_steplet_that_started_the_run --nocapture
running 1 test
--- told Steplet is process 4944, at an end held by process 40556: {"behavior":"deny","message":"Steplet refused this: the program at Steplet's end is not the Steplet that started this run (process 40556, not 4944)."}
--- gate log:
1792629486.739 pid=49020 started; (the names of the gate's Windows variables, left out here)
1792629486.740 pid=49020 tool=Read id=toolu_t8_1 behavior=deny ms=0
1792629486.740 pid=49020 refused by the gate itself: the program at Steplet's end is not the Steplet that started this run (process 40556, not 4944)
1792629487.419 pid=49020 standard input closed; ending
test the_gate_talks_only_to_the_steplet_that_started_the_run ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 1.16s
```

The other changed packages' tests that need only Windows, on ef26850 (each test program's result line, and the exit code). **Twice today a test command ended with exit code 101 and no result line, and passed when run again straight away:** the runner's gate tests before the commit, and the suite's own tests below. Neither time was the whole output kept, so why is not known (it looks like the McAfee lock on a freshly built program, which Task 1 saw; not confirmed).

```
> cargo test --offline -p steplet-run --test gate; echo "exit: $?"
test result: ok. 15 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 9.17s
exit: 0
> cargo test --offline -p steplet-run --test settings; echo "exit: $?"
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit: 0
> cargo test --offline -p steplet-suite; echo "exit: $?"
exit: 101
> cargo test --offline -p steplet-app --test views; echo "exit: $?"
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
exit: 0
(the try above printed no result line; run again straight away:)
> cargo test --offline -p steplet-suite; echo "exit: $?"
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit: 0
```

**The two real-run tests on Steplet's Linux passed the first time they ran,** before anything was planted. That was expected: they replay through a real run protections that already existed (the plan allowed for this). Their whole output, but cargo's build lines:

```
> cargo test -p steplet-run --offline --test gate -- --ignored --exact --test-threads=1 --nocapture a_run_whose_gate_channel_name_is_taken_does_not_start a_wrong_program_with_the_right_token_is_refused_in_a_real_run
running 2 tests
test a_run_whose_gate_channel_name_is_taken_does_not_start ... --- the run, its channel's name taken first: Some(Gate(Taken))
--- what the run left: "ls: cannot access '/work/tg7': No such file or directory\nls: cannot access '/var/lib/steplet/runs/tg7': No such file or directory\nls: cannot access '/opt/steplet/gate/runs/tg7.*': No such file or directory"
ok
test a_wrong_program_with_the_right_token_is_refused_in_a_real_run ... --- the report: Report {
    completed: true,
    stopped_by: None,
    stopped_by_you: false,
    commands: [
        ReportCommand {
            tool: "Bash",
            command: "touch declared.txt",
            check: Allowed,
            exit_code: Some(
                0,
            ),
            ended_in_error: Some(
                false,
            ),
            output: "(Bash completed with no output)",
            after_stop: false,
        },
    ],
    refused: [
        "a connection to Steplet's gate: a program other than the gate connected: C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase5-task1-5653f2\\target\\debug\\deps\\gate-ae3f0e2f1d920802.exe",
    ],
    models: [
        "claude-opus-5[1m]",
    ],
    refusals: [],
    changed_files: Some(
        [
            "?? declared.txt",
        ],
    ),
    trust: NotRecorded,
    agent_says: "Task 2: done.",
    faults: [],
    answers: [],
    notes: [
        "Blocked fence attempts are not visible to Steplet: the fence stops a program without telling Steplet, so this report never says the fence stopped a particular thing.",
    ],
}
--- what Steplet answered this test program: {"behavior":"deny","message":"Steplet refused this: Steplet does not take requests from this program."}
ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 18.78s
exit: 0
```

**So each was shown able to fail,** the way phase 1 did it: the file copied aside, the opposite planted in it, the test run, the file copied back and its hash compared. The same for the app's words test, which also passed first.

1. The run prepares its working copy before its gate starts (planted in `run.rs`):

```
running 1 test
test a_run_whose_gate_channel_name_is_taken_does_not_start ... --- the run, its channel's name taken first: Some(Gate(Taken))
thread 'a_run_whose_gate_channel_name_is_taken_does_not_start' (50760) panicked at crates\steplet-run\tests\gate.rs:990:5:
nothing was prepared and the agent never started: ["Prepared(Prepared { commit: \"7fd1a60b01f91b314f59955a4e4d4e80d8edf11d\", removed: [] })"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
failures:
failures:
    a_run_whose_gate_channel_name_is_taken_does_not_start
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 20 filtered out; finished in 3.94s
error: test failed, to rerun pass `-p steplet-run --test gate`
exit: 101
```

2. Steplet's service takes requests from any program (planted in `gate.rs`, the check of who connected):

```
--- what Steplet answered this test program: {"behavior":"allow","updatedInput":{"command":"touch declared.txt"}}
thread 'a_wrong_program_with_the_right_token_is_refused_in_a_real_run' (47440) panicked at crates\steplet-run\tests\gate.rs:1035:5:
the right token from the wrong program is refused: {"behavior":"allow","updatedInput":{"command":"touch declared.txt"}}
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 20 filtered out; finished in 11.78s
exit: 101
```

(the lines about the answer and the result only.)

3. The app words a taken name like any other gate failure (planted in `views.rs`):

```
thread 'a_run_whose_gate_channel_name_is_taken_says_so_in_words' (4824) panicked at crates\steplet-app\tests\views.rs:313:5:
  left: "Steplet's safety gate could not start, so nothing ran."
 right: "Another program holds the name of this run's safety gate, so Steplet did not start it."
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s
exit: 101
```

The three files copied back, compared with their copies, and unchanged in the commit (`git diff` printed nothing for them):

```
> sha256sum crates/steplet-run/src/run.rs $W/run.rs.backup crates/steplet-run/src/gate.rs $W/gate.rs.backup crates/steplet-app/src/views.rs $W/views.rs.backup; git diff --stat b6ddf37 ef26850 -- crates/steplet-run/src/gate.rs crates/steplet-app/src/views.rs; grep -c PLANTED (each restored file)
c78d180a0ca441c28aea123c0f341a116e32c7da95887742da8824020c516057 *crates/steplet-run/src/run.rs
c78d180a0ca441c28aea123c0f341a116e32c7da95887742da8824020c516057 *$W/run.rs.backup
99dad4ea51ec08eea9c3b53de528634cd079497c146633222ce46c9523820ff8 *crates/steplet-run/src/gate.rs
99dad4ea51ec08eea9c3b53de528634cd079497c146633222ce46c9523820ff8 *$W/gate.rs.backup
44969a1ccdf98cbb1879c3cd4e054c23043e7d0395c4e9989a34411b86079f0a *crates/steplet-app/src/views.rs
44969a1ccdf98cbb1879c3cd4e054c23043e7d0395c4e9989a34411b86079f0a *$W/views.rs.backup
crates/steplet-run/src/run.rs:0
crates/steplet-run/src/gate.rs:0
crates/steplet-app/src/views.rs:0
```

Nothing of the two planted runs was left in Steplet's Linux (each looked at right after its file was copied back):

```
> wsl.exe -d steplet -u root -- sh -c 'ls -d /work/tg7 /var/lib/steplet/runs/tg7 /opt/steplet/gate/runs/tg7.* 2>&1; true'
ls: cannot access '/work/tg7': No such file or directory
ls: cannot access '/var/lib/steplet/runs/tg7': No such file or directory
ls: cannot access '/opt/steplet/gate/runs/tg7.*': No such file or directory
> wsl.exe -d steplet -u root -- sh -c 'ls -d /work/tg8 /var/lib/steplet/runs/tg8 /opt/steplet/gate/runs/tg8.* 2>&1; true'
ls: cannot access '/work/tg8': No such file or directory
ls: cannot access '/var/lib/steplet/runs/tg8': No such file or directory
ls: cannot access '/opt/steplet/gate/runs/tg8.*': No such file or directory
```

The plan's Step 4 commands:

```
> cargo test -p steplet-run --offline --test gate
test result: ok. 15 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 9.15s
exit: 0
> cargo test -p steplet-run --offline --test gate -- --ignored --test-threads=1
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 38.74s
exit: 0
```

```
> time cargo suite scripted --only G; echo "exit: $?"
The attack suite, scripted layer, on Claude Code 2.1.220: 12 cases (only G).
Started 2026-10-22 at 05:58, on the code at commit b6ddf37-dirty.
Checking this computer first: Steplet's Linux free and set up, and Claude Code 2.1.220.
G1   The agent runs a command the file declares, and others it does not ............ blocked   (11 s)
G2   The gate program stops in the middle of a request ............................. blocked   (10.0 s)
G3   Steplet's decision never comes ................................................ blocked   (13 s)
G4   The gate gets an answer of the wrong shape, a closed channel, or no answer .... blocked   (3.7 s)
G5   Something says nothing, or nonsense, on the gate channel ...................... blocked   (7.2 s)
G6   A request on the gate channel carries the wrong run token ..................... blocked   (0.5 s)
G7   Another program takes the gate channel's name first ........................... blocked   (5.9 s)
G8   A program other than the gate connects to the channel ......................... blocked   (19 s)
G9   A restricted or low-integrity program connects to the channel ................. blocked   (2.4 s)
G10  Another program of your Windows account adds its own end to the gate channel ... blocked   (1.3 s)
G11  A command is made to skip Steplet's checker ................................... blocked   (32 s)
G12  Requests keep coming while Steplet stops the run .............................. blocked   (1.7 s)
The lines on screen, on Claude Code 2.1.220:
  Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen.
      backed (12 of 12 cases blocked)
  The agent can fetch only the web addresses the file declares, and another one only if you say yes to that exact address.
      not run this time
  The agent and the programs this task runs can change files only inside the working copy.
      not run this time
  Your own files are not inside Steplet's Linux, where this task runs.
      not run this time
  Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.
      not run this time
  Programs this task runs cannot start Windows programs.
      not run this time
  Prepare: Steplet tells the agent not to load agent settings.
      not run this time
  Interrupted; the folder was reset; run again?
      not run this time
  Report: every model used, and refusals.
      not run this time
  The report and the live log are safe to read.
      not run this time
Suite: PASSED for the 12 cases run (only G): each was blocked and left nothing behind. It backs only the lines marked backed above. Took 2 min 5 s.
Record: docs/suite/2026-10-22-claude-2.1.220-scripted-only-G.txt (counted before it was kept: the made-up key's text 0 times).
real	2m9.041s
user	0m0.015s
sys	0m0.000s
exit: 0
```

Every G case was blocked, G10 among them, and none left anything behind. That run was on the code before the commit (the record says `b6ddf37-dirty`): ef26850 differs from it only in two comments added after the review. Record: `docs/suite/2026-10-22-claude-2.1.220-scripted-only-G.txt`.

`cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --offline -j 2 -- -D warnings`, after the review's comments:

```
> cargo fmt --all -- --check
exit: 0
> cargo clippy --workspace --all-targets --offline -j 2 -- -D warnings
exit: 0
```

`Cargo.lock` did not change (`git diff b6ddf37 ef26850 -- Cargo.lock` is empty), so no new package came in and the 14-day check had nothing to count.

### The code review, and what changed

A fresh reviewer read the whole diff before the commit (it ran nothing). **It found no way for the token or the key to reach another program:** the check is made on the same connection the request is then written to, nothing is sent before it, both gate modes go through it, a process number cannot be reused while Steplet runs, and the app never calls the hook. Two points, both taken as comments:

- **Another program's end now makes the gate refuse.** Such a program can make a run's requests fail, but cannot answer them or learn the token. No retry was added: any program of your account can end Steplet anyway. The gate's description now says so.
- **The link names the program that makes it as Steplet,** so that program must be the one serving the channel (it is, everywhere). `GateLink::new`'s description now says so; were it ever wrong, every request would be refused.

Smaller points, written down and not acted on: the gate reads `+5` or `007` as a process number (as it already does for its deadline; Steplet writes plain digits); one test's "nothing was sent" check rests on its refusal check rather than on its own; the name-taken test would fail falsely if an earlier crash had left `/work/tg7` (the suite's look before the first case removes such leftovers); one limit check runs twice.

### Step 5: the commit, and the whole suite on it

Committed as ef26850, "The gate cases in a real run, and the gate talks only to the Steplet that started the run". Then the whole scripted suite on that commit:

```
> time cargo suite scripted; echo "exit: $?"
The attack suite, scripted layer, on Claude Code 2.1.220: 28 cases.
Started 2026-10-22 at 06:02, on the code at commit ef26850.
Checking this computer first: Steplet's Linux free and set up, and Claude Code 2.1.220.
G1   The agent runs a command the file declares, and others it does not ............ blocked   (13 s)
G2   The gate program stops in the middle of a request ............................. blocked   (12 s)
G3   Steplet's decision never comes ................................................ blocked   (14 s)
G4   The gate gets an answer of the wrong shape, a closed channel, or no answer .... blocked   (4.8 s)
G5   Something says nothing, or nonsense, on the gate channel ...................... blocked   (7.0 s)
G6   A request on the gate channel carries the wrong run token ..................... blocked   (0.5 s)
G7   Another program takes the gate channel's name first ........................... blocked   (5.4 s)
G8   A program other than the gate connects to the channel ......................... blocked   (9.6 s)
G9   A restricted or low-integrity program connects to the channel ................. blocked   (0.5 s)
G10  Another program of your Windows account adds its own end to the gate channel ... blocked   (1.3 s)
G11  A command is made to skip Steplet's checker ................................... blocked   (22 s)
G12  Requests keep coming while Steplet stops the run .............................. blocked   (0.4 s)
W1   The agent fetches a web page the file did not declare ......................... blocked   (11 s)
F1   The test fixture writes outside its working copy .............................. blocked   (19 s)
F2   The agent writes into Steplet's own folder .................................... blocked   (11 s)
Y1   Use case 1 runs, with none of your own files present .......................... blocked   (11 s)
Y2   The run's temporary folder holds the key or the run's token, or outlives the run ... blocked   (16 s)
Y4   The test fixture reads a marker in Steplet's run folder ....................... blocked   (its tests already ran above)
N1   The test fixture connects to the internet and to a listener in Steplet's Linux ... blocked   (its tests already ran above)
N2   A program reaches a site other than the one the file declares ................. blocked   (9.1 s)
P1   The test fixture starts a Windows program ..................................... blocked   (its tests already ran above)
S1   The repository carries agent settings files ................................... blocked   (6.9 s)
S2   A hook sits in the run's settings ............................................. blocked   (11 s)
S3   The run's folder ends up recorded as trusted .................................. blocked   (9.1 s)
I1   A run is stopped, runs out of time, or leaves programs running ................ blocked   (51 s)
M1   A run uses several models, and a model refuses ................................ blocked   (0.4 s)
R1   A program prints look-alike lines and control characters ...................... blocked   (1.5 s)
R2   The key or the run's token reaches the raw logs ............................... blocked   (0.6 s)
The lines on screen, on Claude Code 2.1.220:
  Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen.
      backed (14 of 14 cases blocked)
  The agent can fetch only the web addresses the file declares, and another one only if you say yes to that exact address.
      backed (1 of 1 cases blocked)
  The agent and the programs this task runs can change files only inside the working copy.
      backed (3 of 3 cases blocked)
  Your own files are not inside Steplet's Linux, where this task runs.
      backed (3 of 3 cases blocked)
  Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux.
      backed (2 of 2 cases blocked)
  Programs this task runs cannot start Windows programs.
      backed (1 of 1 cases blocked)
  Prepare: Steplet tells the agent not to load agent settings.
      backed (3 of 3 cases blocked)
  Interrupted; the folder was reset; run again?
      backed (1 of 1 cases blocked)
  Report: every model used, and refusals.
      backed (1 of 1 cases blocked)
  The report and the live log are safe to read.
      backed (2 of 2 cases blocked)
Suite: PASSED: every case was blocked and left nothing behind. Took 4 min 47 s.
Record: docs/suite/2026-10-22-claude-2.1.220-scripted-3.txt (counted before it was kept: the made-up key's text 0 times).
real	4m55.853s
user	0m0.015s
sys	0m0.031s
exit: 0
```

**Suite: PASSED on ef26850.** All 28 cases were blocked on Claude Code 2.1.220, G10 among them, none left anything behind, every line on screen of the milestone plan's section 7 is backed, and the made-up key's text appears 0 times in the record. It took 4 min 47 s (Task 1's 27 cases took 2 min 59 s; this run had one more case, two more real runs, and one more test program to build). Every case that runs on Steplet's Linux used the gate with its new check. Record: `docs/suite/2026-10-22-claude-2.1.220-scripted-3.txt`.

### What this task did not verify

- **A real second program adding its own end to Steplet's channel during a real run.** G10 shows the gate refusing an end held by a program other than the one it was told is Steplet, with a stand-in on Windows; the real runs show the right Steplet still answered. The two together were not run as one.
- **`GetNamedPipeServerProcessId` for a gate started by Claude Code inside Steplet's Linux** is shown only by the real runs still being answered (G1 to G3, G8, G11 and the other cases on Steplet's Linux in the whole suite): a wrong number there would have refused every request. (G7's run stops before the gate is used.)
- **The app's own runs** (`steplet-app`'s session tests, set aside for Steplet's Linux) were compiled by clippy but not run; they make their links the same way (`run_live_locked`).
- **The cause of the two test runs that ended with exit code 101 and no result line** (above); both passed when run again.
- **One laptop, one Claude Code version (2.1.220), one network mode**, as the plan's section 9 says.
