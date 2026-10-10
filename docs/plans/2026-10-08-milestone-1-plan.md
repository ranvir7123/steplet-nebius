# Milestone 1 plan: a developer receives a steplet and runs it safely

Written 8 Oct 2026. Status: **approved by the user on 8 Oct 2026, with the recommended option for P1 to P6** (recorded in `docs/decisions.md`, "Milestone 1 plan choices"). Nothing below has been built or installed yet. Phase 0's build plan (`docs/plans/2026-10-08-milestone-1-phase-0.md`) is not written yet. **Phase 0 finished on 10 Oct 2026; results in `docs/milestone-1-spikes.md`** (summary at its end).

Sources: `docs/handoff.md` (milestone 1 scope), `docs/decisions.md` (R-rows win over older rows; "After milestone 0" rows D1 to D8 and corrections C1 to C7 win over both where they touch), `docs/milestone-0-wrap-up.md` (what each runner may promise), three reports from 8 Oct 2026 (two architects, one documentation lookup; their key claims are quoted where used). Settled decisions are not re-opened.

**Why one milestone plan and several build plans.** Milestone 1 covers separate parts that each need their own tests: the file core, rating, the WSL runner, the screens, the check page, the hostile test suite, the Codex trials. A single step-by-step plan for all of it would run to thousands of lines and go stale before the later parts start. This file fixes who it is for, the parts, the order, how the parts connect, what "done" means, and the choices you need to make. Each phase then gets its own step-by-step build plan (tests first, exact files, exact commands), written just before that phase starts and approved by you before any code.

Words used below, explained once:
- **Crate:** a Rust package. **WASM (WebAssembly):** a way to run Rust code inside a web browser.
- **Fence (sandbox), checker (gate), loopback, hook, journal:** as in the milestone 0 wrap-up.
- **Spike:** a short, throwaway test that answers one question before we build on it.
- **Interop:** WSL's way of starting a Windows program from Linux.
- **Named pipe:** a Windows-only channel between two programs on the same computer; it has a permission list and no network port.
- **Working copy:** the folder of the repository's files that a run works in. Steplet makes a fresh one for every run.

---

## 1. Who uses milestone 1, and what they do

### The receiver (the main user)

**Who:** a developer on Windows 11 with WSL2 (row 2: developers first). They receive a `.steplet` file from another developer, by chat or email.

**What they are trying to get done:** one of R27's contained use cases:
- **Use case 1:** "check out this repository at this exact commit, run its tests, tell me the result".
- **Use case 2:** "run this script to reproduce a bug and send me the log".
- **Use case 5:** a file that looks exactly like use case 1, but whose test helper tries to read a private key and send it out. The receiver must not be able to tell it apart, and must still be safe.

**What they see and touch, in order:**
1. **Open.** They double-click the file, or drag it onto the app. The app opens it without any network access.
2. **Preview.** Counts first, then every step, everything the file asks to touch, everything it must never do, its expiry, and one line on who signed it ("signature valid, sender unknown" until identity is established, R14). The colour comes from code rules only. Nothing is cut off, and invisible characters show as visible badges (row 28). The Run button comes after the full list.
3. **Run chooser.** Each runner is listed with the concrete things it blocks, worded only as milestone 0 measured (wrap-up section 2). In milestone 1 that means Claude Code in WSL. Codex is shown as "not available for files that run code on this computer", with the reason (D1). If no contained runner is set up, the app says "cannot run safely on this computer" and shows what to set up.
4. **First-time setup** (once per computer). The app checks WSL2, the Linux distribution, the sandbox tools, git, and Steplet's own copy of Claude Code. It shows the exact `sudo` commands the receiver must type themselves; Steplet never runs `sudo`. Then a one-time sign-in (see decision P1).
5. **Prepare.** Steplet makes a fresh working copy:
   - It clones the repository from the address the file declares, without checking out.
   - It checks out the declared commit inside the fence.
   - It removes the repository's agent settings files.
   - It shows "repository ready: commit `abc123…`; removed `.claude/settings.json` (2 hooks), `.mcp.json`; Steplet tells the agent not to load agent settings" (D3, D6).
