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

## Task 4: the header reader (a small, strict part of YAML)

**Step 1, the failing state** (the tests exist; `header.rs` holds the types, and `tree` always returns an empty list):

```
> cargo test -p steplet-core --test header_tree
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.03s
     Running tests\header_tree.rs (target\debug\deps\header_tree-e4442433822f47b1.exe)

running 4 tests
test every_refusal_has_words_and_help ... FAILED
test refusals_name_the_line ... FAILED
test values_yaml_reads_as_written ... FAILED
test maps_lists_and_values ... FAILED

failures:

---- every_refusal_has_words_and_help stdout ----

thread 'every_refusal_has_words_and_help' (18008) panicked at crates\steplet-core\tests\header_tree.rs:89:30:
called `Result::unwrap_err()` on an `Ok` value: []
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- refusals_name_the_line stdout ----

thread 'refusals_name_the_line' (44204) panicked at crates\steplet-core\tests\header_tree.rs:10:26:
called `Result::unwrap_err()` on an `Ok` value: []

---- values_yaml_reads_as_written stdout ----

thread 'values_yaml_reads_as_written' (8076) panicked at crates\steplet-core\tests\header_tree.rs:29:51:
removal index (is 0) should be < len (is 0)

---- maps_lists_and_values stdout ----

thread 'maps_lists_and_values' (28352) panicked at crates\steplet-core\tests\header_tree.rs:17:5:
assertion `left == right` failed
  left: []
 right: [Entry { key: "steplet", line: 2, value: Scalar(Scalar { text: "1", line: 2 }) }, Entry { key: "allow", line: 3, value: Map([Entry { key: "commands", line: 4, value: List([Scalar { text: "cargo test", line: 5 }, Scalar { text: "echo '#1'", line: 6 }]) }]) }, Entry { key: "title", line: 9, value: Scalar(Scalar { text: "it's", line: 9 }) }]


failures:
    every_refusal_has_words_and_help
    maps_lists_and_values
    refusals_name_the_line
    values_yaml_reads_as_written

test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p steplet-core --test header_tree`
(exit code 101)
```

All 4 fail.

**Step 3, after writing the reader:**

```
> cargo test -p steplet-core --test header_tree
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.46s
     Running tests\header_tree.rs (target\debug\deps\header_tree-e4442433822f47b1.exe)

running 4 tests
test maps_lists_and_values ... ok
test every_refusal_has_words_and_help ... ok
test refusals_name_the_line ... ok
test values_yaml_reads_as_written ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

**Check before committing:**

```
> cargo test --workspace
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.39s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-38718543c69260ea.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-0f861d96947b4089.exe)

running 12 tests
test bad_utf8_names_its_line ... ok
test a_lone_carriage_return_is_refused ... ok
test every_error_has_words_and_help ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test leading_and_inner_spaces_stay ... ok
test exactly_one_final_newline ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test windows_line_endings_become_unix_ones ... ok
test no_unicode_normalisation ... ok
test the_header_opens_on_line_one_and_closes ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-2906a30ba1950f50.exe)

running 10 tests
test block_in_the_body ... ok
test block_not_last_in_the_header ... ok
test block_lines_must_be_indented_base64 ... ok
test end_marker_missing ... ok
test markers_without_the_key_line ... ok
test the_key_line_must_be_exact ... ok
test two_blocks ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok
test the_block_is_split_off ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-5a189c9e65d3ab0d.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test refuses_anything_else ... ok
test adds_days_across_months_and_years ... ok
test counts_days_from_1970 ... ok
test reads_and_writes_dates ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-e4442433822f47b1.exe)

running 4 tests
test maps_lists_and_values ... ok
test values_yaml_reads_as_written ... ok
test every_refusal_has_words_and_help ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `release` profile [optimized] target(s) in 0.85s
(exit code 0)
```

**Verdict: done.** 4 of 4 reader tests pass; 31 tests pass in the workspace; no compiler warnings; the core still builds for `wasm32v1-none`. The plan's code compiled as written; nothing was changed from the plan.

## Task 5: the field list (`Steplet`) and the sender's example file

**Step 1, the failing state** (the tests and `examples/run-tests.steplet` exist; `header.rs` has the new types and error kinds, words and help are empty for them, and `parse` always answers "bad date"):

```
> cargo test -p steplet-core --test header_fields
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.43s
     Running tests\header_fields.rs (target\debug\deps\header_fields-c6763aa5ad6ce613.exe)

running 8 tests
test unknown_and_missing_fields ... FAILED
test optional_parts_may_be_left_out ... FAILED
test typed_values ... FAILED
test every_field_error_has_words_and_help ... FAILED
test the_version_comes_first_and_is_1 ... FAILED
test reads_every_field_with_its_line ... FAILED
test the_example_file_reads ... FAILED
test wrong_shapes ... FAILED

failures:

---- unknown_and_missing_fields stdout ----

thread 'unknown_and_missing_fields' (7128) panicked at crates\steplet-core\tests\header_fields.rs:81:5:
assertion `left == right` failed
  left: BadDate
 right: UnknownKey { key: "colour", allowed: ["steplet", "title", "sender", "expires", "repository", "steps", "allow", "never"] }

---- optional_parts_may_be_left_out stdout ----

thread 'optional_parts_may_be_left_out' (42096) panicked at crates\steplet-core\tests\header_fields.rs:64:101:
called `Result::unwrap()` on an `Err` value: ParseError { line: 1, kind: BadDate }

---- typed_values stdout ----

thread 'typed_values' (14512) panicked at crates\steplet-core\tests\header_fields.rs:107:9:
assertion `left == right` failed: http://github.com/x
  left: BadDate
 right: BadUrl

---- every_field_error_has_words_and_help stdout ----

thread 'every_field_error_has_words_and_help' (38052) panicked at crates\steplet-core\tests\header_fields.rs:134:9:
ParseError { line: 4, kind: BadDate }

---- the_version_comes_first_and_is_1 stdout ----

thread 'the_version_comes_first_and_is_1' (45712) panicked at crates\steplet-core\tests\header_fields.rs:73:5:
assertion `left == right` failed
  left: BadDate
 right: UnsupportedVersion { found: "2" }

---- reads_every_field_with_its_line stdout ----

thread 'reads_every_field_with_its_line' (15776) panicked at crates\steplet-core\tests\header_fields.rs:45:24:
called `Result::unwrap()` on an `Err` value: ParseError { line: 1, kind: BadDate }

---- the_example_file_reads stdout ----

thread 'the_example_file_reads' (42204) panicked at crates\steplet-core\tests\header_fields.rs:140:71:
called `Result::unwrap()` on an `Err` value: ParseError { line: 1, kind: BadDate }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- wrong_shapes stdout ----

thread 'wrong_shapes' (35856) panicked at crates\steplet-core\tests\header_fields.rs:91:5:
assertion `left == right` failed
  left: BadDate
 right: ExpectedText { key: "title" }


failures:
    every_field_error_has_words_and_help
    optional_parts_may_be_left_out
    reads_every_field_with_its_line
    the_example_file_reads
    the_version_comes_first_and_is_1
    typed_values
    unknown_and_missing_fields
    wrong_shapes

test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p steplet-core --test header_fields`
(exit code 101)
```

All 8 fail. (In `typed_values`, the first two checks expect "bad date" and so pass against this stub; the URL checks after them fail.)

**Step 3, after writing the field list, every core test so far** (the workspace has only this crate, so this is also the whole-workspace check):

```
> cargo test -p steplet-core
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.30s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-38718543c69260ea.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-0f861d96947b4089.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... ok
test every_error_has_words_and_help ... ok
test exactly_one_final_newline ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test leading_and_inner_spaces_stay ... ok
test no_unicode_normalisation ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test bad_utf8_names_its_line ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test the_header_opens_on_line_one_and_closes ... ok
test windows_line_endings_become_unix_ones ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-2906a30ba1950f50.exe)

running 10 tests
test block_in_the_body ... ok
test block_lines_must_be_indented_base64 ... ok
test block_not_last_in_the_header ... ok
test markers_without_the_key_line ... ok
test the_block_is_split_off ... ok
test the_key_line_must_be_exact ... ok
test two_blocks ... ok
test end_marker_missing ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-5a189c9e65d3ab0d.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test adds_days_across_months_and_years ... ok
test counts_days_from_1970 ... ok
test reads_and_writes_dates ... ok
test refuses_anything_else ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_fields.rs (target\debug\deps\header_fields-c6763aa5ad6ce613.exe)

running 8 tests
test optional_parts_may_be_left_out ... ok
test the_example_file_reads ... ok
test reads_every_field_with_its_line ... ok
test the_version_comes_first_and_is_1 ... ok
test wrong_shapes ... ok
test unknown_and_missing_fields ... ok
test every_field_error_has_words_and_help ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-e4442433822f47b1.exe)

running 4 tests
test every_refusal_has_words_and_help ... ok
test maps_lists_and_values ... ok
test values_yaml_reads_as_written ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `release` profile [optimized] target(s) in 1.00s
(exit code 0)
```

The example file is stored with Unix line endings, and Git's settings keep it that way (`git check-attr text eol -- examples/run-tests.steplet` printed `text: set` and `eol: lf`; `file` printed `ASCII text`, with no mention of CRLF).

**Verdict: done.** date 5, canon 12, canon_signature 10, header_tree 4, header_fields 8: all 39 pass; no compiler warnings; the core still builds for `wasm32v1-none`. The plan's code compiled as written; nothing was changed from the plan.

## Task 7: size limits

**Run before Task 6.** Task 6 needs download B1, which waits for the user's yes. Task 7 uses only the reader and the field list (Tasks 4 and 5), and nothing from Task 6, so it was done first. Task 6's section comes after this one.

**Step 1, the failing state** (the tests exist; `limits.rs` has the types, `check` finds nothing, and the words and help are empty):

```
> cargo test -p steplet-core --test limits
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.44s
     Running tests\limits.rs (target\debug\deps\limits-8eba36e20f711b28.exe)

running 5 tests
test a_command_of_300_characters_is_too_long_counted_in_characters ... FAILED
test domains_and_folders ... FAILED
test words_and_help ... FAILED
test one_over_each_limit_names_the_first_line_over ... FAILED
test at_the_limits_nothing_is_reported ... ok

failures:

---- a_command_of_300_characters_is_too_long_counted_in_characters stdout ----

thread 'a_command_of_300_characters_is_too_long_counted_in_characters' (2340) panicked at crates\steplet-core\tests\limits.rs:45:5:
assertion `left == right` failed
  left: []
 right: [LimitBreach { what: CommandLength, line: 10, found: 300, max: 299 }]

---- domains_and_folders stdout ----

thread 'domains_and_folders' (4748) panicked at crates\steplet-core\tests\limits.rs:66:5:
assertion `left == right` failed
  left: []
 right: [Domains, ReadFolders, WriteFolders]

---- words_and_help stdout ----

thread 'words_and_help' (31500) panicked at crates\steplet-core\tests\limits.rs:72:5:
assertion `left == right` failed
  left: ""
 right: "line 37: 31 steps; at most 30 are allowed"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- one_over_each_limit_names_the_first_line_over stdout ----

thread 'one_over_each_limit_names_the_first_line_over' (34604) panicked at crates\steplet-core\tests\limits.rs:34:5:
assertion `left == right` failed
  left: []
 right: [LimitBreach { what: Steps, line: 37, found: 31, max: 30 }]


failures:
    a_command_of_300_characters_is_too_long_counted_in_characters
    domains_and_folders
    one_over_each_limit_names_the_first_line_over
    words_and_help

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p steplet-core --test limits`
(exit code 101)
```

4 fail and `at_the_limits_nothing_is_reported` passes, as the plan predicts.

**Step 3, after writing `check`:**

```
> cargo test -p steplet-core --test limits
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.83s
     Running tests\limits.rs (target\debug\deps\limits-8eba36e20f711b28.exe)

running 5 tests
test words_and_help ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test domains_and_folders ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

**Check before committing:**

```
> cargo test --workspace
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.83s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-38718543c69260ea.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-0f861d96947b4089.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... ok
test bad_utf8_names_its_line ... ok
test leading_and_inner_spaces_stay ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test every_error_has_words_and_help ... ok
test no_unicode_normalisation ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test the_header_opens_on_line_one_and_closes ... ok
test windows_line_endings_become_unix_ones ... ok
test exactly_one_final_newline ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-2906a30ba1950f50.exe)

running 10 tests
test block_in_the_body ... ok
test block_not_last_in_the_header ... ok
test block_lines_must_be_indented_base64 ... ok
test the_key_line_must_be_exact ... ok
test markers_without_the_key_line ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test two_blocks ... ok
test the_block_is_split_off ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok
test end_marker_missing ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-5a189c9e65d3ab0d.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test adds_days_across_months_and_years ... ok
test counts_days_from_1970 ... ok
test refuses_anything_else ... ok
test reads_and_writes_dates ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_fields.rs (target\debug\deps\header_fields-c6763aa5ad6ce613.exe)

running 8 tests
test optional_parts_may_be_left_out ... ok
test reads_every_field_with_its_line ... ok
test the_example_file_reads ... ok
test the_version_comes_first_and_is_1 ... ok
test unknown_and_missing_fields ... ok
test wrong_shapes ... ok
test every_field_error_has_words_and_help ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-e4442433822f47b1.exe)

running 4 tests
test values_yaml_reads_as_written ... ok
test every_refusal_has_words_and_help ... ok
test maps_lists_and_values ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-8eba36e20f711b28.exe)

