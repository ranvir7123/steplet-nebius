# Nebius hackathon copy: results

Evidence for each task of `docs/plans/2026-10-29-nebius-hackathon.md`: the commands run and their output, unchanged
except where a long output is cut (said each time). Branch `nebius-hackathon` in `C:\dev\steplet-nebius`, started from
`956e4ab`.

## Task 1: Starting point of the copy (29 Oct 2026)

Run from a script, one log per step, at 16:23 to 16:32 IST. The summary it wrote:

```
01-npm-ci exit=0 3s
02-npm-build exit=0 3s
03-fmt exit=0 1s
04-clippy exit=0 103s
05-test-build-1 exit=0 144s
06-test exit=0 266s
07-npm-check exit=0 6s
08-npm-test exit=1 6s
09-lock-cargo exit=1 2331s
10-lock-npm exit=0 2s
DONE
```

Steps 08, 09 and 10 are explained below; none of them is a fault in the copy.

**npm ci and build:**

```
> npm --prefix web ci --offline
added 53 packages, and audited 54 packages in 2s
found 0 vulnerabilities
> npm --prefix web run build
dist/assets/index-BJFQ5Kn6.css   29.67 kB │ gzip:  4.82 kB
dist/assets/index-CzL14Byi.js   102.25 kB │ gzip: 33.45 kB
✓ built in 490ms
```

**fmt and clippy:** `cargo fmt --all -- --check` printed nothing (exit 0). `cargo clippy --workspace --all-targets
--offline -- -D warnings`, last line:

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 42s
```

**Tests:** `cargo test --workspace --offline --no-run` built on the first try (exit 0, 144 s); then
`cargo test --workspace --offline` (exit 0, 266 s). Its 81 `test result:` lines, added up:

```
81 test programs: 407 passed, 0 failed, 72 ignored
```

The 72 ignored tests are the ones that need Steplet's Linux, the network or a real key; they run only when asked for
by name, as before.

**Screen checks:** `npm --prefix web run check`:

```
1793271713635 COMPLETED 159 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 70 pairs
```

**Screen tests.** `npm --prefix web test` failed twice before any test ran, and neither time was the copy at fault:

1. The script had not built the screens first: `Error: The directory "dist-screens" does not exist.` Fixed by
   running `npm --prefix web run build:screens` (exit 0, 2 s) before the tests.
2. Then every test stopped at the start with `browserType.launch: Executable doesn't exist at
   C:\Users\RANVIR\AppData\Local\ms-playwright\chromium_headless_shell-1243\...`. Playwright 1.63.0 in this copy
   needs its Chromium build 1243. That build was on this computer on 18 Oct (it is listed in
   `docs/plans/2026-10-18-milestone-1-phase-4.md`) but is gone now; only build 1234 is left. Another Playwright
   install on this computer most likely cleared it on 24 Oct, when `ffmpeg-1013` appeared there.

Downloading build 1243 again would need the owner's yes. Instead the screen tests ran in Microsoft Edge 156, which is
already installed and is the engine inside WebView2, the real app's window. A local settings file,
`web/playwright.edge.config.ts`, runs the same tests and the same "screens" project with `channel: "msedge"`; it is
listed in `.git/info/exclude` and never committed. The tests check text, layout and behaviour and save screenshots;
none compares pixels, so the browser's own drawing does not change a result.

```
> npx --offline playwright test --config playwright.edge.config.ts
  1 failed
    [screens] › app\tests\screens.spec.ts:497:7 › report-finished-light-560
  329 passed (35.1m)
```

The one failure was a click that got no answer within 30 seconds (`locator.click: Test timeout of 30000ms
exceeded`), with eight Edge windows running at once. The same test alone:

```
> npx --offline playwright test --config playwright.edge.config.ts -g "report-finished"
  ok 1 [screens] › app\tests\screens.spec.ts:497:7 › report-finished-light-1100 (1.2s)
  ok 2 [screens] › app\tests\screens.spec.ts:497:7 › report-finished-light-560 (3.3s)
  ok 3 [screens] › app\tests\screens.spec.ts:497:7 › report-finished-dark-1100 (1.0s)
  ok 4 [screens] › app\tests\screens.spec.ts:497:7 › report-finished-dark-560 (948ms)
  4 passed (10.4s)