6. **Run.** A live log. It shows what the agent says (labelled as such), each command, each checker decision, each command that ran without asking (D8), and each model change (D7). There is a Stop button.
7. **Ask.** If the agent proposes something the file did not declare, the run pauses and the receiver is shown the exact change ("allow `pip install pytest` for this run?"), bound to that exact request (R7).
8. **Report.** What ran, with exit codes taken from the run itself. Then:
   - test or script output, with secrets scrubbed (R17);
   - what the checker refused;
   - every model used, and any refusal (D7);
   - what changed in the working copy;
   - the agent's own summary, shown last and labelled "the agent says" (F47).

   The receiver can save the report as a file. Sending it back to the sender is milestone 3 (R22).

**Where they get stuck, and what milestone 1 does about each:**

| Stuck point | What the app does |
|---|---|
| WSL2 not installed, or not an Ubuntu distribution | Setup screen explains, links to Microsoft's instructions, and says the file cannot run safely until then. Milestone 1 supports Ubuntu only (the only one tested). |
| Missing `socat` or `bubblewrap`, or the Ubuntu AppArmor setting the Claude Code docs mention | Shows the exact command to type; re-checks when they press "Check again". |
| No sign-in or API key | One-time sign-in step (decision P1). |
| The file is red | It cannot run, no override (row 44b, R9); the preview says which rule and why. |
| The file needs something not declared | The ask screen. If the receiver does not answer within Steplet's limit, the answer is "no". |
| Clone fails (wrong address, private repo, no network) | Clear error naming the address; nothing runs. Private repositories are out of milestone 1 (decision P3). |
| Run interrupted (crash, laptop sleep, Stop) | "Interrupted; the folder was reset; run again?" (D4). |
| The checker stalls | Steplet's own deadline refuses the action (F36). |

### The sender

**Who:** a developer who wants someone else to run a task. **What they do in milestone 1:** write the file by hand from a documented template, check it with `steplet-sign check`, sign it with `steplet-sign sign` (which calls their own `ssh-keygen`, so their SSH agent, passphrase and hardware keys work as usual), and send it. **Where they get stuck:** the strict header format rejects YAML styles it does not allow (for example `[a, b]` lists). Every error names the line, says what is allowed, and shows a corrected example.

### Anyone checking a file before installing the app

**What they do:** open the public check page and drop the file on it. They see the same findings as the app's preview (R10: file findings only; whether it can run depends on their machine). The file never leaves the browser: the page is built so the browser itself blocks any upload.

---

## 2. What milestone 1 delivers, and what it does not

**Delivers** (handoff milestone 1 list, adjusted by D1 to D8):
1. The file core: strict header parser, clean-up and signature checks, display segments.
2. The sender tool `steplet-sign` (check, sign) with test vectors.
3. The rating library and the one rule checker `Policy::check`, native and as a WASM build.
4. The WSL runner and the Claude Code adapter, with every requirement from wrap-up section 2.1 and D3, D4, D6, D7, D8.
5. The desktop app: per-user file association, preview, run chooser, setup, prepare, live log, ask screen, report.
6. The public check page (built and tested locally; where it is hosted is decided later).
7. The R20 hostile test suite as code, for the Claude Code adapter.
8. The Codex trials from D1 and D2, recorded like milestone 0. The full Codex adapter is built only if a trial passes (decision P2).

**Does not deliver:**
- the built-in agent and the hosted relay (milestone 2);
- native-mode use cases 3 and 4, and with them the change journal (R18), which those use cases need (milestone 2);
- sending reports (milestone 3), link page (milestone 3), kill switch and verified senders (milestone 4);
- browser control (version 2, R24);
- "Export as SKILL.md" (R12; not in the milestone 1 list);
- private repositories and local folders as sources (decision P3);
- macOS.

---

## 3. Technical choices, decided from the reports

Per the working rule (technical choices: architect evidence, then decide), these are proposed as decided. You can still object when approving the plan. Each line gives the choice, the runner-up, and the reason.

