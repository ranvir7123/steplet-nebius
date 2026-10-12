# Milestone 1, phase 2: results

Plan: `docs/plans/2026-10-12-milestone-1-phase-2.md`. Each command was run as `cmd /c "<command> > file 2>&1"` from the worktree root, so both of cargo's output streams are captured; the text under each command is that file, unchanged.

## Task 1: the choices recorded; line limits, the body's first line, space-like characters

Run 12 Oct 2026, in the worktree `.claude\worktrees\steplet-phase2-task1-07fd4c` (branch `claude/steplet-phase2-task1-07fd4c`), starting from `800c549` (the plan commit, fast-forwarded onto this fresh worktree first).

Changes from the plan while running it:
- None to the code or the tests: every file compiled and passed as written in the plan.
- `docs/decisions.md` in the working copy has Windows line endings (git's `core.autocrlf`), so the first insertion script, which looked for a Unix-ending anchor, found nothing and changed nothing; the second matched the file's own line endings. The committed file is unchanged in form (git stores it with Unix endings).
- `cargo fmt --check` was tried as an extra check and is **not installed** on the stable toolchain (`rustup component add rustfmt` would be a new download, so it was not run). Phase 1 did not use it either.

**Step 4, the failing state** (the three test files exist; `limits`, `header` and `display` are still phase 1's):

```
> cargo test -p steplet-core --offline --test limits --test body_line --test space_like
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.26
   Compiling quote v1.0.47
   Compiling cfg-if v1.0.5
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling stable_deref_trait v1.2.1
   Compiling getrandom v0.3.4
   Compiling semver v1.0.28
   Compiling cpufeatures v0.2.17
   Compiling getrandom v0.4.3
   Compiling base64ct v1.8.3
   Compiling zerocopy v0.8.58
   Compiling signature v2.2.0
   Compiling windows-link v0.2.1
   Compiling subtle v2.6.1
   Compiling autocfg v1.5.1
   Compiling icu_properties_data v2.3.0
   Compiling writeable v0.6.4
   Compiling windows-sys v0.61.2
   Compiling ed25519 v2.2.3
   Compiling rustc_version v0.4.1
   Compiling generic-array v0.14.7
   Compiling pem-rfc7468 v0.7.0
   Compiling litemap v0.8.3
   Compiling once_cell v1.21.4
   Compiling thiserror v2.0.21
   Compiling fastrand v2.5.0
   Compiling curve25519-dalek v4.1.3
   Compiling bit-vec v0.8.0
   Compiling utf8_iter v1.0.4
   Compiling num-traits v0.2.19
   Compiling rand_core v0.6.4
   Compiling quick-error v1.2.3
   Compiling fnv v1.0.7
   Compiling zeroize v1.9.0
   Compiling wait-timeout v0.2.1
   Compiling bit-set v0.8.0
   Compiling unarray v0.1.4
   Compiling arraydeque v0.5.1
   Compiling regex-syntax v0.8.11
   Compiling bitflags v2.13.2
   Compiling tempfile v3.27.0
   Compiling rand_core v0.9.5
   Compiling rand v0.9.5
   Compiling rand_xorshift v0.4.0
   Compiling rusty-fork v0.3.1
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling inout v0.1.4
   Compiling syn v3.0.6
   Compiling syn v2.0.119
   Compiling cipher v0.4.4
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ssh-encoding v0.2.0
   Compiling ssh-cipher v0.2.0
   Compiling synstructure v0.14.0
   Compiling zerovec-derive v0.11.6
   Compiling displaydoc v0.2.7
   Compiling thiserror-impl v2.0.21
   Compiling zerofrom-derive v0.1.8
   Compiling ppv-lite86 v0.2.21
   Compiling zerofrom v0.1.8
   Compiling rand_chacha v0.9.0
   Compiling synstructure v0.13.2
   Compiling proptest v1.11.0
   Compiling saphyr-parser v0.1.0
   Compiling yoke-derive v0.8.2
   Compiling curve25519-dalek-derive v0.1.1
   Compiling yoke v0.8.3
   Compiling zerovec v0.11.8
   Compiling zerotrie v0.2.5
   Compiling ed25519-dalek v2.2.0
   Compiling ssh-key v0.6.7
   Compiling tinystr v0.8.4
   Compiling potential_utf v0.1.6
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
   Compiling icu_collections v2.3.0
   Compiling icu_locale_core v2.3.0
   Compiling icu_provider v2.3.1
   Compiling icu_properties v2.3.0
error[E0609]: no field `body_line` on type `Steplet`
 --> crates\steplet-core\tests\body_line.rs:5:58
  |
5 |     parse(&canonicalize(f.as_bytes()).unwrap()).unwrap().body_line
  |                                                          ^^^^^^^^^ unknown field
  |
  = note: available fields are: `title`, `sender`, `expires`, `expires_line`, `repository` ... and 4 others

For more information about this error, try `rustc --explain E0609`.
error[E0432]: unresolved import `steplet_core::display::is_space_like`
 --> crates\steplet-core\tests\space_like.rs:3:43
  |
3 | use steplet_core::display::{is_invisible, is_space_like};
  |                                           ^^^^^^^^^^^^^ no `is_space_like` in `display`

error: could not compile `steplet-core` (test "body_line") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
For more information about this error, try `rustc --explain E0432`.
error[E0599]: no variant, associated function, or constant named `TitleLength` found for enum `Limited` in the current scope
  --> crates\steplet-core\tests\limits.rs:91:42
   |
91 |             LimitBreach { what: Limited::TitleLength, line: 3, found: 121, max: 120 },
   |                                          ^^^^^^^^^^^ variant, associated function, or constant not found in `Limited`

error[E0599]: no variant, associated function, or constant named `SenderLength` found for enum `Limited` in the current scope
  --> crates\steplet-core\tests\limits.rs:92:42
   |
92 |             LimitBreach { what: Limited::SenderLength, line: 4, found: 81, max: 80 },
   |                                          ^^^^^^^^^^^^ variant, associated function, or constant not found in `Limited`

error[E0599]: no variant, associated function, or constant named `ValueLength` found for enum `Limited` in the current scope
  --> crates\steplet-core\tests\limits.rs:93:42
   |
93 |             LimitBreach { what: Limited::ValueLength, line: 7, found: 300, max: 299 },
   |                                          ^^^^^^^^^^^ variant, associated function, or constant not found in `Limited`

error[E0599]: no variant, associated function, or constant named `ValueLength` found for enum `Limited` in the current scope
  --> crates\steplet-core\tests\limits.rs:94:42
   |
94 |             LimitBreach { what: Limited::ValueLength, line: 9, found: 300, max: 299 },
   |                                          ^^^^^^^^^^^ variant, associated function, or constant not found in `Limited`

error: could not compile `steplet-core` (test "space_like") due to 1 previous error
error[E0599]: no variant, associated function, or constant named `TitleLength` found for enum `Limited` in the current scope
   --> crates\steplet-core\tests\limits.rs:115:42
    |
115 |     let b = LimitBreach { what: Limited::TitleLength, line: 3, found: 121, max: 120 };
    |                                          ^^^^^^^^^^^ variant, associated function, or constant not found in `Limited`

error[E0599]: no variant, associated function, or constant named `ValueLength` found for enum `Limited` in the current scope
   --> crates\steplet-core\tests\limits.rs:117:42
    |
117 |     let b = LimitBreach { what: Limited::ValueLength, line: 7, found: 300, max: 299 };
    |                                          ^^^^^^^^^^^ variant, associated function, or constant not found in `Limited`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `steplet-core` (test "limits") due to 6 previous errors
```

**Step 6, after the code** (expected 8, 2 and 1 passing):

```
> cargo test -p steplet-core --offline --test limits --test body_line --test space_like
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.10s
     Running tests\body_line.rs (target\debug\deps\body_line-66210a24256bf8e2.exe)

running 2 tests
test signature_lines_count_as_lines_of_the_file ... ok
test the_body_starts_after_the_closing_line ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-5e5fc15be066c18a.exe)

running 8 tests
test words_and_help ... ok
test long_title_sender_and_lines_name_their_line ... ok
test words_for_line_limits ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test at_the_line_limits_nothing_is_reported ... ok
test domains_and_folders ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\space_like.rs (target\debug\deps\space_like-9d861dbbf56f414e.exe)

running 1 test
test space_like_is_exactly_category_zs_without_the_space ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
```

**Step 7, the whole workspace:**

```
> cargo test --workspace --offline
   Compiling steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-sign)
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.27s
     Running unittests src\lib.rs (target\debug\deps\steplet_core-66bcd76587c676e3.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\body_line.rs (target\debug\deps\body_line-66210a24256bf8e2.exe)

running 2 tests
test signature_lines_count_as_lines_of_the_file ... ok
test the_body_starts_after_the_closing_line ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon.rs (target\debug\deps\canon-5ef887b940c8a763.exe)

running 12 tests
test bad_utf8_names_its_line ... ok
test a_lone_carriage_return_is_refused ... ok
test one_byte_order_mark_is_dropped_and_only_one ... ok
test windows_line_endings_become_unix_ones ... ok
test header_and_body_are_split_at_the_first_closing_line ... ok
test no_unicode_normalisation ... ok
test every_error_has_words_and_help ... ok
test spaces_and_tabs_at_line_ends_go ... ok
test the_header_opens_on_line_one_and_closes ... ok
test exactly_one_final_newline ... ok
test leading_and_inner_spaces_stay ... ok
test size_limits ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\canon_signature.rs (target\debug\deps\canon_signature-118d954e88ed4bc4.exe)

running 10 tests
test block_in_the_body ... ok
test with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one ... ok
test block_lines_must_be_indented_base64 ... ok
test markers_without_the_key_line ... ok
test the_block_is_split_off ... ok
test end_marker_missing ... ok
test the_key_line_must_be_exact ... ok
test two_blocks ... ok
test block_not_last_in_the_header ... ok
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

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.14s

     Running tests\display.rs (target\debug\deps\display-a78ff8cda0f19313.exe)

running 5 tests
test invisible_characters_become_badges ... ok
test bare_addresses ... ok
test markdown_links_show_their_real_address ... ok
test nothing_is_ever_dropped ... ok
test the_invisible_table_matches_unicode ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s

     Running tests\header_fields.rs (target\debug\deps\header_fields-ec969918d7875e33.exe)

running 8 tests
test the_example_file_reads ... ok
test optional_parts_may_be_left_out ... ok
test reads_every_field_with_its_line ... ok
test the_version_comes_first_and_is_1 ... ok
test wrong_shapes ... ok
test unknown_and_missing_fields ... ok
test every_field_error_has_words_and_help ... ok
test typed_values ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\header_tree.rs (target\debug\deps\header_tree-9529288ab4b5c7ab.exe)

running 4 tests
test every_refusal_has_words_and_help ... ok
test maps_lists_and_values ... ok
test values_yaml_reads_as_written ... ok
test refusals_name_the_line ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\limits.rs (target\debug\deps\limits-5e5fc15be066c18a.exe)

running 8 tests
test at_the_line_limits_nothing_is_reported ... ok
test words_and_help ... ok
test long_title_sender_and_lines_name_their_line ... ok
test words_for_line_limits ... ok
test a_command_of_300_characters_is_too_long_counted_in_characters ... ok
test domains_and_folders ... ok
test one_over_each_limit_names_the_first_line_over ... ok
test at_the_limits_nothing_is_reported ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\properties.rs (target\debug\deps\properties-d192c70b6252f6c0.exe)

running 3 tests
test cleaning_up_twice_changes_nothing ... ok
test any_edit_to_non_whitespace_breaks_the_signature has been running for over 60 seconds
test windows_and_unix_line_endings_sign_the_same has been running for over 60 seconds
test any_edit_to_non_whitespace_breaks_the_signature ... ok
test windows_and_unix_line_endings_sign_the_same ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 196.61s

     Running tests\sig.rs (target\debug\deps\sig-aefa57c352282fe8.exe)

running 9 tests
test unsigned ... ok
test reasons_in_words ... ok
test not_a_signature ... ok
test an_edited_length_before_the_public_key_is_refused ... ok
test an_edited_length_before_the_signature_bytes_is_refused ... ok
test re_wrapped_signature_lines_are_refused ... ok
test changed_body ... ok
test valid_with_the_keys_fingerprint ... ok
test wrong_name_space_and_wrong_hash ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests\space_like.rs (target\debug\deps\space_like-9d861dbbf56f414e.exe)

running 1 test
test space_like_is_exactly_category_zs_without_the_space ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests\vectors.rs (target\debug\deps\vectors-f9059f174805bfbc.exe)

running 5 tests
test missing_cut_doubled_and_moved_blocks ... ok
test the_signed_header_reads ... ok
test windows_line_endings_clean_up_to_the_same_signed_text ... ok
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

running 10 tests
test wrong_use_prints_how_to_use_it ... ok
Good "steplet-v1" signature with ED25519 key SHA256:zbv/nU7iZdO0fB0DalMI70dP6/yFD8sctXvvW4+OokY
test ssh_keygen_accepts_a_signature_made_in_rust ... ok
test a_file_that_cannot_be_read_is_exit_code_2 ... ok
test header_problems_stop_signing ... ok
test expiry_must_be_from_today_to_365_days_ahead ... ok
Good "steplet-v1" signature with ED25519 key SHA256:ockyRcZXAoR/snkVFmYqZSELjTVv+AXmNgu802W9Pe4
test windows_line_endings_are_written_back_as_unix_ones_and_still_verify ... ok
test sign_then_check_shows_ssh_keygens_fingerprint ... ok
Signature verification failed: incorrect signature
Could not verify signature.
test ssh_keygen_accepts_what_we_signed_and_what_we_cut_out ... ok
test signing_again_after_an_edit_replaces_the_signature ... ok
test an_rsa_key_is_refused_and_the_file_is_left_alone ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s

   Doc-tests steplet_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests steplet_sign

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Extra, not in the plan's steps: clippy with warnings as errors** (exit code 0):

```
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Checking cfg-if v1.0.5
    Checking typenum v1.20.1
    Checking cpufeatures v0.2.17
    Checking base64ct v1.8.3
    Checking subtle v2.6.1
    Checking signature v2.2.0
    Checking stable_deref_trait v1.2.1
    Checking zeroize v1.9.0
    Checking rand_core v0.6.4
    Checking windows-link v0.2.1
    Checking writeable v0.6.4
    Checking zerocopy v0.8.58
    Checking zerofrom v0.1.8
    Checking litemap v0.8.3
    Checking once_cell v1.21.4
    Checking fastrand v2.5.0
    Checking getrandom v0.3.4
    Checking getrandom v0.4.3
    Checking ed25519 v2.2.3
    Checking utf8_iter v1.0.4
    Checking wait-timeout v0.2.1
    Checking windows-sys v0.61.2
    Checking quick-error v1.2.3
    Checking pem-rfc7468 v0.7.0
    Checking fnv v1.0.7
    Checking yoke v0.8.3
    Checking bit-vec v0.8.0
    Checking num-traits v0.2.19
    Checking thiserror v2.0.21
    Checking rand_core v0.9.5
    Checking icu_properties_data v2.3.0
    Checking bitflags v2.13.2
    Checking regex-syntax v0.8.11
    Checking arraydeque v0.5.1
    Checking unarray v0.1.4
    Checking generic-array v0.14.7
    Checking zerovec v0.11.8
    Checking zerotrie v0.2.5
    Checking bit-set v0.8.0
    Checking rand v0.9.5
    Checking rand_xorshift v0.4.0
    Checking saphyr-parser v0.1.0
    Checking tempfile v3.27.0
    Checking crypto-common v0.1.7
    Checking block-buffer v0.10.4
    Checking inout v0.1.4
    Checking tinystr v0.8.4
    Checking potential_utf v0.1.6
    Checking rusty-fork v0.3.1
    Checking cipher v0.4.4
    Checking digest v0.10.7
    Checking icu_collections v2.3.0
    Checking icu_locale_core v2.3.0
    Checking sha2 v0.10.9
    Checking curve25519-dalek v4.1.3
    Checking ssh-encoding v0.2.0
    Checking ssh-cipher v0.2.0
    Checking ed25519-dalek v2.2.0
    Checking icu_provider v2.3.1
    Checking ssh-key v0.6.7
    Checking icu_properties v2.3.0
    Checking steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Checking ppv-lite86 v0.2.21
    Checking steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-sign)
    Checking rand_chacha v0.9.0
    Checking proptest v1.11.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.82s
```

**Verdict:** done. 11 new or changed tests pass (8 in `limits`, 2 in `body_line`, 1 in `space_like`); every other test in the workspace still passes (0 failures); clippy is clean. Not verified: `rustfmt` formatting (tool not installed).

## Task 2: a command read as exact words, the same words bash makes

Run 12 Oct 2026, same worktree, starting from `c53f1ef`.

Changes from the plan while running it:
- None to the code, the tests or the script: all compiled and passed as written.
- The failing run (Step 1) made proptest write `tests/command_split.proptest-regressions`, recording the stub's failure on the input `[""]`. That file was deleted before the commit: it records a failure of the empty stub, not of `split`.
- Y2 (bash in the WSL distribution `steplet-test` splitting 2,000 made-up lines) ran as approved with the plan.

**Step 1, the failing state** (`split` is a stub that always answers `Empty`):

```
> cargo test -p steplet-core --offline --test command_split
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.27s
     Running tests\command_split.rs (target\debug\deps\command_split-2df096c8ea86e630.exe)

running 5 tests
test other_reasons ... FAILED
test shell_symbols_are_not_read ... FAILED
test quotes_make_one_word ... FAILED
test plain_words_and_spaces ... FAILED
test single_quoted_words_come_back_unchanged ... FAILED

failures:

---- other_reasons stdout ----

thread 'other_reasons' (6476) panicked at crates\steplet-core\tests\command_split.rs:59:5:
assertion `left == right` failed
  left: Err(Empty)
 right: Err(Control)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- shell_symbols_are_not_read stdout ----

thread 'shell_symbols_are_not_read' (3172) panicked at crates\steplet-core\tests\command_split.rs:51:9:
assertion `left == right` failed: cargo test | tail
  left: Err(Empty)
 right: Err(Symbol('|'))

---- quotes_make_one_word stdout ----

thread 'quotes_make_one_word' (40232) panicked at crates\steplet-core\tests\command_split.rs:18:5:
assertion `left == right` failed
  left: Err(Empty)
 right: Ok(["pytest", "-k", "a or b"])

---- plain_words_and_spaces stdout ----

thread 'plain_words_and_spaces' (33696) panicked at crates\steplet-core\tests\command_split.rs:10:5:
assertion `left == right` failed
  left: Err(Empty)
 right: Ok(["cargo", "test", "--release"])

---- single_quoted_words_come_back_unchanged stdout ----
proptest: FileFailurePersistence::SourceParallel set, but failed to find lib.rs or main.rs
proptest: FileFailurePersistence::SourceParallel set, but failed to find lib.rs or main.rs
proptest: Saving this and future failures in C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core\tests\command_split.proptest-regressions
proptest: If this test was run on a CI system, you may wish to add the following line to your copy of the file. (You may need to create it.)
cc 7e0f8aed738d6f0e8c28ff8186b1872b13f4f0d4968106da9be4e122621c0f39

thread 'single_quoted_words_come_back_unchanged' (28324) panicked at crates\steplet-core\tests\command_split.rs:68:1:
Test failed: assertion failed: `(left == right)` 
  left: `Err(Empty)`,
 right: `Ok([""])` at crates\steplet-core\tests\command_split.rs:75.
minimal failing input: words = [
    "",
]
	successes: 0
	local rejects: 0
	global rejects: 0



failures:
    other_reasons
    plain_words_and_spaces
    quotes_make_one_word
    shell_symbols_are_not_read
    single_quoted_words_come_back_unchanged

test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p steplet-core --test command_split`
```

**Step 3, after `split` is written:**

```
> cargo test -p steplet-core --offline --test command_split
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.30s
     Running tests\command_split.rs (target\debug\deps\command_split-2df096c8ea86e630.exe)

running 5 tests
test other_reasons ... ok
test plain_words_and_spaces ... ok
test quotes_make_one_word ... ok
test shell_symbols_are_not_read ... ok
test single_quoted_words_come_back_unchanged ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

**Step 4, the vectors made by bash** (the script stops if bash and the script disagree on any line; it did not):

```
> python -I crates/steplet-core/tests/vectors/make_command_vectors.py
wrote 2000 command lines to C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core\tests\vectors\commands.txt; bash made the same words for every one
```

The first five lines of `commands.txt`, with the unit separator (U+001F) shown as `␟`. Each line is the command as written, then each word bash made of it. Line 4 ends in an empty word (the closing `""`), and line 5 is one empty word:

```
 b "y8=xY"-Bc/",'B}C"  '-;~<\|'   'Az=#}'␟b␟y8=xY-Bc/,'B}C␟-;~<\|␟Az=#}
_b'_)z8''x' ␟_b_)z8x
 9zY1z"?0&"c._ zB  '#_b' 1.aB"BxYbC0"␟9zY1z?0&c._␟zB␟#_b␟1.aBBxYbC0
  "Z)"@_1C'X}:'  = "'"  "#Y,; *"'!,=c:'x1zB8 ""␟Z)@_1CX}:␟=␟'␟#Y,; *!,=c:x1zB8␟
