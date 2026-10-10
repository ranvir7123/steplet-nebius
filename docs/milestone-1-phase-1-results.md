# Milestone 1, phase 1: results

Plan: `docs/plans/2026-10-11-milestone-1-phase-1.md`. Each task: the commands, their pasted output, and a verdict.

## Task 1: workspace, dates, limit numbers

Run 11 Oct 2026, in the worktree `.claude\worktrees\steplet-phase1-task1-run-e26214`, starting from `93c9fe1`. Each command was run as `cmd /c "<command> > file 2>&1"` so both of cargo's output streams are captured; the text below is that file, unchanged.

**Step 2, the failing state** (the tests exist, `Date` is still empty):

```
> cargo test -p steplet-core --test date
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
 --> crates\steplet-core\tests\date.rs:5:22
  |
5 |     assert_eq!(Date::parse("2026-10-25").unwrap().to_string(), "2026-10-25");
  |                      ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
 --> crates\steplet-core\tests\date.rs:6:19
  |
6 |     assert!(Date::parse("2028-02-29").is_some());
  |                   ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
 --> crates\steplet-core\tests\date.rs:7:19
  |
7 |     assert!(Date::parse("2000-02-29").is_some());
  |                   ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:17:26
   |
17 |         assert_eq!(Date::parse(s), None, "{s:?}");
   |                          ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:23:19
   |
23 |     let d = Date::parse("2026-10-11").unwrap();
   |                   ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `from_days_since_epoch` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:25:22
   |
25 |     assert_eq!(Date::from_days_since_epoch(20_737), Some(d));
   |                      ^^^^^^^^^^^^^^^^^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:26:22
   |
26 |     assert_eq!(Date::parse("2000-01-01").unwrap().days_since_epoch(), 10_957);
   |                      ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:31:19
   |
31 |     let d = Date::parse("2026-10-11").unwrap();
   |                   ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `from_unix` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:33:22
   |
33 |     assert_eq!(Date::from_unix(1_791_763_199), Some(d));
   |                      ^^^^^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `from_unix` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:34:22
   |
34 |     assert_eq!(Date::from_unix(1_791_763_200), Date::parse("2026-10-12"));
   |                      ^^^^^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:34:54
   |
34 |     assert_eq!(Date::from_unix(1_791_763_200), Date::parse("2026-10-12"));
   |                                                      ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:39:19
   |
39 |     let d = Date::parse("2026-10-11").unwrap();
   |                   ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:40:38
   |
40 |     assert_eq!(d.add_days(14), Date::parse("2026-10-25"));
   |                                      ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:41:39
   |
41 |     assert_eq!(d.add_days(365), Date::parse("2027-10-11"));
   |                                       ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:42:22
   |
42 |     assert_eq!(Date::parse("2028-02-28").unwrap().add_days(1), Date::parse("2028-02-29"));
   |                      ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:42:70
   |
42 |     assert_eq!(Date::parse("2028-02-28").unwrap().add_days(1), Date::parse("2028-02-29"));
   |                                                                      ^^^^^ associated function or constant not found in `Date`

error[E0599]: no associated function or constant named `parse` found for struct `Date` in the current scope
  --> crates\steplet-core\tests\date.rs:43:22
   |
43 |     assert_eq!(Date::parse("9999-12-31").unwrap().add_days(1), None);
   |                      ^^^^^ associated function or constant not found in `Date`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `steplet-core` (test "date") due to 17 previous errors
(exit code 101)
```

The plan expected the wording `no function or associated item named parse`; Rust 1.98.0 says `no associated function or constant named parse`. Same error (E0599), different words. Not a problem.

**Step 4, after writing `date.rs`:**

```
> cargo test -p steplet-core --test date
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.97s
     Running tests\date.rs (target\debug\deps\date-5a189c9e65d3ab0d.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test reads_and_writes_dates ... ok
test counts_days_from_1970 ... ok
test adds_days_across_months_and_years ... ok
test refuses_anything_else ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `release` profile [optimized] target(s) in 0.57s
(exit code 0)
```