| Area | Choice | Runner-up | Why |
|---|---|---|---|
| Header parser | **Our own parser for a small YAML subset**: block maps, block lists, lists of small maps, plain or quoted values, `\|` blocks. Field types come from our schema, never from YAML guessing. `saphyr-parser` is used in tests only, to check that anything we accept parses the same way there. | `saphyr-parser` 0.1.0 | Row 15 needs anchors, aliases, tags, extra documents and duplicate keys refused before anything else sees them. The serde-based crates expand aliases first. `serde_yml` is flagged unsound (RUSTSEC-2025-0068) and `serde_yaml` is deprecated. `saphyr-parser` is three weeks old. Cost: every parser bug is ours, hence fuzzing and the differential test. |
| Signatures | **RustCrypto `ssh-key` 0.6.7**, only the `alloc` and `ed25519` features. We also refuse non-Ed25519 keys, hashes other than sha512, and versions above 1. Signing calls the sender's own `ssh-keygen -Y sign`, then re-checks the result with our core. | About 150 lines of our own code on `ed25519-dalek` | The architect read 0.6.7's source: `verify` checks the embedded key and the namespace. A 0.7 release candidate exists, so an upgrade is coming. |
| Core shape | **`steplet-core`: `no_std` + `alloc`** (it cannot touch files, network or clock). Time and already-resolved paths come in as arguments. Modules: `canon`, `header`, `sig`, `rules`, `policy`, `display`. | Normal Rust plus a lint banning operating-system calls | In the browser, file calls compile but fail at run time, so the app and the check page could quietly disagree. `no_std` makes that impossible to build. |
| WASM build | **`cargo build --target wasm32-unknown-unknown` plus the `wasm-bindgen` command-line tool pinned to the crate's version** (0.2.129) | `wasm-pack` 0.15.0 | `wasm-pack` moved to one maintainer after the rustwasm group closed (Rust blog, 21 Jul 2025), and it downloads tools during builds. |
| Screens | **Svelte 5 + TypeScript + Vite, no SvelteKit.** Folders `web/shared`, `web/app`, `web/check`. File text reaches the screen only through one `FileText` component that draws the core's display segments. Svelte's raw-HTML tag is banned by a lint rule. Markdown is never turned into HTML. TypeScript types are generated from Rust (`ts-rs`). | React 19 | Smaller check page; one shared findings view for the app and the check page. Tauri recommends Vite. |
| Desktop shell | **Tauri 2.12.x** (not the 3.0 alpha) | none | Row 32. |
| Gate channel, WSL to app | **A small Windows program, `steplet-gate.exe`, that Claude Code starts through interop as its permission-prompt server.** It asks the app over a per-run named pipe that only the user's own account may open (restricted tokens and other computers refused, name created by the app first, process checked, per-run token). It keeps no memory between requests; the app sets a deadline on every decision; a broken pipe means "refuse"; it starts no other programs. | `--permission-prompt-tool stdio` (the app answers over the stream it already has) | Nothing listens on a network port, which D1 shows is dangerous. The runner-up uses an undocumented protocol, and issue #34046 (closed "not planned", 3 May 2026) reports tools running without asking in that mode. **Spike S1 must pass first.** |
| Launching the run | **`wsl.exe --distribution-id <id> --cd <work> --exec /usr/bin/timeout … /usr/bin/env …`**. The app starts `wsl.exe` with an emptied Windows environment, so no Windows variables cross over. `env` sets only R15's list: `HOME` (Steplet's private Linux home, so files like F49's git ignore land there and not in the receiver's home), a minimal `PATH`, `LANG`, `CLAUDE_CONFIG_DIR`, `DISABLE_AUTOUPDATER=1`, `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`. The task goes in through a file, not the command line (files can be 256 KiB). Stop: `SIGTERM` (a clean refusal, F37), `SIGKILL` after a deadline, and `timeout` as the backstop. | Our own small Linux launcher program | Every option is in `wsl --help` (WSL 2.6.3). **Spike S2 must pass first.** |
| Linux-side folders | Private folder `~/.local/share/steplet` (Steplet's pinned Claude Code, the private Claude settings folder with the sign-in, Steplet's private home, per-run settings) and a separate work folder for working copies. Logs and the gate stay on Windows. `denyRead` covers all of `/mnt`, the private folder and the work folder; `allowRead` re-opens only this run's copy (the docs say "the narrower path applies"). Matching `Read`/`Edit` deny rules cover Claude Code's own file tools. | none | Wrap-up 2.1 and row 24: the agent's own credentials must be unreadable from inside the fence. **The suite tests it.** |
| Claude Code version | **Steplet installs its own pinned copy into its private folder** and never uses the `claude` on `PATH` (in WSL that is the Windows wrapper). Milestone 1 starts on 2.1.220, the only version measured. A newer version is added only after the automated suite passes on it (R20). | The receiver's own install | One known, tested version; unaffected by the receiver's own settings and hooks. |
| Clone and checkout (D6) | **The receiver's git inside WSL, by absolute path, hardened:** `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, an empty `HOME`, `GIT_TERMINAL_PROMPT=0`, `GIT_LFS_SKIP_SMUDGE=1`, and `-c protocol.allow=never -c protocol.https.allow=always -c core.hooksPath=/dev/null -c core.fsmonitor=false -c transfer.fsckObjects=true -c transfer.bundleURI=false -c credential.helper= clone --no-checkout --no-recurse-submodules --template=`, then a check that the commit exists. **Checkout runs fenced with no agent**, under Anthropic's `srt` sandbox tool (pinned). **`.git` is read-only for the agent**: `sandbox.filesystem.denyWrite` on `<work>/.git`, plus an `Edit` deny rule. | `gix` (git in Rust) | `gix` would have to write across into the Linux disk from Windows. Whether it never runs hooks was not researched. Risks of the choice: the receiver's git version is not ours (Ubuntu patched CVE-2025-48385 inside a package still numbered 2.43.0), and `srt` calls itself a "Beta Research Preview". **Spike S4 must pass first.** |
| Hostile suite form | **A Rust test program in the repository that drives the real launcher, gate and checkout code**, with the milestone 0 fixture and a fake key made fresh each run. Two layers: (1) a **scripted model**, a fake model server Claude Code talks to through `ANTHROPIC_BASE_URL` (documented), so results repeat exactly, no "cyber" refusals and no INCONCLUSIVE runs; (2) a **live-model layer**, once per supported version, with recorded output. | Milestone 0's scripts | The scripts test a different code path from the app's. A scripted model can hide behaviour that depends on the real model, so the live layer stays. **Spike S7 must pass first.** |