""␟
```

`wc -l` gives `2000`; the file is 109,248 bytes; `git check-attr` shows `binary: set`, so its bytes never change on checkout.

**Step 5, with the vector test** (expected 6 passing):

```
> cargo test -p steplet-core --offline --test command_split
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.88s
     Running tests\command_split.rs (target\debug\deps\command_split-2df096c8ea86e630.exe)

running 6 tests
test other_reasons ... ok
test quotes_make_one_word ... ok
test plain_words_and_spaces ... ok
test shell_symbols_are_not_read ... ok
test split_gives_the_words_bash_gave_for_every_vector ... ok
test single_quoted_words_come_back_unchanged ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

**Extra: clippy with warnings as errors** (exit code 0, last lines):

```
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Checking steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Checking steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-sign)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.51s
```

**Verdict:** done. 6 tests pass in `command_split`, including 2,000 bash-made vectors matched word for word; clippy is clean. Not run in this task: the full workspace test (`cargo test --workspace`), since only a new module and a new test file were added; it runs again in Task 14 and at every task that touches existing code.

## Task 3: the always-blocked list, paths outside the repository, protected folders

Run 12 Oct 2026, same worktree, starting from `ea60966`.

Changes from the plan while running it: none. The code and the tests compiled and passed as written.

**Step 1, the failing state** (`paths.rs` is a stub where every function answers no):

