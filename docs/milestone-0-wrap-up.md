# Milestone 0 wrap-up: what each runner may promise, and what is left to decide

Written 8 Oct 2026. Status: **approved by the user on 8 Oct 2026, who chose the recommended option for every item D1 to D8.** The decisions and the corrections to earlier rows are recorded in `docs/decisions.md`, section "After milestone 0" (rows D1 to D8, C1 to C7). The option tables below are kept as the reasoning.

Sources: `docs/milestone-0-results.md` (every command and its pasted output, findings F1 to F49, the step 2 and step 3 summary tables), `docs/plans/2026-10-07-milestone-0-execution-boundary.md`, `docs/milestone-0-handoff.md`, `docs/decisions.md` (R-rows win over older rows). Test IDs (T1a, T3c…) and finding numbers (F7…) point into the results file. One architect review of this draft was run; points it raised that I checked are marked "checked 8 Oct", and its general-knowledge claims are marked "not verified".

Words used below, explained once:
- **Fence (sandbox):** what the operating system puts around the programs an agent starts, so they can only touch what we allow.
- **Checker (gate):** Steplet's own code that the agent must ask before each action, and that answers yes or no. In milestone 0 it was a stand-in we controlled.
- **Loopback:** connections to `127.0.0.1`, meaning to other programs running on the same computer.
- **Trust entry:** a line in an agent's own settings that marks a folder as trusted, which makes the agent load that folder's own settings files.
- **Hook:** a command an agent runs by itself at set moments (start of session, before each tool). A hook is an ordinary program.
- **Permission mark (ACL):** the list on a Windows folder that says which accounts may read or write it.
- **Journal:** Steplet's own record of each change, written before the change happens (R18).
- **WSL:** Windows Subsystem for Linux, a Linux system running inside Windows.

---

## 1. Start from the receiver

**Who:** a developer who received a steplet from another developer (row 2: developers first). The typical file is use case 1: "check out this repository at this commit, run its tests, send me a report". Use case 5, the attacker's version, looks exactly the same from the outside: its test helper tries to read a private key and send it out.

**What they want:** the tests run and a report comes back. Nothing else happens: no secret leaves the laptop, nothing outside the folder changes, and if something goes wrong the run stops.

**What they see and touch:** the preview (the plan, the rating, who signed it), the run chooser (each runner listed with the concrete things it blocks, R6), the approve button, a live log, and the final report.

**Where they get stuck, from what was measured:**
- **No runner on their machine counts as "contained".** Under R2, a file that runs tests may only run contained; otherwise the app says "cannot run safely on this computer". Codex non-admin does not meet R2's definition (F4). Codex admin meets it only if D1 says so. Claude Code in WSL needs WSL2, a Linux install of Claude Code, as set up here the `socat` package (installed with `sudo`), and a sign-in inside the app's own Claude folder, because that is where the sign-in landed (P4.1 to P4.4).
- **Codex admin mode's one-time setup needs administrator rights**: it adds two Windows accounts, a group and four firewall rules (T1b-direct).
- **Codex admin mode cannot start tools installed inside the user's own folder**, such as a default Python install (F10). The run would end with "tests did not run".
- **Codex cannot check out a commit from inside its fence** (F12), which is step one of use case 1.
- **An interrupted run that is resumed repeats a step** (F46).
- **A checker that stops answering stalls the run for good** unless the app has its own timer (F36).

---

## 2. What each runner may honestly promise

R1 splits the promise in two, and the screens keep them apart:
- **(a) What Steplet's checker stops:** the agent's own proposed actions (commands, file edits, web fetches), checked before they happen.
- **(b) What the operating system stops:** the programs the agent starts (the tests, scripts, their child programs).

Each line below is allowed on the screen only because a test passed. Where no test passed, the screen says nothing, or says the gap.

### 2.1 Claude Code 2.1.220 in WSL

Measured with the full required set of options (handoff: own Claude folder, no auto-update, `--setting-sources user`, `--strict-mcp-config`, our own `--settings` file with `denyRead`, `--permission-prompt-tool`).

| The screen may say | Evidence |
|---|---|
| (a) "Steplet checks the agent's shell commands that change something, its file writes and its web fetches before they happen. If Steplet's checker crashes, stalls or answers wrongly, the action is refused." | T1c gate log; T2a crash, T2c garbage and wrong shape, T2d killed: all refused (F33, F38, F39). T2b stall: refused when the app stops the run (F37). T2e WebFetch (F40) |
| (a) "The agent can fetch only these web addresses: *list*." | T2e: WebFetch asked the checker and was refused (F40) |
| (b) "Programs this task runs can change files only inside *this folder*." | T1c: outside write failed, `Read-only file system` |
| (b) "Programs this task runs cannot read *these paths*." Only the paths Steplet lists. | T1c (Linux side), T1c-b1 (`/mnt/c`): both blocked when listed |
| (b) "Programs this task runs cannot connect to the internet, or to other programs running in WSL." | T1c: internet `Network is unreachable`; loopback inside WSL `Connection refused`; listener got 0 connections (F28). The listener ran inside WSL; a listener on the Windows side was not tried |
| (b) "Programs this task runs cannot start Windows programs." | T1c: attempt G failed, no marker file. Which part of the fence did it is not known (F28) |
| (b) The same holds for programs those programs start. | T1c: both child attempts blocked |