---

## 4. Choices for you (product), each with a recommendation

### P1. How Claude Code signs in

**What was found** (documentation lookup, 8 Oct 2026, quoting `code.claude.com/docs/en/legal-and-compliance`):
- Forbidden: Anthropic does not permit third-party developers "to route requests through Free, Pro, or Max plan credentials on behalf of their users". Also: "developers may not collect, store, or intermediate Claude.ai credentials or session tokens."
- Allowed: an end user "signing in to the unmodified Claude Code binary with their own Claude subscription".
- Steplet sits in between, and keeping the sign-in in Steplet's private folder is the risky part.
- An API key route is documented: "In non-interactive mode (`-p`), the key is always used when present" (authentication page).
- Not opened: the Commercial Terms and the Usage Policy pages.

| Option | What goes wrong |
|---|---|
| **A. API key only in milestone 1.** The receiver uses their own Anthropic API key. You ask Anthropic in writing whether subscription use is allowed for an open-source app like this; subscription sign-in comes only after a written yes. | Receivers pay per use, and many developers have a subscription but no API key, so fewer people can run files at first. The key must not be visible to fenced programs: spike S3. |
| **B. Subscription sign-in in Steplet's private folder now.** | May break the terms quoted above; Anthropic says it may enforce "without prior notice". Receivers could lose their accounts because of our app. |
| **C. Use the receiver's own Claude Code settings folder.** Steplet stores nothing. | Loses the isolation milestone 0 depended on: the receiver's own hooks and settings would load (F41 shows hooks run outside the fence). Still unclear under the terms. |

**Recommendation: A.** Asking Anthropic is an outside message only you can send; I can draft it.

### P2. What milestone 1 builds for Codex

| Option | What goes wrong |
|---|---|
| **A. Trials only:** `mxc` first (no Steplet code needed), then Steplet's own loopback block, then D2's tests through a thin `app-server` test client. The adapter is built only if a trial passes. | If a trial passes late, Codex support slips to milestone 2. |
| **B. Build the full Codex adapter now.** | Weeks of work on something that cannot be offered (C7). `app-server` is marked "[experimental]" in 0.160.1, and one unofficial write-up (21 Apr 2026) says its approval request may not include the command. |

**Recommendation: A.**

### P3. Where a repository may come from in milestone 1