running 5 tests
test words_and_help ... ok
test domains_and_folders ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `release` profile [optimized] target(s) in 1.02s
(exit code 0)
```

**Verdict: done.** 5 of 5 limit tests pass; 44 tests pass in the workspace; no compiler warnings; the core still builds for `wasm32v1-none`. One change from the plan, layout only: the plan said "add to `limits.rs`", which would put the `use` lines below the constants; they were moved to the top of the file, where Rust code normally keeps them. The code itself is the plan's.

## Task 6: the differential test against `saphyr-parser` (yes: B1)

**B1:** the user said yes on 11 Oct 2026, in this session, after Tasks 1 to 5 and 7 were committed. This task tests code that already exists (the reader from Task 4), so the plan has no failing step for it.

**Step 3, first run** (cargo downloads B1 here):

```
> cargo test -p steplet-core --test differential
    Updating crates.io index
     Locking 44 packages to latest Rust 1.98 compatible versions
      Adding arraydeque v0.5.1
      Adding autocfg v1.5.1
      Adding bit-set v0.8.0
      Adding bit-vec v0.8.0
      Adding bitflags v2.13.2
      Adding cfg-if v1.0.5
      Adding errno v0.3.14
      Adding fastrand v2.5.0
      Adding fnv v1.0.7
      Adding getrandom v0.3.4
      Adding getrandom v0.4.3
      Adding libc v0.2.190
      Adding linux-raw-sys v0.12.1
      Adding num-traits v0.2.19
      Adding once_cell v1.21.4
      Adding ppv-lite86 v0.2.21
      Adding proc-macro2 v1.0.107
      Adding proptest v1.11.0 (available: v1.12.0)
      Adding quick-error v1.2.3
      Adding quote v1.0.47
      Adding r-efi v5.3.0
      Adding r-efi v6.0.0
      Adding rand v0.9.5
      Adding rand_chacha v0.9.0
      Adding rand_core v0.9.5
      Adding rand_xorshift v0.4.0
      Adding regex-syntax v0.8.11
      Adding rustix v1.1.5
      Adding rusty-fork v0.3.1
      Adding saphyr-parser v0.1.0
      Adding syn v2.0.119
      Adding syn v3.0.7
      Adding tempfile v3.27.0
      Adding thiserror v2.0.21
      Adding thiserror-impl v2.0.21
      Adding unarray v0.1.4
      Adding unicode-ident v1.0.26
      Adding wait-timeout v0.2.1
      Adding wasip2 v1.0.4+wasi-0.2.12
      Adding windows-link v0.2.1
      Adding windows-sys v0.61.2
      Adding wit-bindgen v0.57.1
      Adding zerocopy v0.8.62
      Adding zerocopy-derive v0.8.62
 Downloading crates ...
  Downloaded unarray v0.1.4
  Downloaded wait-timeout v0.2.1
  Downloaded rand_chacha v0.9.0
  Downloaded arraydeque v0.5.1
  Downloaded rand_core v0.9.5
  Downloaded rand_xorshift v0.4.0
  Downloaded rusty-fork v0.3.1
  Downloaded rand v0.9.5
  Downloaded saphyr-parser v0.1.0
  Downloaded proptest v1.11.0
  Downloaded syn v3.0.7
  Downloaded zerocopy v0.8.62
   Compiling cfg-if v1.0.5
   Compiling proc-macro2 v1.0.107
   Compiling getrandom v0.3.4
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.26
   Compiling getrandom v0.4.3
   Compiling zerocopy v0.8.62
   Compiling windows-link v0.2.1
   Compiling autocfg v1.5.1
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling thiserror v2.0.21
   Compiling wait-timeout v0.2.1
   Compiling quick-error v1.2.3
   Compiling bit-vec v0.8.0
   Compiling fnv v1.0.7
   Compiling windows-sys v0.61.2
   Compiling bit-set v0.8.0
   Compiling bitflags v2.13.2
   Compiling regex-syntax v0.8.11
   Compiling arraydeque v0.5.1
   Compiling unarray v0.1.4
   Compiling num-traits v0.2.19
   Compiling tempfile v3.27.0
   Compiling rand_core v0.9.5
   Compiling rand v0.9.5
   Compiling rand_xorshift v0.4.0
   Compiling rusty-fork v0.3.1
   Compiling syn v3.0.7
   Compiling thiserror-impl v2.0.21
   Compiling ppv-lite86 v0.2.21
   Compiling rand_chacha v0.9.0
   Compiling saphyr-parser v0.1.0
   Compiling proptest v1.11.0
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.98s
     Running tests\differential.rs (target\debug\deps\differential-7f13b114595f3f8a.exe)

running 3 tests
test every_accepted_corpus_header_reads_the_same ... ok
test whatever_we_accept_saphyr_reads_the_same ... ok
test anything_quoted_is_accepted_and_agrees ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.35s
(exit code 0)
```

3 of 3 pass on the first run: no disagreement between our reader and `saphyr-parser` was found, so `CORPUS` and the reader are unchanged. `saphyr-parser`'s names (`Event`, `Parser::new_from_str`, the `Event` fields) matched the plan; nothing was renamed.

**A problem found after the run: two downloaded crates broke the 14-day rule.** The plan pins `proptest` and `saphyr-parser` by hand, each at least 14 days old. Cargo then picked the newest versions of the libraries those two use. Publish dates of the 12 crates cargo downloaded, read from crates.io:

```
> python -I ages.py unarray@0.1.4 wait-timeout@0.2.1 rand_chacha@0.9.0 arraydeque@0.5.1 rand_core@0.9.5 rand_xorshift@0.4.0 rusty-fork@0.3.1 rand@0.9.5 saphyr-parser@0.1.0 proptest@1.11.0 syn@3.0.7 zerocopy@0.8.62
unarray          0.1.4      published 2022-11-30   1411 days old
wait-timeout     0.2.1      published 2025-02-03    615 days old
rand_chacha      0.9.0      published 2025-01-27    622 days old
arraydeque       0.5.1      published 2023-02-05   1344 days old
rand_core        0.9.5      published 2026-01-13    271 days old
rand_xorshift    0.4.0      published 2025-01-27    622 days old
rusty-fork       0.3.1      published 2025-10-04    372 days old
rand             0.9.5      published 2026-07-11     92 days old
saphyr-parser    0.1.0      published 2026-09-19     22 days old
proptest         1.11.0     published 2026-03-24    201 days old
syn              3.0.7      published 2026-10-10      1 days old  <-- under 14 days
zerocopy         0.8.62     published 2026-10-08      3 days old  <-- under 14 days
(exit code 0)
```

Both had already been compiled by the first run. `syn` is a library that runs code on this laptop while the build happens. Where they come from:

```
> cargo tree -p steplet-core -e normal,build,dev --offline -i syn@3.0.7
syn v3.0.7
└── thiserror-impl v2.0.21 (proc-macro)
    └── thiserror v2.0.21
        └── saphyr-parser v0.1.0
            [dev-dependencies]
            └── steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
> cargo tree -p steplet-core -e normal,build,dev --offline -i zerocopy
zerocopy v0.8.62
└── ppv-lite86 v0.2.21
    └── rand_chacha v0.9.0
        └── proptest v1.11.0
            [dev-dependencies]
            └── steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
```

Older versions of both were already in cargo's cache from earlier work (`syn` 3.0.4 and 3.0.6, `zerocopy` 0.8.58). Their ages:

```
> python -I ages.py syn@3.0.4 syn@3.0.6 zerocopy@0.8.58 thiserror@2.0.21 thiserror-impl@2.0.21 thiserror@2.0.20
syn              3.0.4      published 2026-08-24     48 days old
syn              3.0.6      published 2026-09-16     25 days old
zerocopy         0.8.58     published 2026-09-24     17 days old
thiserror        2.0.21     published 2026-09-23     18 days old
thiserror-impl   2.0.21     published 2026-09-23     18 days old
thiserror        2.0.20     published 2026-08-08     64 days old
(exit code 0)
```

So the lock file was pinned to the cached older versions, with cargo told not to use the network (`--offline`), so nothing more was downloaded:

```
> cargo update -p syn@3.0.7 --precise 3.0.6 --offline
 Downgrading syn v3.0.7 -> v3.0.6
(exit code 0)
```

```
> cargo update -p zerocopy --precise 0.8.58 --offline
 Downgrading zerocopy v0.8.62 -> v0.8.58
 Downgrading zerocopy-derive v0.8.62 -> v0.8.58 (available: v0.8.62)
note: pass `--verbose` to see 1 unchanged dependencies behind latest
(exit code 0)
```

Then every crates.io package in the lock file was checked, not only the downloaded ones:

```
> python -I lock_ages.py Cargo.lock
arraydeque         0.5.1                  published 2023-02-05  1344 days old
autocfg            1.5.1                  published 2026-05-22   142 days old
bit-set            0.8.0                  published 2024-07-16   817 days old
bit-vec            0.8.0                  published 2024-07-16   817 days old
bitflags           2.13.2                 published 2026-09-10    31 days old
cfg-if             1.0.5                  published 2026-09-16    25 days old
errno              0.3.14                 published 2025-09-09   397 days old
fastrand           2.5.0                  published 2026-07-19    84 days old
fnv                1.0.7                  published 2020-05-14  2341 days old
getrandom          0.3.4                  published 2025-10-14   362 days old
getrandom          0.4.3                  published 2026-06-17   116 days old
libc               0.2.190                published 2026-10-02     9 days old  <-- under 14 days
linux-raw-sys      0.12.1                 published 2025-12-23   292 days old
num-traits         0.2.19                 published 2024-05-03   891 days old
once_cell          1.21.4                 published 2026-03-12   213 days old
ppv-lite86         0.2.21                 published 2025-03-09   581 days old
proc-macro2        1.0.107                published 2026-07-19    84 days old
proptest           1.11.0                 published 2026-03-24   201 days old
quick-error        1.2.3                  published 2020-01-02  2474 days old
quote              1.0.47                 published 2026-07-19    84 days old
r-efi              5.3.0                  published 2025-06-17   481 days old
r-efi              6.0.0                  published 2026-02-03   250 days old
rand               0.9.5                  published 2026-07-11    92 days old
rand_chacha        0.9.0                  published 2025-01-27   622 days old
rand_core          0.9.5                  published 2026-01-13   271 days old
rand_xorshift      0.4.0                  published 2025-01-27   622 days old
regex-syntax       0.8.11                 published 2026-06-09   124 days old
rustix             1.1.5                  published 2026-09-16    25 days old
rusty-fork         0.3.1                  published 2025-10-04   372 days old
saphyr-parser      0.1.0                  published 2026-09-19    22 days old
syn                2.0.119                published 2026-07-15    88 days old
syn                3.0.6                  published 2026-09-16    25 days old
tempfile           3.27.0                 published 2026-03-11   214 days old
thiserror          2.0.21                 published 2026-09-23    18 days old
thiserror-impl     2.0.21                 published 2026-09-23    18 days old
unarray            0.1.4                  published 2022-11-30  1411 days old
unicode-ident      1.0.26                 published 2026-09-17    24 days old
wait-timeout       0.2.1                  published 2025-02-03   615 days old
wasip2             1.0.4+wasi-0.2.12      published 2026-06-12   121 days old
windows-link       0.2.1                  published 2025-10-06   370 days old
windows-sys        0.61.2                 published 2025-10-06   370 days old
wit-bindgen        0.57.1                 published 2026-04-17   177 days old
zerocopy           0.8.58                 published 2026-09-24    17 days old
zerocopy-derive    0.8.58                 published 2026-09-24    17 days old
44 crates.io packages in the lock file; 1 under 14 days old
(exit code 0)
```

`libc` 0.2.190 is used only when building for Linux or macOS; on this laptop it was not downloaded or compiled. It was pinned back too, to the cached 0.2.189 (published 2026-07-21, 82 days old), so the committed lock file meets the rule on every platform:

```
> cargo update -p libc --precise 0.2.189 --offline
 Downgrading libc v0.2.190 -> v0.2.189
note: pass `--verbose` to see 3 unchanged dependencies behind latest
(exit code 0)
```

```
> python -I lock_ages.py Cargo.lock
arraydeque         0.5.1                  published 2023-02-05  1344 days old
autocfg            1.5.1                  published 2026-05-22   142 days old
bit-set            0.8.0                  published 2024-07-16   817 days old
bit-vec            0.8.0                  published 2024-07-16   817 days old
bitflags           2.13.2                 published 2026-09-10    31 days old
cfg-if             1.0.5                  published 2026-09-16    25 days old
errno              0.3.14                 published 2025-09-09   397 days old
fastrand           2.5.0                  published 2026-07-19    84 days old
fnv                1.0.7                  published 2020-05-14  2341 days old
getrandom          0.3.4                  published 2025-10-14   362 days old
getrandom          0.4.3                  published 2026-06-17   116 days old
libc               0.2.189                published 2026-07-21    82 days old
linux-raw-sys      0.12.1                 published 2025-12-23   292 days old
num-traits         0.2.19                 published 2024-05-03   891 days old
once_cell          1.21.4                 published 2026-03-12   213 days old
ppv-lite86         0.2.21                 published 2025-03-09   581 days old
proc-macro2        1.0.107                published 2026-07-19    84 days old
proptest           1.11.0                 published 2026-03-24   201 days old
quick-error        1.2.3                  published 2020-01-02  2474 days old
quote              1.0.47                 published 2026-07-19    84 days old
r-efi              5.3.0                  published 2025-06-17   481 days old
r-efi              6.0.0                  published 2026-02-03   250 days old
rand               0.9.5                  published 2026-07-11    92 days old
rand_chacha        0.9.0                  published 2025-01-27   622 days old
rand_core          0.9.5                  published 2026-01-13   271 days old
rand_xorshift      0.4.0                  published 2025-01-27   622 days old
regex-syntax       0.8.11                 published 2026-06-09   124 days old
rustix             1.1.5                  published 2026-09-16    25 days old
rusty-fork         0.3.1                  published 2025-10-04   372 days old
saphyr-parser      0.1.0                  published 2026-09-19    22 days old
syn                2.0.119                published 2026-07-15    88 days old
syn                3.0.6                  published 2026-09-16    25 days old
tempfile           3.27.0                 published 2026-03-11   214 days old
thiserror          2.0.21                 published 2026-09-23    18 days old
thiserror-impl     2.0.21                 published 2026-09-23    18 days old
unarray            0.1.4                  published 2022-11-30  1411 days old
unicode-ident      1.0.26                 published 2026-09-17    24 days old
wait-timeout       0.2.1                  published 2025-02-03   615 days old
wasip2             1.0.4+wasi-0.2.12      published 2026-06-12   121 days old
windows-link       0.2.1                  published 2025-10-06   370 days old
windows-sys        0.61.2                 published 2025-10-06   370 days old
wit-bindgen        0.57.1                 published 2026-04-17   177 days old
zerocopy           0.8.58                 published 2026-09-24    17 days old
zerocopy-derive    0.8.58                 published 2026-09-24    17 days old
44 crates.io packages in the lock file; 0 under 14 days old
(exit code 0)
```

**The test again, with the pinned versions:**

```
> cargo test -p steplet-core --test differential --offline
   Compiling zerocopy v0.8.58
   Compiling syn v3.0.6
   Compiling thiserror-impl v2.0.21
   Compiling thiserror v2.0.21
   Compiling saphyr-parser v0.1.0
   Compiling ppv-lite86 v0.2.21
   Compiling rand_chacha v0.9.0
   Compiling proptest v1.11.0
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.18s
     Running tests\differential.rs (target\debug\deps\differential-3bb24f948dd3a47c.exe)

running 3 tests
test every_accepted_corpus_header_reads_the_same ... ok
test whatever_we_accept_saphyr_reads_the_same ... ok
test anything_quoted_is_accepted_and_agrees ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.26s
(exit code 0)
```

**Check before committing:**

```
> cargo test --workspace --offline
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.76s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-8cd4bc4b16d7439d.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-4c6e6c7e98b733b2.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test every_error_has_words_and_help ... ok
test bad_utf8_names_its_line ... ok
test the_header_opens_on_line_one_and_closes ... ok
test no_unicode_normalisation ... ok
test exactly_one_final_newline ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test windows_line_endings_become_unix_ones ... ok
test leading_and_inner_spaces_stay ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-d30cc554b5de1acd.exe)

running 10 tests
test the_block_is_split_off ... ok
test block_in_the_body ... ok
test the_key_line_must_be_exact ... ok
test block_lines_must_be_indented_base64 ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test end_marker_missing ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok
test block_not_last_in_the_header ... ok
test two_blocks ... ok
test markers_without_the_key_line ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-dbf9024d7237e3f5.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test adds_days_across_months_and_years ... ok
test counts_days_from_1970 ... ok
test reads_and_writes_dates ... ok
test refuses_anything_else ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\differential.rs (target\debug\deps\differential-3bb24f948dd3a47c.exe)

running 3 tests
test every_accepted_corpus_header_reads_the_same ... ok
test whatever_we_accept_saphyr_reads_the_same ... ok
test anything_quoted_is_accepted_and_agrees ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.24s

     Running tests\header_fields.rs (target\debug\deps\header_fields-98056bd6b17d32c2.exe)

