# Milestone 1, phase 1: the file core and the sender tool — build plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans, run inline in one session (the user's choice from phase 0: not through separate agents). Steps use checkbox (`- [ ]`) syntax for tracking.

Written 11 Oct 2026. Status: **approved by the user on 11 Oct 2026, with the recommended option for F1, F2 and F3** (recorded in `docs/decisions.md`, "Milestone 1, phase 1 choices"). To be run inline in a new session, starting at Task 1. Before approval, nothing in it had been built, installed or run, except the read-only checks in section 2.

**Goal:** a strict, signed `.steplet` file format that the app (phase 4) and the check page (phase 6) read with the same code, and a command-line tool, `steplet-sign`, that lets a sender check and sign a file.

**Architecture:** one Rust library, `steplet-core`, that cannot touch files, network or clock (`no_std` + `alloc`), with five modules: `canon` (the fixed clean-up and the split into header, signature block and body), `header` (our own reader for a small part of YAML, then the field list), `sig` (signature check), `limits` (row 12's size limits) and `display` (text broken into pieces the screens can draw safely). A small program, `steplet-sign`, uses the library plus Windows' own `ssh-keygen` to check and sign files. Tests come first in every task.

**Tech stack:** Rust 1.98.0 stable (MSVC), edition 2024, nightly only for fuzzing; `ssh-key` 0.6.7; test-only: `proptest` 1.11.0, `saphyr-parser` 0.1.0, `icu_properties` 2.3.0, `libfuzzer-sys` 0.4.13; Windows OpenSSH `ssh-keygen` 9.5; Python 3.13 (one script that makes test files).

Words used below, explained once:
- **Crate:** a Rust package. **Workspace:** several crates built together.
- **`no_std`:** Rust code that does not use the operating system at all (no files, network or clock). **WASM:** a way to run Rust code in a web browser.
- **Clean-up (canonical form):** the fixed changes made to a file before its signature is checked (decision row 14), so Windows line endings and stray spaces do not break a signature.
- **Header:** the lines between the first two `---` lines. **Body:** everything after the second `---`.
- **YAML:** the text format the header is written in. We accept only a small, strict part of it.
- **SSHSIG:** the OpenSSH signature format (row 13). Its text starts with `-----BEGIN SSH SIGNATURE-----`. **Name space:** a word inside each signature saying what it was made for; ours is `steplet-v1` (P4).
- **Fingerprint:** a short code that names one key, like `SHA256:IVYu…`.
- **Test vector:** a fixed test file with a known right answer, made by a tool other than ours.
- **Property test:** a test that tries thousands of generated inputs and checks a rule holds for each one (`proptest`).
- **Differential test:** feeding the same input to our reader and to another reader and checking they agree.
- **Fuzzing:** feeding a program huge numbers of random inputs to find crashes.
- **Invisible character:** a character that shows nothing on screen, or looks like a space (for example U+200B, "zero width space"). The screens must show these as visible badges (row 28).
- **Exit code:** the number a program returns when it ends; `0` means "fine".

---

## 1. Who this phase is for, and what they get

### The sender (the person this phase is built for)

**Who:** a developer who wants someone else to run a task: "check out my repository at this commit and run its tests".

**What they are trying to get done:** write a `.steplet` file, make sure it is right, sign it with their own SSH key, and send it.

**What they see and touch, in order:**
1. **A template and a guide.** `docs/steplet-format.md` shows a complete example file and every field, one per row, with what is allowed. `examples/run-tests.steplet` is a working copy to start from.
2. **Their editor.** They write the header (title, sender, expiry, repository, steps, what the agent may do, what it must never do) and the body (instructions for the agent, in Markdown).
3. **`steplet-sign check my-task.steplet`.** Either `my-task.steplet: ok` with a one-screen summary (counts, repository, expiry, signature), or each problem with its line number, what is allowed, and a corrected example:
   ```
   my-task.steplet: 1 problem
     problem: line 7: a tab character
       allowed: indent with spaces, two per level; no tab characters anywhere in the header
       for example:
         allow:
           commands:
             - cargo test
   ```
4. **`steplet-sign sign my-task.steplet --key C:\Users\them\.ssh\id_ed25519`.** Their own `ssh-keygen` asks for their passphrase as usual. The tool re-checks the new signature with our own code before it writes anything, then adds the signature block to the file and prints the key's fingerprint and the expiry.
5. **They send the file.** Editing it after signing breaks the signature; running `sign` again replaces it.

**Where they get stuck, and what this phase does about each:**

| Stuck point | What the tool does |
|---|---|
| A YAML style we do not accept (`[a, b]` lists, `|` blocks, tabs, comments after a value) | Names the line, says what is allowed, shows a corrected example. Never guesses. |
| A value that YAML would read differently from how it looks (`title: Note: x`, `title: Fix #12`) | Refuses it and shows the same value in double quotes. |
| A missing or unknown field | Names it, and lists the allowed names at that level. |
| Expiry missing, in the past, or more than 365 days away (row 19) | `check` reports it; `sign` refuses, showing a date 14 days from today. |
| Over a size limit (row 12) | Names the line of the first item over the limit and the limit. |
| An RSA or other non-Ed25519 key | `sign` refuses before writing anything: "the signing key is not an Ed25519 key", and shows how to make one. |
| `ssh-keygen` not found, wrong passphrase, key not in the SSH agent | `ssh-keygen`'s own message is shown, then ours with the exit code; the file is not changed. |
| They edit after signing | `check` says "the file was changed after it was signed"; `sign` again fixes it. |

### The receiver (later phases)

Never sees this phase directly. But everything the preview shows in phase 4 comes from here: the parsed fields with their line numbers, the signature line ("signature valid, sender unknown"), invisible characters as badges, and links showing their real address. The check page (phase 6) runs the same library as WASM.

### You, the builder

You get: the two crates, their tests, a results file (`docs/milestone-1-phase-1-results.md`) with every command and its pasted output, and answers to three choices (section 4).

---

## 2. What is on this laptop today (read-only checks, 11 Oct 2026)

```
$ git log --oneline -1          (in the worktree .claude\worktrees\steplet-phase1-plan-15bade)
90b0e33 Update the handoffs: phase 0 decisions recorded
$ rustup toolchain list
stable-x86_64-pc-windows-msvc (active, default)
nightly-x86_64-pc-windows-msvc
$ rustup target list --installed
wasm32-unknown-unknown
wasm32v1-none
x86_64-pc-windows-msvc
$ cargo --version; cargo +nightly --version; cargo fuzz --version
cargo 1.98.0 (797e8a9bc 2026-08-05)
cargo 1.101.0-nightly (f3865b2a4 2026-09-29)
cargo-fuzz 0.13.2
```

Newest versions on crates.io (`https://crates.io/api/v1/crates/<name>/<version>`, read 11 Oct 2026):

| Crate | Newest | Published | Used here |
|---|---|---|---|
| `proptest` | 1.12.0 | 10 Oct 2026 (one day old) | **1.11.0** (24 Mar 2026), by the 14-day rule from phase 0 |
| `saphyr-parser` | 0.1.0 | 19 Sep 2026 | 0.1.0 |
| `icu_properties` | 2.3.0 | 13 Aug 2026 | 2.3.0 (already in cargo's cache from phase 0's spikes) |
| `libfuzzer-sys` | 0.4.13 | 4 Jun 2026 | 0.4.13 |
| `ssh-key` | 0.7.0-rc.11 | (release candidate) | **0.6.7**, as the milestone plan says |

Facts read from source on this laptop (cargo's cache):
- `ssh-key` 0.6.7: `PrivateKey::sign(&self, namespace, hash_alg, msg) -> Result<SshSig>` exists with the `alloc` feature (`src/private.rs:314`), so tests can make signatures with no extra features. `Ed25519Keypair::from_seed` (`src/private/ed25519.rs:166`). `SshSig::namespace()` (`src/sshsig.rs:191`). RSA public keys still decode with only `alloc` (`src/public/key_data.rs:188`), so an RSA signature reads as "wrong key type", not "unreadable". Versions above 1 are refused while decoding (`src/sshsig.rs:239`), so our own version check can never be reached; it stays as a second lock.
- `icu_properties` 2.3.0: `CodePointSetData::new::<DefaultIgnorableCodePoint>()` and `.contains(self, ch: char) -> bool` (`src/code_point_set.rs:160`); `CodePointMapData::<GeneralCategory>::new().get(ch)`; `props` module is public; `compiled_data` is a default feature.
- `saphyr-parser` 0.1.0 (docs.rs, via a documentation lookup): `Parser::new_from_str(&str)` is an iterator of `Result<(Event, Span), ScanError>`; `Event::Scalar(Cow<str>, ScalarStyle, usize, Option<Cow<Tag>>)`, `SequenceStart(usize, Option<…>)`, `MappingStart(usize, Option<…>)`, `Alias(usize)`, plus stream and document start and end events. The crate itself is `no_std`. Not compiled yet.
- Phase 0 (spikes/results/setup.md): fuzzing works on Windows only with the AddressSanitizer folder of the Visual Studio build tools on `PATH` for the fuzz command: `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64`.
- Phase 0 (S9): `ssh-key` 0.6.7 builds for `wasm32v1-none` and verifies a Windows `ssh-keygen` signature.

**No document anywhere lists the header's fields.** Decisions give pieces (expiry row 19, size limits row 12, repository source P3, signature placement in the signing memo, typed fields R12), but not the field list. Section 4, choice F1, proposes one.

---

## 3. Global constraints

Every task includes these.

- Rust 1.98.0 stable (MSVC), edition 2024. Nightly only for `cargo +nightly fuzz`.
- `steplet-core` is `#![no_std]` + `alloc`. Its only normal dependency is `ssh-key = "=0.6.7"`, `default-features = false`, features `alloc`, `ed25519`. It must build for `wasm32v1-none` (a target with no standard library at all) at the end of every task that changes its dependencies, and in Task 14.
- Signatures: Ed25519 keys, sha512, SSHSIG version 1, name space `steplet-v1` (`steplet_core::NAMESPACE`).
- Size limits (row 12): file 256 KiB (262,144 bytes), header 32 KiB (32,768 bytes), at most 25 commands, 15 domains, 10 folders per list, 30 steps, any command under 300 characters (counted as Unicode characters, so at most 299).
- Clean-up (row 14): strict UTF-8; drop one byte-order mark; CRLF to LF; drop spaces and tabs at line ends; exactly one final newline; no Unicode normalisation. Signed bytes are the cleaned-up text minus the signature block lines.
- Parse once, from the signed bytes (row 15). Refuse duplicate keys, anchors, aliases, tags and extra documents.
- `ssh-keygen` is always called by full path on Windows: `C:\Windows\System32\OpenSSH\ssh-keygen.exe` (Git Bash finds a different one first).
- Test keys are throwaway. Keys made for the committed test files live in `C:\dev\m1\p1` and are never committed; only public halves and signed files are.
- New crate versions: the newest release at least 14 days old (section 2). Pinned with `=`.
- Commits are local, on this worktree's branch `claude/steplet-phase1-plan-15bade`, never pushed. No attribution lines. Plain words in messages. Approving this plan approves the commits it lists; each download still gets its own yes (section 5).
- Evidence: no task is done without its test command and pasted output in `docs/milestone-1-phase-1-results.md`.
- All commands run from the repository root of this worktree, `C:\dev\runFile\.claude\worktrees\steplet-phase1-plan-15bade`, in PowerShell unless marked. (`cd` lines below say "the repository root".)

---

## 4. Choices for you, each with a recommendation

### F1. The header's fields

Nothing decided says which fields a file has. This is my proposal. Everything not listed is refused (strict), so later milestones add fields under a new format version.

```yaml
---
steplet: 1                     # format version; must be the first line, and exactly 1
title: Run the tests of Hello-World
sender: Ana Lee                # who says they sent it (a claim; the signature line says who really signed)
expires: 2026-10-25            # YYYY-MM-DD, required; the file is expired once that day ends (UTC)
repository:                    # optional in the format; milestone 1 runs need it (P3)
  url: https://github.com/octocat/Hello-World.git
  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d   # full hash, 40 (or 64) lowercase hex
steps:                         # required, 1 to 30 one-line steps, shown to the receiver
  - Count the lines in the README file.
  - Report the count.
allow:                         # optional; each list optional
  commands:                    # up to 25, each under 300 characters; their meaning is phase 2's (R8)
    - wc -l README
  domains:                     # up to 15 exact lowercase host names, no wildcards
    - crates.io
  read:                        # up to 10 folders inside the repository; "." means all of it
    - .
  write:                       # up to 10
    - target
never:                         # optional, up to 25 one-line things the agent must never do
  - Do not push to any remote.
signature: |                   # added by steplet-sign; always the last thing in the header
  -----BEGIN SSH SIGNATURE-----
  …
  -----END SSH SIGNATURE-----
---
# Body: instructions for the agent, in Markdown (shown as text, never turned into a web page)
```

Points worth your attention:
- **Steps live in the header, not the body.** The preview needs an exact count and one row per step (row 28: counts first, one item per row). Counting steps in free Markdown would be guesswork.
- **Commands are stored as plain text in this phase.** What a command rule means (R8: program plus argument rules) is phase 2's design. If phase 2 needs structured rules, it changes this field then.
- **No wildcards in domains** (`*.example.com` would also need quoting, because `*` means something in YAML). Phase 2 can add an explicit form if real files need it.

| Option | What goes wrong |
|---|---|
| **A. This field list (recommended).** | If phase 2 needs a different shape for commands, the format changes once more before anything is published. Nothing is published yet, so that is cheap. |
| B. Steps in the Markdown body (numbered list) | The step count and step rows depend on how Markdown is read; a sender can hide a step inside a nested list or a code block. |
| C. Wait and design the field list together first | Phase 1 cannot start: the parser's types and every error message depend on it. |

### F2. Where `steplet-sign sign` writes

| Option | What goes wrong |
|---|---|
| **A. In place (recommended):** it replaces the file with the signed version, written to a temporary file next to it first and then renamed, so a crash cannot leave half a file. Windows line endings become Unix ones. | The sender's own file changes. If they wanted to keep an unsigned copy, they copy it first. |
| B. To a new file (`my-task.signed.steplet`) | Two near-identical files; senders will send the wrong one. |

### F3. Expiry on every file, not only signed ones

Row 19 says "expiry required on signed files". I propose **required on every file**: one rule, and an unsigned draft becomes signable without a new field.

| Option | What goes wrong |
|---|---|
| **A. Required on every file (recommended).** | An unsigned file written by hand needs one more line. |
| B. Required only on signed files (row 19 as written) | Two rules, and phase 2's rating needs a separate case for a signed file without expiry. |

### Smaller technical choices I made (tell me if you object)

1. **The parser accepts less than the milestone plan listed.** Plan section 3 listed `|` blocks and lists of small maps. Neither is needed: the only multi-line value is the signature, which the clean-up step cuts out by fixed markers before the parser runs, and no field above is a list of maps. Less grammar, fewer ways to disagree with other YAML readers. Phase 2 adds lists of maps if R8 needs them.
2. **A lone carriage return (`\r` not followed by `\n`) refuses the file** instead of becoming a new line. Row 14 only mentions CRLF; the signing memo said CR too. A lone `\r` is a known way to hide text in terminals, and no current editor writes it.
3. **Signature markers anywhere else refuse the file.** A `-----BEGIN SSH SIGNATURE` or `-----END SSH SIGNATURE` line anywhere but in the one allowed place (body included), or a second `signature:` line, is an error, not ignored text (R13: fixed markers, no guessing).
4. **A new limit: `never` has at most 25 lines.** Row 12 does not cover it; 25 matches the command limit. A starting value to tune in phase 2.
5. **`display::Segment::Link { shown: Option<…>, real }`** instead of the milestone plan's `Link { shown, real }`: a bare address has no separate shown text. This lets a test prove nothing is ever dropped (the pieces always join back into the exact original text).
6. **Invisible characters** (badges): every character that Unicode calls "default ignorable", every control, format, private-use and separator character, except tab, newline and the ordinary space. The table is written by hand (no Unicode library in the core, to keep the WASM small) and checked in a test against `icu_properties` for all 1,114,112 code points.
7. **Exit codes of `steplet-sign`:** 0 fine, 1 problems found (or signing failed), 2 the command was used wrongly or the file could not be read.

---

## 5. What needs your yes, and when

| # | When | What happens | Size | Undo |
|---|---|---|---|---|
| B1 | Task 6 | cargo downloads `proptest` 1.11.0 and `saphyr-parser` 0.1.0, with their dependencies, from crates.io (test-only) | a few MB | delete the two lines from `crates/steplet-core/Cargo.toml` |
| B2 | Task 13 | cargo downloads `libfuzzer-sys` 0.4.13, if it is not already in the cache from phase 0's fuzz smoke test (I check first and tell you) | under 1 MB | delete `crates/steplet-core/fuzz` |
| B3 | Task 12, optional | **You** make a passphrase-protected throwaway key and sign one file with it in your own terminal (I cannot type a passphrase) | none | delete `C:\dev\m1\p1\pass*` |

Also, without a separate yes (covered by approving this plan): Task 9 makes two throwaway keys (Ed25519 and RSA) in `C:\dev\m1\p1`; the `steplet-sign` tests make throwaway keys in a fresh folder under `%TEMP%` and delete that folder when they finish.

---

## 6. Tasks

Order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12 → 13 → 14. Task 10 (`display`) needs only Task 1 and can move anywhere after it.

Files at the end of the phase:

| File | Responsibility |
|---|---|
| `Cargo.toml` (new, repository root) | The product workspace. `spikes/` stays a separate workspace. |
| `crates/steplet-core/src/lib.rs` | Module list, `NAMESPACE`, `Help` (allowed + example for every refusal) |
| `crates/steplet-core/src/date.rs` | `YYYY-MM-DD` dates in UTC, conversion to and from Unix time |
| `crates/steplet-core/src/limits.rs` | Row 12's numbers and `limits::check` |
| `crates/steplet-core/src/canon.rs` | Clean-up, header/body split, signature block by fixed markers, `with_signature` |
| `crates/steplet-core/src/header.rs` | The YAML-subset reader (`tree`) and the field list (`parse` → `Steplet`) |
| `crates/steplet-core/src/sig.rs` | `verify` → `SigStatus` |
| `crates/steplet-core/src/display.rs` | `segments`, `is_invisible` |
| `crates/steplet-core/tests/*.rs` | One test file per module, plus `properties.rs`, `differential.rs`, `vectors.rs` |
| `crates/steplet-core/tests/vectors/` | Files signed by Windows' `ssh-keygen`, and `make_vectors.py` that made them |
| `crates/steplet-core/fuzz/` | The fuzz target (its own small workspace, as `cargo fuzz` requires) |
| `crates/steplet-sign/src/{lib.rs,main.rs,check.rs,sign.rs}` | The sender tool |
| `crates/steplet-sign/tests/cli.rs` | Runs the real program with real `ssh-keygen` |
| `examples/run-tests.steplet` | The sender's starting file |
| `docs/steplet-format.md` | The sender's guide |
| `docs/milestone-1-phase-1-results.md` | Commands and pasted output, task by task |

---

### Task 1: the workspace, a core that builds without the operating system, dates and limit numbers

**Files:**
- Create: `Cargo.toml`, `crates/steplet-core/Cargo.toml`, `crates/steplet-core/src/lib.rs`, `crates/steplet-core/src/date.rs`, `crates/steplet-core/src/limits.rs`
- Test: `crates/steplet-core/tests/date.rs`
- Create: `docs/milestone-1-phase-1-results.md`

**Interfaces:**
- Produces: `steplet_core::NAMESPACE: &str = "steplet-v1"`; `steplet_core::Help { allowed: &'static str, example: &'static str }`; `steplet_core::date::Date` with `new(u16, u8, u8) -> Option<Date>`, `parse(&str) -> Option<Date>`, `days_since_epoch(self) -> i64`, `from_days_since_epoch(i64) -> Option<Date>`, `from_unix(i64) -> Option<Date>`, `end_unix(self) -> i64`, `add_days(self, i64) -> Option<Date>`, `Display` as `YYYY-MM-DD`; `steplet_core::limits::{MAX_FILE_BYTES, MAX_HEADER_BYTES, MAX_COMMANDS, MAX_DOMAINS, MAX_FOLDERS_PER_LIST, MAX_STEPS, MAX_COMMAND_CHARS, MAX_NEVER, MAX_EXPIRY_DAYS, DEFAULT_EXPIRY_DAYS}` (all `usize` except the two `EXPIRY` ones, `i64`).

- [ ] **Step 1: the workspace files.**

`Cargo.toml` (repository root):
```toml
# Steplet product code. spikes/ is a separate, throwaway workspace and is not part of this one.
[workspace]
resolver = "3"
members = ["crates/steplet-core"]
exclude = ["spikes", "crates/steplet-core/fuzz"]

[workspace.package]
edition = "2024"
rust-version = "1.98"
publish = false
```

`crates/steplet-core/Cargo.toml`:
```toml
[package]
name = "steplet-core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
```

`crates/steplet-core/src/lib.rs`:
```rust
//! Steplet's file core: the clean-up, the header reader, signature checks, size limits and display pieces.
//! It uses no operating system at all (`no_std`), so the desktop app and the browser check page run the same code.
#![no_std]
extern crate alloc;

pub mod date;
pub mod limits;

/// The signature name space (decision P4).
pub const NAMESPACE: &str = "steplet-v1";

/// What a refused file may contain instead, and a corrected example. Shown by `steplet-sign check`
/// and, from phase 4, by the preview. `example` may be empty when no example helps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Help {
    pub allowed: &'static str,
    pub example: &'static str,
}
```

`crates/steplet-core/src/limits.rs`:
```rust
//! Size limits (decision row 12). Starting values, to tune on the test corpus in phase 2. Over a limit is red.

pub const MAX_FILE_BYTES: usize = 256 * 1024;
pub const MAX_HEADER_BYTES: usize = 32 * 1024;
pub const MAX_COMMANDS: usize = 25;
pub const MAX_DOMAINS: usize = 15;
pub const MAX_FOLDERS_PER_LIST: usize = 10;
pub const MAX_STEPS: usize = 30;
/// "Any command under 300 characters": 299 is the longest allowed. Characters are Unicode scalar values.
pub const MAX_COMMAND_CHARS: usize = 299;
/// Not in row 12. A starting value for the `never` list, equal to the command limit.
pub const MAX_NEVER: usize = 25;
/// The furthest an `expires` date may be from the day of signing (row 19).
pub const MAX_EXPIRY_DAYS: i64 = 365;
/// The expiry `steplet-sign` suggests (row 19).
pub const DEFAULT_EXPIRY_DAYS: i64 = 14;
```

- [ ] **Step 2: write the failing date tests.** `crates/steplet-core/tests/date.rs`:
```rust
use steplet_core::date::Date;

#[test]
fn reads_and_writes_dates() {
    assert_eq!(Date::parse("2026-10-25").unwrap().to_string(), "2026-10-25");
    assert!(Date::parse("2028-02-29").is_some());
    assert!(Date::parse("2000-02-29").is_some());
}

#[test]
fn refuses_anything_else() {
    for s in [
        "2026-02-29", "1900-02-29", "2026-13-01", "2026-00-10", "2026-10-32", "2026-10-00",
        "26-10-25", "2026/10/25", "2026-1-05", "2026-10-25 ", " 2026-10-25", "\u{ff12}026-10-25",
        "1999-12-31", "+026-10-25", "2026-10-2a", "",
    ] {
        assert_eq!(Date::parse(s), None, "{s:?}");
    }
}

#[test]
fn counts_days_from_1970() {
    let d = Date::parse("2026-10-11").unwrap();
    assert_eq!(d.days_since_epoch(), 20_737);
    assert_eq!(Date::from_days_since_epoch(20_737), Some(d));
    assert_eq!(Date::parse("2000-01-01").unwrap().days_since_epoch(), 10_957);
}

#[test]
fn a_day_ends_at_midnight_utc() {
    let d = Date::parse("2026-10-11").unwrap();
    assert_eq!(d.end_unix(), 1_791_763_200);
    assert_eq!(Date::from_unix(1_791_763_199), Some(d));
    assert_eq!(Date::from_unix(1_791_763_200), Date::parse("2026-10-12"));
}

#[test]
fn adds_days_across_months_and_years() {
    let d = Date::parse("2026-10-11").unwrap();
    assert_eq!(d.add_days(14), Date::parse("2026-10-25"));
    assert_eq!(d.add_days(365), Date::parse("2027-10-11"));
    assert_eq!(Date::parse("2028-02-28").unwrap().add_days(1), Date::parse("2028-02-29"));
    assert_eq!(Date::parse("9999-12-31").unwrap().add_days(1), None);
}
```
`crates/steplet-core/src/date.rs` with an empty body so the test file compiles:
```rust
//! Calendar dates in UTC, written `YYYY-MM-DD`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date;
```
Run (repository root): `cargo test -p steplet-core --test date`
Expected: does not compile (`no function or associated item named parse`). That is the failing state.

- [ ] **Step 3: write `date.rs`.**
```rust
//! Calendar dates in UTC, written `YYYY-MM-DD`. A file expires when its `expires` day ends, UTC.

use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    /// A real calendar date between 2000-01-01 and 9999-12-31.
    pub fn new(year: u16, month: u8, day: u8) -> Option<Date> {
        if !(2000..=9999).contains(&year) || !(1..=12).contains(&month) {
            return None;
        }
        if day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Date { year, month, day })
    }

    /// Reads exactly `YYYY-MM-DD`: ten ASCII characters, digits and two hyphens.
    pub fn parse(s: &str) -> Option<Date> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let num = |from: usize, to: usize| -> Option<u16> {
            let mut n: u16 = 0;
            for &c in &b[from..to] {
                if !c.is_ascii_digit() {
                    return None;
                }
                n = n * 10 + u16::from(c - b'0');
            }
            Some(n)
        };
        Date::new(num(0, 4)?, num(5, 7)? as u8, num(8, 10)? as u8)
    }

    /// Days since 1970-01-01.
    pub fn days_since_epoch(self) -> i64 {
        days_from_civil(i64::from(self.year), i64::from(self.month), i64::from(self.day))
    }

    pub fn from_days_since_epoch(days: i64) -> Option<Date> {
        let (y, m, d) = civil_from_days(days);
        if !(2000..=9999).contains(&y) {
            return None;
        }
        Date::new(y as u16, m as u8, d as u8)
    }

    /// The UTC date that contains this Unix time (seconds since 1970-01-01 00:00 UTC).
    pub fn from_unix(secs: i64) -> Option<Date> {
        Date::from_days_since_epoch(secs.div_euclid(86_400))
    }

    /// The first second after this day, UTC. A file whose `expires` is this date is expired from then on.
    pub fn end_unix(self) -> i64 {
        (self.days_since_epoch() + 1) * 86_400
    }

    pub fn add_days(self, n: i64) -> Option<Date> {
        Date::from_days_since_epoch(self.days_since_epoch() + n)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn is_leap(y: u16) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: u16, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if is_leap(y) => 29,
        _ => 28,
    }
}

// Howard Hinnant's day-count algorithms (public domain): http://howardhinnant.github.io/date_algorithms.html
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}
```

- [ ] **Step 4: run the tests and the no-operating-system build.**
```powershell
cargo test -p steplet-core --test date
cargo build -p steplet-core --target wasm32v1-none --release
cd spikes; cargo test -p s9-core; cd ..
```
Expected: `5 passed`; the `wasm32v1-none` build finishes; `s9-core` still passes 3 of 3 (the new root workspace does not swallow the spikes).

- [ ] **Step 5: start the results file and commit.** Create `docs/milestone-1-phase-1-results.md`:
```markdown
# Milestone 1, phase 1: results

Plan: `docs/plans/2026-10-11-milestone-1-phase-1.md`. Each task: the commands, their pasted output, and a verdict.

## Task 1: workspace, dates, limit numbers
```
Paste the three commands and their output under it.
```powershell
git add Cargo.toml crates/steplet-core docs/milestone-1-phase-1-results.md
git commit -m "Start the file core: workspace, dates, size limit numbers"
```
(`Cargo.lock` appears once the core has a dependency, in Task 6; it is committed then.)

---

### Task 2: the clean-up and the header/body split

**Files:**
- Create: `crates/steplet-core/src/canon.rs`
- Modify: `crates/steplet-core/src/lib.rs` (add `pub mod canon;`)
- Test: `crates/steplet-core/tests/canon.rs`

**Interfaces:**
- Consumes: `limits::{MAX_FILE_BYTES, MAX_HEADER_BYTES}`, `Help`.
- Produces: `canon::canonicalize(raw: &[u8]) -> Result<Canonical, CanonError>`; `Canonical` (derives `Clone, Debug, PartialEq, Eq`) with `text() -> &str` (whole cleaned-up file), `signed() -> &str` (what the signature covers), `header() -> &str` (lines between the two `---` lines, of `signed`), `body() -> &str`, `signature() -> Option<&SignatureBlock>`, `Canonical::HEADER_FIRST_LINE: u32 = 2`; `CanonError` variants `FileTooLarge { bytes: usize }`, `NotUtf8 { line: u32 }`, `LoneCarriageReturn { line: u32 }`, `NoHeaderStart`, `NoHeaderEnd`, `HeaderTooLarge { bytes: usize }`, `SignatureMisplaced { line: u32 }`, `SignatureTwice { line: u32 }`, `SignatureBroken { line: u32 }`, with `Display` and `help() -> Help`. (The three signature variants are used in Task 3; they are declared now so the type is stable.)

- [ ] **Step 1: write the failing tests.** `crates/steplet-core/tests/canon.rs`:
```rust
use steplet_core::canon::{CanonError, Canonical, canonicalize};

fn ok(raw: &str) -> Canonical {
    canonicalize(raw.as_bytes()).expect("should clean up")
}

#[test]
fn windows_line_endings_become_unix_ones() {
    assert_eq!(ok("---\r\nsteplet: 1\r\n---\r\nbody\r\n").text(), "---\nsteplet: 1\n---\nbody\n");
}

#[test]
fn spaces_and_tabs_at_line_ends_go() {
    assert_eq!(ok("---  \nsteplet: 1\t \n---\nbody \t\n").text(), "---\nsteplet: 1\n---\nbody\n");
}

#[test]
fn exactly_one_final_newline() {
    assert_eq!(ok("---\na: b\n---\nbody").text(), "---\na: b\n---\nbody\n");
    assert_eq!(ok("---\na: b\n---\nbody\n\n\n \n").text(), "---\na: b\n---\nbody\n");
}

#[test]
fn one_byte_order_mark_is_dropped_and_only_one() {
    assert_eq!(ok("\u{feff}---\na: b\n---\n").text(), "---\na: b\n---\n");
    assert_eq!(canonicalize("\u{feff}\u{feff}---\na: b\n---\n".as_bytes()), Err(CanonError::NoHeaderStart));
}

#[test]
fn no_unicode_normalisation() {
    let c = ok("---\na: e\u{301}\n---\n\u{e9}\n");
    assert!(c.text().contains("e\u{301}"));
    assert!(c.text().contains('\u{e9}'));
}

#[test]
fn leading_and_inner_spaces_stay() {
    assert_eq!(ok("---\na:  b  c\n---\n  x\n").text(), "---\na:  b  c\n---\n  x\n");
}

#[test]
fn bad_utf8_names_its_line() {
    assert_eq!(canonicalize(b"---\na: b\n\xff\n---\n"), Err(CanonError::NotUtf8 { line: 3 }));
}

#[test]
fn a_lone_carriage_return_is_refused() {
    assert_eq!(canonicalize(b"---\na: b\rc\n---\n"), Err(CanonError::LoneCarriageReturn { line: 2 }));
}

#[test]
fn the_header_opens_on_line_one_and_closes() {
    for raw in ["", "\n---\na: b\n---\n", "title: x\n", " ---\na: b\n---\n"] {
        assert_eq!(canonicalize(raw.as_bytes()), Err(CanonError::NoHeaderStart), "{raw:?}");
    }
    assert_eq!(canonicalize(b"---\na: b\n"), Err(CanonError::NoHeaderEnd));
}

#[test]
fn header_and_body_are_split_at_the_first_closing_line() {
    let c = ok("---\nsteplet: 1\ntitle: x\n---\n# Hi\n---\nmore\n");
    assert_eq!(c.header(), "steplet: 1\ntitle: x\n");
    assert_eq!(c.body(), "# Hi\n---\nmore\n");
    assert_eq!(c.signed(), c.text());
    assert!(c.signature().is_none());
    assert_eq!(ok("---\na: b\n---\n").body(), "");
}

#[test]
fn size_limits() {
    let mut at_limit = b"---\na: b\n---\n".to_vec();
    at_limit.resize(256 * 1024, b'x');
    assert!(canonicalize(&at_limit).is_ok());
    at_limit.push(b'x');
    assert_eq!(canonicalize(&at_limit), Err(CanonError::FileTooLarge { bytes: 262_145 }));

    let mut big_header = String::from("---\n");
    while big_header.len() < 32 * 1024 + 4 {
        big_header.push_str("never: x\n");
    }
    big_header.push_str("---\n");
    assert!(matches!(canonicalize(big_header.as_bytes()), Err(CanonError::HeaderTooLarge { .. })));
}

#[test]
fn every_error_has_words_and_help() {
    let all = [
        CanonError::FileTooLarge { bytes: 1 }, CanonError::NotUtf8 { line: 1 },
        CanonError::LoneCarriageReturn { line: 1 }, CanonError::NoHeaderStart, CanonError::NoHeaderEnd,
        CanonError::HeaderTooLarge { bytes: 1 }, CanonError::SignatureMisplaced { line: 1 },
        CanonError::SignatureTwice { line: 1 }, CanonError::SignatureBroken { line: 1 },
    ];
    for e in all {
        assert!(!e.to_string().is_empty());
        assert!(!e.help().allowed.is_empty(), "{e:?}");
    }
}
```
Add `pub mod canon;` to `lib.rs`, and a `canon.rs` holding only the type declarations from **Interfaces** with `canonicalize` returning `Err(CanonError::NoHeaderStart)`, `Display` writing `"not written yet"`, and `help()` returning `Help { allowed: "", example: "" }`.

Run: `cargo test -p steplet-core --test canon`
Expected: all 12 tests FAIL (the stub always answers "no header start", has no words and no help).

- [ ] **Step 2: write `canon.rs` (clean-up and split; the signature part comes in Task 3).**
```rust
//! The fixed clean-up (decision row 14) and the split into header, signature block and body (R13).
//! Signed bytes are the cleaned-up text minus the signature block lines.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

use crate::Help;
use crate::limits::{MAX_FILE_BYTES, MAX_HEADER_BYTES};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canonical {
    text: String,
    signed: String,
    /// Into `signed`: the lines between the two `---` lines.
    header: Range<usize>,
    /// Into `signed`: everything after the closing `---` line.
    body: Range<usize>,
    signature: Option<SignatureBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignatureBlock {
    /// The armoured signature with the two-space indent removed, a `\n` after each line.
    pub pem: String,
    /// Line numbers in the cleaned-up text of the `signature: |` line and of the END line.
    pub first_line: u32,
    pub last_line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonError {
    FileTooLarge { bytes: usize },
    NotUtf8 { line: u32 },
    LoneCarriageReturn { line: u32 },
    NoHeaderStart,
    NoHeaderEnd,
    HeaderTooLarge { bytes: usize },
    SignatureMisplaced { line: u32 },
    SignatureTwice { line: u32 },
    SignatureBroken { line: u32 },
}

impl Canonical {
    /// Line number of the header's first line (the line after the opening `---`).
    pub const HEADER_FIRST_LINE: u32 = 2;

    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn signed(&self) -> &str {
        &self.signed
    }
    pub fn header(&self) -> &str {
        &self.signed[self.header.clone()]
    }
    pub fn body(&self) -> &str {
        &self.signed[self.body.clone()]
    }
    pub fn signature(&self) -> Option<&SignatureBlock> {
        self.signature.as_ref()
    }
}

pub fn canonicalize(raw: &[u8]) -> Result<Canonical, CanonError> {
    if raw.len() > MAX_FILE_BYTES {
        return Err(CanonError::FileTooLarge { bytes: raw.len() });
    }
    let s = core::str::from_utf8(raw)
        .map_err(|e| CanonError::NotUtf8 { line: line_of(&raw[..e.valid_up_to()]) })?;
    let s = s.strip_prefix('\u{feff}').unwrap_or(s);
    let mut lines: Vec<&str> = Vec::new();
    for (i, piece) in s.split('\n').enumerate() {
        let piece = piece.strip_suffix('\r').unwrap_or(piece);
        if piece.contains('\r') {
            return Err(CanonError::LoneCarriageReturn { line: i as u32 + 1 });
        }
        lines.push(piece.trim_end_matches([' ', '\t']));
    }
    while lines.last() == Some(&"") {
        lines.pop();
    }
    split(&lines)
}

fn split(lines: &[&str]) -> Result<Canonical, CanonError> {
    if lines.first() != Some(&"---") {
        return Err(CanonError::NoHeaderStart);
    }
    let close = lines.iter().skip(1).position(|l| *l == "---").ok_or(CanonError::NoHeaderEnd)? + 1;
    let block = signature_block(lines, close)?;

    let mut text = String::new();
    let mut signed = String::new();
    let (mut header, mut body) = (0..0, 0..0);
    for (i, line) in lines.iter().enumerate() {
        text.push_str(line);
        text.push('\n');
        if block.as_ref().is_some_and(|(k, e, _)| (*k..=*e).contains(&i)) {
            continue;
        }
        if i == close {
            header.end = signed.len();
        }
        signed.push_str(line);
        signed.push('\n');
        if i == 0 {
            header.start = signed.len();
        }
        if i == close {
            body.start = signed.len();
        }
    }
    body.end = signed.len();
    if header.len() > MAX_HEADER_BYTES {
        return Err(CanonError::HeaderTooLarge { bytes: header.len() });
    }
    let signature = block.map(|(k, e, pem)| SignatureBlock { pem, first_line: k as u32 + 1, last_line: e as u32 + 1 });
    Ok(Canonical { text, signed, header, body, signature })
}

/// Task 3 replaces this with the real search. For now: no signature block.
fn signature_block(_lines: &[&str], _close: usize) -> Result<Option<(usize, usize, String)>, CanonError> {
    Ok(None)
}

fn line_of(before: &[u8]) -> u32 {
    before.iter().filter(|b| **b == b'\n').count() as u32 + 1
}

impl fmt::Display for CanonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonError::FileTooLarge { bytes } => {
                write!(f, "the file is {bytes} bytes; at most {MAX_FILE_BYTES} bytes (256 KiB) are allowed")
            }
            CanonError::NotUtf8 { line } => write!(f, "line {line}: the text is not valid UTF-8"),
            CanonError::LoneCarriageReturn { line } => {
                write!(f, "line {line}: a carriage return (\\r) that is not part of a Windows line ending")
            }
            CanonError::NoHeaderStart => write!(f, "line 1: the file must start with a line holding only `---`"),
            CanonError::NoHeaderEnd => write!(f, "the header has no closing `---` line"),
            CanonError::HeaderTooLarge { bytes } => {
                write!(f, "the header is {bytes} bytes; at most {MAX_HEADER_BYTES} bytes (32 KiB) are allowed")
            }
            CanonError::SignatureMisplaced { line } => write!(
                f,
                "line {line}: a signature block in the wrong place; it must be the last thing in the header"
            ),
            CanonError::SignatureTwice { line } => write!(f, "line {line}: a second signature block"),
            CanonError::SignatureBroken { line } => write!(f, "line {line}: the signature block is damaged"),
        }
    }
}

const RESIGN: &str = "steplet-sign sign my-task.steplet --key C:\\Users\\you\\.ssh\\id_ed25519";

impl CanonError {
    pub fn help(&self) -> Help {
        let (allowed, example) = match self {
            CanonError::FileTooLarge { .. } => ("a file of at most 256 KiB; split a bigger task into several files", ""),
            CanonError::NotUtf8 { .. } => ("text saved as UTF-8 (most editors call it \"UTF-8\")", ""),
            CanonError::LoneCarriageReturn { .. } => ("Windows (\\r\\n) or Unix (\\n) line endings only", ""),
            CanonError::NoHeaderStart => (
                "`---` alone on the first line, then the header",
                "---\nsteplet: 1\ntitle: Run the tests",
            ),
            CanonError::NoHeaderEnd => (
                "a line holding only `---` after the header, then the body",
                "never:\n  - Do not push.\n---\n# Instructions",
            ),
            CanonError::HeaderTooLarge { .. } => ("a header of at most 32 KiB; move long text into the body", ""),
            CanonError::SignatureMisplaced { .. }
            | CanonError::SignatureTwice { .. }
            | CanonError::SignatureBroken { .. } => (
                "one signature block, made by steplet-sign, as the last thing in the header; delete the damaged lines and sign again",
                RESIGN,
            ),
        };
        Help { allowed, example }
    }
}
```

- [ ] **Step 3: run the tests.** `cargo test -p steplet-core --test canon`
Expected: `12 passed`.

- [ ] **Step 4: paste the output into the results file under "## Task 2" and commit.**
```powershell
git add crates/steplet-core docs/milestone-1-phase-1-results.md
git commit -m "File core: the fixed clean-up and the split into header and body"
```

---

### Task 3: the signature block, by fixed markers

**Files:**
- Modify: `crates/steplet-core/src/canon.rs` (replace `signature_block`; add `with_signature`)
- Test: `crates/steplet-core/tests/canon_signature.rs`

**Interfaces:**
- Produces: `canon::BEGIN = "-----BEGIN SSH SIGNATURE-----"`, `canon::END = "-----END SSH SIGNATURE-----"`, `canon::SIGNATURE_KEY_LINE = "signature: |"`; `Canonical::with_signature(&self, pem: &str) -> String` (the signed text with `pem` embedded as the last thing in the header; replaces any block already there).

Rules (R13, fixed markers, no guessing):
1. More than one header line starting with `signature:` → `SignatureTwice` at the second one.
2. One such line at index `k`: it must be exactly `signature: |`; the next line exactly two spaces + BEGIN; then one or more lines of two spaces + base64 characters (`A–Z a–z 0–9 + / =`); then two spaces + END; and the line after END must be the closing `---`. A wrong line → `SignatureBroken` at that line; the block not last → `SignatureMisplaced` at `k`.
3. Any line anywhere (header or body) outside that block whose text, after leading spaces, starts with `-----BEGIN SSH SIGNATURE` or `-----END SSH SIGNATURE` → `SignatureMisplaced` at that line.

- [ ] **Step 1: write the failing tests.** `crates/steplet-core/tests/canon_signature.rs`:
```rust
use steplet_core::canon::{CanonError, canonicalize};

const SIG: &str = "signature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n  -----END SSH SIGNATURE-----\n";
const PEM: &str = "-----BEGIN SSH SIGNATURE-----\nU1NIU0lHAAAAAQ==\n-----END SSH SIGNATURE-----\n";

fn read(raw: &str) -> Result<steplet_core::canon::Canonical, CanonError> {
    canonicalize(raw.as_bytes())
}

#[test]
fn the_block_is_split_off() {
    let c = read(&format!("---\nsteplet: 1\ntitle: x\n{SIG}---\nbody\n")).unwrap();
    assert_eq!(c.signed(), "---\nsteplet: 1\ntitle: x\n---\nbody\n");
    assert_eq!(c.header(), "steplet: 1\ntitle: x\n");
    assert_eq!(c.body(), "body\n");
    let s = c.signature().unwrap();
    assert_eq!(s.pem, PEM);
    assert_eq!((s.first_line, s.last_line), (4, 7));
}

#[test]
fn windows_line_endings_and_trailing_spaces_in_the_block_are_cleaned() {
    let raw = format!("---\nsteplet: 1\n{SIG}---\nb\n").replace('\n', " \r\n");
    assert_eq!(read(&raw).unwrap().signature().unwrap().pem, PEM);
}

#[test]
fn two_blocks() {
    assert_eq!(read(&format!("---\nsteplet: 1\n{SIG}{SIG}---\n")), Err(CanonError::SignatureTwice { line: 7 }));
}

#[test]
fn block_not_last_in_the_header() {
    assert_eq!(read(&format!("---\nsteplet: 1\n{SIG}title: x\n---\n")), Err(CanonError::SignatureMisplaced { line: 3 }));
}

#[test]
fn block_in_the_body() {
    assert_eq!(read(&format!("---\nsteplet: 1\n---\nbody\n{SIG}")), Err(CanonError::SignatureMisplaced { line: 6 }));
}

#[test]
fn markers_without_the_key_line() {
    let raw = "---\nsteplet: 1\n  -----BEGIN SSH SIGNATURE-----\n---\n";
    assert_eq!(read(raw), Err(CanonError::SignatureMisplaced { line: 3 }));
    let raw = "---\nsteplet: 1\n---\nsee -----END SSH SIGNATURE-----\n-----END SSH SIGNATURE----- trailing\n";
    assert_eq!(read(raw), Err(CanonError::SignatureMisplaced { line: 5 }));
}

#[test]
fn end_marker_missing() {
    let sig = "signature: |\n  -----BEGIN SSH SIGNATURE-----\n  U1NIU0lHAAAAAQ==\n";
    assert_eq!(read(&format!("---\nsteplet: 1\n{sig}---\n")), Err(CanonError::SignatureBroken { line: 6 }));
}

#[test]
fn the_key_line_must_be_exact() {
    for key in ["signature: >", "signature: |-", "signature: abc", "signature:"] {
        let raw = format!("---\nsteplet: 1\n{}---\n", SIG.replacen("signature: |", key, 1));
        assert_eq!(read(&raw), Err(CanonError::SignatureBroken { line: 3 }), "{key}");
    }
}

#[test]
fn block_lines_must_be_indented_base64() {
    for bad in ["  U1NI U0lH", "U1NIU0lH", "   U1NIU0lH", "  U1NI#U0lH", "  "] {
        let raw = format!("---\nsteplet: 1\n{}---\n", SIG.replacen("  U1NIU0lHAAAAAQ==", bad, 1));
        assert!(matches!(read(&raw), Err(CanonError::SignatureBroken { line: 5 })), "{bad:?}");
    }
}

#[test]
fn with_signature_puts_the_block_last_in_the_header_and_replaces_an_old_one() {
    let unsigned = read("---\nsteplet: 1\ntitle: x\n---\nbody\n").unwrap();
    let signed = unsigned.with_signature("-----BEGIN SSH SIGNATURE-----\r\nU1NIU0lHAAAAAQ==\r\n-----END SSH SIGNATURE-----\r\n\r\n");
    assert_eq!(signed, format!("---\nsteplet: 1\ntitle: x\n{SIG}---\nbody\n"));
    let again = read(&signed).unwrap();
    assert_eq!(again.signed(), unsigned.signed());
    assert_eq!(again.with_signature(PEM), signed);
}
```
Note `"  "` in the last bad case becomes an empty line after the clean-up (trailing spaces go), so it fails as "not a block line" too.

Run: `cargo test -p steplet-core --test canon_signature`
Expected: does not compile (`no method named with_signature`). Add a stub `pub fn with_signature(&self, _pem: &str) -> String { String::new() }` and run again: all 10 tests FAIL (the placeholder search never finds a block).

- [ ] **Step 2: write the real search and `with_signature`** in `canon.rs`, replacing the placeholder `signature_block`:
```rust
pub const BEGIN: &str = "-----BEGIN SSH SIGNATURE-----";
pub const END: &str = "-----END SSH SIGNATURE-----";
pub const SIGNATURE_KEY_LINE: &str = "signature: |";

/// Finds the signature block: (index of `signature: |`, index of the END line, armoured text).
/// `close` is the index of the closing `---` line. Indexes are 0-based.
fn signature_block(lines: &[&str], close: usize) -> Result<Option<(usize, usize, String)>, CanonError> {
    let line_no = |i: usize| i as u32 + 1;
    let mut keys = (1..close).filter(|&i| lines[i].starts_with("signature:"));
    let found = keys.next();
    if let Some(second) = keys.next() {
        return Err(CanonError::SignatureTwice { line: line_no(second) });
    }
    let mut block = None;
    if let Some(k) = found {
        if lines[k] != SIGNATURE_KEY_LINE {
            return Err(CanonError::SignatureBroken { line: line_no(k) });
        }
        if lines.get(k + 1).and_then(|l| l.strip_prefix("  ")) != Some(BEGIN) {
            return Err(CanonError::SignatureBroken { line: line_no(k + 1) });
        }
        let mut pem = String::from(BEGIN);
        pem.push('\n');
        let mut j = k + 2;
        let end = loop {
            if j >= close {
                return Err(CanonError::SignatureBroken { line: line_no(close) });
            }
            let content = lines[j].strip_prefix("  ").unwrap_or("");
            if content == END {
                break j;
            }
            let base64 = |b: u8| b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'=';
            if content.is_empty() || !content.bytes().all(base64) {
                return Err(CanonError::SignatureBroken { line: line_no(j) });
            }
            pem.push_str(content);
            pem.push('\n');
            j += 1;
        };
        if end == k + 2 {
            return Err(CanonError::SignatureBroken { line: line_no(end) });
        }
        if end + 1 != close {
            return Err(CanonError::SignatureMisplaced { line: line_no(k) });
        }
        pem.push_str(END);
        pem.push('\n');
        block = Some((k, end, pem));
    }
    let is_marker = |l: &str| {
        let t = l.trim_start_matches(' ');
        t.starts_with("-----BEGIN SSH SIGNATURE") || t.starts_with("-----END SSH SIGNATURE")
    };
    let inside = |i: usize| block.as_ref().is_some_and(|(k, e, _)| (*k..=*e).contains(&i));
    if let Some(i) = (0..lines.len()).find(|&i| !inside(i) && is_marker(lines[i])) {
        return Err(CanonError::SignatureMisplaced { line: line_no(i) });
    }
    Ok(block)
}
```
Note on `block_not_last_in_the_header`: the block there is complete, then `title: x` follows, so the END line is not followed by `---` and the answer is `SignatureMisplaced` at the key line (3).

And in `impl Canonical`:
```rust
    /// The signed text with `pem` embedded as the signature block, the last thing in the header.
    /// A block already present is left out, so this also replaces an old signature.
    /// Line endings and empty lines in `pem` do not matter.
    pub fn with_signature(&self, pem: &str) -> String {
        let at = self.header.end;
        let mut out = String::with_capacity(self.signed.len() + pem.len() + 64);
        out.push_str(&self.signed[..at]);
        out.push_str(SIGNATURE_KEY_LINE);
        out.push('\n');
        for line in pem.lines().map(str::trim_end).filter(|l| !l.is_empty()) {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(&self.signed[at..]);
        out
    }
```

- [ ] **Step 3: run both canon test files.** `cargo test -p steplet-core --test canon --test canon_signature`
Expected: `12 passed` and `10 passed`.

- [ ] **Step 4: paste into the results file under "## Task 3" and commit.**
```powershell
git add crates/steplet-core docs/milestone-1-phase-1-results.md
git commit -m "File core: find the signature block by fixed markers, refuse it anywhere else"
```

---

### Task 4: the header reader (a small, strict part of YAML)

**Files:**
- Create: `crates/steplet-core/src/header.rs`
- Modify: `crates/steplet-core/src/lib.rs` (add `pub mod header;`)
- Test: `crates/steplet-core/tests/header_tree.rs`

**Interfaces:**
- Produces: `header::tree(header: &str, first_line: u32) -> Result<Vec<Entry>, ParseError>`; `Entry { key: String, line: u32, value: Node }`; `Node::{Map(Vec<Entry>), List(Vec<Scalar>), Scalar(Scalar)}`; `Scalar { text: String, line: u32 }`; `ParseError { line: u32, kind: ParseErrorKind }` with `Display` (`"line N: …"`) and `help() -> Help`. All derive `Clone, Debug, PartialEq, Eq`.
- `ParseErrorKind` (this task's variants): `Tab`, `ControlCharacter { codepoint: u32 }`, `NotAKeyLine`, `BadIndent { expected: usize }`, `EmptyValue { key: String }`, `ListNotIndented { key: String }`, `NotAListItem`, `EmptyListItem`, `DuplicateKey { key: String, first_line: u32 }`, `StartsWithSymbol { symbol: char }`, `ColonInPlainText`, `HashInPlainText`, `UnclosedQuote`, `BadEscape { found: char }`, `TextAfterQuote`. Task 5 adds the field-list variants.

What the reader accepts, and nothing else:
- `name: value` lines; names are lowercase ASCII letters, digits and hyphens, starting with a letter; one or more spaces after the colon.
- `name:` followed by lines indented exactly two more spaces: either a map (more `name:` lines) or a list (`- value` lines).
- Values on one line: plain text, `'single quoted'` (`''` is a quote), or `"double quoted"` (only `\"` and `\\` escapes).
- Blank lines and lines starting with `#` (after spaces) are skipped.
- Refused: tabs; control characters, U+0085, U+2028, U+2029, U+FEFF and anything else YAML does not allow in a document; plain text starting with any of `` - ? : , [ ] { } # & * ! | > ' " % @ ` ``; plain text containing `: ` or ` #` or ending with `:`; anything after a closing quote; duplicate names at one level; anything indented wrongly.

- [ ] **Step 1: write the failing tests.** `crates/steplet-core/tests/header_tree.rs`:
```rust
use steplet_core::header::{Entry, Node, ParseErrorKind as K, Scalar, tree};

fn s(text: &str, line: u32) -> Scalar {
    Scalar { text: text.into(), line }
}
fn e(key: &str, line: u32, value: Node) -> Entry {
    Entry { key: key.into(), line, value }
}
fn refused(h: &str) -> (u32, K) {
    let err = tree(h, 2).unwrap_err();
    (err.line, err.kind)
}

#[test]
fn maps_lists_and_values() {
    let h = "steplet: 1\nallow:\n  commands:\n    - cargo test\n    - \"echo '#1'\"\n# a comment\n\ntitle: 'it''s'\n";
    assert_eq!(
        tree(h, 2).unwrap(),
        vec![
            e("steplet", 2, Node::Scalar(s("1", 2))),
            e("allow", 3, Node::Map(vec![e("commands", 4, Node::List(vec![s("cargo test", 5), s("echo '#1'", 6)]))])),
            e("title", 9, Node::Scalar(s("it's", 9))),
        ]
    );
}

#[test]
fn values_yaml_reads_as_written() {
    let one = |h: &str| match tree(h, 2).unwrap().remove(0).value {
        Node::Scalar(x) => x.text,
        other => panic!("{other:?}"),
    };
    assert_eq!(one("title: a#b\n"), "a#b");
    assert_eq!(one("title:    spaced out\n"), "spaced out");
    assert_eq!(one("title: https://x.org/a:b\n"), "https://x.org/a:b");
    assert_eq!(one("title: \"say \\\"hi\\\" \\\\ ok # :\"\n"), "say \"hi\" \\ ok # :");
    assert_eq!(one("title: a, b [c] {d} e|f g>h\n"), "a, b [c] {d} e|f g>h");
    assert_eq!(one("title: ''\n"), "");
    assert_eq!(one("title: x\u{200b}y\n"), "x\u{200b}y");
}

#[test]
fn refusals_name_the_line() {
    assert_eq!(refused("title:\tx\n"), (2, K::Tab));
    assert_eq!(refused("title: a\u{7}b\n"), (2, K::ControlCharacter { codepoint: 7 }));
    assert_eq!(refused("title: a\u{feff}\n"), (2, K::ControlCharacter { codepoint: 0xfeff }));
    assert_eq!(refused("title: a\u{85}\n"), (2, K::ControlCharacter { codepoint: 0x85 }));
    assert_eq!(refused("title: a\u{2028}\n"), (2, K::ControlCharacter { codepoint: 0x2028 }));
    assert_eq!(refused("Title: x\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("title:x\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("\"title\": x\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("? title\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("...\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("- a\n"), (2, K::NotAKeyLine));
    assert_eq!(refused("title: x\n title2: y\n"), (3, K::BadIndent { expected: 0 }));
    assert_eq!(refused("  title: x\n"), (2, K::BadIndent { expected: 0 }));
    assert_eq!(refused("allow:\n   commands:\n"), (3, K::BadIndent { expected: 2 }));
    assert_eq!(refused("title: a\n  b\n"), (3, K::BadIndent { expected: 0 }));
    assert_eq!(refused("title:\n"), (2, K::EmptyValue { key: "title".into() }));
    assert_eq!(refused("steps:\n- a\n"), (2, K::ListNotIndented { key: "steps".into() }));
    assert_eq!(refused("steps:\n  - a\n  b: c\n"), (4, K::NotAListItem));
    assert_eq!(refused("steps:\n  -\n"), (3, K::EmptyListItem));
    assert_eq!(refused("title: a\ntitle: b\n"), (3, K::DuplicateKey { key: "title".into(), first_line: 2 }));
    assert_eq!(
        refused("allow:\n  read:\n    - a\n  read:\n    - b\n"),
        (5, K::DuplicateKey { key: "read".into(), first_line: 3 })
    );
    for symbol in "-?:,[]{}#&*!|>%@`".chars() {
        assert_eq!(refused(&format!("title: {symbol}x\n")), (2, K::StartsWithSymbol { symbol }), "{symbol}");
    }
    assert_eq!(refused("steps:\n  - - a\n"), (3, K::StartsWithSymbol { symbol: '-' }));
    assert_eq!(refused("title: a: b\n"), (2, K::ColonInPlainText));
    assert_eq!(refused("title: a:\n"), (2, K::ColonInPlainText));
    assert_eq!(refused("steps:\n  - key: v\n"), (3, K::ColonInPlainText));
    assert_eq!(refused("title: a #b\n"), (2, K::HashInPlainText));
    assert_eq!(refused("title: \"abc\n"), (2, K::UnclosedQuote));
    assert_eq!(refused("title: 'abc\n"), (2, K::UnclosedQuote));
    assert_eq!(refused("title: \"abc\\\"\n"), (2, K::UnclosedQuote));
    assert_eq!(refused("title: \"a\\nb\"\n"), (2, K::BadEscape { found: 'n' }));
    assert_eq!(refused("title: \"a\" b\n"), (2, K::TextAfterQuote));
    assert_eq!(refused("title: 'a' # note\n"), (2, K::TextAfterQuote));
}

#[test]
fn every_refusal_has_words_and_help() {
    for h in ["title:\tx\n", "title: a\u{7}\n", "Title: x\n", "  title: x\n", "title:\n", "steps:\n- a\n",
              "steps:\n  - a\n  b: c\n", "steps:\n  -\n", "title: a\ntitle: b\n", "title: [x]\n",
              "title: a: b\n", "title: a #b\n", "title: \"a\n", "title: \"\\n\"\n", "title: \"a\" b\n"] {
        let err = tree(h, 2).unwrap_err();
        assert!(err.to_string().starts_with("line "), "{h:?}");
        assert!(!err.help().allowed.is_empty() && !err.help().example.is_empty(), "{h:?}");
    }
}
```
Add `pub mod header;` to `lib.rs` and a `header.rs` with the types from **Interfaces**, `tree` returning `Ok(Vec::new())`, `Display` writing `"line {}: "`, and `help()` returning empty strings.

Run: `cargo test -p steplet-core --test header_tree`
Expected: FAIL in all four tests.

- [ ] **Step 2: write the reader.** `crates/steplet-core/src/header.rs` (Task 5 adds the field list below it):
```rust
//! The header reader (decision rows 15 and R12). A small, strict part of YAML, read once from the signed bytes:
//! maps and lists of one-line values, plain or quoted. Anything else is refused with its line, what is
//! allowed, and an example. Field types come from the field list (`parse`), never from YAML's guessing.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

use crate::Help;

/// One value and the line it is on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scalar {
    pub text: String,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Map(Vec<Entry>),
    List(Vec<Scalar>),
    Scalar(Scalar),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub line: u32,
    pub value: Node,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: u32,
    pub kind: ParseErrorKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    Tab,
    ControlCharacter { codepoint: u32 },
    NotAKeyLine,
    BadIndent { expected: usize },
    EmptyValue { key: String },
    ListNotIndented { key: String },
    NotAListItem,
    EmptyListItem,
    DuplicateKey { key: String, first_line: u32 },
    StartsWithSymbol { symbol: char },
    ColonInPlainText,
    HashInPlainText,
    UnclosedQuote,
    BadEscape { found: char },
    TextAfterQuote,
}