| Option | What goes wrong |
|---|---|
| **A. A public `https` address plus the full commit hash only.** | No private repositories and no "use the copy I already have" in milestone 1. |
| **B. Also local folders.** | Cloning from a local folder brings back git's local-clone security holes, and the receiver's own repository could carry planted hooks. |
| **C. Also private repositories.** | Needs the receiver's git credentials near the run, against row 24 and R15. |

**Recommendation: A.**

### P4. The signature name space

Row 13 says `runfile-v1`, chosen before the rename. Nothing has been signed for real yet.

| Option | What goes wrong |
|---|---|
| **A. Change it to `steplet-v1` now.** | Nothing; this is the cheapest moment. |
| **B. Keep `runfile-v1`.** | The old name stays inside every signature forever. |

**Recommendation: A.**

### P5. Where the hostile suite runs

The architect recommended a separate machine with a test account, because this laptop holds real sign-ins. Windows 11 Home has no Hyper-V, so a local virtual machine running WSL2 is hard.

| Option | What goes wrong |
|---|---|
| **A. This laptop, in a separate WSL distribution made only for tests**, with a test-only Anthropic API key that has a spending cap. Your main Ubuntu and its sign-ins are untouched. | Shares one Windows with your real files. The Windows-side tests (Codex, the loopback block) still touch the real Windows. |
| **B. A second computer or a cloud Windows machine.** | Cost and setup; slower to iterate. |
| **C. This laptop and your main Ubuntu, as in milestone 0.** | Real credentials sit next to hostile test code (the wrap-up found two readable sign-in files). |

**Recommendation: A for milestone 1, and B before launch.**

### P6. Making `C:\dev\runFile` a git repository

Needed for commits. First, `keys.txt` and `sk.json` must move out of the folder (already an open item), so they can never enter history. A licence (row 4: open source, permissive) is needed before anything is published, not before local commits.

**Recommendation:**
- You move the two files out.
- I run `git init` with a `.gitignore`, after your yes.
- The licence is decided before the first public push. My suggestion then would be Apache-2.0, because row 23 copies rule ideas from Apache-2.0 projects. That is your call later.

---

## 5. How the parts connect

```
steplet-core (no_std, pure)          steplet-wasm ──► web/check (public check page)
  canon ─ header ─ sig ─ rules ─ policy ─ display
        │                                   │
        ▼                                   ▼
steplet-sign (sender tool)          src-tauri (desktop app, Windows)
                                      ├─ file open, pins (trust on first use)
                                      ├─ runner: Runner trait ── wsl_claude adapter
                                      │      ├─ setup checks, pinned install
                                      │      ├─ working copy (clone, fenced checkout, strip agent files)
                                      │      ├─ launcher (wsl.exe), stream reader, stop
                                      │      └─ gate service (named pipe) ◄── steplet-gate.exe (started by Claude Code via interop)
                                      └─ web/app (Svelte screens, shares web/shared with the check page)
tests/hostile (R20 suite: scripted model + live layer)
```

Main interfaces. Each phase's build plan fixes the exact types:
- `steplet_core::canon::canonicalize(raw: &[u8]) -> Result<Canonical, CanonError>`. The clean-up from row 14; it splits off the signature block by fixed markers.
- `steplet_core::header::parse(c: &Canonical) -> Result<Steplet, ParseError>`. Parse once, from the signed bytes (row 15).
- `steplet_core::sig::verify(c: &Canonical, namespace: &str) -> SigStatus`, one of `Unsigned`, `Valid { fingerprint }`, `Invalid { reason }`.
- `steplet_core::rules::rate(s: &Steplet, sig: &SigStatus, now_unix: i64) -> Rating`. `Rating { outcome: Blocked | RedFindings | Amber | Green, findings: Vec<Finding> }` (R9); each `Finding` has a fixed rule id, a severity and a plain-words reason (row 23).
- `steplet_core::policy::Policy::from_steplet(s: &Steplet) -> Policy` and `Policy::check(&self, action: &Action) -> Decision`, one of `Allow`, `Deny { rule, reason }`, `Ask { reason }`. This is the one matcher (row 29), used by the preview, the gate and the ask screen.
- `steplet_core::display::segments(text: &str) -> Vec<Segment>`, one of `Text`, `InvisibleBadge { codepoint }`, `Link { shown, real }`.
- `steplet-wasm`: `check(bytes: Uint8Array, now_unix: i64) -> string`, the same `Rating` as JSON.
- Runner trait (adapter memo, minus resume, per D4): `detect`, `capabilities`, `prepare`, `start`, `events`, `stop`.
- Normalised run events: `AgentText`, `ToolUse`, `ToolResult`, `GateDecision`, `SkippedGate { command, on_harmless_list }`, `HookEvent` (stops the run, D3), `ModelSwitch { from, to, category }` (D7), `Result`, `Stderr`.
- Gate request over the pipe: `{ run_token, tool_name, input, tool_use_id }`. Answer: `{ behavior: "allow" | "deny", message?, updatedInput? }`, within the app's deadline.

