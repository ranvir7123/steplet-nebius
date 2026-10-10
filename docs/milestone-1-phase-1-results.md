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

## Task 3: the signature block, by fixed markers

**Step 1, the failing state, first run** (the tests exist; `with_signature` does not):

```
> cargo test -p steplet-core --test canon_signature
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
error[E0599]: no method named `with_signature` found for struct `Canonical` in the current scope
  --> crates\steplet-core\tests\canon_signature.rs:75:27
   |
75 |     let signed = unsigned.with_signature("-----BEGIN SSH SIGNATURE-----\r\nU1NIU0lHAAAAAQ==\r\n-----END SSH SIGNATURE-----\r\n\r\n");
   |                           ^^^^^^^^^^^^^^
   |
help: there is a method `signature` with a similar name, but with different arguments
  --> crates\steplet-core\src\canon.rs:61:5
   |
61 |     pub fn signature(&self) -> Option<&SignatureBlock> {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `with_signature` found for struct `Canonical` in the current scope
  --> crates\steplet-core\tests\canon_signature.rs:79:22
   |
79 |     assert_eq!(again.with_signature(PEM), signed);
   |                      ^^^^^^^^^^^^^^
   |
help: there is a method `signature` with a similar name, but with different arguments
  --> crates\steplet-core\src\canon.rs:61:5
   |
61 |     pub fn signature(&self) -> Option<&SignatureBlock> {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

For more information about this error, try `rustc --explain E0599`.
error: could not compile `steplet-core` (test "canon_signature") due to 2 previous errors
(exit code 101)
```

**Step 1, second run** (a stub `with_signature` that returns an empty text; the search is still the placeholder that never finds a block):

```
> cargo test -p steplet-core --test canon_signature
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.47s
     Running tests\canon_signature.rs (target\debug\deps\canon_signature-2906a30ba1950f50.exe)

running 10 tests
test block_lines_must_be_indented_base64 ... FAILED
test end_marker_missing ... FAILED
test the_key_line_must_be_exact ... FAILED
test two_blocks ... FAILED
test the_block_is_split_off ... FAILED
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... FAILED
test block_not_last_in_the_header ... FAILED
test block_in_the_body ... FAILED
test markers_without_the_key_line ... FAILED
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... FAILED

failures:

---- block_lines_must_be_indented_base64 stdout ----

thread 'block_lines_must_be_indented_base64' (35768) panicked at crates\steplet-core\tests\canon_signature.rs:68:9:
"  U1NI U0lH"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- end_marker_missing stdout ----

thread 'end_marker_missing' (39360) panicked at crates\steplet-core\tests\canon_signature.rs:53:5:
assertion `left == right` failed
  left: Ok(Canonical { text: "---\nsteplet: 1\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n---\n", signed: "---\nsteplet: 1\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n---\n", header: 4..79, body: 83..83, signature: None })
 right: Err(SignatureBroken { line: 6 })

---- the_key_line_must_be_exact stdout ----

thread 'the_key_line_must_be_exact' (29640) panicked at crates\steplet-core\tests\canon_signature.rs:60:9:
assertion `left == right` failed: signature: >
  left: Ok(Canonical { text: "---\nsteplet: 1\nsignature: >\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n---\n", signed: "---\nsteplet: 1\nsignature: >\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n---\n", header: 4..109, body: 113..113, signature: None })
 right: Err(SignatureBroken { line: 3 })

---- two_blocks stdout ----

thread 'two_blocks' (36964) panicked at crates\steplet-core\tests\canon_signature.rs:29:5:
assertion `left == right` failed
  left: Ok(Canonical { text: "---\nsteplet: 1\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n---\n", signed: "---\nsteplet: 1\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n---\n", header: 4..203, body: 207..207, signature: None })
 right: Err(SignatureTwice { line: 7 })

---- the_block_is_split_off stdout ----

thread 'the_block_is_split_off' (32288) panicked at crates\steplet-core\tests\canon_signature.rs:13:5:
assertion `left == right` failed
  left: "---\nsteplet: 1\ntitle: x\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n---\nbody\n"
 right: "---\nsteplet: 1\ntitle: x\n---\nbody\n"

---- with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one stdout ----

thread 'with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one' (31352) panicked at crates\steplet-core\tests\canon_signature.rs:76:5:
assertion `left == right` failed
  left: ""
 right: "---\nsteplet: 1\ntitle: x\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n---\nbody\n"

---- block_not_last_in_the_header stdout ----

thread 'block_not_last_in_the_header' (35356) panicked at crates\steplet-core\tests\canon_signature.rs:34:5:
assertion `left == right` failed
  left: Ok(Canonical { text: "---\nsteplet: 1\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\ntitle: x\n---\n", signed: "---\nsteplet: 1\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\ntitle: x\n---\n", header: 4..118, body: 122..122, signature: None })
 right: Err(SignatureMisplaced { line: 3 })

---- block_in_the_body stdout ----

thread 'block_in_the_body' (23320) panicked at crates\steplet-core\tests\canon_signature.rs:39:5:
assertion `left == right` failed
  left: Ok(Canonical { text: "---\nsteplet: 1\n---\nbody\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n", signed: "---\nsteplet: 1\n---\nbody\nsignature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n", header: 4..15, body: 19..118, signature: None })
 right: Err(SignatureMisplaced { line: 6 })

---- markers_without_the_key_line stdout ----

thread 'markers_without_the_key_line' (24396) panicked at crates\steplet-core\tests\canon_signature.rs:45:5:
assertion `left == right` failed
  left: Ok(Canonical { text: "---\nsteplet: 1\n  -----BEGIN SSH SIGNATURE-----\n---\n", signed: "---\nsteplet: 1\n  -----BEGIN SSH SIGNATURE-----\n---\n", header: 4..47, body: 51..51, signature: None })
 right: Err(SignatureMisplaced { line: 3 })

---- windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned stdout ----

thread 'windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned' (33952) panicked at crates\steplet-core\tests\canon_signature.rs:24:48:
called `Option::unwrap()` on a `None` value


failures:
    block_in_the_body
    block_lines_must_be_indented_base64
    block_not_last_in_the_header
    end_marker_missing
    markers_without_the_key_line
    the_block_is_split_off
    the_key_line_must_be_exact
    two_blocks
    windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned
    with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one

test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p steplet-core --test canon_signature`
(exit code 101)
```

All 10 fail, each because no block is ever found.

**Step 3, after writing the real search and `with_signature`:**

```
> cargo test -p steplet-core --test canon --test canon_signature
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.71s
     Running tests\canon.rs (target\debug\deps\canon-0f861d96947b4089.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... ok
test every_error_has_words_and_help ... ok
test bad_utf8_names_its_line ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test exactly_one_final_newline ... ok
test the_header_opens_on_line_one_and_closes ... ok
test leading_and_inner_spaces_stay ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test windows_line_endings_become_unix_ones ... ok
test no_unicode_normalisation ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-2906a30ba1950f50.exe)