use ParseErrorKind as K;

fn err(line: u32, kind: ParseErrorKind) -> ParseError {
    ParseError { line, kind }
}

struct Line<'a> {
    no: u32,
    indent: usize,
    text: &'a str,
}

/// Characters YAML allows in a document, minus U+0085, U+2028 and U+2029 (line breaks in older YAML)
/// and U+FEFF (byte-order mark), so no YAML reader can see a different line structure. Tab is refused separately.
fn allowed_char(c: char) -> bool {
    matches!(c, '\u{20}'..='\u{7e}' | '\u{a0}'..='\u{d7ff}' | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
        && !matches!(c, '\u{2028}' | '\u{2029}' | '\u{feff}')
}

/// Reads `header` (the lines between the two `---` lines; its first line is line `first_line` of the file)
/// into a tree, without knowing any field names.
pub fn tree(header: &str, first_line: u32) -> Result<Vec<Entry>, ParseError> {
    let mut lines = Vec::new();
    for (i, raw) in header.lines().enumerate() {
        let no = first_line + i as u32;
        let raw = raw.trim_end_matches(' ');
        if raw.contains('\t') {
            return Err(err(no, K::Tab));
        }
        if let Some(c) = raw.chars().find(|&c| !allowed_char(c)) {
            return Err(err(no, K::ControlCharacter { codepoint: c as u32 }));
        }
        let text = raw.trim_start_matches(' ');
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        lines.push(Line { no, indent: raw.len() - text.len(), text });
    }
    let mut pos = 0;
    map(&lines, &mut pos, 0)
}

fn is_item(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

fn map(lines: &[Line], pos: &mut usize, indent: usize) -> Result<Vec<Entry>, ParseError> {
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(line) = lines.get(*pos).filter(|l| l.indent == indent) {
        let (key, rest) = split_key(line)?;
        if let Some(first) = entries.iter().find(|e| e.key == key) {
            return Err(err(line.no, K::DuplicateKey { key: key.to_string(), first_line: first.line }));
        }
        *pos += 1;
        let value = if !rest.is_empty() {
            Node::Scalar(scalar(rest, line.no)?)
        } else {
            match lines.get(*pos) {
                Some(next) if next.indent > indent => {
                    if next.indent != indent + 2 {
                        return Err(err(next.no, K::BadIndent { expected: indent + 2 }));
                    }
                    if is_item(next.text) {
                        Node::List(list(lines, pos, indent + 2)?)
                    } else {
                        Node::Map(map(lines, pos, indent + 2)?)
                    }
                }
                Some(next) if next.indent == indent && is_item(next.text) => {
                    return Err(err(line.no, K::ListNotIndented { key: key.to_string() }));
                }
                _ => return Err(err(line.no, K::EmptyValue { key: key.to_string() })),
            }
        };
        entries.push(Entry { key: key.to_string(), line: line.no, value });
    }
    if let Some(next) = lines.get(*pos).filter(|l| l.indent > indent) {
        return Err(err(next.no, K::BadIndent { expected: indent }));
    }
    Ok(entries)
}

fn list(lines: &[Line], pos: &mut usize, indent: usize) -> Result<Vec<Scalar>, ParseError> {
    let mut items = Vec::new();
    while let Some(line) = lines.get(*pos).filter(|l| l.indent == indent) {
        if !is_item(line.text) {
            return Err(err(line.no, K::NotAListItem));
        }
        let rest = line.text[1..].trim_start_matches(' ');
        if rest.is_empty() {
            return Err(err(line.no, K::EmptyListItem));
        }
        items.push(scalar(rest, line.no)?);
        *pos += 1;
    }
    if let Some(next) = lines.get(*pos).filter(|l| l.indent > indent) {
        return Err(err(next.no, K::BadIndent { expected: indent }));
    }
    Ok(items)
}

fn split_key<'a>(line: &Line<'a>) -> Result<(&'a str, &'a str), ParseError> {
    let bad = || err(line.no, K::NotAKeyLine);
    let colon = line.text.find(':').ok_or_else(bad)?;
    let key = &line.text[..colon];
    let mut bytes = key.bytes();
    let first_ok = bytes.next().is_some_and(|c| c.is_ascii_lowercase());
    if !first_ok || !bytes.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-') {
        return Err(bad());
    }
    let after = &line.text[colon + 1..];
    if after.is_empty() {
        return Ok((key, ""));
    }
    let value = after.strip_prefix(' ').ok_or_else(bad)?.trim_start_matches(' ');
    Ok((key, value))
}