---

## 6. Phases, in order

Each phase starts with its own build plan for your approval, and ends with the command output that shows its tests pass. Phases 4 and 6 depend on the spikes in phase 0.

### Phase 0: setup and spikes (no product code)

Setup, each with your yes first:
- `git init` (P6).
- `rustup target add wasm32-unknown-unknown`.
- `cargo install wasm-bindgen-cli --version 0.2.129`.
- Node packages for `web/`.
- Playwright and its browser download.
- The Tauri command-line tool.
- The nightly Rust toolchain and `cargo-fuzz`.
- In WSL: Steplet's pinned Claude Code, `srt` (pinned), and the test distribution (P5).

Each spike is a throwaway test whose command and output go into `docs/milestone-1-spikes.md`, the same way milestone 0 recorded its tests:

| # | Question | Pass means | If it fails |
|---|---|---|---|
| S1 | Can Claude Code start a Windows `.exe` through interop as its permission-prompt server? Does that program exit when Claude Code is stopped or killed, and come back after a restart? Can a fenced program still not start Windows programs (F28)? How long does one decision take? | All yes; the time is recorded | Switch to the stdio runner-up, but only after the suite shows no action skips the checker |
| S2 | What does `wsl.exe --exec` add to the environment? Do Linux processes outlive a killed `wsl.exe`? Does Claude Code work with Steplet's private `HOME`, and does F49's git ignore land there? | No Windows variables cross; no process outlives the run once the clean-up runs; F49's file lands in the private home | Own Linux launcher (runner-up) |
| S3 | With API-key sign-in (P1-A), can a fenced program see the key, either in its environment or through an `apiKeyHelper` file? Does a key stop Claude Code using any stored sign-in? | The key is invisible to fenced programs | Find a route where it is; until then, P1-A cannot ship |
| S4 | Checkout under `srt` with no agent. Does `denyWrite` on `.git` refuse writes from commands **and** from the Write tool? Optional: the clone itself under `srt`, allowing only the git host. | Checkout works; both `.git` writes are refused | Steplet's own hardened checkout, unfenced (D6 fallback) |
| S5 | Is there a setting that makes every command ask the checker (D8-C)? Does `--tools` limit the tool list as its help text says? | Recorded either way | D8's harmless list (already decided) |
| S6 | Can the refusal model switch be turned off (D7-C)? Does Claude Code report blocked fence attempts anywhere the app can read (stream, stderr, a log)? | Recorded either way | D7-A (already decided); the report says "blocked attempts are not visible to Steplet" |
| S7 | With a fake model server (`ANTHROPIC_BASE_URL`), does every action still go through the same gate and fence? | Same gate requests and fence results as T1c | Suite runs live-only, and costs more |
| S8 | From inside the WSL fence, can a program reach a listener on the Windows side (127.0.0.1, the address WSL uses for the host, all addresses), in WSL's default network mode? | All refused | The chooser keeps "programs running in WSL" wording only; it becomes a finding |
| S9 | Does `ssh-key` build `no_std` for `wasm32`? Does Svelte run in WebView2? Does a check page with embedded WASM still work under `connect-src 'none'`? | All yes | Adjust the core or page build before phase 1 |

### Phase 1: the file core and the sender tool

- `steplet-core`: `canon`, `header`, `sig`, `display`, and the size limits (row 12).
- `steplet-sign check` and `sign`.

**Done means:**
- Test vectors made by Windows' own `C:\Windows\System32\OpenSSH\ssh-keygen.exe` (by full path: Git Bash finds a different one first) verify in our core, and ours verify in `ssh-keygen`. The vectors are: valid; tampered; wrong name space; sha256 hash; RSA key; signature block missing, doubled or moved; Windows line endings, which must still pass.
- Property tests show that cleaning up twice gives the same result, that Windows and Unix line endings sign the same, and that any edit to non-whitespace breaks the signature.
- The differential test against `saphyr-parser` agrees on every file we accept.
- The parser fuzz target runs for a recorded time without a crash.

