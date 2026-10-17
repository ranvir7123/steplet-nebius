# Milestone 1, phase 4: the desktop app — build plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans, run inline in one session (the owner's choice since phase 0: not through separate agents). Steps use checkbox (`- [ ]`) syntax for tracking.

Written 18 Oct 2026. Status: **written, not approved.** Nothing in it has been built, installed or run, apart from the read-only checks in section 3.6. The choices in section 4 (J1 to J9) wait for the owner.

**Goal:** a Windows app in which a receiver opens a `.steplet` file, sees everything it asks for, chooses to run it, answers the agent's questions while it runs, and saves an honest report; driving phase 3's runner, never adding a second checker.

**Architecture:** one new Rust crate, `steplet-app`, is the Tauri 2 shell: it reads files with `steplet-core`, runs them with `steplet-run`, keeps the receiver's key and known senders, and sends the screens plain data in which every piece of file text is already cut into the core's display segments. The screens are Svelte 5 in `web/app`, with the parts the public check page will reuse in `web/shared`. The screens talk to Rust through one interface with two versions: the real one (Tauri) and a stand-in that replays saved data, so every screen in every state can be screenshotted without a run. `steplet-run` gains a live mode (events as they happen, cleaned; Stop; the receiver's answers to questions) and product setup; `steplet-core` gains one rule and one checker change.

**Tech stack:** Rust 1.98.0 stable (MSVC), edition 2024; Tauri 2.12.1 (crates already in cargo's cache, see 3.6); Svelte 5.57.1, Vite 8.3.1, TypeScript 6.0.3, svelte-check 4.7.6, `@tauri-apps/api` and `@tauri-apps/cli` 2.12.1, `@playwright/test` 1.63.0 with its Chromium 1243 (all already in the npm cache and on disk, from phase 0's S9); `windows-sys` 0.61.2 (cached) for Windows' own Credential Manager, registry and Save dialog; `ts-rs` 12.0.1 if J7 is A (a download). WebView2 as Windows ships it.

**Spec:** `docs/plans/2026-10-08-milestone-1-plan.md` (section 1 "The receiver", section 6 "Phase 4: the desktop app" and its done-means, section 7), `docs/decisions.md` (rows 16, 22, 28 to 30, 32, R6, R7, R9 to R11, R16, D3, D4, E1, H1 to H13), and the open points at the end of `docs/milestone-1-phase-3-results.md`.

Words used below, explained once:
- **Screen:** one page of the app (open, preview, run chooser, setup, run, ask, report).
- **Tauri:** the kit the app is built with: a Rust program that shows its screens in Windows' own web view (**WebView2**, the part of Edge that draws web pages inside other programs).
- **Svelte:** the tool the screens are written in. **Playwright:** a tool that drives a browser from a test and takes screenshots.
- **Display segments:** phase 1's way of cutting file text into plain text, badges for invisible characters, and links with their real address (`steplet_core::display::segments`).
- **Stand-in backend:** a fake version of the app's Rust side, used only by screen tests, that replays data saved from real Rust output.
- **Fingerprint:** a SHA-256 code computed from an exact request; any change to the request gives another code.
- **Pin:** Steplet's note that a sender name was seen with a given signing key (trust on first use, row 16).
- **CDP:** the Chrome DevTools Protocol, the channel Playwright uses to drive a Chromium browser such as WebView2.

## Global constraints

Every task includes these. Values come from the milestone plan (section 8), the decisions log and the phase 3 results.

- Rust 1.98.0 stable (MSVC), edition 2024. `cargo` always runs with `--offline`; a crate not in cargo's cache stops the task until the owner says yes (section 5).
- `npm` always runs with `--offline` (and `npx` only for programs already installed in `web/node_modules`); a package not in the npm cache stops the task until the owner says yes.
- `steplet-core` stays `no_std` + `alloc`. Its only changes in this phase: the checker change of J1 (Task 1) and the sender-key rule (Task 7).
- **One checker (row 29):** the gate's answer and the question on the ask screen come only from `Policy::check`. The app writes no matcher and never decides a colour: it shows the core's outcome, plus findings about this computer that come from the core's own rule (Task 7) and from the runner's own facts (its host list, its setup state).
- **File text on screen:** every text that comes from the file, the repository, the agent or a command reaches the screens only as display segments and is drawn only by `web/shared/FileText.svelte`. Svelte's raw-HTML tag, `innerHTML`, `outerHTML`, `insertAdjacentHTML` and `document.write` are refused by a check script (Task 8). Markdown is never turned into HTML.
- **Fixed copy (milestone plan, phase 4):** the window title is always `Steplet`; no text from a file appears in a window title, a dialog, the taskbar, or the default name of a saved report. Text from the file sits only inside panels marked "From the file" (Task 10).
- **No network while previewing (row 28):** opening and previewing a file make no network request. The app's content-security rule allows nothing from outside the app; links in file text are shown with their real address and are not clickable.
- **Opening a file changes nothing on disk.** Pins are written only when the receiver presses Run.
- **Promises (R6, R20):** each line the run chooser shows names the test that backs it (Task 5), worded no wider than phase 3's results showed.
- **Runs:** everything phase 3 fixed stays (launch with `SystemRoot` only, deny-only settings, the tripwires, the report built from Steplet's records). The only settings change is Task 1's rule.
- **Steplet never asks for the receiver's password and never changes their own Windows or Linux,** except what they switch on in Setup themselves (J4).
- **UI quality (owner's rule):** every screen, in every state of section 8, is screenshotted, looked at and fixed until right; light and dark (J8); at the normal size (1100 × 800) and the narrow size (560 × 720); every control reachable by keyboard with a visible focus ring; colour never the only signal (each outcome has a word and an icon); text contrast at least 4.5 : 1, checked by a script (Task 8).
- Evidence: no task is done without its command and its pasted, unchanged output in `docs/milestone-1-phase-4-results.md`; screenshots go in `docs/screens/phase-4/`.
- Commits go on this worktree's branch, never straight onto `main`. Nothing is pushed unless the owner says so. No attribution lines anywhere. Plain words in messages. Approving this plan approves the commits it lists.
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --offline -- -D warnings` pass for every task that changes Rust; from Task 8 on, `npm --prefix web run check` passes for every task that changes `web/`.
- **F12:** every crates.io package in every lock file is at least 14 days old and not yanked; checked with `python -I C:\Users\RANVIR\.claude\handoffs\2026-10-11-1606-steplet-tools\lock_ages.py <lock file>` after any lock file changes and at the wrap-up. The same rule covers `web/package-lock.json` (smaller choice S10).
- McAfee on this laptop sometimes holds a freshly built program (`LNK1104`): build with `--no-run` first, retry up to three times, then run. Never build under `%TEMP%`.

## Review Focus

Inputs and conditions the milestone plan implies but no feature test would meet by accident, most likely first. Each has its test in the task named.

1. **A question arrives while the receiver is in another window, or two arrive at once.** Expected: the taskbar button flashes, the question waits with its timer, the second waits behind the first, and each answer goes to its own question. Task 12: `a_question_flashes_the_taskbar_and_waits_its_turn`, and Task 4: `two_questions_at_once_each_get_their_own_answer`.
2. **A second `.steplet` file is double-clicked while a run is going in the first window.** Expected: the second window previews freely, and its Run says another run is in progress; the first run is untouched (one `/work` folder holds one run). Task 3: `a_second_run_cannot_start_while_one_is_running`; Task 12: `run_is_refused_while_another_window_runs`.
3. **A task prints a huge amount** (50,000 lines). Expected: the live log stays responsive and shows the first 40 lines of each output with "Show all"; the report holds everything. Task 12: `a_50000_line_output_keeps_the_log_responsive`.
4. **Steplet is closed, or crashes, during a run.** Expected: closing asks first; after a crash, the next start stops what is left, removes the run's folders, and says so. Task 12: `closing_during_a_run_asks_first` and `a_run_left_by_a_crash_is_cleared_at_start`.
5. **File text made to look like Steplet's own words** (a title such as "✅ Verified by Steplet — safe to run", newlines inside one entry, a right-to-left override). Expected: it stays inside its "From the file" panel in file style; a newline inside one entry shows as a badge, so one entry never looks like two. Task 9: `a_newline_inside_one_entry_shows_as_a_badge`; Task 10: `text_that_imitates_steplet_stays_in_its_panel`.

---

## 1. Who this phase is for, and what they see

### The receiver (the person this phase is for)

**Who:** a developer on Windows 11 who has received a `.steplet` file from another developer by chat or email. They may have used WSL before, or not. They have an Anthropic API key, or can make one (P1).

**What they are trying to get done:** one of R27's contained use cases: run a public repository's tests at an exact commit and get the result (use case 1); run a triage script and get its log (use case 2); and stay safe when the file is the look-alike attack (use case 5).

**What they see and touch, screen by screen:**

1. **Open.** They double-click the file (once J4's switch is on), drag it onto the window, or press "Open a file…". The window is titled `Steplet` whatever the file says. Empty state: "Open a .steplet file to see what it asks for. Nothing in it runs until you press Run." A small "Setup" link leads to screen 4 at any time.
2. **Preview.** From top to bottom:
   - the file's colour as a word, an icon and one line of meaning (green: "Nothing in this file asks for more than it declares, and everything declared is low risk" (R11); amber: "Continue only if you trust the sender"; red: "Run only if you are sure: you will type a sentence first"; blocked: "This file cannot run");
   - one identity line from the signature and the pins: "Not signed", "Signature valid, sender unknown" (R14), "Signed with the same key as the file from 'Ana Lee' you ran before", or the red finding "This sender name was used before with another key" (Task 7);
   - the counts first: steps, commands, web sites, folders it reads and writes, never rules, expiry date;
   - the findings, worst first, each with its title, detail, advice and line number;
   - then, in panels marked "From the file": title, sender, repository and commit, every step, every allowed command (whole, wrapped, never cut), sites, folders, the never list, and the body as plain text;
   - "On this computer": what is true here and not in the file (a changed sender key; a repository host Steplet does not fetch from; no repository);
   - **the Run button last**, after the full list (row 28). Red: a box above it where they type "I understand the risks and want to run this file" before Run works. Blocked: no Run, just the first blocked finding's advice.
3. **Run chooser.** One runner that can run: "Claude Code, in Steplet's own Linux", with the lines it may promise for this file (Task 5), and what leaves the computer: "Claude Code sends what the agent reads, the commands' output and this task to Anthropic, using your API key" (R16). Codex is listed as "Not available for files that run code on this computer", with the reason (D1). If Steplet's Linux or the key is missing, the button says "Set up first".
4. **Setup (once).** A checklist with a state for each line: Windows' Linux feature (WSL 2); Steplet's own Linux; its tools (git, Python, Claude Code 2.1.220, `srt` 0.0.77 and the rest of H4's list); the gate program for this build; the Anthropic key; "Open .steplet files with Steplet" (J4). One button installs Steplet's Linux, with a progress log; "Check again" re-reads every line. The key goes in a password box; saved, it shows only "Saved", never the key.
5. **Prepare.** "Fetching github.com/… → Checking out commit 7fd1a60b… inside the fence → Removing agent settings files: `.claude`, `.mcp.json`. Steplet tells the agent not to load agent settings" (D3), then straight into the run.
6. **Run and live log.** Elapsed time and time limit, a Stop button, and one row per event: "The agent says" (quoted style), each command or file action with Steplet's check ("Allowed: the file asks for this command", "You allowed this", "Refused: …", "Ran without asking Steplet: a harmless read"), its exit code and its output (first 40 lines, "Show all"), a model refusal (E1), and a tripwire stop in plain words. The log follows the newest row unless they scrolled up.
7. **Ask.** When the agent wants something the file did not ask for, the run pauses on a question that covers the window and flashes the taskbar: what exactly ("Run this command", "Change this file", "Read this file", "Open this web address") with the whole request, nothing cut; why Steplet asks (the checker's reason); what the agent says about it, labelled; what still holds if they say yes ("It still runs inside the fence: …"); a countdown ("No answer in 4:59 means no"). Two buttons: **No** (focused first) and **Yes, allow this exact request for this run**, which works only one second after the question appears, so a click meant for something else cannot answer it. "2 more questions are waiting" when there are.
8. **Report.** First the outcome in plain words (finished; Steplet stopped it and why; you stopped it: "Interrupted; the folder was reset; run again?"; the model refused, with the model, its category and its explanation, and a "Run again, allowing another model" button (E1); it did not finish). Then every command with its exit code from the run and its full output; what the checker refused; the receiver's answers; every model used and any refusal; what changed in the working copy (which is already deleted); the trust check; what went wrong around the run; Steplet's note that blocked fence attempts are invisible to it; and last, labelled, "The agent says". Buttons: "Save report…" (Windows' own Save dialog, a `.txt` file), "Run again", "Open another file".

**Where they get stuck, and what the app does:**

| Stuck point | What the app does |
|---|---|
| WSL is not on this computer, or too old | Setup says Steplet cannot run files safely here yet, shows the exact command for them to type in a terminal opened as administrator (`wsl --install --no-distribution`, then a restart), and links to Microsoft's page. Steplet never runs it. |
| Installing Steplet's Linux fails (no network, no disk space) | The log shows WSL's own message; "Try again" re-runs the setup script (it is safe to repeat); "Start over" removes Steplet's Linux only (`wsl --unregister steplet`; nothing of theirs is in it). |
| No key, or a mistyped key | The chooser sends them to Setup. A wrong key ends the first run with "Anthropic did not accept the key. Change it in Setup." (measured in Task 12, never guessed). |
| They do not know what amber or red means | Each colour has its one-line meaning; red needs the typed sentence; blocked shows the advice of its first finding. |
| The repository is on a host Steplet does not fetch from (H9), or the file names none | The preview's "On this computer" says so before Run; Run is not offered. |
| The fetch fails (wrong address, private repository, no network) or the commit is missing | Prepare says which, in Steplet's words with git's reason cleaned, and that nothing ran; "Try again" and "Back". |
| The agent asks for something | The ask screen; no answer in 5 minutes means no (smaller choice S1). |
| The run stops (Stop, a tripwire, a crash, sleep) | "Interrupted; the folder was reset; run again?" (D4); after a crash, the next start clears what is left. |
| A second file is opened during a run | It previews in its own window; its Run says another run is in progress. |
| The report is long | Each section scrolls; nothing is cut; Save writes all of it. |

### The sender

Unchanged by this phase: they write the file by hand and use `steplet-sign check` and `sign`. What changes for them is only what the receiver sees: the identity line, and the red finding if they sign with a new key under a name the receiver already knows.

### You, the builder

You get: the app (`steplet-app` and `web/`), the runner's live mode and product setup, two safety closings carried over from phase 3 (Task 1 and Task 3), screenshots of every screen in every state, one real use case 1 run from double-click to saved report, and a results file with every command and its output.

---

## 2. What this phase delivers, and what it does not

**Delivers** (milestone plan, "### Phase 4", adjusted as section 3 explains):
1. The Tauri shell with a fixed window title, a locked-down content-security rule, no Tauri plugins, and its own small Windows code (Credential Manager, the per-user file-type entry, the Save dialog).
2. The screens: open, preview, run chooser, setup (with the key and the file-type switch), prepare, run and live log, ask, report.
3. Sender pins: trust on first use, and a red finding when a known sender name comes with another key (rows 16, 22).
4. Re-checks when Run is pressed (row 30): expiry with the time now, the file's SHA-256 against the one previewed, the red sentence; the exact command is still checked by the gate at run time.
5. The runner's live mode: events on screen as they happen, cleaned exactly like the report; Stop; one run at a time on this computer; the receiver's answers, each bound to the exact request (R7).
6. The two phase 3 open points: a deny rule for writing into Steplet's own run folder (and J1), and cleaning of the live log (and J6 for the raw logs on Windows).
7. Product setup in `steplet-run` (until now a test script).
8. Screenshots of every screen in every state, looked at and fixed, and one use case 1 run from double-click to saved report.

**Does not deliver:**
- the `steplet://` link scheme (J5, recommended for milestone 3 with the link page);
- an installer, code signing, or the winget listing (rows 45, 46);
- the public check page (phase 6; this phase only puts the parts it will share in `web/shared`);
- the hostile suite's live layer and the gate's new cases in depth (phase 5);
- Task 3 of phase 3 (the agent's own file tools against links out of the copy), still deferred on the owner's word;
- a program listening on the Windows side (phase 5); until then the chooser keeps the narrower network line;
- sending reports back (milestone 3), Codex (phase 7), macOS.

---

## 3. Design

### 3.1 The parts

```
web/app (Svelte screens) ──► lib/backend.ts (one interface)
     │                          ├─ lib/tauri.ts : the real app (Tauri commands and events)
     │                          └─ lib/fake.ts  : screen tests only; replays fixtures saved from real Rust output
web/shared (FileText, Findings, Outcome, tokens.css, generated types) ◄── also for the check page (phase 6)

crates/steplet-app (Tauri shell, new)
  ├─ open.rs     : read at most 256 KiB + 1 byte, steplet_core::report::check, pins, host list → PreviewView
  ├─ pins.rs     : known senders, %LOCALAPPDATA%\Steplet\senders.json (written only when Run starts)
  ├─ key.rs      : the Anthropic key in Windows Credential Manager (smaller choice S3)
  ├─ register.rs : "Open .steplet files with Steplet" for this Windows user only (J4)
  ├─ setup.rs    : glue to steplet_run::setup
  ├─ session.rs  : one run on a thread: run_live, questions and answers, Stop, closing, leftovers
  ├─ save.rs     : the report as text, and Windows' Save dialog
  └─ views.rs    : everything the screens get, file text already in display segments
crates/steplet-run (phase 3, extended): live.rs, ask.rs, lock.rs, task.rs, promises.rs, setup.rs (+ setup.sh, look.sh)
crates/steplet-core (phase 1–2, extended): policy.rs (J1), identity.rs (the sender-key rule)
```

### 3.2 How a run reaches the screen

The app starts `steplet_run::run_live` on its own thread. `run_live` is phase 3's `run` with a `Live` value added: a function that receives each piece of progress, a `StopSignal`, the receiver's `Answers`, and the ask limit. Each event passes through the report's own `Scrubber` before the screen sees it (control and invisible characters removed, the key and the run's token hidden), so the live log is exactly as safe to read as the report. Phase 3's `run` stays, as `run_live` with no one to ask, and keeps every phase 3 test meaning what it meant.

### 3.3 The question path (R7)

The gate service already calls a decision function on a worker thread with Steplet's deadline (phase 3 Task 8). For a live run that function is `Asker::decide`: the checker's Allow and Deny go back at once; an Ask becomes a `Question` with an id and the fingerprint of the exact request (the tool name and the request's JSON, keys in sorted order, which `serde_json` already gives). The question goes to the screen; the function waits for an answer, at most the ask limit, and stops waiting at once if the run is stopping. Only `Yes` carrying the same fingerprint as the question allows it, and the allow sends back exactly the request that was asked (`updated_input` is the request's own input). A changed request has another fingerprint, so it asks again (J2). The gate's deadline is the ask limit plus 15 s, so the gate never gives up before the receiver has had their time.

### 3.4 What changes in the runner and the core, and why

| Change | Why |
|---|---|
| The checker refuses file-tool writes outside the working copy, without a question (J1, core) | Phase 3's open point: once the receiver can say yes, the agent's Write tool, which runs outside the fence, could write into its own config folder (a planted settings file would load hooks) or into `/tmp` for the next run. Nothing of the receiver's is outside the copy, so no task needs that. |
| `Edit(//var/lib/steplet/**)` joins the run's deny rules | The rule phase 3 asked for: Claude Code refuses such a write itself, before any question; a second lock behind J1. Claude Code's documentation says Edit deny rules also become part of the fence's write limits, so Task 1 shows the fence's own temporary folder still works. |
| `prepare` returns what it removed | The prepare screen lists the agent settings files the repository carried (D3). |
| `task_text` builds what the agent is told | Phase 3's tests wrote their own; the app needs one fixed form, which also lists the allowed commands word for word so the agent's commands match them more often (G1). |
| `run_live`, `Progress`, `StopSignal`, `RunLock` | Live log, Stop, and one run at a time (`prepare` empties `/work`, so a second run would delete the first one's copy). |
| `Asker`, `Question`, `Reply`, `Answers` | The ask screen (R7). |
| `RunRequest.allow_model_switch` | E1: after a model refusal the report offers "Run again, allowing another model"; only then does the run's settings file say `switchModelsOnFlag: true`. |
| The report gains `stopped_by_you`, `answers`, `Check::AllowedByYou`; faults in words; a correct label for a call turned away before a decision | What the report screen must say (three of phase 3's deferred points are about these words). |
| `promises(s)` | The chooser's lines, each with the names of the tests that back it (R6). |
| `setup::{look, install, remove}` | Setup becomes product code; `tests/support/setup_steplet.sh` moves into `src/setup.sh`. |
| Raw logs on Windows hide the key and token as they are written (J6) | Phase 3's final review: those files were not cleaned. |
| `identity` and the rule `id.key-changed` (core) | Rows 16 and 22; the rule lives where every rule lives, but only the app applies it, since it needs this computer's pins (R10). |

### 3.5 Look and feel

Windows 11's own fonts (Segoe UI Variable for text, Cascadia Mono, then Consolas, for commands and output: nothing downloaded); a 4-pixel spacing scale; one column, at most 860 pixels wide, centred; panels with a quiet border; text from the file inside "From the file" panels with a left rule and the label, so it never looks like Steplet's own messages. Outcome colours come with a word and an icon (green ✓, amber !, red ✕, blocked ⛔ drawn as inline SVG, not emoji). Colours are tokens in `web/shared/tokens.css`, one set for light and one for dark (J8), each text pair checked at 4.5 : 1 by `web/scripts/contrast.mjs`. The app follows Windows' light or dark setting. Minimum window 560 × 720.

### 3.6 Checked before writing this plan (read-only, 18 Oct 2026)

```
> cargo generate-lockfile --offline        (a scratch crate needing tauri =2.12.1, tauri-build =2.7.1, serde with derive)
     Locking 406 packages to latest Rust 1.98.0 compatible versions
      Adding toml_edit v0.20.2 (available: v0.20.7)
> npm ci --offline --ignore-scripts --no-audit --no-fund     (spikes/s9-check's package.json and package-lock.json, in a scratch folder)
added 53 packages in 2s
npm exit: 0
> ls C:/Users/RANVIR/AppData/Local/ms-playwright/
b
chromium-1234
chromium-1243
chromium_headless_shell-1234
chromium_headless_shell-1243
ffmpeg-1011
firefox-1538
firefox-1543
webkit-2336
webkit-2359
winldd-1007
> crates.io API, ts-rs versions (version, date published, minimum Rust), the first three of eight lines
12.0.1 2026-01-31  1.78.0
12.0.0 2026-01-31  1.78.0
11.1.0 2025-10-14  1.78.0
> wsl.exe --list --verbose; wsl.exe --version   (the first line of the second command's three)
  NAME            STATE           VERSION
* Ubuntu          Stopped         2
  steplet         Running         2
  steplet-test    Stopped         2
WSL version: 2.6.3.0
```

So Tauri and every screen tool phase 0 used install with no download; `ts-rs` is not in the cache (J7). Claude Code's sandbox documentation (code.claude.com/docs/en/sandboxing, read 18 Oct 2026) says "Paths and domains from both sandbox settings and permission rules are merged into the final sandbox configuration", and that by default a fenced command may write to "The working directory, a per-user temp directory": Task 1 measures whether the new Edit rule touches that temp directory on 2.1.220 (the page describes the newest Claude Code, not necessarily 2.1.220).

### 3.7 Points in the earlier documents that are wrong or in conflict

Said plainly, so you can correct me:
1. **Red files.** The milestone plan's table says "The file is red: it cannot run, no override (row 44b, R9)". But R9 and `docs/rules.md` (approved in phase 2) say red runs **after the receiver types a confirmation sentence**; only blocked has no override. R-rows win, so this plan follows R9 unless you choose otherwise (J3).
2. **The network line.** Section 7's "cannot connect to the internet" is false for a file that declares web sites: the fence lets programs reach exactly those. The chooser's line must depend on the file, and no phase 3 test covers the declared-site case yet; Task 5 adds it.
3. **"On this computer".** The milestone plan allows that wording only after a test with a program listening on the Windows side, which phase 3 did not run. The chooser keeps "or to other programs in Steplet's Linux" until phase 5 adds that test.
4. **The setup promise.** H4's wording, "Steplet never asks for your password and never changes your own Windows or Linux", is broken by registering `.steplet` files for Steplet, which writes to the receiver's Windows settings. J4 keeps it true by making it a switch the receiver turns on.
5. **The prepare screen's "(2 hooks)".** Counting hooks means reading a stranger's settings file in one more place. This plan lists the removed files only (smaller choice S7).

---

## 4. Choices for you, each with a recommendation

### J1. Can the receiver let the agent's own file tools write outside the working copy?

The checker today says "ask" for a write outside the repository. Once the ask screen exists, a yes would let Claude Code's Write tool, which runs outside the fence, write anywhere the `steplet` account can, for example its own config folder (a planted settings file there could load hooks) or `/tmp` (which outlives the run).

| Option | What goes wrong |
|---|---|
| **A (recommended). Refuse without asking:** the checker answers "the agent's file tools may change files only inside the working copy". Reads outside still ask. The rule `Edit(//var/lib/steplet/**)` is added as well, so Claude Code itself refuses writes into Steplet's own folder. | A task whose agent wants to write a file outside its copy cannot. No milestone 1 use case needs it, and the programs a task runs already cannot (the fence). |
| B. Keep asking; add only the deny rule for Steplet's folder. | A yes can still put files in `/tmp` and other places the `steplet` account can write, which outlive the run and could affect the next one. You would be asked about paths that mean nothing to you. |

### J2. What a "yes" on the ask screen covers

| Option | What goes wrong |
|---|---|
| A. That one request only; the same request again asks again. | Agents often retry a command; you get asked the same question again, which teaches clicking yes without reading. |
| **B (recommended). That exact request, byte for byte, for the rest of this run;** never saved, never anything broader. The button says so. | A repeated command runs again without asking, even if its effect differs the second time (for example it deletes a folder that has new files in it). It is still exactly the command you approved, still inside the fence. |

### J3. Red files

| Option | What goes wrong |
|---|---|
| **A (recommended). Follow R9 and `docs/rules.md`:** red runs only after typing "I understand the risks and want to run this file"; the Rust side checks the sentence too. | A receiver persuaded by the sender can type it. |
| B. Red cannot run in milestone 1, as the milestone plan's table says. | Rules that only suspect a trick (hidden characters in prose, long encoded blocks) become hard blocks; R9 split them from blocked on purpose, because hard blocks on suspicion push people to copy the task out and run it with no checks at all. |

### J4. Opening `.steplet` files by double-click

| Option | What goes wrong |
|---|---|
| **A (recommended). A switch in Setup, off until you turn it on:** "Open .steplet files with Steplet", for this Windows user only (`HKEY_CURRENT_USER`), no administrator rights; turning it off removes the entries. The setup promise becomes "Steplet never asks for your password and never changes your own Windows or Linux, except what you switch on here." | One more click in Setup. The entry points at the program where it is now (this laptop's build folder); if the program moves, Setup shows the switch as broken and offers to fix it. |
| B. Register automatically at first start. | Steplet changes your Windows without asking, which the promise says it never does. |
| C. No registration in phase 4 (drag and drop and the Open button only). | Double-click does not work, and the phase's done-means asks for a run from double-click. |

### J5. The `steplet://` link scheme

| Option | What goes wrong |
|---|---|
| **A (recommended). Not in phase 4; registered in milestone 3 with the link page.** Nothing sends such links yet. | One line of the milestone plan's phase 4 list moves to milestone 3. |
| B. Register it now, with a screen saying links come later. | A handler that does nothing yet, but which any web page could start, is something to secure for no use. |

### J6. The raw logs Steplet keeps on Windows (`<run>.jsonl`, `<run>.stderr.txt`)

| Option | What goes wrong |
|---|---|
| **A (recommended). Hide the key and the run's token in each line as it is written;** everything else stays exactly as Claude Code wrote it. | A line holding a secret is no longer byte for byte what Claude Code wrote. |
| B. Keep them raw. | If a run ever printed the key, it would sit in a file you might send to someone for help. |
| C. Delete them once the report is built. | Nothing is left to find out what went wrong in a run. |

### J7. How the screens learn the shape of Steplet's data

| Option | What goes wrong |
|---|---|
| **A (recommended). Generate the TypeScript types from the Rust types with `ts-rs` 12.0.1** (published 31 Jan 2026), only in `steplet-app`. A test fails when the generated files are out of date. | A download (section 5, Z2): `ts-rs` and any crates it needs that are not cached; one more library to trust. |
| B. Write the TypeScript types by hand; a test makes Rust save sample data of every type, and the type checker checks those samples against the hand-written types. | A field that no sample happens to fill can drift unnoticed between Rust and the screens. |

### J8. Dark mode

| Option | What goes wrong |
|---|---|
| **A (recommended). Follow Windows' light or dark setting;** every screenshot is taken in both. | Twice the screenshots to look at and fix. |
| B. Light only in milestone 1. | A white window at night for receivers who use dark mode; adding it later means looking at every screen again. |

### J9. The repository for the use case 1 run

| Option | What goes wrong |
|---|---|
| **A (recommended). A tiny public repository on your GitHub,** `steplet-sample`: a small Python module, its `unittest` tests, a README; nothing to install. It can later also hold the use case 5 fixture (H6). | It is published under your name (an outward step, Z5). |
| B. An existing public repository, pinned by its commit. | We do not control it; it may need installs the fence blocks, and may vanish. |

### Smaller choices I made (tell me if you object)

- **S1. Ask limit: 5 minutes**, then the answer is no. The gate's deadline becomes 5 min 15 s for every decision, so a checker that stalls is refused after 5 min 15 s instead of phase 3's few seconds (Claude Code waits longer than 10 minutes, F36). The run's own time limit keeps running while a question waits.
- **S2. Run limits:** 30 minutes and 40 turns per run.
- **S3. The key is kept in Windows Credential Manager** (a "generic" entry `Steplet/anthropic-api-key`, locked to your Windows account), not in a file; Setup has "Forget key".
- **S4. Saved reports are plain `.txt`**, default name `steplet-report-YYYY-MM-DD-HHMM.txt`, never the file's title.
- **S5. A pin is written when Run is pressed,** not when a file is opened. After a red key-change run, the new key replaces the old one, and the red screen says so before Run.
- **S6. A file that is unsigned but uses a pinned sender name gets the same red finding** as a changed key (row 22's "same sender, different key", read as including "no key").
- **S7. The prepare screen lists the removed files, not their hook counts** (3.7 point 5).
- **S8. No pause between prepare and run.** The removed files cannot act any more; the list stays at the top of the run screen.
- **S9. A second file opens in a second window;** only running is limited to one at a time.
- **S10. The 14-day rule (F12) also covers `web/package-lock.json`,** checked by a small script like `lock_ages.py` against the npm registry.
- **S11. Links in file text are not clickable;** Steplet's own help links (Microsoft's WSL page, Anthropic's key page) open in the browser from a fixed list.
- **S12. Raw HTML is refused by a script** (`web/scripts/no-raw-html.mjs`), not by a linting tool that would be a download.
- **S13. Real-app screenshots go through CDP** from a test build (`--features e2e`) that opens WebView2's debugging port on 127.0.0.1 only; normal builds have no such port (a test checks). If Playwright cannot attach, the fallback is a window capture with PowerShell, recorded as such.

---

## 5. What needs your yes, and when

| # | When | What happens | Size | Undo |
|---|---|---|---|---|
| Z1 | Approving this plan | The commits below; Tauri's 406 crates and the 53 npm packages come from the caches (no download) | about 2 GB of build output in `target` | `cargo clean` |
| Z2 | Task 8, if J7 is A | Download `ts-rs` 12.0.1 and any crates it needs that are not cached. Cargo is asked first which those are; I stop, show you the list and their ages, and wait | a few small crates | remove the dependency |
| Z3 | Task 6 | A second Steplet Linux, `steplet-setup-test`, installed from the internet through the new setup code, to test first-time setup for real; removed at the end. Your `Ubuntu`, `steplet` and `steplet-test` are not touched | about 1 to 2 GB download and disk, for the length of the test | `wsl --unregister steplet-setup-test` |
| Z4 | Task 11 | The registry test writes and removes a made-up type `.steplet-test` for your Windows user; then you turn on the real switch (J4) yourself | a few registry entries | the switch, or the test's own removal |
| Z5 | Task 15 | Publishing `steplet-sample` on your GitHub (J9 A), and one real-model run of use case 1 with your test key (P5), which **you paste into Steplet's key box yourself** | a few cents | delete the repository; forget the key |
| Z6 | Any task | If npm wants a package not in its cache, I stop and ask | n/a | n/a |

---

## 6. How the parts connect: the exact names the tasks rely on

`steplet-core` (Task 1, Task 7):
```rust
// policy.rs: a Write outside the working copy (or not a plain absolute path) is
// Decision::Deny { rule: "cap.outside-path", reason: "the agent's file tools may change files only inside the working copy" };
// a write into an always-blocked place keeps "block.secret-path".
pub mod identity {
    pub enum Identity { Unsigned, Invalid, NewKey { key: String }, SameKey { key: String, runs: u32 },
                        KeyKnownAs { key: String, names: Vec<String> } }
    pub struct Pin { pub sender: String, pub key: String, pub runs: u32 }
    /// The identity line, and the red finding `id.key-changed` when `sender` is pinned to another key, or the file is
    /// unsigned under a pinned sender name.
    pub fn identity(sender: &str, sig: &SigStatus, pins: &[Pin]) -> (Identity, Option<Finding>);
}
// rules.rs: pub static KEY_CHANGED: Rule (id "id.key-changed", Red); pub const MACHINE_RULES: &[&Rule] = &[&KEY_CHANGED];
// KEY_CHANGED is not in RULES: no file alone can hit it.
```

`steplet-run` (Tasks 1 to 6):
```rust
pub struct Prepared { pub commit: String, pub removed: Vec<String> }        // prepare() -> Result<Prepared, PrepareError>
pub fn task_text(s: &Steplet, commit: &str) -> String;
pub struct StopSignal(/* Arc<AtomicBool> */);  impl StopSignal { pub fn new() -> Self; pub fn stop(&self); pub fn is_set(&self) -> bool }
pub enum Progress { Prepared(Prepared), Event(Event), Asking(Question), Answered { question: u64, reply: Reply } }
pub struct Question { pub id: u64, pub tool: String, pub what: String, pub input: serde_json::Value,
                      pub reason: String, pub fingerprint: String, pub asked_at: Instant }   // `what`, `input`: cleaned
pub enum Reply { Yes { fingerprint: String }, No, NoAnswer, Withdrawn, Reused }
#[derive(Clone, Default)] pub struct Answers(/* Arc<(Mutex<HashMap<u64, Reply>>, Condvar)> */);
impl Answers { pub fn give(&self, question: u64, reply: Reply) -> bool }   // false: no such open question
pub struct Live { pub on_progress: Box<dyn Fn(Progress) + Send + Sync>, pub stop: StopSignal,
                  pub answers: Option<Answers>, pub ask_limit: Duration }    // answers None: nobody to ask (phase 3)
impl Live { pub fn unattended() -> Live }
pub fn run_live(req: &RunRequest, paths: &RunPaths, policy: &Policy, setup: &RunSetup, live: Arc<Live>) -> Result<Report, RunError>;
pub fn run(req, paths, policy, setup) -> Result<Report, RunError>;          // = run_live(.., Arc::new(Live::unattended()))
/// One run's question path: the gate's decision function for a live run. Holds the run's approvals (fingerprints,
/// in memory only), its question numbers and its record of answers.
pub struct Asker { /* private */ }
impl Asker { pub fn new(live: Arc<Live>) -> Asker;
             pub fn decide(&self, policy: &Policy, req: &GateRequest) -> GateAnswer;   // blocks at most live.ask_limit
             pub fn answers(&self) -> Vec<Answer> }
pub fn fingerprint(req: &GateRequest) -> String;                           // sha256 hex of tool name, 0x00, input JSON
// RunRequest gains `pub allow_model_switch: bool` (E1); settings_json(s, paths, gate, allow_model_switch: bool).
pub struct RunLock(/* named mutex Local\Steplet-run */);  impl RunLock { pub fn take() -> Result<RunLock, RunError> }
// RunError gains Busy (another run holds the lock) and Limits (ask limit + 15 s above the gate deadline).
// Report gains: pub stopped_by_you: bool, pub answers: Vec<Answer>; Check gains AllowedByYou { reused: bool };
pub struct Answer { pub tool: String, pub what: String, pub reply: Reply }
pub struct Promise { pub text: &'static str, pub items: Vec<String>, pub tests: &'static [&'static str] }
pub fn promises(s: &Steplet) -> Vec<Promise>;
pub mod setup {
    pub const LINUX: &str = "steplet"; pub const CLAUDE_VERSION: &str = "2.1.220"; pub const SRT_VERSION: &str = "0.0.77";
    pub enum Wsl { Missing, TooOld { version: String }, Ready { version: String } }
    pub struct State { pub wsl: Wsl, pub linux: Option<String>, pub missing: Vec<String>, pub gate: bool }
    pub fn look(name: &str, gate_program: &[u8]) -> State;
    pub fn install(name: &str, location: &Path, gate_program: &[u8], line: &dyn Fn(&str)) -> Result<String, SetupError>;
    pub fn remove(name: &str) -> Result<(), SetupError>;
}
```

`steplet-app` views (Task 9; every `Shown` is file, repository, agent or command text):
```rust
pub type Shown = Vec<Seg>;
pub enum Seg { Text { text: String }, Invisible { codepoint: u32 }, Link { shown: Option<String>, real: String } }
pub enum RunStatus { Blocked, Red, Amber, Green, NotRunnable { reason: String } }
pub struct PreviewView { pub sha256: String, pub outcome: String, pub run_status: RunStatus, pub identity: IdentityView,
    pub findings: Vec<FindingView>, pub here: Vec<FindingView>, pub file: Option<FileView>, pub counts: Counts }
pub struct FindingView { pub rule: String, pub severity: String, pub line: Option<u32>, pub title: String,
    pub detail: Option<Shown>, pub advice: String, pub quote: Option<Shown> }
pub struct FileView { pub title: Shown, pub sender: Shown, pub expires: String, pub repository: Option<(Shown, Shown)>,
    pub steps: Vec<Shown>, pub commands: Vec<Shown>, pub domains: Vec<Shown>, pub read: Vec<Shown>, pub write: Vec<Shown>,
    pub never: Vec<Shown>, pub body: Shown }
pub struct ChooserView { pub claude: RunnerView, pub codex_reason: String }
pub struct RunnerView { pub ready: bool, pub promises: Vec<PromiseView>, pub data_leaves: String }
pub struct SetupView { pub wsl: Line, pub linux: Line, pub tools: Line, pub gate: Line, pub key_saved: bool,
    pub opens_steplet_files: Switch, pub ready: bool }
pub enum ProgressView { Prepared { commit: String, removed: Vec<Shown> }, Event(EventView), Question(QuestionView),
    QuestionClosed { id: u64, how: String }, Finished(ReportView), Failed { stage: String, reason: String } }
pub struct ReportView { /* each Report field; every text a Shown; `answers`; `stopped_by_you` */ }
```
Tauri commands (the screens call nothing else): `initial_preview() -> Option<PreviewView>`, `open_path(path) -> PreviewView`, `open_bytes(bytes) -> PreviewView`, `chooser() -> ChooserView`, `setup_status() -> SetupView`, `setup_install()` (events `setup-line`), `save_key(key) -> SetupView`, `forget_key() -> SetupView`, `set_opens_steplet_files(on) -> SetupView`, `start_run(sha256, typed: Option<String>)` (events `run-progress`), `answer(question, yes, fingerprint) -> bool`, `stop_run()`, `save_report() -> Option<String>`, `open_help(topic)`.

---

## 7. Tasks

Order: 1 → 16. Tasks 1 to 7 change the runner and the core and need no screen; Tasks 8 to 13 build the app; Task 14 screenshots every state; Task 15 is the real run; Task 16 wraps up. Linux-only tests are `#[ignore]` (they need Steplet's Linux) and run with `-- --ignored --test-threads=1`. Each task ends with its output pasted under its heading in `docs/milestone-1-phase-4-results.md` (created by Task 1).

### Task 1: the agent's file tools cannot write outside the working copy (J1), and Steplet's own folder is denied

**Files:** modify `crates/steplet-core/src/policy.rs`, `crates/steplet-core/tests/policy.rs`, `crates/steplet-run/src/settings.rs`, `crates/steplet-run/tests/settings.rs`, `crates/steplet-run/tests/contain.rs`, `docs/rules.md` (the reason line under `cap.outside-path`); create `docs/milestone-1-phase-4-results.md`, `crates/steplet-run/tests/support/writable.sh`.

**Interfaces:** Produces the policy change and the fourth deny rule of section 6. Consumes `RunPaths`, `settings_json` (phase 3).

- [ ] **Step 1: record where the `steplet` account can write outside the copy.** `writable.sh` runs as `steplet` and lists every writable folder on every mounted file system except `/proc`, `/sys` and `/dev/pts`. Run it through `wsl::run(distro, "steplet", ["/bin/bash", "-s"], script)` from a small `#[ignore]` test `where_steplet_can_write` in `tests/contain.rs`, and paste the list. Expected to include `/tmp`, `/var/tmp`, `/dev/shm`, `/run/lock`; anything under `/var/lib/steplet` appears only while a run exists.
- [ ] **Step 2: write the failing tests.**

In `crates/steplet-core/tests/policy.rs`, new test:
```rust
#[test]
fn a_file_tool_write_outside_the_working_copy_is_refused_without_a_question() {
    let p = policy("  write:\n    - target\n");
    let outside = Decision::Deny { rule: "cap.outside-path", reason: "the agent's file tools may change files only inside the working copy" };
    assert_eq!(p.check(&Action::Write("/tmp/x")), outside);
    assert_eq!(p.check(&Action::Write("/var/lib/steplet/runs/r1/config/settings.json")), outside);
    assert_eq!(p.check(&Action::Write("src/main.rs")), outside, "not a plain absolute path, so not known to be inside");
    assert!(denies(p.check(&Action::Write("/home/tester/.ssh/authorized_keys")), "block.secret-path"));
    assert!(asks(p.check(&Action::Read("/etc/hostname"))), "reading outside still asks");
    assert!(asks(p.check(&Action::Write(&format!("{WORK}/src/main.rs")))), "inside, undeclared: still asks");
}
```
In `crates/steplet-run/tests/settings.rs`, the existing exact-JSON test's `permissions.deny` becomes `["Read(//mnt/**)", "Read(//var/lib/steplet/**)", "Edit(//work/<id>/repo/.git/**)", "Edit(//var/lib/steplet/**)"]`.
In `crates/steplet-run/tests/contain.rs`, new `#[ignore]` test `a_write_into_steplets_own_folder_is_refused_before_the_gate`: scripted model makes three calls: `Write` to `<run_config>/settings.json`, `Write` to `/tmp/steplet-probe.txt`, then `Bash` `python3 -c "import tempfile; print(tempfile.mkstemp()[1])"` (declared in the file). Assert from `RunOutcome`/`Report`, never the agent's words: the first has **no** gate request and an error result (Claude Code's own deny rule); the second has a gate request answered `Deny` with the J1 reason; neither file exists afterwards (checked as root before `discard`); the third is `Allowed` with exit code 0 (the fence's temporary folder still works).
- [ ] **Step 3: run them, see them fail.**
  `cargo test -p steplet-core --offline --test policy` — Expected: FAIL, `left: Ask { reason: "this path is outside the repository" }`.
  `cargo test -p steplet-run --offline --test settings` — Expected: FAIL on the deny list.
- [ ] **Step 4: write the change.** In `Policy::path`, a write that is not a plain absolute path, or is outside the work folder and not a secret place, is `deny(rules::CAP_OUTSIDE_PATH, "the agent's file tools may change files only inside the working copy")`; reads unchanged. In `settings_json`, add `rule("Edit", STATE_ROOT)`.
- [ ] **Step 5: run, see them pass, and nothing else move.**
  `cargo test -p steplet-core --offline` — Expected: every test passes, including `corpus` (`every_file_gives_its_expected_json` unchanged: the rating does not use this path).
  `cargo test -p steplet-run --offline --test contain -- --ignored --test-threads=1` — Expected: `test result: ok. 5 passed; 0 failed` (phase 3's three, plus Steps 1 and 2).
  **If the tempfile command fails** (the merged rule took the fence's temporary folder away): stop, record the output, and bring it to the owner; J1's checker change alone still closes the open point.
- [ ] **Step 6: commit.** `git commit -m "The agent's file tools cannot write outside the working copy, and Claude Code itself refuses writes into Steplet's own folder"`

### Task 2: what the agent is told, and what preparing removed

**Files:** create `crates/steplet-run/src/task.rs`, `crates/steplet-run/tests/task.rs`; modify `src/prepare.rs`, `src/prepare.sh`, `src/run.rs`, `src/lib.rs`, `tests/prepare.rs`.

**Interfaces:** Produces `task_text(s, commit) -> String` and `Prepared { commit, removed }` (`prepare` now returns `Result<Prepared, PrepareError>`).

- [ ] **Step 1: write the failing tests.** `tests/task.rs`, `task_text_is_steplets_fixed_form` on `examples/run-tests.steplet` with commit `7fd1a60b01f91b314f59955a4e4d4e80d8edf11d`, asserting the whole text equals:
```
You are doing a task that someone sent as a .steplet file. Steplet, the app running you, checks every action before it happens.

The repository is checked out in the current folder at commit 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d.

Steps:
1. Check that the README file is there.
2. Count the lines in the README file.
3. Report the count.

Commands the file allows, exactly as written (any other command that changes something needs the receiver's yes):
- wc -l README

Web sites the file allows: none.

Never:
- Do not change any file.
- Do not use the network.

The sender's notes:
<the file's body, exactly>
```
  plus `task_text_says_when_nothing_is_allowed` ("The file allows no commands…") and `task_text_keeps_the_body_exactly`. In `tests/prepare.rs`: `#[ignore]` `prepare_reports_the_commit_and_no_removed_files_for_hello_world` (`removed == []`, `commit` equals the declared one), and `#[ignore]` `the_strip_step_lists_what_it_removed`: plant `.claude/settings.json`, `.mcp.json` and `.codex/config.toml` in a scratch copy as root, run `prepare.sh strip <copy>` on it, assert `removed == [".claude", ".codex", ".mcp.json"]` and that they are gone.
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-run --offline --test task` — Expected: FAIL, `unresolved import` naming `task_text`.
- [ ] **Step 3: write `task.rs`; give `prepare.sh` a second entry, `strip <copy>`, which the prepare entry also calls, and which prints one `removed: <name>` line for each agent file it removes, in the fixed order `.claude .codex .cursor .gemini .mcp.json`; `prepare` returns `Prepared`.** `run` passes `Prepared` on (Task 3 shows it).
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-run --offline --test task` → `test result: ok. 3 passed`; `cargo test -p steplet-run --offline --test prepare -- --ignored --test-threads=1` → every test passes.
- [ ] **Step 5: commit.** `git commit -m "The runner says what the agent is told and which agent settings files it removed"`

### Task 3: live runs: progress as it happens, cleaned; Stop; one run at a time; raw logs

**Files:** create `crates/steplet-run/src/live.rs`, `src/lock.rs`, `tests/lock.rs`; modify `src/run.rs`, `src/report.rs` (the `Scrubber` gets `pub(crate) fn event(&self, e: &Event) -> Event`; `Report.stopped_by_you`), `src/launch.rs` (J6 A), `src/lib.rs`, `tests/watch_run.rs`, `tests/report.rs`.

**Interfaces:** Produces `Live`, `Progress`, `StopSignal`, `run_live`, `RunLock`, `RunError::Busy` (section 6). `watch_run` gains `live: &Live`; phase 3's tests pass `&Live::unattended()`. `run_live` takes `Arc<Live>` because the gate's worker thread (Task 4) needs its own handle.

- [ ] **Step 1: write the failing tests.** In `tests/watch_run.rs` (stand-in agents, no Linux):
  - `each_event_reaches_the_screen_cleaned_and_in_order`: the stand-in sends `AgentText("\u{1b}[2Kdone support::MADE_UP_KEY")`, a `ToolUse`, its `ToolResult`; the collected `Progress::Event`s are the same three in order, with the escape gone and the key shown as `[hidden by Steplet]`, and equal to the report's own cleaning of the same text.
  - `stop_signal_stops_the_run_and_the_report_says_you_stopped_it`: the stand-in runs until stopped; `StopSignal::stop()` after the first event; `stop()` was called once, `outcome.stopped_by == None`, the report has `stopped_by_you: true` and `completed: false`.
  In `tests/lock.rs`: `a_second_run_cannot_start_while_one_is_running` (`RunLock::take()` twice: the second is `Err(RunError::Busy)`; after dropping the first, `take()` works again).
  In `tests/report.rs`: `raw_log_lines_hide_the_key_and_the_token` (the log writer given the made-up key and a token writes `[hidden by Steplet]` in their place and leaves every other byte, escapes included, as it was).
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-run --offline --test watch_run --test lock --test report` — Expected: FAIL, `unresolved imports` naming `Live` and `RunLock`.
- [ ] **Step 3: write `live.rs` and `lock.rs`; join them into `run_live`** (take the lock first; after `prepare`, `on_progress(Prepared)`; each judged event and each `SkippedGate` record goes to `on_progress` after `Scrubber::event`; `watch_run` checks `live.stop` on every turn of its loop and stops as at a tripwire, but records `stopped_by_you` instead of a tripwire). The lock is the named mutex `Local\Steplet-run` (`CreateMutexW`, feature `Win32_System_Threading`, already on).
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-run --offline` — Expected: every summary line `ok`, `0 failed`. `cargo test -p steplet-run --offline -- --ignored --test-threads=1` — Expected: every Linux-only test still passes (phase 3's 22 plus Tasks 1 and 2's), `0 failed` on every line.
- [ ] **Step 5: commit.** `git commit -m "Live runs: each event reaches the screen cleaned like the report, Stop works, and only one run at a time"`

### Task 4: the ask path (R7)

**Files:** create `crates/steplet-run/src/ask.rs`, `tests/ask.rs`, `tests/ask_live.rs`; modify `src/live.rs`, `src/run.rs`, `src/report.rs`, `src/watch.rs` (only if Step 1 requires it), `src/lib.rs`.

**Interfaces:** Produces `Asker`, `Question`, `Reply`, `Answers`, `fingerprint`, `Check::AllowedByYou`, `Report.answers`, `RunError::Limits`. Consumes `Policy::check` and `decide` (phase 3; it stays for unattended runs). `run_live` passes `move |r| asker.decide(&policy, r)` to `serve` when `live.answers` is set, and phase 3's `decide` otherwise.

- [ ] **Step 1: measure (record only).** `#[ignore]` test `does_claude_code_ask_again_after_a_yes` in `tests/ask_live.rs`: a stand-in that answers every question yes; the scripted model makes the same undeclared `Bash`, `Write` (inside the copy, not in `allow: write`), `Read` (of `/etc/hostname`) and `WebFetch` twice each. Record for each whether the second one reached the gate. **If any second call skipped the gate,** the watcher (H11) must treat it as allowed only when the receiver said yes to that exact fingerprint in this run; Step 2 then also gets `a_repeat_of_an_approved_read_does_not_stop_the_run`.
- [ ] **Step 2: write the failing tests** (`tests/ask.rs`, no Linux; a policy from a small file; `Asker::decide` driven directly with a 200 ms ask limit, the screen played by a thread that reads `Progress::Asking` and calls `Answers::give`):
```rust
#[test] fn a_yes_allows_exactly_the_request_shown()          // Yes{same fingerprint} → GateAnswer::Allow { updated_input } == the request's input
#[test] fn a_yes_with_another_fingerprint_is_a_no()          // → Deny("the answer did not match the request on screen, so Steplet refused it")
#[test] fn no_answer_within_the_limit_is_a_no()              // → Deny("no answer in time, so Steplet refused it"); returned in under 1 s
#[test] fn a_stop_withdraws_the_open_question_at_once()      // stop after 100 ms → Deny within 500 ms; Progress::Answered{reply: Withdrawn}
#[test] fn the_same_exact_request_later_in_the_run_is_allowed_without_asking_again()   // J2 B: no second Progress::Asking; Reply::Reused recorded
#[test] fn a_request_that_differs_by_one_character_asks_again()
#[test] fn the_checkers_allow_and_deny_never_ask()            // no Progress::Asking at all
#[test] fn the_question_shows_the_request_cleaned()           // an ESC and support::MADE_UP_KEY in the command: neither in `what` nor `input`
#[test] fn two_questions_at_once_each_get_their_own_answer()  // two threads; answers given in reverse order; each gets its own
#[test] fn run_live_refuses_an_ask_limit_the_gate_deadline_cannot_cover()  // ask 300 s, deadline 310 s → Err(RunError::Limits)
```
  In `tests/report.rs`: `answers_and_allowed_by_you_show_in_the_report` (a yes, a reused yes, a no and no answer, each in `answers` with its reply; the yes'd call's `check` is `AllowedByYou { reused: false }`, the reused one `AllowedByYou { reused: true }`).
  In `tests/ask_live.rs`, `#[ignore]`: `a_yes_on_the_ask_screen_lets_exactly_that_command_run` (undeclared `touch asked.txt`; the stand-in says yes; `changed_files` holds `?? asked.txt`; the call is `AllowedByYou`) and `no_answer_refuses_it` (stand-in never answers, 2 s limit; `asked.txt` absent; the report lists `NoAnswer`).
- [ ] **Step 3: run, see them fail.** `cargo test -p steplet-run --offline --test ask` — Expected: FAIL, `unresolved import` naming `Asker`.
- [ ] **Step 4: write `ask.rs`** (section 3.3). Approvals live in the `Asker`'s own set of fingerprints, never on disk. Every question and answer also goes to `RunOutcome.answers` (from `Asker::answers`) so the report is built from Steplet's records.
- [ ] **Step 5: run, see them pass.** `cargo test -p steplet-run --offline --test ask --test report` → `ok`; `cargo test -p steplet-run --offline --test ask_live -- --ignored --test-threads=1` → `ok. 3 passed` (or 4, with Step 1's extra test).
- [ ] **Step 6: commit.** `git commit -m "The ask path: the receiver's yes allows exactly the request on screen, and no answer is a no"`

### Task 5: the chooser's lines, each tied to its test

**Files:** create `crates/steplet-run/src/promises.rs`, `tests/promises.rs`; modify `tests/contain.rs`, `src/lib.rs`.

**Interfaces:** Produces `Promise` and `promises(s: &Steplet) -> Vec<Promise>`.

The lines (text exact; `{sites}` are the file's domains, shown as file text):

| # | Text | Backed by (test function names) |
|---|---|---|
| 1 | "Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen. If Steplet's checker fails, the action is refused." | `a_declared_command_runs_and_what_the_file_did_not_ask_for_does_not`, `a_gate_that_dies_mid_request_refuses_it_and_the_restarted_gate_answers_the_next`, `a_decision_that_never_comes_is_refused_at_the_deadline_in_a_real_run`, `a_writing_command_made_to_skip_the_gate_stops_the_run` |
| 2a | "The agent cannot fetch any web address." (no sites) / 2b "The agent can fetch only these web addresses: {sites}" | `a_web_fetch_is_allowed_only_to_a_site_the_file_declares` |
| 3 | "The agent and the programs this task runs can change files only inside the working copy." | `every_try_of_the_hostile_fixture_is_contained`, `a_write_into_steplets_own_folder_is_refused_before_the_gate` |
| 4 | "Your own files are not inside Steplet's Linux, where this task runs." | `use_case_1_runs_with_none_of_the_receiver_s_files_present` |
| 5a | "Programs this task runs cannot connect to the internet, or to other programs in Steplet's Linux." (no sites) / 5b "Programs this task runs can connect only to {sites}: not to the rest of the internet, or to other programs in Steplet's Linux." | `every_try_of_the_hostile_fixture_is_contained`, and for 5b also `a_declared_site_is_the_only_one_a_program_can_reach` |
| 6 | "Programs this task runs cannot start Windows programs." | `every_try_of_the_hostile_fixture_is_contained` |

- [ ] **Step 1: write the failing tests.** `tests/promises.rs`: `every_promise_names_tests_that_exist` (reads every `crates/steplet-run/tests/*.rs` as text and finds `fn <name>(` for each name), `the_lines_follow_the_files_sites` (no domains → 2a and 5a; `example.org` declared → 2b and 5b with `items == ["example.org"]`), `no_line_says_on_this_computer` (3.7 point 3). `tests/contain.rs`, `#[ignore]`: `a_declared_site_is_the_only_one_a_program_can_reach` (file declares `example.org`; a declared command `python3 probe.py` inside the fence reaches `https://example.org` and fails to reach `https://example.net`; judged from the probe's own output file, with a control from outside the fence that reaches both).
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-run --offline --test promises` — Expected: FAIL, `unresolved import` naming `promises`.
- [ ] **Step 3: write `promises.rs`.**
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-run --offline --test promises` — Expected: `test result: ok. 3 passed; 0 failed`. `cargo test -p steplet-run --offline --test contain a_declared_site_is_the_only_one_a_program_can_reach -- --ignored` — Expected: `test result: ok. 1 passed; 0 failed`.
- [ ] **Step 5: commit.** `git commit -m "The run chooser's lines, each tied to the tests that back it, and a test for a declared site"`

### Task 6: setup as product code

**Files:** create `crates/steplet-run/src/setup.rs`, `src/setup.sh` (moved from `tests/support/setup_steplet.sh`, which is deleted; `tests/support/mod.rs` uses the product script), `src/look.sh`, `tests/setup.rs`; modify `src/lib.rs`.

**Interfaces:** Produces `setup::{look, install, remove, State, Wsl, LINUX, CLAUDE_VERSION, SRT_VERSION}`. `install` runs `wsl.exe --install Ubuntu-26.04 --name <name> --location <location> --no-launch` (H3 A), then `setup.sh` as root, then `wsl.exe --terminate <name>` (so `wsl.conf` applies), then `install_gate`. `look.sh` prints one line per check (`ok <what> <version>` or `missing <what>`); `look` never changes anything.

- [ ] **Step 1: write the failing tests.** `reads_wsl_version_output` (pure: this laptop's `wsl --version` text, the same text in UTF-16 as `wsl.exe` writes it without `WSL_UTF8`, and the "not installed" text, give `Ready { version: "2.6.3.0" }`, the same, and `Missing`); `#[ignore]` `look_finds_steplets_linux_ready` (on `steplet`: `missing == []`, `gate == true` after `install_gate`); `#[ignore]` **(Z3)** `install_makes_a_ready_linux_from_nothing` (`install("steplet-setup-test", %LOCALAPPDATA%\Steplet\wsl-setup-test, gate, …)`, then `look` gives `missing == []`; then `remove`, and `wsl --list` no longer shows it). Record how long the install took.
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-run --offline --test setup` — Expected: FAIL, `unresolved import` naming `setup`.
- [ ] **Step 3: write `setup.rs`, `setup.sh`, `look.sh`.**
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-run --offline --test setup` — Expected: `test result: ok. 1 passed; 0 failed; 2 ignored`. `cargo test -p steplet-run --offline --test setup look_finds_steplets_linux_ready -- --ignored` — Expected: `ok. 1 passed`. Then, with Z3: `cargo test -p steplet-run --offline --test setup install_makes_a_ready_linux_from_nothing -- --ignored --nocapture` — Expected: `ok. 1 passed`; paste the install's log lines and its time.
- [ ] **Step 5: commit.** `git commit -m "Setup of Steplet's Linux is product code, and can be checked without changing anything"`

### Task 7: sender pins in the core: the identity line and the key-changed rule

**Files:** create `crates/steplet-core/src/identity.rs`, `crates/steplet-core/tests/identity.rs`; modify `src/rules.rs` (`KEY_CHANGED`, `MACHINE_RULES`), `src/lib.rs`, `docs/rules.md` (new section "Rules about this computer, applied by the app only").

**Interfaces:** Produces `identity(sender, sig, pins) -> (Identity, Option<Finding>)`, `Pin`, `KEY_CHANGED` (id `id.key-changed`, red, title "This sender name was used before with another key", advice "Someone may be using this name. Ask the sender, another way, whether they changed their key. If they did, type the sentence to run it; Steplet then remembers the new key.").

- [ ] **Step 1: write the failing tests:** `a_known_sender_with_another_key_is_red` (hit), `a_known_sender_unsigned_is_red` (S6), `the_same_key_gives_the_same_key_line_and_no_finding` (near-miss; `SameKey { runs: 2 }`), `a_new_sender_is_new_key`, `a_known_key_under_another_name_is_said_but_not_a_finding`, `an_invalid_signature_gives_no_second_finding` (`sig.invalid` already blocks), `the_machine_rule_is_not_in_rules` (the corpus test's "every rule has a hit" stays true).
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-core --offline --test identity` — Expected: FAIL, `unresolved import` naming `identity`.
- [ ] **Step 3: write `identity.rs`.**
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-core --offline` — Expected: `0 failed` on every line, `corpus` included (the corpus is unchanged). `cargo build -p steplet-wasm --target wasm32-unknown-unknown --release --offline` — Expected: `Finished` (the check page's build still works).
- [ ] **Step 5: commit.** `git commit -m "The core says who signed a file as far as this computer knows, and warns when a known sender's key changes"`

### Task 8: the app's shell

**Files:** create `crates/steplet-app/{Cargo.toml, build.rs, tauri.conf.json, capabilities/main.json, icons/, src/main.rs, src/lib.rs, tests/shell.rs}`; `web/{package.json, package-lock.json, vite.config.ts, svelte.config.js, tsconfig.json, playwright.config.ts, index.html, app/main.ts, app/App.svelte, app/tests/real-open.spec.ts, shared/tokens.css, scripts/no-raw-html.mjs, scripts/contrast.mjs, scripts/npm_ages.py, scripts/make_icon.py}`; modify `Cargo.toml` (members), `.gitignore` (`web/node_modules`, `web/dist`, `web/dist-screens`).

**Interfaces:** `steplet-app` depends on `tauri = { version = "=2.12.1", features = ["custom-protocol"] }`, `tauri-build = "=2.7.1"`, `serde` (derive), `serde_json = "=1.0.151"`, `windows-sys = "=0.61.2"` (adds `Win32_Security_Credentials`, `Win32_System_Registry`, `Win32_UI_Controls_Dialogs`, `Win32_UI_Shell`), `steplet-core`, `steplet-run`, and with J7 A `ts-rs = "=12.0.1"`; feature `e2e` (S13). `web/package.json` has exactly the S9 versions (3.6) and the scripts `check` (`svelte-check` + `node scripts/no-raw-html.mjs` + `node scripts/contrast.mjs`), `build` (`vite build --mode app`), `build:screens` (`vite build --mode screens`), `test` (`playwright test`).

- [ ] **Step 1: install from the caches.** `npm --prefix web ci --offline` → Expected `added 53 packages`. `cargo build -p steplet-app --offline` with an empty `App.svelte` → Expected: `Finished`. **Z2:** with J7 A, first `cargo add ts-rs@=12.0.1 --offline -p steplet-app`; if cargo says the crate is not cached, stop, list what it needs with each crate's age, and wait for the owner. Then `python -I …\lock_ages.py Cargo.lock` → Expected `… 0 under 14 days old; 0 yanked` (if not: `cargo update --offline -p <crate> --precise <older cached version>`, and record it).
- [ ] **Step 2: write the failing tests.** `tests/shell.rs`, `the_window_and_security_settings_are_fixed`, reading `tauri.conf.json`: `productName` and the one window's `title` are `Steplet`; `minWidth` 560, `minHeight` 720; `dragDropEnabled` false (so drops reach the page as files); `withGlobalTauri` false; `app.security.csp` is exactly `default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'`; `plugins` is empty; `capabilities/main.json` grants only `core:event:allow-listen`, `core:event:allow-unlisten` and the app's own commands; `no_debug_port_without_the_e2e_feature` (built without `e2e`, the window's extra browser arguments hold no `remote-debugging`). `web/scripts/no-raw-html.mjs` exits 1 on a planted `{@html x}` in a temporary file (its own self-test) and 0 on the tree. `web/scripts/contrast.mjs` exits 1 when a token pair is below 4.5 : 1.
- [ ] **Step 3: run, see them fail.** `cargo test -p steplet-app --offline --test shell` — Expected: FAIL, the config does not have these values yet. `node web\scripts\no-raw-html.mjs --self-test` — Expected: `self-test passed` (the script works before the screens exist).
- [ ] **Step 4: write the shell.** `src/lib.rs` makes the window from its config entry (`"create": false` in the config, then `WebviewWindowBuilder::from_config`), refuses every navigation away from the app's own address (`on_navigation`) and every new window, and never calls `set_title`; `App.svelte` shows the empty Open screen.
- [ ] **Step 5: measure S13.** `cargo build -p steplet-app --offline --features e2e`, start `target\debug\steplet.exe`, then `npm --prefix web test -- real-open` (a one-test file that runs `chromium.connectOverCDP("http://127.0.0.1:9333")` and saves `docs/screens/phase-4/real-open-empty-light.png`). Paste the result; if it cannot attach, record why and use the PowerShell window capture for every later "real" screenshot.
- [ ] **Step 6: run, see them pass.** `cargo test -p steplet-app --offline` — Expected: `0 failed`. `npm --prefix web run check` — Expected: `0 ERRORS 0 WARNINGS`, `no raw HTML`, `every colour pair at least 4.5:1`. `python -I web\scripts\npm_ages.py web\package-lock.json` — Expected: `0 under 14 days old`.
- [ ] **Step 7: commit.** `git commit -m "The desktop app's shell: fixed title, locked-down page rules, and checks against raw HTML and low contrast"`

### Task 9: what the screens get: view types, the one backend interface, and fixtures from real Rust output

**Files:** create `crates/steplet-app/src/views.rs`, `tests/views.rs`, `tests/fixtures.rs`; `web/shared/types/` (generated, J7 A); `web/app/lib/{backend.ts, tauri.ts, fake.ts, scenarios.ts}`; `web/app/tests/fixtures/*.json` (written by `tests/fixtures.rs`).

**Interfaces:** Produces the views of section 6 and `interface Backend` in `backend.ts`, one method per Tauri command plus `onRunProgress(cb)` and `onSetupLine(cb)`. `fake.ts` reads `?scenario=<name>` and replays `fixtures/<name>.json` with the timings in `scenarios.ts`; `vite.config.ts` puts `fake.ts` only in the `screens` build.

- [ ] **Step 1: write the failing tests.** `tests/views.rs`: `the_preview_agrees_with_the_core` (for every corpus file: `outcome` and the findings' rule ids and lines equal `steplet_core::report::check`'s); `file_text_reaches_the_screen_only_as_segments` (a step holding U+200B gives `Seg::Invisible { codepoint: 0x200b }`); `a_newline_inside_one_entry_shows_as_a_badge` (a changed-file entry `"?? a\nb"` gives `Text("?? a")`, `Invisible { 0x0a }`, `Text("b")`: phase 3's deferred point); `no_view_holds_the_key` (a report with the made-up key in every field: its JSON does not hold the key). `tests/fixtures.rs`: `fixtures_are_current` (with `STEPLET_BLESS=1` writes; otherwise compares) for the scenarios of section 8, built from real corpus files and from reports made by `build_report` on made-up events, as phase 3's `tests/report.rs` does. With J7 A, `cargo test -p steplet-app` also writes `web/shared/types/`, and `git diff --exit-code web/shared/types` must be clean afterwards.
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-app --offline --test views --test fixtures` — Expected: FAIL, `unresolved import` naming `views`.
- [ ] **Step 3: write `views.rs` and the backend files;** then `$env:STEPLET_BLESS='1'; cargo test -p steplet-app --offline --test fixtures` once, and look at the written fixtures before keeping them.
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-app --offline --test views --test fixtures` (with `STEPLET_BLESS` unset) — Expected: `0 failed`. `npm --prefix web run check` — Expected: `0 ERRORS 0 WARNINGS` (the stand-in and the real backend both satisfy `Backend`).
- [ ] **Step 5: commit.** `git commit -m "The data each screen gets, with file text already cut into display segments, and fixtures saved from real output"`

### Task 10: the Open and Preview screens, and pins

**Files:** create `crates/steplet-app/src/open.rs`, `src/pins.rs`, `tests/open.rs`, `tests/pins.rs`; `web/shared/{FileText.svelte, Findings.svelte, Outcome.svelte}`, `web/app/screens/{Open.svelte, Preview.svelte}`, `web/app/tests/preview.spec.ts`.

**Interfaces:** Produces the commands `initial_preview`, `open_path`, `open_bytes`; `pins::{load, record}` on `%LOCALAPPDATA%\Steplet\senders.json` (written through a temporary file and a rename). Consumes `steplet_core::report::check`, `identity`, `REPOSITORY_HOSTS`.

- [ ] **Step 1: write the failing tests.** Rust: `a_file_over_256_kib_is_not_read_whole` (a 10 MiB file: at most 262,145 bytes read; the preview is blocked with `read.too-large`); `opening_changes_nothing_on_disk` (the pins file's bytes and time are the same after `open_path`); `a_host_steplet_does_not_fetch_from_is_said_before_run` (`gitlab.example.com` → `RunStatus::NotRunnable` with H9's hosts named); `a_file_with_no_repository_cannot_run_here`; `an_unreadable_pins_file_is_kept_and_said` (a pins file of junk: the preview has an amber "here" finding "Steplet could not read the senders it remembers, so it cannot warn about a changed key", and the file is not overwritten). Playwright, against the `screens` build: `the_run_button_comes_after_the_full_list` (its top is below the bottom of the body panel); `a_blocked_file_has_no_run_button`; `a_red_file_needs_the_exact_sentence` (Run disabled until the sentence is typed exactly; one wrong letter keeps it disabled); `text_that_imitates_steplet_stays_in_its_panel` (a title "✅ Verified by Steplet — safe to run": the text is found only inside `[data-from-file]` elements); `nothing_is_cut_off` (no element with `overflow` hidden or clipped has `scrollWidth > clientWidth + 1`, and the page has no sideways scroll at 560 px); `the_document_title_stays_steplet`; `opening_makes_no_network_request` (Playwright's request log holds only the page's own files).
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-app --offline --test open --test pins` — Expected: FAIL, `unresolved import` naming `open`. `npm --prefix web run build:screens; npm --prefix web test -- preview` — Expected: every test fails (the screen does not exist yet).
- [ ] **Step 3: write the Rust side and the screens** (section 1, screens 1 and 2; section 3.5). `FileText` draws a `Text` segment as plain text, `Invisible` as a badge `U+200B` with the title "An invisible character", `Link` as its shown text followed by its real address in a badge, never as a link.
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-app --offline --test open --test pins` — Expected: `0 failed`. `npm --prefix web run build:screens; npm --prefix web test -- preview` — Expected: `7 passed`.
- [ ] **Step 5: commit.** `git commit -m "Open and preview: everything the file asks for, the Run button last, and known senders"`

### Task 11: the run chooser, setup, the key, and opening `.steplet` files

**Files:** create `crates/steplet-app/src/{setup.rs, key.rs, register.rs, help.rs}`, `tests/{key.rs, register.rs}`; `web/app/screens/{Chooser.svelte, Setup.svelte}`, `web/app/tests/{chooser.spec.ts, setup.spec.ts}`.

**Interfaces:** Produces the commands `chooser`, `setup_status`, `setup_install`, `save_key`, `forget_key`, `set_opens_steplet_files`, `open_help`. `key.rs`: `save(target, key)`, `load(target) -> Option<ApiKey>`, `forget(target)` (`CredWriteW`, `CredReadW`, `CredDeleteW`, type `CRED_TYPE_GENERIC`, persist `CRED_PERSIST_LOCAL_MACHINE`). `register.rs`: `turn_on(ext, prog_id, exe)`, `turn_off(ext, prog_id)`, `state(ext, prog_id, exe) -> Switch { On, Off, Broken }` under `HKCU\Software\Classes`, then `SHChangeNotify(SHCNE_ASSOCCHANGED)`.

- [ ] **Step 1: write the failing tests.** Rust: `key_is_stored_read_and_forgotten` (target `Steplet-test/made-up-key`, the value `support::MADE_UP_KEY`-style made-up text; removed at the end, passed or failed); `the_setup_view_never_holds_the_key`; **(Z4)** `file_type_switch_round_trip` (`.steplet-test`, ProgID `Steplet.TestFile`: on → `state == On`, and `HKCU\Software\Classes\.steplet-test` names the ProgID; exe moved → `Broken`; off → no key left). Playwright: `codex_is_shown_unavailable_with_its_reason`; `only_the_lines_promises_returns_are_shown`; `wsl_missing_shows_the_command_to_type`; `install_progress_then_failure_offers_try_again_and_start_over`; `the_key_box_never_shows_a_saved_key`.
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-app --offline --test key --test register` — Expected: FAIL, `unresolved import` naming `key`. `npm --prefix web test -- chooser setup` — Expected: every test fails.
- [ ] **Step 3: write them.**
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-app --offline --test key --test register` — Expected: `0 failed`. `npm --prefix web run build:screens; npm --prefix web test -- chooser setup` — Expected: `5 passed`.
- [ ] **Step 5: measure double-click.** The owner turns the real switch on in Setup (J4 A); then `Invoke-Item examples\run-tests.steplet` (Windows' own "open" path, the one a double-click takes) — record whether Steplet opens it straight away or Windows asks which app to use, and paste.
- [ ] **Step 6: commit.** `git commit -m "The run chooser, setup with its checklist, the key in Credential Manager, and the switch for opening .steplet files"`

### Task 12: prepare, the run with its live log, and the ask screen

**Files:** create `crates/steplet-app/src/session.rs`, `tests/session.rs`, `tests/session_live.rs`; `web/app/screens/{Run.svelte, Ask.svelte}`, `web/app/tests/run.spec.ts`.

**Interfaces:** Produces `start_run`, `answer`, `stop_run`, and the `run-progress` events. `Session` takes the run function as a value (`fn(&RunRequest, &RunPaths, &Policy, &RunSetup, Arc<Live>) -> Result<Report, RunError>`), so its tests can use a stand-in; the app passes `steplet_run::run_live` with `ask_limit` 5 minutes and `decision_deadline` 5 min 15 s (S1), and limits of S2. A marker `%LOCALAPPDATA%\Steplet\run-in-progress.json` (run id, distribution id) is written before the run and removed after; at start, a marker left behind means a crash: stop and `discard` that run, then say so on the Open screen.

- [ ] **Step 1: write the failing tests.** Rust (`session.rs`, stand-in run function): `start_run_checks_the_file_again_now` (a file previewed before its expiry and started after it, with an injected clock: refused, "This file expired after you opened it"); `start_run_refuses_bytes_that_changed`; `red_needs_the_exact_sentence_on_the_rust_side_too`; `run_is_refused_while_another_window_runs` (a `RunLock` held elsewhere → "Another Steplet window is running a file. Finish or stop it first."); `an_answer_reaches_the_runner_bound_to_its_question`; `a_question_flashes_the_taskbar_and_waits_its_turn` (a stand-in window records `request_user_attention(Critical)` once per question; the second question is sent to the screen only after the first is answered); `closing_during_a_run_asks_first`; `a_run_left_by_a_crash_is_cleared_at_start` (a planted marker → `discard` called for that run, and the Open screen gets the note); `the_pins_are_written_when_run_starts_and_not_before`. `session_live.rs`, `#[ignore]`: `a_session_runs_use_case_1_with_the_scripted_model_and_asks_once` (the scripted model runs the declared command, then one undeclared one; the stand-in screen answers yes; `run-progress` holds `Prepared`, the events, one `Question`, `QuestionClosed { how: "yes" }`, and `Finished` with `completed: true`); `a_wrong_key_says_so` (**measure first:** the scripted model answers HTTP 401 and the test records what Claude Code 2.1.220 puts on its stream; then assert the run ends with `Failed`/report fault "Anthropic did not accept the key. Change it in Setup."). Playwright (`run.spec.ts`, stand-in backend): `ask_starts_on_no_and_yes_waits_one_second`; `ask_shows_the_whole_request` (a 300-character command and a 200-line file write, every character present); `a_withdrawn_question_closes_with_a_note`; `the_log_follows_the_end_unless_you_scrolled_up`; `stop_asks_first_and_then_says_interrupted`; `a_50000_line_output_keeps_the_log_responsive` (the screen is usable within 2 s of the event, shows 40 lines and "Show all 50,000 lines").
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-app --offline --test session` — Expected: FAIL, `unresolved import` naming `session`. `npm --prefix web test -- run` — Expected: every test fails.
- [ ] **Step 3: write `session.rs` and the screens** (section 1, screens 5 to 7; the ask screen's countdown runs from `Question.asked_at`).
- [ ] **Step 4: run, see them pass.** `cargo test -p steplet-app --offline --test session` — Expected: `0 failed`. `cargo test -p steplet-app --offline --test session_live -- --ignored --test-threads=1` — Expected: `ok. 2 passed`. `npm --prefix web run build:screens; npm --prefix web test -- run` — Expected: `6 passed`.
- [ ] **Step 5: commit.** `git commit -m "Prepare, the live log and the ask screen, with Stop and one question at a time"`

### Task 13: the report screen and saving it

**Files:** create `crates/steplet-app/src/save.rs`, `tests/save.rs`, `tests/fixtures/report.txt` (expected text); `web/app/screens/Report.svelte`, `web/app/tests/report.spec.ts`; modify `crates/steplet-run/src/run.rs` (faults in words, not `{e:?}`), `src/report.rs` (a call the gate turned away before deciding is labelled "Steplet turned this call's connection away", not "did not ask the gate"), `src/types.rs` and `src/settings.rs` (`RunRequest.allow_model_switch`, E1), `tests/settings.rs`.

**Interfaces:** Produces `report_text(&ReportView) -> String` and `save_report`, which opens `GetSaveFileNameW` with the default name of S4 and writes UTF-8 with Windows line endings. `settings_json` gains `allow_model_switch: bool`.

- [ ] **Step 1: write the failing tests.** `report_text_has_every_section_in_order` (equals `tests/fixtures/report.txt`: outcome, commands with exit codes and outputs, refused, your answers, models and refusals, changed files, trust, faults, notes, then "The agent says" last); `saved_text_has_no_control_characters_and_no_secret`; `the_default_name_never_comes_from_the_file`; `faults_are_in_words` (a `PrepareError` fault reads as a sentence, without `{` or `PrepareError`); `a_call_turned_away_is_labelled_as_such`; in `crates/steplet-run/tests/settings.rs`, `switch_models_only_when_the_receiver_allowed_it` (`switchModelsOnFlag` is `false` unless `allow_model_switch` is true). Playwright: `the_agent_says_comes_last_and_is_labelled`; `an_interrupted_run_offers_run_again`; `exit_codes_come_from_the_run` (a forged "exit 0" in the output does not change the shown exit code 1); `a_refusal_offers_run_again_allowing_another_model` (E1: the report of a refused run names the model, the category and the explanation, and offers that button; a run started from it lists every model used, D7).
- [ ] **Step 2: run, see them fail.** `cargo test -p steplet-app --offline --test save` and `cargo test -p steplet-run --offline --test settings` — Expected: FAIL (`unresolved import` naming `save`; the settings test's missing argument). `npm --prefix web test -- report` — Expected: every test fails.
- [ ] **Step 3: write them.**
- [ ] **Step 4: run, see them pass.** The same three commands — Expected: `0 failed`; Playwright `4 passed`.
- [ ] **Step 5: commit.** `git commit -m "The report screen, saved as plain text with a name Steplet chooses"`

### Task 14: every screen in every state: screenshots, looked at and fixed

**Files:** create `web/app/tests/screens.spec.ts` (one screenshot per row of section 8's table, per theme, per size), `docs/screens/phase-4/*.png`; modify the screens as the looking finds.

- [ ] **Step 1: take them all.** `npm --prefix web run build:screens; npm --prefix web test -- screens` — Expected: every test passes, and one PNG per scenario, theme and size in `docs/screens/phase-4/` (name `<screen>-<state>-<light|dark>-<1100|560>.png`). The test also fails on: any clipped text (Task 10's check, on every scenario), sideways page scroll, a missing focus ring when tabbing through every control, and any request the page makes.
- [ ] **Step 2: look at every screenshot** and write one line per screenshot in the results file: what was wrong and what was changed, or "right as it is". Check spacing against the 4-pixel scale, alignment of every column, sizes, colours against the tokens, and the forgotten states (empty, loading, error, very long text, small window).
- [ ] **Step 3: fix and repeat** until a full pass finds nothing. Paste the last run's summary line.
- [ ] **Step 4: commit.** `git commit -m "Screenshots of every screen in every state, light and dark, normal and narrow, each looked at and fixed"`

### Task 15: use case 1 on this laptop, from double-click to saved report

**Files:** create `examples/sample-run.steplet` (signed), `web/app/tests/real-app.spec.ts`; the results file.

- [ ] **Step 1 (Z5): the sample repository.** With J9 A and the owner's yes: a public `steplet-sample` on the owner's GitHub (a small `calc.py`, `tests/test_calc.py` using `unittest`, a README), and its full commit hash. The owner publishes it, or says I may.
- [ ] **Step 2: the sample file.** `examples/sample-run.steplet`: that repository and commit; steps to run the tests; `allow: commands: python3 -m unittest discover -s tests -t .`; signed with a throwaway key made for this, as phase 0 S9 did: `C:\Windows\System32\OpenSSH\ssh-keygen.exe -t ed25519 -f C:\dev\m1\p4\sample-key -N '""' -C steplet-sample`, then `cargo run -p steplet-sign --offline -- sign examples\sample-run.steplet --key C:\dev\m1\p4\sample-key --ssh-keygen C:\Windows\System32\OpenSSH\ssh-keygen.exe`, then `cargo run -p steplet-sign --offline -- check examples\sample-run.steplet` — Expected: the signature valid and the colour amber (`cap.runs-code`, G15); paste both.
- [ ] **Step 3: the key.** The owner pastes the test key (P5) into Steplet's key box. I do not type it.
- [ ] **Step 4: the run, driven.** Build with `--features e2e`; turn the switch on for that build; `Invoke-Item examples\sample-run.steplet`; `real-app.spec.ts` attaches through CDP and goes Preview → Run → Chooser → Prepare → Run → Report, saving a real-app screenshot of each screen (`real-<screen>-light.png`), with `STEPLET_SAVE_TO` (an `e2e`-only setting) in place of the Save dialog. Paste: the test's output, the saved report's text, and the line showing the test command's real exit code (0, unless the sample's tests fail).
- [ ] **Step 5: the run by hand.** With the normal build and the switch pointing at it, the owner double-clicks the file in Explorer, runs it, and saves the report through the Save dialog. Paste the saved file's first lines and its size.
- [ ] **Step 6: commit.** `git commit -m "Use case 1 on this laptop, from double-click to saved report"`

### Task 16: wrap-up

**Files:** `docs/milestone-1-phase-4-results.md` (summary), `docs/decisions.md` (J1 to J9 as chosen, S1 to S13, and anything decided while the plan ran), `docs/handoff.md`, `docs/plans/2026-10-08-milestone-1-plan.md` (status lines), `STATUS.md` (through its tool), the memory file.

- [ ] **Step 1: run every check and paste it:** `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --offline -- -D warnings`; `cargo test --workspace --offline --no-fail-fast`; `cargo test -p steplet-run -p steplet-app --offline --no-fail-fast -- --ignored --nocapture --test-threads=1` (without the Z3 install test, which Task 6 ran once); `npm --prefix web run check`; `npm --prefix web run build:screens; npm --prefix web test`; `lock_ages.py` on both Cargo lock files and `npm_ages.py` on `web/package-lock.json`; `rustc --version; cargo --version; node --version`.
- [ ] **Step 2: write the summary:** one row per done-means and per line of section 8 with PASS or FAIL and where its output is; "Changes from the plan"; "What phase 4 did not verify" (section 9, brought up to date).
- [ ] **Step 3: record the decisions, update the status lines and memory.**
- [ ] **Step 4: commit, then ask the owner** whether to fast-forward `main` and whether to push.

---

## 8. Every screen state, and every promise, with the test that backs it

**Done-means (milestone plan, phase 4):**

| Done means | Backed by |
|---|---|
| Screenshots of every screen in each state: empty, loading, error, a 300-character command, 25-item lists, a narrow window | Task 14 (the table below), Task 8 (the real app attaches) |
| Each screenshot looked at and fixed until it is right | Task 14 Steps 2 and 3, one line per screenshot |
| A full use case 1 run, from double-click to saved report, on this laptop | Task 15 |

**The states screenshotted** (each in light and dark, at 1100 and 560 pixels wide):

| Screen | States |
|---|---|
| Open | empty; reading; could not read; a crash's run was cleared |
| Preview | green; amber (the example); red with the sentence box; blocked; header unreadable; the longest file the limits allow (30 steps, 25 commands of 299 characters, 15 sites, 10 folders per list, 25 never lines, a long body, invisible badges, links); each identity line (not signed, new key, same key, key known under another name, key changed); not runnable here (host, no repository) |
| Run chooser | ready, no sites; ready, 15 sites; needs setup; WSL missing |
| Setup | nothing done; WSL missing; installing; install failed; all ready; key saved; switch broken |
| Prepare | fetching; ready with removed files; fetch failed; commit missing |
| Run | just started (empty log); a busy log; a 300-character command; long output closed and open; a tripwire stop; a model refusal; stopping; a wrong key |
| Ask | a command; a file write of 200 lines; a web address; 10 seconds left; 2 more waiting; withdrawn |
| Report | finished; finished with a failing test; stopped by a tripwire; interrupted by you; with faults; 40 commands; a forged result in the output; saved; save failed |

**The promises on screen this phase draws** (milestone plan section 7):

| Line on screen | Test |
|---|---|
| The chooser's six lines (Task 5's table) | the tests named there; `every_promise_names_tests_that_exist` |
| Prepare: "Steplet tells the agent not to load agent settings" | `the_strip_step_lists_what_it_removed`, and phase 3's trust check (`use_case_1_report_shows_the_test_command_s_real_exit_code_and_no_trust`) |
| Ask: "Yes allows this exact request for this run", "No answer in 5:00 means no" | `a_yes_allows_exactly_the_request_shown`, `a_request_that_differs_by_one_character_asks_again`, `no_answer_within_the_limit_is_a_no` |
| "Interrupted; the folder was reset; run again?" | `stop_signal_stops_the_run_and_the_report_says_you_stopped_it`, `stop_asks_first_and_then_says_interrupted`, phase 3's `stop_ends_a_run_mid_command_and_leaves_nothing_running` |
| Report: every model used, and refusals | phase 3's report tests; Task 14's "model refusal" state |
| Report and live log are safe to read | `each_event_reaches_the_screen_cleaned_and_in_order`, `saved_text_has_no_control_characters_and_no_secret`, `no_view_holds_the_key` |
| Codex: "Not available for files that run code on this computer" | `codex_is_shown_unavailable_with_its_reason` (stays until phase 7) |

---

## 9. What this plan does not verify

- **One laptop:** one Windows 11 Home, one WebView2, WSL 2.6.3, Claude Code 2.1.220; another Windows, a Windows 10, or another WebView2 version is not tried.
- **Only one real-model run** (Task 15). Everything else uses the scripted model or the stand-in backend.
- **How real receivers use the screens.** No one but the owner tries them in this phase.
- **An installed app.** Phase 4 runs the program from the build folder; an installer, code signing and SmartScreen's warning (row 45) are not tried.
- **The screens with assistive technology** (a screen reader). Keyboard use and focus are checked; spoken labels are written but not heard.
- **The Windows side of the network** (3.7 point 3) and **Task 3 of phase 3** (links out of the copy), both still open.
- **Whether WebView2's debugging port** (e2e builds only) could be reached by other programs during the e2e run; it is on 127.0.0.1, in a test build, for the length of a test.
- **Claude Code versions after 2.1.220:** their stream, their prompt-tool behaviour after a yes (Task 4 Step 1), and their rule merging (Task 1).
- **The Anthropic terms question (P1)** is unchanged.
- **What the fence cannot report:** blocked attempts stay invisible to Steplet (phase 0 S6); the report says so.