const SYMBOLS: &str = "-?:,[]{}#&*!|>'\"%@`";

fn scalar(s: &str, line: u32) -> Result<Scalar, ParseError> {
    let text = match s.chars().next() {
        Some('"') => double_quoted(&s[1..], line)?,
        Some('\'') => single_quoted(&s[1..], line)?,
        Some(c) if SYMBOLS.contains(c) => return Err(err(line, K::StartsWithSymbol { symbol: c })),
        _ if s.contains(": ") || s.ends_with(':') => return Err(err(line, K::ColonInPlainText)),
        _ if s.contains(" #") => return Err(err(line, K::HashInPlainText)),
        _ => s.to_string(),
    };
    Ok(Scalar { text, line })
}

fn double_quoted(s: &str, line: u32) -> Result<String, ParseError> {
    let mut out = String::new();
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some((_, q @ ('"' | '\\'))) => out.push(q),
                Some((_, found)) => return Err(err(line, K::BadEscape { found })),
                None => return Err(err(line, K::UnclosedQuote)),
            },
            '"' if s[i + 1..].is_empty() => return Ok(out),
            '"' => return Err(err(line, K::TextAfterQuote)),
            _ => out.push(c),
        }
    }
    Err(err(line, K::UnclosedQuote))
}

fn single_quoted(s: &str, line: u32) -> Result<String, ParseError> {
    let mut out = String::new();
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '\'' {
            out.push(c);
        } else if chars.peek().is_some_and(|&(_, n)| n == '\'') {
            chars.next();
            out.push('\'');
        } else if s[i + 1..].is_empty() {
            return Ok(out);
        } else {
            return Err(err(line, K::TextAfterQuote));
        }
    }
    Err(err(line, K::UnclosedQuote))
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: ", self.line)?;
        match &self.kind {
            K::Tab => write!(f, "a tab character"),
            K::ControlCharacter { codepoint } => write!(f, "an invisible or control character U+{codepoint:04X}"),
            K::NotAKeyLine => write!(f, "this line is not `name: value`"),
            K::BadIndent { expected } => write!(f, "this line should start {expected} spaces in"),
            K::EmptyValue { key } => write!(f, "`{key}` has no value"),
            K::ListNotIndented { key } => write!(f, "the items of `{key}` are not indented"),
            K::NotAListItem => write!(f, "this line is inside a list but does not start with `- `"),
            K::EmptyListItem => write!(f, "an empty list item"),
            K::DuplicateKey { key, first_line } => write!(f, "`{key}` again; it is already on line {first_line}"),
            K::StartsWithSymbol { symbol } => write!(f, "a value starting with `{symbol}` must be in double quotes"),
            K::ColonInPlainText => write!(f, "a value with a colon then a space, or ending in a colon, must be in double quotes"),
            K::HashInPlainText => write!(f, "a value with a space then `#` must be in double quotes"),
            K::UnclosedQuote => write!(f, "a quote that is not closed on the same line"),
            K::BadEscape { found } => write!(f, "`\\{found}` inside double quotes"),
            K::TextAfterQuote => write!(f, "text after the closing quote"),
        }
    }
}