```

So all 330 screen tests pass in Edge. Not run: the "real" project (`real-*.spec.ts`), which drives the app's e2e build
and needs that build running first.

**Lock ages (F12).** Both lock files are unchanged from `956e4ab` (`git diff --stat 956e4ab -- Cargo.lock
web/package-lock.json` printed nothing). The last change to `Cargo.lock`, on 27 Oct (`3033229`), added only a link
between two of Steplet's own crates.

The first run of `lock_ages.py` on `Cargo.lock` stopped after 39 minutes on a network outage:
`urllib.error.URLError: <urlopen error [Errno 11001] getaddrinfo failed>`; name lookups worked again at 17:11. The
step for `web/package-lock.json` ran `lock_ages.py`, which reads only Cargo lock files (`0 crates.io packages in the
lock file`); the npm lock is checked by `web/scripts/npm_ages.py` (decision S10). Both were run again:

```
> python -I lock_ages.py Cargo.lock        (second run, 17:20)
urllib.error.URLError: <urlopen error [WinError 10054] An existing connection was forcibly closed by the remote host>
(94 packages read before that; counted from 29 Oct, 0 of them under 14 days old)
> python -I web\scripts\npm_ages.py web\package-lock.json
http.client.IncompleteRead: IncompleteRead(37592 bytes read, 38349 more expected)
```

Both scripts stop at the first network error, and this network keeps dropping connections. The rule still holds from
the records, because a package only gets older: `docs/milestone-1-phase-5-results.md` (22 Oct) counted all 444
crates.io packages in this `Cargo.lock` from 22 Oct, with 0 under 14 days old and the youngest published on 6 Oct;
`docs/milestone-1-phase-4-results.md` counted all 88 npm packages in this `web/package-lock.json` (unchanged since
21 Oct, `2a0498b`) with 0 under 14 days old. A full check with retries is still owed before the release (Task 12),
and after any change to a lock file.

**Commit:** "Starting point of the Nebius copy: builds, tests and lock ages recorded".
## Task 2: Talking to Nebius (stand-in only, no spend) (29 Oct 2026)

New: `crates/steplet-run/src/model.rs` (the conversation, the `Model` trait, the `Scripted` stand-in model),
`winhttp.rs` (one request through Windows' own WinHTTP, decision N1), `tokenfactory.rs`, `price.rs`; the test
`crates/steplet-run/tests/tokenfactory.rs`, whose stand-in Nebius is a small web server on `127.0.0.1` inside the test
file itself (the plan had it in `tests/support/fake_nebius.rs`; only this file uses it). `Cargo.toml` gains the
`windows-sys` feature `Win32_Networking_WinHttp`. Not as the plan wrote it: the answer type is `ModelReply` (N10),
`TokenFactory::models` is a method on a `TokenFactory` rather than a free function taking the key, and the errors keep
Nebius's own words (cut to 300 characters, with the key taken out) so the report can say what Nebius said.

Before the code was connected:

```
> cargo test -p steplet-run --offline --test tokenfactory --no-run
error[E0432]: unresolved imports `steplet_run::Finish`, `steplet_run::HttpLimits`, `steplet_run::Message`, `steplet_run::Model`, `steplet_run::ModelError`, `steplet_run::Nanodollars`, `steplet_run::ToolSpec`, `steplet_run::TokenFactory`, `steplet_run::Usage`, `steplet_run::cost`, `steplet_run::price`, `steplet_run::worst_case`
error: could not compile `steplet-run` (test "tokenfactory") due to 1 previous error
```

After:

```
> cargo test -p steplet-run --offline --test tokenfactory
test worst_case_is_at_least_the_cost_of_any_reply_within_max_tokens ... ok
test cost_counts_both_directions ... ok
test only_https_or_this_laptop_is_accepted ... ok
test an_answer_that_is_not_json_is_a_bad_reply ... ok
test an_oversized_reply_is_refused ... ok
test the_models_list_is_read ... ok
test reads_tool_calls_text_and_usage ... ok
test sends_the_key_as_a_bearer_header_and_nowhere_else ... ok
test a_refused_key_is_key_refused ... ok
test a_tool_result_goes_back_as_a_tool_message ... ok
test no_credit_is_said_as_such ... ok
test the_key_never_appears_in_an_error ... ok
test busy_and_server_errors_are_told_apart ... ok
test nobody_listening_is_a_network_error ... ok
test a_server_that_goes_silent_is_too_slow ... ok
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.02s
```

They passed on the first run, so the code was broken on purpose once to see a test catch it: the header was changed
to send `Bearer x<key>`, then put back (the file compared equal to the original afterwards).

```
> cargo test -p steplet-run --offline --test tokenfactory -- sends_the_key      (header broken on purpose)
test sends_the_key_as_a_bearer_header_and_nowhere_else ... FAILED
assertion `left == right` failed
  left: Some("Bearer xnebius-STEPLET-TEST-MADE-UP-NOT-A-KEY")
 right: Some("Bearer nebius-STEPLET-TEST-MADE-UP-NOT-A-KEY")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.01s
```

The run crate's own unit tests, the format check, clippy and the lock file:

```
> cargo test -p steplet-run --offline --lib
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.60s
> cargo fmt --all -- --check          (after cargo fmt --all rewrapped two lines in model.rs)
(nothing printed, exit 0)
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 35.30s
> git diff --stat -- Cargo.lock
(nothing printed: the lock file is unchanged, so no package was added and F12 still holds)
```

Not shown by these tests: a call to the real Nebius (Task 4), and WinHTTP through a proxy (this computer has none).

## Task 3: The Nebius key box (29 Oct 2026)

Setup now has two key boxes, Nebius first, then Anthropic, each with its own Credential Manager entry:
`Steplet/nebius-api-key` (new) and `Steplet/anthropic-api-key` (the original's, unchanged so a saved key still
works). `save_key` and `forget_key` take `which` (`KeyFor::Nebius` or `KeyFor::Anthropic`), and the setup screen's
data says which keys are saved (`keys: { nebius, anthropic }`), never the keys themselves.

Not as the plan wrote it: the screen's data holds `keys: Keys` rather than a lone `nebius_key: bool`, so the two boxes
read the same way; `scenarios.ts` did not change, because the new states are saved fixtures (`setup-nebius-key-saved`,
`setup-key-refused`); the size limit is 2,048 bytes rather than 512 characters (N11); and "Setup is ready" still means
ready for Claude Code (N13).

Before the code was connected:

```
> cargo test -p steplet-app --offline --test key --no-run
error[E0432]: unresolved import `steplet_app::key::Entries`
error[E0432]: unresolved import `steplet_app::views::KeyFor`
error[E0425]: cannot find function `check` in module `key`
error[E0425]: cannot find value `ENTRIES` in module `key`
error[E0609]: no field `keys` on type `SetupView`
error: could not compile `steplet-app` (test "key") due to 12 previous errors
```

After, every test in the app crate (the ignored ones need Steplet's Linux and run only on request):

```
> cargo test -p steplet-app --offline
     Running tests\fixtures.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.55s
     Running tests\help.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\key.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests\open.rs
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests\pins.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests\register.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.77s
     Running tests\save.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\session.rs
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s
     Running tests\session_live.rs