running 10 tests
test block_in_the_body ... ok
test block_not_last_in_the_header ... ok
test end_marker_missing ... ok
test block_lines_must_be_indented_base64 ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok
test the_key_line_must_be_exact ... ok
test markers_without_the_key_line ... ok
test two_blocks ... ok
test the_block_is_split_off ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

**Check before committing:**

```
> cargo test --workspace
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.28s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-38718543c69260ea.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-0f861d96947b4089.exe)

running 12 tests
test bad_utf8_names_its_line ... ok
test a_lone_carriage_return_is_refused ... ok
test exactly_one_final_newline ... ok
test every_error_has_words_and_help ... ok
test no_unicode_normalisation ... ok
test leading_and_inner_spaces_stay ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test windows_line_endings_become_unix_ones ... ok
test the_header_opens_on_line_one_and_closes ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-2906a30ba1950f50.exe)

running 10 tests
test block_in_the_body ... ok
test block_not_last_in_the_header ... ok
test end_marker_missing ... ok
test markers_without_the_key_line ... ok
test the_block_is_split_off ... ok
test the_key_line_must_be_exact ... ok
test two_blocks ... ok
test block_lines_must_be_indented_base64 ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-5a189c9e65d3ab0d.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test refuses_anything_else ... ok
test counts_days_from_1970 ... ok
test reads_and_writes_dates ... ok
test adds_days_across_months_and_years ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `release` profile [optimized] target(s) in 0.74s
(exit code 0)
```

**Verdict: done.** 12 of 12 clean-up tests and 10 of 10 signature-block tests pass; 27 tests pass in the workspace; no compiler warnings; the core still builds for `wasm32v1-none`. The plan's code compiled as written; nothing was changed from the plan.