impl ParseError {
    pub fn help(&self) -> Help {
        let (allowed, example) = match &self.kind {
            K::Tab => ("indent with spaces, two per level; no tab characters anywhere in the header", "allow:\n  commands:\n    - cargo test"),
            K::ControlCharacter { .. } => ("ordinary visible text; control characters, byte-order marks and Unicode line separators are not allowed in the header", "title: Run the tests"),
            K::NotAKeyLine => ("one `name: value` per line; names are lowercase letters, digits and hyphens, with a space after the colon", "title: Run the tests"),
            K::BadIndent { .. } => ("each level indented exactly two spaces more than the line it belongs to; every value on one line", "repository:\n  url: https://github.com/octocat/Hello-World.git"),
            K::EmptyValue { .. } => ("a value after the colon, or lines indented two spaces below it; leave out a field you do not need", "title: Run the tests"),
            K::ListNotIndented { .. } => ("list items indented two spaces more than the list's name", "steps:\n  - Build the project."),
            K::NotAListItem => ("every line of a list starts with `- `", "steps:\n  - Build the project.\n  - Run the tests."),
            K::EmptyListItem => ("text after every `- `", "steps:\n  - Run the tests."),
            K::DuplicateKey { .. } => ("each name once at each level", "title: Run the tests"),
            K::StartsWithSymbol { .. } => ("text starting with a letter, digit or other ordinary character; otherwise put the value in double quotes", "title: \"[draft] Run the tests\""),
            K::ColonInPlainText => ("double quotes around text that contains a colon followed by a space, or ends with a colon", "title: \"Note: run the tests\""),
            K::HashInPlainText => ("double quotes around text that contains a space followed by #", "title: \"Fix issue #12\""),
            K::UnclosedQuote => ("the closing quote on the same line", "title: \"Run the tests\""),
            K::BadEscape { .. } => ("inside double quotes, a backslash only before a quote (\\\") or another backslash (\\\\)", "title: \"Say \\\"hi\\\"\""),
            K::TextAfterQuote => ("nothing after the closing quote, not even a comment", "title: \"Run the tests\""),
        };
        Help { allowed, example }
    }
}
```

- [ ] **Step 3: run the tests.** `cargo test -p steplet-core --test header_tree`
Expected: `4 passed`.

- [ ] **Step 4: paste into the results file under "## Task 4" and commit.**
```powershell
git add crates/steplet-core docs/milestone-1-phase-1-results.md
git commit -m "File core: a strict reader for a small part of YAML, with line numbers and help"
```

---

### Task 5: the field list (`Steplet`) and the sender's example file

**Files:**
- Modify: `crates/steplet-core/src/header.rs` (field list, new error kinds and their words)
- Create: `examples/run-tests.steplet`
- Test: `crates/steplet-core/tests/header_fields.rs`

**Interfaces:**
- Consumes: `canon::Canonical` (Task 2), `date::Date` (Task 1), `tree` (Task 4).
- Produces: `header::parse(c: &Canonical) -> Result<Steplet, ParseError>`; `Steplet { title: Scalar, sender: Scalar, expires: Date, expires_line: u32, repository: Option<Repository>, steps: Vec<Scalar>, allow: Allow, never: Vec<Scalar>, body: String }`; `Repository { url: Scalar, commit: Scalar }`; `Allow { commands, domains, read, write: Vec<Scalar> }` (`Default`); `header::{TOP_KEYS, REPOSITORY_KEYS, ALLOW_KEYS}: &[&str]`. All derive `Clone, Debug, PartialEq, Eq`.
- New `ParseErrorKind` variants: `UnknownKey { key: String, allowed: &'static [&'static str] }`, `MissingKey { key: &'static str }`, `FirstKeyNotSteplet`, `UnsupportedVersion { found: String }`, `ExpectedText { key: String }`, `ExpectedList { key: String }`, `ExpectedMap { key: String }`, `BadDate`, `BadUrl`, `BadCommit`, `BadDomain`, `BadFolder`.

Value rules (section 4, F1):
- `steplet`: first entry, exactly `1`. Checked before anything else, so a newer file gets "needs a newer Steplet", not a list of unknown fields.
- `url`: `https://`, then a host name, then optionally a path of `A–Z a–z 0–9 . _ ~ / -`. No user name, port, `?` or `#`.
- Host names (in `url` and `domains`): lowercase ASCII letters, digits and hyphens in dot-separated parts of 1 to 63 characters, not starting or ending with a hyphen; at least two parts; the last part has a letter (so `127.0.0.1` is refused); at most 253 characters.
- `commit`: 40 or 64 lowercase hexadecimal characters.
- `read` and `write` items: `.` or a relative path of parts made of `A–Z a–z 0–9 . _ -`, separated by `/`, no empty part, no `.` or `..` part, no leading or trailing `/`.
- Empty text (`""`) is refused wherever text is expected.

- [ ] **Step 1: write the failing tests.** `crates/steplet-core/tests/header_fields.rs`:
```rust
use steplet_core::canon::canonicalize;
use steplet_core::date::Date;
use steplet_core::header::{ALLOW_KEYS, ParseError, ParseErrorKind as K, Steplet, TOP_KEYS, parse};

const GOOD: &str = "---
steplet: 1
title: Run the tests
sender: Ana Lee
expires: 2026-10-25
repository:
  url: https://github.com/octocat/Hello-World.git
  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
steps:
  - Build it.
  - Test it.
allow:
  commands:
    - cargo test
  domains:
    - crates.io
  read:
    - .
  write:
    - target/debug
never:
  - Do not push.
---
# Body
";

fn read(f: &str) -> Result<Steplet, ParseError> {
    parse(&canonicalize(f.as_bytes()).unwrap())
}
fn kind(from: &str, to: &str) -> K {
    let f = GOOD.replacen(from, to, 1);
    assert_ne!(f, GOOD, "the replacement did not apply: {from:?}");
    read(&f).unwrap_err().kind
}
fn texts(v: &[steplet_core::header::Scalar]) -> Vec<&str> {
    v.iter().map(|s| s.text.as_str()).collect()
}

#[test]
fn reads_every_field_with_its_line() {
    let s = read(GOOD).unwrap();
    assert_eq!((s.title.text.as_str(), s.title.line), ("Run the tests", 3));
    assert_eq!(s.sender.text, "Ana Lee");
    assert_eq!((s.expires, s.expires_line), (Date::parse("2026-10-25").unwrap(), 5));
    let r = s.repository.unwrap();
    assert_eq!(r.url.text, "https://github.com/octocat/Hello-World.git");
    assert_eq!((r.commit.text.as_str(), r.commit.line), ("7fd1a60b01f91b314f59955a4e4d4e80d8edf11d", 8));
    assert_eq!(texts(&s.steps), ["Build it.", "Test it."]);
    assert_eq!(s.steps[1].line, 11);
    assert_eq!(texts(&s.allow.commands), ["cargo test"]);
    assert_eq!(texts(&s.allow.domains), ["crates.io"]);
    assert_eq!(texts(&s.allow.read), ["."]);
    assert_eq!(texts(&s.allow.write), ["target/debug"]);
    assert_eq!(texts(&s.never), ["Do not push."]);
    assert_eq!(s.body, "# Body\n");
}

#[test]
fn optional_parts_may_be_left_out() {
    let s = read("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n  - a\n---\n").unwrap();
    assert!(s.repository.is_none());
    assert!(s.allow.commands.is_empty() && s.allow.domains.is_empty() && s.allow.read.is_empty() && s.allow.write.is_empty());
    assert!(s.never.is_empty());
    assert_eq!(s.body, "");
}

#[test]
fn the_version_comes_first_and_is_1() {
    assert_eq!(kind("steplet: 1\n", "steplet: 2\n"), K::UnsupportedVersion { found: "2".into() });
    assert_eq!(kind("steplet: 1\ntitle: Run the tests\n", "title: Run the tests\nsteplet: 1\n"), K::FirstKeyNotSteplet);
    assert_eq!(read("---\n---\n").unwrap_err(), ParseError { line: 1, kind: K::MissingKey { key: "steplet" } });
    assert_eq!(kind("steplet: 1\n", "steplet: 2\ncolour: green\n"), K::UnsupportedVersion { found: "2".into() });
}

#[test]
fn unknown_and_missing_fields() {
    assert_eq!(kind("steps:\n", "colour: green\nsteps:\n"), K::UnknownKey { key: "colour".into(), allowed: TOP_KEYS });
    assert_eq!(kind("  read:\n", "  network:\n    - x\n  read:\n"), K::UnknownKey { key: "network".into(), allowed: ALLOW_KEYS });
    assert_eq!(kind("sender: Ana Lee\n", ""), K::MissingKey { key: "sender" });
    assert_eq!(kind("expires: 2026-10-25\n", ""), K::MissingKey { key: "expires" });
    assert_eq!(kind("steps:\n  - Build it.\n  - Test it.\n", ""), K::MissingKey { key: "steps" });
    assert_eq!(kind("  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d\n", ""), K::MissingKey { key: "commit" });
}

#[test]
fn wrong_shapes() {
    assert_eq!(kind("title: Run the tests\n", "title:\n  - a\n"), K::ExpectedText { key: "title".into() });
    assert_eq!(kind("title: Run the tests\n", "title: \"\"\n"), K::EmptyValue { key: "title".into() });
    assert_eq!(kind("steps:\n  - Build it.\n  - Test it.\n", "steps: Build it.\n"), K::ExpectedList { key: "steps".into() });
    assert_eq!(kind("  - Build it.\n", "  - ''\n"), K::EmptyListItem);
    assert_eq!(
        kind("repository:\n  url: https://github.com/octocat/Hello-World.git\n  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d\n", "repository: x\n"),
        K::ExpectedMap { key: "repository".into() }
    );
}

#[test]
fn typed_values() {
    assert_eq!(kind("2026-10-25", "2026-02-30"), K::BadDate);
    assert_eq!(kind("2026-10-25", "25 Oct 2026"), K::BadDate);
    for url in ["http://github.com/x", "https://user@github.com/x", "https://GitHub.com/x", "https://127.0.0.1/x",
                "https://github.com:443/x", "https://github.com/x?y=1", "https://localhost/x", "https://github.com/a b"] {
        assert_eq!(kind("https://github.com/octocat/Hello-World.git", url), K::BadUrl, "{url}");
    }
    for commit in ["7fd1a60", "7FD1A60B01F91B314F59955A4E4D4E80D8EDF11D", "7fd1a60b01f91b314f59955a4e4d4e80d8edf11g"] {
        assert_eq!(kind("7fd1a60b01f91b314f59955a4e4d4e80d8edf11d", commit), K::BadCommit, "{commit}");
    }
    for domain in ["Crates.io", "localhost", "\"*.crates.io\"", "crates.io:443", "https://crates.io", "\"-a.io\"", "1.2.3.4"] {
        assert_eq!(kind("    - crates.io\n", &format!("    - {domain}\n")), K::BadDomain, "{domain}");
    }
    for folder in ["../x", "/etc", "a\\b", "a//b", "target/", "./target", "a/../b", "my folder"] {
        // Single quotes, so the backslash in `a\b` is not read as an escape.
        assert_eq!(kind("    - target/debug\n", &format!("    - '{folder}'\n")), K::BadFolder, "{folder}");
    }
}

#[test]
fn every_field_error_has_words_and_help() {
    let kinds = [
        kind("steps:\n", "colour: green\nsteps:\n"), kind("sender: Ana Lee\n", ""),
        kind("steplet: 1\ntitle: Run the tests\n", "title: Run the tests\nsteplet: 1\n"),
        kind("steplet: 1\n", "steplet: 2\n"), kind("title: Run the tests\n", "title:\n  - a\n"),
        kind("steps:\n  - Build it.\n  - Test it.\n", "steps: Build it.\n"), kind("2026-10-25", "2026-02-30"),
        kind("https://github.com/octocat/Hello-World.git", "http://x.org"), kind("7fd1a60b01f91b314f59955a4e4d4e80d8edf11d", "7fd"),
        kind("    - crates.io\n", "    - Crates.io\n"), kind("    - target/debug\n", "    - \"..\"\n"),
    ];
    for k in kinds {
        let e = ParseError { line: 4, kind: k };
        assert!(e.to_string().starts_with("line 4: "), "{e:?}");
        assert!(!e.help().allowed.is_empty() && !e.help().example.is_empty(), "{e:?}");
    }
}

#[test]
fn the_example_file_reads() {
    let s = read(include_str!("../../../examples/run-tests.steplet")).unwrap();
    assert_eq!(s.steps.len(), 3);
    assert!(s.repository.is_some());
}
```
Create `examples/run-tests.steplet`:
```
---
steplet: 1
title: Count the lines of the README in Hello-World
sender: Ana Lee
expires: 2026-10-25
repository:
  url: https://github.com/octocat/Hello-World.git
  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
steps:
  - Check that the README file is there.
  - Count the lines in the README file.
  - Report the count.
allow:
  commands:
    - wc -l README
never:
  - Do not change any file.
  - Do not use the network.
---
# Count the README lines

The repository is already checked out at the commit above.

1. Check that `README` exists.
2. Run `wc -l README`.
3. Report the number it prints, and nothing else.
```
Add stubs so the test file compiles: the new error variants (with one `_ => …` arm in `Display` writing nothing and in `help()` returning empty strings), the three key lists, the types from **Interfaces**, and `parse` returning `Err(ParseError { line: 1, kind: ParseErrorKind::BadDate })`.

Run: `cargo test -p steplet-core --test header_fields`
Expected: all eight tests FAIL.

- [ ] **Step 2: write the field list.** Add to `header.rs` (imports at the top: `use crate::canon::Canonical;`, `use crate::date::Date;`):
```rust
pub const TOP_KEYS: &[&str] = &["steplet", "title", "sender", "expires", "repository", "steps", "allow", "never"];
pub const REPOSITORY_KEYS: &[&str] = &["url", "commit"];
pub const ALLOW_KEYS: &[&str] = &["commands", "domains", "read", "write"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Steplet {
    pub title: Scalar,
    pub sender: Scalar,
    pub expires: Date,
    pub expires_line: u32,
    pub repository: Option<Repository>,
    pub steps: Vec<Scalar>,
    pub allow: Allow,
    pub never: Vec<Scalar>,
    /// The cleaned-up, signed body: what the agent is given.
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repository {
    pub url: Scalar,
    pub commit: Scalar,
}

/// What the file asks to be allowed. Commands are kept as written; their meaning is phase 2's (R8).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Allow {
    pub commands: Vec<Scalar>,
    pub domains: Vec<Scalar>,
    pub read: Vec<Scalar>,
    pub write: Vec<Scalar>,
}

/// Reads the header of a cleaned-up file, once, from the signed bytes (row 15).
pub fn parse(c: &Canonical) -> Result<Steplet, ParseError> {
    let entries = tree(c.header(), Canonical::HEADER_FIRST_LINE)?;
    let first = entries.first().ok_or(err(1, K::MissingKey { key: "steplet" }))?;
    if first.key != "steplet" {
        return Err(err(first.line, K::FirstKeyNotSteplet));
    }
    let version = text(first)?;
    if version.text != "1" {
        return Err(err(version.line, K::UnsupportedVersion { found: version.text }));
    }
    let top = Fields::new(&entries, TOP_KEYS)?;
    let expires = text(top.required("expires", 1)?)?;
    let repository = match top.optional("repository") {
        None => None,
        Some(e) => {
            let f = Fields::new(map_of(e)?, REPOSITORY_KEYS)?;
            let url = text(f.required("url", e.line)?)?;
            if !is_https_url(&url.text) {
                return Err(err(url.line, K::BadUrl));
            }
            let commit = text(f.required("commit", e.line)?)?;
            if !is_commit(&commit.text) {
                return Err(err(commit.line, K::BadCommit));
            }
            Some(Repository { url, commit })
        }
    };
    let allow = match top.optional("allow") {
        None => Allow::default(),
        Some(e) => {
            let f = Fields::new(map_of(e)?, ALLOW_KEYS)?;
            Allow {
                commands: f.list("commands")?,
                domains: checked(f.list("domains")?, is_domain, K::BadDomain)?,
                read: checked(f.list("read")?, is_folder, K::BadFolder)?,
                write: checked(f.list("write")?, is_folder, K::BadFolder)?,
            }
        }
    };
    Ok(Steplet {
        title: text(top.required("title", 1)?)?,
        sender: text(top.required("sender", 1)?)?,
        expires: Date::parse(&expires.text).ok_or(err(expires.line, K::BadDate))?,
        expires_line: expires.line,
        repository,
        steps: list_of(top.required("steps", 1)?)?,
        allow,
        never: top.list("never")?,
        body: c.body().to_string(),
    })
}

struct Fields<'a> {
    entries: &'a [Entry],
}

impl<'a> Fields<'a> {
    fn new(entries: &'a [Entry], allowed: &'static [&'static str]) -> Result<Self, ParseError> {
        if let Some(e) = entries.iter().find(|e| !allowed.contains(&e.key.as_str())) {
            return Err(err(e.line, K::UnknownKey { key: e.key.clone(), allowed }));
        }
        Ok(Fields { entries })
    }
    fn optional(&self, key: &str) -> Option<&'a Entry> {
        self.entries.iter().find(|e| e.key == key)
    }
    fn required(&self, key: &'static str, parent_line: u32) -> Result<&'a Entry, ParseError> {
        self.optional(key).ok_or(err(parent_line, K::MissingKey { key }))
    }
    fn list(&self, key: &str) -> Result<Vec<Scalar>, ParseError> {
        self.optional(key).map_or(Ok(Vec::new()), list_of)
    }
}

fn text(e: &Entry) -> Result<Scalar, ParseError> {
    match &e.value {
        Node::Scalar(s) if s.text.is_empty() => Err(err(e.line, K::EmptyValue { key: e.key.clone() })),
        Node::Scalar(s) => Ok(s.clone()),
        _ => Err(err(e.line, K::ExpectedText { key: e.key.clone() })),
    }
}

fn list_of(e: &Entry) -> Result<Vec<Scalar>, ParseError> {
    match &e.value {
        Node::List(items) => match items.iter().find(|s| s.text.is_empty()) {
            Some(empty) => Err(err(empty.line, K::EmptyListItem)),
            None => Ok(items.clone()),
        },
        _ => Err(err(e.line, K::ExpectedList { key: e.key.clone() })),
    }
}

fn map_of(e: &Entry) -> Result<&[Entry], ParseError> {
    match &e.value {
        Node::Map(m) => Ok(m),
        _ => Err(err(e.line, K::ExpectedMap { key: e.key.clone() })),
    }
}

fn checked(items: Vec<Scalar>, ok: fn(&str) -> bool, kind: ParseErrorKind) -> Result<Vec<Scalar>, ParseError> {
    match items.iter().find(|s| !ok(&s.text)) {
        Some(bad) => Err(err(bad.line, kind)),
        None => Ok(items),
    }
}

fn is_domain(s: &str) -> bool {
    let label_ok = |l: &str| {
        !l.is_empty()
            && l.len() <= 63
            && !l.starts_with('-')
            && !l.ends_with('-')
            && l.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    };
    s.len() <= 253
        && s.split('.').count() >= 2
        && s.split('.').all(label_ok)
        && s.rsplit('.').next().is_some_and(|tld| tld.bytes().any(|b| b.is_ascii_lowercase()))
}

fn is_https_url(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("https://") else { return false };
    let (host, path) = rest.find('/').map_or((rest, ""), |i| (&rest[..i], &rest[i..]));
    is_domain(host) && path.bytes().all(|b| b.is_ascii_alphanumeric() || b"._~/-".contains(&b))
}

fn is_commit(s: &str) -> bool {
    (s.len() == 40 || s.len() == 64) && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn is_folder(s: &str) -> bool {
    s == "."
        || s.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
```
Add the new variants to `ParseErrorKind`, and their arms to `Display`:
```rust
            K::UnknownKey { key, allowed } => {
                write!(f, "unknown name `{key}`; allowed here: ")?;
                for (i, a) in allowed.iter().enumerate() {
                    write!(f, "{}{a}", if i == 0 { "" } else { ", " })?;
                }
                Ok(())
            }
            K::MissingKey { key } => write!(f, "`{key}` is missing"),
            K::FirstKeyNotSteplet => write!(f, "the header must start with `steplet: 1`"),
            K::UnsupportedVersion { found } => write!(f, "format version {found}; this Steplet reads version 1 only"),
            K::ExpectedText { key } => write!(f, "`{key}` must be one line of text"),
            K::ExpectedList { key } => write!(f, "`{key}` must be a list"),
            K::ExpectedMap { key } => write!(f, "`{key}` must have named fields below it"),
            K::BadDate => write!(f, "not a date written YYYY-MM-DD"),
            K::BadUrl => write!(f, "not a public https address Steplet accepts"),
            K::BadCommit => write!(f, "not a full commit hash"),
            K::BadDomain => write!(f, "not a host name Steplet accepts"),
            K::BadFolder => write!(f, "not a folder inside the repository"),
```
and to `help()`:
```rust
            K::UnknownKey { .. } => ("only the names listed in this message", "title: Run the tests"),
            K::MissingKey { key } => ("every required field", match *key {
                "steplet" => "steplet: 1",
                "sender" => "sender: Ana Lee",
                "expires" => "expires: 2026-10-25",
                "steps" => "steps:\n  - Run the tests.",
                "url" => "  url: https://github.com/octocat/Hello-World.git",
                "commit" => "  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d",
                _ => "title: Run the tests",
            }),
            K::FirstKeyNotSteplet => ("`steplet: 1` as the first line of the header", "---\nsteplet: 1\ntitle: Run the tests"),
            K::UnsupportedVersion { .. } => ("format version 1", "steplet: 1"),
            K::ExpectedText { .. } => ("a single line of text", "title: Run the tests"),
            K::ExpectedList { .. } => ("a list, one `- ` item per line, indented two spaces", "steps:\n  - Run the tests."),
            K::ExpectedMap { .. } => ("named fields indented two spaces below", "repository:\n  url: https://github.com/octocat/Hello-World.git\n  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d"),
            K::BadDate => ("a real calendar date written YYYY-MM-DD", "expires: 2026-10-25"),
            K::BadUrl => ("https://, a lowercase host name, then a path of letters, digits and . _ ~ / -; no user name, port, ? or #", "  url: https://github.com/octocat/Hello-World.git"),
            K::BadCommit => ("the full commit hash: 40 (or 64) lowercase hexadecimal characters", "  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d"),
            K::BadDomain => ("an exact lowercase host name with at least one dot; no *, scheme, port or path", "    - crates.io"),
            K::BadFolder => ("a folder inside the repository written with /, no .. and no leading or trailing /; . means all of it", "    - target/debug"),
```

- [ ] **Step 3: run every core test so far.** `cargo test -p steplet-core`
Expected: all pass (`date` 5, `canon` 12, `canon_signature` 10, `header_tree` 4, `header_fields` 8).

- [ ] **Step 4: paste into the results file under "## Task 5" and commit.**
```powershell
git add crates/steplet-core examples docs/milestone-1-phase-1-results.md
git commit -m "File core: the header's field list, typed values, and the sender's example file"
```

---

### Task 6: the differential test against `saphyr-parser` (yes: B1)

**Files:**
- Modify: `crates/steplet-core/Cargo.toml` (dev-dependencies)
- Test: `crates/steplet-core/tests/differential.rs`
- Create: `Cargo.lock` (first dependency)

**Interfaces:** Consumes `header::{tree, Entry, Node}`. Produces nothing new.

**Question this answers:** whenever our reader accepts a header, does a full YAML 1.2 reader see exactly the same names, lists and values? (Row 15's attack is two readers disagreeing.)

- [ ] **Step 1: ask the user for yes B1**, then add to `crates/steplet-core/Cargo.toml`:
```toml
[dev-dependencies]
proptest = "=1.11.0"
saphyr-parser = "=0.1.0"
```

- [ ] **Step 2: write the test.** `crates/steplet-core/tests/differential.rs`:
```rust
//! Whatever our header reader accepts, saphyr-parser (a full YAML 1.2 reader) must read the same way.

use proptest::prelude::*;
use saphyr_parser::{Event, Parser};
use steplet_core::header::{Entry, Node, tree};

#[derive(Debug, PartialEq)]
enum Yaml {
    Map(Vec<(String, Yaml)>),
    List(Vec<Yaml>),
    Text(String),
}

fn ours(entries: &[Entry]) -> Yaml {
    Yaml::Map(entries.iter().map(|e| (e.key.clone(), node(&e.value))).collect())
}

fn node(n: &Node) -> Yaml {
    match n {
        Node::Map(es) => ours(es),
        Node::List(items) => Yaml::List(items.iter().map(|s| Yaml::Text(s.text.clone())).collect()),
        Node::Scalar(s) => Yaml::Text(s.text.clone()),
    }
}

/// What saphyr-parser makes of the same text. Tags, aliases and a second document count as a disagreement.
fn theirs(src: &str) -> Result<Yaml, String> {
    let mut events = Vec::new();
    for item in Parser::new_from_str(src) {
        let (ev, _span) = item.map_err(|e| format!("{e:?}"))?;
        if !matches!(ev, Event::StreamStart | Event::StreamEnd | Event::DocumentStart(_) | Event::DocumentEnd | Event::Nothing) {
            events.push(ev);
        }
    }
    let mut i = 0;
    let y = build(&events, &mut i)?;
    if i != events.len() {
        return Err(format!("{} events left over", events.len() - i));
    }
    Ok(y)
}

fn build(ev: &[Event], i: &mut usize) -> Result<Yaml, String> {
    let e = ev.get(*i).ok_or("ran out of events")?;
    *i += 1;
    match e {
        Event::Scalar(v, _, _, None) => Ok(Yaml::Text(v.to_string())),
        Event::SequenceStart(_, None) => {
            let mut items = Vec::new();
            while !matches!(ev.get(*i), Some(Event::SequenceEnd)) {
                items.push(build(ev, i)?);
            }
            *i += 1;
            Ok(Yaml::List(items))
        }
        Event::MappingStart(_, None) => {
            let mut pairs = Vec::new();
            while !matches!(ev.get(*i), Some(Event::MappingEnd)) {
                let Yaml::Text(k) = build(ev, i)? else { return Err("a key that is not text".into()) };
                pairs.push((k, build(ev, i)?));
            }
            *i += 1;
            Ok(Yaml::Map(pairs))
        }
        other => Err(format!("not allowed here: {other:?}")),
    }
}

fn agree(h: &str) -> bool {
    match tree(h, 2) {
        Ok(es) => {
            assert_eq!(theirs(h), Ok(ours(&es)), "header:\n{h}");
            true
        }
        Err(_) => false,
    }
}

const CORPUS: &[&str] = &[
    "steplet: 1\ntitle: plain text\n",
    "title: a#b\n",
    "title:    spaced\n",
    "title: https://x.org/a:b\n",
    "title: 'it''s'\n",
    "title: \"say \\\"hi\\\" \\\\\"\n",
    "title: a, b [c] {d} e|f g>h\n",
    "title: yes\nother: no\nnum: 012\nnul: null\ntilde: ~\nfloat: 1e3\n",
    "title: \"\"\n",
    "allow:\n  commands:\n    - cargo test\n    - 'echo \"#1\"'\n  domains:\n    - crates.io\n",
    "# comment\n\nsteps:\n  - one\n  # inner comment\n  - two\n",
    "title: x\u{200b}y\u{202e}z\n",
    "title: ...\n",
    "title: a -b\n",
    // The rest are refused by ours; they are here so the test shows they stay refused.
    "title: a # b\n",
    "title: key: v\n",
    "steps:\n- a\n",
    "title: &a x\nother: *a\n",
    "title: !!str x\n",
    "title: |\n  x\n",
    "title: [a, b]\n",
    "title: a\ntitle: b\n",
    "title: a\n  continued\n",
];

#[test]
fn every_accepted_corpus_header_reads_the_same() {
    let accepted = CORPUS.iter().filter(|h| agree(h)).count();
    assert_eq!(accepted, 14, "the first 14 corpus entries are accepted, the last 9 refused");
    for file in [include_str!("../../../examples/run-tests.steplet")] {
        let c = steplet_core::canon::canonicalize(file.as_bytes()).unwrap();
        assert!(agree(c.header()));
    }
}

#[derive(Clone, Debug)]
enum Gen {
    Text(String, u8),
    List(Vec<(String, u8)>),
    Map(Vec<Gen>),
}

fn scalar_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(vec![
            "a", "Z", "0", " ", ":", "#", "-", "'", "\"", "\\", "[", "]", "{", "}", ",", "&", "*", "!", "|", ">",
            "%", "@", "`", "?", "é", "\u{200b}", "...", "~", "null",
        ]),
        1..8,
    )
    .prop_map(|v| v.concat())
}