test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\setup.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\shell.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests\suite_live.rs
test result: ok. 0 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\views.rs
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

The six key tests are `key_is_stored_read_and_forgotten`, `what_is_not_a_key_is_refused_and_nothing_is_saved`,
`a_pasted_key_is_trimmed` (a 2,048-character key with spaces around it is saved without them),
`an_empty_key_is_refused`, `the_nebius_key_has_its_own_entry` (saving or forgetting one key leaves the other alone)
and `the_setup_view_never_holds_the_key`. They save only made-up keys, under made-up entry names, and remove them.

The web side:

```
> npm --prefix web run check
COMPLETED 161 FILES 0 ERRORS 0 WARNINGS 0 FILES_WITH_PROBLEMS
no raw HTML
every colour pair at least 4.5:1 (focus rings and control borders at least 3:1): 70 pairs
> npx --offline playwright test --config playwright.edge.config.ts setup.spec.ts
  9 passed
> npx --offline playwright test --config playwright.edge.config.ts screens.spec.ts -g "(?<!needs-)setup-|every_state_has_its_proof"
  53 passed (55.5s)
> npx --offline playwright test --config playwright.edge.config.ts      (every web test, after a fresh build:screens)
  341 passed (5.9m)
```

That last run also redraws the original's pictures in Edge, so `docs/screens/phase-4/` was put back with
`git restore` afterwards.

The web tests were written after the web code, so the code was broken on purpose once to see them catch it: every
Save button was made to save the Anthropic key, then the file was put back (it compared equal to the original).

```
> npx --offline playwright test --config playwright.edge.config.ts setup.spec.ts      (Save broken on purpose)
  x  4 [screens] › app\tests\setup.spec.ts:60:1 › the_nebius_key_has_its_own_box_first (5.3s)
  x  6 [screens] › app\tests\setup.spec.ts:101:1 › a_refused_key_says_why_in_its_own_box (5.3s)
  2 failed
  7 passed (19.5s)
restored: True
```

The first `screens.spec.ts` run also caught a real fault: the exact match for "Saved" failed in 8 pictures, because
the saved line's markup carried stray spaces. The line was put on one line and all 53 passed.

Format, lint and lock files:

```
> cargo fmt --all -- --check          (after cargo fmt --all rewrapped five calls in tests/fixtures.rs and tests/views.rs)
fmt exit: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.05s
clippy exit: 0
> git status --short -- Cargo.lock web/package-lock.json
(nothing printed: neither lock file changed)
```

Looked at in dark mode at 1100 and 560 wide (`docs/screens/nebius/`): nothing set up, all ready, only the Nebius key
saved, only the Anthropic key saved, and a refused key. The Nebius box sits between Steplet's Linux and the Anthropic
box, the "Saved." line fits on one line at 560, and the refusal stays inside the Nebius box. Nothing needed fixing.
The original's pictures in `docs/screens/phase-4/` are unchanged.

Not shown yet: the saved line says "Steplet sends it only to api.tokenfactory.nebius.com." Task 2's
`sends_the_key_as_a_bearer_header_and_nowhere_else` shows the key goes only in the request header to the address it
is given; that the key never reaches Steplet's Linux is Task 9's `the_nebius_key_never_reaches_linux`. A real key
typed into the real window is Task 4.

Added after that commit, because no test drove the real window's key commands: `real-nebius-key.spec.ts` drives the
e2e build's window (opened with no file, after checking that no run held `Local\Steplet-run`). It saves a made-up key
through the real `save_key`, sees "Saved.", forgets it through `forget_key`, and checks that the Anthropic box did not
change. It skips itself if a Nebius key is already saved, so it never replaces the owner's key, and forgets the
made-up key if it fails after saving.

```
> npx --offline playwright test --project=real real-nebius-key.spec.ts      (e2e build's window open)
Anthropic key saved before: yes
made-up Nebius key: saved, and the box says so
made-up Nebius key: forgotten; the Anthropic box did not change
  ok 1 [real] › app\tests\real-nebius-key.spec.ts:11:1 › the_nebius_key_box_saves_and_forgets_through_the_real_commands (3.1s)
  1 passed (6.3s)
> target\debug\examples\nebius_check.exe --yes-spend 0.05      (afterwards: the entry is empty again)
There is no Nebius key in Setup. Type it into Setup's Nebius box first.
```

The normal build was then rebuilt for the owner, and its debugging port checked closed (`window running: True;
debugging port open: False`).
## Task 4, first part: the real Nebius server, with no key (30 Oct 2026, no spend)