Qualifiers the screen must carry with the first (a) line: commands that Claude Code itself treats as read-only (`ls`, `echo`, `cat`) run without asking the checker; they still run inside the fence (F34, and see D8). Only Bash, Write and WebFetch were tested against the checker; the agent's other tools were not.

**Must not say:**
- "Cannot read your files." The whole Windows disk is readable unless listed (T1c-b2, F31; in T1c, attempt H read `C:\Windows\win.ini` through `/mnt/c`).
- "Cannot connect to anything on this computer." Only loopback inside WSL was tested.
- "Leaves nothing behind outside the folder." It created the receiver's global git ignore file (F49).
- "Picks up where it left off." Resume repeated a step, 3 of 3 (F46).
- "Done" because the run ended `success`. A run ended `success` with the file not created (F35).
- "Model X did this", read from the start of the run. The model can change mid-run (F32, see D7).
- Anything the agent says about what happened. Its account was wrong in T3a (F43).

**Only true if Steplet does these** (required by the findings or by settled rows; not open):
- Write `denyRead` for every always-blocked path (row 24), on the Linux side **and** under `/mnt/c` (F31, R4), plus a matching `permissions.deny` rule for the agent's own Read tool (that rule was not exercised, see section 5).
- Put Steplet's own agent folders on that list, because row 24 covers agent credential files. In milestone 0 the settings file blocked only the fake-key folder, so the Claude sign-in in `~/m0/claude-home` was, by F31's default, readable by fenced programs. Not tested.
- Offer the agent only tools whose checker behaviour has been tested. 2.1.220's `--help` lists `--tools` ("Specify the list of available tools from the built-in set"); its behaviour was not tested.
- Put its own deadline on every checker decision (F36), and keep no in-memory state in the checker, because Claude Code restarts it after a crash (F33, F39).
- Keep WebFetch on the refuse-unless-allowed list (F40).
- Judge the outcome from the disk, not the agent's words or the result type (F3, F35).
- Show the agent's final answer as "the agent says", never parse it for status (F47); strip control characters after decoding the event stream, not only from raw bytes (T5b note); scrub secrets from every text field of an event (F48).
- Never let child programs inherit the input stream the app uses to talk to the agent (F1).
- Re-run the hostile suite on every Claude Code version it supports (R20): two flags it depends on are hidden in 2.1.220 (F27).

### 2.2 Codex 0.160.1, admin mode (`elevated`), with a profile Steplet writes