running 8 tests
test optional_parts_may_be_left_out ... ok
test reads_every_field_with_its_line ... ok
test the_example_file_reads ... ok
test the_version_comes_first_and_is_1 ... ok
test wrong_shapes ... ok
test unknown_and_missing_fields ... ok
test every_field_error_has_words_and_help ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-c32e58672202eb9b.exe)

running 4 tests
test every_refusal_has_words_and_help ... ok
test maps_lists_and_values ... ok
test values_yaml_reads_as_written ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-a3a9da96a156aa91.exe)

running 5 tests
test words_and_help ... ok
test domains_and_folders ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release --offline
    Finished `release` profile [optimized] target(s) in 0.04s
(exit code 0)
```

**Verdict: done.** 3 of 3 differential tests pass (thousands of generated headers compared with `saphyr-parser`, 5,000 cases per property); 47 tests pass in the workspace; no compiler warnings; the core still builds for `wasm32v1-none`, and the two new libraries are test-only, so they are not part of it. All 44 crates.io packages in `Cargo.lock` are now at least 14 days old. **Not undone:** `syn` 3.0.7 and `zerocopy` 0.8.62 were compiled once, by the first run, before the pinning; they are still in cargo's cache (`%USERPROFILE%\.cargo\registry`) and in `target\`. Nothing in this repository uses them any more.

## Task 8: the signature check, and the property tests

`ssh-key` 0.6.7 was added as the core's one normal dependency. It and everything it uses were already in cargo's cache, and every cargo command from here on runs with `--offline`, so cargo could only pick versions already on this laptop: nothing was downloaded.

**Step 2, the failing state, first try: the plan's test code did not compile.**

```
> cargo test -p steplet-core --test sig --test properties --offline
     Locking 26 packages to latest Rust 1.98 compatible versions
      Adding base64ct v1.8.3
      Adding block-buffer v0.10.4
      Adding cipher v0.4.4
      Adding cpufeatures v0.2.17
      Adding crypto-common v0.1.7
      Adding curve25519-dalek v4.1.3
      Adding curve25519-dalek-derive v0.1.1
      Adding digest v0.10.7
      Adding ed25519 v2.2.3
      Adding ed25519-dalek v2.2.0
      Adding fiat-crypto v0.2.9
      Adding generic-array v0.14.7
      Adding inout v0.1.4
      Adding pem-rfc7468 v0.7.0
      Adding rand_core v0.6.4
      Adding rustc_version v0.4.1
      Adding semver v1.0.28
      Adding sha2 v0.10.9
      Adding signature v2.2.0
      Adding ssh-cipher v0.2.0
      Adding ssh-encoding v0.2.0
      Adding ssh-key v0.6.7
      Adding subtle v2.6.1
      Adding typenum v1.20.1
      Adding version_check v0.9.5
      Adding zeroize v1.9.1
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling cpufeatures v0.2.17
   Compiling semver v1.0.28
   Compiling base64ct v1.8.3
   Compiling subtle v2.6.1
   Compiling signature v2.2.0
   Compiling rand_core v0.6.4
   Compiling zeroize v1.9.1
   Compiling syn v2.0.119
   Compiling ed25519 v2.2.3
   Compiling generic-array v0.14.7
   Compiling rustc_version v0.4.1
   Compiling pem-rfc7468 v0.7.0
   Compiling curve25519-dalek v4.1.3
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling inout v0.1.4
   Compiling digest v0.10.7
   Compiling cipher v0.4.4
   Compiling sha2 v0.10.9
   Compiling ssh-encoding v0.2.0
   Compiling ssh-cipher v0.2.0
   Compiling curve25519-dalek-derive v0.1.1
   Compiling ed25519-dalek v2.2.0
   Compiling ssh-key v0.6.7
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
error: invalid format string: expected `}`, found `.`
  --> crates\steplet-core\tests\properties.rs:56:9
   |
56 |         prop_assert!(matches!(verify(&unix, NAMESPACE), SigStatus::Valid { .. }));
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `}` in format string
   |
   = note: if you intended to print `{`, you can escape it using `{{`
   = note: this error originates in the macro `::core::concat` which comes from the expansion of the macro `prop_assert` (in Nightly builds, run with -Z macro-backtrace for more info)

error: could not compile `steplet-core` (test "properties") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
(exit code 101)
```

Without a message, `prop_assert!` puts the checked expression into a format string, where `{ .. }` is read as a placeholder. Fix, in `tests/properties.rs`: that one assertion got an explicit message (`"the Unix version does not verify"`). What it checks did not change.

**Step 2, the failing state** (stub `verify` always answers "unsigned"). `--no-fail-fast` makes cargo run the second test file even after the first one fails:

```
> cargo test -p steplet-core --test sig --test properties --offline --no-fail-fast
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running tests\properties.rs (target\debug\deps\properties-bac0f9351251acff.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test windows_and_unix_line_endings_sign_the_same ... FAILED
test any_edit_to_non_whitespace_breaks_the_signature has been running for over 60 seconds
test any_edit_to_non_whitespace_breaks_the_signature ... ok

failures:

---- windows_and_unix_line_endings_sign_the_same stdout ----
proptest: FileFailurePersistence::SourceParallel set, but failed to find lib.rs or main.rs

thread 'windows_and_unix_line_endings_sign_the_same' (34412) panicked at crates\steplet-core\tests\properties.rs:39:1:
Test failed: the Unix version does not verify at crates\steplet-core\tests\properties.rs:58.
minimal failing input: f = "---\nsteplet: 1\ntitle: \"Run\"\nsender: Test\nexpires: 2026-12-31\nsteps:\n  - \"Run\"\n---\n", spaces = 0
	successes: 0
	local rejects: 0
	global rejects: 0

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    windows_and_unix_line_endings_sign_the_same

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 62.96s

error: test failed, to rerun pass `-p steplet-core --test properties`
     Running tests\sig.rs (target\debug\deps\sig-b2a9950fe3901a40.exe)

running 6 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... FAILED
test wrong_name_space_and_wrong_hash ... FAILED
test changed_body ... FAILED
test valid_with_the_keys_fingerprint ... FAILED

failures:

---- not_a_signature stdout ----

thread 'not_a_signature' (37684) panicked at crates\steplet-core\tests\sig.rs:45:5:
assertion `left == right` failed
  left: Unsigned
 right: Invalid { reason: NotASignature }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- wrong_name_space_and_wrong_hash stdout ----

thread 'wrong_name_space_and_wrong_hash' (41852) panicked at crates\steplet-core\tests\sig.rs:36:5:
assertion `left == right` failed
  left: Unsigned
 right: Invalid { reason: WrongNamespace { found: "runfile-v1" } }

---- changed_body stdout ----

thread 'changed_body' (22380) panicked at crates\steplet-core\tests\sig.rs:29:5:
assertion `left == right` failed
  left: Unsigned
 right: Invalid { reason: DoesNotVerify }

---- valid_with_the_keys_fingerprint stdout ----

thread 'valid_with_the_keys_fingerprint' (24624) panicked at crates\steplet-core\tests\sig.rs:23:5:
assertion `left == right` failed
  left: Unsigned
 right: Valid { fingerprint: "SHA256:fe85JkIjo8VPe+XqXJGH5Mau1EMFdK1OdKvJUFicyA8" }


failures:
    changed_body
    not_a_signature
    valid_with_the_keys_fingerprint
    wrong_name_space_and_wrong_hash

test result: FAILED. 2 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p steplet-core --test sig`
error: 2 targets failed:
    `-p steplet-core --test properties`
    `-p steplet-core --test sig`
(exit code 101)
```

As the plan predicts: `sig` 4 of 6 fail (`unsigned` and `reasons_in_words` pass on the stub); `properties` 1 of 3 fails. Proptest saved the stub's failing case to `tests/properties.proptest-regressions`; that file recorded a stub failure, not a real one, so it was moved to the Recycle Bin before the real code was written.

### A finding: a signature could be edited and stay valid

**Step 4, first run with the plan's `sig.rs`:**

```
> cargo test -p steplet-core --test sig --test properties --offline --no-fail-fast
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.60s
     Running tests\properties.rs (target\debug\deps\properties-bac0f9351251acff.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test any_edit_to_non_whitespace_breaks_the_signature ... FAILED
test windows_and_unix_line_endings_sign_the_same has been running for over 60 seconds
test windows_and_unix_line_endings_sign_the_same ... ok

failures:

---- any_edit_to_non_whitespace_breaks_the_signature stdout ----
proptest: FileFailurePersistence::SourceParallel set, but failed to find lib.rs or main.rs
proptest: FileFailurePersistence::SourceParallel set, but failed to find lib.rs or main.rs
proptest: Saving this and future failures in C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core\tests\properties.proptest-regressions
proptest: If this test was run on a CI system, you may wish to add the following line to your copy of the file. (You may need to create it.)
cc dfd2ad03981c841886be5a0adf7784a281d790348d2ec3a1a1b728c6339f4404

thread 'any_edit_to_non_whitespace_breaks_the_signature' (8884) panicked at crates\steplet-core\tests\properties.rs:39:1:
Test failed: still valid after an edit at byte 170:
---
steplet: 1
title: "Run Run Run"
sender: Test
expires: 2026-12-31
steps:
  - "Run tests"
  - "#1 café"
signature: |
  -----BEGIN SSH SIGNATURE-----
  U1NIU0lHAAAAAQAAxDMAAAALc3NoLWVkMjU1MTkAAAAg/RckOFqgx1tk+3jNYC+h2ZH96/
  drE8WO1wLqyDXp9hgAAAAKc3RlcGxldC12MQAAAAAAAAAGc2hhNTEyAAAAUwAAAAtzc2gt
  ZWQyNTUxOQAAAEDguK2WRvaDY2eIlstoGlJqMCuhzNzIT8YAS4Nhe6O3tDnN9HGgM9br7u
  PWPRFo7P5I+e3/gLEK1m+UTEDRtUMO
  -----END SSH SIGNATURE-----
---
 at crates\steplet-core\tests\properties.rs:80.
minimal failing input: f = "---\nsteplet: 1\ntitle: \"Run Run Run\"\nsender: Test\nexpires: 2026-12-31\nsteps:\n  - \"Run tests\"\n  - \"#1 café\"\n---\n", pick = Index(
    6527309441466456726,
), op = 0, ch = 'x'
	successes: 67
	local rejects: 0
	global rejects: 0

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    any_edit_to_non_whitespace_breaks_the_signature

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 103.23s

error: test failed, to rerun pass `-p steplet-core --test properties`
     Running tests\sig.rs (target\debug\deps\sig-b2a9950fe3901a40.exe)

running 6 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

error: 1 target failed:
    `-p steplet-core --test properties`
(exit code 101)
```

The third property found a file that still counts as valid after one character was changed. The changed character is inside the signature block, not in the signed text: the 17th base64 character of the first signature line, `A`, became `x`. (The `cafÃ©` above is how PowerShell displayed `café` when the output was first shown; the saved bytes are UTF-8 `caf\xc3\xa9`.)

**Why.** Every Ed25519 signature starts with the same bytes; those characters encode the length written before the public key (`00 00 00 33`, 51 bytes). After the edit the length reads 50,227. `ssh-key` reads a length-prefixed field like this (cached source, `ssh-encoding` 0.2.0, `src/reader.rs`):

```
36:    fn read_prefixed<'r, T, E, F>(&'r mut self, f: F) -> core::result::Result<T, E>
...
41:        let len = usize::decode(self)?;
42:
43:        f(&mut NestedReader {
44:            inner: self,
45:            remaining_len: len,
46:        })
47:    }
```

It never checks that the field used exactly `len` bytes. So the 51-byte key is read, the claimed 50,227 is ignored, and the rest of the signature reads as before. The same holds for the length before the signature bytes. The signed text cannot be changed this way, but one signed file has many different texts that all pass.

**A test that reproduces it,** added to `tests/sig.rs` (`an_edited_length_inside_the_signature_is_refused`), before any fix:

```
> cargo test -p steplet-core --test sig --offline
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.40s
     Running tests\sig.rs (target\debug\deps\sig-b2a9950fe3901a40.exe)

running 7 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... ok
test changed_body ... ok
test an_edited_length_inside_the_signature_is_refused ... FAILED
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

failures:

---- an_edited_length_inside_the_signature_is_refused stdout ----

thread 'an_edited_length_inside_the_signature_is_refused' (7408) panicked at crates\steplet-core\tests\sig.rs:58:5:
assertion `left == right` failed
  left: Valid { fingerprint: "SHA256:fe85JkIjo8VPe+XqXJGH5Mau1EMFdK1OdKvJUFicyA8" }
 right: Invalid { reason: DoesNotVerify }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    an_edited_length_inside_the_signature_is_refused

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p steplet-core --test sig`
(exit code 101)
```

**What OpenSSH does with the same two signatures.** A throwaway test file (moved to the Recycle Bin straight after, not committed) wrote the signed text and both signatures to a scratch folder; then Windows' own `ssh-keygen` checked each:

```
> C:\Windows\System32\OpenSSH\ssh-keygen.exe -Y check-novalidate -n steplet-v1 -s original.sig < msg.txt
Good "steplet-v1" signature with ED25519 key SHA256:fe85JkIjo8VPe+XqXJGH5Mau1EMFdK1OdKvJUFicyA8
(exit code 0)
```

```
> C:\Windows\System32\OpenSSH\ssh-keygen.exe -Y check-novalidate -n steplet-v1 -s edited.sig < msg.txt
sshsig_peek_hashalg: parse signature object: incomplete message
Could not verify signature.
(exit code -1)
```

So before the fix, our check accepted a signature that OpenSSH refuses: two readers disagreeing, which decision row 15 forbids.

**The fix, in `src/sig.rs`.** After the version, name space, hash and key-type checks, `verify` writes the signature it read back out (`to_pem`) and requires that text to equal the block in the file exactly; otherwise the file is refused (first as `DoesNotVerify`; after the review, as its own reason `NotCanonical`, below). This refuses every non-standard encoding, not only this one. It sits after the key-type check, so an RSA signature still gets `WrongKeyType`, as Task 9 expects. The test was not narrowed.

**After this first version of the fix:**

```
> cargo test -p steplet-core --test sig --test properties --offline --no-fail-fast
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.26s
     Running tests\properties.rs (target\debug\deps\properties-bac0f9351251acff.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test any_edit_to_non_whitespace_breaks_the_signature has been running for over 60 seconds
test windows_and_unix_line_endings_sign_the_same has been running for over 60 seconds
test any_edit_to_non_whitespace_breaks_the_signature ... ok
test windows_and_unix_line_endings_sign_the_same ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 112.03s

     Running tests\sig.rs (target\debug\deps\sig-b2a9950fe3901a40.exe)

running 7 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... ok
test an_edited_length_inside_the_signature_is_refused ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
(exit code 0)
```

The saved failing case (`tests/properties.proptest-regressions`, committed) is re-run first on every run from now on.

**Review before committing.** The `code-reviewer` agent checked the staged fix against the cached `ssh-key` and `ssh-encoding` source. No blockers: the check closes the hole for both length fields (before the public key and before the signature bytes), and any other non-standard text fails the comparison too. Two points to fix, both done:
1. The words were wrong for this case ("the file was changed after it was signed"). It now has its own reason, `InvalidReason::NotCanonical`: "the signature block was edited after signing; it must stay exactly as it was written".
2. Only one of the two length fields had a test. Added `an_edited_length_before_the_signature_bytes_is_refused` (length 83 changed to 339) and `re_wrapped_signature_lines_are_refused`; the first test was renamed `an_edited_length_before_the_public_key_is_refused`.

**Mutation check:** the new check switched off (`if false && …`) in a copy of `sig.rs`, the `sig` tests run, then the file restored from the backup (hash compared: identical):

```
> cargo test -p steplet-core --test sig --offline
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.39s
     Running tests\sig.rs (target\debug\deps\sig-c4623a06b1aa6458.exe)

running 9 tests
test reasons_in_words ... ok
test not_a_signature ... ok
test unsigned ... ok
test changed_body ... ok
test an_edited_length_before_the_signature_bytes_is_refused ... FAILED
test an_edited_length_before_the_public_key_is_refused ... FAILED
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok
test re_wrapped_signature_lines_are_refused ... ok

failures:

---- an_edited_length_before_the_signature_bytes_is_refused stdout ----

thread 'an_edited_length_before_the_signature_bytes_is_refused' (15224) panicked at crates\steplet-core\tests\sig.rs:89:5:
assertion `left == right` failed
  left: Valid { fingerprint: "SHA256:fe85JkIjo8VPe+XqXJGH5Mau1EMFdK1OdKvJUFicyA8" }
 right: Invalid { reason: NotCanonical }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- an_edited_length_before_the_public_key_is_refused stdout ----

thread 'an_edited_length_before_the_public_key_is_refused' (22636) panicked at crates\steplet-core\tests\sig.rs:77:5:
assertion `left == right` failed
  left: Valid { fingerprint: "SHA256:fe85JkIjo8VPe+XqXJGH5Mau1EMFdK1OdKvJUFicyA8" }
 right: Invalid { reason: NotCanonical }


failures:
    an_edited_length_before_the_public_key_is_refused
    an_edited_length_before_the_signature_bytes_is_refused

test result: FAILED. 7 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

error: test failed, to rerun pass `-p steplet-core --test sig`
(exit code 101)
```

Both length tests fail without the check (the edited signatures read as `Valid`), so both holes were real and both tests catch them. The re-wrap test passed even without the check: `ssh-key` already refuses lines that are not 70 characters while reading. So the re-wrap test now pins that exact answer (`NotASignature`), and the words of `NotCanonical` no longer mention re-wrapping.

**The `sig` tests on the final code:**

```
> cargo test -p steplet-core --test sig --offline
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running tests\sig.rs (target\debug\deps\sig-c4623a06b1aa6458.exe)

running 9 tests
test reasons_in_words ... ok
test not_a_signature ... ok
test unsigned ... ok
test an_edited_length_before_the_signature_bytes_is_refused ... ok
test re_wrapped_signature_lines_are_refused ... ok
test an_edited_length_before_the_public_key_is_refused ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
(exit code 0)
```

### One more young crate

`ssh-key` brought 26 crates into the lock file, all from the cache. Their ages:

```
> python -I lock_ages.py Cargo.lock
arraydeque         0.5.1                  published 2023-02-05  1344 days old
autocfg            1.5.1                  published 2026-05-22   142 days old
base64ct           1.8.3                  published 2026-01-12   272 days old
bit-set            0.8.0                  published 2024-07-16   817 days old
bit-vec            0.8.0                  published 2024-07-16   817 days old
bitflags           2.13.2                 published 2026-09-10    31 days old
block-buffer       0.10.4                 published 2023-03-09  1312 days old
cfg-if             1.0.5                  published 2026-09-16    25 days old
cipher             0.4.4                  published 2023-03-09  1312 days old
cpufeatures        0.2.17                 published 2025-01-25   624 days old
crypto-common      0.1.7                  published 2025-11-12   333 days old
curve25519-dalek   4.1.3                  published 2024-06-18   845 days old
curve25519-dalek-derive 0.1.1                  published 2023-10-31  1076 days old
digest             0.10.7                 published 2023-05-19  1241 days old
ed25519            2.2.3                  published 2023-10-15  1092 days old
ed25519-dalek      2.2.0                  published 2025-07-09   459 days old
errno              0.3.14                 published 2025-09-09   397 days old
fastrand           2.5.0                  published 2026-07-19    84 days old
fiat-crypto        0.2.9                  published 2024-05-12   882 days old
fnv                1.0.7                  published 2020-05-14  2341 days old
generic-array      0.14.7                 published 2023-03-28  1293 days old
getrandom          0.3.4                  published 2025-10-14   362 days old
getrandom          0.4.3                  published 2026-06-17   116 days old
inout              0.1.4                  published 2025-02-21   597 days old
libc               0.2.189                published 2026-07-21    82 days old
linux-raw-sys      0.12.1                 published 2025-12-23   292 days old
num-traits         0.2.19                 published 2024-05-03   891 days old
once_cell          1.21.4                 published 2026-03-12   213 days old
pem-rfc7468        0.7.0                  published 2023-02-26  1323 days old
ppv-lite86         0.2.21                 published 2025-03-09   581 days old
proc-macro2        1.0.107                published 2026-07-19    84 days old
proptest           1.11.0                 published 2026-03-24   201 days old
quick-error        1.2.3                  published 2020-01-02  2474 days old
quote              1.0.47                 published 2026-07-19    84 days old
r-efi              5.3.0                  published 2025-06-17   481 days old
r-efi              6.0.0                  published 2026-02-03   250 days old
rand               0.9.5                  published 2026-07-11    92 days old
rand_chacha        0.9.0                  published 2025-01-27   622 days old
rand_core          0.6.4                  published 2022-09-15  1487 days old
rand_core          0.9.5                  published 2026-01-13   271 days old
rand_xorshift      0.4.0                  published 2025-01-27   622 days old
regex-syntax       0.8.11                 published 2026-06-09   124 days old
rustc_version      0.4.1                  published 2024-08-28   774 days old
rustix             1.1.5                  published 2026-09-16    25 days old
rusty-fork         0.3.1                  published 2025-10-04   372 days old
saphyr-parser      0.1.0                  published 2026-09-19    22 days old
semver             1.0.28                 published 2026-04-04   190 days old
sha2               0.10.9                 published 2025-04-30   529 days old
signature          2.2.0                  published 2023-11-12  1064 days old
ssh-cipher         0.2.0                  published 2023-08-12  1156 days old
ssh-encoding       0.2.0                  published 2023-08-11  1157 days old
ssh-key            0.6.7                  published 2024-10-15   726 days old
subtle             2.6.1                  published 2024-06-24   839 days old
syn                2.0.119                published 2026-07-15    88 days old
syn                3.0.6                  published 2026-09-16    25 days old
tempfile           3.27.0                 published 2026-03-11   214 days old
thiserror          2.0.21                 published 2026-09-23    18 days old
thiserror-impl     2.0.21                 published 2026-09-23    18 days old
typenum            1.20.1                 published 2026-05-29   135 days old
unarray            0.1.4                  published 2022-11-30  1411 days old
unicode-ident      1.0.26                 published 2026-09-17    24 days old
version_check      0.9.5                  published 2024-07-25   808 days old
wait-timeout       0.2.1                  published 2025-02-03   615 days old
wasip2             1.0.4+wasi-0.2.12      published 2026-06-12   121 days old
windows-link       0.2.1                  published 2025-10-06   370 days old
windows-sys        0.61.2                 published 2025-10-06   370 days old
wit-bindgen        0.57.1                 published 2026-04-17   177 days old
zerocopy           0.8.58                 published 2026-09-24    17 days old
zerocopy-derive    0.8.58                 published 2026-09-24    17 days old
zeroize            1.9.1                  published 2026-10-06     5 days old  <-- under 14 days
70 crates.io packages in the lock file; 1 under 14 days old
(exit code 0)
```

`zeroize` 1.9.1 (5 days old) came from the cache: phase 0's `spikes/Cargo.lock` uses it. The cache also holds 1.9.0:

```
> python -I ages.py zeroize@1.9.0
zeroize          1.9.0      published 2026-06-12    121 days old
(exit code 0)
```

```
> cargo update -p zeroize --precise 1.9.0 --offline
 Downgrading zeroize v1.9.1 -> v1.9.0
note: pass `--verbose` to see 4 unchanged dependencies behind latest
(exit code 0)
```

```
> python -I lock_ages.py Cargo.lock   (last two lines)
zeroize            1.9.0                  published 2026-06-12   121 days old
70 crates.io packages in the lock file; 0 under 14 days old
```

### Check before committing (final code, final lock file)

```
> cargo test --workspace --offline
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.23s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-9cadb7b7c5e7a274.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-accea582b118f23b.exe)

running 12 tests
test bad_utf8_names_its_line ... ok
test a_lone_carriage_return_is_refused ... ok
test every_error_has_words_and_help ... ok
test exactly_one_final_newline ... ok
test windows_line_endings_become_unix_ones ... ok
test no_unicode_normalisation ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test the_header_opens_on_line_one_and_closes ... ok
test leading_and_inner_spaces_stay ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-b59b9cc975aad58c.exe)

