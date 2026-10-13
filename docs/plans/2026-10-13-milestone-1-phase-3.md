# Milestone 1, phase 3: the WSL runner and the Claude Code adapter — build plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans, run inline in one session (the user's choice since phase 0: not through separate agents). Steps use checkbox (`- [ ]`) syntax for tracking.

Written 13 Oct 2026. Status: **approved by the user on 13 Oct 2026 with the recommended option for H3 and H4 (A, A); H1, H2 and H5 to H7 kept.** Nothing in it had been built, installed or run at approval. The folder layout (decision E3) was chosen by the user on 13 Oct 2026: **Steplet installs and owns its own WSL Linux system** (option D below). This plan is written around that choice.

**Goal:** run a signed `.steplet` file's task with Claude Code inside a Linux sandbox that Steplet owns, so that the stranger's code and the agent's own tools can touch only this run's working copy, and every action the agent proposes is allowed, refused or brought to the receiver by Steplet's one checker (`Policy::check`, built in phase 2).

**Architecture:** two new crates. `steplet-run` (normal Rust, not `no_std`) holds the runner: it prepares a fresh working copy, writes the agent's settings from the checker, starts Claude Code through `wsl.exe`, reads its event stream into Steplet's own event types, answers the agent's permission requests over a Windows named pipe, watches for anything that skipped the checker, and builds the report. `steplet-gate` is a tiny Windows program that Claude Code starts inside Linux (through WSL interop) as its permission-prompt server; it carries no state and simply relays each request to `steplet-run` over the pipe and waits for the answer. The desktop app (phase 4) will drive `steplet-run`; this phase drives it from a test harness instead, with a scripted stand-in model so the tests repeat exactly.