```
> cargo test --manifest-path spikes/Cargo.toml -p s9-core
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.26
   Compiling quote v1.0.47
   Compiling semver v1.0.28
   Compiling cfg-if v1.0.5
   Compiling cpufeatures v0.2.17
   Compiling base64ct v1.8.3
   Compiling signature v2.2.0
   Compiling subtle v2.6.1
   Compiling zeroize v1.9.1
   Compiling rand_core v0.6.4
   Compiling ed25519 v2.2.3
   Compiling rustc_version v0.4.1
   Compiling pem-rfc7468 v0.7.0
   Compiling generic-array v0.14.7
   Compiling curve25519-dalek v4.1.3
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling inout v0.1.4
   Compiling cipher v0.4.4
   Compiling syn v2.0.119
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ssh-encoding v0.2.0
   Compiling ssh-cipher v0.2.0
   Compiling curve25519-dalek-derive v0.1.1
   Compiling ed25519-dalek v2.2.0
   Compiling ssh-key v0.6.7
   Compiling s9-core v0.0.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\spikes\s9-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.58s
     Running unittests src\lib.rs (spikes\target\debug\deps\s9_core-b2d990b6f721dc20.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\vectors.rs (spikes\target\debug\deps\vectors-2796e6285c70ace0.exe)

running 3 tests
test wrong_namespace_fails ... ok
test ssh_keygen_signature_verifies_with_the_same_fingerprint ... ok
test changed_message_fails ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests s9_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

**Verdict: done.** 5 of 5 date tests pass; the core builds for `wasm32v1-none` (no operating system); the spike workspace still builds on its own and passes 3 of 3, so the new root workspace does not swallow it. `Cargo.lock` was written with only `steplet-core` in it; `spikes/Cargo.lock` did not change (`git status` showed no change under `spikes/`). No compile errors in the plan's code for this task; nothing was changed from the plan.

## Task 2: the clean-up and the header/body split

**Step 1, the failing state** (the tests exist; `canon.rs` is a stub that always answers "no header start", with no words and no help):

```
> cargo test -p steplet-core --test canon
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.50s
     Running tests\canon.rs (target\debug\deps\canon-0f861d96947b4089.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... FAILED
test exactly_one_final_newline ... FAILED
test the_header_opens_on_line_one_and_closes ... FAILED
test no_unicode_normalisation ... FAILED
test spaces_and_tabs_at_line_ends_go ... FAILED
test windows_line_endings_become_unix_ones ... FAILED
test one_byte_order_mark_is_dropped_and_only_one ... FAILED
test bad_utf8_names_its_line ... FAILED
test every_error_has_words_and_help ... FAILED
test header_and_body_are_split_at_the_first_closing_line ... FAILED
test leading_and_inner_spaces_stay ... FAILED
test size_limits ... FAILED

failures:

---- a_lone_carriage_return_is_refused stdout ----

thread 'a_lone_carriage_return_is_refused' (8104) panicked at crates\steplet-core\tests\canon.rs:48:5:
assertion `left == right` failed
  left: Err(NoHeaderStart)
 right: Err(LoneCarriageReturn { line: 2 })

---- exactly_one_final_newline stdout ----

thread 'exactly_one_final_newline' (40012) panicked at crates\steplet-core\tests\canon.rs:4:34:
should clean up: NoHeaderStart

---- the_header_opens_on_line_one_and_closes stdout ----

thread 'the_header_opens_on_line_one_and_closes' (10336) panicked at crates\steplet-core\tests\canon.rs:56:5:
assertion `left == right` failed
  left: Err(NoHeaderStart)
 right: Err(NoHeaderEnd)

---- no_unicode_normalisation stdout ----

thread 'no_unicode_normalisation' (41856) panicked at crates\steplet-core\tests\canon.rs:4:34:
should clean up: NoHeaderStart

---- spaces_and_tabs_at_line_ends_go stdout ----

thread 'spaces_and_tabs_at_line_ends_go' (34440) panicked at crates\steplet-core\tests\canon.rs:4:34:
should clean up: NoHeaderStart

---- windows_line_endings_become_unix_ones stdout ----

thread 'windows_line_endings_become_unix_ones' (44604) panicked at crates\steplet-core\tests\canon.rs:4:34:
should clean up: NoHeaderStart

---- one_byte_order_mark_is_dropped_and_only_one stdout ----

thread 'one_byte_order_mark_is_dropped_and_only_one' (45880) panicked at crates\steplet-core\tests\canon.rs:4:34:
should clean up: NoHeaderStart

---- bad_utf8_names_its_line stdout ----

thread 'bad_utf8_names_its_line' (31924) panicked at crates\steplet-core\tests\canon.rs:43:5:
assertion `left == right` failed
  left: Err(NoHeaderStart)
 right: Err(NotUtf8 { line: 3 })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- every_error_has_words_and_help stdout ----

thread 'every_error_has_words_and_help' (44296) panicked at crates\steplet-core\tests\canon.rs:95:9:
FileTooLarge { bytes: 1 }

---- header_and_body_are_split_at_the_first_closing_line stdout ----

thread 'header_and_body_are_split_at_the_first_closing_line' (35640) panicked at crates\steplet-core\tests\canon.rs:4:34:
should clean up: NoHeaderStart

---- leading_and_inner_spaces_stay stdout ----

thread 'leading_and_inner_spaces_stay' (18164) panicked at crates\steplet-core\tests\canon.rs:4:34:
should clean up: NoHeaderStart

---- size_limits stdout ----

thread 'size_limits' (41136) panicked at crates\steplet-core\tests\canon.rs:73:5:
assertion failed: canonicalize(&at_limit).is_ok()


failures:
    a_lone_carriage_return_is_refused
    bad_utf8_names_its_line
    every_error_has_words_and_help
    exactly_one_final_newline
    header_and_body_are_split_at_the_first_closing_line
    leading_and_inner_spaces_stay
    no_unicode_normalisation
    one_byte_order_mark_is_dropped_and_only_one
    size_limits
    spaces_and_tabs_at_line_ends_go
    the_header_opens_on_line_one_and_closes
    windows_line_endings_become_unix_ones

test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p steplet-core --test canon`
(exit code 101)
```

All 12 fail, each for the reason the plan gives.

**Step 3, after writing `canon.rs`:**

```
> cargo test -p steplet-core --test canon
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.31s
     Running tests\canon.rs (target\debug\deps\canon-0f861d96947b4089.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... ok
test bad_utf8_names_its_line ... ok
test every_error_has_words_and_help ... ok
test exactly_one_final_newline ... ok
test leading_and_inner_spaces_stay ... ok
test the_header_opens_on_line_one_and_closes ... ok
test no_unicode_normalisation ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test windows_line_endings_become_unix_ones ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

**Check before committing** (the whole workspace, and the no-operating-system build again):

```
> cargo test --workspace
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.25s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-38718543c69260ea.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-0f861d96947b4089.exe)

running 12 tests
test exactly_one_final_newline ... ok
test leading_and_inner_spaces_stay ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test a_lone_carriage_return_is_refused ... ok
test the_header_opens_on_line_one_and_closes ... ok
test bad_utf8_names_its_line ... ok
test every_error_has_words_and_help ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test no_unicode_normalisation ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test windows_line_endings_become_unix_ones ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-5a189c9e65d3ab0d.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test adds_days_across_months_and_years ... ok
test counts_days_from_1970 ... ok
test reads_and_writes_dates ... ok
test refuses_anything_else ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `release` profile [optimized] target(s) in 0.86s
(exit code 0)
```

**Verdict: done.** 12 of 12 clean-up tests pass; 17 tests pass in the workspace; no compiler warnings; the core still builds for `wasm32v1-none`. The plan's code compiled as written; nothing was changed from the plan.