```
> cargo test -p steplet-core --offline --test paths
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.28s
     Running tests\paths.rs (target\debug\deps\paths-1e09dc0109ff8170.exe)

running 5 tests
test in_text_only_places_outside_the_repository_count ... FAILED
test protected_folders_at_any_depth ... FAILED
test places_that_hold_keys_by_their_names ... FAILED
test paths_outside_the_repository ... FAILED
test cleaning_up_paths_as_text ... FAILED

failures:

---- in_text_only_places_outside_the_repository_count stdout ----

thread 'in_text_only_places_outside_the_repository_count' (44944) panicked at crates\steplet-core\tests\paths.rs:43:61:
called `Option::unwrap()` on a `None` value

---- protected_folders_at_any_depth stdout ----

thread 'protected_folders_at_any_depth' (30152) panicked at crates\steplet-core\tests\paths.rs:69:9:
.git/hooks/pre-commit

---- places_that_hold_keys_by_their_names stdout ----

thread 'places_that_hold_keys_by_their_names' (24908) panicked at crates\steplet-core\tests\paths.rs:28:53:
called `Option::unwrap()` on a `None` value

---- paths_outside_the_repository stdout ----

thread 'paths_outside_the_repository' (21928) panicked at crates\steplet-core\tests\paths.rs:19:9:
/etc/hostname

---- cleaning_up_paths_as_text stdout ----

thread 'cleaning_up_paths_as_text' (42544) panicked at crates\steplet-core\tests\paths.rs:53:5:
assertion `left == right` failed
  left: None
 right: Some("/w/r1/src/b.rs")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    cleaning_up_paths_as_text
    in_text_only_places_outside_the_repository_count
    paths_outside_the_repository
    places_that_hold_keys_by_their_names
    protected_folders_at_any_depth

test result: FAILED. 0 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p steplet-core --test paths`
```

**Step 3, after `paths.rs` is written:**

```
> cargo test -p steplet-core --offline --test paths
   Compiling steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.63s
     Running tests\paths.rs (target\debug\deps\paths-1e09dc0109ff8170.exe)

running 5 tests
test cleaning_up_paths_as_text ... ok
test protected_folders_at_any_depth ... ok
test in_text_only_places_outside_the_repository_count ... ok
test paths_outside_the_repository ... ok
test places_that_hold_keys_by_their_names ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Extra: clippy with warnings as errors** (exit code 0, last lines):

```
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Checking steplet-core v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-core)
    Checking steplet-sign v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase2-task1-07fd4c\crates\steplet-sign)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.63s
```

**Verdict:** done. 5 tests pass in `paths`; clippy is clean. Not run in this task: the full workspace test (no existing code changed; clippy compiled every test target).