fn value() -> impl Strategy<Value = Gen> {
    let leaf = prop_oneof![
        (scalar_text(), 0u8..3).prop_map(|(t, s)| Gen::Text(t, s)),
        prop::collection::vec((scalar_text(), 0u8..3), 1..4).prop_map(Gen::List),
    ];
    leaf.prop_recursive(3, 24, 4, |inner| prop::collection::vec(inner, 1..4).prop_map(Gen::Map))
}

/// Style 0 is plain, 1 single-quoted, 2 double-quoted. `force_quotes` turns plain into double-quoted.
fn quote(t: &str, style: u8, force_quotes: bool) -> String {
    match if force_quotes && style == 0 { 2 } else { style } {
        0 => t.to_string(),
        1 => format!("'{}'", t.replace('\'', "''")),
        _ => format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\"")),
    }
}

/// Keys are k0, k1, … so they never repeat (duplicates have their own unit tests).
fn render(items: &[Gen], indent: usize, force_quotes: bool, out: &mut String) {
    let pad = " ".repeat(indent);
    for (i, g) in items.iter().enumerate() {
        match g {
            Gen::Text(t, s) => out.push_str(&format!("{pad}k{i}: {}\n", quote(t, *s, force_quotes))),
            Gen::List(list) => {
                out.push_str(&format!("{pad}k{i}:\n"));
                for (t, s) in list {
                    out.push_str(&format!("{pad}  - {}\n", quote(t, *s, force_quotes)));
                }
            }
            Gen::Map(children) => {
                out.push_str(&format!("{pad}k{i}:\n"));
                render(children, indent + 2, force_quotes, out);
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5000))]

    #[test]
    fn whatever_we_accept_saphyr_reads_the_same(items in prop::collection::vec(value(), 1..5)) {
        let mut h = String::new();
        render(&items, 0, false, &mut h);
        agree(&h);
    }

    #[test]
    fn anything_quoted_is_accepted_and_agrees(items in prop::collection::vec(value(), 1..5)) {
        let mut h = String::new();
        render(&items, 0, true, &mut h);
        prop_assert!(agree(&h), "refused although every value is quoted:\n{}", h);
    }
}
```
The second property makes sure the first is not empty by accident: when every value is quoted, our reader must accept, so the comparison really runs on thousands of headers.

- [ ] **Step 3: run it.** `cargo test -p steplet-core --test differential`
Expected: `3 passed`. A failure is a real disagreement: shrink it (proptest prints the smallest header), add that header to `CORPUS`, and fix **our** reader so it refuses or reads it the YAML way. Record each one in the results. Never weaken the comparison. If `saphyr-parser`'s names differ from section 2 (for example the `Event` fields), read `https://docs.rs/saphyr-parser/0.1.0` and use its names; record the change.

- [ ] **Step 4: paste into the results file under "## Task 6" and commit.**
```powershell
git add Cargo.lock crates/steplet-core docs/milestone-1-phase-1-results.md
git commit -m "File core: test that a full YAML reader agrees with ours on everything we accept"
```

---

### Task 7: size limits

**Files:**
- Modify: `crates/steplet-core/src/limits.rs`
- Test: `crates/steplet-core/tests/limits.rs`

**Interfaces:**
- Consumes: `header::{Steplet, Scalar}`.
- Produces: `limits::check(s: &Steplet) -> Vec<LimitBreach>`; `LimitBreach { what: Limited, line: u32, found: usize, max: usize }` with `Display` and `help() -> Help`; `Limited::{Steps, Commands, Domains, ReadFolders, WriteFolders, Never, CommandLength}`. For a list over its limit, `line` is the line of the first item over the limit.

- [ ] **Step 1: write the failing tests.** `crates/steplet-core/tests/limits.rs`:
```rust
use steplet_core::canon::canonicalize;
use steplet_core::header::{Steplet, parse};
use steplet_core::limits::{LimitBreach, Limited, check};

fn file(steps: usize, commands: &[String], never: usize) -> Steplet {
    let mut f = String::from("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n");
    for i in 0..steps {
        f.push_str(&format!("  - step {i}\n"));
    }
    if !commands.is_empty() {
        f.push_str("allow:\n  commands:\n");
        for c in commands {
            f.push_str(&format!("    - \"{c}\"\n"));
        }
    }
    if never > 0 {
        f.push_str("never:\n");
        for i in 0..never {
            f.push_str(&format!("  - never {i}\n"));
        }
    }
    f.push_str("---\n");
    parse(&canonicalize(f.as_bytes()).unwrap()).unwrap()
}

#[test]
fn at_the_limits_nothing_is_reported() {
    let commands: Vec<String> = (0..25).map(|i| format!("{i:0>299}")).collect();
    assert_eq!(check(&file(30, &commands, 25)), vec![]);
}

#[test]
fn one_over_each_limit_names_the_first_line_over() {
    assert_eq!(check(&file(31, &[], 0)), vec![LimitBreach { what: Limited::Steps, line: 37, found: 31, max: 30 }]);
    let commands: Vec<String> = (0..26).map(|i| format!("c{i}")).collect();
    let b = check(&file(1, &commands, 0));
    assert_eq!(b, vec![LimitBreach { what: Limited::Commands, line: 35, found: 26, max: 25 }]);
    let b = check(&file(1, &[], 26));
    assert_eq!(b, vec![LimitBreach { what: Limited::Never, line: 34, found: 26, max: 25 }]);
}

#[test]
fn a_command_of_300_characters_is_too_long_counted_in_characters() {
    let b = check(&file(1, &["é".repeat(300)], 0));
    assert_eq!(b, vec![LimitBreach { what: Limited::CommandLength, line: 10, found: 300, max: 299 }]);
    assert_eq!(check(&file(1, &["é".repeat(299)], 0)), vec![]);
}

#[test]
fn domains_and_folders() {
    let mut f = String::from("---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n  - a\nallow:\n  domains:\n");
    for i in 0..16 {
        f.push_str(&format!("    - d{i}.example.org\n"));
    }
    f.push_str("  read:\n");
    for i in 0..11 {
        f.push_str(&format!("    - r{i}\n"));
    }
    f.push_str("  write:\n");
    for i in 0..11 {
        f.push_str(&format!("    - w{i}\n"));
    }
    f.push_str("---\n");
    let s = parse(&canonicalize(f.as_bytes()).unwrap()).unwrap();
    let what: Vec<Limited> = check(&s).into_iter().map(|b| b.what).collect();
    assert_eq!(what, [Limited::Domains, Limited::ReadFolders, Limited::WriteFolders]);
}

#[test]
fn words_and_help() {
    let b = LimitBreach { what: Limited::Steps, line: 37, found: 31, max: 30 };
    assert_eq!(b.to_string(), "line 37: 31 steps; at most 30 are allowed");
    let b = LimitBreach { what: Limited::CommandLength, line: 10, found: 300, max: 299 };
    assert_eq!(b.to_string(), "line 10: this command is 300 characters long; it must be under 300");
    assert!(!b.help().allowed.is_empty());
}
```
Line numbers: steps start on line 7, so step 31 is on line 37; with one step, `allow:` is line 8, `commands:` 9, the first command 10 and the 26th 35; `never:` is line 8 and its 26th item line 34.

Stub: add the types to `limits.rs` and `check` returning `Vec::new()`.
Run: `cargo test -p steplet-core --test limits` — Expected: FAIL in four tests (`at_the_limits_nothing_is_reported` passes on the stub).

- [ ] **Step 2: write `check`.** Add to `limits.rs`:
```rust
use alloc::vec::Vec;
use core::fmt;

use crate::Help;
use crate::header::{Scalar, Steplet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limited {
    Steps,
    Commands,
    Domains,
    ReadFolders,
    WriteFolders,
    Never,
    CommandLength,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LimitBreach {
    pub what: Limited,
    /// For a list: the line of the first item over the limit. For a command: its line.
    pub line: u32,
    pub found: usize,
    pub max: usize,
}

pub fn check(s: &Steplet) -> Vec<LimitBreach> {
    let mut out = Vec::new();
    let mut count = |what, items: &[Scalar], max: usize| {
        if items.len() > max {
            out.push(LimitBreach { what, line: items[max].line, found: items.len(), max });
        }
    };
    count(Limited::Steps, &s.steps, MAX_STEPS);
    count(Limited::Commands, &s.allow.commands, MAX_COMMANDS);
    count(Limited::Domains, &s.allow.domains, MAX_DOMAINS);
    count(Limited::ReadFolders, &s.allow.read, MAX_FOLDERS_PER_LIST);
    count(Limited::WriteFolders, &s.allow.write, MAX_FOLDERS_PER_LIST);
    count(Limited::Never, &s.never, MAX_NEVER);
    for c in &s.allow.commands {
        let n = c.text.chars().count();
        if n > MAX_COMMAND_CHARS {
            out.push(LimitBreach { what: Limited::CommandLength, line: c.line, found: n, max: MAX_COMMAND_CHARS });
        }
    }
    out
}

impl fmt::Display for LimitBreach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, found, max) = (self.line, self.found, self.max);
        let what = match self.what {
            Limited::Steps => "steps",
            Limited::Commands => "commands",
            Limited::Domains => "domains",
            Limited::ReadFolders => "read folders",
            Limited::WriteFolders => "write folders",
            Limited::Never => "never lines",
            Limited::CommandLength => {
                return write!(f, "line {line}: this command is {found} characters long; it must be under {}", max + 1);
            }
        };
        write!(f, "line {line}: {found} {what}; at most {max} are allowed")
    }
}

impl LimitBreach {
    pub fn help(&self) -> Help {
        let allowed = match self.what {
            Limited::CommandLength => "a shorter command; put long scripts in the repository and run them by name",
            _ => "a shorter list; everything must fit on the receiver's screen, so split a bigger task into several files",
        };
        Help { allowed, example: "" }
    }
}
```

- [ ] **Step 3: run.** `cargo test -p steplet-core --test limits` — Expected: `5 passed`.

- [ ] **Step 4: paste under "## Task 7" and commit.**
```powershell
git add crates/steplet-core docs/milestone-1-phase-1-results.md
git commit -m "File core: size limits, each naming the first line over the limit"
```

---

### Task 8: the signature check, and the property tests

**Files:**
- Create: `crates/steplet-core/src/sig.rs`
- Modify: `crates/steplet-core/src/lib.rs` (add `pub mod sig;`), `crates/steplet-core/Cargo.toml` (`ssh-key`)
- Test: `crates/steplet-core/tests/common/mod.rs`, `crates/steplet-core/tests/sig.rs`, `crates/steplet-core/tests/properties.rs`

**Interfaces:**
- Consumes: `canon::{Canonical, canonicalize}`, `NAMESPACE`.
- Produces: `sig::verify(c: &Canonical, namespace: &str) -> SigStatus`; `SigStatus::{Unsigned, Valid { fingerprint: String }, Invalid { reason: InvalidReason }}`; `InvalidReason::{NotASignature, WrongVersion, WrongNamespace { found: String }, WrongHash, WrongKeyType, DoesNotVerify}` with `Display`. Fingerprints look like `SHA256:…`. Test helper `common::sign(file: &str, seed: u8) -> String` (signs with a fixed throwaway Ed25519 key made from `[seed; 32]`).