| The screen may say | Evidence |
|---|---|
| (b) "Programs this task runs can change files only inside *this folder*." | T1b-direct, T1b-agent: outside write `Permission denied` |
| (b) "Programs this task runs cannot read *these paths*." Only listed paths. | T1b-direct, profile `m0`: key read `Permission denied`, also for the child. (T1b-agent agrees, but by then F9's lasting deny mark blocked every admin run whatever its profile, so it is not separate evidence.) |
| (b) "Programs this task runs cannot connect to the internet." | T1b: `WinError 10013`, blocked by Codex's firewall rules |

Nothing under (a): no checker was measured for Codex. `codex exec` never asks anyone (F14); the app-server approval route is milestone 1.

**Must also say, if it is offered at all:**
- To block a path, Codex puts a lasting "deny" permission mark on that folder, which stays after the run (F9).
- Setting up admin mode once adds two Windows accounts, a group and four firewall rules (T1b-direct). Separately, Codex's first sandbox run (non-admin, in milestone 0) created a folder `.sbx-denybin` in the user's folder (F6).

**Must not say:**
- "No network" or "cannot connect to other programs on this computer." Loopback is open (F7, see D1).
- "Cannot read your other files." Any folder ordinary Windows users can read is readable unless listed (F8, attempt H). The user's own folder is closed only because of Windows' default permissions, not because of Codex.
- Codex's own description of its fence. It said `workspace-write` and network off while the fence was leaking or switched off (F15, F17).

**Only true if Steplet does these:**
- Keep a received repository's `.codex/` files from loading. Without that, the first run switched the fence off (T3c, see D2).
- Keep Codex's own sign-in out of reach of fenced programs. Checked 8 Oct: `C:\dev\m0\codex-home\auth.json` (a real credential) carries `BUILTIN\Users:(RX)` and `NT AUTHORITY\Authenticated Users:(M)`, inherited from `C:\dev`, and the sandbox accounts were added to `Users` (T1b-direct). By F8, admin-mode fenced programs could most likely read it; non-admin ones certainly could (F4). Not tested.
- The same output handling as 2.1 (T5: no forged events; control characters come back after decoding; the key appears unmasked when a read is let through, F24).

### 2.3 Codex 0.160.1, non-admin mode (`unelevated`)

| The screen may say | Evidence |
|---|---|
| (b) "Programs this task runs can change files only inside *this folder*." | T1a-direct, T1a-agent |

Nothing else. Reads are open everywhere; asking for a read block makes it refuse to start (F5). Network is "off" only for programs that obey proxy settings; a direct connection to the internet and to loopback got through, while Codex reported network off (F4, F5, F15).

This mode does not meet R2's definition of contained (programs are OS-restricted), so it is never offered for a file that installs, builds, tests or runs scripts. **R2's own text names "Codex's native Windows sandbox" as contained without saying which mode**, so that row needs a correcting line (section 3, last part). If this mode is ever listed for a file that runs no programs, its only line is the write promise.

### 2.4 Side by side

| | Claude Code in WSL | Codex admin | Codex non-admin |
|---|---|---|---|
| Write outside the folder | blocked | blocked | blocked |
| Read a listed path | blocked | blocked (lasting deny mark, F9) | refuses to start (F5) |
| Read an unlisted path | open (F31) | open if Windows users can read it (F8) | open |
| Internet | blocked | blocked | **open** (F4) |
| Loopback | blocked inside WSL (F28); Windows side not tried | **open** (F7) | **open** |
| Start a Windows program | blocked (F28) | n/a | n/a |
| Received repo's own settings, no counter-measure | **hooks run outside the fence** (F41) | **fence switched off** (F17) | not tested (F20 shown in admin mode only) |
| Same, with the counter-measure | kept out (F45) | kept out (F18, F21), through `codex exec` only | not tested |
| Checker crash, stall, bad reply | refused (T2a–T2d) | not measured (milestone 1) | not measured |
| Resume after interruption | repeats a step (F46) | not tested | not tested |
| Check out a commit inside the fence | works (F30) | fails (F12) | fails (F12) |
| Contained under R2's definition | yes | **open: D1** | no |

---

## 3. Decisions milestone 0 leaves for you

D1 to D7 are the ones the handoff named. D8 came up while writing this: a settled row collides with a measurement. Each item gives what was measured, the options in plain words, what goes wrong with each, and a recommendation. **None is decided here.** After D8: how they connect, and which settled rows the results contradict.

### D1. Loopback is open in Codex admin mode (F7)

**Measured:** in admin mode a fenced program connected to a listener on `127.0.0.1`, parent and child, in both the direct test and the agent test (T1b-direct, T1b-agent). Codex had created a firewall rule to block exactly that, and it did not take effect; the likely reason, not verified, is that Windows Firewall does not filter loopback. Claude Code in WSL closed loopback inside WSL (T1c).

**Why it matters:** anything listening on the receiver's computer becomes reachable from the hostile program: a local database, a dev server, Ollama, a VPN or proxy client (which forwards to the internet, so this is use case 5's "phone home" again), Docker's control port if it is switched on, a browser's debugging port. Also Steplet itself, if the app ever listens on loopback.

| Option | What goes wrong |
|---|---|
| **A. Accept it and say it.** Codex admin counts as contained; the chooser says "cannot reach the internet; can connect to other programs on this computer". | The receiver cannot judge what is listening on their own machine. One local proxy turns it into a full leak. It goes against R2's reasoning that a warning does not repair missing containment, and against the plan's own pass rule. |
| **B. Not contained until loopback is shown closed.** Codex admin is not offered for files that run code. Without WSL2, the app says "cannot run safely on this computer". | Fewer receivers can run use cases 1, 2 and 5 in version 1; they need WSL2. All of R27's version 1 use cases that use an outside agent are contained ones, so the Codex adapter (rows 8c, 39) would have no use case in version 1 until D or E passes. |
| **C. Contained only if a check right before the run finds nothing listening.** | A snapshot is not a fence: a service can start mid-run. On a typical Windows machine some system services listen on all addresses, which includes loopback (not measured here), so the check would almost always fail anyway. |
| **D. Steplet closes loopback itself** for Codex's sandbox accounts, at the layer underneath Windows Firewall (the Windows Filtering Platform), then proves it with T1b. | Not tried; not known to work. Needs administrator rights, either at every run or through a background service with full system rights, which breaks the per-user, no-admin install (row 32), unless it is installed once during Codex's own admin setup. A filter tied to Steplet's session would vanish if Steplet crashed, reopening loopback mid-run. It is keyed to Codex's account names; if a later Codex renames them, the block silently stops. Codex's own runner might use loopback itself. (These four points are general knowledge, not verified.) |
| **E. Test Codex's `mxc` engine.** It is already listed in 0.160.1's configuration reference (results, "What the installed version says"); it was skipped in milestone 0 by your decision, not because it was missing. | Unknown whether it closes loopback, or what it changes on the machine. |

**Recommendation: B for version 1, with D and E as short, measured trials in milestone 1.** B is the only option where every promise on screen is backed by a passed test today. If either trial passes T1b with loopback blocked, Codex admin moves to contained with no change to the screen wording. A and C hand the safety judgment to the receiver. Also add to the suite a listener on the Windows side for the WSL runner, so both runners are held to the same test.

### D2. Codex: a received repository's `.codex/` files, and Codex trusting folders by itself (F16, F17, F20, F21)