The rest of Task 4 waits for the Nebius key. This part needs none: Steplet's own client, at its built-in address
(`https://api.tokenfactory.nebius.com/v1/`), asked the real server for its list of models without a key, then sent one
chat request with a made-up key. Neither can cost anything. `nebius_check.rs` gained `--no-key` for this.

```
cargo run -p steplet-app --offline --example nebius_check -- --no-key
```

```
Spending: none. No key is used.

== 1. The list of models, without a key
Refused, so the list needs the key: KeyRefused { status: 401, detail: "Couldn't authenticate. Reason: token is not present" }
Steplet would say: Nebius did not accept the key. Change it in Setup.

== 2. A made-up key
KeyRefused { status: 401, detail: "Couldn't authenticate. Reason: Unable authenticate" }
Steplet would say: Nebius did not accept the key. Change it in Setup.
```

What this shows: Steplet reaches the real Token Factory over HTTPS with Windows' own web library; the server's "wrong
key" answer is read as a refused key, not as a network fault; and the owner is told, in plain words, to change it in
Setup. What it does not show: anything about a real key, the models Nebius offers, tool calls, or prices. Those are the
paid part of Task 4, once the key is in.

## Task 5: The agent's moves, inside the fence (29 Oct 2026)

Each move Steplet's own agent asks for now runs in Steplet's Linux through `start-hands.sh`, one at a time: as the
steplet account with no powers, inside the run's fence, with a time limit. The fence is `srt-hands.json`, which
`prepare_own` writes from `fence_json`; `the_hands_fence_matches_claude_codes_fence` holds it to the same reading
rules and sites as Claude Code's. Whatever a move leaves running is ended before its answer comes back. The answer is
root's own first line (`steplet-hands exit=<n> timed_out=<0|1> bytes=<total>`), then at most 1 MiB of what the move
printed. The file tools (read, write, edit, list, find, search) are `hands.py`, run inside the same fence; they judge
nothing, because the checker has already decided. `root-common.sh` holds what `start-run.sh` (Claude Code) and
`start-hands.sh` share.

Not as the plan wrote it (rows N14 to N20 in `docs/decisions.md`):

- `LinuxHands::new` takes the run's `&RunPaths`, not two strings; `HandsError` has a fourth case, `Stopped`; and
  `start-hands.sh` has a fourth exit code, 33, for a program that would not stop.
- `hands.rs` installs `start-hands` and `hands.py` itself, before a run's first move; `launch.rs` installs only
  `root-common.sh`, beside `start-run`.
- What a move prints goes through a pipe that root reads, not straight into a file: the first 1 MiB is kept and the
  rest only counted, so a move that prints without end never fills the disk.
- The fence program starts in the run's temporary folder, and the move steps into the working copy from there.
- The fence holds the agent settings file names with empty placeholder files while a move runs, and normally removes
  them when it ends. If a move had to be killed, `start-hands.sh` removes any placeholders left behind, and `hands.py`
  leaves such files out of its lists.
- Two checks use different targets from the plan. The `/mnt` read uses a marker file in `/mnt`, because Windows' C:
  drive is never mounted in Steplet's Linux, so a read of `/mnt/c` would fail even without the fence. The network
  check uses two public sites, `example.org` (declared) and `example.net` (not), not the listener: a file declares
  sites by name, and the listener is an address on this computer.

Run 1, with all nine tests written first: the fence program would not start at all.

```
> cargo test -p steplet-run --offline --test hands -- --ignored --nocapture --test-threads=1
    exit Some(1), timed out false, 77 bytes: "bwrap: Can't create file at /work/n5a/repo/.git/hooks: Read-only file system\n"
test result: FAILED. 1 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 116.01s
```

The fence program guards some file names in the folder it starts in. Started in the working copy, it tried to put one
inside `.git`, which the fence keeps read-only. It now starts in the run's temporary folder (N17).

Run 2: two failures left.

```
thread 'a_command_runs_in_the_copy_and_its_exit_code_is_steplets' panicked at crates\steplet-run\tests\hands.rs:179:5:
assertion `left == right` failed: the steplet account, the run's own folder, no input
  left: "steplet\n/tmp/steplet-run-n5b /tmp/claude\nend\n"
 right: "steplet\n/tmp/steplet-run-n5b /tmp/steplet-run-n5b\nend\n"
thread 'a_command_cannot_forge_its_exit_code' panicked at crates\steplet-run\tests\hands.rs:190:5:
  left: "steplet-hands exit=0 timed_out=0 bytes=1\n/bin/bash: line 1: /dev/stdout: Permission denied\n"
test result: FAILED. 7 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 131.73s
```

The fence program set the move's temporary folder to `/tmp/claude`, which the fence keeps read-only; it now names the
run's own (N18). A move could not open its own output a second time, as `> /dev/stdout` does, because root's pipe
let only root write to it; the move may now write to it, and still only root can read it (N16).

Run 3:

```
> cargo test -p steplet-run --offline --test hands -- --ignored --nocapture --test-threads=1
     Running tests\hands.rs (target\debug\deps\hands-c857ad0f19d638b2.exe)
--- control, outside the fence: {"proxy": "none", "https://example.org/": "reached", "https://example.net/": "reached"}
--- Command { line: "python3 -I probe.py probe_results.json" }
    exit Some(0), timed out false, 153 bytes: "{\"proxy\": \"set\", \"https://example.org/\": \"reached\", \"https://example.net/\": \"failed: URLError: <urlopen error Tunnel connection failed: 403 Fo
    "sleep 60" came back after 3.5 s
    "trap '' TERM; sleep 60" came back after 3.6 s
test the_fence_holds_without_the_checker ... --- control, outside the fence: "grep -q steplet-marker-hidden /mnt/steplet-marker-n5d": Some(0)
--- control, outside the fence: "test -w /tmp/steplet-marker-n5d": Some(0)
--- control, outside the fence: "touch /work/n5d/repo/.git/steplet-control": Some(0)
--- Read { path: "/mnt/steplet-marker-n5d", offset: None, limit: None }
    exit Some(1), timed out false, 37 bytes: "There is no /mnt/steplet-marker-n5d.\n"
--- Write { path: "/tmp/steplet-marker-n5d", content: "changed" }
    exit Some(1), timed out false, 48 bytes: "/tmp/steplet-marker-n5d: Read-only file system.\n"
--- Command { line: "echo changed >> .git/HEAD; echo changed >> .git/config" }
    exit Some(1), timed out false, 106 bytes: "/bin/bash: line 1: .git/HEAD: Read-only file system\n/bin/bash: line 1: .git/config: Read-only file system\n"
--- Write { path: ".mcp.json", content: "{}" }
    exit Some(1), timed out false, 24 bytes: "Not allowed: .mcp.json.\n"
--- Write { path: "inside.txt", content: "inside" }
    exit Some(0), timed out false, 29 bytes: "Wrote 6 bytes to inside.txt.\n"
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 99.84s
```

(Lines cut from that output: the other moves of each test, each also as expected.) The moves in
`the_fence_holds_without_the_checker` go straight to the hands, with no checker in front, so what stopped them is the
fence alone.

The tests were then shown to catch a missing fence: the fence program was taken out of the move's line in
`start-hands.sh`, the three fence tests were run, and the script was put back (`cmp` against a copy: identical).

```
--- the deliberate break:
64c64
<     "$SRT" --settings "$R/srt-hands.json" -- /usr/bin/env -C "/work/$ID/repo" "${MOVE[@]}" < "$INPUT" >&3 2>&3 3>&-
---
>     /usr/bin/env -C "/work/$ID/repo" "${MOVE[@]}" < "$INPUT" >&3 2>&3 3>&-
> cargo test -p steplet-run --offline --test hands -- --ignored --test-threads=1 the_fence_holds the_run_folder a_command_reaches
a site the file does not declare: reached
assertion `left != right` failed: Read { path: "/mnt/steplet-marker-n5d", offset: None, limit: None } must fail inside the fence:      1	steplet-marker-hidden
assertion `left != right` failed: Read { path: "/var/lib/steplet/runs/n5i/srt-hands.json", offset: None, limit: None } must fail inside the fence:      1	{
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 6 filtered out; finished in 18.41s
```

Afterwards nothing was left in Steplet's Linux: `/work`, the runs folder and `/tmp` held no run or marker, and no
program of the steplet account was running.

The Claude Code runner still works after the shared root helpers moved into `root-common.sh` and prepare learned the
`own` mode: every Linux test of Claude Code's runs was run again (no run held `Local\Steplet-run` at 18:29).

```
> cargo test -p steplet-run --offline --test ask_live --test contain --test gate --test launch --test links --test prepare --test report --test restart --test settings_live --test sites --test tripwires --test uc5 -- --ignored --nocapture --test-threads=1
     Running tests\ask_live.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 122.98s
     Running tests\contain.rs
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 182.55s
     Running tests\gate.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 58.34s
     Running tests\launch.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 75.48s
     Running tests\links.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 93.19s
     Running tests\prepare.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 38.65s
     Running tests\report.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 30.37s
     Running tests\restart.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 76.52s
     Running tests\settings_live.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 75.11s
     Running tests\sites.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 9.14s
     Running tests\tripwires.rs
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 50.85s
     Running tests\uc5.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 29.17s
```

(56 passed, none failed.) Then `cargo fmt --all`, `cargo clippy -p steplet-run --offline --all-targets -- -D warnings`
(`Finished`, no warnings) and `cargo test -p steplet-run --offline`: every test binary `ok`, none failed (the Linux
tests are ignored in that run and were run above).

## Task 6: The agent loop, with a stand-in model (29 Oct 2026)

Steplet's own agent now runs end to end in code: `run_agent_locked` prepares the run, then `drive` asks the model
for its next answer, puts each move it asks for through the checker (`Asker::decide`, recorded as `Asked::InApp`),
and runs only the allowed ones through the hands, built from the request the checker saw. The model is called on a
thread of its own, so Stop and the time limit are looked at every 100 ms even while a call hangs. Before each call,
the most it could cost is held against the cap. Each answer's tokens and cost go to the screen (`Progress::Spent`).
Busy and failing service answers are tried again after 2, 5 and 10 s, each shown as `Event::ApiRetry`; a refused key
or an empty account ends the run at once with plain words. The run's events use Claude Code's own shapes, so
`build_report` serves both runners. This task used no Linux and no Nebius: the model is the stand-in `Scripted`,
and the hands are a stand-in that records each move.

Not as the plan wrote it: rows N21 to N35 in `docs/decisions.md`. The biggest: no `exit` field on a result (the
result's text carries "Exit code N", as Claude Code's does); `drive` takes one `Agent` value and is reachable from
the tests; a call that fails after Nebius may have charged is counted at the most it could have cost; and a limit
reached is recorded as a limit, not as a fault.