**Tech stack:** Rust 1.98.0 stable (MSVC), edition 2024. `steplet-run` depends on `steplet-core` (the checker and the file core) plus `tokio` and `serde_json` (both already proven in the phase 0 stand-ins). `steplet-gate` depends on `serde_json` only. Windows 11 host; one Linux system that Steplet installs: Ubuntu 26.04 in WSL2 (the only release measured). Claude Code 2.1.220, pinned, installed by Steplet. `srt` (Anthropic's sandbox runtime) 0.0.77, pinned, for the no-agent checkout step. A Python stand-in model and a Python listener as test support only (ported and cleaned from the phase 0 harness). `bash` 5.3 in Steplet's Linux is the shell that runs the agent's commands (confirmed in Task 1).

Words used below, explained once:
- **Working copy:** the folder of the repository's files one run works in. Steplet makes a fresh one per run and deletes it after.
- **Fence (sandbox):** the operating-system limit on what a program can touch, enforced by Claude Code's own sandbox (`bubblewrap` through `srt`). Shell commands run inside it.
- **Checker (`Policy::check`):** phase 2's one function that answers *allow*, *ask the receiver*, or *deny* for each action. The preview, this runner's gate, and the ask screen all call it (decision row 29).
- **Gate:** the path by which the agent's permission request reaches the checker: `steplet-gate.exe` (inside Linux) → a Windows named pipe → `steplet-run` → `Policy::check`.
- **Named pipe:** a Windows-only channel between two programs on the same computer, with a permission list and no network port.
- **Interop:** WSL's way of starting a Windows program from inside Linux.
- **The agent's own file tools:** Read, Write, Edit, Glob, Grep. These run inside Claude Code's process, which is **not** inside the fence (measured, phase 0). So the fence does not limit them; the operating-system layout (option D) and the checker do.
- **Scripted model:** a stand-in for the real model that returns a fixed list of tool calls, so a test run does the same thing every time. It talks to Claude Code through `ANTHROPIC_BASE_URL` (a documented setting).
- **Tripwire:** a check during the run that stops the whole run when something the design forbids is seen (a hook ran, a command skipped the checker, an action skipped the gate).

## Global constraints

Every task includes these. Values are copied from the milestone plan (section 8), the decisions log, and the phase 0 and phase 2 results.

- Rust 1.98.0 stable (MSVC), edition 2024. `cargo` always runs with `--offline`; a crate not in cargo's cache stops the task until the user says yes (as in phase 2).
- `steplet-core` is unchanged by this phase (it stays `no_std` + `alloc`). `steplet-run` and `steplet-gate` are the only new crates, and they are normal `std` crates (they touch the operating system).
- **The one checker (row 29):** the gate's allow/deny/ask answer comes only from `steplet_core::policy::Policy::check`. This phase writes no second matcher.
- **No generated allow rules (R4):** Steplet's settings file contains deny rules only. The permission-prompt tool decides every allow.
- **Launch (C8):** `wsl.exe` is started with `SystemRoot` as its only Windows variable, never an empty environment (an empty one fails) and never the whole environment. The task text goes to Claude Code on its standard input, which is then closed; no child program inherits that input (F1).
- **The run outcome comes from the run, not the agent (F35, F47):** success is judged from the `result` event plus the disk, never from `subtype` alone; the agent's final words are shown as "the agent says" and never parsed for status.
- **Steplet's own deadline on every gate decision (F36):** a decision not returned in time is a refusal. The gate keeps no state between requests (F33).
- **WebFetch is refused unless the file allows the host (F40).**
- **A fresh agent config folder per run,** deleted after the run; no run's state (session transcript, memory, trust) reaches the next run.
- **Steplet never asks for the receiver's password** and makes no change to the receiver's own Windows or Linux outside Steplet's own install folders. (New wording for "never runs sudo"; see choice H4.)
- Evidence: no task is done without its command and its pasted, unchanged output in `docs/milestone-1-phase-3-results.md`.
- Commits go on this worktree's branch, never straight onto `main`. Nothing is pushed unless the user says so. No attribution lines. Plain words in messages. Approving this plan approves the commits it lists.
- `cargo fmt --all -- --check` passes (G16) and `cargo clippy --workspace --all-targets --offline -- -D warnings` is clean, for every task that changes Rust.
- **F12:** every crates.io package in every lock file is at least 14 days old and not yanked; check both lock files with `lock_ages.py` at the wrap-up.
- McAfee on this laptop sometimes locks a freshly built test program (`LNK1104`); build with `--no-run`, retry up to three times, then run.

---

## 1. Who this phase is for, and what they get

### The receiver (the person this phase is for)

**Who:** a developer on Windows 11 with WSL2 who received a `.steplet` file and, in the preview (phase 4), saw a colour they are willing to act on. This phase is what happens after they press Run. They do not see a screen yet; phase 4 draws it. This phase makes the run safe and produces the report the screen will show.

**What they are trying to get done:** one of R27's contained use cases, in milestone 1:
- **Use case 1:** check out a public repository at an exact commit, run its tests, get the result.
- **Use case 2:** run a triage script and get its log.
- **Use case 5:** a file that looks exactly like use case 1 but whose test helper tries to read a private key and send it out. The receiver must be safe without being able to tell it apart.

**What this phase guarantees, and how option D makes it true:**
- The stranger's programs run inside the fence and can change files only inside this run's working copy; they cannot reach the internet except hosts the file declared, cannot reach other programs on the computer, and cannot start Windows programs (all measured in phase 0, re-measured here on the new layout).
- The agent's own file tools run outside the fence, but in Steplet's own Linux the receiver's files are simply **not present** (their Windows drives are not mounted; their own Linux is a different system). So there is nothing of the receiver's for those tools to open, whatever the file tries. This is the whole point of option D.
- Every command that changes something, every file write, and every web fetch is brought to the checker before it happens; a harmless read inside the working copy runs without interrupting the receiver (G2).
- When the run ends (or is stopped, or crashes), the working copy and the run's agent config are deleted, so nothing carries to the next run.

**Where the run can get stuck, and what this phase does:**

| Stuck point | What the runner does |
|---|---|
| Steplet's Linux is not installed yet | First-time setup (choice H3) installs it; until then the app says the file cannot run safely (drawn in phase 4). |
| The declared commit or address is wrong, or there is no network | The clone fails with a clear message naming the address; nothing runs. |
| The agent proposes something the file did not declare | The run pauses; the checker's `Ask` reason is shown to the receiver, bound to that exact request (R7). No answer within Steplet's limit means "no". |
| The gate crashes, stalls, or answers wrongly | The action is refused (F33, F36, F38); the run can be stopped cleanly (F37). |
| A hook fires, or a command skips the checker, or an action skips the gate | The run stops at once and the adapter is marked broken (D3, D8, row 38). |
| The run is interrupted (crash, sleep, Stop) | The working copy is discarded; the report says "interrupted; the folder was reset; run again?" (D4). No resume (D4). |

### You, the builder

You get: Steplet's own Linux system set up and proven (Tasks 1–3, recorded like phase 0); the two crates `steplet-run` and `steplet-gate`; a scripted-model test harness; a results file with every command and its pasted output; and answers to the choices in section 4. The desktop app and its screens are phase 4; this phase ends at a tested runner with a report object.

---

## 2. What this phase delivers, and what it does not

**Delivers** (milestone plan, "### Phase 3", adjusted for option D):
1. Setup for Steplet's own Linux: install the distribution, its tools, the pinned Claude Code and `srt`, the `steplet` account, the layout in section 3 — and the proof that the layout holds (Tasks 1–3).
2. The generated settings file and MCP config from the checker and the run's paths (deny rules only; no allow rules; the no-Read-rule-on-the-work-folder point from E3/C9).
3. The working copy: hardened clone, fenced checkout with no agent, removal of agent settings files, `.git` read-only, discard after a run.
4. The launcher (`wsl.exe`), the stream reader, Steplet's normalised run events, and clean stop.
5. The gate: `steplet-run`'s pipe service and `steplet-gate.exe`, with Steplet's deadline.
6. The tripwires (hook event, skipped-checker command, skipped-gate action).
7. Model-switch reporting (D7/E1); the trust check after each run (D3); the report builder (outcome from the run, F35); output handling (control characters stripped after decoding, secrets scrubbed in every text field).
8. The scripted-model test harness and the containment tests for the items of milestone plan section 7 that this phase covers.

**Does not deliver:**
- the desktop app and any screen (phase 4);
- the public check page (phase 6);
- the full R20 hostile suite's live-model layer beyond one recorded confirmation run, and the gate's new cases in full depth (phase 5 finishes these alongside);
- Codex (phase 7);
- use case 5 published as a public repository (phase 4/launch): this phase uses the existing local fixture;
- the change journal and native-mode use cases 3 and 4 (milestone 2);
- macOS.

---

## 3. The layout: Steplet's own Linux system (decision E3 = option D)

Steplet installs one WSL2 distribution named `steplet`, separate from the receiver's own. Inside it, everything Steplet owns is owned by `root`; the agent runs as a plain account `steplet` that cannot become root and whose password is locked. The receiver's Windows drives are not mounted, and the receiver's own Linux is a different system, so neither is present. This is what closes the agent's file tools without relying on the checker or on file-permission modes Steplet does not control.

### On Windows

| Place | Holds |
|---|---|
| `%LOCALAPPDATA%\Steplet\wsl\` | the distribution's disk image |
| `%LOCALAPPDATA%\Steplet\gate\steplet-gate.exe` | the gate program; the only Windows folder the Linux side can reach |
| `%LOCALAPPDATA%\Steplet\logs\` | the run's event stream and errors (on Windows, outside Linux) |

### Inside the `steplet` distribution

| Path | Owner | Mode | Holds |
|---|---|---|---|
| `/etc/wsl.conf` | root | 644 | `[automount] enabled=false`; `[interop] enabled=true, appendWindowsPath=false`; `[user] default=steplet` (systemd kept on, as the tested distro has it) |
| gate reach | root | — | the gate program made reachable to interop (measured in Task 1: a read-only mount of the Windows gate folder at `/opt/steplet/gate`, or the program copied onto the Linux disk) |
| `/opt/steplet/claude-cli`, `/opt/steplet/srt` | root | 755 | the pinned Claude Code and `srt` |
| `/var/lib/steplet/runs/<id>/settings.json`, `mcp.json` | root | 644 | the run's settings and MCP config, which the `steplet` account cannot rewrite |
| `/var/lib/steplet/runs/<id>/{home,config,tmp}` | steplet | 700 | the run's `HOME`, `CLAUDE_CONFIG_DIR`, `TMPDIR`; fresh each run, deleted after |
| `/work` | root | 755 | holds at most one run's folder at a time |
| `/work/<id>/repo` | steplet | 755 | the working copy; `.git` is chowned to `root` after the fenced checkout |

`/work` is kept apart from `/var/lib/steplet` so a `Read` deny rule can cover the private folder without also covering the working copy (the S6L lesson: a `Read` rule on the work folder refused the agent's own copy).

### The settings file Steplet writes per run

```
sandbox:
  enabled: true, failIfUnavailable: true
  allowUnsandboxedCommands: false, autoAllowBashIfSandboxed: false
  filesystem:
    denyRead:  ["/mnt", "/var/lib/steplet", "/work"]
    allowRead: ["/work/<id>/repo"]          # re-opens the copy inside the denied /work (narrower path wins)
    denyWrite: ["/work/<id>/repo/.git"]
  network:
    allowedDomains: <the file's allow: domains>   # [] when the file declares none
    strictAllowlist: true
permissions:
  deny: ["Read(//mnt/**)", "Read(//var/lib/steplet/**)", "Edit(//work/<id>/repo/.git/**)"]
  # No Read rule on /work (E3, C9): such a rule also refused the agent's own copy.
  # No allow rules at all (R4).
```

The operating-system layout is the real guard; these deny rules are the backup. The `Read(//work/...)`-style rule is deliberately absent, and option D is what makes its absence safe: other runs' copies do not exist (one run at a time, `/work` emptied before launch), and the private folder is both a different owner and on the deny list.

### The launch command (C8)

```
wsl.exe --distribution-id <id> -u steplet --cd /work/<id>/repo --exec \
  /usr/bin/timeout -k 10 <seconds> \
  /usr/bin/env -i \
     HOME=/var/lib/steplet/runs/<id>/home \
     PATH=/usr/local/bin:/usr/bin:/bin \
     LANG=C.UTF-8 \
     CLAUDE_CONFIG_DIR=/var/lib/steplet/runs/<id>/config \
     TMPDIR=/var/lib/steplet/runs/<id>/tmp \
     DISABLE_AUTOUPDATER=1 \
     CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 \
     CLAUDE_CODE_DISABLE_AUTO_MEMORY=1 \
  /opt/steplet/claude-cli/bin/claude \
     -p --output-format stream-json --verbose --include-hook-events \
     --max-turns <n> --setting-sources user --strict-mcp-config \
     --settings /var/lib/steplet/runs/<id>/settings.json \
     --mcp-config /var/lib/steplet/runs/<id>/mcp.json \
     --permission-prompt-tool mcp__gate__decide \
     --tools Bash,Read,Write,Edit,Glob,Grep,WebFetch
```

`wsl.exe` itself gets `SystemRoot` only. The MCP config names the gate program as a `stdio` server. The Anthropic key reaches Claude Code through `apiKeyHelper` only (see choice H2), never in this environment.

---

## 4. Choices for you, each with a recommendation

### H3. Which Linux, and how it is installed, for milestone 1

| Option | What goes wrong |
|---|---|
| **A (recommended). `wsl --install Ubuntu-26.04 --name steplet --no-launch`, then Steplet configures it as root.** Same release as the tested distribution; installable by name on this laptop (checked: it is in `wsl --list --online`). | The exact image can change over time, so two receivers may get slightly different systems. Pinning the image by its hash is a launch-blocker, done before the first public release, not now. |
| B. Import a fixed Ubuntu image pinned by its hash now (`wsl --import`). | More setup to build and host the image; heavier for milestone 1, whose job is to prove the runner, not to ship. |

**Recommendation: A for milestone 1, pin-by-hash before launch.**

### H4. What tools Steplet's Linux has, and the wording of the setup promise

Option D means the receiver's own language tools (their `nvm`, `rustup`, `pyenv`, and anything they installed by hand) are **not** in Steplet's Linux. Steplet must install what a task needs. This is the biggest cost of option D, and it is a product decision.

| Option | What goes wrong |
|---|---|
| **A (recommended). Steplet installs a fixed base at setup** — `git`, `python3`, `build-essential`, `ca-certificates`, `bubblewrap`, `socat` — and, for milestone 1, the runtimes the two launch use cases need to run a repository's tests. A task that needs a tool not present is reported as "this task needs X, which Steplet's runner does not have yet", not run. The list grows in later milestones. | Some real files will not run in milestone 1. The receiver cannot add tools themselves yet. |
| B. Pre-install a wide set of runtimes (several Node and Python versions, Java, Go, …). | A large download and a large attack surface to keep patched, most of it unused for any one file. |
| C. Let the task install its own toolchain inside the fence. | The fence has no network except declared hosts, and installs are the amber case; a toolchain install is itself code, which is what the fence is there to contain. Does not remove the need for a base. |

**Recommendation: A.** Also: because Steplet now acts as root **inside its own Linux only**, the promise changes from "Steplet never runs `sudo`" to **"Steplet never asks for your password and never changes your own Windows or Linux."** The receiver types no setup commands by hand. Tell me if you want the old wording kept instead.

### Smaller choices I made (tell me if you object)

- **H1. Layout details.** `systemd` stays on (the tested distribution has it on). `/mnt/wsl` and `/mnt/wslg` (folders WSL shares between distributions) are unmounted inside Steplet's distribution at start-up if Task 1 finds anything readable there. How the gate is reached (a read-only mount of the gate folder, or the program copied onto the Linux disk) is settled by Task 1's measurement, not by guessing.
- **H2. The Anthropic key.** `apiKeyHelper` is the gate program in a key mode (`steplet-gate.exe --key …`), which asks the app for the key over the same pipe, so the key is **never written to the Linux disk**. If Task 1 finds interop cannot start the gate as `apiKeyHelper`, the fallback is a root-owned key file that only `root` can read, delivered to a `steplet`-readable path for the one moment of the call and covered by a `Read` deny rule; this fallback is recorded and measured (its text must appear zero times anywhere the agent can reach).
- **H5. The agent's tools.** The run offers only `Bash,Read,Write,Edit,Glob,Grep,WebFetch` (`--tools`, which phase 0 S5t showed works). WebSearch, the Task, Cron, Workflow and plan tools, Skill, NotebookEdit and the rest are left out, so the model never sees tools whose checker behaviour has not been tested.
- **H6. The hostile test repository.** Phase 3's scripted tests reuse the existing local fixture (`C:\dev\m0\repo`: a small project whose test helper tries to read a planted canary file, connect to a local listener, and start a Windows program). Publishing it as a public repository for the end-to-end use-case-5 run is phase 4/launch.
- **H7. `.git` after checkout.** `.git` is chowned to `root` after the fenced checkout, so even the agent's out-of-fence tools cannot write it; a `denyWrite` and an `Edit` deny rule stay as backup (D6, S4).

---

## 5. What needs your yes, and when

| # | When | What happens | Size | Undo |
|---|---|---|---|---|
| Z1 | Approving this plan | Steplet installs its own WSL distribution `steplet` (option D), the pinned Claude Code 2.1.220 and `srt` 0.0.77, and the base tools (H4-A), all inside it | 1–2 GB on disk, a download at setup | `wsl --unregister steplet` removes all of it |
| Z2 | Tasks 1–3 | Measured runs inside `steplet`, driven by the scripted stand-in model, plus **one** real-model run for the key path (needs the test-only Anthropic key with its spending cap, P5) | a few cents | delete the run folders |
| Z3 | Any task, only if needed | If cargo asks for a crate not in its cache (`tokio`, `serde_json` and their trees are already cached from the phase 0 stand-ins), the task stops and I ask | n/a | n/a |
| Z4 | Task 1 | If a layout check fails (interop cannot start the gate with the drives unmounted, or the distribution cannot be made without administrator rights), I **stop and ask**; the fallback is the dedicated-account layout (option C), which needs a change to the receiver's own Linux and which you would decide then | n/a | n/a |

---

## 6. How the parts connect

```
steplet-core (phase 1 + 2)         test harness (plays the app, phase 3+5)
  policy::Policy::check ───────┐      ├─ scripted model (ANTHROPIC_BASE_URL)
                               │      ├─ listener (proves the fence blocks it)
steplet-run (new, std) ────────┘      └─ drives steplet-run, answers via Policy::check
  ├─ setup      : install/verify the distribution, tools, pinned Claude Code, srt, the layout
  ├─ prepare    : clone (hardened) → fenced checkout (srt, no agent) → strip agent files → chown .git
  ├─ settings   : write settings.json + mcp.json from the Steplet + run paths (deny rules only)
  ├─ launch     : wsl.exe … ; write the task to stdin; timeout backstop; stop (SIGTERM, then SIGKILL)
  ├─ stream     : read Claude Code's stream-json → Steplet's Event enum
  ├─ gate       : named-pipe service; each request → Policy::check → allow/deny within the deadline
  ├─ tripwires  : hook event / skipped-checker command / skipped-gate action → stop the run
  └─ report     : outcome from the run (not subtype), models + refusals, scrubbed, agent text labelled
steplet-gate (new, std, Windows .exe): started by Claude Code via interop; relays each request over the pipe
```

Main types `steplet-run` defines (exact names the later tasks rely on):

- `RunPaths { distro_id: String, run_id: String, work_repo: String, run_home: String, run_config: String, run_tmp: String, settings: String, mcp: String }` — the Linux paths of section 3, filled per run.
- `RunRequest { steplet: steplet_core::header::Steplet, repo_url: String, commit: String, task_text: String, max_turns: u32, deadline: Duration }`.
- `enum Event { AgentText(String), ToolUse { id: String, name: String, input: serde_json::Value }, ToolResult { id: String, is_error: bool, text: String }, GateDecision { id: String, behavior: String }, SkippedGate { command: String, on_harmless_list: bool }, HookEvent { name: String }, ModelSwitch { from: String, to: Option<String>, category: String }, Result { is_error: bool, stop_reason: String, models: Vec<String> }, Stderr(String) }`.
- `struct GateRequest { run_token: String, tool_name: String, input: serde_json::Value, tool_use_id: String }` and `enum GateAnswer { Allow { updated_input: serde_json::Value }, Deny { message: String } }`.
- `struct Report { completed: bool, commands: Vec<ReportCommand>, refused: Vec<String>, models: Vec<String>, refusals: Vec<Refusal>, changed_files: Vec<String>, agent_says: String, notes: Vec<&'static str> }` (the "blocked attempts are not visible to Steplet" line is one of `notes`).
- `fn decide(policy: &steplet_core::policy::Policy, req: &GateRequest) -> GateAnswer` — the one place the gate turns a request into an answer, by mapping the request to a `policy::Action` and calling `Policy::check`.

`steplet-gate` defines only its command line (`--pipe`, `--token`, `--log`, and `--key`) and the MCP `stdio` protocol it speaks to Claude Code; it holds no logic of its own.

---

## 7. Tasks

Order: 1 → 12. Tasks 1–3 prove the layout and must pass before the runner is built on it (as phase 0's spikes gated the earlier choices). Tasks 4–10 build `steplet-run` and `steplet-gate`. Task 11 assembles the harness and the containment tests. Task 12 wraps up. Each task ends with its command output pasted into `docs/milestone-1-phase-3-results.md`.

The scripted model, the listener, and the run driver are ported from the phase 0 harness (`spikes/tools/fake_model.py`, `spikes/tools/listener.py`, `spikes/wsl/spike.py`) into `crates/steplet-run/tests/support/`, cleaned and kept as test-only code. Nothing from `spikes/` is product code.

### Task 1: build Steplet's Linux and prove the layout holds

**Files:** create `docs/milestone-1-phase-3-results.md`; create `crates/steplet-run/tests/support/setup_steplet.sh` (the setup script, ported and adapted for option D: `wsl.conf`, the `steplet` account, the gate reach, the base tools, the pinned Claude Code and `srt`, the folders of section 3).

This task writes no product Rust. It is measurements, recorded like phase 0, each as a pass/fail. The product gate program does not exist until Task 8, so Checks 1 and 5 use the phase 0 stand-ins (`spike-gate.exe` and `spike-host.exe`, built from `spikes/` with `cargo build --release --offline`), copied into `%LOCALAPPDATA%\Steplet\gate\`. Build the distribution (Z1), then record:

- [ ] **Check 1 — the gate starts with the drives unmounted.** With `[automount] enabled=false`, Claude Code running as `steplet` starts `steplet-gate.exe` through interop and a scripted run's writes all reach the gate (the phase 0 S1 pattern). *Record the gate's own account and its integrity/logon session* (the phase 0 summary flagged these as needed for the pipe's permission list). **If it fails:** copy the gate program onto the Linux disk and retry; if that also fails, **stop and ask** (fall back to option C, Z4).
- [ ] **Check 2 — no administrator rights needed.** A standard Windows account creates and configures the distribution. **If it fails:** stop and ask (Z4); amending row 32's "no admin" is the user's call.
- [ ] **Check 3 — the receiver's files are absent.** As `steplet`: `/mnt/c` does not exist; `mount` shows no Windows drive except the gate reach; the receiver's own Linux home is not present.
- [ ] **Check 4 — shared folders.** Record what `steplet` can read in `/mnt/wsl` and `/mnt/wslg`. **If anything is readable:** unmount them at start-up and re-record (H1).
- [ ] **Check 5 — `bash` is the shell, and its version.** Record `echo "$BASH_VERSION"` as seen by a command the agent runs, so phase 2's command-splitting (which agrees with `bash` 5.3.9) is known to apply.

Pass means checks 1–3 pass and 4–5 are recorded. Paste every command and output under "## Task 1".

### Task 2: the key path, and one real-model run

**Files:** `crates/steplet-run/tests/support/fake_model.py`, `listener.py` (ported); `docs/milestone-1-phase-3-results.md`.

- [ ] **Step 1 — the key is never on the Linux disk.** With `apiKeyHelper` set to `steplet-gate.exe --key …` (H2), run a scripted task that reads files and lists folders. Using the phase 0 S3 method (counts only, the key text never printed), show the key text appears **0 times** in the run config folder, the run home, the event stream, and the error output. **If interop cannot start the gate as the helper:** use the fallback key file (H2) and show the same zero counts; record which path was used.
- [ ] **Step 2 — one real-model run (L1-style, Z2).** Run use case 1 once with the real test key and the real model: the checkout and the test command go through the checker, the fence results match the scripted run, the key count is still 0, and there is exactly one `result` event. Paste the parsed stream and the counts.

### Task 3: the agent's own tools reach nothing but the working copy

**Files:** `crates/steplet-run/tests/support/` test repository fixtures; `docs/milestone-1-phase-3-results.md`.

This is the test that proves option D does its job: because the operating system, not a text rule, is the guard, the agent's out-of-fence tools must find nothing of the receiver's.

- [ ] **Step 1 — shortcut files.** Put, in a test repository, shortcut files that point out of the working copy (to a home path, to a `/mnt` path, to the run's own config folder). Drive the scripted model to open each with Read, Glob and Grep, and to write each with Write and Edit.
- [ ] **Step 2 — record what happened.** Pass means: every read returns nothing outside the working copy and the public system files (the targets do not exist in this Linux, or the deny rule refuses them), and every write outside the working copy fails with a permission error. Record, for each, whether it reached the checker (the phase 0 S5x finding was that the Read tool can skip the checker, which is why the layout, not the checker, must be the guard). Paste the results.

### Task 4: the two crates, the event types, and the gate types

**Files:**
- Create: `crates/steplet-run/Cargo.toml`, `crates/steplet-run/src/lib.rs`, `crates/steplet-run/src/types.rs`; `crates/steplet-gate/Cargo.toml`, `crates/steplet-gate/src/main.rs`.
- Modify: `Cargo.toml` (workspace `members`).

**Interfaces:**
- Produces: the `RunPaths`, `RunRequest`, `Event`, `GateRequest`, `GateAnswer`, `Report`, `ReportCommand`, `Refusal` types of section 6, and `fn decide(policy, req) -> GateAnswer`.
- Consumes: `steplet_core::policy::{Policy, Action, Decision}`.

- [ ] **Step 1: write the failing test** for `decide`, in `crates/steplet-run/tests/decide.rs`:

```rust
use serde_json::json;
use steplet_core::canon::canonicalize;
use steplet_core::header::parse;
use steplet_core::policy::Policy;
use steplet_run::{decide, GateAnswer, GateRequest};

fn policy() -> Policy {
    let f = "---\nsteplet: 1\ntitle: t\nsender: s\nexpires: 2026-12-31\nsteps:\n  - a\nallow:\n  commands:\n    - cargo test\n---\n";
    Policy::from_steplet(&parse(&canonicalize(f.as_bytes()).unwrap()).unwrap(), "/work/r1/repo").unwrap()
}

fn req(tool: &str, input: serde_json::Value) -> GateRequest {
    GateRequest { run_token: "t".into(), tool_name: tool.into(), input, tool_use_id: "u1".into() }
}

#[test]
fn maps_requests_to_the_checker() {
    let p = policy();
    assert!(matches!(decide(&p, &req("Bash", json!({"command": "cargo test"}))), GateAnswer::Allow { .. }));
    assert!(matches!(decide(&p, &req("Bash", json!({"command": "cargo test --release"}))), GateAnswer::Deny { .. }),
        "an Ask from the checker becomes a Deny with the reason, so a run with no receiver cannot auto-allow it");
    assert!(matches!(decide(&p, &req("WebFetch", json!({"url": "https://example.com"}))), GateAnswer::Deny { .. }));
    assert!(matches!(decide(&p, &req("Read", json!({"file_path": "/work/r1/repo/README.md"}))), GateAnswer::Allow { .. }));
}
```

(Note: with no receiver — the test harness and, later, an unattended app — an `Ask` is returned to Claude Code as a `Deny` carrying the reason. The receiver's "yes" on the ask screen is phase 4; there it becomes an `Allow` bound to the exact request, R7. Record this mapping in the task.)

- [ ] **Step 2: run, see it fail** (`decide` not defined). `cargo test -p steplet-run --offline --test decide`.
- [ ] **Step 3: write the types and `decide`.** Map `Bash` → `Action::Command`, `Read`/`Glob`/`Grep` → `Action::Read`, `Write`/`Edit` → `Action::Write`, `WebFetch` → `Action::Fetch`; `Decision::Allow` → `Allow { updated_input: input }`, `Decision::Deny`/`Decision::Ask` → `Deny { message }`.
- [ ] **Step 4: run, see it pass.**
- [ ] **Step 5: commit.**
```
git add crates/steplet-run crates/steplet-gate Cargo.toml docs/milestone-1-phase-3-results.md
git commit -m "The runner crates: event types, gate types, and the checker mapping"
```

### Task 5: the settings and MCP config, from the checker and the run paths

**Files:** create `crates/steplet-run/src/settings.rs`, `crates/steplet-run/tests/settings.rs`; modify `src/lib.rs`.

**Interfaces:** `fn settings_json(s: &Steplet, paths: &RunPaths) -> String` and `fn mcp_json(paths: &RunPaths, pipe: &str, token: &str, gate_exe: &str) -> String`.

- [ ] **Step 1: write the failing test** that asserts the exact JSON of section 3: `denyRead` is `["/mnt", "/var/lib/steplet", "/work"]`; `allowRead` is `["/work/<id>/repo"]`; `denyWrite` is `["/work/<id>/repo/.git"]`; `network.allowedDomains` equals the file's domains; `permissions.deny` is the three rules with **no** `Read(//work/**)` rule and **no** allow rules; the MCP config names the gate as a `stdio` server with the pipe, token and log.
- [ ] **Step 2: run, see it fail.**
- [ ] **Step 3: write `settings.rs`.**
- [ ] **Step 4: run, see it pass.**
- [ ] **Step 5: paste the generated settings for a sample file, and commit.**

### Task 6: the working copy — clone, fenced checkout, strip, discard

**Files:** create `crates/steplet-run/src/prepare.rs`, `crates/steplet-run/src/wsl.rs` (a thin wrapper that runs one command in the distribution and returns status, stdout, stderr); modify `src/lib.rs`. Integration test `crates/steplet-run/tests/prepare.rs`, marked `#[ignore]` so it runs only when asked (it needs the distribution).

**Interfaces:** `fn prepare(req: &RunRequest, paths: &RunPaths) -> Result<(), PrepareError>` does, as `root`: empty and recreate `/work`; hardened clone (the D6 option list: `GIT_CONFIG_NOSYSTEM=1`, empty `HOME`, `core.hooksPath=/dev/null`, `protocol.allow=never`/`https.allow=always`, `transfer.fsckObjects=true`, `--no-checkout --template=`); fenced checkout of the commit under `srt` with no agent (every `srt` call puts `--` before the command, S4); remove `.claude`, `.codex`, `.gemini`, `.cursor`, `.mcp.json` from the copy; `chown -R root:root <repo>/.git`; `chown` the rest to `steplet`.

- [ ] **Step 1: write the integration test** (`#[ignore]`): after `prepare`, `HEAD` is the declared commit; `.git` is owned by root and a `steplet` write into it fails; the agent settings files are gone; `/work` holds exactly one run folder.
- [ ] **Step 2: run it, see it fail** (`prepare` not written). `cargo test -p steplet-run --offline --test prepare -- --ignored`.
- [ ] **Step 3: write `prepare.rs` and `wsl.rs`.**
- [ ] **Step 4: run it, see it pass.** Paste the output.
- [ ] **Step 5: commit.**

### Task 7: the launcher, the stream reader, the events, and stop

**Files:** create `crates/steplet-run/src/launch.rs`, `crates/steplet-run/src/stream.rs`; modify `src/lib.rs`. Unit test `tests/stream.rs` (pure: parse recorded stream lines into `Event`s). Integration test `tests/launch.rs` (`#[ignore]`).

**Interfaces:** `fn parse_event(line: &str) -> Option<Event>` (pure); `fn launch(paths, task_text, deadline) -> RunHandle` and `RunHandle::events()` (a channel of `Event`), `RunHandle::stop()` (SIGTERM via `pkill` on the run's settings-path marker, then SIGKILL after 10 s, with `timeout` as the backstop).

- [ ] **Step 1: write the `parse_event` unit tests** against copied lines of a recorded stream (from `C:\dev\m1\logs\S7a.jsonl` and `S1t.jsonl`): an `assistant` `tool_use` becomes `ToolUse`; a `user` `tool_result` becomes `ToolResult` with `is_error`; a `system` `hook_started` becomes `HookEvent`; a `system` `model_refusal_fallback`/`model_refusal_no_fallback` becomes `ModelSwitch` with the category; the `result` event becomes `Result` with `is_error`, `stop_reason` and the `modelUsage` keys as `models`. A line that is not JSON is `None`, not a panic.
- [ ] **Step 2: run, see it fail.**
- [ ] **Step 3: write `stream.rs`, then `launch.rs`** (the launch command of section 3; `wsl.exe` with `SystemRoot` only; the task on stdin, stdin then closed; no child inherits it).
- [ ] **Step 4: run the unit tests, see them pass.** Then the `#[ignore]` launch test with the scripted model: the event sequence for use case 1 is `ToolUse`(checkout), `ToolResult`, `ToolUse`(tests), `ToolResult`, `Result`; a SIGTERM mid-command ends with one `Result` (`error_during_execution`) and no process left (F37). Paste both.
- [ ] **Step 5: commit.**

### Task 8: the gate service and `steplet-gate.exe`, with the deadline

**Files:** `crates/steplet-run/src/gate.rs` (the pipe service: create the pipe before launch, reject remote clients, one decision per request within the deadline, a broken or late reply is a refusal, no state between requests); `crates/steplet-gate/src/main.rs` (the MCP `stdio` server and the `--key` mode); integration test `tests/gate.rs` (`#[ignore]`).

**Interfaces:** `fn serve(pipe: &str, token: &str, policy: Policy, deadline: Duration) -> GateService` with `GateService::requests()` for the tripwire watcher to see what was asked.

- [ ] **Step 1: write the `#[ignore]` tests** mirroring phase 0 S1: a declared command is allowed and runs (marker exists); an undeclared one is denied and does not; a gate that crashes once fails that request closed and the restart answers the next (F33/F39); a gate that never answers is refused at Steplet's deadline (F36); a wrong run token is refused; the new gate cases — someone holding the pipe name first, a program other than the gate connecting, a restricted-token client — each refused.
- [ ] **Step 2: run, see it fail.**
- [ ] **Step 3: write `gate.rs` and the gate program.** The pipe server is the phase 0 `spike-host` pipe code made into product shape; the gate program is the phase 0 `spike-gate` plus the `--key` mode (H2).
- [ ] **Step 4: run, see the tests pass.** Paste, including the decision time.
- [ ] **Step 5: commit.**

### Task 9: the checker wired in, and the tripwires

**Files:** `crates/steplet-run/src/watch.rs` (the tripwire watcher); modify `src/lib.rs` to join launch + stream + gate + watch into one `run(req) -> Report` (report built in Task 10). Integration tests `tests/tripwires.rs` (`#[ignore]`).

**Interfaces:** `fn run(req: &RunRequest, paths: &RunPaths, policy: &Policy) -> RunOutcome` where `RunOutcome` carries the events, the gate requests, and a `stopped_by: Option<Tripwire>` with `enum Tripwire { Hook, SkippedCommand(String), SkippedGate(String) }`.

The watcher compares each `ToolUse` against the gate's requests: a `Bash` command with no matching gate request is allowed only if `command::assess` reads it as a harmless read inside the repository (G2/D8/E2); any other un-gated `ToolUse`, or any `HookEvent`, stops the run (D3, D8, row 38).

- [ ] **Step 1: write the `#[ignore]` tests** with the scripted model producing: a `Bash ls src` that skips the gate (allowed, no stop); a hook event injected into the run's settings (the run stops, `Tripwire::Hook`); a writing command made to skip the gate (the run stops, `Tripwire::SkippedCommand`). Each checked from `RunOutcome`, not the agent's words.
- [ ] **Step 2: run, see it fail.**
- [ ] **Step 3: write `watch.rs` and `run`.**
- [ ] **Step 4: run, see it pass.** Paste.
- [ ] **Step 5: commit.**

### Task 10: the report, and output handling

**Files:** `crates/steplet-run/src/report.rs`; modify `src/lib.rs` so `run` returns a `Report`. Unit tests `tests/report.rs` (pure, from recorded events) plus an `#[ignore]` end-to-end check.

**Interfaces:** `fn build_report(outcome: &RunOutcome, changed: &[String]) -> Report`.

- [ ] **Step 1: write the unit tests:** `completed` is false when a `Result` is `error_during_execution` or a tripwire fired, even if `subtype` says success (F35); `models` lists every model from `modelUsage` and from `ModelSwitch`, with each refusal (D7/E1); a control character and a planted forged-result string in agent text are stripped/shown only as "the agent says" (F47, T5b); the test key text is scrubbed from every field (F48); `notes` includes "blocked fence attempts are not visible to Steplet" (phase 0 S6).
- [ ] **Step 2: run, see it fail.**
- [ ] **Step 3: write `report.rs`.**
- [ ] **Step 4: run the unit tests, see them pass;** then an `#[ignore]` end-to-end run of use case 1 whose report shows the test command's real exit code and output, and a check that the run's config folder recorded no trust for the run folder (D3) before it is deleted. Paste.
- [ ] **Step 5: commit.**

### Task 11: the containment battery (use case 1 and the hostile fixture)

**Files:** `crates/steplet-run/tests/contain.rs` (`#[ignore]`); `crates/steplet-run/tests/support/` (the fixture copy and its config).

- [ ] **Step 1: use case 1, end to end, scripted model:** the run touches no file of the receiver's, because none are present in Steplet's Linux; the run ends in a `Report` with `completed: true` and the real test exit code.
- [ ] **Step 2: the hostile fixture:** run it; from the fixture's own results file and the listener log (never the agent's words), show every attempt was contained — the planted-file read, the listener connection and the external connection fail inside the fence, and starting a Windows program fails; the working copy is the only thing written.
- [ ] **Step 3: WebFetch** to an undeclared host is refused at the gate; to a declared host it is allowed.
- [ ] **Step 4: paste all three and commit.**

Pass of this task is the phase's "done means" for the milestone plan section 7 lines that phase 3 covers (section 8 below).

### Task 12: wrap-up

**Files:** `docs/milestone-1-phase-3-results.md` (summary), `docs/decisions.md` (E3 recorded as decided, plus any run-time decisions), `docs/handoff.md` and `docs/plans/2026-10-08-milestone-1-plan.md` (status lines), `C:\Users\RANVIR\.claude\projects\C--dev-runFile\memory\project-steplet-state.md`.

- [ ] **Step 1: run every check and paste it:** `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --offline -- -D warnings`; `cargo test --workspace --offline` (the `#[ignore]` integration tests run with `-- --ignored` and are listed separately, since they need the distribution); both lock files through `lock_ages.py`; `rustc --version; cargo --version`.
- [ ] **Step 2: write the summary** — one row per phase 3 "done means" (section 8) with PASS/FAIL and where its output is; then "Changes from the plan" and "What phase 3 did not verify" (section 9 below, updated with anything found).
- [ ] **Step 3: record decisions.** `docs/decisions.md` gets the E3 row settled (option D, the layout of section 3) and a row for H4's wording change and anything decided differently during the run.
- [ ] **Step 4: update the status lines and memory.**
- [ ] **Step 5: commit, then ask the user** whether to fast-forward `main` and whether to push.

---

## 8. Every promise on screen, and the test that backs it (the subset phase 3 covers)

From the milestone plan, section 7. Phase 4 draws the words; this phase makes each true and tested.

| Line on screen | Phase 3 test |
|---|---|
| "Steplet checks the agent's shell commands that change something, its file writes and its web fetches before they happen. If Steplet's checker fails, the action is refused." | Task 8 (crash, stall, wrong reply, wrong token); Task 9 (skipped-command tripwire); Task 11 Step 3 (WebFetch) |
| "The agent can fetch only these web addresses: …" | Task 11 Step 3 |
| "Programs this task runs can change files only inside this folder." | Task 11 Step 2 (outside write fails); Task 6 (`.git` read-only) |
| "Programs this task runs cannot read these paths: …" and the receiver's own files are not even present | Task 3 (the agent's own tools reach nothing outside the copy); Task 2 (the key unreadable) |
| "Programs this task runs cannot connect to the internet, or to other programs on this computer." | Task 11 Step 2 (listener and external both fail); Task 1 Check 4 (shared folders) |
| "Programs this task runs cannot start Windows programs." | Task 11 Step 2 |
| Prepare: "Steplet tells the agent not to load agent settings" | Task 6 (agent files removed); Task 10 (trust check after the run) |
| "Interrupted; the folder was reset; run again?" | Task 7 (SIGTERM ends cleanly); Task 9/10 (a tripwire ends the run) |
| Report: every model used, and refusals | Task 10 |
| Report and live log are safe to read | Task 10 (control characters, forged text, scrubbed secrets) |

---

## 9. What this plan does not verify

- **One machine, one Windows and one WSL version, and mostly the scripted model.** Only one real-model run (Task 2 Step 2). The full live-model layer of the hostile suite is phase 5.
- **The desktop app and its screens** (phase 4); the ask screen that turns a checker `Ask` into a receiver "yes" bound to the request (R7) is phase 4, so this phase maps `Ask` to `Deny`-with-reason.
- **How often a real agent's commands match word for word** under G1; the scripted model uses fixed commands.
- **Toolchains beyond H4-A's base;** files needing other tools are reported as not runnable yet, not run.
- **The pinned image by hash** (H3-B), a launch-blocker, not done now; milestone 1 installs by name.
- **The gate program's new cases in full depth** (pipe squatting, wrong program, restricted token) are tested here at the level phase 0 reached; phase 5 deepens them, including the gate's integrity/logon session recorded in Task 1.
- **`srt`'s own extra write paths** (noted in phase 0 S4) on the new layout: recorded in Task 6, not exhaustively probed.
- **Whether a WSL update can break interop mid-life:** the setup check runs before each run, but a mid-run update is not simulated.
- **The Anthropic terms question (P1)** is unchanged by this phase.
- **Everything the fence cannot report:** blocked attempts are not visible to Steplet (phase 0 S6), so the report says so and never claims the fence stopped a specific thing.