**Measured:**
- `codex exec` writes `trust_level = "trusted"` for its working folder into its own settings file, with no prompt (F16): in 9 of the last 9 runs in an untrusted folder. 4 earlier runs did not; why is not known. It writes the folder path in lower case (`[projects.'c:\dev\m0\repo']`, checked 8 Oct).
- Once trusted, the repo's `rules/default.rules` loaded. Its `allow` rule ran `git` and `python` **outside the fence** on the very first run: the key was read, a file written outside, the internet reached, and the key sent to the listener (T3c, F17, F20). Codex's own record still said `workspace-write`.
- `--ignore-user-config` kept the repo's files out, trusted or not (T3d, T3f, F18). It also drops Steplet's own permission profiles (so the read blocks in 2.2 are lost), and Codex still writes a trust entry into the file it was told to ignore.
- An explicit `trust_level = "untrusted"` entry written before the run kept the repo out and kept our profiles (F21). **One run only.**
- `--ignore-rules` would stop repo rules but also drops Steplet's own `forbidden` rules, which R5 relies on.
- Repo hooks never ran without the bypass flag (F19); when allowed, they ran outside the fence (F22).
- **All of this was measured through `codex exec`.** R5 commits the Codex adapter to the app-server route, where none of T3c to T3f was run.

| Option | What goes wrong |
|---|---|
| **A. Always pass `--ignore-user-config`.** | Steplet's own read blocks are lost, unless profiles can be passed on the command line under that flag (not tested). A trust entry still lands in the ignored file; if that Codex settings folder is ever used without the flag, the folder is trusted. |
| **B. Steplet owns a private Codex settings folder**, writes an explicit `untrusted` entry for the run folder in the exact form Codex writes (lower case), puts the file back from a known copy before every run, watches it during the run and stops the run if it changes. | Measured once only; a different letter case was not tested. Depends on Codex keeping to its own trust rule in later versions. |
| **C. A and B together, with profiles passed on the command line.** | Only possible if a milestone 1 test shows profiles still apply under `--ignore-user-config`. Unknown today. |
| **D. Steplet prepares a fresh working copy without the repo's `.codex/` folder, every run.** | Changes what the repo contains for the run, so the preview and report must say so. It holds only at start-up: fenced code can write the folder back inside the workspace (for example with `git show`, since `.git` is readable), so a reused working copy would carry it into the next run. Covers only file names we know; a later Codex version could read others. Good as a second layer, weak alone. |

Whatever is chosen, the private Codex folder (with the sign-in) must sit where fenced programs cannot read it, be on the always-blocked list (row 24), and the hostile suite should try to read it (see 2.2).

**Recommendation: B and D together, as two independent layers, with T3c to T3f and the `untrusted` follow-up in the hostile suite for every Codex version, run through the app-server route the adapter actually uses.** Test C in milestone 1; if it works, it can replace B, because it does not depend on Codex reading a trust entry correctly. If you choose D1-B, Codex does not run received code in version 1, but D1's trials need this decided.

### D3. Claude Code: a received repository's hooks and allow rules (F41, F44, F45)

**Measured:**
- In a folder that was never trusted, the repo's `.claude/settings.json` hooks ran: `SessionStart` before the agent did anything, and `PreToolUse` three times. They wrote into a folder the fence makes read-only, so they ran **outside the fence**, as the user, with no checker (T3a, F41). This contradicts handoff fact 3, V2 and R4 for 2.1.220.
- The repo's allow rules were ignored while the folder was untrusted. Once the folder was trusted, they took Steplet's checker out of the loop completely; only the fence kept that run contained (T3a-trusted, F44).
- `--setting-sources user` kept every repo file out, trusted or not (T3b, T3b-trusted, F45). Claude Code did not trust a folder by itself (F29).
- Hook runs show up in the event stream as `system/hook_started` and `system/hook_response` (T3a).

The handoff already makes `--setting-sources user` required for every contained run. What is open: whether that one option is enough.