The 14 tests in `crates/steplet-run/tests/agent.rs`, with the spending test's last case tightened: a cap of exactly
the most the first call could cost lets that one call through, and its move runs, then the run ends at the cap.

```
> cargo test -p steplet-run --offline --test agent
running 14 tests
test a_malformed_tool_call_is_answered_not_run ... ok
test a_refused_key_ends_the_run_with_plain_words ... ok
test what_runs_is_what_was_checked ... ok
test the_report_shows_steplets_exit_code_not_the_models_claim ... ok
test the_step_limit_ends_the_run ... ok
test a_refusal_goes_back_to_the_model ... ok
test the_nebius_key_never_appears_in_the_report_or_logs ... ok
test three_unreadable_replies_in_a_row_end_the_run ... ok
test every_call_goes_through_the_checker ... ok
test huge_output_is_cut_for_the_model ... ok
test the_spending_cap_ends_the_run_before_the_call_that_could_pass_it ... ok
test busy_nebius_is_retried_and_shown ... ok
test the_time_limit_ends_the_run ... ok
test stop_during_a_model_call_ends_the_run_promptly ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s
```

Three deliberate breaks in `agent.rs`, each put back byte for byte (`cmp` with a saved copy) before the next. The
breaks were made before `cargo fmt` reflowed `agent.rs`, and before the spending test's tightened case.

```
BREAK 1: a refused call runs anyway
test a_refusal_goes_back_to_the_model ... FAILED
test every_call_goes_through_the_checker ... FAILED
test result: FAILED. 12 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

BREAK 2: the spending cap is never checked
test the_spending_cap_ends_the_run_before_the_call_that_could_pass_it ... FAILED
test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

BREAK 3: events go to the screen without the key hidden
test the_nebius_key_never_appears_in_the_report_or_logs ... FAILED
test result: FAILED. 13 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
```

Format and lint, after the tightened test:

```
> cargo fmt --all -- --check
fmt exit 0
> cargo clippy --offline --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.60s
clippy exit 0
```

### A fault found on the way: a Stop at the very start of a run can break name lookup

The first rerun of the Linux tests failed three `launch` tests, each at the clone:

```
test a_stop_right_after_launch_never_lets_the_agent_act ... --- stopped at once; exit code Some(9); every event:
--- acted: no; programs of the steplet account left: 0; start scripts left: 0
ok
test stop_ends_a_run_mid_command_and_leaves_nothing_running ...
thread 'stop_ends_a_run_mid_command_and_leaves_nothing_running' (55152) panicked at crates\steplet-run\tests\launch.rs:74:34:
called `Result::unwrap()` on an `Err` value: Step { step: "clone the repository", detail: "fatal: unable to access 'https://github.com/octocat/Hello-World.git/': Could not resolve host: github.com\nsteplet-prepare: could not clone https://github.com/octocat/Hello-World.git" }
test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 1 filtered out; finished in 52.43s
```

