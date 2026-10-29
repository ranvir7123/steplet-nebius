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