- [ ] **Step 1: add the dependency.** In `crates/steplet-core/Cargo.toml`:
```toml
[dependencies]
ssh-key = { version = "=0.6.7", default-features = false, features = ["alloc", "ed25519"] }
```
(Already in cargo's cache from phase 0; no download.)

- [ ] **Step 2: write the failing tests.** `crates/steplet-core/tests/common/mod.rs`:
```rust
//! Signs test files with a throwaway key made from a fixed seed. Only for tests.

use ssh_key::private::Ed25519Keypair;
use ssh_key::{HashAlg, LineEnding, PrivateKey};
use steplet_core::canon::canonicalize;

pub fn key(seed: u8) -> PrivateKey {
    PrivateKey::from(Ed25519Keypair::from_seed(&[seed; 32]))
}

pub fn pem_for(text: &str, key: &PrivateKey, namespace: &str, hash: HashAlg) -> String {
    key.sign(namespace, hash, text.as_bytes()).unwrap().to_pem(LineEnding::LF).unwrap()
}

/// `file` signed in the Steplet way: the signature covers the cleaned-up text.
pub fn sign(file: &str, seed: u8) -> String {
    let c = canonicalize(file.as_bytes()).unwrap();
    c.with_signature(&pem_for(c.signed(), &key(seed), steplet_core::NAMESPACE, HashAlg::Sha512))
}
```
`crates/steplet-core/tests/sig.rs`:
```rust
mod common;

use ssh_key::HashAlg;
use steplet_core::NAMESPACE;
use steplet_core::canon::canonicalize;
use steplet_core::sig::{InvalidReason, SigStatus, verify};

const FILE: &str = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-10-25\nsteps:\n  - a\n---\nbody\n";

fn status(file: &str) -> SigStatus {
    verify(&canonicalize(file.as_bytes()).unwrap(), NAMESPACE)
}

#[test]
fn unsigned() {
    assert_eq!(status(FILE), SigStatus::Unsigned);
}

#[test]
fn valid_with_the_keys_fingerprint() {
    let fp = common::key(1).public_key().fingerprint(HashAlg::Sha256).to_string();
    assert!(fp.starts_with("SHA256:"));
    assert_eq!(status(&common::sign(FILE, 1)), SigStatus::Valid { fingerprint: fp });
}

#[test]
fn changed_body() {
    let signed = common::sign(FILE, 1).replace("body", "bodY");
    assert_eq!(status(&signed), SigStatus::Invalid { reason: InvalidReason::DoesNotVerify });
}

#[test]
fn wrong_name_space_and_wrong_hash() {
    let c = canonicalize(FILE.as_bytes()).unwrap();
    let other = c.with_signature(&common::pem_for(c.signed(), &common::key(1), "runfile-v1", HashAlg::Sha512));
    assert_eq!(status(&other), SigStatus::Invalid { reason: InvalidReason::WrongNamespace { found: "runfile-v1".into() } });
    let sha256 = c.with_signature(&common::pem_for(c.signed(), &common::key(1), NAMESPACE, HashAlg::Sha256));
    assert_eq!(status(&sha256), SigStatus::Invalid { reason: InvalidReason::WrongHash });
}

#[test]
fn not_a_signature() {
    let c = canonicalize(FILE.as_bytes()).unwrap();
    let junk = c.with_signature("-----BEGIN SSH SIGNATURE-----\nU1NIU0lHAAAAAQ==\n-----END SSH SIGNATURE-----\n");
    assert_eq!(status(&junk), SigStatus::Invalid { reason: InvalidReason::NotASignature });
}

#[test]
fn reasons_in_words() {
    for r in [InvalidReason::NotASignature, InvalidReason::WrongVersion, InvalidReason::WrongNamespace { found: "x".into() },
              InvalidReason::WrongHash, InvalidReason::WrongKeyType, InvalidReason::DoesNotVerify] {
        assert!(!r.to_string().is_empty());
    }
}
```
`crates/steplet-core/tests/properties.rs`:
```rust
//! The three rules the milestone plan asks for: cleaning up twice changes nothing; Windows and Unix line
//! endings sign the same; any edit to a non-whitespace character breaks the signature.

mod common;

use proptest::prelude::*;
use steplet_core::NAMESPACE;
use steplet_core::canon::canonicalize;
use steplet_core::sig::{SigStatus, verify};

fn messy_text() -> impl Strategy<Value = String> {
    let piece = prop::sample::select(vec![
        "---", "\n", "\r\n", " ", "\t", "a", "é", "\u{feff}", "\u{200b}", "signature: |",
        "  -----BEGIN SSH SIGNATURE-----", "  -----END SSH SIGNATURE-----", "  QUJD", "steplet: 1", "\r",
    ]);
    (any::<bool>(), prop::collection::vec(piece, 0..40))
        .prop_map(|(head, v)| format!("{}{}", if head { "---\n" } else { "" }, v.concat()))
}

fn file_text() -> impl Strategy<Value = String> {
    let words = prop::sample::select(vec!["Run", "the", "tests", "café", "#1", "a:b", "x", "\u{200b}", "--", "→"]);
    let line = prop::collection::vec(words, 1..6).prop_map(|w| w.join(" "));
    (line.clone(), prop::collection::vec(line.clone(), 1..5), prop::collection::vec(line, 0..6)).prop_map(
        |(title, steps, body)| {
            let mut f = format!("---\nsteplet: 1\ntitle: \"{title}\"\nsender: Test\nexpires: 2026-12-31\nsteps:\n");
            for s in &steps {
                f.push_str(&format!("  - \"{s}\"\n"));
            }
            f.push_str("---\n");
            for b in &body {
                f.push_str(b);
                f.push('\n');
            }
            f
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn cleaning_up_twice_changes_nothing(raw in messy_text()) {
        if let Ok(c) = canonicalize(raw.as_bytes()) {
            prop_assert_eq!(canonicalize(c.text().as_bytes()), Ok(c));
        }
    }

    #[test]
    fn windows_and_unix_line_endings_sign_the_same(f in file_text(), spaces in 0usize..3) {
        let signed = common::sign(&f, 7);
        let windows = signed.replace('\n', &format!("{}\r\n", " ".repeat(spaces)));
        let unix = canonicalize(signed.as_bytes()).unwrap();
        let win = canonicalize(windows.as_bytes()).unwrap();
        prop_assert_eq!(win.signed(), unix.signed());
        prop_assert!(matches!(verify(&unix, NAMESPACE), SigStatus::Valid { .. }));
        prop_assert_eq!(verify(&win, NAMESPACE), verify(&unix, NAMESPACE));
    }

    #[test]
    fn any_edit_to_non_whitespace_breaks_the_signature(
        f in file_text(),
        pick in any::<prop::sample::Index>(),
        op in 0u8..3,
        ch in prop::sample::select(vec!['x', 'Q', '7', '-', '#', 'é']),
    ) {
        let signed = common::sign(&f, 9);
        let spots: Vec<usize> = signed.char_indices().filter(|(_, c)| !c.is_whitespace()).map(|(i, _)| i).collect();
        let at = spots[pick.index(spots.len())];
        let old = signed[at..].chars().next().unwrap();
        let after = &signed[at + old.len_utf8()..];
        let edited = match op {
            0 if old != ch => format!("{}{ch}{after}", &signed[..at]),
            0 | 1 => format!("{}{after}", &signed[..at]),
            _ => format!("{}{ch}{}", &signed[..at], &signed[at..]),
        };
        if let Ok(c) = canonicalize(edited.as_bytes()) {
            prop_assert!(!matches!(verify(&c, NAMESPACE), SigStatus::Valid { .. }), "still valid after an edit at byte {}:\n{}", at, edited);
        }
    }
}
```
Add `pub mod sig;` and a `sig.rs` with the types and `verify` returning `SigStatus::Unsigned`.
Run: `cargo test -p steplet-core --test sig --test properties`
Expected: `sig`: 4 of 6 fail (`unsigned` and `reasons_in_words` pass on the stub); `properties`: `windows_and_unix_line_endings_sign_the_same` fails, the other two pass (they do not need a real check to hold vacuously: the third only fails if something *is* valid).

- [ ] **Step 3: write `sig.rs`.**
```rust
//! Signature checks (decision rows 13 and 14, P4). Ed25519 keys, sha512 and SSHSIG version 1 only.

use alloc::string::{String, ToString};
use core::fmt;

use ssh_key::{Algorithm, HashAlg, PublicKey, SshSig};

use crate::canon::Canonical;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SigStatus {
    Unsigned,
    Valid { fingerprint: String },
    Invalid { reason: InvalidReason },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidReason {
    NotASignature,
    /// `ssh-key` 0.6.7 already refuses versions above 1 while reading (so this shows as `NotASignature`);
    /// this check stays as a second lock.
    WrongVersion,
    WrongNamespace { found: String },
    WrongHash,
    WrongKeyType,
    DoesNotVerify,
}

pub fn verify(c: &Canonical, namespace: &str) -> SigStatus {
    let Some(block) = c.signature() else { return SigStatus::Unsigned };
    let bad = |reason| SigStatus::Invalid { reason };
    let Ok(sig) = SshSig::from_pem(&block.pem) else { return bad(InvalidReason::NotASignature) };
    if sig.version() != 1 {
        return bad(InvalidReason::WrongVersion);
    }
    if sig.namespace() != namespace {
        return bad(InvalidReason::WrongNamespace { found: sig.namespace().to_string() });
    }
    if sig.hash_alg() != HashAlg::Sha512 {
        return bad(InvalidReason::WrongHash);
    }
    if sig.algorithm() != Algorithm::Ed25519 {
        return bad(InvalidReason::WrongKeyType);
    }
    let key = PublicKey::from(sig.public_key().clone());
    if key.verify(namespace, c.signed().as_bytes(), &sig).is_err() {
        return bad(InvalidReason::DoesNotVerify);
    }
    SigStatus::Valid { fingerprint: key.fingerprint(HashAlg::Sha256).to_string() }
}

impl fmt::Display for InvalidReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidReason::NotASignature => write!(f, "the signature block is not a readable SSH signature"),
            InvalidReason::WrongVersion => write!(f, "the signature uses a format version other than 1"),
            InvalidReason::WrongNamespace { found } => {
                write!(f, "the signature was made for \"{found}\", not for Steplet files (steplet-v1)")
            }
            InvalidReason::WrongHash => write!(f, "the signature uses a hash other than sha512"),
            InvalidReason::WrongKeyType => write!(f, "the signing key is not an Ed25519 key"),
            InvalidReason::DoesNotVerify => {
                write!(f, "the file was changed after it was signed, or the signature belongs to another file")
            }
        }
    }
}
```

- [ ] **Step 4: run, then the no-operating-system build.**
```powershell
cargo test -p steplet-core --test sig --test properties
cargo build -p steplet-core --target wasm32v1-none --release
```
Expected: `6 passed`, `3 passed`; the build finishes. If the third property ever fails only for edits inside the signature's base64 text (where two different texts can decode to the same bytes), that is a finding about the signature text, not the signed content: record it, show it to the user, and do not hide it by narrowing the test without their yes.

- [ ] **Step 5: paste under "## Task 8" and commit.**
```powershell
git add Cargo.lock crates/steplet-core docs/milestone-1-phase-1-results.md
git commit -m "File core: signature check, plus property tests for clean-up and signing"
```

---

### Task 9: test vectors from Windows' own `ssh-keygen`

**Files:**
- Create: `crates/steplet-core/tests/vectors/make_vectors.py`, the files it writes (listed below), `crates/steplet-core/tests/vectors.rs`
- Modify: `.gitattributes` (add `crates/steplet-core/tests/vectors/*.steplet binary`)

**Interfaces:** Consumes `canonicalize`, `verify`, `header::parse`. Produces the committed vectors, which Task 12 also reads.

The vectors are made by `ssh-keygen` and Python only, without our code, so a mistake in our clean-up cannot hide itself.

| File | What it is | Expected |
|---|---|---|
| `unsigned.steplet` | the base file (also "signature block missing") | `Unsigned` |
| `valid.steplet` | Ed25519, `steplet-v1`, sha512 | `Valid`, fingerprint from `fingerprint.txt` |
| `crlf.steplet` | `valid.steplet` with Windows line endings | `Valid`, same fingerprint |
| `tampered.steplet` | one step changed after signing | `Invalid { DoesNotVerify }` |
| `wrong-namespace.steplet` | signed with `-n runfile-v1` | `Invalid { WrongNamespace { found: "runfile-v1" } }` |
| `sha256.steplet` | signed with `-O hashalg=sha256` | `Invalid { WrongHash }` |
| `rsa.steplet` | signed with an RSA 3072 key | `Invalid { WrongKeyType }` |
| `cut.steplet` | the END line removed | `Err(SignatureBroken)` |
| `doubled.steplet` | the block twice | `Err(SignatureTwice)` |
| `moved-in-header.steplet` | the block before `steps:` | `Err(SignatureMisplaced { line: 9 })` |
| `moved-to-body.steplet` | the block at the end of the body | `Err(SignatureMisplaced)` |
| `ed25519.pub`, `rsa.pub`, `fingerprint.txt` | public halves and the Ed25519 fingerprint | |

- [ ] **Step 1: the script.** `crates/steplet-core/tests/vectors/make_vectors.py`:
```python
"""Makes phase 1's signature test files with Windows' own ssh-keygen, without Steplet's code.
Run once: python -I crates/steplet-core/tests/vectors/make_vectors.py
Keys are throwaway and stay in C:\\dev\\m1\\p1 (never committed); only their public halves are copied here."""
import pathlib
import shutil
import subprocess
import sys

KEYGEN = r"C:\Windows\System32\OpenSSH\ssh-keygen.exe"
KEYS = pathlib.Path(r"C:\dev\m1\p1")
OUT = pathlib.Path(__file__).resolve().parent
BASE = """---
steplet: 1
title: Count the lines of the README in Hello-World
sender: Steplet test vectors
expires: 2026-12-31
repository:
  url: https://github.com/octocat/Hello-World.git
  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
steps:
  - Count the lines in the README file.
  - Report the count.
allow:
  commands:
    - wc -l README
---
# Count the README lines

Run `wc -l README` and report the number.
"""


def run(*args, stdin=None):
    r = subprocess.run([KEYGEN, *args], input=stdin, capture_output=True)
    out = (r.stdout + r.stderr).decode(errors="replace").strip()
    print(f"$ ssh-keygen {' '.join(args)}\n  exit {r.returncode}: {out}")
    if r.returncode != 0:
        sys.exit("ssh-keygen failed")
    return out


def key(name, *kind):
    k = KEYS / name
    if not k.exists():
        run("-t", *kind, "-N", "", "-q", "-C", "steplet-p1-vectors", "-f", str(k))
    return k


def sign(text, k, *options):
    msg = KEYS / "msg.txt"
    msg.write_bytes(text.encode("utf-8"))
    sig = KEYS / "msg.txt.sig"
    sig.unlink(missing_ok=True)
    run("-Y", "sign", "-f", str(k), *options, str(msg))
    return sig.read_text(encoding="ascii")


def block(pem):
    lines = [l.strip() for l in pem.splitlines() if l.strip()]
    return "signature: |\n" + "".join("  " + l + "\n" for l in lines)


def embed(text, blk):
    head, _, body = text.partition("\n---\n")
    return head + "\n" + blk + "---\n" + body


KEYS.mkdir(parents=True, exist_ok=True)
ed = key("ed25519", "ed25519")
rsa = key("rsa", "rsa", "-b", "3072")
good = block(sign(BASE, ed, "-n", "steplet-v1", "-O", "hashalg=sha512"))
run("-Y", "check-novalidate", "-n", "steplet-v1", "-s", str(KEYS / "msg.txt.sig"), stdin=BASE.encode("utf-8"))
valid = embed(BASE, good)
files = {
    "unsigned.steplet": BASE,
    "valid.steplet": valid,
    "crlf.steplet": valid.replace("\n", "\r\n"),
    "tampered.steplet": valid.replace("Report the count.", "Report the count and push."),
    "wrong-namespace.steplet": embed(BASE, block(sign(BASE, ed, "-n", "runfile-v1"))),
    "sha256.steplet": embed(BASE, block(sign(BASE, ed, "-n", "steplet-v1", "-O", "hashalg=sha256"))),
    "rsa.steplet": embed(BASE, block(sign(BASE, rsa, "-n", "steplet-v1"))),
    "cut.steplet": valid.replace("  -----END SSH SIGNATURE-----\n", ""),
    "doubled.steplet": embed(BASE, good + good),
    "moved-in-header.steplet": BASE.replace("steps:\n", good + "steps:\n", 1),
    "moved-to-body.steplet": BASE + good,
}
for name, text in files.items():
    (OUT / name).write_bytes(text.encode("utf-8"))
shutil.copy(KEYS / "ed25519.pub", OUT / "ed25519.pub")
shutil.copy(KEYS / "rsa.pub", OUT / "rsa.pub")
fingerprint = run("-l", "-f", str(KEYS / "ed25519.pub")).split()[1]
(OUT / "fingerprint.txt").write_bytes(fingerprint.encode("ascii"))
print("fingerprint:", fingerprint)
```

- [ ] **Step 2: make the vectors and protect their line endings.**
```powershell
python -I crates/steplet-core/tests/vectors/make_vectors.py
```
Expected: every `ssh-keygen` line `exit 0`; the check line says `Good "steplet-v1" signature with ED25519 key SHA256:…`; the same fingerprint printed last. If `-O hashalg=sha256` is refused by `ssh-keygen` 9.5, record its message, drop that vector, and note it under "not verified".

Add to `.gitattributes`:
```
crates/steplet-core/tests/vectors/*.steplet binary
```
(so git never changes `crlf.steplet`'s line endings).

- [ ] **Step 3: write the tests.** `crates/steplet-core/tests/vectors.rs`:
```rust
use steplet_core::NAMESPACE;
use steplet_core::canon::{CanonError, canonicalize};
use steplet_core::header::parse;
use steplet_core::sig::{InvalidReason, SigStatus, verify};

const FINGERPRINT: &str = include_str!("vectors/fingerprint.txt");

fn status(raw: &[u8]) -> Result<SigStatus, CanonError> {
    canonicalize(raw).map(|c| verify(&c, NAMESPACE))
}
fn invalid(reason: InvalidReason) -> Result<SigStatus, CanonError> {
    Ok(SigStatus::Invalid { reason })
}

#[test]
fn ssh_keygen_signatures_verify_with_the_same_fingerprint() {
    let valid = Ok(SigStatus::Valid { fingerprint: FINGERPRINT.trim().into() });
    assert_eq!(status(include_bytes!("vectors/valid.steplet")), valid);
    assert_eq!(status(include_bytes!("vectors/crlf.steplet")), valid);
}

#[test]
fn windows_line_endings_clean_up_to_the_same_signed_text() {
    let a = canonicalize(include_bytes!("vectors/valid.steplet")).unwrap();
    let b = canonicalize(include_bytes!("vectors/crlf.steplet")).unwrap();
    assert!(include_bytes!("vectors/crlf.steplet").windows(2).any(|w| w == b"\r\n"));
    assert_eq!(a.signed(), b.signed());
    assert_eq!(a.signed().as_bytes(), include_bytes!("vectors/unsigned.steplet"));
}

#[test]
fn refused_signatures() {
    assert_eq!(status(include_bytes!("vectors/tampered.steplet")), invalid(InvalidReason::DoesNotVerify));
    assert_eq!(
        status(include_bytes!("vectors/wrong-namespace.steplet")),
        invalid(InvalidReason::WrongNamespace { found: "runfile-v1".into() })
    );
    assert_eq!(status(include_bytes!("vectors/sha256.steplet")), invalid(InvalidReason::WrongHash));
    assert_eq!(status(include_bytes!("vectors/rsa.steplet")), invalid(InvalidReason::WrongKeyType));
}

#[test]
fn missing_cut_doubled_and_moved_blocks() {
    assert_eq!(status(include_bytes!("vectors/unsigned.steplet")), Ok(SigStatus::Unsigned));
    assert!(matches!(status(include_bytes!("vectors/cut.steplet")), Err(CanonError::SignatureBroken { .. })));
    assert!(matches!(status(include_bytes!("vectors/doubled.steplet")), Err(CanonError::SignatureTwice { .. })));
    assert_eq!(status(include_bytes!("vectors/moved-in-header.steplet")), Err(CanonError::SignatureMisplaced { line: 9 }));
    assert!(matches!(status(include_bytes!("vectors/moved-to-body.steplet")), Err(CanonError::SignatureMisplaced { .. })));
}

#[test]
fn the_signed_header_reads() {
    let c = canonicalize(include_bytes!("vectors/valid.steplet")).unwrap();
    assert_eq!(parse(&c).unwrap().steps.len(), 2);
}
```

- [ ] **Step 4: run.** `cargo test -p steplet-core --test vectors` — Expected: `5 passed`. (No failing run first: the code under test exists since Task 8; this task adds outside evidence. If one fails, that is a real disagreement with `ssh-keygen`: stop and debug, do not change the vector.)

- [ ] **Step 5: paste the script output and the test output under "## Task 9" and commit.**
```powershell
git add .gitattributes crates/steplet-core/tests docs/milestone-1-phase-1-results.md
git status --short
git commit -m "File core: test files signed by Windows' own ssh-keygen"
```
Check in `git status` that no private key (`ed25519`, `rsa` without `.pub`) is listed.

---

### Task 10: display pieces (invisible characters and links)

**Files:**
- Create: `crates/steplet-core/src/display.rs`
- Modify: `crates/steplet-core/src/lib.rs` (add `pub mod display;`), `crates/steplet-core/Cargo.toml` (dev-dependency `icu_properties`)
- Test: `crates/steplet-core/tests/display.rs`

**Interfaces:**
- Produces: `display::segments(text: &str) -> Vec<Segment>`; `Segment::{Text(String), InvisibleBadge { codepoint: u32 }, Link { shown: Option<Vec<Segment>>, real: Vec<Segment> }}` (inside a `Link`, only `Text` and `InvisibleBadge`); `display::is_invisible(c: char) -> bool`.

Links recognised:
- Markdown `[shown](real)`: `shown` not empty and without `[`, `]` or a line break; `real` not empty and without whitespace, `(` or `)`.
- Bare `http://` or `https://` addresses (any letter case), up to whitespace or one of `` < > " ' ` ``, minus trailing `. , ; : ! ? ) ]`.

- [ ] **Step 1: add `icu_properties = "=2.3.0"` under `[dev-dependencies]`** (in cargo's cache already).

- [ ] **Step 2: write the failing tests.** `crates/steplet-core/tests/display.rs`:
```rust
use icu_properties::props::{DefaultIgnorableCodePoint, GeneralCategory};
use icu_properties::{CodePointMapData, CodePointSetData};
use proptest::prelude::*;
use steplet_core::display::{Segment, is_invisible, segments};

fn t(s: &str) -> Segment {
    Segment::Text(s.into())
}
fn badge(cp: u32) -> Segment {
    Segment::InvisibleBadge { codepoint: cp }
}

/// Joins the pieces back into the text they came from.
fn source(segs: &[Segment]) -> String {
    segs.iter()
        .map(|s| match s {
            Segment::Text(x) => x.clone(),
            Segment::InvisibleBadge { codepoint } => char::from_u32(*codepoint).unwrap().to_string(),
            Segment::Link { shown: Some(shown), real } => format!("[{}]({})", source(shown), source(real)),
            Segment::Link { shown: None, real } => source(real),
        })
        .collect()
}

#[test]
fn invisible_characters_become_badges() {
    assert_eq!(segments("a\u{200b}b"), [t("a"), badge(0x200b), t("b")]);
    assert_eq!(segments("\u{202e}abc"), [badge(0x202e), t("abc")]);
    assert_eq!(segments("a\u{a0}b\u{feff}"), [t("a"), badge(0xa0), t("b"), badge(0xfeff)]);
    assert_eq!(segments("a\tb\nc d"), [t("a\tb\nc d")]);
    assert_eq!(segments(""), []);
}

#[test]
fn markdown_links_show_their_real_address() {
    assert_eq!(
        segments("see [docs](https://evil.example/x) now"),
        [t("see "), Segment::Link { shown: Some(vec![t("docs")]), real: vec![t("https://evil.example/x")] }, t(" now")]
    );
    assert_eq!(
        segments("[a](https://x\u{200b}.org)"),
        [Segment::Link { shown: Some(vec![t("a")]), real: vec![t("https://x"), badge(0x200b), t(".org")] }]
    );
    assert_eq!(segments("[a](javascript:alert(1))"), [t("[a](javascript:alert(1))")]);
    for not_a_link in ["[a] (b)", "[](x)", "[a]()", "[a](b c)", "[a\nb](c)"] {
        assert_eq!(source(&segments(not_a_link)), not_a_link);
        assert!(!segments(not_a_link).iter().any(|s| matches!(s, Segment::Link { shown: Some(_), .. })), "{not_a_link:?}");
    }
}

#[test]
fn bare_addresses() {
    assert_eq!(
        segments("go to https://a.org/x."),
        [t("go to "), Segment::Link { shown: None, real: vec![t("https://a.org/x")] }, t(".")]
    );
    assert_eq!(segments("HTTPS://A.ORG"), [Segment::Link { shown: None, real: vec![t("HTTPS://A.ORG")] }]);
    assert_eq!(segments("(http://a.org)"), [t("("), Segment::Link { shown: None, real: vec![t("http://a.org")] }, t(")")]);
    assert_eq!(segments("https://"), [t("https://")]);
}

#[test]
fn the_invisible_table_matches_unicode() {
    let di = CodePointSetData::new::<DefaultIgnorableCodePoint>();
    let gc = CodePointMapData::<GeneralCategory>::new();
    let mut wrong = Vec::new();
    for cp in 0..=0x10_ffff_u32 {
        let Some(c) = char::from_u32(cp) else { continue };
        let expected = !matches!(c, '\t' | '\n' | ' ')
            && (di.contains(c)
                || matches!(
                    gc.get(c),
                    GeneralCategory::Control
                        | GeneralCategory::Format
                        | GeneralCategory::SpaceSeparator
                        | GeneralCategory::LineSeparator
                        | GeneralCategory::ParagraphSeparator
                        | GeneralCategory::PrivateUse
                ));
        if is_invisible(c) != expected {
            wrong.push(format!("U+{cp:04X} (should be {expected})"));
        }
    }
    assert!(wrong.is_empty(), "{} code points differ, first: {:?}", wrong.len(), &wrong[..wrong.len().min(40)]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    #[test]
    fn nothing_is_ever_dropped(parts in prop::collection::vec(prop::sample::select(vec![
        "a", " ", "\n", "[", "]", "(", ")", "x", "https://", "http://", "e.org", ".", "\u{200b}", "\u{202e}",
        "é", "<", ">", "\"", "`", "!", "?",
    ]), 0..30)) {
        let text = parts.concat();
        prop_assert_eq!(source(&segments(&text)), text);
    }
}
```
Add `pub mod display;` and a stub `display.rs` with the types, `segments` returning `vec![Segment::Text(text.into())]`, and `is_invisible` returning `false`.
Run: `cargo test -p steplet-core --test display` — Expected: 4 failed, 1 passed (`nothing_is_ever_dropped` passes on the stub, because the stub keeps the text whole).

- [ ] **Step 3: write `display.rs`.**
```rust
//! Display pieces (decision row 28): file text broken into plain text, visible badges for invisible
//! characters, and links that carry their real address. The screens draw only these, never raw file text,
//! and joining the pieces back always gives the exact original text: nothing is dropped.

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    InvisibleBadge { codepoint: u32 },
    /// `shown` is `None` for a bare address. Inside a link there are only `Text` and `InvisibleBadge` pieces.
    Link { shown: Option<Vec<Segment>>, real: Vec<Segment> },
}