running 10 tests
test block_in_the_body ... ok
test end_marker_missing ... ok
test block_not_last_in_the_header ... ok
test block_lines_must_be_indented_base64 ... ok
test markers_without_the_key_line ... ok
test the_block_is_split_off ... ok
test the_key_line_must_be_exact ... ok
test two_blocks ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-4eab5dc7b8e842f7.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test reads_and_writes_dates ... ok
test counts_days_from_1970 ... ok
test adds_days_across_months_and_years ... ok
test refuses_anything_else ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\differential.rs (target\debug\deps\differential-7eb12dda76371c10.exe)

running 3 tests
test every_accepted_corpus_header_reads_the_same ... ok
test whatever_we_accept_saphyr_reads_the_same ... ok
test anything_quoted_is_accepted_and_agrees ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.27s

     Running tests\header_fields.rs (target\debug\deps\header_fields-ae1973f54db25e03.exe)

running 8 tests
test optional_parts_may_be_left_out ... ok
test reads_every_field_with_its_line ... ok
test the_example_file_reads ... ok
test the_version_comes_first_and_is_1 ... ok
test wrong_shapes ... ok
test every_field_error_has_words_and_help ... ok
test unknown_and_missing_fields ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-6b7e71545926ed04.exe)

running 4 tests
test maps_lists_and_values ... ok
test every_refusal_has_words_and_help ... ok
test values_yaml_reads_as_written ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-54439d63506cdf51.exe)