The test before it had checked that `/mnt/wsl` was Steplet's folder after its run. After the Stop-at-once test,
`/mnt/wsl` held no mount at all, so `/etc/resolv.conf` (WSL's link to `/mnt/wsl/resolv.conf`) led nowhere. Steplet's
Linux log shows `mnt-wsl.mount: Deactivated successfully` at 13:42:21 UTC, the moment of that test, and nothing put
back after it. The cause is in `close_shared_folders` (`root-common.sh`, the same lines as `start-run.sh` at 956e4ab,
so the fault predates this copy): it reads the name lookup file, removes WSL's `/mnt/wsl`, then mounts Steplet's own
folder and writes the copy into it. A Stop that kills the start step between the removal and the new mount loses the
only copy, and every later run fails at the site lookup or the clone until Steplet's Linux restarts.

Three tries of the Stop-at-once test alone did not hit the gap (it is a few milliseconds wide). To confirm the cause
without luck, a throwaway copy of the step inside Linux had a 5 s pause put into the gap and was killed there:

```
--- before: /mnt/wsl is none
--- before: lookup ok
--- killed in the gap, exit code 137
--- after: /mnt/wsl is NONE
--- after: lookup FAILED
--- after: lookup file UNREADABLE
--- the next run's step, unchanged, from this state:
cat: /etc/resolv.conf: No such file or directory
steplet-start: could not read the name lookup file
--- its exit code: 30
```

`wsl --terminate steplet` (lock free, nothing of Steplet's running) put WSL's folder back each time; `getent hosts
github.com` then answered `20.207.73.82`. The repair itself is not in this task: it changes the shared start step and
both runners' first step, so it waits for a yes (Task 6b in the plan).

Steplet's own agent meets the same gap once per move, since `start-hands.sh` runs the same step at the start of
every move. A Stop ends a move by killing `wsl.exe` from Windows. Whether that ends the root script inside Linux at
once was checked with two throwaway scripts that count to 10, one with traps that write down any signal and one
without; each had its `wsl.exe` killed at 1.5 s, all three streams piped, as `LinuxHands` starts it:

```
exp.sh (traps: yes): counted [1 2 3 4 5 6 7 8 9 10 finished ] signals [HUP ]
exp2.sh (traps: no): counted [1 2 3 4 ] signals []
```

The script is hung up at the kill and, like `start-hands.sh`, which sets no trap, dies on the spot.

### The Linux tests, rerun

`ask_live`, `contain` and `gate` passed in the first rerun, before the fault above (4, 12 and 6 passed). With
Steplet's Linux restarted, the groups from `launch` on, each followed by a look at `/mnt/wsl` and the name lookup
(`hands` ran in Task 5 and is unchanged here; `setup` reinstalls Linux and is not run):

```
19:24
lock held: False
     Running tests\launch.rs
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 91.52s
=== after: /mnt/wsl is steplet-wsl; lookup ok
     Running tests\links.rs
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 146.86s
=== after: /mnt/wsl is steplet-wsl; lookup ok
     Running tests\prepare.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 57.06s
=== after: /mnt/wsl is steplet-wsl; lookup ok
     Running tests\report.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 39.41s
=== after: /mnt/wsl is steplet-wsl; lookup ok
     Running tests\restart.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 101.97s
=== after: /mnt/wsl is steplet-wsl; lookup ok
     Running tests\settings_live.rs
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 109.44s
=== after: /mnt/wsl is steplet-wsl; lookup ok
     Running tests\sites.rs
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 24.24s
=== after: /mnt/wsl is steplet-wsl; lookup ok
     Running tests\tripwires.rs
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 70.03s
=== after: /mnt/wsl is steplet-wsl; lookup ok
     Running tests\uc5.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 51.65s
=== after: /mnt/wsl is steplet-wsl; lookup ok
```

The one failure is `a_hook_in_the_run_s_settings_stops_the_run`. From the tripwire on, the gate refused every
request, as it must; but the stand-in model reached the last of its 31 quick steps before Steplet's Stop ended
Claude Code:

```
--- 4.1 s; exit code Some(0); stopped_by Some(Hook("PreToolUse:Bash")); faults []; every event:
--- requests that reached the gate after the hook: 30
thread 'a_hook_in_the_run_s_settings_stops_the_run' (59172) panicked at crates\steplet-run\tests\tripwires.rs:303:5:
the run was stopped before the script's end
```

Neither `watch_run` nor `stop` changed in this task; in Task 5's rerun the same test stopped the run at 2.8 s after
9 requests. Three reruns of the test alone:

```
--- 3.0 s; exit code Some(143); stopped_by Some(Hook("PreToolUse:Bash")); faults []; every event:
--- requests that reached the gate after the hook: 13
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 10.60s
--- 3.1 s; exit code Some(143); stopped_by Some(Hook("PreToolUse:Bash")); faults []; every event:
--- requests that reached the gate after the hook: 12
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 10.74s
--- 3.0 s; exit code Some(124); stopped_by Some(Hook("PreToolUse:Bash")); faults []; every event:
--- requests that reached the gate after the hook: 13
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 10.74s
```

So the Stop was slower that once, and 31 quick steps are too thin a margin for the test's last checks. Widening it
is Task 6c in the plan, waiting for a yes.

### The ordinary tests, after the last change

`cargo test -p steplet-run -p steplet-app --offline`: 43 test programs, every one `ok`, 266 passed, none failed,
77 ignored (the Linux and live tests). The runner: 178 passed, 69 ignored; the app: 88 passed, 8 ignored.

Not checked in this task: a whole run of Steplet's own agent in real Linux (`run_agent_locked` with `prepare_own`,
`LinuxHands` and the loop together). The loop's tests use stand-in hands; the hands' tests (Task 5) use no loop. The
first whole run is Task 10's, with Nemotron.

## Task 6b and 6c: a Stop at any moment no longer breaks name lookup (29 to 30 Oct)

### What was wrong

Root's start step swaps WSL's `/mnt/wsl` for Steplet's own folder, and that folder held the only copy of the name
lookup file. A Stop that killed the step between "take /mnt/wsl away" and "put the file back" left Steplet's Linux
unable to look up any web address until Windows or WSL restarted. Steplet's own agent runs that step at every move, so
it met the gap once per move. Now root keeps its own copy (`/var/lib/steplet/resolv.conf`, root's, 644), saves it
whenever the lookup file reads, and puts the file back from it: at the start of every start step, and before every run
needs a web address (decision N36).

### The tests, seen failing before the repair

The first failing run (all five new tests) had one mistake of my own: the helper expected `cat README` to be
"allowed", but the checker rightly passes it as "not asked, harmless". That check was dropped from the helper. The
other failures were the real ones:

```
--- the run, with lookup broken and no copy: Some(Prepare(Step { step: "clone the repository", detail: "fatal: unable to access 'https://github.com/octocat/Hello-World.git/': Could not resolve host: github.com\nsteplet-prepare: could not clone https://github.com/octocat/Hello-World.git" }))
test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 3 filtered out; finished in 55.29s
Command { line: "true" }: Layout("steplet-hands: could not read the name lookup file")
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 7.53s
```

The second failing run, with the helper fixed: both gap tests fail at the clone, and the copy test finds no copy
(root's `cat` of it exits 1):

```
called `Result::unwrap()` on an `Err` value: Prepare(Step { step: "clone the repository", detail: "fatal: unable to access 'https://github.com/octocat/Hello-World.git/': Could not resolve host: github.com\nsteplet-prepare: could not clone https://github.com/octocat/Hello-World.git" })
  left: Some(1)
 right: Some(0)
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 4 filtered out; finished in 81.62s
```

### With the repair

```
test a_run_after_a_stop_in_the_gap_still_looks_up_names ... --- run n6ba: 12.4 s
test a_run_after_a_stop_just_after_the_new_folder_still_looks_up_names ... --- run n6bb: 13.3 s
test the_lookup_copy_is_roots_alone ... --- run n6bd: 9.6 s
test with_no_copy_a_run_after_a_stop_in_the_gap_is_refused_in_plain_words ... --- after the break (umount -R -l /mnt/wsl): the lookup file does not read; a lookup fails
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 117.11s
test a_move_after_a_stop_in_the_gap_still_runs ... --- Command { line: "true" }
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 9.68s
```

### Task 6c: the tripwire group, four times

The hook test's script now has 300 refused steps instead of 30. The group passed every time it ran (once more than
the three the plan asks for, because a background run was restarted part way):

```
--- requests that reached the gate after the hook: 13
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 53.16s
--- requests that reached the gate after the hook: 16
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 69.72s
--- requests that reached the gate after the hook: 15
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 58.51s
--- requests that reached the gate after the hook: 32
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 90.80s
```

The 32 in the last run shows why the margin matters: that Stop came well after the old limit of 30 steps would have
let the script finish.

### Every Linux group, after 6b and 6c

`--ignored --test-threads=1`, every group but `setup`:

| Group | Result |
|---|---|
| ask_live | 4 passed, 112.51 s |
| contain | 12 passed, 270.55 s |
| gate | 6 passed, 93.58 s |
| hands | 10 passed, 126.86 s |
| launch | 6 passed, 92.38 s |
| links | 5 passed, 85.53 s |
| prepare | 8 passed, 49.17 s |
| report | 2 passed, 31.79 s |
| restart | 7 passed, 170.58 s |
| settings_live | 3 passed, 88.38 s |
| sites | 2 passed, 32.89 s |
| tripwires | 4 passed, 60.03 s |
| uc5 | 1 passed, 31.47 s |

70 passed, none failed.

### After review

A reviewer read the whole change and found nothing blocking: no moment where a kill leaves lookup broken for good, and
no way for the steplet account to change the copy. Four of its points were taken:

1. The refusal said "Steplet's Linux cannot look up web addresses", which is wrong when only saving the copy failed
   (a full disk, say). It now says "Steplet could not keep name lookup working in its Linux (<reason>)", which is true
   either way. The test was changed first and seen failing:
   ```
    right: "Steplet will not run this file: Steplet could not keep name lookup working in its Linux (the name lookup file is gone, and Steplet has no copy of it)."
   test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 21.68s
   ```
2. A step that failed without saying why gave an empty "()". It now gives the exit code. New unit test, seen failing
   (`cannot find function plain_reason`), then:
   ```
   test launch::tests::a_root_step_s_reason_is_said_in_plain_words ... ok
   ```
3. The tests' put-back helper now says so if lookup still fails after it put the file back. It only says it, because
   a panic there, while a failed test unwinds, would end the whole test program.
4. The tripwire comment now says that 300 widens the margin; it does not remove the race.

Two were declined: saving the copy only when it has a `nameserver` line (the lookup file is written in one small
write, so a kill leaves it empty, which is already handled, or whole; and the check would refuse a Linux set up
without that line), and syncing the copy to disk (that matters only after a full crash, and WSL rebuilds the lookup
file when it restarts).

### After the review changes

Rerun of the four Linux groups the review changes touch, then the ordinary tests, format and lint (30 Oct, 05:45 to 06:16, the run lock free throughout):

| Group | Command | Result |
|---|---|---|
| restart | `cargo test -p steplet-run --offline --test restart -- --ignored --test-threads=1` | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 550.88s` |
| hands | `cargo test -p steplet-run --offline --test hands -- --ignored --test-threads=1` | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 257.09s` |
| launch | `cargo test -p steplet-run --offline --test launch -- --ignored --test-threads=1` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 203.99s` |
| contain | `cargo test -p steplet-run --offline --test contain -- --ignored --test-threads=1` | `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 651.10s` |
| ordinary | `cargo test -p steplet-run -p steplet-app --offline` | every target `ok`; summed: 267 passed, 0 failed, 82 ignored (the Linux ones); `ordinary exit 0` |
| format | `cargo fmt --all -- --check` | `fmt exit 0` |
| lint | `cargo clippy --workspace --all-targets --offline -- -D warnings` | `` Finished `dev` profile [unoptimized + debuginfo] target(s) in 28.15s ``; `clippy exit 0` |

With `--ignored`, the launch group's one ordinary test is left out ("1 filtered out"); it runs with the ordinary tests.

### Departures from the plan

- The run tests went in `restart.rs`, not `launch.rs`: `restart.rs` already holds the "a run after an earlier run was
  cut short" tests and their helpers.
- One extra test: with no copy, the run is refused in plain words before anything is made.
- The attack suite's look after each case now expects the copy in `/var/lib/steplet`.
- `long_request`'s turn limit went from 60 to 400, so a run Steplet did not stop would reach the end of the 300-step
  script rather than the turn limit.
- The refusal's wording changed after review (above).
- The `setup` group was not run: it reinstalls Steplet's Linux, and one of its tests installs a second Linux from the
  internet, which needs the owner's yes.

### Not checked

- A real Stop landing inside the gap. The tests break lookup the way the gap does, with the same commands; they do not
  time a real Stop into those few milliseconds.
- The copy after a full crash of WSL: it is not synced to disk.