pub fn segments(text: &str) -> Vec<Segment> {
    let mut out = Vec::new();
    let (mut plain, mut i) = (0, 0);
    while i < text.len() {
        let rest = &text[i..];
        let found = markdown_link(rest).map(|(len, shown, real)| (len, Some(shown), real)).or_else(|| bare_url(rest).map(|len| (len, None, &rest[..len])));
        if let Some((len, shown, real)) = found {
            out.extend(pieces(&text[plain..i]));
            out.push(Segment::Link { shown: shown.map(pieces), real: pieces(real) });
            i += len;
            plain = i;
        } else {
            i += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    out.extend(pieces(&text[plain..]));
    out
}

fn pieces(s: &str) -> Vec<Segment> {
    let mut v = Vec::new();
    let mut start = 0;
    for (j, c) in s.char_indices() {
        if is_invisible(c) {
            if start < j {
                v.push(Segment::Text(s[start..j].into()));
            }
            v.push(Segment::InvisibleBadge { codepoint: c as u32 });
            start = j + c.len_utf8();
        }
    }
    if start < s.len() {
        v.push(Segment::Text(s[start..].into()));
    }
    v
}

/// `[shown](real)` at the start of `s`: (length, shown, real).
fn markdown_link(s: &str) -> Option<(usize, &str, &str)> {
    let inner = s.strip_prefix('[')?;
    let close = inner.find(|c: char| c == ']' || c == '[' || c == '\n')?;
    if !inner[close..].starts_with(']') || close == 0 {
        return None;
    }
    let after = inner[close + 1..].strip_prefix('(')?;
    let end = after.find(|c: char| c == ')' || c == '(' || c.is_whitespace())?;
    if !after[end..].starts_with(')') || end == 0 {
        return None;
    }
    Some((close + end + 4, &inner[..close], &after[..end]))
}

/// A bare `http://` or `https://` address at the start of `s`: its length.
fn bare_url(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let scheme = if b.len() >= 8 && b[..8].eq_ignore_ascii_case(b"https://") {
        8
    } else if b.len() >= 7 && b[..7].eq_ignore_ascii_case(b"http://") {
        7
    } else {
        return None;
    };
    let mut end = s.find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\'' | '`')).unwrap_or(s.len());
    while end > scheme && s[..end].ends_with(['.', ',', ';', ':', '!', '?', ')', ']']) {
        end -= 1;
    }
    (end > scheme).then_some(end)
}

/// True for characters that show nothing, or look like a space, and so get a badge: Unicode's
/// Default_Ignorable_Code_Point, and the general categories Cc, Cf, Zs, Zl, Zp and Co, except tab, newline
/// and the ordinary space. Checked against icu_properties for every code point in tests/display.rs.
pub fn is_invisible(c: char) -> bool {
    let cp = c as u32;
    if matches!(cp, 0x09 | 0x0a | 0x20) {
        return false;
    }
    INVISIBLE
        .binary_search_by(|&(lo, hi)| {
            if hi < cp {
                core::cmp::Ordering::Less
            } else if lo > cp {
                core::cmp::Ordering::Greater
            } else {
                core::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// Sorted, non-overlapping ranges, both ends included.
const INVISIBLE: &[(u32, u32)] = &[
    (0x0000, 0x001f), (0x007f, 0x00a0), (0x00ad, 0x00ad), (0x034f, 0x034f), (0x0600, 0x0605),
    (0x061c, 0x061c), (0x06dd, 0x06dd), (0x070f, 0x070f), (0x0890, 0x0891), (0x08e2, 0x08e2),
    (0x115f, 0x1160), (0x1680, 0x1680), (0x17b4, 0x17b5), (0x180b, 0x180f), (0x2000, 0x200f),
    (0x2028, 0x202f), (0x205f, 0x206f), (0x3000, 0x3000), (0x3164, 0x3164), (0xe000, 0xf8ff),
    (0xfe00, 0xfe0f), (0xfeff, 0xfeff), (0xffa0, 0xffa0), (0xfff0, 0xfffb), (0x110bd, 0x110bd),
    (0x110cd, 0x110cd), (0x13430, 0x1343f), (0x1bca0, 0x1bca3), (0x1d173, 0x1d17a), (0xe0000, 0xe0fff),
    (0xf0000, 0xffffd), (0x100000, 0x10fffd),
];
```
Note on `markdown_link`'s length: `[` + shown + `](` + real + `)` = `close + end + 4` bytes.
Note on `[a](javascript:alert(1))`: `real` may not contain `(`, so this is not read as a link; its text stays visible as written, which is the safe result.

- [ ] **Step 4: run, and fix the table from the test's list if it differs.** `cargo test -p steplet-core --test display`
Expected: `5 passed`. The table above was written from Unicode 16's data; if `icu_properties`' data is from a newer Unicode, `the_invisible_table_matches_unicode` prints the code points that differ. Then I change the table (never the test) until it passes, and record the differences in the results.

- [ ] **Step 5: paste under "## Task 10" and commit.**
```powershell
git add Cargo.lock crates/steplet-core docs/milestone-1-phase-1-results.md
git commit -m "File core: display pieces with badges for invisible characters and real link addresses"
```

---

### Task 11: `steplet-sign check`

**Files:**
- Create: `crates/steplet-sign/Cargo.toml`, `crates/steplet-sign/src/lib.rs`, `crates/steplet-sign/src/main.rs`, `crates/steplet-sign/src/check.rs`
- Modify: `Cargo.toml` (add `"crates/steplet-sign"` to `members`)
- Test: `crates/steplet-sign/tests/check.rs`

**Interfaces:**
- Consumes: everything in `steplet-core`.
- Produces: `steplet_sign::today() -> Date` (the UTC date now); `steplet_sign::check::check(name: &str, raw: &[u8], today: Date) -> Report`; `Report { text: String, ok: bool }`; `steplet_sign::check::content(c: &Canonical, today: Date) -> (Option<Steplet>, Vec<String>)` (header, limit and expiry problems, each already formatted; used by `sign` in Task 12); `steplet_sign::check::problem(message: &str, allowed: &str, example: &str) -> String`.

Report format (two-space indents, exactly):
```
<name>: ok                      or   <name>: N problem(s)
  <summary line>
  repository: <url> at <commit>   or   repository: none
  expires: <date> (in N days)     or   (today) / (N days ago)
  signature: not signed           or   signature: valid, key SHA256:…
  problem: <message>
    allowed: <allowed>
    for example:
      <example lines>
```
Summary line: `3 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 2 never lines`.

- [ ] **Step 1: the crate files.** `crates/steplet-sign/Cargo.toml`:
```toml
[package]
name = "steplet-sign"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
steplet-core = { path = "../steplet-core" }

[dev-dependencies]
ssh-key = { version = "=0.6.7", default-features = false, features = ["alloc", "ed25519"] }
```
`crates/steplet-sign/src/lib.rs`:
```rust
//! The sender's tool: check a .steplet file, and sign it with the sender's own ssh-keygen.

pub mod check;

use std::time::{SystemTime, UNIX_EPOCH};

use steplet_core::date::Date;

/// Today's date in UTC, from this computer's clock.
pub fn today() -> Date {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
    Date::from_unix(secs).expect("this computer's clock is between the years 2000 and 9999")
}
```
In the root `Cargo.toml`: `members = ["crates/steplet-core", "crates/steplet-sign"]`.

- [ ] **Step 2: write the failing tests.** `crates/steplet-sign/tests/check.rs`:
```rust
use steplet_core::date::Date;
use steplet_sign::check::check;

fn today() -> Date {
    Date::parse("2026-10-11").unwrap()
}

const GOOD: &str = include_str!("../../../examples/run-tests.steplet");

#[test]
fn a_good_unsigned_file() {
    let r = check("run-tests.steplet", GOOD.as_bytes(), today());
    assert!(r.ok, "{}", r.text);
    assert_eq!(
        r.text,
        "run-tests.steplet: ok
  3 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 2 never lines
  repository: https://github.com/octocat/Hello-World.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
  expires: 2026-10-25 (in 14 days)
  signature: not signed
"
    );
}

#[test]
fn a_tab_names_its_line_and_shows_an_example() {
    let r = check("t.steplet", GOOD.replacen("  - Report", "\t- Report", 1).as_bytes(), today());
    assert!(!r.ok);
    assert_eq!(
        r.text,
        "t.steplet: 1 problem
  signature: not signed
  problem: line 12: a tab character
    allowed: indent with spaces, two per level; no tab characters anywhere in the header
    for example:
      allow:
        commands:
          - cargo test
"
    );
}

#[test]
fn expiry_in_the_past_or_too_far_ahead() {
    let r = check("t", GOOD.as_bytes(), Date::parse("2026-10-26").unwrap());
    assert!(!r.ok && r.text.contains("problem: line 5: this file expired on 2026-10-25"), "{}", r.text);
    assert!(r.text.contains("expires: 2026-11-09"), "the example shows today + 14 days:\n{}", r.text);
    let r = check("t", GOOD.as_bytes(), Date::parse("2025-10-24").unwrap());
    assert!(!r.ok && r.text.contains("more than 365 days"), "{}", r.text);
    assert!(check("t", GOOD.as_bytes(), Date::parse("2026-10-25").unwrap()).text.contains("(today)"));
}

#[test]
fn a_changed_signed_file_is_a_problem() {
    let signed = include_str!("../../steplet-core/tests/vectors/valid.steplet");
    let r = check("v", signed.as_bytes(), today());
    assert!(r.ok && r.text.contains("signature: valid, key SHA256:"), "{}", r.text);
    let r = check("v", signed.replace("Report the count.", "Report it.").as_bytes(), today());
    assert!(!r.ok && r.text.contains("problem: signature: the file was changed after it was signed"), "{}", r.text);
}

#[test]
fn limits_and_clean_up_errors() {
    let many: String = (0..31).map(|i| format!("  - step {i}\n")).collect();
    let f = GOOD.replacen("  - Check that the README file is there.\n  - Count the lines in the README file.\n  - Report the count.\n", &many, 1);
    let r = check("t", f.as_bytes(), today());
    assert!(!r.ok && r.text.contains("problem: line 40: 31 steps; at most 30 are allowed"), "{}", r.text);
    let r = check("t", b"title: x\n", today());
    assert!(!r.ok && r.text.contains("problem: line 1: the file must start with a line holding only `---`"), "{}", r.text);
}
```
Line numbers: in `examples/run-tests.steplet`, `Report the count.` is line 12 and `expires` line 5; with 31 steps starting on line 10, step 31 is on line 40.

Stub `crates/steplet-sign/src/check.rs`:
```rust
use steplet_core::date::Date;
pub struct Report { pub text: String, pub ok: bool }
pub fn check(_name: &str, _raw: &[u8], _today: Date) -> Report { Report { text: String::new(), ok: false } }
```
and `pub mod check;` in `lib.rs`. A `main.rs` with `fn main() {}`.
Run: `cargo test -p steplet-sign --test check` — Expected: FAIL in all five.

- [ ] **Step 3: write `check.rs`.**
```rust
//! `steplet-sign check`: everything the file core finds, in words, with line numbers and examples.

use steplet_core::NAMESPACE;
use steplet_core::canon::{Canonical, canonicalize};
use steplet_core::date::Date;
use steplet_core::header::{self, Steplet};
use steplet_core::limits::{self, DEFAULT_EXPIRY_DAYS, MAX_EXPIRY_DAYS};
use steplet_core::sig::{self, SigStatus};

pub struct Report {
    pub text: String,
    pub ok: bool,
}

pub fn check(name: &str, raw: &[u8], today: Date) -> Report {
    let mut info = Vec::new();
    let mut problems = Vec::new();
    match canonicalize(raw) {
        Err(e) => problems.push(problem(&e.to_string(), e.help().allowed, e.help().example)),
        Ok(c) => {
            let (steplet, found) = content(&c, today);
            if let Some(s) = steplet {
                info.push(summary(&s));
                info.push(match &s.repository {
                    Some(r) => format!("repository: {} at {}", r.url.text, r.commit.text),
                    None => "repository: none".into(),
                });
                info.push(format!("expires: {} ({})", s.expires, days_text(s.expires, today)));
            }
            problems.extend(found);
            match sig::verify(&c, NAMESPACE) {
                SigStatus::Unsigned => info.push("signature: not signed".into()),
                SigStatus::Valid { fingerprint } => info.push(format!("signature: valid, key {fingerprint}")),
                SigStatus::Invalid { reason } => problems.push(problem(
                    &format!("signature: {reason}"),
                    "a signature made by steplet-sign over the file exactly as it is now",
                    "steplet-sign sign my-task.steplet --key C:\\Users\\you\\.ssh\\id_ed25519",
                )),
            }
        }
    }
    let ok = problems.is_empty();
    let mut text = if ok { format!("{name}: ok\n") } else { format!("{name}: {}\n", count(problems.len(), "problem")) };
    for line in info {
        text.push_str("  ");
        text.push_str(&line);
        text.push('\n');
    }
    for p in problems {
        text.push_str(&p);
    }
    Report { text, ok }
}

/// Header, limit and expiry problems of a cleaned-up file: everything except the signature.
pub fn content(c: &Canonical, today: Date) -> (Option<Steplet>, Vec<String>) {
    let s = match header::parse(c) {
        Ok(s) => s,
        Err(e) => return (None, vec![problem(&e.to_string(), e.help().allowed, e.help().example)]),
    };
    let mut out: Vec<String> =
        limits::check(&s).iter().map(|b| problem(&b.to_string(), b.help().allowed, b.help().example)).collect();
    let days = s.expires.days_since_epoch() - today.days_since_epoch();
    let suggested = today.add_days(DEFAULT_EXPIRY_DAYS).map(|d| format!("expires: {d}")).unwrap_or_default();
    if days < 0 {
        out.push(problem(
            &format!("line {}: this file expired on {}", s.expires_line, s.expires),
            "a date from today to 365 days ahead; 14 days is usual",
            &suggested,
        ));
    } else if days > MAX_EXPIRY_DAYS {
        out.push(problem(
            &format!("line {}: expires {}, more than 365 days from today ({today})", s.expires_line, s.expires),
            "a date from today to 365 days ahead; 14 days is usual",
            &suggested,
        ));
    }
    (Some(s), out)
}

pub fn problem(message: &str, allowed: &str, example: &str) -> String {
    let mut p = format!("  problem: {message}\n    allowed: {allowed}\n");
    if !example.is_empty() {
        p.push_str("    for example:\n");
        for line in example.lines() {
            p.push_str("      ");
            p.push_str(line);
            p.push('\n');
        }
    }
    p
}

fn summary(s: &Steplet) -> String {
    let a = &s.allow;
    format!(
        "{}; allowed: {}, {}, {}, {}; {}",
        count(s.steps.len(), "step"),
        count(a.commands.len(), "command"),
        count(a.domains.len(), "domain"),
        count(a.read.len(), "read folder"),
        count(a.write.len(), "write folder"),
        count(s.never.len(), "never line"),
    )
}

fn count(n: usize, word: &str) -> String {
    if n == 1 { format!("1 {word}") } else { format!("{n} {word}s") }
}

fn days_text(expires: Date, today: Date) -> String {
    match expires.days_since_epoch() - today.days_since_epoch() {
        0 => "today".into(),
        1 => "in 1 day".into(),
        n if n > 1 => format!("in {n} days"),
        -1 => "1 day ago".into(),
        n => format!("{} days ago", -n),
    }
}
```
And `main.rs`:
```rust
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage:
  steplet-sign check <file>
  steplet-sign sign <file> --key <private key file, or its .pub when the key is in ssh-agent> [--ssh-keygen <path>]
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") if args.len() == 2 => check(PathBuf::from(&args[1])),
        _ => usage(),
    }
}

fn check(path: PathBuf) -> ExitCode {
    match std::fs::read(&path) {
        Err(e) => {
            eprintln!("cannot read {}: {e}", path.display());
            ExitCode::from(2)
        }
        Ok(raw) => {
            let r = steplet_sign::check::check(&path.display().to_string(), &raw, steplet_sign::today());
            print!("{}", r.text);
            if r.ok { ExitCode::SUCCESS } else { ExitCode::from(1) }
        }
    }
}

fn usage() -> ExitCode {
    eprint!("{USAGE}");
    ExitCode::from(2)
}
```

- [ ] **Step 4: run, then try it by hand.**
```powershell
cargo test -p steplet-sign --test check
cargo run -q -p steplet-sign -- check examples/run-tests.steplet; "exit code: $LASTEXITCODE"
cargo run -q -p steplet-sign -- check crates/steplet-core/tests/vectors/tampered.steplet; "exit code: $LASTEXITCODE"
cargo run -q -p steplet-sign -- check; "exit code: $LASTEXITCODE"
```
Expected: `5 passed`; the example prints `ok` with exit code 0 (until 25 Oct 2026; after that it says expired, exit 1); the tampered vector prints the signature problem with exit code 1; no file prints the usage with exit code 2.

- [ ] **Step 5: paste under "## Task 11" and commit.**
```powershell
git add Cargo.toml Cargo.lock crates/steplet-sign docs/milestone-1-phase-1-results.md
git commit -m "Sender tool: steplet-sign check, with line numbers, what is allowed, and examples"
```

---

### Task 12: `steplet-sign sign`, and `ssh-keygen` agreeing with us (yes: B3, optional)

**Files:**
- Create: `crates/steplet-sign/src/sign.rs`
- Modify: `crates/steplet-sign/src/lib.rs` (add `pub mod sign;`), `crates/steplet-sign/src/main.rs` (the `sign` command)
- Test: `crates/steplet-sign/tests/cli.rs`

**Interfaces:**
- Consumes: `check::{content, problem}` (Task 11), `Canonical::with_signature` (Task 3), `sig::verify`.
- Produces: `steplet_sign::sign::SignRequest<'a> { file: &'a Path, key: &'a Path, ssh_keygen: &'a Path, today: Date }`; `steplet_sign::sign::sign(req: &SignRequest) -> Result<String, String>` (Ok: what was done; Err: the problem, in words); `steplet_sign::sign::DEFAULT_SSH_KEYGEN`.

What `sign` does, in order; it writes nothing unless every step succeeds:
1. Reads and cleans up the file. An existing signature block is dropped from the text to sign (so signing again replaces it).
2. Runs `content` (header, limits, expiry). Any problem: stop, show them.
3. Writes the text to sign into a new folder under `%TEMP%`, runs `ssh-keygen -Y sign -f <key> -n steplet-v1 -O hashalg=sha512 <file>` with this terminal attached, so passphrase prompts work as usual. A non-zero exit: stop.
4. Builds the signed file (`with_signature`), cleans it up again, and checks the signature with our own core. Not valid (for example an RSA key): stop.
5. Writes `<file>.steplet-sign-tmp` next to the file, then renames it over the file. Deletes the temporary folder in every case.

- [ ] **Step 1: write the failing tests.** `crates/steplet-sign/tests/cli.rs`:
```rust
//! Runs the real steplet-sign program with Windows' own ssh-keygen and throwaway keys.
#![cfg(windows)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ssh_key::private::Ed25519Keypair;
use ssh_key::{HashAlg, LineEnding, PrivateKey};
use steplet_core::canon::canonicalize;
use steplet_core::date::Date;

const KEYGEN: &str = r"C:\Windows\System32\OpenSSH\ssh-keygen.exe";

struct Scratch(PathBuf);
impl Scratch {
    fn new(name: &str) -> Scratch {
        let d = std::env::temp_dir().join(format!("steplet-sign-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        Scratch(d)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn file_text(expires: Option<Date>) -> String {
    let expires = expires.unwrap_or_else(|| steplet_sign::today().add_days(14).unwrap());
    format!("---\nsteplet: 1\ntitle: Count the README lines\nsender: Test\nexpires: {expires}\nsteps:\n  - Count the lines.\n---\nCount them.\n")
}

fn keygen(dir: &Path, name: &str, kind: &[&str]) -> PathBuf {
    let k = dir.join(name);
    let ok = Command::new(KEYGEN).arg("-t").args(kind).args(["-N", "", "-q", "-C", "steplet-sign-test", "-f"]).arg(&k).status().unwrap();
    assert!(ok.success());
    k
}

fn run(args: &[&std::ffi::OsStr]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_steplet-sign")).args(args).stdin(Stdio::null()).output().unwrap();
    (o.status.code().unwrap(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn sign(file: &Path, key: &Path) -> (i32, String) {
    run(&["sign".as_ref(), file.as_os_str(), "--key".as_ref(), key.as_os_str()])
}

fn check(file: &Path) -> (i32, String) {
    run(&["check".as_ref(), file.as_os_str()])
}

fn fingerprint(public_key: &Path) -> String {
    let o = Command::new(KEYGEN).arg("-l").arg("-f").arg(public_key).output().unwrap();
    String::from_utf8_lossy(&o.stdout).split_whitespace().nth(1).unwrap().to_string()
}

/// ssh-keygen's own check of `pem` over `payload`, with no key list: exit code 0 means the signature is good.
fn keygen_accepts(dir: &Path, payload: &str, pem: &str) -> bool {
    let (p, s) = (dir.join("payload"), dir.join("payload.sig"));
    fs::write(&p, payload).unwrap();
    fs::write(&s, pem).unwrap();
    Command::new(KEYGEN)
        .args(["-Y", "check-novalidate", "-n", "steplet-v1", "-s"]).arg(&s)
        .stdin(fs::File::open(&p).unwrap())
        .status().unwrap().success()
}

#[test]
fn sign_then_check_shows_ssh_keygens_fingerprint() {
    let d = Scratch::new("sign");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None)).unwrap();
    let (code, out) = sign(&file, &key);
    assert_eq!(code, 0, "{out}");
    let fp = fingerprint(&key.with_extension("pub"));
    assert!(out.contains(&fp), "{out}");
    let (code, out) = check(&file);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains(&format!("signature: valid, key {fp}")), "{out}");
}

#[test]
fn ssh_keygen_accepts_what_we_signed_and_what_we_cut_out() {
    let d = Scratch::new("agree");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None)).unwrap();
    assert_eq!(sign(&file, &key).0, 0);
    let c = canonicalize(&fs::read(&file).unwrap()).unwrap();
    assert!(keygen_accepts(&d.0, c.signed(), &c.signature().unwrap().pem));
    assert!(!keygen_accepts(&d.0, &c.signed().replace("Count them.", "Count them!"), &c.signature().unwrap().pem));
}

#[test]
fn ssh_keygen_accepts_a_signature_made_in_rust() {
    let d = Scratch::new("rust");
    let c = canonicalize(file_text(None).as_bytes()).unwrap();
    let key = PrivateKey::from(Ed25519Keypair::from_seed(&[3; 32]));
    let pem = key.sign("steplet-v1", HashAlg::Sha512, c.signed().as_bytes()).unwrap().to_pem(LineEnding::LF).unwrap();
    assert!(keygen_accepts(&d.0, c.signed(), &pem));
}

#[test]
fn an_rsa_key_is_refused_and_the_file_is_left_alone() {
    let d = Scratch::new("rsa");
    let key = keygen(&d.0, "k", &["rsa", "-b", "3072"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None)).unwrap();
    let (code, out) = sign(&file, &key);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("not an Ed25519 key") && out.contains("ssh-keygen -t ed25519"), "{out}");
    assert_eq!(fs::read_to_string(&file).unwrap(), file_text(None));
}

#[test]
fn expiry_must_be_from_today_to_365_days_ahead() {
    let d = Scratch::new("expiry");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    for days in [-1, 366] {
        let text = file_text(steplet_sign::today().add_days(days));
        fs::write(&file, &text).unwrap();
        let (code, out) = sign(&file, &key);
        assert_eq!(code, 1, "{days}: {out}");
        assert_eq!(fs::read_to_string(&file).unwrap(), text, "{days}");
    }
}

#[test]
fn signing_again_after_an_edit_replaces_the_signature() {
    let d = Scratch::new("again");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None)).unwrap();
    assert_eq!(sign(&file, &key).0, 0);
    let edited = fs::read_to_string(&file).unwrap().replace("Count them.", "Count them, then stop.");
    fs::write(&file, edited).unwrap();
    assert_eq!(check(&file).0, 1);
    let (code, out) = sign(&file, &key);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("replaced the signature that was there before"), "{out}");
    assert_eq!(check(&file).0, 0);
    assert_eq!(fs::read_to_string(&file).unwrap().matches("BEGIN SSH SIGNATURE").count(), 1);
}

