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