running 5 tests
test words_and_help ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test domains_and_folders ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\properties.rs (target\debug\deps\properties-711abe513802300a.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test any_edit_to_non_whitespace_breaks_the_signature has been running for over 60 seconds
test windows_and_unix_line_endings_sign_the_same has been running for over 60 seconds
test any_edit_to_non_whitespace_breaks_the_signature ... ok
test windows_and_unix_line_endings_sign_the_same ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 122.14s

     Running tests\sig.rs (target\debug\deps\sig-c4623a06b1aa6458.exe)

running 9 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... ok
test an_edited_length_before_the_public_key_is_refused ... ok
test re_wrapped_signature_lines_are_refused ... ok
test an_edited_length_before_the_signature_bytes_is_refused ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release --offline
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `release` profile [optimized] target(s) in 1.03s
(exit code 0)
```

**Verdict: done, with one security fix beyond the plan.** `sig` 9 of 9 (the plan's 6, plus 3 for the finding); `properties` 3 of 3; 59 tests pass in the workspace; no compiler warnings; the core, now with `ssh-key` and the new check, builds for `wasm32v1-none`. All 70 crates.io packages in `Cargo.lock` are at least 14 days old. Changes from the plan:
1. `tests/properties.rs`: an explicit message on one `prop_assert!`, so it compiles.
2. `src/sig.rs`: the exact re-encoding check above, and a new reason `InvalidReason::NotCanonical`. **For the user to look at:** this makes Steplet stricter than `ssh-key` alone, and as strict as OpenSSH on this point. Later tasks only print reasons as words and never list them all, so the new reason does not affect them.
3. `tests/sig.rs`: three tests for the finding, and `NotCanonical` added to `reasons_in_words`.
4. `tests/properties.proptest-regressions`: committed, as proptest advises, so the found case is always re-run.
5. `Cargo.lock`: `zeroize` pinned to 1.9.0.

**Not verified here:** that the re-encoding check accepts real `ssh-keygen` signatures inside a `.steplet` file; Task 9's `valid.steplet` and `crlf.steplet` test exactly that, and must pass. The property tests take about 100 to 120 seconds per run (debug build, 2,000 signatures each).

## Task 9: test vectors from Windows' own `ssh-keygen`

`C:\dev\m1\p1` did not exist before this task; the script made it and the two throwaway keys in it (covered by approving the plan). The private keys stay there and are not committed.

**Step 2, making the vectors** (with `ssh-keygen` and Python only, without Steplet's code):

```
> python -I crates/steplet-core/tests/vectors/make_vectors.py
$ ssh-keygen -t ed25519 -N  -q -C steplet-p1-vectors -f C:\dev\m1\p1\ed25519
  exit 0: 
$ ssh-keygen -t rsa -b 3072 -N  -q -C steplet-p1-vectors -f C:\dev\m1\p1\rsa
  exit 0: 
$ ssh-keygen -Y sign -f C:\dev\m1\p1\ed25519 -n steplet-v1 -O hashalg=sha512 C:\dev\m1\p1\msg.txt
  exit 0: Signing file C:\dev\m1\p1\msg.txt

Write signature to C:\dev\m1\p1\msg.txt.sig
$ ssh-keygen -Y check-novalidate -n steplet-v1 -s C:\dev\m1\p1\msg.txt.sig
  exit 0: Good "steplet-v1" signature with ED25519 key SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk
$ ssh-keygen -Y sign -f C:\dev\m1\p1\ed25519 -n runfile-v1 C:\dev\m1\p1\msg.txt
  exit 0: Signing file C:\dev\m1\p1\msg.txt

Write signature to C:\dev\m1\p1\msg.txt.sig
$ ssh-keygen -Y sign -f C:\dev\m1\p1\ed25519 -n steplet-v1 -O hashalg=sha256 C:\dev\m1\p1\msg.txt
  exit 0: Signing file C:\dev\m1\p1\msg.txt

Write signature to C:\dev\m1\p1\msg.txt.sig
$ ssh-keygen -Y sign -f C:\dev\m1\p1\rsa -n steplet-v1 C:\dev\m1\p1\msg.txt
  exit 0: Signing file C:\dev\m1\p1\msg.txt

Write signature to C:\dev\m1\p1\msg.txt.sig
$ ssh-keygen -l -f C:\dev\m1\p1\ed25519.pub
  exit 0: 256 SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk steplet-p1-vectors (ED25519)
fingerprint: SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk
(exit code 0)
```

Every `ssh-keygen` call ended with exit 0; its own check says `Good "steplet-v1" signature with ED25519 key SHA256:tVgKaRjITaHHfnNKaCKzbMxCYl3SuTLHuORBTjsI9xk`, the same fingerprint printed last. `ssh-keygen` 9.5 accepted `-O hashalg=sha256`, so the `sha256.steplet` vector stays.

`.gitattributes` still has both lines (`*.steplet text eol=lf` and `crates/steplet-core/tests/vectors/*.steplet binary`), and git applies them to the vectors:

```
> git check-attr text eol binary -- crates/steplet-core/tests/vectors/crlf.steplet crates/steplet-core/tests/vectors/valid.steplet
crates/steplet-core/tests/vectors/crlf.steplet: text: unset
crates/steplet-core/tests/vectors/crlf.steplet: eol: lf
crates/steplet-core/tests/vectors/crlf.steplet: binary: set
crates/steplet-core/tests/vectors/valid.steplet: text: unset
crates/steplet-core/tests/vectors/valid.steplet: eol: lf
crates/steplet-core/tests/vectors/valid.steplet: binary: set
(exit code 0)
```

```
> crlf.steplet as stored in git's index (git show :crates/steplet-core/tests/vectors/crlf.steplet, counted in Python)
crlf.steplet as stored in git: bytes 760 CRLF pairs 25 lone LF 0
```

**Step 4, the tests** (no failing run first, as the plan says: the code under test exists since Task 8; this task adds outside evidence):

```
> cargo test -p steplet-core --test vectors --offline
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.88s
     Running tests\vectors.rs (target\debug\deps\vectors-a6c32ae3162402c1.exe)

running 5 tests
test windows_line_endings_clean_up_to_the_same_signed_text ... ok
test missing_cut_doubled_and_moved_blocks ... ok
test the_signed_header_reads ... ok
test refused_signatures ... ok
test ssh_keygen_signatures_verify_with_the_same_fingerprint ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
(exit code 0)
```

This also settles the one open point from Task 8: the exact re-encoding check accepts real `ssh-keygen` signatures, with Unix and Windows line endings.

**Before committing,** the staged files (no private key: only `ed25519.pub` and `rsa.pub`):

```
> git status --short
A  crates/steplet-core/tests/vectors.rs
A  crates/steplet-core/tests/vectors/crlf.steplet
A  crates/steplet-core/tests/vectors/cut.steplet
A  crates/steplet-core/tests/vectors/doubled.steplet
A  crates/steplet-core/tests/vectors/ed25519.pub
A  crates/steplet-core/tests/vectors/fingerprint.txt
A  crates/steplet-core/tests/vectors/make_vectors.py
A  crates/steplet-core/tests/vectors/moved-in-header.steplet
A  crates/steplet-core/tests/vectors/moved-to-body.steplet
A  crates/steplet-core/tests/vectors/rsa.pub
A  crates/steplet-core/tests/vectors/rsa.steplet
A  crates/steplet-core/tests/vectors/sha256.steplet
A  crates/steplet-core/tests/vectors/tampered.steplet
A  crates/steplet-core/tests/vectors/unsigned.steplet
A  crates/steplet-core/tests/vectors/valid.steplet
A  crates/steplet-core/tests/vectors/wrong-namespace.steplet
(exit code 0)
```

```
> cargo test --workspace --offline
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.13s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-9cadb7b7c5e7a274.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-accea582b118f23b.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... ok
test bad_utf8_names_its_line ... ok
test every_error_has_words_and_help ... ok
test no_unicode_normalisation ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test windows_line_endings_become_unix_ones ... ok
test exactly_one_final_newline ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test the_header_opens_on_line_one_and_closes ... ok
test leading_and_inner_spaces_stay ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-b59b9cc975aad58c.exe)

running 10 tests
test block_in_the_body ... ok
test block_not_last_in_the_header ... ok
test markers_without_the_key_line ... ok
test two_blocks ... ok
test block_lines_must_be_indented_base64 ... ok
test the_block_is_split_off ... ok
test the_key_line_must_be_exact ... ok
test end_marker_missing ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-4eab5dc7b8e842f7.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test counts_days_from_1970 ... ok
test adds_days_across_months_and_years ... ok
test reads_and_writes_dates ... ok
test refuses_anything_else ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\differential.rs (target\debug\deps\differential-7eb12dda76371c10.exe)

running 3 tests
test every_accepted_corpus_header_reads_the_same ... ok
test whatever_we_accept_saphyr_reads_the_same ... ok
test anything_quoted_is_accepted_and_agrees ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.20s

     Running tests\header_fields.rs (target\debug\deps\header_fields-ae1973f54db25e03.exe)

running 8 tests
test optional_parts_may_be_left_out ... ok
test the_example_file_reads ... ok
test reads_every_field_with_its_line ... ok
test the_version_comes_first_and_is_1 ... ok
test wrong_shapes ... ok
test unknown_and_missing_fields ... ok
test every_field_error_has_words_and_help ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-6b7e71545926ed04.exe)

running 4 tests
test every_refusal_has_words_and_help ... ok
test maps_lists_and_values ... ok
test refusals_name_the_line ... ok
test values_yaml_reads_as_written ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-54439d63506cdf51.exe)

running 5 tests
test words_and_help ... ok
test domains_and_folders ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\properties.rs (target\debug\deps\properties-711abe513802300a.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test any_edit_to_non_whitespace_breaks_the_signature ... ok
test windows_and_unix_line_endings_sign_the_same has been running for over 60 seconds
test windows_and_unix_line_endings_sign_the_same ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 100.27s

     Running tests\sig.rs (target\debug\deps\sig-c4623a06b1aa6458.exe)

running 9 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... ok
test an_edited_length_before_the_public_key_is_refused ... ok
test re_wrapped_signature_lines_are_refused ... ok
test an_edited_length_before_the_signature_bytes_is_refused ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests\vectors.rs (target\debug\deps\vectors-a6c32ae3162402c1.exe)

running 5 tests
test windows_line_endings_clean_up_to_the_same_signed_text ... ok
test missing_cut_doubled_and_moved_blocks ... ok
test the_signed_header_reads ... ok
test refused_signatures ... ok
test ssh_keygen_signatures_verify_with_the_same_fingerprint ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

**Verdict: done.** 5 of 5 vector tests pass; 64 tests pass in the workspace; no compiler warnings. Our check agrees with Windows' `ssh-keygen` on all eleven files. Nothing was changed from the plan.

## Task 10: display pieces (invisible characters and links)

**Step 1:** `icu_properties = "=2.3.0"` added under `[dev-dependencies]`. It and its libraries were in cargo's cache; cargo ran with `--offline`, so nothing was downloaded.

**Step 2, the failing state** (stub: `segments` returns the whole text as one piece, `is_invisible` is always false):

```
> cargo test -p steplet-core --test display --offline
     Locking 20 packages to latest Rust 1.98 compatible versions
      Adding displaydoc v0.2.7
      Adding icu_collections v2.3.0
      Adding icu_locale_core v2.3.0
      Adding icu_properties v2.3.0
      Adding icu_properties_data v2.3.0
      Adding icu_provider v2.3.1
      Adding litemap v0.8.3
      Adding potential_utf v0.1.6
      Adding stable_deref_trait v1.2.1
      Adding synstructure v0.14.0
      Adding tinystr v0.8.4
      Adding utf8_iter v1.0.4
      Adding writeable v0.6.4
      Adding yoke v0.8.3
      Adding yoke-derive v0.8.4
      Adding zerofrom v0.1.8
      Adding zerofrom-derive v0.1.8
      Adding zerotrie v0.2.5
      Adding zerovec v0.11.8
      Adding zerovec-derive v0.11.6
   Compiling stable_deref_trait v1.2.1
   Compiling icu_properties_data v2.3.0
   Compiling litemap v0.8.3
   Compiling writeable v0.6.4
   Compiling utf8_iter v1.0.4
   Compiling syn v3.0.6
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
   Compiling synstructure v0.14.0
   Compiling zerovec-derive v0.11.6
   Compiling displaydoc v0.2.7
   Compiling thiserror-impl v2.0.21
   Compiling zerofrom-derive v0.1.8
   Compiling yoke-derive v0.8.4
   Compiling zerofrom v0.1.8
   Compiling thiserror v2.0.21
   Compiling yoke v0.8.3
   Compiling saphyr-parser v0.1.0
   Compiling zerovec v0.11.8
   Compiling zerotrie v0.2.5
   Compiling tinystr v0.8.4
   Compiling potential_utf v0.1.6
   Compiling icu_collections v2.3.0
   Compiling icu_locale_core v2.3.0
   Compiling icu_provider v2.3.1
   Compiling icu_properties v2.3.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.99s
     Running tests\display.rs (target\debug\deps\display-5bfa17e6a306b0e8.exe)

running 5 tests
test markdown_links_show_their_real_address ... FAILED
test bare_addresses ... FAILED
test invisible_characters_become_badges ... FAILED
test nothing_is_ever_dropped ... ok
test the_invisible_table_matches_unicode ... FAILED

failures:

---- markdown_links_show_their_real_address stdout ----

thread 'markdown_links_show_their_real_address' (12276) panicked at crates\steplet-core\tests\display.rs:36:5:
assertion `left == right` failed
  left: [Text("see [docs](https://evil.example/x) now")]
 right: [Text("see "), Link { shown: Some([Text("docs")]), real: [Text("https://evil.example/x")] }, Text(" now")]

---- bare_addresses stdout ----

thread 'bare_addresses' (42080) panicked at crates\steplet-core\tests\display.rs:53:5:
assertion `left == right` failed
  left: [Text("go to https://a.org/x.")]
 right: [Text("go to "), Link { shown: None, real: [Text("https://a.org/x")] }, Text(".")]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- invisible_characters_become_badges stdout ----

thread 'invisible_characters_become_badges' (15844) panicked at crates\steplet-core\tests\display.rs:27:5:
assertion `left == right` failed
  left: [Text("a\u{200b}b")]
 right: [Text("a"), InvisibleBadge { codepoint: 8203 }, Text("b")]

---- the_invisible_table_matches_unicode stdout ----

thread 'the_invisible_table_matches_unicode' (45792) panicked at crates\steplet-core\tests\display.rs:84:5:
141755 code points differ, first: ["U+0000 (should be true)", "U+0001 (should be true)", "U+0002 (should be true)", "U+0003 (should be true)", "U+0004 (should be true)", "U+0005 (should be true)", "U+0006 (should be true)", "U+0007 (should be true)", "U+0008 (should be true)", "U+000B (should be true)", "U+000C (should be true)", "U+000D (should be true)", "U+000E (should be true)", "U+000F (should be true)", "U+0010 (should be true)", "U+0011 (should be true)", "U+0012 (should be true)", "U+0013 (should be true)", "U+0014 (should be true)", "U+0015 (should be true)", "U+0016 (should be true)", "U+0017 (should be true)", "U+0018 (should be true)", "U+0019 (should be true)", "U+001A (should be true)", "U+001B (should be true)", "U+001C (should be true)", "U+001D (should be true)", "U+001E (should be true)", "U+001F (should be true)", "U+007F (should be true)", "U+0080 (should be true)", "U+0081 (should be true)", "U+0082 (should be true)", "U+0083 (should be true)", "U+0084 (should be true)", "U+0085 (should be true)", "U+0086 (should be true)", "U+0087 (should be true)", "U+0088 (should be true)"]


failures:
    bare_addresses
    invisible_characters_become_badges
    markdown_links_show_their_real_address
    the_invisible_table_matches_unicode

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s

error: test failed, to rerun pass `-p steplet-core --test display`
(exit code 101)
```

4 fail and `nothing_is_ever_dropped` passes, as the plan predicts.

**Step 4, after writing `display.rs`:**

```
> cargo test -p steplet-core --test display --offline
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.01s
     Running tests\display.rs (target\debug\deps\display-5bfa17e6a306b0e8.exe)

running 5 tests
test invisible_characters_become_badges ... ok
test bare_addresses ... ok
test markdown_links_show_their_real_address ... ok
test nothing_is_ever_dropped ... ok
test the_invisible_table_matches_unicode ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
(exit code 0)
```

The hand-written table of invisible characters matched `icu_properties` 2.3.0 for all 1,114,112 code points on the first run, so the table was not changed.

### Lock file: one young crate, and a withdrawn one avoided

The 20 crates `icu_properties` brought in, from the full check of the lock file:

```
> python -I lock_ages.py Cargo.lock
arraydeque         0.5.1                  published 2023-02-05  1344 days old
autocfg            1.5.1                  published 2026-05-22   142 days old
base64ct           1.8.3                  published 2026-01-12   272 days old
bit-set            0.8.0                  published 2024-07-16   817 days old
bit-vec            0.8.0                  published 2024-07-16   817 days old
bitflags           2.13.2                 published 2026-09-10    31 days old
block-buffer       0.10.4                 published 2023-03-09  1312 days old
cfg-if             1.0.5                  published 2026-09-16    25 days old
cipher             0.4.4                  published 2023-03-09  1312 days old
cpufeatures        0.2.17                 published 2025-01-25   624 days old
crypto-common      0.1.7                  published 2025-11-12   333 days old
curve25519-dalek   4.1.3                  published 2024-06-18   845 days old
curve25519-dalek-derive 0.1.1                  published 2023-10-31  1076 days old
digest             0.10.7                 published 2023-05-19  1241 days old
displaydoc         0.2.7                  published 2026-07-28    75 days old
ed25519            2.2.3                  published 2023-10-15  1092 days old
ed25519-dalek      2.2.0                  published 2025-07-09   459 days old
errno              0.3.14                 published 2025-09-09   397 days old
fastrand           2.5.0                  published 2026-07-19    84 days old
fiat-crypto        0.2.9                  published 2024-05-12   882 days old
fnv                1.0.7                  published 2020-05-14  2341 days old
generic-array      0.14.7                 published 2023-03-28  1293 days old
getrandom          0.3.4                  published 2025-10-14   362 days old
getrandom          0.4.3                  published 2026-06-17   116 days old
icu_collections    2.3.0                  published 2026-08-13    59 days old
icu_locale_core    2.3.0                  published 2026-08-13    59 days old
icu_properties     2.3.0                  published 2026-08-13    59 days old
icu_properties_data 2.3.0                  published 2026-08-13    59 days old
icu_provider       2.3.1                  published 2026-08-20    52 days old
inout              0.1.4                  published 2025-02-21   597 days old
libc               0.2.189                published 2026-07-21    82 days old
linux-raw-sys      0.12.1                 published 2025-12-23   292 days old
litemap            0.8.3                  published 2026-08-13    59 days old
num-traits         0.2.19                 published 2024-05-03   891 days old
once_cell          1.21.4                 published 2026-03-12   213 days old
pem-rfc7468        0.7.0                  published 2023-02-26  1323 days old
potential_utf      0.1.6                  published 2026-08-13    59 days old
ppv-lite86         0.2.21                 published 2025-03-09   581 days old
proc-macro2        1.0.107                published 2026-07-19    84 days old
proptest           1.11.0                 published 2026-03-24   201 days old
quick-error        1.2.3                  published 2020-01-02  2474 days old
quote              1.0.47                 published 2026-07-19    84 days old
r-efi              5.3.0                  published 2025-06-17   481 days old
r-efi              6.0.0                  published 2026-02-03   250 days old
rand               0.9.5                  published 2026-07-11    92 days old
rand_chacha        0.9.0                  published 2025-01-27   622 days old
rand_core          0.6.4                  published 2022-09-15  1487 days old
rand_core          0.9.5                  published 2026-01-13   271 days old
rand_xorshift      0.4.0                  published 2025-01-27   622 days old
regex-syntax       0.8.11                 published 2026-06-09   124 days old
rustc_version      0.4.1                  published 2024-08-28   774 days old
rustix             1.1.5                  published 2026-09-16    25 days old
rusty-fork         0.3.1                  published 2025-10-04   372 days old
saphyr-parser      0.1.0                  published 2026-09-19    22 days old
semver             1.0.28                 published 2026-04-04   190 days old
sha2               0.10.9                 published 2025-04-30   529 days old
signature          2.2.0                  published 2023-11-12  1064 days old
ssh-cipher         0.2.0                  published 2023-08-12  1156 days old
ssh-encoding       0.2.0                  published 2023-08-11  1157 days old
ssh-key            0.6.7                  published 2024-10-15   726 days old
stable_deref_trait 1.2.1                  published 2025-10-09   367 days old
subtle             2.6.1                  published 2024-06-24   839 days old
syn                2.0.119                published 2026-07-15    88 days old
syn                3.0.6                  published 2026-09-16    25 days old
synstructure       0.14.0                 published 2026-07-30    73 days old
tempfile           3.27.0                 published 2026-03-11   214 days old
thiserror          2.0.21                 published 2026-09-23    18 days old
thiserror-impl     2.0.21                 published 2026-09-23    18 days old
tinystr            0.8.4                  published 2026-08-13    59 days old
typenum            1.20.1                 published 2026-05-29   135 days old
unarray            0.1.4                  published 2022-11-30  1411 days old
unicode-ident      1.0.26                 published 2026-09-17    24 days old
utf8_iter          1.0.4                  published 2023-12-01  1045 days old
version_check      0.9.5                  published 2024-07-25   808 days old
wait-timeout       0.2.1                  published 2025-02-03   615 days old
wasip2             1.0.4+wasi-0.2.12      published 2026-06-12   121 days old
windows-link       0.2.1                  published 2025-10-06   370 days old
windows-sys        0.61.2                 published 2025-10-06   370 days old
wit-bindgen        0.57.1                 published 2026-04-17   177 days old
writeable          0.6.4                  published 2026-08-13    59 days old
yoke               0.8.3                  published 2026-06-03   130 days old
yoke-derive        0.8.4                  published 2026-09-30    11 days old  <-- under 14 days
zerocopy           0.8.58                 published 2026-09-24    17 days old
zerocopy-derive    0.8.58                 published 2026-09-24    17 days old
zerofrom           0.1.8                  published 2026-05-12   152 days old
zerofrom-derive    0.1.8                  published 2026-09-15    26 days old
zeroize            1.9.0                  published 2026-06-12   121 days old
zerotrie           0.2.5                  published 2026-08-13    59 days old
zerovec            0.11.8                 published 2026-08-19    53 days old
zerovec-derive     0.11.6                 published 2026-08-20    52 days old
90 crates.io packages in the lock file; 1 under 14 days old
(exit code 0)
```

`yoke-derive` 0.8.4 is 11 days old. The cache also holds 0.8.3 and 0.8.2:

```
> python -I ages.py yoke-derive@0.8.3 yoke-derive@0.8.2 yoke-derive@0.8.1
yoke-derive      0.8.3      published 2026-09-15     26 days old
yoke-derive      0.8.2      published 2026-04-01    193 days old
yoke-derive      0.8.1      published 2025-10-28    348 days old
(exit code 0)
```

```
> cargo update -p yoke-derive --precise 0.8.3 --offline
warning: selected package `yoke-derive@0.8.3` was yanked by the author
  |
  = help: if possible, try a compatible non-yanked version
 Downgrading yoke-derive v0.8.4 -> v0.8.3
note: pass `--verbose` to see 5 unchanged dependencies behind latest
(exit code 0)
```

Cargo warned that 0.8.3 was **yanked**: its author withdrew it, which usually means a fault. Its versions on crates.io:

```
> python -I -c "…list the last 6 versions of yoke-derive from https://crates.io/api/v1/crates/yoke-derive/versions…"
0.8.4 2026-09-30 not yanked
0.8.3 2026-09-15 yanked
0.8.2 2026-04-01 not yanked
0.8.1 2025-10-28 not yanked
0.8.0 2025-02-26 not yanked
0.7.5 2024-11-23 not yanked
```

So 0.8.2 instead (193 days old, not withdrawn, cached). It uses `synstructure` 0.13.2, also cached:

```
> cargo update -p yoke-derive --precise 0.8.2 --offline
      Adding synstructure v0.13.2
 Downgrading yoke-derive v0.8.3 -> v0.8.2
note: pass `--verbose` to see 5 unchanged dependencies behind latest
(exit code 0)
```

The lock-file check now also reports withdrawn versions. The whole lock file, after the pin:

```
> python -I lock_ages.py Cargo.lock
arraydeque         0.5.1                  published 2023-02-05  1344 days old
autocfg            1.5.1                  published 2026-05-22   142 days old
base64ct           1.8.3                  published 2026-01-12   272 days old
bit-set            0.8.0                  published 2024-07-16   817 days old
bit-vec            0.8.0                  published 2024-07-16   817 days old
bitflags           2.13.2                 published 2026-09-10    31 days old
block-buffer       0.10.4                 published 2023-03-09  1312 days old
cfg-if             1.0.5                  published 2026-09-16    25 days old
cipher             0.4.4                  published 2023-03-09  1312 days old
cpufeatures        0.2.17                 published 2025-01-25   624 days old
crypto-common      0.1.7                  published 2025-11-12   333 days old
curve25519-dalek   4.1.3                  published 2024-06-18   845 days old
curve25519-dalek-derive 0.1.1                  published 2023-10-31  1076 days old
digest             0.10.7                 published 2023-05-19  1241 days old
displaydoc         0.2.7                  published 2026-07-28    75 days old
ed25519            2.2.3                  published 2023-10-15  1092 days old
ed25519-dalek      2.2.0                  published 2025-07-09   459 days old
errno              0.3.14                 published 2025-09-09   397 days old
fastrand           2.5.0                  published 2026-07-19    84 days old
fiat-crypto        0.2.9                  published 2024-05-12   882 days old
fnv                1.0.7                  published 2020-05-14  2341 days old
generic-array      0.14.7                 published 2023-03-28  1293 days old
getrandom          0.3.4                  published 2025-10-14   362 days old
getrandom          0.4.3                  published 2026-06-17   116 days old
icu_collections    2.3.0                  published 2026-08-13    59 days old
icu_locale_core    2.3.0                  published 2026-08-13    59 days old
icu_properties     2.3.0                  published 2026-08-13    59 days old
icu_properties_data 2.3.0                  published 2026-08-13    59 days old
icu_provider       2.3.1                  published 2026-08-20    52 days old
inout              0.1.4                  published 2025-02-21   597 days old
libc               0.2.189                published 2026-07-21    82 days old
linux-raw-sys      0.12.1                 published 2025-12-23   292 days old
litemap            0.8.3                  published 2026-08-13    59 days old
num-traits         0.2.19                 published 2024-05-03   891 days old
once_cell          1.21.4                 published 2026-03-12   213 days old
pem-rfc7468        0.7.0                  published 2023-02-26  1323 days old
potential_utf      0.1.6                  published 2026-08-13    59 days old
ppv-lite86         0.2.21                 published 2025-03-09   581 days old
proc-macro2        1.0.107                published 2026-07-19    84 days old
proptest           1.11.0                 published 2026-03-24   201 days old
quick-error        1.2.3                  published 2020-01-02  2474 days old
quote              1.0.47                 published 2026-07-19    84 days old
r-efi              5.3.0                  published 2025-06-17   481 days old
r-efi              6.0.0                  published 2026-02-03   250 days old
rand               0.9.5                  published 2026-07-11    92 days old
rand_chacha        0.9.0                  published 2025-01-27   622 days old
rand_core          0.6.4                  published 2022-09-15  1487 days old
rand_core          0.9.5                  published 2026-01-13   271 days old
rand_xorshift      0.4.0                  published 2025-01-27   622 days old
regex-syntax       0.8.11                 published 2026-06-09   124 days old
rustc_version      0.4.1                  published 2024-08-28   774 days old
rustix             1.1.5                  published 2026-09-16    25 days old
rusty-fork         0.3.1                  published 2025-10-04   372 days old
saphyr-parser      0.1.0                  published 2026-09-19    22 days old
semver             1.0.28                 published 2026-04-04   190 days old
sha2               0.10.9                 published 2025-04-30   529 days old
signature          2.2.0                  published 2023-11-12  1064 days old
ssh-cipher         0.2.0                  published 2023-08-12  1156 days old
ssh-encoding       0.2.0                  published 2023-08-11  1157 days old
ssh-key            0.6.7                  published 2024-10-15   726 days old
stable_deref_trait 1.2.1                  published 2025-10-09   367 days old
subtle             2.6.1                  published 2024-06-24   839 days old
syn                2.0.119                published 2026-07-15    88 days old
syn                3.0.6                  published 2026-09-16    25 days old
synstructure       0.13.2                 published 2025-04-30   529 days old
synstructure       0.14.0                 published 2026-07-30    73 days old
tempfile           3.27.0                 published 2026-03-11   214 days old
thiserror          2.0.21                 published 2026-09-23    18 days old
thiserror-impl     2.0.21                 published 2026-09-23    18 days old
tinystr            0.8.4                  published 2026-08-13    59 days old
typenum            1.20.1                 published 2026-05-29   135 days old
unarray            0.1.4                  published 2022-11-30  1411 days old
unicode-ident      1.0.26                 published 2026-09-17    24 days old
utf8_iter          1.0.4                  published 2023-12-01  1045 days old
version_check      0.9.5                  published 2024-07-25   808 days old
wait-timeout       0.2.1                  published 2025-02-03   615 days old
wasip2             1.0.4+wasi-0.2.12      published 2026-06-12   121 days old
windows-link       0.2.1                  published 2025-10-06   370 days old
windows-sys        0.61.2                 published 2025-10-06   370 days old
wit-bindgen        0.57.1                 published 2026-04-17   177 days old
writeable          0.6.4                  published 2026-08-13    59 days old
yoke               0.8.3                  published 2026-06-03   130 days old
yoke-derive        0.8.2                  published 2026-04-01   193 days old
zerocopy           0.8.58                 published 2026-09-24    17 days old
zerocopy-derive    0.8.58                 published 2026-09-24    17 days old
zerofrom           0.1.8                  published 2026-05-12   152 days old
zerofrom-derive    0.1.8                  published 2026-09-15    26 days old
zeroize            1.9.0                  published 2026-06-12   121 days old
zerotrie           0.2.5                  published 2026-08-13    59 days old
zerovec            0.11.8                 published 2026-08-19    53 days old
zerovec-derive     0.11.6                 published 2026-08-20    52 days old
91 crates.io packages in the lock file; 0 under 14 days old; 0 yanked
(exit code 0)
```

This also covers the crates pinned in Tasks 6 and 8: none of the 91 is withdrawn, and none is under 14 days old.

### Check before committing (final code, final lock file)

```
> cargo test --workspace --offline
   Compiling syn v2.0.119
   Compiling synstructure v0.13.2
   Compiling curve25519-dalek-derive v0.1.1
   Compiling yoke-derive v0.8.2
   Compiling curve25519-dalek v4.1.3
   Compiling yoke v0.8.3
   Compiling zerovec v0.11.8
   Compiling zerotrie v0.2.5
   Compiling ed25519-dalek v2.2.0
   Compiling ssh-key v0.6.7
   Compiling tinystr v0.8.4
   Compiling potential_utf v0.1.6
   Compiling icu_collections v2.3.0
   Compiling icu_locale_core v2.3.0
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-core)
   Compiling icu_provider v2.3.1
   Compiling icu_properties v2.3.0
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.42s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-66bcd76587c676e3.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-5ef887b940c8a763.exe)

running 12 tests
test bad_utf8_names_its_line ... ok
test a_lone_carriage_return_is_refused ... ok
test every_error_has_words_and_help ... ok
test exactly_one_final_newline ... ok
test leading_and_inner_spaces_stay ... ok
test no_unicode_normalisation ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test windows_line_endings_become_unix_ones ... ok
test the_header_opens_on_line_one_and_closes ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-118d954e88ed4bc4.exe)

running 10 tests
test two_blocks ... ok
test block_in_the_body ... ok
test block_not_last_in_the_header ... ok
test the_block_is_split_off ... ok
test markers_without_the_key_line ... ok
test the_key_line_must_be_exact ... ok
test block_lines_must_be_indented_base64 ... ok
test end_marker_missing ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-bf07833a61cd0ef4.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test adds_days_across_months_and_years ... ok
test counts_days_from_1970 ... ok
test refuses_anything_else ... ok
test reads_and_writes_dates ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\differential.rs (target\debug\deps\differential-7474fcedf0d59ba4.exe)

running 3 tests
test every_accepted_corpus_header_reads_the_same ... ok
test whatever_we_accept_saphyr_reads_the_same ... ok
test anything_quoted_is_accepted_and_agrees ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.45s

     Running tests\display.rs (target\debug\deps\display-a78ff8cda0f19313.exe)

running 5 tests
test bare_addresses ... ok
test invisible_characters_become_badges ... ok
test markdown_links_show_their_real_address ... ok
test nothing_is_ever_dropped ... ok
test the_invisible_table_matches_unicode ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s

     Running tests\header_fields.rs (target\debug\deps\header_fields-ec969918d7875e33.exe)

running 8 tests
test reads_every_field_with_its_line ... ok
test optional_parts_may_be_left_out ... ok
test the_version_comes_first_and_is_1 ... ok
test the_example_file_reads ... ok
test wrong_shapes ... ok
test unknown_and_missing_fields ... ok
test every_field_error_has_words_and_help ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-9529288ab4b5c7ab.exe)

running 4 tests
test maps_lists_and_values ... ok
test values_yaml_reads_as_written ... ok
test every_refusal_has_words_and_help ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-5e5fc15be066c18a.exe)

running 5 tests
test words_and_help ... ok
test domains_and_folders ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\properties.rs (target\debug\deps\properties-d192c70b6252f6c0.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test any_edit_to_non_whitespace_breaks_the_signature has been running for over 60 seconds
test windows_and_unix_line_endings_sign_the_same has been running for over 60 seconds
test any_edit_to_non_whitespace_breaks_the_signature ... ok
test windows_and_unix_line_endings_sign_the_same ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 113.57s

     Running tests\sig.rs (target\debug\deps\sig-aefa57c352282fe8.exe)

running 9 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... ok
test an_edited_length_before_the_signature_bytes_is_refused ... ok
test an_edited_length_before_the_public_key_is_refused ... ok
test re_wrapped_signature_lines_are_refused ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests\vectors.rs (target\debug\deps\vectors-f9059f174805bfbc.exe)

running 5 tests
test windows_line_endings_clean_up_to_the_same_signed_text ... ok
test missing_cut_doubled_and_moved_blocks ... ok
test the_signed_header_reads ... ok
test refused_signatures ... ok
test ssh_keygen_signatures_verify_with_the_same_fingerprint ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> cargo build -p steplet-core --target wasm32v1-none --release --offline
    Finished `release` profile [optimized] target(s) in 0.18s
(exit code 0)
```

**Verdict: done.** 5 of 5 display tests pass; 69 tests pass in the workspace; no compiler warnings; the core with `display` still builds for `wasm32v1-none` (the Unicode library is test-only and not part of it). The plan's code compiled and passed as written; the only change is `yoke-derive` pinned to 0.8.2 in `Cargo.lock`.

## Task 11: `steplet-sign check`

**Step 2, the failing state** (stub `check` returns an empty report that is not ok):

```
> cargo test -p steplet-sign --test check --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.17s
     Running tests\check.rs (target\debug\deps\check-becb58ae1f5e1601.exe)

running 5 tests
test a_changed_signed_file_is_a_problem ... FAILED
test expiry_in_the_past_or_too_far_ahead ... FAILED
test a_good_unsigned_file ... FAILED
test a_tab_names_its_line_and_shows_an_example ... FAILED
test limits_and_clean_up_errors ... FAILED

failures:

---- a_changed_signed_file_is_a_problem stdout ----

thread 'a_changed_signed_file_is_a_problem' (3764) panicked at crates\steplet-sign\tests\check.rs:57:5:


---- expiry_in_the_past_or_too_far_ahead stdout ----

thread 'expiry_in_the_past_or_too_far_ahead' (4992) panicked at crates\steplet-sign\tests\check.rs:46:5:


---- a_good_unsigned_file stdout ----

thread 'a_good_unsigned_file' (16116) panicked at crates\steplet-sign\tests\check.rs:13:5:

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- a_tab_names_its_line_and_shows_an_example stdout ----

thread 'a_tab_names_its_line_and_shows_an_example' (39024) panicked at crates\steplet-sign\tests\check.rs:29:5:
assertion `left == right` failed
  left: ""
 right: "t.steplet: 1 problem\n  signature: not signed\n  problem: line 12: a tab character\n    allowed: indent with spaces, two per level; no tab characters anywhere in the header\n    for example:\n      allow:\n        commands:\n          - cargo test\n"

---- limits_and_clean_up_errors stdout ----

thread 'limits_and_clean_up_errors' (38880) panicked at crates\steplet-sign\tests\check.rs:67:5:



failures:
    a_changed_signed_file_is_a_problem
    a_good_unsigned_file
    a_tab_names_its_line_and_shows_an_example
    expiry_in_the_past_or_too_far_ahead
    limits_and_clean_up_errors

test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p steplet-sign --test check`
(exit code 101)
```

All 5 fail, as the plan predicts.

**Step 4, after writing `check.rs` and `main.rs`, first run: a linker error, not a test failure.**

```
> cargo test -p steplet-sign --test check --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
error: linking with `link.exe` failed: exit code: 1104
  |
  = note: "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.44.35207\\bin\\HostX64\\x64\\link.exe" "/NOLOGO" "C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps\\rustc04SLfR\\symbols.o" "<28 object files omitted>" "<sysroot>\\lib\\rustlib\\x86_64-pc-windows-msvc\\lib/{libtest-*,libgetopts-*,librustc_std_workspace_std-*}.rlib" "C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps/{libsteplet_sign-4fbecb01ecd3002e,libsteplet_core-e12a9e40f354bfb4,libssh_key-0f1e95c93583d61f,libed25519_dalek-2f65962830291be0,libcurve25519_dalek-85311eb48ec5504b,libed25519-05c837217e31d15c,libsignature-380f442534370788,libssh_cipher-534e4c2bac3c31c6,librand_core-d32b75db77435624,libzeroize-f3366170dcc3acbb,libsubtle-2d823eaec270a8fd,libssh_encoding-dca5edaa7d9e59b2,libsha2-21b320959b7aa6df,libcpufeatures-7c683ec64b9c4309,libcfg_if-dfda34af91a39ce5,libdigest-e80e694e4755cb1d,libblock_buffer-9672a667267db7c2,libcrypto_common-305a8e0676fed5af,libgeneric_array-3f6784c5b59eb82c,libtypenum-6d29c9342295d5b5,libpem_rfc7468-5547dd0b9ef0a608,libbase64ct-3cacfb0945ab8e47}.rlib" "<sysroot>\\lib\\rustlib\\x86_64-pc-windows-msvc\\lib/{libstd-*,libpanic_unwind-*,libcfg_if-*,libwindows_link-*,librustc_demangle-*,libstd_detect-*,libhashbrown-*,librustc_std_workspace_alloc-*,libunwind-*,librustc_std_workspace_core-*,liballoc-*,libcore-*,libcompiler_builtins-*}.rlib" "kernel32.lib" "kernel32.lib" "kernel32.lib" "kernel32.lib" "ntdll.lib" "userenv.lib" "ws2_32.lib" "dbghelp.lib" "/defaultlib:msvcrt" "/NXCOMPAT" "/OUT:C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps\\check-becb58ae1f5e1601.exe" "/OPT:REF,NOICF" "/DEBUG" "/PDBALTPATH:%_PDB%" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\intrinsic.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\liballoc.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\libcore.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\libstd.natvis"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: LINK : fatal error LNK1104: cannot open file 'C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\target\debug\deps\check-becb58ae1f5e1601.exe'
          

error: could not compile `steplet-sign` (test "check") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
(exit code 101)
```

The linker could not open the old test program to replace it (`LNK1104`). Straight after, no such program was running and the file was gone (`Get-Process` listed nothing matching; `Test-Path` on the file: not there). So the lock was brief; most likely the virus scanner reading the newly built program. The same command, run again unchanged:

```
> cargo test -p steplet-sign --test check --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.73s
     Running tests\check.rs (target\debug\deps\check-becb58ae1f5e1601.exe)

running 5 tests
test a_tab_names_its_line_and_shows_an_example ... ok
test limits_and_clean_up_errors ... ok
test a_good_unsigned_file ... ok
test expiry_in_the_past_or_too_far_ahead ... ok
test a_changed_signed_file_is_a_problem ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
(exit code 0)
```

**Trying it by hand** (the program was built first with `cargo build -q -p steplet-sign --offline`, exit 0):

```
> cargo run -q -p steplet-sign --offline -- check examples/run-tests.steplet
examples/run-tests.steplet: ok
  3 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 2 never lines
  repository: https://github.com/octocat/Hello-World.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
  expires: 2027-09-30 (in 354 days)
  signature: not signed
(exit code 0)
```

```
> cargo run -q -p steplet-sign --offline -- check crates/steplet-core/tests/vectors/tampered.steplet
crates/steplet-core/tests/vectors/tampered.steplet: 1 problem
  2 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 0 never lines
  repository: https://github.com/octocat/Hello-World.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
  expires: 2026-12-31 (in 81 days)
  problem: signature: the file was changed after it was signed, or the signature belongs to another file
    allowed: a signature made by steplet-sign over the file exactly as it is now
    for example:
      steplet-sign sign my-task.steplet --key C:\Users\you\.ssh\id_ed25519
(exit code 1)
```

```
> cargo run -q -p steplet-sign --offline -- check
usage:
  steplet-sign check <file>
  steplet-sign sign <file> --key <private key file, or its .pub when the key is in ssh-agent> [--ssh-keygen <path>]
(exit code 2)
```

All three as the plan expects: the example is `ok` with exit code 0; the tampered vector shows the signature problem with exit code 1; no file prints the usage with exit code 2.

**Check before committing.** The lock file gains only the new crate itself, no new libraries:

```
> git diff -- Cargo.lock
warning: in the working copy of 'Cargo.lock', LF will be replaced by CRLF the next time Git touches it
diff --git a/Cargo.lock b/Cargo.lock
index b863023..ce3e056 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -571,6 +571,14 @@ dependencies = [
  "ssh-key",
 ]
 
+[[package]]
+name = "steplet-sign"
+version = "0.1.0"
+dependencies = [
+ "ssh-key",
+ "steplet-core",
+]
+
 [[package]]
 name = "subtle"
 version = "2.6.1"
(exit code 0)
```

```
> cargo test --workspace --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.19s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-66bcd76587c676e3.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-5ef887b940c8a763.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... ok
test exactly_one_final_newline ... ok
test every_error_has_words_and_help ... ok
test bad_utf8_names_its_line ... ok
test leading_and_inner_spaces_stay ... ok
test no_unicode_normalisation ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test the_header_opens_on_line_one_and_closes ... ok
test windows_line_endings_become_unix_ones ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-118d954e88ed4bc4.exe)

running 10 tests
test block_in_the_body ... ok
test end_marker_missing ... ok
test the_block_is_split_off ... ok
test block_lines_must_be_indented_base64 ... ok
test markers_without_the_key_line ... ok
test block_not_last_in_the_header ... ok
test the_key_line_must_be_exact ... ok
test two_blocks ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-bf07833a61cd0ef4.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test reads_and_writes_dates ... ok
test counts_days_from_1970 ... ok
test adds_days_across_months_and_years ... ok
test refuses_anything_else ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\differential.rs (target\debug\deps\differential-7474fcedf0d59ba4.exe)

running 3 tests
test every_accepted_corpus_header_reads_the_same ... ok
test whatever_we_accept_saphyr_reads_the_same ... ok
test anything_quoted_is_accepted_and_agrees ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s

     Running tests\display.rs (target\debug\deps\display-a78ff8cda0f19313.exe)

running 5 tests
test bare_addresses ... ok
test invisible_characters_become_badges ... ok
test markdown_links_show_their_real_address ... ok
test nothing_is_ever_dropped ... ok
test the_invisible_table_matches_unicode ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests\header_fields.rs (target\debug\deps\header_fields-ec969918d7875e33.exe)

running 8 tests
test optional_parts_may_be_left_out ... ok
test the_example_file_reads ... ok
test reads_every_field_with_its_line ... ok
test the_version_comes_first_and_is_1 ... ok
test wrong_shapes ... ok
test unknown_and_missing_fields ... ok
test every_field_error_has_words_and_help ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-9529288ab4b5c7ab.exe)

running 4 tests
test maps_lists_and_values ... ok
test every_refusal_has_words_and_help ... ok
test values_yaml_reads_as_written ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-5e5fc15be066c18a.exe)

running 5 tests
test words_and_help ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test domains_and_folders ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\properties.rs (target\debug\deps\properties-d192c70b6252f6c0.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test any_edit_to_non_whitespace_breaks_the_signature ... ok
test windows_and_unix_line_endings_sign_the_same has been running for over 60 seconds
test windows_and_unix_line_endings_sign_the_same ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 108.45s

     Running tests\sig.rs (target\debug\deps\sig-aefa57c352282fe8.exe)

running 9 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... ok
test re_wrapped_signature_lines_are_refused ... ok
test an_edited_length_before_the_public_key_is_refused ... ok
test an_edited_length_before_the_signature_bytes_is_refused ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests\vectors.rs (target\debug\deps\vectors-f9059f174805bfbc.exe)

running 5 tests
test windows_line_endings_clean_up_to_the_same_signed_text ... ok
test missing_cut_doubled_and_moved_blocks ... ok
test the_signed_header_reads ... ok
test refused_signatures ... ok
test ssh_keygen_signatures_verify_with_the_same_fingerprint ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src\lib.rs (target\debug\deps\steplet_sign-4b89c9d42e35cede.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\main.rs (target\debug\deps\steplet_sign-c053de2a7e1b4390.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\check.rs (target\debug\deps\check-3dd7976f398a28b2.exe)

running 5 tests
test a_tab_names_its_line_and_shows_an_example ... ok
test a_good_unsigned_file ... ok
test limits_and_clean_up_errors ... ok
test expiry_in_the_past_or_too_far_ahead ... ok
test a_changed_signed_file_is_a_problem ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_sign

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

**Verdict: done.** 5 of 5 `check` tests pass; 74 tests pass in the workspace; no compiler warnings. `steplet-core` did not change in this task, so the `wasm32v1-none` build was not repeated. The plan's code compiled and passed as written; nothing was changed from the plan.

## Task 12: `steplet-sign sign`, and `ssh-keygen` agreeing with us (yes: B3, optional)

**Step 1, the failing state** (stub `sign` always answers "not written yet"; the `sign` command in `main.rs` is already the real one):

```
> cargo test -p steplet-sign --test cli --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.79s
     Running tests\cli.rs (target\debug\deps\cli-dd89b904ca1b9d81.exe)

running 9 tests
test wrong_use_prints_how_to_use_it ... ok
Good "steplet-v1" signature with ED25519 key SHA256:zbv/nU7iZdO0fB0DalMI70dP6/yFD8sctXvvW4+OokY
test ssh_keygen_accepts_a_signature_made_in_rust ... ok
test ssh_keygen_accepts_what_we_signed_and_what_we_cut_out ... FAILED
test header_problems_stop_signing ... FAILED
test windows_line_endings_are_written_back_as_unix_ones_and_still_verify ... FAILED
test signing_again_after_an_edit_replaces_the_signature ... FAILED
test sign_then_check_shows_ssh_keygens_fingerprint ... FAILED
test expiry_must_be_from_today_to_365_days_ahead ... ok
test an_rsa_key_is_refused_and_the_file_is_left_alone ... FAILED

failures:

---- ssh_keygen_accepts_what_we_signed_and_what_we_cut_out stdout ----

thread 'ssh_keygen_accepts_what_we_signed_and_what_we_cut_out' (45648) panicked at crates\steplet-sign\tests\cli.rs:92:5:
assertion `left == right` failed
  left: 1
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- header_problems_stop_signing stdout ----

thread 'header_problems_stop_signing' (29816) panicked at crates\steplet-sign\tests\cli.rs:171:5:
not written yet


---- windows_line_endings_are_written_back_as_unix_ones_and_still_verify stdout ----

thread 'windows_line_endings_are_written_back_as_unix_ones_and_still_verify' (28352) panicked at crates\steplet-sign\tests\cli.rs:156:5:
assertion `left == right` failed
  left: 1
 right: 0

---- signing_again_after_an_edit_replaces_the_signature stdout ----

thread 'signing_again_after_an_edit_replaces_the_signature' (10328) panicked at crates\steplet-sign\tests\cli.rs:139:5:
assertion `left == right` failed
  left: 1
 right: 0

---- sign_then_check_shows_ssh_keygens_fingerprint stdout ----

thread 'sign_then_check_shows_ssh_keygens_fingerprint' (43348) panicked at crates\steplet-sign\tests\cli.rs:78:5:
assertion `left == right` failed: not written yet

  left: 1
 right: 0

---- an_rsa_key_is_refused_and_the_file_is_left_alone stdout ----

thread 'an_rsa_key_is_refused_and_the_file_is_left_alone' (33820) panicked at crates\steplet-sign\tests\cli.rs:115:5:
not written yet



failures:
    an_rsa_key_is_refused_and_the_file_is_left_alone
    header_problems_stop_signing
    sign_then_check_shows_ssh_keygens_fingerprint
    signing_again_after_an_edit_replaces_the_signature
    ssh_keygen_accepts_what_we_signed_and_what_we_cut_out
    windows_line_endings_are_written_back_as_unix_ones_and_still_verify

test result: FAILED. 3 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.23s

error: test failed, to rerun pass `-p steplet-sign --test cli`
(exit code 101)
```

3 pass and 6 fail, the split the plan predicts. (The `Good "steplet-v1" signature` line is `ssh-keygen`'s own output in `ssh_keygen_accepts_a_signature_made_in_rust`.)

**Step 2, after writing `sign.rs`, first run: the same linker lock as in Task 11.**

```
> cargo test -p steplet-sign --test cli --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
error: linking with `link.exe` failed: exit code: 1104
  |
  = note: "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.44.35207\\bin\\HostX64\\x64\\link.exe" "/NOLOGO" "C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps\\rustcInWQoB\\symbols.o" "<47 object files omitted>" "C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps/libsteplet_sign-4fbecb01ecd3002e.rlib" "<sysroot>\\lib\\rustlib\\x86_64-pc-windows-msvc\\lib/{libtest-*,libgetopts-*,librustc_std_workspace_std-*}.rlib" "C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps/{libsteplet_core-e12a9e40f354bfb4,libssh_key-0f1e95c93583d61f,libed25519_dalek-2f65962830291be0,libcurve25519_dalek-85311eb48ec5504b,libed25519-05c837217e31d15c,libsignature-380f442534370788,libssh_cipher-534e4c2bac3c31c6,librand_core-d32b75db77435624,libzeroize-f3366170dcc3acbb,libsubtle-2d823eaec270a8fd,libssh_encoding-dca5edaa7d9e59b2,libsha2-21b320959b7aa6df,libcpufeatures-7c683ec64b9c4309,libcfg_if-dfda34af91a39ce5,libdigest-e80e694e4755cb1d,libblock_buffer-9672a667267db7c2,libcrypto_common-305a8e0676fed5af,libgeneric_array-3f6784c5b59eb82c,libtypenum-6d29c9342295d5b5,libpem_rfc7468-5547dd0b9ef0a608,libbase64ct-3cacfb0945ab8e47}.rlib" "<sysroot>\\lib\\rustlib\\x86_64-pc-windows-msvc\\lib/{libstd-*,libpanic_unwind-*,libcfg_if-*,libwindows_link-*,librustc_demangle-*,libstd_detect-*,libhashbrown-*,librustc_std_workspace_alloc-*,libunwind-*,librustc_std_workspace_core-*,liballoc-*,libcore-*,libcompiler_builtins-*}.rlib" "kernel32.lib" "kernel32.lib" "kernel32.lib" "kernel32.lib" "ntdll.lib" "userenv.lib" "ws2_32.lib" "dbghelp.lib" "/defaultlib:msvcrt" "/NXCOMPAT" "/OUT:C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps\\cli-dd89b904ca1b9d81.exe" "/OPT:REF,NOICF" "/DEBUG" "/PDBALTPATH:%_PDB%" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\intrinsic.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\liballoc.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\libcore.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\libstd.natvis"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: LINK : fatal error LNK1104: cannot open file 'C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\target\debug\deps\cli-dd89b904ca1b9d81.exe'
          

error: could not compile `steplet-sign` (test "cli") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
(exit code 101)
```

The same command, run again unchanged (before it: no `steplet-sign*` folder in `%TEMP%`):

```
> cargo test -p steplet-sign --test cli --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.71s
     Running tests\cli.rs (target\debug\deps\cli-dd89b904ca1b9d81.exe)

running 9 tests
test wrong_use_prints_how_to_use_it ... ok
Good "steplet-v1" signature with ED25519 key SHA256:zbv/nU7iZdO0fB0DalMI70dP6/yFD8sctXvvW4+OokY
test ssh_keygen_accepts_a_signature_made_in_rust ... ok
test header_problems_stop_signing ... ok
test windows_line_endings_are_written_back_as_unix_ones_and_still_verify ... ok
test expiry_must_be_from_today_to_365_days_ahead ... ok
Good "steplet-v1" signature with ED25519 key SHA256:6S61VUOOR3RospgQwrTae9lsZuY+mpWNRCM9szg3YbY
Signature verification failed: incorrect signature
Could not verify signature.
test ssh_keygen_accepts_what_we_signed_and_what_we_cut_out ... ok
test sign_then_check_shows_ssh_keygens_fingerprint ... ok
test signing_again_after_an_edit_replaces_the_signature ... ok
test an_rsa_key_is_refused_and_the_file_is_left_alone ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.39s
(exit code 0)
```

```
> (Get-ChildItem $env:TEMP -Directory -Filter "steplet-sign*").Count   (after the run)
steplet-sign folders in %TEMP% after: 0
```

9 of 9 pass. `Signature verification failed: incorrect signature` is `ssh-keygen` refusing the edited text in `ssh_keygen_accepts_what_we_signed_and_what_we_cut_out`, as that test requires. Every temporary folder (the tool's own and the tests') was removed.

### The linker lock (`LNK1104`), looked into

It came back on the whole-workspace run, each time on a different test program that already existed from an earlier build:

```
> cargo test --workspace --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
error: linking with `link.exe` failed: exit code: 1104
  |
  = note: "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.44.35207\\bin\\HostX64\\x64\\link.exe" "/NOLOGO" "C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps\\rustcd8Mk5i\\symbols.o" "<6 object files omitted>" "<sysroot>\\lib\\rustlib\\x86_64-pc-windows-msvc\\lib/{libtest-*,libgetopts-*,librustc_std_workspace_std-*}.rlib" "C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps/{libsteplet_core-bd21636b1d53d930,libssh_key-7f1632d69040e380,libed25519_dalek-3fdaa7ab15863965,libcurve25519_dalek-8c4b7d9cacf46e72,libed25519-05c837217e31d15c,libsignature-380f442534370788,libssh_cipher-534e4c2bac3c31c6,librand_core-d32b75db77435624,libzeroize-f3366170dcc3acbb,libsubtle-2d823eaec270a8fd,libssh_encoding-dca5edaa7d9e59b2,libsha2-21b320959b7aa6df,libcpufeatures-7c683ec64b9c4309,libcfg_if-dfda34af91a39ce5,libdigest-e80e694e4755cb1d,libblock_buffer-9672a667267db7c2,libcrypto_common-305a8e0676fed5af,libgeneric_array-3f6784c5b59eb82c,libtypenum-6d29c9342295d5b5,libpem_rfc7468-5547dd0b9ef0a608,libbase64ct-3cacfb0945ab8e47}.rlib" "<sysroot>\\lib\\rustlib\\x86_64-pc-windows-msvc\\lib/{libstd-*,libpanic_unwind-*,libcfg_if-*,libwindows_link-*,librustc_demangle-*,libstd_detect-*,libhashbrown-*,librustc_std_workspace_alloc-*,libunwind-*,librustc_std_workspace_core-*,liballoc-*,libcore-*,libcompiler_builtins-*}.rlib" "kernel32.lib" "kernel32.lib" "kernel32.lib" "kernel32.lib" "ntdll.lib" "userenv.lib" "ws2_32.lib" "dbghelp.lib" "/defaultlib:msvcrt" "/NXCOMPAT" "/OUT:C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase1-task1-run-e26214\\target\\debug\\deps\\steplet_sign-4b89c9d42e35cede.exe" "/OPT:REF,NOICF" "/DEBUG" "/PDBALTPATH:%_PDB%" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\intrinsic.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\liballoc.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\libcore.natvis" "/NATVIS:<sysroot>\\lib\\rustlib\\etc\\libstd.natvis"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: LINK : fatal error LNK1104: cannot open file 'C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\target\debug\deps\steplet_sign-4b89c9d42e35cede.exe'
          

error: could not compile `steplet-sign` (lib test) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
(exit code 101)
```

What was checked, straight after a failure:
- The program file named in the error did not exist (`Test-Path`: False), and no program with that name was running (`Get-Process`: nothing). So it was most likely deleted but still held open by something ("delete pending"), which stops a new file with the same name being made until the holder lets go.
- Microsoft Defender's real-time scanning is off (`Get-MpComputerStatus`: `RealTimeProtectionEnabled : False`, `AMRunningMode : Not running`).
- McAfee is installed and active (`Get-CimInstance -Namespace root/SecurityCenter2 -ClassName AntiVirusProduct`: `displayName : McAfee`, `productState : 266240`).

**Most likely cause, not proven:** McAfee checks each newly built or newly run program in `target\` and holds it briefly. Each old program fails once and builds on the next try. This is a property of this laptop, not of Steplet's code. Excluding this repository's `target` folders from McAfee's scanning would probably stop it; that is a security setting, so it is left to the user. The workaround used here: build first, trying up to three times, then run the tests once.

**Step 3, every test in the workspace:**

```
> cargo test --workspace --offline --no-run
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase1-task1-run-e26214\crates\steplet-sign)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.53s
  Executable unittests src\lib.rs (target\debug\deps\steplet_core-66bcd76587c676e3.exe)
  Executable tests\canon.rs (target\debug\deps\canon-5ef887b940c8a763.exe)
  Executable tests\canon_signature.rs (target\debug\deps\canon_signature-118d954e88ed4bc4.exe)
  Executable tests\date.rs (target\debug\deps\date-bf07833a61cd0ef4.exe)
  Executable tests\differential.rs (target\debug\deps\differential-7474fcedf0d59ba4.exe)
  Executable tests\display.rs (target\debug\deps\display-a78ff8cda0f19313.exe)
  Executable tests\header_fields.rs (target\debug\deps\header_fields-ec969918d7875e33.exe)
  Executable tests\header_tree.rs (target\debug\deps\header_tree-9529288ab4b5c7ab.exe)
  Executable tests\limits.rs (target\debug\deps\limits-5e5fc15be066c18a.exe)
  Executable tests\properties.rs (target\debug\deps\properties-d192c70b6252f6c0.exe)
  Executable tests\sig.rs (target\debug\deps\sig-aefa57c352282fe8.exe)
  Executable tests\vectors.rs (target\debug\deps\vectors-f9059f174805bfbc.exe)
  Executable unittests src\lib.rs (target\debug\deps\steplet_sign-4b89c9d42e35cede.exe)
  Executable unittests src\main.rs (target\debug\deps\steplet_sign-c053de2a7e1b4390.exe)
  Executable tests\check.rs (target\debug\deps\check-3dd7976f398a28b2.exe)
  Executable tests\cli.rs (target\debug\deps\cli-eb833adb47cb9915.exe)
(exit code 0)
```

```
> cargo test --workspace --offline
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-66bcd76587c676e3.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-5ef887b940c8a763.exe)

running 12 tests
test a_lone_carriage_return_is_refused ... ok
test exactly_one_final_newline ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test bad_utf8_names_its_line ... ok
test no_unicode_normalisation ... ok
test leading_and_inner_spaces_stay ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test every_error_has_words_and_help ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test the_header_opens_on_line_one_and_closes ... ok
test windows_line_endings_become_unix_ones ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-118d954e88ed4bc4.exe)

running 10 tests
test block_in_the_body ... ok
test markers_without_the_key_line ... ok
test block_not_last_in_the_header ... ok
test end_marker_missing ... ok
test block_lines_must_be_indented_base64 ... ok
test the_block_is_split_off ... ok
test the_key_line_must_be_exact ... ok
test two_blocks ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok
test windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\date.rs (target\debug\deps\date-bf07833a61cd0ef4.exe)

running 5 tests
test a_day_ends_at_midnight_utc ... ok
test adds_days_across_months_and_years ... ok
test counts_days_from_1970 ... ok
test reads_and_writes_dates ... ok
test refuses_anything_else ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\differential.rs (target\debug\deps\differential-7474fcedf0d59ba4.exe)

running 3 tests
test every_accepted_corpus_header_reads_the_same ... ok
test whatever_we_accept_saphyr_reads_the_same ... ok
test anything_quoted_is_accepted_and_agrees ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s

     Running tests\display.rs (target\debug\deps\display-a78ff8cda0f19313.exe)

running 5 tests
test invisible_characters_become_badges ... ok
test bare_addresses ... ok
test markdown_links_show_their_real_address ... ok
test nothing_is_ever_dropped ... ok
test the_invisible_table_matches_unicode ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.76s

     Running tests\header_fields.rs (target\debug\deps\header_fields-ec969918d7875e33.exe)

running 8 tests
test optional_parts_may_be_left_out ... ok
test the_example_file_reads ... ok
test reads_every_field_with_its_line ... ok
test the_version_comes_first_and_is_1 ... ok
test unknown_and_missing_fields ... ok
test wrong_shapes ... ok
test every_field_error_has_words_and_help ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-9529288ab4b5c7ab.exe)

running 4 tests
test maps_lists_and_values ... ok
test every_refusal_has_words_and_help ... ok
test values_yaml_reads_as_written ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-5e5fc15be066c18a.exe)

running 5 tests
test words_and_help ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test domains_and_folders ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\properties.rs (target\debug\deps\properties-d192c70b6252f6c0.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test any_edit_to_non_whitespace_breaks_the_signature ... ok
test windows_and_unix_line_endings_sign_the_same has been running for over 60 seconds
test windows_and_unix_line_endings_sign_the_same ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 107.23s

     Running tests\sig.rs (target\debug\deps\sig-aefa57c352282fe8.exe)

running 9 tests
test reasons_in_words ... ok
test unsigned ... ok
test not_a_signature ... ok
test an_edited_length_before_the_public_key_is_refused ... ok
test an_edited_length_before_the_signature_bytes_is_refused ... ok
test re_wrapped_signature_lines_are_refused ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests\vectors.rs (target\debug\deps\vectors-f9059f174805bfbc.exe)

running 5 tests
test missing_cut_doubled_and_moved_blocks ... ok
test windows_line_endings_clean_up_to_the_same_signed_text ... ok
test the_signed_header_reads ... ok
test refused_signatures ... ok
test ssh_keygen_signatures_verify_with_the_same_fingerprint ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src\lib.rs (target\debug\deps\steplet_sign-4b89c9d42e35cede.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\main.rs (target\debug\deps\steplet_sign-c053de2a7e1b4390.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\check.rs (target\debug\deps\check-3dd7976f398a28b2.exe)

running 5 tests
test a_tab_names_its_line_and_shows_an_example ... ok
test a_good_unsigned_file ... ok
test limits_and_clean_up_errors ... ok
test expiry_in_the_past_or_too_far_ahead ... ok
test a_changed_signed_file_is_a_problem ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests\cli.rs (target\debug\deps\cli-eb833adb47cb9915.exe)

running 9 tests
test wrong_use_prints_how_to_use_it ... ok
Good "steplet-v1" signature with ED25519 key SHA256:zbv/nU7iZdO0fB0DalMI70dP6/yFD8sctXvvW4+OokY
test ssh_keygen_accepts_a_signature_made_in_rust ... ok
test header_problems_stop_signing ... ok
test expiry_must_be_from_today_to_365_days_ahead ... ok
Good "steplet-v1" signature with ED25519 key SHA256:WGVgQj2fma8vwXsYDKE6k0hiquq6XNATsuPsEGJtjSA
test windows_line_endings_are_written_back_as_unix_ones_and_still_verify ... ok
Signature verification failed: incorrect signature
test sign_then_check_shows_ssh_keygens_fingerprint ... ok
Could not verify signature.
test ssh_keygen_accepts_what_we_signed_and_what_we_cut_out ... ok
test signing_again_after_an_edit_replaces_the_signature ... ok
test an_rsa_key_is_refused_and_the_file_is_left_alone ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.62s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_sign

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(exit code 0)
```

```
> (Get-ChildItem $env:TEMP -Directory -Filter "steplet-sign*").Count; git diff --stat -- Cargo.lock
steplet-sign folders in %TEMP% after: 0
```
(`git diff --stat -- Cargo.lock` printed nothing: no new libraries.)

**Step 4, B3 (optional): not done.** It needs the user to type a passphrase in their own terminal, which this session cannot do. It is listed under "not verified".

**Verdict: done, except optional B3.** 9 of 9 command-line tests pass, run against the real `steplet-sign` program and Windows' own `ssh-keygen`; 83 tests pass in the workspace; no compiler warnings. `ssh-keygen` accepts what `sign` writes, refuses it after an edit, and accepts a signature made in Rust. An RSA key, a bad expiry and a header problem each stop signing and leave the file unchanged. The plan's code compiled and passed as written; nothing was changed from the plan.