#[test]
fn windows_line_endings_are_written_back_as_unix_ones_and_still_verify() {
    let d = Scratch::new("crlf");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    fs::write(&file, file_text(None).replace('\n', "\r\n")).unwrap();
    assert_eq!(sign(&file, &key).0, 0);
    let written = fs::read(&file).unwrap();
    assert!(!written.contains(&b'\r'));
    assert_eq!(check(&file).0, 0);
}

#[test]
fn header_problems_stop_signing() {
    let d = Scratch::new("bad");
    let key = keygen(&d.0, "k", &["ed25519"]);
    let file = d.0.join("task.steplet");
    let text = file_text(None).replace("  - Count", "\t- Count");
    fs::write(&file, &text).unwrap();
    let (code, out) = sign(&file, &key);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("line 7: a tab character"), "{out}");
    assert_eq!(fs::read_to_string(&file).unwrap(), text);
}

#[test]
fn wrong_use_prints_how_to_use_it() {
    let (code, out) = run(&["sign".as_ref(), "x.steplet".as_ref()]);
    assert_eq!(code, 2);
    assert!(out.contains("usage:"), "{out}");
}
```
Stub `crates/steplet-sign/src/sign.rs` with the types and `sign` returning `Err("not written yet".into())`, `pub mod sign;` in `lib.rs`, and the `sign` command in `main.rs` (Step 2's version).
Run: `cargo test -p steplet-sign --test cli`
Expected: 3 passed, 6 failed. The three that pass on the stub are `ssh_keygen_accepts_a_signature_made_in_rust` (no signing tool involved), `expiry_must_be_from_today_to_365_days_ahead` (the stub refuses everything, which is right there) and `wrong_use_prints_how_to_use_it`.

- [ ] **Step 2: write `sign.rs` and the command.**
```rust
//! `steplet-sign sign`: signs with the sender's own ssh-keygen, then re-checks with Steplet's own core
//! before anything is written. The file is replaced only when every step succeeded.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use steplet_core::NAMESPACE;
use steplet_core::canon::canonicalize;
use steplet_core::date::Date;
use steplet_core::sig::{self, SigStatus};

use crate::check::{content, problem};

/// Windows' own ssh-keygen. Called by full path: Git Bash's PATH finds a different one first.
pub const DEFAULT_SSH_KEYGEN: &str = if cfg!(windows) { r"C:\Windows\System32\OpenSSH\ssh-keygen.exe" } else { "ssh-keygen" };

pub struct SignRequest<'a> {
    pub file: &'a Path,
    pub key: &'a Path,
    pub ssh_keygen: &'a Path,
    pub today: Date,
}

pub fn sign(req: &SignRequest) -> Result<String, String> {
    let name = req.file.display();
    let raw = fs::read(req.file).map_err(|e| format!("cannot read {name}: {e}"))?;
    let c = canonicalize(&raw).map_err(|e| format!("{name}: not signed\n{}", problem(&e.to_string(), e.help().allowed, e.help().example)))?;
    let (steplet, problems) = content(&c, req.today);
    let steplet = match steplet {
        Some(s) if problems.is_empty() => s,
        _ => return Err(format!("{name}: not signed\n{}", problems.concat())),
    };
    let replacing = c.signature().is_some();

    let work = TempDir::new().map_err(|e| format!("cannot make a temporary folder: {e}"))?;
    let payload = work.0.join("payload.steplet");
    fs::write(&payload, c.signed()).map_err(|e| format!("cannot write {}: {e}", payload.display()))?;
    let status = Command::new(req.ssh_keygen)
        .args(["-Y", "sign", "-f"])
        .arg(req.key)
        .args(["-n", NAMESPACE, "-O", "hashalg=sha512"])
        .arg(&payload)
        .status()
        .map_err(|e| format!("cannot start {}: {e}", req.ssh_keygen.display()))?;
    if !status.success() {
        return Err(format!(
            "{name}: not signed; ssh-keygen stopped with exit code {} (its message is above)",
            status.code().map_or("none".into(), |c| c.to_string())
        ));
    }
    let pem = fs::read_to_string(work.0.join("payload.steplet.sig")).map_err(|e| format!("cannot read ssh-keygen's signature: {e}"))?;

    let signed = c.with_signature(&pem);
    let again = canonicalize(signed.as_bytes()).map_err(|e| format!("{name}: not signed; internal error: {e}"))?;
    let fingerprint = match sig::verify(&again, NAMESPACE) {
        SigStatus::Valid { fingerprint } => fingerprint,
        SigStatus::Invalid { reason } => {
            return Err(format!("{name}: not signed; {reason}. Steplet needs an Ed25519 key; make one with: ssh-keygen -t ed25519"));
        }
        SigStatus::Unsigned => return Err(format!("{name}: not signed; internal error: the signature block went missing")),
    };

    let tmp = req.file.with_extension("steplet-sign-tmp");
    fs::write(&tmp, &signed).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, req.file).map_err(|e| format!("cannot replace {name}: {e}"))?;
    let mut msg = format!("signed {name}\n  key {fingerprint}\n  expires {}", steplet.expires);
    if replacing {
        msg.push_str("\n  replaced the signature that was there before");
    }
    Ok(msg)
}

/// A new, empty folder under %TEMP%, deleted when dropped.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> std::io::Result<TempDir> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        let path = std::env::temp_dir().join(format!("steplet-sign-{}-{nanos}", std::process::id()));
        fs::create_dir(&path)?;
        Ok(TempDir(path))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
```
In `main.rs`, the `match` becomes:
```rust
        Some("check") if args.len() == 2 => check(PathBuf::from(&args[1])),
        Some("sign") => match sign_args(&args[1..]) {
            Some((file, key, keygen)) => {
                let req = steplet_sign::sign::SignRequest { file: &file, key: &key, ssh_keygen: &keygen, today: steplet_sign::today() };
                match steplet_sign::sign::sign(&req) {
                    Ok(msg) => {
                        println!("{msg}");
                        ExitCode::SUCCESS
                    }
                    Err(msg) => {
                        println!("{msg}");
                        ExitCode::from(1)
                    }
                }
            }
            None => usage(),
        },
        _ => usage(),
```
plus:
```rust
fn sign_args(rest: &[String]) -> Option<(PathBuf, PathBuf, PathBuf)> {
    let (mut file, mut key, mut keygen) = (None, None, None);
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--key" => key = Some(PathBuf::from(it.next()?)),
            "--ssh-keygen" => keygen = Some(PathBuf::from(it.next()?)),
            _ if file.is_none() && !a.starts_with("--") => file = Some(PathBuf::from(a)),
            _ => return None,
        }
    }
    Some((file?, key?, keygen.unwrap_or_else(|| PathBuf::from(steplet_sign::sign::DEFAULT_SSH_KEYGEN))))
}
```

- [ ] **Step 3: run every test in the workspace.** `cargo test --workspace`
Expected: everything passes; `cli` shows `9 passed`.

- [ ] **Step 4 (optional, yes B3): a passphrase-protected key, by you.** In your own terminal (I cannot type a passphrase), from the repository root:
```powershell
cargo build -p steplet-sign
& C:\Windows\System32\OpenSSH\ssh-keygen.exe -t ed25519 -C steplet-passphrase-test -f C:\dev\m1\p1\pass
Copy-Item examples\run-tests.steplet C:\dev\m1\p1\pass-test.steplet
.\target\debug\steplet-sign.exe sign C:\dev\m1\p1\pass-test.steplet --key C:\dev\m1\p1\pass
.\target\debug\steplet-sign.exe check C:\dev\m1\p1\pass-test.steplet
```
Expected: `ssh-keygen` asks for the passphrase while signing; `signed …`; then `ok` with `signature: valid`. Paste your terminal's output to me, or tell me which Terminal tab it is in. Skipped, it goes under "not verified".

- [ ] **Step 5: paste under "## Task 12" and commit.**
```powershell
git add Cargo.lock crates/steplet-sign docs/milestone-1-phase-1-results.md
git commit -m "Sender tool: steplet-sign sign, re-checked by our core, and ssh-keygen agreeing with us"
```

---

### Task 13: the fuzz target and a recorded run (yes: B2 if a download is needed)

**Files:**
- Create: `crates/steplet-core/fuzz/Cargo.toml`, `crates/steplet-core/fuzz/fuzz_targets/read_file.rs`
- Modify: `.gitignore` (add `crates/steplet-core/fuzz/corpus/`, `crates/steplet-core/fuzz/artifacts/`, `crates/steplet-core/fuzz/coverage/`)

**Interfaces:** Consumes every core function. Produces nothing new.

- [ ] **Step 1: check whether `libfuzzer-sys` 0.4.13 is already in cargo's cache.**
```powershell
Get-ChildItem "$env:USERPROFILE\.cargo\registry\src\*\libfuzzer-sys-0.4.13" -Directory -ErrorAction SilentlyContinue | Select-Object FullName
```
If nothing prints, ask the user for yes B2 before Step 3.

- [ ] **Step 2: the target.** `crates/steplet-core/fuzz/Cargo.toml`:
```toml
[package]
name = "steplet-core-fuzz"
version = "0.0.0"
publish = false
edition = "2024"

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "=0.4.13"
steplet-core = { path = ".." }

[[bin]]
name = "read_file"
path = "fuzz_targets/read_file.rs"
test = false
doc = false
bench = false

[workspace]
members = ["."]
```
`crates/steplet-core/fuzz/fuzz_targets/read_file.rs`:
```rust
//! Any bytes: no crash anywhere in the file core, cleaning up twice changes nothing,
//! and display pieces always join back into the text they came from.
#![no_main]

use libfuzzer_sys::fuzz_target;
use steplet_core::canon::canonicalize;
use steplet_core::display::{Segment, segments};
use steplet_core::{NAMESPACE, header, limits, sig};

fn source(segs: &[Segment]) -> String {
    segs.iter()
        .map(|s| match s {
            Segment::Text(x) => x.clone(),
            Segment::InvisibleBadge { codepoint } => char::from_u32(*codepoint).unwrap().to_string(),
            Segment::Link { shown: Some(shown), real } => format!("[{}]({})", source(shown), source(real)),
            Segment::Link { shown: None, real } => source(real),
        })
        .collect()
}

fuzz_target!(|raw: &[u8]| {
    let Ok(c) = canonicalize(raw) else { return };
    assert_eq!(canonicalize(c.text().as_bytes()).as_ref(), Ok(&c));
    let _ = sig::verify(&c, NAMESPACE);
    assert_eq!(source(&segments(c.body())), c.body());
    if let Ok(s) = header::parse(&c) {
        let _ = limits::check(&s);
        for item in s.steps.iter().chain(&s.never).chain(&s.allow.commands) {
            assert_eq!(source(&segments(&item.text)), item.text);
        }
    }
});
```
Add to `.gitignore`:
```
# Fuzzing output (a crash input is copied into a test if one is found)
crates/steplet-core/fuzz/corpus/
crates/steplet-core/fuzz/artifacts/
crates/steplet-core/fuzz/coverage/
```

- [ ] **Step 3: seed and run for 15 minutes.**
```powershell
$asan = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64"
New-Item -ItemType Directory -Force crates\steplet-core\fuzz\corpus\read_file | Out-Null
Copy-Item crates\steplet-core\tests\vectors\*.steplet, examples\*.steplet crates\steplet-core\fuzz\corpus\read_file\
cd crates\steplet-core
$env:PATH = "$asan;$env:PATH"
cmd /c "cargo +nightly fuzz run read_file fuzz/corpus/read_file -- -max_total_time=900 -max_len=65536 > C:\dev\m1\p1\fuzz.txt 2>&1"
"exit code: $LASTEXITCODE"
cd ..\..
Get-Content C:\dev\m1\p1\fuzz.txt -Tail 25
```
Expected: the last lines show `Done N runs in 901 second(s)` and exit code 0. If it crashes: the input is in `fuzz/artifacts/read_file/`; I turn it into a normal test in the right test file, fix the code, and run again for the full 15 minutes. Each crash and fix is recorded.

- [ ] **Step 4: paste the last lines (runs, time, corpus size) under "## Task 13" and commit.**
```powershell
git add .gitignore crates/steplet-core/fuzz docs/milestone-1-phase-1-results.md
git status --short
git commit -m "File core: fuzz target, run for 15 minutes without a crash"
```
(`crates/steplet-core/fuzz/Cargo.lock` is committed too, so the fuzz build is repeatable.)

---

### Task 14: the sender's guide, the full checks, and wrap-up

**Files:**
- Create: `docs/steplet-format.md`
- Modify: `docs/milestone-1-phase-1-results.md` (summary), `docs/handoff.md` (status line), `docs/decisions.md` (F1 to F3 as decided rows, once approved), `docs/plans/2026-10-08-milestone-1-plan.md` (status line for phase 1), `C:\Users\RANVIR\.claude\projects\C--dev-runFile\memory\project-steplet-state.md`

- [ ] **Step 1: write `docs/steplet-format.md`.** Contents, in this order:
  1. What a `.steplet` file is, in three sentences.
  2. "Start here": copy `examples/run-tests.steplet`, change it, run `steplet-sign check`, then `steplet-sign sign … --key …`.
  3. The complete example (the F1 block from section 4 of this plan, without the `#` remarks).
  4. A table with one row per field: name, required or not, what is allowed, limit, example. (Values from Task 5's rules and row 12.)
  5. "Writing values": plain text; when to use double quotes (starts with a symbol, contains `: ` or ` #`, ends with `:`); the only two escapes; no tabs; two spaces per level; one line per value; comments only on their own line.
  6. "Signing": Ed25519 keys only (`ssh-keygen -t ed25519`); what `sign` changes in the file (adds the block; Windows line endings become Unix ones); editing after signing breaks the signature; spaces and empty lines at line ends are not covered by the signature by design (R13); sign again to replace.
  7. "What the messages mean": the exit codes 0, 1, 2, and two sample outputs (one ok, one problem) copied from Task 11's real output.

- [ ] **Step 2: run every check and paste the output.**
```powershell
cargo test --workspace 2>$null | Select-String -Pattern "^running|^test result|FAILED|panicked"
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p steplet-core --target wasm32v1-none --release
cargo build -p steplet-core --target wasm32-unknown-unknown --release
cargo tree -p steplet-core -e normal --prefix none | Sort-Object -Unique
rustc --version; cargo --version
```
Expected: every `test result: ok`; clippy prints no warnings; both WASM builds finish; the dependency list contains no crate that needs the operating system (it is the same tree S9 built for `wasm32v1-none`). Clippy warnings are fixed in the code, then the tests run again.

- [ ] **Step 3: write the summary** at the end of the results file: a table with one row per "done means" item from the milestone plan (section 6, phase 1), each with PASS or FAIL and where its output is:

| Done means (milestone plan) | Evidence |
|---|---|
| Windows `ssh-keygen` vectors verify in our core: valid, tampered, wrong name space, sha256, RSA, block missing / doubled / moved, Windows line endings | Task 9 |
| Ours verify in `ssh-keygen` | Task 12: `ssh_keygen_accepts_what_we_signed_and_what_we_cut_out`, `ssh_keygen_accepts_a_signature_made_in_rust` |
| Clean-up twice gives the same result; Windows and Unix line endings sign the same; any non-whitespace edit breaks the signature | Task 8, `properties.rs` |
| The differential test against `saphyr-parser` agrees on every file we accept | Task 6 |
| The parser fuzz target runs for a recorded time without a crash | Task 13 |

Then "What phase 1 did not verify" (Step 5's list, updated with anything found on the way).

- [ ] **Step 4: update the status lines and memory.** `docs/handoff.md` and the milestone plan get one dated status line each ("phase 1 finished on <date>; results in `docs/milestone-1-phase-1-results.md`"). `docs/decisions.md` already has the section "Milestone 1, phase 1 choices (11 Oct 2026)"; add a row only for anything decided differently during the phase (for example a change to the invisible-character table or a vector that `ssh-keygen` could not make). The memory file `project-steplet-state.md` gets the new state and next step (phase 2's build plan).

- [ ] **Step 5: commit, then ask the user** whether to merge this branch into `main` (fast-forward, as after phase 0) and whether to remove the two old worktrees.
```powershell
git add docs
git commit -m "Phase 1 wrap-up: sender's guide, full checks, results summary"
git log --oneline main..HEAD
```

---

## 7. What this plan does not verify

- Hardware security keys (`ed25519-sk`) and keys held only in the Windows SSH agent: `ssh-keygen` handles both, but `sk` keys produce a different signature type that our core refuses as "not an Ed25519 key". Whether to accept `sk-ssh-ed25519` keys is a later decision; nothing here tests it. The Windows SSH agent service is off by default and turning it on needs administrator rights, so the agent route is not tested.
- A passphrase-protected key, unless you do Task 12's optional step.
- Which Unicode version `icu_properties` 2.3.0's data uses; the invisible table matches that data, whatever version it is.
- `saphyr-parser` 0.1.0 is three weeks old; the differential test trusts it as the reference. Where it might be wrong itself is not checked.
- The transit test through WhatsApp, Gmail and Outlook (R13): a launch blocker, not part of milestone 1's phase 1.
- How the preview draws the pieces (phase 4), the rating that turns these results into colours (phase 2), and the WASM build's results matching the native ones (phase 2's parity test). Task 14 only shows the core *builds* for WASM.
- A fuzz run is a sample, not a proof: 15 minutes without a crash says nothing about inputs it did not try.
- macOS and Linux: `steplet-sign`'s tests run on Windows only (`#![cfg(windows)]`).