### Phase 2: rating and the rule checker

- `rules` (the row 23 groups and the R9 outcomes), `policy` (`Policy::check`, R8 command rules, path rules, the always-blocked list from row 24).
- `steplet-wasm`.
- The test corpus, about 60 ordinary and 150 hostile files: one hit and one near-miss per rule id, enforced by a test. Seeds come from Cisco skill-scanner (Apache-2.0, credited).

**Done means:**
- Every rule id has its hit and near-miss.
- Golden files (`insta`) are stable.
- The native and WASM builds give identical results on the whole corpus.
- The amber and red counts on the ordinary files are recorded, as the first data for tuning rows 12 and 26.

### Phase 3: the WSL runner and the Claude Code adapter

- Setup checks and the pinned install.
- Generated settings: `denyRead` on `/mnt`, the private and work folders, and every row 24 path on both the Linux side and under `/mnt/c`; `allowRead` for this run's copy; `denyWrite` on `.git`; deny rules for Read and Edit; the sandbox settings from milestone 0. Never allow rules (R4).
- The launcher, the stream reader and normalised events, stop.
- The working copy (clone, fenced checkout, removing agent settings files, discarding it after a run or interruption).
- The gate service and `steplet-gate.exe`, with deadlines.
- The tripwires: any hook event stops the run (D3); a skipped command not on the harmless list stops the run (D8); any action that skipped the gate otherwise stops the run (row 38).
- Model-switch reporting (D7).
- The check after each run that no trust was recorded (D3).
- The report builder (outcome from the run itself, F35).
- Output handling: control characters removed after decoding, secrets scrubbed in every text field (F48, T5b).

**Done means:** the scripted-model layer of the suite (phase 5) passes for every item in section 7 that this phase covers.

### Phase 4: the desktop app

- Tauri shell; per-user `.steplet` file association and the `steplet://` scheme registered without admin rights (row 32).
- Screens: open, preview, run chooser, setup, prepare, run and live log, ask, report.
- Sender pins (trust on first use, row 16; warnings on a changed key, row 22).
- Re-checks at run time (row 30): expiry, the file's fingerprint against the approved one, the exact command.
- Copy rules: fixed window titles and dialog text, because the sender chooses the file name.

**Done means:**
- Playwright screenshots of every screen in each state: empty, loading, error, a 300-character command, 25-item lists, a narrow window.
- Each screenshot looked at and fixed until it is right.
- A full use case 1 run, from double-click to saved report, on this laptop.

### Phase 5: the hostile suite as code (R20), for the Claude Code adapter

- Built alongside phase 3, finished here.
- Scripted-model layer, target about 15 minutes, with Steplet's deadlines set short. Live-model layer, about 45 minutes, once per supported version, output recorded.
- Covers section 7's table, plus new gate cases: someone else holding the pipe name first, a wrong program connecting, a restricted-token client.

### Phase 6: the public check page