| Option | What goes wrong |
|---|---|
| **A. The option alone** (with the private Claude folder, `--strict-mcp-config` and Steplet's own `--settings`), checked by the hostile suite on every version. | One option on one tested version stands between a received repo and running any command as the receiver with no fence. If a version changes what it means, or adds a repo file type it does not cover, only the suite catches it, and only if the suite tests that file type. |
| **B. A, plus a preview line** listing the agent settings files the repo carries (`.claude/`, `.mcp.json`, `CLAUDE.md`, and the Codex ones) and saying Steplet tells the agent not to load them. | More code; the list of file names can go out of date. It tells, it does not block. It must not say "will not be loaded": only `.claude/settings.json` and `.mcp.json` were tested; `CLAUDE.md`, `settings.local.json`, skills, agents and commands were not. |
| **C. A, plus a fresh working copy without the repo's `.claude/` and `.mcp.json`.** | Changes what the repo contains for the run; preview and report must say so. In WSL `.git` is writable (F30), so fenced code can restore the files with git mid-run; whether Claude Code re-reads settings mid-run was not tested. A second layer only, unless D6 makes `.git` read-only there too. |
| **D. Refuse any repo that carries agent settings with hooks.** | Blocks ordinary repositories and pushes receivers to run them outside the app, the copy-paste bypass R9 warns about. |
| **E. A tripwire:** any hook event in a contained run's stream stops the run and marks the adapter broken, as row 38 already does for actions that skip the checker. | Cheap, but it fires after the hook has run once. A detector, not a block. |

In every option Steplet should also check after each run that its private Claude folder recorded no trust for the run folder, since trust is what let the allow rules in (F44).

**Recommendation: A, C and E together, B's preview line worded as above, and the trust check after each run.** This is the same layered pattern as D2: the agent is told not to load repo settings, the files are not there to load, and if a hook runs anyway the run stops.

### D4. A resumed run repeats its last step (F46, R18)

**Measured:** in T4 the run was killed the moment `STEP1` was written, then resumed. All 3 of 3 ended `STEP1, STEP1, STEP2`. Reason: Claude Code saves a command before running it and the result only after; a kill in between leaves a command with no result, and the resumed agent runs it again (F46). Codex has `codex exec resume`; it was not tested.

R18 already decides the journal. What is open: what "resume" means for an agent run in version 1.

| Option | What goes wrong |
|---|---|
| **A. No agent resume for contained runs.** After an interruption, Steplet throws the working copy away, prepares a fresh one, and offers "run again from the start". | Work is lost and repeated. A restart also repeats every effect outside the folder, such as allowed web fetches, so "no duplicates" holds only where those are harmless to repeat. It changes R20's fourth test from "resumes without duplicate effects" to "starts again clean"; that needs a correcting row. |
| **B. Resume the agent, and let the checker refuse any command that matches one the journal shows as started but not confirmed**, asking the receiver. | The checker sees commands, not their effects. The agent can word the same command differently, and a script can repeat an effect inside the fence where the checker cannot see it. It catches only exact repeats, so it looks like a guarantee and is not one. |
| **C. Resume, and tell the agent what already happened** (a note from the journal in the resume prompt). | Relies on the model obeying. Helpful, not a guarantee. |
| **D. Resume only steps the sender marks as safe to repeat.** | New field in the file format; senders can mark wrongly and the checker cannot verify the claim. |

Why A uses a fresh copy and not the journal: the journal records what the agent and Steplet do, not what fenced programs write (R1, F34). Files a hostile program planted, such as `.claude/`, `.codex/` or `.git/hooks`, would survive a journal-based restore.

Use case 4 (backup, "resumes") is not affected: there Steplet's own code copies the files and keeps the journal (R3), so no agent repeats anything.

**Recommendation: A for agent runs in version 1.** Resume stays where Steplet's own code makes the changes (use cases 3 and 4). The receiver sees "interrupted; the folder was reset; run again?". How long a restart costs on real repositories is unknown: milestone 0's runs were 10 to 57 seconds, but on a one-test fixture.

### D5. Codex admin mode cannot start tools installed inside the user's own folder (F10)

**Measured:** Python under `C:\Users\RANVIR\AppData\Local\Programs` failed to start with `Access is denied`. The fenced program runs as a separate Windows account that cannot enter the user's folder. Milestone 0 worked around it with a 35.2 MB copy of Python in `C:\dev\m0\py`. The same Windows permissions are what keep the user's private files closed in this mode (F8), so opening them has a cost.

| Option | What goes wrong |
|---|---|
| **A. Give the sandbox accounts read-and-run access to exactly the tool folders a file needs**, shown on the preview, recorded in the journal, removed after the run (and at next start if the app crashed). | Steplet changes permissions inside the receiver's own folder. A missed clean-up leaves access open (F9 shows such marks stay). Per-user package folders may be needed too, so the list grows. |
| **B. Steplet supplies its own copies of common tools** in a place the sandbox can read, downloaded with pinned versions and checked fingerprints. | Large downloads, a catalogue of tools and versions to maintain, may not match what a repo needs, and "the app downloads programs" is its own trust question. |
| **C. Require tools installed for all users** (under Program Files). Before the run, the preview checks and says "Codex cannot reach your Python; install it for all users, or use Claude Code in WSL". | Friction; many developers will not reinstall, which in practice rules Codex out for them. Even an all-users Python may still look for per-user package folders, and Microsoft Store Python shortcuts would probably fail too (both not tested). |
| **D. Copy the tool for each run** (what milestone 0 did). | Copies can break (missing parts, fixed install paths), are large, and need special handling per tool. |

In every option: a check before the run that the fence can start each needed program, taken from R8's list of programs resolved to absolute paths, so the receiver gets a clear reason instead of "tests did not run".

**Recommendation: C plus the before-run check for version 1.** Steplet changes no permissions in the receiver's folder. Revisit A or B if Codex admin becomes contained (D1) and receivers hit this often. If you choose D1-B, this matters only after a D1 trial passes.

### D6. `.git` is read-only inside the Codex fence (F12)

**Measured:** in both Codex modes, git could not create `.git/index.lock`, so the agent could not check out a commit. Other git commands that write inside `.git` would likely fail the same way (not tested). In Claude Code's WSL fence, `.git` is writable and checkout worked (F30).

Why Codex probably blocks it (not verified): a program that can write inside `.git` can plant git hooks or settings that run later, **outside any fence**, the next time anyone runs git in that folder, including the receiver or Steplet itself.

| Option | What goes wrong |
|---|---|
| **A. Steplet's own code does the checkout, unfenced, before the agent starts**: a fresh clone every run, a fixed git command (absolute path, R8, a pinned git version), with git's ways of running repo-supplied programs switched off (hooks, the file-system monitor, filters), the receiver's own git settings ignored, and no submodules or large-file downloads. The agent only runs the tests, with `.git` read-only for both runners where the fence can express it (for Claude Code, not tested). | More app code. Git runs on hostile content with no fence, so any git security hole is open; the list of settings to switch off must be right (not verified here). A steplet that needs the agent to commit or branch cannot do that. |
| **B. Make `.git` writable inside the fence.** | Opens the planted-hook path above: whoever runs git there next runs what the hostile program left, unfenced. |
| **C. Run in a fresh copy of the files at that commit, with no `.git` at all.** | Tests that call git (for example, to read a version tag) fail. |
| **D. Steplet clones without checking out (unfenced, hardened as in A), then runs the checkout inside the fence as its own step, with no agent and no repo programs; the agent's step follows with `.git` read-only.** | Two fenced launches. Works in WSL today (F30). For Codex it needs a profile that lets that one step write `.git`, which was not tested. |

In every option: a working copy whose `.git` was writable during a run must never be handed back to the receiver as a folder to run git in.

**Recommendation: D where the runner supports it (Claude Code in WSL now; Codex if a milestone 1 test of a checkout-only profile passes), otherwise A.** Either way the checkout appears on the preview as a step Steplet does, not the agent. The git-hardening list itself needs checking in milestone 1; the items above come from the architect review's general knowledge.

### D7. Claude Code switches model silently after a refusal (F32)

**Measured:** in 6 of the 20 single Claude Code runs (T1c-b1, T1c-b2, T3a, T3b, T3a-trusted, T3b-trusted), the provider refused a request as "cyber", and Claude Code switched from `claude-opus-5[1m]` to `claude-opus-4-8` and finished with `success`. Only a `model_refusal_fallback` event in the stream shows it. `--help` lists `--model` and `--fallback-model` (described for an overloaded or unavailable model); whether either controls this switch was not tested. The provider stays the same, so R16's "which provider, which data leaves" does not change.

| Option | What goes wrong |
|---|---|
| **A. Allow it, and never let it be silent:** the adapter watches for the event, and the live log and report say "the model changed from X to Y after the provider refused a request". The refusal itself is shown too, as a hint that the content may be security-related. | The receiver approved one model and got another, though they are told. If a later version renames the event, only the suite catches it, and only if it includes a refusal case. |
| **B. Treat it as a stop:** the run ends "stopped: the model provider refused", and the receiver may run again. | Repositories with security-related content, use case 5 included, would stop often. In T1c-b1 and T1c-b2 the switch came while the agent wrote its final answer, so those runs would have lost their report. |
| **C. Switch it off with a setting**, if one exists, and then treat a refusal as B. | Not known to exist; needs a milestone 1 test. |
| **D. Ignore it.** | The report names the wrong model. |

**Recommendation: A for version 1, and test C in milestone 1.** Hostile-suite results should also record every model that ran, since milestone 0's results came from a mix.

### D8. Commands that skip the checker, against row 38's stop rule (F34)

Not in the handoff's list; found while writing this.

**Measured:** some shell commands never reached the checker: `echo ok`, and an `ls … || echo` check (T2a), and an `ls … && cat tests/fixtures.py` (T3a). They ran inside the fence. Every command that wrote something did reach the checker. This happened with `autoAllowBashIfSandboxed: false` set. Claude Code's rule for which commands skip was not verified (F34).

**The collision:** row 38 says "if the event stream ever shows an action that skipped the gate, stop the run and mark the adapter broken". Applied as written, almost every Claude Code run would stop on its first `ls`.

| Option | What goes wrong |
|---|---|
| **A. Keep row 38 as written.** | Runs stop constantly; the adapter is unusable. |
| **B. Exempt shell commands that ran without asking, and rely on the fence for them.** The stream's tool-use events, not the checker's log, are the record of what ran; skipped commands are shown in the live log. | The (a) promise becomes "commands that change something", not "every command". Because Claude Code's rule is unknown, a command it wrongly treats as read-only could change something inside the folder unchecked. The fence still holds (T1c), but the checker's list is incomplete. |
| **C. Find a setting that makes every command ask**, then keep row 38 as written. | Not found in 2.1.220 (the one tried, `autoAllowBashIfSandboxed: false`, did not do it). Needs a milestone 1 test; may not exist. |
| **D. Exempt only a fixed list of commands Steplet itself judges harmless**, and treat any other skipped command as a row 38 stop. | Steplet's list and Claude Code's rule can drift apart between versions; the suite must test both. |

**Recommendation: D, with C tested in milestone 1.** D keeps row 38's tripwire for everything except commands Steplet itself has judged, rather than trusting Claude Code's unknown rule; if C turns out to exist, use it and drop the list.

### How the decisions connect

- **One piece of app code serves D2-D, D3-C, D4-A and D6:** Steplet prepares a fresh working copy itself (clone at the exact commit with hardened git, without the agents' settings files) and prepares it again after an interruption. If you choose all four, milestone 1 builds it once.
- **D1 sets how much D5 and the Codex half of D6 matter in version 1.** With D1-B, Codex runs no received code in version 1. They are still worth deciding so that D1's trials have a target.
- **D3-E and D8 both lean on row 38's tripwire**: decide them together.

### Settled rows that the results contradict

These are not re-opened here. They are rows whose wording a measurement now contradicts, or which a recommendation above would change. The correcting lines were recorded in `docs/decisions.md` as C1 to C7 on 8 Oct 2026, after the user's approval.

| Row | What it says | What was measured | Correcting line needed |
|---|---|---|---|
| R2 | Contained runners include "Codex's native Windows sandbox" | Non-admin mode does not restrict reads or network (F4); admin mode leaves loopback open (F7) | Name the mode; admin mode depends on D1 |
| R4, V2, handoff fact 3 | Project-folder hooks are inert in headless mode without trust | They ran, outside the fence (F41) | Replace with "`--setting-sources user` is required" (F45) and D3's outcome |
| V6 | Both Codex modes prevent network access | Non-admin: direct connections get through (F4, F5). Admin: loopback gets through (F7) | Correct per mode |
| Row 38 | Any action that skipped the gate stops the run | Read-only commands skip it routinely (F34) | Depends on D8 |
| Row 40 | Codex runs with `--ignore-user-config --ignore-rules` | `--ignore-rules` drops Steplet's own `forbidden` rules (help text), which R5 needs; `--ignore-user-config` drops our profiles (F18) | Mark replaced by D2's outcome |
| R20, test 4 | An interrupted run resumes without duplicate effects | Agent resume repeats a step (F46) | Only if you choose D4-A: "an interrupted run starts again clean" |
| Rows 8c, 39 | Codex adapter in version 1 | With D1-B it has no version 1 use case | Only if you choose D1-B |

---

## 4. Measured numbers

The handoff asked milestone 0 for the first measured data behind the effort estimates. **No app code was written, so nothing here measures how long building takes; the architect estimates (for example "six to ten extra weeks" for browser control, row 33) stay judgments.** What was measured:

| What | Number | Source |
|---|---|---|
| Agent runs recorded | 24 `codex exec` runs; 20 single Claude Code runs; 3 kill-and-resume pairs (6 runs) | results file |
| Time per Codex run (16 timed; 8 one-word probe runs not timed) | 10 to 57 s, middle value 20.5 s, 359 s in total | `codex exit code … seconds` lines |
| Time per Claude Code run (16 normal) | 15 to 44 s, middle value 23.5 s, 422 s in total | `claude exit code … seconds` lines |
| Claude Code runs with a stalled or killed checker | 4 runs, stopped by our timer at 300, 301, 600 and 601 s; Claude Code never gave up by itself (F36) | T2b, T2d |
| Model use per Codex run | input 14,692 to 92,163 tokens (most of it reused from cache), output 5 to 1,078 tokens | `TURN COMPLETED usage` lines |
| Turns per Claude Code run | 3 to 6 | `num_turns` |
| Installs | Codex: 2 packages in 1 min. Claude Code: 2 packages in 58 s. Checker stand-in: 94 packages in 7 s. Python copy for Codex admin: 35.2 MB | P3, P4.3, step 1 |
| Changes a runner made outside the test folders | Codex admin setup: 2 accounts, 1 group, 4 firewall rules. Codex's first sandbox run (non-admin): `.sbx-denybin` with 4 files of 26 bytes. Claude Code: 1 file, `~/.config/git/ignore` | T1b-direct, F6, F49 |
| Codex trusting its own folder | 9 of the last 9 runs in an untrusted folder; 4 earlier runs did not | F16 |
| Silent model switch | 6 of 20 single Claude Code runs | F32 |
| Elapsed time of the tests | First control 03:44 local, 7 Oct; last agent test 14:03 UTC (19:33 local), 7 Oct: about 16 hours, across several sessions, including sign-ins and approvals. Elapsed time, not working hours | P2.1, T5 |
| Findings | 49 (F1 to F49) | results file |
| Documented or planned claims the tests contradicted | 5: project hooks inert without trust (V2, R4, fact 3; F41); Codex non-admin blocks network (V6; F4, F5); Codex admin blocks network (V6; loopback open, F7); a WSL program can start Windows programs unless the extra filter is installed (docs; F28, safer than documented); the plan's `codex exec` approval flag does not exist (step 2) | as cited |

What these numbers do say about effort:
- **Re-running the hostile suite for one new agent version** took at least 6 minutes of machine time for Codex (8 runs were not timed, so this is a lower bound) and 37 minutes for Claude Code, as run here. 30 of those 37 minutes were four deliberate checker-stall tests; with the app's own timer they can be much shorter. Reading and recording the results is extra.
- **Run times come from a one-test fixture** (middle value about 20 seconds). They say how long the agents take to start, read and report, not how long a real repository's tests take.
- **The documentation was wrong or silent on points that changed the design five times**, plus several it never mentioned (F16, F36, F46). R20's per-version suite is recurring work for every supported agent version and belongs in the plan's budget next to the code.

---

## 5. Not verified, carried forward

**Scope of every result**
- One machine (Windows 11 Home, build 26200), one run per test (three for T4), one version per agent (Codex 0.160.1, Claude Code 2.1.220), and one model per agent (Codex `gpt-6-luna`; Claude `claude-opus-5[1m]`, with `claude-opus-4-8` after refusals). Windows 10, Windows Pro, antivirus products, and other machines were not tested.

**Codex**
- The `mxc` engine (skipped by your decision; D1-E).
- Why loopback is open (F7): Windows Firewall skipping loopback is the likely reason, not verified.
- Why 4 early runs wrote no trust entry (F16).
- F21 (explicit `untrusted`) on more than one run, and with the path written in different capital letters.
- Every repo-files result (T3c to T3f, F21) through the app-server route that R5 commits to; all were through `codex exec`.
- Whether the rules escape (F20) also works in non-admin mode.
- Whether keys in a repo's `config.toml` other than the three tried change anything when trusted.
- Whether permission profiles passed on the command line still apply under `--ignore-user-config` (D2-C).
- Whether `--ignore-user-config` also skips `$CODEX_HOME\hooks.json` (F23).
- Whether fenced programs can read Codex's own sign-in file (`auth.json` carries `Users:(RX)`; checked 8 Oct, read not tried).
- Whether git commands other than checkout fail on the read-only `.git` (F12), and whether a checkout-only profile can make `.git` writable for one step (D6-D).
- Codex's own file-edit tool, as opposed to commands, against a path outside the folder.
- Which environment variables reach fenced programs (non-admin looked like the whole user environment, not checked one by one; admin not checked). R15 needs this in milestone 1.
- Whether Codex's update check can install a new version mid-run (T3g ran on a day with no newer version).
- The app-server approval flow and what happens when it crashes (milestone 1, as planned).
- Resume (T4) for Codex: `codex exec resume` exists; not run.

**Claude Code in WSL**
- T1d (the extra seccomp filter): skipped. Which part of the fence blocked starting Windows programs (F28).
- A listener on the Windows side, reached from inside the WSL fence. Only loopback inside WSL was tried, and WSL's network mode was not recorded.
- Claude Code's rule for which commands skip the checker, and whether a setting can make every command ask (F34, D8).
- The agent's own Read tool against a blocked path: the `permissions.deny` rule was never exercised, because the agent never tried.
- Whether fenced programs can read Claude Code's own sign-in in the private Claude folder (not on the `denyRead` list in milestone 0; read not tried).
- Whether the checker is asked for the other tools in the run's tool list (WebSearch, Task, Monitor, CronCreate, RemoteTrigger, Workflow and others), and whether `--tools` limits the list as its help text says. Only Bash, Write and WebFetch were tested.
- Repo files other than `.claude/settings.json` and `.mcp.json` under `--setting-sources user`: `.claude/settings.local.json`, `CLAUDE.md`, `.claude/agents`, `.claude/skills`, `.claude/commands`.
- Whether Claude Code re-reads settings files that change mid-run (D3-C).
- Whether `.git` can be made read-only inside the WSL fence (D6).
- Whether hooks in the user's own settings still run under `--setting-sources user` (F41). With Steplet's private Claude folder, those settings are Steplet's.
- Where the repeated copies of program output inside one event come from (F48).
- Why Claude Code writes `~/.config/git/ignore`, and whether it writes anything else outside its folder on other paths (F49).
- Self-update without `DISABLE_AUTOUPDATER=1`.
- Whether `--model` or `--fallback-model` controls the refusal switch (F32).
- Whether Claude Code needs Node 22 at run time for any feature (F26).
- Whether the hidden flags stay in later versions (F27).

**From the architect review, general knowledge, not checked here**
- That a Windows Filtering Platform filter can block loopback for one account, and the four failure modes listed in D1-D.
- The git-hardening list in D6-A (which git settings can run repo-supplied programs, and the known git security holes).
- That an all-users Python still reads per-user package folders, and that Microsoft Store Python shortcuts fail for the sandbox account (D5-C).

**Product questions, unchanged**
- Whether a third-party app may drive a receiver's Claude Code subscription without a screen. Not found in any terms page. It bites at milestone 1.
- Everything on the app's side: the rule checker, journal, secret scrubber, preview, timers. Milestone 1.
- Whether Codex admin mode's account and firewall setup behaves the same on a clean machine.

---

## 6. Housekeeping

- **Clean-up (undo list in `docs/milestone-0-handoff.md`):** not done. It includes deleting two real sign-in credentials (`C:\dev\m0\codex-home\auth.json`, which ordinary Windows users can read, and `~/m0/claude-home` in WSL), removing Codex's Windows accounts, group and firewall rules (needs administrator rights), and `sudo apt-get remove socat libwrap0` (yours to run). **I will ask before any of it.** If D1's trials go ahead in milestone 1, the Codex admin setup and the test folders are useful for them; the sign-in files could still be deleted now and recreated later.
- **Still open with you, unrelated to the tests:** `keys.txt` and `sk.json` should move out of `C:\dev\runFile`; `C:\Users\RANVIR\.claude\settings.json` has a plain-text ServiceNow password inside an `allow` rule.
- **Next:** after you approve this wrap-up and decide D1 to D8, the milestone 1 plan (plan first, wait for approval).