`web/check`: drop a file, see findings, using the shared findings view and the embedded WASM. Content-security rule `connect-src 'none'`. **Done means:** same results as the app on the corpus (phase 2's parity test run in a real browser), and screenshots in the same states as phase 4.

### Phase 7: Codex trials (P2-A)

`mxc` first, then Steplet's loopback block (the Windows Filtering Platform), then T3c to T3f and the `untrusted` follow-up through a thin `app-server` test client. Recorded like milestone 0.

**Done means:** a written go or no-go for the Codex adapter, with the output that backs it. A "go" adds the adapter to milestone 2 (C7).

Order: phase 0, then 1, then 2. Phase 3 needs `Policy::check` from phase 2. Phase 5 grows alongside phase 3. Phase 4 needs phases 1 to 3. Phase 6 needs phase 2. Phase 7 can run any time after phase 0.

### Milestone 1 is done when (each with pasted output)

1. **Use case 1:** a signed steplet naming a public repository and a commit is double-clicked on this laptop, previewed, run contained through Claude Code in WSL, and ends in a saved report that shows the test command's real exit code and output.
2. **Use case 2:** the same, with a triage script, and its log in the report.
3. **Use case 5:** the milestone 0 hostile fixture, published as a public test repository. Every attempt is blocked, checked by the suite from the fixture's own results file and the listener log, not from the agent's words. The preview shows the colour the rules give that file, with no special case.
4. **Same answer in two places:** the app's preview and the check page show identical findings on every corpus file.
5. **Suite:** the scripted layer passes for the pinned Claude Code version. The live layer has run once with its output recorded. Every line in section 7 maps to a passing test.
6. **Screens:** every screen has been screenshotted in every state and fixed until right.
7. **Codex:** phase 7's go or no-go is written down.

---

## 7. Every promise on screen, and the test that backs it

The run chooser may show a line only while its test passes on the pinned version (R6, R20). The wording comes from wrap-up section 2.1.

| Line on screen | Suite test |
|---|---|
| "Steplet checks the agent's shell commands that change something, its file writes and its web fetches before they happen. If Steplet's checker fails, the action is refused." | Gate crash, stall (Steplet's deadline), wrong reply, killed (T2a to T2d); WebFetch refused (T2e); a skipped command not on the harmless list stops the run (D8); pipe squatting, wrong program, restricted token (new) |
| "The agent can fetch only these web addresses: …" | T2e |
| "Programs this task runs can change files only inside this folder." | T1c attempt C |
| "Programs this task runs cannot read these paths: …" | T1c attempt B, T1c-b1 (`/mnt/c`); new: Steplet's private folder, the Claude sign-in and the API key unreadable (S3) |
| "Programs this task runs cannot connect to the internet, or to other programs running in WSL." Extended to "on this computer" only if S8 and its suite test pass. | T1c attempts D and E; S8's Windows-side listener |
| "Programs this task runs cannot start Windows programs." | T1c attempt G |
| Prepare screen: "Steplet tells the agent not to load agent settings" | T3a and T3b, untrusted and trusted; hook tripwire (D3); trust check after the run |
| "Interrupted; the folder was reset; run again?" | New restart-clean test (replaces T4, C6) |
| Report: every model used, and refusals | Model-switch test (scripted if the fake server can produce a refusal, otherwise the live layer) |
| Report and live log are safe to read | T5a to T5c, after decoding |
| Codex: "not available for files that run code on this computer" | Stays until a phase 7 trial passes |

---

## 8. Global constraints for every phase

- Windows 11 first; per-user install, no admin rights (row 32). Ubuntu in WSL2 only for milestone 1.
- Rust stable (1.98.0 on this laptop, MSVC); `steplet-core` is `no_std` + `alloc`.
- `ssh-key` 0.6.7 with features `alloc`, `ed25519` only; Ed25519, sha512 and SSHSIG version 1 only. Name space per P4.
- Size limits (row 12): file 256 KiB, header 32 KiB, at most 25 commands, 15 domains, 10 folders per list, 30 steps, any command under 300 characters; over a limit is red.
- Colour is the worst single finding; outcomes per R9; red cannot run, no override (row 44b).
- One matcher, `Policy::check`, for preview, gate and ask screen (row 29).
- No network access while previewing (row 28). The check page never uploads.
- Contained Claude Code runs always use:
  - Steplet's pinned copy and private folders;
  - `--setting-sources user`, `--strict-mcp-config`, Steplet's own `--settings`, and `--permission-prompt-tool`;
  - D3's layers;
  - no generated allow rules (R4).
- Child environment: R15's allowlist only.
- Evidence rule: no step is "done" without the command and its pasted output.
- Plain words in all screen text, commit messages and documents. No attribution lines in commits.

---

## 9. What this plan does not verify

- Every spike in phase 0 is unanswered until it runs. The gate channel (S1), the launcher (S2), key exposure (S3) and the fenced checkout (S4) are each a real risk to the design.
- The terms question is answered only partly (P1). The Commercial Terms and Usage Policy pages were not opened.
- The architects' library facts (versions, dates, maintenance) were read on 8 Oct 2026 and will drift.
- The git-hardening list comes from the architect's general knowledge plus git's own option names; S4 and the suite test it only partly.
- Whether WSL distributions other than Ubuntu, Windows 10, or Windows Pro behave the same.
- How long any phase takes. No estimate is given; milestone 0 measured no build effort.
- The receiver's experience of first-time setup; it has never been tried by anyone but you.
