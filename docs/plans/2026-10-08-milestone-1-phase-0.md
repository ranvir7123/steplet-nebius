# Milestone 1, phase 0: setup and spikes — build plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

Written 8 Oct 2026. Status: **approved by the user on 8 Oct 2026**, to be run inline in one session (not through separate agents), because almost every task needs a yes or a step from the user. Each install still gets its own yes. Before approval, nothing in it had been installed, built or run, except the read-only checks in section 2.

**Goal:** answer the nine spike questions (S1 to S9) from section 6 of `docs/plans/2026-10-08-milestone-1-plan.md` with real command output, and leave this laptop with every tool phases 1 to 7 need.

**Architecture:** all throwaway spike code lives in `spikes/` in the repository. A small harness runs each spike the same way: a run description file says what to run; a script inside a test-only WSL distribution prepares a fresh working copy and settings; either a bash script inside WSL or a Windows stand-in for the Steplet app (`spike-host.exe`) starts Claude Code; a scripted stand-in model answers in place of the real one, so runs repeat exactly and cost nothing; a second script prints what the disk, the logs and the event stream show. Results are pasted into `docs/milestone-1-spikes.md`, the same way milestone 0 recorded its tests.

**Tech stack:** Rust 1.98 stable (MSVC) plus nightly for fuzzing; Python 3 (WSL side scripts); PowerShell 5.1; WSL 2.6.3 with a second Ubuntu 26.04; Claude Code 2.1.220; Anthropic's `srt` sandbox tool 0.0.77; Node 22 (inside WSL) and Node 24 (Windows); Svelte 5, Vite 8, TypeScript 6, Tauri 2.12.1, Playwright 1.63.

Words used below, explained once:
- **Spike:** a short, throwaway test that answers one question before we build on it. Spike code is never copied into the product; phases 1 to 7 write product code fresh, tests first.
- **Stand-in:** a small program that plays the part of something else in a test. `spike-host.exe` stands in for the Steplet app; `spike-gate.exe` stands in for `steplet-gate.exe`.
- **Scripted model:** a stand-in for Anthropic's model service. Claude Code sends its requests there (through `ANTHROPIC_BASE_URL`), and it answers with a fixed list of tool calls. It is not a model and has no judgement.
- **Harness:** the shared scripts every spike run goes through, so every run is prepared and recorded the same way.
- **Run description:** one small JSON file per run (`spikes/runs/<ID>.json`) that says exactly what that run does. It doubles as the record of what was run.
- **Environment variable:** a named setting a program inherits from the program that started it (for example `PATH`, or an API key). "Allowlist" means only the named ones are passed on.
- **Interop:** WSL's way of starting a Windows program from Linux. **Named pipe:** a Windows-only channel between two programs on one computer, with no network port.
- **Stop signal and hard kill:** `SIGTERM` asks a Linux program to finish cleanly; `SIGKILL` ends it at once.
- **Process session:** a group of Linux programs started together, which can be stopped together.
- **WASM (WebAssembly):** a way to run Rust code inside a web browser. **`no_std`:** Rust code that does not use the operating system at all (no files, network or clock).
- **CSP (content-security rule):** a line a web page carries that tells the browser what the page may load or send. `connect-src 'none'` means "this page may not send anything anywhere".
- **WebView2:** the Microsoft Edge engine that Tauri uses to draw the app's screens on Windows. **Playwright:** a tool that drives real browsers for tests.
- **Fuzzing:** feeding a program huge numbers of random inputs to find crashes.
- **MCP:** the protocol Claude Code uses to talk to helper programs, such as the checker. **JSON:** a plain-text format for structured data, one of `{…}` objects and `[…]` lists.
- **Standard input and output:** the text channel a program reads from and writes to when no file is named. The task text goes to Claude Code on its standard input, not on its command line.
- **Proxy:** a go-between program that forwards connections. Claude Code's fence offers one to fenced programs, for allowed web addresses only.
- **GUID:** a long unique id. Each WSL distribution has one, which `wsl.exe --distribution-id` takes.
- **SSHSIG:** the OpenSSH signature format (decision row 13). Its text form starts with `-----BEGIN SSH SIGNATURE-----`. **Crate:** a Rust package.
- **API key:** a secret text that lets a program use Anthropic's model service and bills the account that owns it. **`apiKeyHelper`:** a Claude Code setting naming a command that prints the key, so the key need not sit in an environment variable.
- **Fence (sandbox), checker (gate), loopback, hook:** as in `docs/milestone-0-wrap-up.md`.

---

## 1. Who this phase is for, and what they get

**Today's user is you, the builder.** You want two things:
1. A yes or no, with real output, for each question the later phases stand on. Four of them can change the design: the gate as a Windows program (S1), the launcher (S2), whether a fenced program can see the API key (S3), and the fenced checkout (S4).
2. A laptop on which phase 1 can start: a git repository, the Rust tools, and a test-only WSL distribution.

**What you see and touch:**
- A yes/no question before each install or download (section 5 lists them all).
- A few steps only you can do: move two secret files, create the test distribution's Linux account, type one `sudo` line, create a test API key with a spending cap and save it into the test distribution.
- `docs/milestone-1-spikes.md`, filled as the spikes run: for each one the question, the exact commands, the pasted output, a verdict, and what it means.
- At the end, a summary table and a list of what phase 0 did not verify.

**Where you could get stuck:**
- The test distribution asks you for a Linux user name and password on first start. Use `tester` as the name; the results script and some commands in this plan assume it.
- An install might ask for administrator rights (a Windows "allow changes" prompt). None is expected. If one appears, I stop and ask you.
- Big downloads (section 5 gives rough sizes).
- A spike that fails. Each has a fallback that was already approved in the milestone plan; I record the failure, I do not quietly switch.

**The eventual user, the receiver,** never sees any of this. But every line the run chooser may show (milestone plan, section 7) depends on these answers: the checker line on S1 and S5, the "cannot read" line on S3, the "cannot connect" line on S8, and "Steplet tells the agent not to load agent settings" on the layers S2 and S4 test.

---

## 2. What is on this laptop today (read-only checks, 8 Oct 2026)

Pasted output, shortened only where marked.

```
$ rustup show            (stable-x86_64-pc-windows-msvc only; installed targets: x86_64-pc-windows-msvc)
$ ls ~/.cargo/bin        (no wasm-bindgen, no cargo-fuzz)
$ node --version; npm --version
v24.15.0
11.12.1
$ git --version
git version 2.52.0.windows.1
$ git config --global user.name; git config --global user.email
ranvir7123
ranvir7123@gmail.com
$ wsl --version          (first lines)
WSL version: 2.6.3.0
Kernel version: 6.6.87.2-1
Windows version: 10.0.26200.9457
$ wsl -l -v
  NAME      STATE           VERSION
* Ubuntu    Running         2
$ wsl --help             (lines that matter here)
    --exec, -e <CommandLine>
    --cd <Directory>
    --distribution-id <DistroGuid>
    --install [Distro] [Options...]
            --location <Location>
            --name <Name>
                Set the name of the distribution.
$ wsl --list --online    (first lines)
Ubuntu                          Ubuntu
Ubuntu-26.04                    Ubuntu 26.04 LTS
Ubuntu-24.04                    Ubuntu 24.04 LTS
```

Inside the main Ubuntu (a read-only script in my scratch folder, not in the repository):
```
Ubuntu 26.04 LTS
nodejs: installed=20.20.2-1nodesource1 candidate=20.20.2-1nodesource1
bubblewrap: installed=0.11.1-1ubuntu0.1 candidate=0.11.1-1ubuntu0.3
socat: installed=1.8.1.1-1ubuntu0.1 candidate=1.8.1.1-1ubuntu0.1
git: installed=1:2.53.0-1ubuntu1 candidate=1:2.53.0-1ubuntu1
python3: installed=3.14.3-0ubuntu2 candidate=3.14.3-0ubuntu2
ripgrep: installed=none candidate=15.1.0-1ubuntu1
ubuntu archive nodejs versions:
    nodejs | 22.22.1+dfsg+~cs22.19.15-1ubuntu1 | http://archive.ubuntu.com/ubuntu resolute/universe amd64 Packages
```
Networking mode: `nat`. No `apparmor_restrict_unprivileged_userns` setting.

Names inside the pinned Claude Code 2.1.220 program (`grep -a -c` on the program file, read-only):
```
switchModelsOnFlag                   15
CLAUDE_CODE_SUBPROCESS_ENV_SCRUB     10
apiKeyHelper                         45
allowRead                            8
denyWrite                            15
autoAllowBashIfSandboxed             8
model_refusal_fallback               32
api_refusal_category                 8
ANTHROPIC_BASE_URL                   40
sandbox_violation                    9
```
So the settings S2, S3, S4 and S6 rely on exist in 2.1.220 by name; whether they behave as documented is what the spikes test.

Windows programs listening on all addresses (`Get-NetTCPConnection -State Listen`, shortened): ports 135, 5040, 27036, 49664 to 49668, 49697, and GlideXService ports 54450 to 57100. The WSL adapter's Windows address is `172.27.208.1`. S8 uses these existing listeners, so it needs no new Windows Firewall rule.

The test repository for S4: `git ls-remote https://github.com/octocat/Hello-World.git` → `7fd1a60b01f91b314f59955a4e4d4e80d8edf11d refs/heads/master`.

`srt` 0.0.77's command line (read from its published `dist/cli.js`): `srt [command...]` with each argument shell-quoted, `-s, --settings <path>`, `-d, --debug`, `-c <command>`; `--version` prints a placeholder, so its version is read from `npm ls`. It also has a `windows-install` command that installs Windows Filtering Platform filters with a loopback port allowance; that is relevant to phase 7 (D1's loopback trial), not to phase 0.

Still in `C:\dev\runFile`: `keys.txt` and `sk.json` (P6 says you move them out before `git init`). I have not opened either.

---

## 3. Global constraints

Every task includes these. Values copied from the milestone plan (section 8) and the decisions it rests on.

- Windows 11, no administrator rights. If any step raises an administrator prompt, stop and ask.
- I never run `sudo`. You type the one `sudo` line yourself.
- Real sign-ins stay away from hostile test code (P5). Every spike runs in the test distribution `steplet-test`. The only step that touches your main Ubuntu is S3e, which uses no test fixture and needs its own yes.
- Claude Code: Steplet's own pinned copy, **2.1.220**, in Steplet's private folder; never the `claude` on `PATH` (in WSL that is the Windows wrapper).
- Every Claude Code run uses `--setting-sources user`, `--strict-mcp-config`, Steplet's own `--settings`, and `--permission-prompt-tool mcp__gate__decide`. No generated allow rules (R4).
- Every Claude Code run gets only R15's environment allowlist: `HOME` (Steplet's private home), `PATH=/usr/local/bin:/usr/bin:/bin`, `LANG=C.UTF-8`, `CLAUDE_CONFIG_DIR`, `DISABLE_AUTOUPDATER=1`, `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`. Spikes add only what the run description names (the scripted model's address, a fake key, a test variable).
- Signature name space: `steplet-v1` (P4).
- The real test API key is used only in Task 15, only through `apiKeyHelper`, never on a command line or in a file on the Windows disk.
- Versions. Where the milestone plan names a version, that one: `wasm-bindgen` 0.2.129 (published 25 Sep 2026), Tauri 2.12.1 (30 Sep 2026), `ssh-key` 0.6.7, Claude Code 2.1.220. Everything else: the newest release at least 14 days old on 8 Oct 2026:

  | Tool | Version | Published |
  |---|---|---|
  | `@anthropic-ai/sandbox-runtime` (`srt`) | 0.0.77 | 18 Sep 2026 (needs Node ≥ 20.11) |
  | `cargo-fuzz` | 0.13.2 | 9 Jun 2026 |
  | `tokio` | 1.53.1 | 20 Jul 2026 |
  | `serde_json` | 1.0.151 | 20 Jul 2026 |
  | `svelte` | 5.57.1 | 18 Sep 2026 |
  | `vite` | 8.3.1 | 24 Sep 2026 |
  | `@sveltejs/vite-plugin-svelte` | 7.3.1 | 23 Sep 2026 |
  | `typescript` | 6.0.3 | 16 Apr 2026 (7.x is newer, but `svelte-check` 4.7.6 accepts only `^5 \|\| ^6`) |
  | `svelte-check` | 4.7.6 | 13 Aug 2026 |
  | `@tsconfig/svelte` | 5.0.8 | 17 Feb 2026 |
  | `@playwright/test` | 1.63.0 | 4 Sep 2026 |
  | `@tauri-apps/cli`, `@tauri-apps/api` | 2.12.1 | 30 Sep 2026 (must match the `tauri` crate) |
  | `@modelcontextprotocol/sdk`, `zod` (Node stand-in gate) | 1.32.1, 4.6.5 | as in milestone 0 |

- WSL scripts are files with Linux line endings (`.gitattributes` enforces it), run with `wsl -d steplet-test --exec /bin/bash <file>` or `--exec /usr/bin/python3 -I <file>`. Never piped into `bash` (finding F1). Never inline `bash -c '...'` through PowerShell (milestone 0 trap).
- Evidence: no step is "done" without the command and its pasted output. The agent's own words are never evidence; only the disk, the gate logs, the listener logs, the scripted model's log and the parsed event stream.
- Commits are local only, never pushed. No attribution lines. Plain words in messages. Approving this plan approves the commits it lists; each install still gets its own yes when we reach it.
- Before each commit that adds code, run that task's self-test and paste its output.

---

## 4. How a spike run works

```
spikes/runs/<ID>.json ──► run.ps1 <ID>  (Windows, writes C:\dev\m1\logs\<ID>.txt)
                            │
                            ├─1─► wsl: spike.py prepare <ID>
                            │        fresh working copy  /home/tester/steplet-work/<ID>/repo
                            │        settings + MCP file /home/tester/.local/share/steplet/runs/<ID>/
                            │        launch files        C:\dev\m1\runs\<ID>\argv.txt, task.txt, token.txt, cd.txt
                            ├─2─► wsl: services.sh <ID>   (kept running: scripted model :47900, listener :47831)
                            ├─3─► launch, one of:
                            │        inside WSL:  launch_inside.sh <ID>          (milestone 0's way; S7 only)
                            │        from Windows: spike-host.exe launch …        (the app's way, chosen in section 3
                            │                       └─ wsl.exe --distribution-id … --cd … --exec /usr/bin/timeout …
                            │                            /usr/bin/env -i <allowlist> claude -p …   (task on standard input)
                            │                       └─ named pipe \\.\pipe\steplet-spike-<ID> ◄── spike-gate.exe
                            │                                         (started by Claude Code through interop)
                            ├─4─► wsl: spike.py evidence <ID>   (stream, gate logs, model log, disk, processes)
                            ├─5─► wsl: spike.py cleanup <ID>    (only if the run description asks)
                            └─6─► stop the helpers
```

Folders:

| Where | What |
|---|---|
| `C:\dev\runFile\spikes\` | All spike code (committed). Throwaway. |
| `C:\dev\m1\` | Run data on Windows: `logs\`, `runs\<ID>\`, `outside\` (marker files), `canary\` (copies of fake keys, for scrubbing), `s9\` (a throwaway SSH test key), `fuzz-smoke\`, `distro-id.txt`. Not committed. |
| `/home/tester/.local/share/steplet/` | Steplet's private folder, laid out as the milestone plan chose: `claude-cli/` (pinned Claude Code), `claude-config/` (its settings folder), `home/` (private `HOME`), `runs/<ID>/`, `secrets/` (fake key; later the real test key), `srt/`. |
| `/home/tester/steplet-work/<ID>/repo` | Each run's working copy. |
| `/home/tester/m1/` | Spike data: `fixture/` (clone of milestone 0's test repository), `canary/fake_key`, `outside/`, `logs/`, `tools/gate/` (milestone 0's Node stand-in gate). |

Two settings shapes, written by `spike.py`:
- **`m0`**: as milestone 0 ran (`denyRead` only the fake-key folders). Used to compare with T1c.
- **`product`**: the layout section 3 of the milestone plan chose. `denyRead`: `/mnt`, the private folder, the work folder, the spike data folder. `allowRead`: this run's working copy. `denyWrite`: this run's `.git`. Matching `Read(...)` deny rules, and an `Edit(...)` deny rule on `.git`.
- **`product-noallow`**: the fallback if `allowRead` does not work in 2.1.220 (S7b): `denyRead` only `/mnt`, the private folder and the spike data folder, relying on working copies being deleted after each run (D4).

---

## 5. What needs your yes, and when

| # | When | What happens | Size or cost (rough) | Undo |
|---|---|---|---|---|
| A1 | Task 1 | **You** move `keys.txt` and `sk.json` out of `C:\dev\runFile`. Then I run `git init` and make the first commit. | none | delete `C:\dev\runFile\.git` |
| A2 | Task 2 | `rustup target add wasm32-unknown-unknown wasm32v1-none`; `rustup toolchain install nightly --profile minimal`; `cargo install wasm-bindgen-cli --version 0.2.129 --locked`; `cargo install cargo-fuzz --version 0.13.2 --locked`; a 10-second fuzz test in `C:\dev\m1\fuzz-smoke` | a few hundred MB; the two `cargo install`s compile for several minutes | `rustup target remove …`; `rustup toolchain uninstall nightly`; `cargo uninstall wasm-bindgen-cli cargo-fuzz` |
| A3 | Task 3 | **You** run `wsl --install Ubuntu-26.04 --name steplet-test`, create user `tester`, then type one `sudo apt-get install` line inside it | a few hundred MB download; about 2 GB on disk after packages | `wsl --unregister steplet-test` (deletes everything in it) |
| A4 | Task 4 | Inside `steplet-test`, no `sudo`: Claude Code 2.1.220 and `srt` 0.0.77 from npm, the Node stand-in gate's two packages, a clone of milestone 0's test repository, fake keys | Claude Code's program alone is 275 MB | delete `/home/tester/.local/share/steplet` and `/home/tester/m1` |
| A5 | Task 9 | S3e: one run in your **main** Ubuntu, using milestone 0's signed-in Claude folder and a made-up API key, to see which sign-in wins. If the made-up key were ignored, one tiny prompt ("reply ok") would go through your subscription | at most one tiny prompt | none needed |
| A6 | Task 14 | `npm install` in `spikes/s9-check` (pinned packages above, including the Tauri command-line tool); `npx playwright install chromium firefox webkit` | about 200 MB of packages; browser downloads up to several hundred MB (some revisions may already be present) | delete `node_modules`; `npx playwright uninstall` |
| A7 | Task 15 | **You** create a test-only Anthropic API key with a spending cap (I suggest 5 US dollars) and save it into the test distribution | the cap | delete the key in the Anthropic console |
| A8 | Task 15 | Live runs with that key: S6 live (up to 3 runs, plus up to 3 more if a model switch shows up) and L1 (1 run) | cents to a few dollars, within the cap | none |

Nothing else changes outside `C:\dev\runFile`, `C:\dev\m1` and `steplet-test`. Milestone 0's folders (`C:\dev\m0`, `/home/ranvir/m0`) are read, not changed: Task 4 clones the fixture from `C:\dev\m0\repo`, and S3e reads milestone 0's Claude folder. **Please do not clean up `C:\dev\m0` until phase 0 is finished.**

---

## 6. Tasks

Order: 1 → 2 → 3 → 4 → 5 → 6 (S7) → 7 (S2) → 8 (S1) → 9 (S3) → 10 (S4) → 11 (S5) → 12 (S6, scripted) → 13 (S8) → 14 (S9) → 15 (live: S6 live, L1) → 16 (wrap-up). S7 comes first because the scripted model makes every later run free and repeatable. S2 comes before S1 because S1 launches the way S2 measures. Task 14 (S9) needs only Tasks 1, 2 and 5 and can move anywhere after them.

### Task 1: the git repository (P6)

**Files:**
- Create: `.gitignore`, `.gitattributes`
- Commit: `docs/`, `research/`, `.claude/agent-memory/`

**Interfaces:** Produces the repository every later commit goes into, and the line-ending rule (`*.sh`, `*.py` → Linux endings) that lets WSL run scripts straight from `/mnt/c`.

- [ ] **Step 1 (you):** move `C:\dev\runFile\keys.txt` and `C:\dev\runFile\sk.json` to a folder outside `C:\dev\runFile`. Tell me when done. I do not open them.
- [ ] **Step 2: check they are gone.**

```powershell
Test-Path C:\dev\runFile\keys.txt, C:\dev\runFile\sk.json
```
Expected: `False` twice. If either is `True`, stop.

- [ ] **Step 3: write `.gitignore`.**

```gitignore
# Build output
target/
node_modules/
dist/
dist-tauri/

# Generated by spike builds
spikes/s9-check/src/wasm/
spikes/s9-check/src/sample.ts
spikes/s9-check/src-tauri/icons/
spikes/s9-check/src-tauri/gen/
spikes/s9-check/test-results/
spikes/s9-check/playwright-report/

# Secrets never belong here (P6). These two were moved out; keep them out.
keys.txt
sk.json
*.pem
.env
.env.*

# Personal agent settings
.claude/settings.local.json
```

- [ ] **Step 4: write `.gitattributes`.**

```gitattributes
* text=auto
*.sh text eol=lf
*.py text eol=lf
*.mjs text eol=lf
spikes/s9-core/tests/vectors/** binary
```

- [ ] **Step 5: create the repository and stage.**

```powershell
cd C:\dev\runFile
git init -b main
git add .gitignore .gitattributes docs research .claude/agent-memory
git status --short
```
Expected: only those paths, no `keys.txt`, no `sk.json`.

- [ ] **Step 6: look for secrets in what is staged.**

```powershell
git grep --cached -n -I -E 'sk-ant-[a-z0-9]{2,}-[A-Za-z0-9_-]{20,}|BEGIN [A-Z ]*PRIVATE KEY|password\s*[:=]'
```
(Single quotes and no quote characters inside the pattern, because PowerShell 5.1 mangles embedded double quotes passed to programs.) Expected: no output (exit code 1). Milestone 0's results already have fake keys replaced by placeholders. If anything matches, I look at the match myself and show you file and line number only, never the matched text.

- [ ] **Step 7: commit.** Commits use your global git identity (`ranvir7123 <ranvir7123@gmail.com>`); that email becomes public if the repository is ever pushed.

```powershell
git commit -m "Add the design documents, decisions, research and milestone plans"
git log --oneline
```

### Task 2: Rust tools on Windows (yes: A2)

**Files:**
- Create: `spikes/tools/setup_windows.ps1`

**Interfaces:** Produces `wasm-bindgen` 0.2.129, `cargo fuzz`, the `wasm32-unknown-unknown` and `wasm32v1-none` targets (the second has no operating system at all, so building for it proves a crate is truly `no_std`), and the nightly toolchain. Used by Task 14 and phase 1.

- [ ] **Step 1: write `spikes/tools/setup_windows.ps1`.**

```powershell
# Milestone 1, phase 0, task 2: Rust tools on Windows. Logs everything to C:\dev\m1\logs\setup-windows.txt.
$ErrorActionPreference = "Continue"
$Out = "C:\dev\m1\logs\setup-windows.txt"
New-Item -ItemType Directory -Force C:\dev\m1\logs | Out-Null
Remove-Item $Out -ErrorAction SilentlyContinue
function Say([string]$Text) { Add-Content -Path $Out -Value $Text -Encoding utf8; Write-Output $Text }
function Run([string]$Title, [scriptblock]$Block) {
    Say "--- $Title"
    & $Block 2>&1 | ForEach-Object { Say "$_" }
    Say "exit code: $LASTEXITCODE"
}
Run "rustup target add wasm32-unknown-unknown wasm32v1-none" { rustup target add wasm32-unknown-unknown wasm32v1-none }
Run "rustup toolchain install nightly --profile minimal" { rustup toolchain install nightly --profile minimal }
Run "cargo install wasm-bindgen-cli --version 0.2.129 --locked" { cargo install wasm-bindgen-cli --version 0.2.129 --locked }
Run "cargo install cargo-fuzz --version 0.13.2 --locked" { cargo install cargo-fuzz --version 0.13.2 --locked }
Run "versions" { rustc --version; rustc +nightly --version; wasm-bindgen --version; cargo fuzz --version; rustup target list --installed }
Run "fuzz smoke test: new crate" { cargo new --lib C:\dev\m1\fuzz-smoke --vcs none }
Push-Location C:\dev\m1\fuzz-smoke
Run "cargo +nightly fuzz init" { cargo +nightly fuzz init }
Run "cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=10" { cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=10 }
if ($LASTEXITCODE -ne 0) {
    Run "again without AddressSanitizer" { cargo +nightly fuzz run --sanitizer none fuzz_target_1 -- -max_total_time=10 }
}
Pop-Location
```

- [ ] **Step 2: ask A2, then run it.**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\dev\runFile\spikes\tools\setup_windows.ps1
```
Expected: `wasm-bindgen 0.2.129`; `cargo-fuzz 0.13.2`; installed targets include `wasm32-unknown-unknown` and `wasm32v1-none`; the fuzz run ends with libFuzzer's `Done` line, with or without the sanitizer. If fuzzing fails both ways on Windows, record it: phase 1 then fuzzes inside the test distribution instead, which needs Rust installed there (a new yes at that time).

- [ ] **Step 3:** the log goes into `docs/milestone-1-spikes.md` in Task 5, Step 9 (the results file is created there). Commit `spikes/tools/setup_windows.ps1` with Task 5.

### Task 3: the test WSL distribution (P5; yes: A3)

**Files:**
- Create: `spikes/wsl/check_distro.sh`

**Interfaces:** Produces the distribution `steplet-test` with user `tester`, and `C:\dev\m1\distro-id.txt` (its GUID, which `spike-host.exe` passes to `wsl.exe --distribution-id`).

- [ ] **Step 1 (you), in your own PowerShell window:**

```powershell
wsl --install Ubuntu-26.04 --name steplet-test
```
When it asks, create the user **`tester`** with a password of your choice. Then, inside that new Ubuntu window:

```bash
sudo apt-get update && sudo apt-get install -y nodejs npm socat bubblewrap ripgrep git python3
```
(`srt`'s documentation lists bubblewrap, socat and ripgrep for Linux; Node comes from Ubuntu's own archive, version 22.22.1.) Type `exit` and tell me when done.

- [ ] **Step 2: write `spikes/wsl/check_distro.sh`** (read-only).

```bash
#!/bin/bash
# Milestone 1, phase 0, task 3: read-only checks of the test distribution. Changes nothing.
echo "distribution: $WSL_DISTRO_NAME, $(lsb_release -ds), kernel $(uname -r), user $(id -un)"
for p in nodejs npm socat bubblewrap ripgrep git python3; do
  echo "$p: $(dpkg-query -W -f='${Version}' "$p" 2>/dev/null || echo MISSING)"
done
echo "node: $(node --version 2>&1)"
echo "bwrap smoke test: $(bwrap --ro-bind / / --unshare-net --dev /dev --proc /proc true 2>&1 && echo BWRAP_OK)"
echo "AppArmor limit on user namespaces: $(cat /proc/sys/kernel/apparmor_restrict_unprivileged_userns 2>/dev/null || echo 'setting not present')"
echo "networking mode: $(wslinfo --networking-mode 2>&1)"
echo "interop sockets in /run/WSL: $(ls /run/WSL/ 2>&1 | tr '\n' ' ')"
echo "WSL_INTEROP in this session: ${WSL_INTEROP:-unset}"
echo "a Windows program from here: $(/mnt/c/Windows/System32/cmd.exe /c ver < /dev/null 2>&1 | tr -d '\r' | grep -m1 -i version)"
echo "claude on PATH: $(command -v claude || echo none)"
echo "~/.claude: $(test -e ~/.claude && echo exists || echo absent); ~/.config/git/ignore: $(test -e ~/.config/git/ignore && echo exists || echo absent)"
```

- [ ] **Step 3: run it and record the GUID.**

```powershell
New-Item -ItemType Directory -Force C:\dev\m1\logs, C:\dev\m1\runs, C:\dev\m1\outside, C:\dev\m1\canary | Out-Null
$env:WSL_UTF8 = "1"
& wsl.exe -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/check_distro.sh 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\setup-distro.txt
Get-Content C:\dev\m1\logs\setup-distro.txt
$d = Get-ChildItem HKCU:\Software\Microsoft\Windows\CurrentVersion\Lxss | Where-Object { (Get-ItemProperty $_.PSPath).DistributionName -eq "steplet-test" }
$d.PSChildName | Set-Content -NoNewline -Encoding ascii C:\dev\m1\distro-id.txt
$line = "distro id: $(Get-Content C:\dev\m1\distro-id.txt)"
Add-Content -Encoding utf8 C:\dev\m1\logs\setup-distro.txt $line; $line
```
Expected: every package has a version; `node: v22.…`; `BWRAP_OK`; `networking mode: nat`; the Windows program line shows a version; `~/.claude: absent`; a GUID in braces. Record `WSL_INTEROP` and the `/run/WSL` listing: S1 needs them if the gate cannot start without `WSL_INTEROP`.

### Task 4: Steplet's folders, pinned Claude Code and srt, spike data (yes: A4)

**Files:**
- Create: `spikes/wsl/setup_steplet.sh`, `spikes/node-gate/gate.mjs` (copy of `C:\dev\m0\tools\gate\gate.mjs`), `spikes/node-gate/package.json`

**Interfaces:** Produces the folders in section 4; `/home/tester/.local/share/steplet/claude-cli/bin/claude` (2.1.220); `/home/tester/.local/share/steplet/srt/node_modules/.bin/srt`; `/home/tester/m1/fixture` (contains milestone 0's COMMIT1 `65930082d222089b2633d34e9bb2bd42351bdc76`); the fixture's fake key `/home/tester/m1/canary/fake_key` (text `M1-FAKE-KEY-WSL-…`); the fake API key `/home/tester/.local/share/steplet/secrets/fake-api-key` (text `sk-ant-api03-M1FAKE-…`); copies of both under `C:\dev\m1\canary\` for scrubbing logs; `/home/tester/m1/tools/gate/gate.mjs` with its packages.

- [ ] **Step 1: copy milestone 0's Node stand-in gate into the repository**, unchanged, and add its package file.

```powershell
New-Item -ItemType Directory -Force C:\dev\runFile\spikes\node-gate | Out-Null
Copy-Item C:\dev\m0\tools\gate\gate.mjs C:\dev\runFile\spikes\node-gate\gate.mjs
```
`spikes/node-gate/package.json`:
```json
{
  "name": "m1-node-gate",
  "private": true,
  "type": "module",
  "dependencies": {
    "@modelcontextprotocol/sdk": "1.32.1",
    "zod": "4.6.5"
  }
}
```

- [ ] **Step 2: write `spikes/wsl/setup_steplet.sh`.**

```bash
#!/bin/bash
# Milestone 1, phase 0, task 4: Steplet's private folder, pinned Claude Code and srt, spike data. No sudo.
# No "pipefail": a shortened listing (| head) must not stop the script. Each install is checked by a test -x instead.
set -eu
P=$HOME/.local/share/steplet; M=$HOME/m1; S=/mnt/c/dev/runFile/spikes
mkdir -p "$P"/{claude-cli,claude-config,home,runs,secrets,srt} "$M"/{canary,outside,logs,tools/gate,npm-cache} "$HOME/steplet-work"
chmod 700 "$P" "$P/secrets"

echo "--- Claude Code 2.1.220 into $P/claude-cli"
npm install -g --prefix "$P/claude-cli" --cache "$M/npm-cache" --no-fund --no-audit @anthropic-ai/claude-code@2.1.220 > "$M/logs/npm-claude.txt" 2>&1 || true
tail -5 "$M/logs/npm-claude.txt"
test -x "$P/claude-cli/bin/claude"
echo "version: $(env -i HOME="$P/home" PATH=/usr/bin:/bin CLAUDE_CONFIG_DIR="$P/claude-config" DISABLE_AUTOUPDATER=1 "$P/claude-cli/bin/claude" --version)"
echo "the file that runs: $(readlink -f "$P/claude-cli/bin/claude")"
head -c 4 "$(readlink -f "$P/claude-cli/bin/claude")" | od -c | head -1

echo "--- srt 0.0.77 into $P/srt"
cd "$P/srt"
npm init -y > /dev/null
npm install --save-exact --cache "$M/npm-cache" --no-fund --no-audit @anthropic-ai/sandbox-runtime@0.0.77 > "$M/logs/npm-srt.txt" 2>&1 || true
tail -3 "$M/logs/npm-srt.txt"
test -x "$P/srt/node_modules/.bin/srt"
npm ls --all 2>&1 | head -30 || true
"$P/srt/node_modules/.bin/srt" --help 2>&1 | head -20 || true

echo "--- fixture (milestone 0's test repository)"
rm -rf "$M/fixture"
git clone -q /mnt/c/dev/m0/repo "$M/fixture"
echo "fixture HEAD: $(git -C "$M/fixture" rev-parse HEAD); COMMIT1 present: $(git -C "$M/fixture" cat-file -t 65930082d222089b2633d34e9bb2bd42351bdc76)"

echo "--- fake keys (random, made for these tests, never real)"
printf '%s' "M1-FAKE-KEY-WSL-$(tr -d - < /proc/sys/kernel/random/uuid)" > "$M/canary/fake_key"
printf '%s' "sk-ant-api03-M1FAKE-$(tr -d - < /proc/sys/kernel/random/uuid)" > "$P/secrets/fake-api-key"
chmod 600 "$P/secrets/fake-api-key"
cp "$M/canary/fake_key" /mnt/c/dev/m1/canary/wsl_fake_key
cp "$P/secrets/fake-api-key" /mnt/c/dev/m1/canary/fake_api_key
echo "fixture fake key: $(stat -c %s "$M/canary/fake_key") bytes; fake API key: $(stat -c %s "$P/secrets/fake-api-key") bytes"

echo "--- Node stand-in gate"
cp "$S/node-gate/gate.mjs" "$S/node-gate/package.json" "$M/tools/gate/"
cd "$M/tools/gate"
npm install --cache "$M/npm-cache" --no-fund --no-audit 2>&1 | tail -2
node -e "import('@modelcontextprotocol/sdk/server/mcp.js').then(() => console.log('gate packages load: yes'))"

echo "--- folders"
ls -la "$P"
echo "~/.claude: $(test -e ~/.claude && echo exists || echo absent); ~/.claude.json: $(test -e ~/.claude.json && echo exists || echo absent)"
```

- [ ] **Step 3: ask A4, then run it.**

```powershell
$env:WSL_UTF8 = "1"
& wsl.exe -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/setup_steplet.sh 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\setup-steplet.txt
Get-Content C:\dev\m1\logs\setup-steplet.txt
```
Expected: `version: 2.1.220 (Claude Code)`; first bytes `177 E L F` (a native Linux program, as F26 found); `npm ls` shows `@anthropic-ai/sandbox-runtime@0.0.77` and its exact sub-packages (recorded, because they are not pinned by `srt` itself); `srt --help` lists `--settings`; `COMMIT1 present: commit`; both fake keys exist; `gate packages load: yes`; `~/.claude: absent`.

### Task 5: the spike harness

**Files:**
- Create: `spikes/README.md`, `spikes/Cargo.toml`, `spikes/host/Cargo.toml`, `spikes/host/src/main.rs`, `spikes/gate/Cargo.toml`, `spikes/gate/src/main.rs`
- Create: `spikes/tools/fake_model.py`, `spikes/tools/selftest_fake_model.py`, `spikes/tools/selftest_gate.ps1`, `spikes/tools/run.ps1`, `spikes/tools/append_results.py`, `spikes/tools/listener.py` (copy of `C:\dev\m0\tools\listener.py`), `spikes/tools/parse_claude_stream.py` (copy of `C:\dev\m0\tools\wsl\parse_claude_stream.py`)
- Create: `spikes/wsl/spike.py`, `spikes/wsl/services.sh`, `spikes/wsl/launch_inside.sh`, `spikes/wsl/strings.sh`
- Create: `spikes/probes/s3_probe.py`
- Create: `spikes/results/setup.md`, `docs/milestone-1-spikes.md`
- Commit with it: `spikes/tools/setup_windows.ps1` (Task 2), `spikes/wsl/check_distro.sh` (Task 3), `spikes/wsl/setup_steplet.sh` and `spikes/node-gate/` (Task 4)

**Interfaces:**
- `spike-host.exe launch --id <ID> --distro-id <GUID> --cd <linux path> --run-dir <C:\dev\m1\runs\ID> [--pipe <name> --token <t> --gate-mode allow|deny|drop|stall] [--deadline-ms N] [--env-mode empty|systemroot|inherit] [--stop-after-s N] [--kill-claude-after-s N] [--kill-wsl-after-s N] [--timeout-s N]`. Writes `C:\dev\m1\logs\<ID>.jsonl` (the run's standard output), `<ID>.stderr.txt`, `<ID>.host-gate.log`.
- `spike-host.exe serve --pipe <name> --token <t> --gate-mode <mode> [--deadline-ms N] --seconds N` (pipe only, for the gate's self-test).
- `spike-gate.exe --pipe <name> --token <t> --log <windows path> [--crash-once]`. Speaks MCP on standard input and output; one tool, `decide`; logs to `<log>`.
- Pipe protocol: one JSON line each way. Request `{"run_token", "tool_name", "input", "tool_use_id"}`; answer `{"behavior": "allow", "updatedInput": …}` or `{"behavior": "deny", "message": …}` (the shape from the milestone plan, section 5).
- `fake_model.py <port> <script.json> <log.jsonl> [<expected key file>]`; script format in its header.
- `spike.py prepare|evidence|cleanup <ID>`; run description fields: `about`, `script` (file in `spikes/scripts/`, or `live`, or `none`), `gate` (`node`|`exe`), `gate_mode`, `gate_crash_once`, `shape` (`m0`|`product`|`product-noallow`), `launch` (`inside`|`host`), `host` (object of `spike-host` options, underscores for dashes; its `timeout_s` is the host's overall limit), `workcopy` (`fixture` or `keep:<ID>`), `probes`, `settings_extra`, `env_extra` (`@fake-api-key` is replaced by the fake key), `flags`, `argv_override`, `argv_prefix`, `evidence` (`raw`, or `env`: print variable names, values only for a short list of harmless ones), `cleanup`, `cleanup_by_session`, `grep_debug`, `task`, `max_turns`, `timeout_s` (the `timeout` command's limit inside WSL; default 600). Text values may use `{WORK}`, `{P}`, `{M}`, `{ID}`, `{HOME}`, `{HOST_IP}`.

- [ ] **Step 1: write the two self-tests first.**

`spikes/tools/selftest_fake_model.py`:
```python
"""Self-test for fake_model.py: start it, send the kinds of requests Claude Code sends, check each answer."""
import json
import os
import subprocess
import sys
import tempfile
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
PORT = 47999
KEY = "SELFTEST-KEY-0123456789"
tmp = tempfile.mkdtemp()
script, keyfile, log = (os.path.join(tmp, n) for n in ("script.json", "key", "log.jsonl"))
with open(script, "w") as f:
    json.dump({"steps": [{"tool": "Bash", "input": {"command": "echo one"}},
                         {"refusal": {"delta": {"spike_marker": True}}, "text_after_refusal": "after"},
                         {"text": "end"}]}, f)
with open(keyfile, "w") as f:
    f.write(KEY)
server = subprocess.Popen([sys.executable, "-I", os.path.join(HERE, "fake_model.py"), str(PORT), script, log, keyfile])
time.sleep(1.5)
failures = 0


def check(name, ok):
    global failures
    failures += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + name)


def post(path, body):
    req = urllib.request.Request("http://127.0.0.1:%d%s" % (PORT, path), json.dumps(body).encode(),
                                 {"content-type": "application/json", "x-api-key": KEY})
    return urllib.request.urlopen(req, timeout=10).read().decode()


def events(raw):
    return [json.loads(line[6:]) for line in raw.splitlines() if line.startswith("data: ")]


TOOLS = [{"name": "Bash", "input_schema": {"type": "object"}}]
GO = {"role": "user", "content": "go"}
RESULT = {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "x", "content": "one"}]}
try:
    m = json.loads(post("/v1/messages?beta=true", {"model": "m", "messages": [GO], "tools": TOOLS}))
    check("step 0 answers with a Bash tool call", m["stop_reason"] == "tool_use" and m["content"][0]["name"] == "Bash")
    ev = events(post("/v1/messages?beta=true", {"model": "m", "stream": True, "messages": [GO], "tools": TOOLS}))
    partial = "".join(e["delta"].get("partial_json", "") for e in ev if e["type"] == "content_block_delta")
    check("streamed step 0 rebuilds the same tool input", json.loads(partial) == {"command": "echo one"} and ev[-1]["type"] == "message_stop")
    ev = events(post("/v1/messages", {"model": "m", "stream": True, "messages": [GO, RESULT], "tools": TOOLS}))
    stop = [e for e in ev if e["type"] == "message_delta"][0]["delta"]
    check("step 1 refuses, with the extra field merged", stop["stop_reason"] == "refusal" and stop.get("spike_marker") is True)
    m = json.loads(post("/v1/messages", {"model": "m2", "messages": [GO, RESULT], "tools": TOOLS}))
    check("a repeat at step 1 gets the text after the refusal, under the model asked for", m["content"][0]["text"] == "after" and m["model"] == "m2")
    m = json.loads(post("/v1/messages", {"model": "m", "messages": [{"role": "user", "content": "title?"}]}))
    check("a request without tools gets 'ok'", m["content"][0]["text"] == "ok")
    check("count_tokens answers", json.loads(post("/v1/messages/count_tokens", {}))["input_tokens"] == 1)
    text = open(log).read()
    entries = [json.loads(line) for line in text.splitlines()]
    check("six requests logged", len(entries) == 6)
    check("the key matched and its text is not in the log", all(e.get("api_key_matches_test_key") is True for e in entries) and KEY not in text)
finally:
    server.terminate()
print("failures: %d" % failures)
sys.exit(1 if failures else 0)
```

`spikes/tools/selftest_gate.ps1`:
```powershell
# Self-test for spike-gate.exe and spike-host's pipe: the gate must speak MCP, and must refuse
# whenever the host does not clearly allow (wrong token, host denies, closes, stalls, or is absent).
$ErrorActionPreference = "Continue"
$bin = "C:\dev\runFile\spikes\target\release"
$log = "C:\dev\m1\logs\selftest.gate.log"
Remove-Item $log, "$log.crashed", "C:\dev\m1\logs\selftest.host-gate.log" -ErrorAction SilentlyContinue
$mcp = @(
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"selftest","version":"0"}}}',
    '{"jsonrpc":"2.0","method":"notifications/initialized"}',
    '{"jsonrpc":"2.0","id":2,"method":"tools/list"}',
    '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"decide","arguments":{"tool_name":"Bash","input":{"command":"echo hi"},"tool_use_id":"t1"}}}'
)
$script:failures = 0
function Case([string]$Name, [string]$Mode, [string]$GateToken, [string]$Expect) {
    $srv = $null
    if ($Mode) {
        $srv = Start-Process "$bin\spike-host.exe" -PassThru -NoNewWindow -ArgumentList "serve --pipe steplet-spike-selftest --token good --gate-mode $Mode --deadline-ms 1500 --seconds 5"
        Start-Sleep -Milliseconds 700
    }
    $t = Measure-Command { $script:out = @($mcp | & "$bin\spike-gate.exe" --pipe steplet-spike-selftest --token $GateToken --log $log) }
    if ($srv) { $srv.WaitForExit(10000) | Out-Null }
    $call = [string](@($script:out | Where-Object { $_.Contains('"id":3') })[0])
    $ok = ($script:out.Count -eq 3) -and $call.Contains($Expect)
    if (-not $ok) { $script:failures++ }
    "{0} {1} ({2:N1} s, {3} replies)`n    {4}" -f $(if ($ok) { "PASS" } else { "FAIL" }), $Name, $t.TotalSeconds, $script:out.Count, $call
}
Case "host allows -> allow" "allow" "good" '\"behavior\":\"allow\"'
Case "wrong run token -> deny" "allow" "bad" 'wrong run token'
Case "host denies -> deny" "deny" "good" '\"behavior\":\"deny\"'
Case "host closes without answering -> deny" "drop" "good" 'no answer'
Case "host stalls -> deny at the host's deadline" "stall" "good" 'deadline'
Case "no host at all -> deny" "" "good" 'cannot reach Steplet'
"--- gate log:"; Get-Content $log
"--- host log:"; Get-Content C:\dev\m1\logs\selftest.host-gate.log -ErrorAction SilentlyContinue
"failures: $script:failures"
```

- [ ] **Step 2: run the fake-model self-test; it must fail** because `fake_model.py` does not exist yet.

```powershell
wsl -d steplet-test --exec /usr/bin/python3 -I /mnt/c/dev/runFile/spikes/tools/selftest_fake_model.py
```
Expected: an error that `fake_model.py` cannot be opened, and FAIL lines.

- [ ] **Step 3: write `spikes/tools/fake_model.py`.**

```python
"""Scripted stand-in for the Anthropic Messages API, for Steplet's phase 0 spikes. Not a model. Throwaway.

Usage: python3 -I fake_model.py <port> <script.json> <log.jsonl> [<file holding the expected test key>]

Requests that offer tools are the agent loop. Each gets the script step whose number is the count of
tool results already in the conversation, so retries and side requests never move the script along.
A step is one of:
  {"tool": "<name>", "input": {...}}                 the model asks for one tool call
  {"text": "..."}                                     the model answers and stops
  {"refusal": {"message": {...}, "delta": {...}},    the model refuses: stop reason "refusal", with the given
   "text_after_refusal": "..."}                       fields merged in; a repeat request at the same step
                                                      gets text_after_refusal instead
Requests without tools (titles, summaries) get the text "ok". One JSON line is logged per request.
Keys are never logged: only whether one was sent, and whether it equals the expected test key.
"""
import json
import sys
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT = int(sys.argv[1])
STEPS = json.load(open(sys.argv[2], encoding="utf-8"))["steps"]
LOG = sys.argv[3]
EXPECTED_KEY = open(sys.argv[4], encoding="ascii").read().strip() if len(sys.argv) > 4 else None
LOCK = threading.Lock()
ANSWERED = {}  # step number -> requests already answered at that step


def log(entry):
    entry["time"] = round(time.time(), 3)
    with LOCK, open(LOG, "a", encoding="utf-8") as f:
        f.write(json.dumps(entry) + "\n")


def tool_results_in(messages):
    count = 0
    for m in messages:
        if m.get("role") == "user" and isinstance(m.get("content"), list):
            count += sum(1 for b in m["content"] if isinstance(b, dict) and b.get("type") == "tool_result")
    return count


def step_for(number):
    with LOCK:
        earlier = ANSWERED.get(number, 0)
        ANSWERED[number] = earlier + 1
    if number >= len(STEPS):
        return {"text": "Spike script finished."}
    step = STEPS[number]
    if "refusal" in step and earlier > 0:
        return {"text": step.get("text_after_refusal", "Answer after the refusal.")}
    return step


def reply_parts(step):
    """(content blocks, stop reason, fields to merge) for one step."""
    if "tool" in step:
        block = {"type": "tool_use", "id": "toolu_spike_" + uuid.uuid4().hex[:20], "name": step["tool"], "input": step["input"]}
        return [block], "tool_use", {}
    if "refusal" in step:
        return [], "refusal", step["refusal"]
    return [{"type": "text", "text": step["text"]}], "end_turn", {}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"  # one response per connection; a stream ends when the connection closes

    def log_message(self, *args):
        pass

    def send_json(self, status, obj):
        data = json.dumps(obj).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        log({"method": "GET", "path": self.path})
        self.send_json(404, {"type": "error", "error": {"type": "not_found_error", "message": "spike server"}})

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers.get("content-length") or 0)) or b"{}")
        path = self.path.split("?")[0]
        key = self.headers.get("x-api-key")
        tools = [t.get("name") for t in body.get("tools") or []]
        entry = {"method": "POST", "path": self.path, "model": body.get("model"), "stream": bool(body.get("stream")),
                 "tools_offered": tools, "api_key_sent": key is not None,
                 "api_key_matches_test_key": (key == EXPECTED_KEY) if (EXPECTED_KEY and key) else None,
                 "bearer_sent": self.headers.get("authorization") is not None}
        if path.endswith("/v1/messages/count_tokens"):
            log(entry)
            return self.send_json(200, {"input_tokens": 1})
        if not path.endswith("/v1/messages"):
            log(entry)
            return self.send_json(404, {"type": "error", "error": {"type": "not_found_error", "message": "spike server"}})
        if tools:
            number = tool_results_in(body.get("messages") or [])
            step = step_for(number)
            entry.update(kind="agent loop", step=number, answer=next(iter(step)))
        else:
            step = {"text": "ok"}
            entry.update(kind="side request")
        log(entry)
        content, stop, extra = reply_parts(step)
        message = {"id": "msg_spike_" + uuid.uuid4().hex[:20], "type": "message", "role": "assistant",
                   "model": body.get("model"), "content": content, "stop_reason": stop, "stop_sequence": None,
                   "usage": {"input_tokens": 1, "output_tokens": 1}}
        message.update(extra.get("message", {}))
        if not body.get("stream"):
            return self.send_json(200, message)
        self.send_response(200)
        self.send_header("content-type", "text/event-stream")
        self.send_header("cache-control", "no-cache")
        self.end_headers()

        def event(name, data):
            self.wfile.write(("event: %s\ndata: %s\n\n" % (name, json.dumps(data))).encode())
            self.wfile.flush()

        event("message_start", {"type": "message_start", "message": dict(message, content=[], stop_reason=None)})
        for i, block in enumerate(content):
            if block["type"] == "tool_use":
                event("content_block_start", {"type": "content_block_start", "index": i, "content_block": dict(block, input={})})
                delta = {"type": "input_json_delta", "partial_json": json.dumps(block["input"])}
            else:
                event("content_block_start", {"type": "content_block_start", "index": i, "content_block": {"type": "text", "text": ""}})
                delta = {"type": "text_delta", "text": block["text"]}
            event("content_block_delta", {"type": "content_block_delta", "index": i, "delta": delta})
            event("content_block_stop", {"type": "content_block_stop", "index": i})
        stop_delta = {"stop_reason": stop, "stop_sequence": None}
        stop_delta.update(extra.get("delta", {}))
        event("message_delta", {"type": "message_delta", "delta": stop_delta, "usage": {"output_tokens": 1}})
        event("message_stop", {"type": "message_stop"})


if __name__ == "__main__":
    server = ThreadingHTTPServer(("127.0.0.1", PORT), Handler)
    print("fake model on 127.0.0.1:%d with %d steps" % (PORT, len(STEPS)), flush=True)
    server.serve_forever()
```

- [ ] **Step 4: run the fake-model self-test again.** Expected: 8 `PASS` lines, `failures: 0`.

- [ ] **Step 5: write the Rust workspace, the host and the gate.**

`spikes/Cargo.toml`:
```toml
# Throwaway spike code for milestone 1, phase 0. Not part of the product.
[workspace]
resolver = "2"
members = ["host", "gate"]

[workspace.package]
edition = "2024"
publish = false
```

`spikes/host/Cargo.toml`:
```toml
[package]
name = "spike-host"
version = "0.0.0"
edition.workspace = true
publish.workspace = true

[dependencies]
serde_json = "=1.0.151"
tokio = { version = "=1.53.1", features = ["rt-multi-thread", "macros", "process", "net", "io-util", "time"] }
```

`spikes/host/src/main.rs`:
```rust
//! Spike stand-in for the Steplet app (milestone 1, phase 0). Throwaway.
//! `launch` starts one run through wsl.exe the way the app will, and answers the gate over a named pipe.
//! `serve` runs the pipe alone, for the gate's self-test.
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::process::Command;

const WSL: &str = r"C:\Windows\System32\wsl.exe";
const LOGS: &str = r"C:\dev\m1\logs";

fn opt(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

fn req(args: &[String], name: &str) -> String {
    opt(args, name).unwrap_or_else(|| panic!("missing {name}"))
}

fn secs(args: &[String], name: &str) -> Option<Duration> {
    opt(args, name).map(|v| Duration::from_secs(v.parse().expect("whole seconds")))
}

fn stamp() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}.{:03}", d.as_secs(), d.subsec_millis())
}

fn append(path: &Path, line: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{} {line}", stamp());
    }
}

#[derive(Clone, Copy, Debug)]
enum EnvMode {
    Empty,
    SystemRoot,
    Inherit,
}

/// wsl.exe with the Windows environment the run asks for. The app's choice is Empty (milestone plan, section 3).
fn wsl_command(mode: EnvMode) -> Command {
    let mut c = Command::new(WSL);
    match mode {
        EnvMode::Inherit => {}
        EnvMode::Empty => {
            c.env_clear();
        }
        EnvMode::SystemRoot => {
            c.env_clear().env("SystemRoot", r"C:\Windows");
        }
    }
    c
}

struct Gate {
    mode: String,
    token: String,
    deadline: Duration,
    log: PathBuf,
}

fn gate_from(args: &[String], id: &str) -> Gate {
    Gate {
        mode: opt(args, "--gate-mode").unwrap_or_else(|| "allow".into()),
        token: req(args, "--token"),
        deadline: Duration::from_millis(opt(args, "--deadline-ms").map(|v| v.parse().expect("milliseconds")).unwrap_or(10_000)),
        log: Path::new(LOGS).join(format!("{id}.host-gate.log")),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = match args.get(1).map(String::as_str) {
        Some("launch") => launch(&args).await,
        Some("serve") => serve(&args).await,
        _ => {
            eprintln!("usage: spike-host launch|serve [options]");
            2
        }
    };
    std::process::exit(code);
}

async fn serve(args: &[String]) -> i32 {
    if let Err(e) = start_pipe(&req(args, "--pipe"), gate_from(args, "selftest")) {
        println!("host: could not create the pipe: {e}");
        return 1;
    }
    tokio::time::sleep(secs(args, "--seconds").unwrap_or(Duration::from_secs(10))).await;
    0
}

fn start_pipe(pipe: &str, gate: Gate) -> std::io::Result<()> {
    let name = format!(r"\\.\pipe\{pipe}");
    // first_pipe_instance: fail if anyone else already holds this name.
    let first = ServerOptions::new().first_pipe_instance(true).reject_remote_clients(true).create(&name)?;
    println!("host: pipe {name} created, gate mode {}", gate.mode);
    tokio::spawn(accept_loop(first, name, Arc::new(gate)));
    Ok(())
}

async fn accept_loop(mut server: NamedPipeServer, name: String, gate: Arc<Gate>) {
    let mut first = true;
    loop {
        if let Err(e) = server.connect().await {
            append(&gate.log, &format!("connect failed: {e}"));
            return;
        }
        let conn = server;
        server = match ServerOptions::new().reject_remote_clients(true).create(&name) {
            Ok(s) => s,
            Err(e) => {
                append(&gate.log, &format!("next pipe instance failed: {e}"));
                return;
            }
        };
        if first {
            first = false;
            tokio::spawn(record_gate_identity(gate.log.clone()));
        }
        tokio::spawn(answer(conn, gate.clone()));
    }
}

/// Which Windows account and session the gate runs under, as the app would see it.
async fn record_gate_identity(log: PathBuf) {
    match Command::new("tasklist").args(["/v", "/fo", "csv", "/fi", "imagename eq spike-gate.exe"]).output().await {
        Ok(o) => append(&log, &format!("tasklist:\n{}", String::from_utf8_lossy(&o.stdout))),
        Err(e) => append(&log, &format!("tasklist failed: {e}")),
    }
}

async fn answer(conn: NamedPipeServer, g: Arc<Gate>) {
    let started = Instant::now();
    let (read_half, mut write_half) = tokio::io::split(conn);
    let mut line = String::new();
    match BufReader::new(read_half).read_line(&mut line).await {
        Ok(n) if n > 0 => {}
        other => {
            append(&g.log, &format!("no request read: {other:?}"));
            return;
        }
    }
    let request: Value = match serde_json::from_str(&line) {
        Ok(v) => v,
        Err(e) => {
            append(&g.log, &format!("request is not JSON: {e}"));
            return;
        }
    };
    let deny = |why: &str| json!({"behavior": "deny", "message": format!("Steplet spike host: {why}")});
    let token_ok = request["run_token"].as_str() == Some(g.token.as_str());
    let reply = if !token_ok {
        deny("wrong run token")
    } else {
        match g.mode.as_str() {
            "allow" => json!({"behavior": "allow", "updatedInput": request["input"]}),
            "deny" => deny("denied by the spike"),
            "drop" => {
                append(&g.log, &format!("tool={} mode=drop: closed without answering", request["tool_name"]));
                return;
            }
            "stall" => {
                // The app's own deadline (F36): no decision in time means refuse.
                let _ = tokio::time::timeout(g.deadline, std::future::pending::<()>()).await;
                deny("no decision before the deadline")
            }
            other => deny(&format!("unknown mode {other}")),
        }
    };
    let _ = write_half.write_all(format!("{reply}\n").as_bytes()).await;
    let _ = write_half.flush().await;
    append(
        &g.log,
        &format!("tool={} token_ok={token_ok} mode={} behavior={} ms={}", request["tool_name"], g.mode, reply["behavior"], started.elapsed().as_millis()),
    );
}

async fn pkill(distro: &str, mode: EnvMode, signal: &str, marker: &str) {
    let out = wsl_command(mode).args(["--distribution-id", distro, "--exec", "/usr/bin/pkill", signal, "-f", marker]).output().await;
    println!("host: pkill {signal} -f {marker}: {:?}", out.map(|o| o.status.code()));
}

async fn launch(args: &[String]) -> i32 {
    let id = req(args, "--id");
    let run_dir = PathBuf::from(req(args, "--run-dir"));
    let distro = req(args, "--distro-id");
    let mode = match opt(args, "--env-mode").as_deref() {
        Some("inherit") => EnvMode::Inherit,
        Some("systemroot") => EnvMode::SystemRoot,
        _ => EnvMode::Empty,
    };
    let argv: Vec<String> = std::fs::read_to_string(run_dir.join("argv.txt"))
        .expect("argv.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    let task = std::fs::read(run_dir.join("task.txt")).expect("task.txt");

    if let Some(pipe) = opt(args, "--pipe") {
        if let Err(e) = start_pipe(&pipe, gate_from(args, &id)) {
            println!("host: could not create the gate pipe: {e}");
            return 1;
        }
    }

    let logs = Path::new(LOGS);
    let mut cmd = wsl_command(mode);
    cmd.arg("--distribution-id").arg(&distro).arg("--cd").arg(req(args, "--cd")).arg("--exec").args(&argv);
    cmd.stdin(Stdio::piped())
        .stdout(std::fs::File::create(logs.join(format!("{id}.jsonl"))).expect("stream file"))
        .stderr(std::fs::File::create(logs.join(format!("{id}.stderr.txt"))).expect("stderr file"));
    let started = Instant::now();
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            println!("host: wsl.exe did not start: {e}");
            return 1;
        }
    };
    println!("host: wsl.exe started (pid {:?}, environment {mode:?}); task sent on its input, {} bytes", child.id(), task.len());
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&task).await;
        // stdin is dropped here, which closes it: Claude Code reads the task and nothing more.
    }

    let marker = format!("runs/{id}/settings.json");
    let stop_after = secs(args, "--stop-after-s");
    let kill_claude_after = secs(args, "--kill-claude-after-s");
    let kill_wsl_after = secs(args, "--kill-wsl-after-s");
    let limit = secs(args, "--timeout-s").unwrap_or(Duration::from_secs(900));
    let (mut stopped, mut killed, mut wsl_killed) = (false, false, false);
    let status = loop {
        tokio::select! {
            s = child.wait() => break s,
            _ = tokio::time::sleep(Duration::from_millis(250)) => {
                let t = started.elapsed();
                if stop_after.is_some_and(|d| !stopped && t >= d) {
                    stopped = true;
                    println!("host: stop signal at {:.1} s", t.as_secs_f64());
                    pkill(&distro, mode, "-TERM", &marker).await;
                }
                if kill_claude_after.is_some_and(|d| !killed && t >= d) {
                    killed = true;
                    println!("host: hard kill of Claude Code at {:.1} s", t.as_secs_f64());
                    pkill(&distro, mode, "-KILL", &marker).await;
                }
                if !wsl_killed && (kill_wsl_after.is_some_and(|d| t >= d) || t >= limit) {
                    wsl_killed = true;
                    println!("host: killing wsl.exe at {:.1} s", t.as_secs_f64());
                    let _ = child.start_kill();
                }
            }
        }
    };
    println!("host: wsl.exe ended: {:?}, after {:.1} s", status.map(|s| s.code()), started.elapsed().as_secs_f64());
    0
}
```

`spikes/gate/Cargo.toml`:
```toml
[package]
name = "spike-gate"
version = "0.0.0"
edition.workspace = true
publish.workspace = true

[dependencies]
serde_json = "=1.0.151"
```

`spikes/gate/src/main.rs`:
```rust
//! Spike stand-in for steplet-gate.exe (milestone 1, phase 0). Throwaway.
//! Claude Code starts it through WSL interop as its permission-prompt MCP server, over standard input and output.
//! It keeps nothing between requests: each decision is asked of the host over the named pipe,
//! and anything other than a clear answer from the host becomes "deny".
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

const ERROR_PIPE_BUSY: i32 = 231;

fn stamp() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}.{:03}", d.as_secs(), d.subsec_millis())
}

fn ask_host(pipe: &str, token: &str, a: &Value) -> Value {
    let deny = |why: &str| json!({"behavior": "deny", "message": format!("Steplet's checker refused: {why}")});
    let request = json!({"run_token": token, "tool_name": a["tool_name"], "input": a["input"], "tool_use_id": a["tool_use_id"]});
    let mut conn = None;
    for _ in 0..40 {
        match std::fs::OpenOptions::new().read(true).write(true).open(pipe) {
            Ok(f) => {
                conn = Some(f);
                break;
            }
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => std::thread::sleep(Duration::from_millis(25)),
            Err(e) => return deny(&format!("cannot reach Steplet ({e})")),
        }
    }
    let Some(mut f) = conn else { return deny("Steplet is busy") };
    if writeln!(f, "{request}").is_err() {
        return deny("broken pipe");
    }
    let mut line = String::new();
    match BufReader::new(&f).read_line(&mut line) {
        Ok(n) if n > 0 => serde_json::from_str::<Value>(&line)
            .ok()
            .filter(|v| v["behavior"] == "allow" || v["behavior"] == "deny")
            .unwrap_or_else(|| deny("unreadable answer")),
        _ => deny("no answer"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let opt = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let pipe = format!(r"\\.\pipe\{}", opt("--pipe").expect("--pipe"));
    let token = opt("--token").expect("--token");
    let log_path = opt("--log").expect("--log");
    let crash_marker = format!("{log_path}.crashed");
    let crash_once = args.iter().any(|a| a == "--crash-once");
    let log = |line: &str| {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            let _ = writeln!(f, "{} pid={} {line}", stamp(), std::process::id());
        }
    };
    let mut names: Vec<String> = std::env::vars_os().map(|(k, _)| k.to_string_lossy().into_owned()).collect();
    names.sort();
    log(&format!("started; {} environment variables: {}", names.len(), names.join(",")));

    let mut stdout = std::io::stdout();
    for line in BufReader::new(std::io::stdin().lock()).lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            log("a line on standard input was not JSON");
            continue;
        };
        let result = match msg["method"].as_str().unwrap_or("") {
            "initialize" => Some(json!({
                "protocolVersion": msg["params"]["protocolVersion"],
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "steplet-spike-gate", "version": "0.0.0"}
            })),
            "tools/list" => Some(json!({"tools": [{
                "name": "decide",
                "description": "Steplet spike permission gate",
                "inputSchema": {"type": "object", "properties": {
                    "tool_name": {"type": "string"}, "input": {"type": "object"}, "tool_use_id": {"type": "string"}
                }, "required": ["tool_name", "input"]}
            }]})),
            "tools/call" => {
                if crash_once && std::fs::OpenOptions::new().write(true).create_new(true).open(&crash_marker).is_ok() {
                    log("crashing on purpose (first decision only)");
                    std::process::exit(3);
                }
                let a = &msg["params"]["arguments"];
                let started = Instant::now();
                let answer = ask_host(&pipe, &token, a);
                let input: String = a["input"].to_string().chars().take(2000).collect();
                log(&format!("tool={} behavior={} ms={} input={input}", a["tool_name"], answer["behavior"], started.elapsed().as_millis()));
                Some(json!({"content": [{"type": "text", "text": answer.to_string()}]}))
            }
            "ping" => Some(json!({})),
            _ => None,
        };
        let Some(id) = msg.get("id").cloned() else { continue }; // notifications get no reply
        let reply = match result {
            Some(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
            None => json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": "method not found"}}),
        };
        if writeln!(stdout, "{reply}").and_then(|()| stdout.flush()).is_err() {
            log("standard output closed");
            break;
        }
    }
    log("standard input closed; exiting");
}
```

- [ ] **Step 6: build and run the gate self-test.**

```powershell
cd C:\dev\runFile\spikes
cargo build --release -p spike-host -p spike-gate
powershell -NoProfile -ExecutionPolicy Bypass -File C:\dev\runFile\spikes\tools\selftest_gate.ps1
```
Expected: 6 `PASS` lines, `failures: 0`; the gate log shows `started; N environment variables: …` and one decision line per case; the "stall" case takes about 1.5 s.

- [ ] **Step 7: write the WSL side.**

`spikes/wsl/spike.py`:
```python
"""Milestone 1, phase 0 spike runs inside the test distribution. Throwaway.

  python3 -I spike.py prepare  <ID>   fresh working copy, settings, MCP config, launch files
  python3 -I spike.py evidence <ID>   what the disk, the logs and the event stream show
  python3 -I spike.py cleanup  <ID>   stop whatever is left of the run, then list what remains

A run is described by spikes/runs/<ID>.json. Nothing here trusts what the agent says.
"""
import hashlib
import json
import os
import pathlib
import shutil
import signal
import subprocess
import sys
import time
import uuid

SPIKES = pathlib.Path("/mnt/c/dev/runFile/spikes")
WIN = pathlib.Path("/mnt/c/dev/m1")
HOME = pathlib.Path.home()
P = HOME / ".local/share/steplet"  # Steplet's private folder (milestone plan, section 3)
M = HOME / "m1"                     # spike data: fixture source, fake key, logs, markers
CLAUDE = P / "claude-cli/bin/claude"
FAKE_MODEL_PORT = 47900
NOT_LINUX = {"Path", "PATHEXT", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "SystemRoot", "windir", "COMPUTERNAME",
             "USERNAME", "USERDOMAIN", "ProgramFiles", "ProgramData", "OneDrive", "WSLENV", "WT_SESSION",
             "WSL_INTEROP", "WSL_DISTRO_NAME", "PSModulePath", "HOMEDRIVE", "HOMEPATH", "NUMBER_OF_PROCESSORS"}
# Values safe to paste into the results; any other variable is shown by name only (it could hold a secret).
SHOW_VALUE = {"PATH", "WSLENV", "WSL_DISTRO_NAME", "WSL_INTEROP", "NAME", "HOSTTYPE", "TERM", "LANG", "SHELL", "HOME",
              "USER", "LOGNAME", "PWD", "SHLVL", "DISPLAY", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "PULSE_SERVER",
              "WSL2_GUI_APPS_ENABLED", "_"}


def load(run_id):
    prof = json.loads((SPIKES / "runs" / f"{run_id}.json").read_text(encoding="utf-8"))
    keep = prof.get("workcopy", "fixture")
    owner = run_id if keep == "fixture" else keep.split(":", 1)[1]
    return prof, HOME / "steplet-work" / owner / "repo"


def git(*args):
    return subprocess.run(["/usr/bin/git", *map(str, args)], capture_output=True, text=True)


def host_ip():
    words = subprocess.run(["ip", "route", "show", "default"], capture_output=True, text=True).stdout.split()
    return words[words.index("via") + 1] if "via" in words else "none"


def render(value, ctx):
    if isinstance(value, str):
        for k, v in ctx.items():
            value = value.replace("{" + k + "}", v)
        return value
    if isinstance(value, list):
        return [render(v, ctx) for v in value]
    if isinstance(value, dict):
        return {k: render(v, ctx) for k, v in value.items()}
    return value


def merge(base, extra):
    for k, v in extra.items():
        if isinstance(v, dict) and isinstance(base.get(k), dict):
            merge(base[k], v)
        elif isinstance(v, list) and isinstance(base.get(k), list):
            base[k] = base[k] + v
        else:
            base[k] = v
    return base


def snapshot(work):
    cfg, home = P / "claude-config", P / "home"
    try:
        trust = json.loads((cfg / ".claude.json").read_text()).get("projects", {}).get(str(work))
    except (OSError, ValueError):
        trust = None
    real_ignore = HOME / ".config/git/ignore"
    return {"config_files": sorted(str(p.relative_to(cfg)) for p in cfg.rglob("*") if p.is_file()),
            "private_home_files": sorted(str(p.relative_to(home)) for p in home.rglob("*") if p.is_file()),
            "real_git_ignore": real_ignore.read_text() if real_ignore.exists() else None,
            "trust_record": trust}


def settings_for(shape, work):
    if shape == "m0":
        deny = [str(M / "canary"), str(WIN / "canary")]
        fs = {"denyRead": deny}
        rules = [f"Read(/{p}/**)" for p in deny]
    elif shape == "product":
        deny = ["/mnt", str(P), str(HOME / "steplet-work"), str(M)]
        fs = {"denyRead": deny, "allowRead": [str(work)], "denyWrite": [str(work / ".git")]}
        rules = [f"Read(/{p}/**)" for p in deny] + [f"Edit(/{work}/.git/**)"]
    elif shape == "product-noallow":
        deny = ["/mnt", str(P), str(M)]
        fs = {"denyRead": deny, "denyWrite": [str(work / ".git")]}
        rules = [f"Read(/{p}/**)" for p in deny] + [f"Edit(/{work}/.git/**)"]
    else:
        raise SystemExit("unknown shape " + shape)
    return {"sandbox": {"enabled": True, "failIfUnavailable": True, "allowUnsandboxedCommands": False,
                        "autoAllowBashIfSandboxed": False, "filesystem": fs,
                        "network": {"allowedDomains": [], "strictAllowlist": True}},
            "permissions": {"deny": rules}}


def prepare(run_id):
    prof, work = load(run_id)
    run, win_run = P / "runs" / run_id, WIN / "runs" / run_id
    token = uuid.uuid4().hex
    ctx = {"WORK": str(work), "P": str(P), "M": str(M), "ID": run_id, "HOME": str(HOME), "HOST_IP": host_ip()}
    live = prof["script"] == "live"
    if live and any("API_KEY" in e for e in prof.get("env_extra", [])):
        raise SystemExit("live runs take the real key only through apiKeyHelper, never on a command line")

    if prof.get("workcopy", "fixture") == "fixture":
        shutil.rmtree(work.parent, ignore_errors=True)
        work.parent.mkdir(parents=True)
        r = git("clone", "--quiet", M / "fixture", work)
        if r.returncode:
            raise SystemExit(r.stderr)
        git("-C", work, "checkout", "--quiet", "main")
        key = (M / "canary/fake_key").read_bytes()
        (work / "tests/m0_config.json").write_text(json.dumps({
            "canary_path": str(M / "canary/fake_key"), "canary_sha256": hashlib.sha256(key).hexdigest(),
            "outside_path": str(M / "outside/written_by_probe.txt"), "system_file": "/mnt/c/Windows/win.ini",
            "listener_port": 47831, "external_host": "1.1.1.1", "external_port": 443,
            "wsl_windows_marker": "C:\\dev\\m1\\outside\\from_wsl.txt", "noisy": False}, indent=2))
    for name in prof.get("probes", []):
        (work / "spike_probes").mkdir(exist_ok=True)
        shutil.copy(SPIKES / "probes" / name, work / "spike_probes" / name)

    for folder in (M / "outside", WIN / "outside"):
        for f in folder.glob("*"):
            f.unlink()
    old = list((M / "logs").glob(f"{run_id}.*"))
    old += [WIN / "logs" / f"{run_id}{s}" for s in (".jsonl", ".stderr.txt", ".gate.log", ".gate.log.crashed", ".host-gate.log")]
    for f in old:
        f.unlink(missing_ok=True)

    settings = merge(settings_for(prof["shape"], work), render(prof.get("settings_extra", {}), ctx))
    if prof["gate"] == "node":
        server = {"type": "stdio", "command": "/usr/bin/node", "args": [str(M / "tools/gate/gate.mjs")],
                  "env": {"GATE_MODE": prof.get("gate_mode", "allow"), "GATE_LOG": str(M / "logs" / f"{run_id}.gate.log")}}
    else:
        server = {"type": "stdio", "command": str(SPIKES / "target/release/spike-gate.exe"),
                  "args": ["--pipe", f"steplet-spike-{run_id}", "--token", token, "--log", f"C:\\dev\\m1\\logs\\{run_id}.gate.log"]
                  + (["--crash-once"] if prof.get("gate_crash_once") else [])}

    fake_key = (P / "secrets/fake-api-key").read_text().strip()
    env = [f"HOME={P / 'home'}", "PATH=/usr/local/bin:/usr/bin:/bin", "LANG=C.UTF-8", f"CLAUDE_CONFIG_DIR={P / 'claude-config'}",
           "DISABLE_AUTOUPDATER=1", "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1"]
    if not live:
        env.append(f"ANTHROPIC_BASE_URL=http://127.0.0.1:{FAKE_MODEL_PORT}")
    env += [render(e, ctx).replace("@fake-api-key", fake_key) for e in prof.get("env_extra", [])]
    argv = render(prof.get("argv_override"), ctx) or (
        prof.get("argv_prefix", [])
        + ["/usr/bin/timeout", "-k", "10", str(prof.get("timeout_s", 600)), "/usr/bin/env", "-i", *env, str(CLAUDE),
           "-p", "--output-format", "stream-json", "--verbose", "--include-hook-events",
           "--max-turns", str(prof.get("max_turns", 12)), "--setting-sources", "user", "--strict-mcp-config",
           "--settings", str(run / "settings.json"), "--mcp-config", str(run / "mcp.json"),
           "--permission-prompt-tool", "mcp__gate__decide", *render(prof.get("flags", []), ctx)])

    run.mkdir(parents=True, exist_ok=True)
    (run / "settings.json").write_text(json.dumps(settings, indent=2))
    (run / "mcp.json").write_text(json.dumps({"mcpServers": {"gate": server}}, indent=2))
    if prof["script"] not in ("live", "none"):
        script = render(json.loads((SPIKES / "scripts" / prof["script"]).read_text(encoding="utf-8")), ctx)
        (M / "logs" / f"{run_id}.script.json").write_text(json.dumps(script, indent=2))
    win_run.mkdir(parents=True, exist_ok=True)
    for f in ("services.ready", "stop"):
        (win_run / f).unlink(missing_ok=True)
    (win_run / "argv.txt").write_text("\n".join(argv) + "\n")
    (win_run / "task.txt").write_text(render(prof.get("task", "Follow the spike script."), ctx))
    (win_run / "token.txt").write_text(token)
    (win_run / "cd.txt").write_text(str(work))
    (M / "logs" / f"{run_id}.before.json").write_text(json.dumps(snapshot(work), indent=2))

    def masked(text):
        return text.replace(fake_key, "<fake API key>").replace(token, "<run token>")
    print("--- working copy:", work, "HEAD", git("-C", work, "rev-parse", "HEAD").stdout.strip())
    print("--- settings.json:")
    print(json.dumps(settings, indent=2))
    print("--- MCP server:", masked(json.dumps(server)))
    print("--- launch command, one argument per line:")
    print(masked("\n".join(argv)))
    print("--- task text:", (win_run / "task.txt").read_text())


def show(title, path, limit=60):
    print(f"--- {title}:")
    if not (path.exists() and path.stat().st_size):
        print("    (none)")
        return
    lines = path.read_text(errors="replace").splitlines()
    for line in lines[:limit]:
        print("    " + line[:400])
    if len(lines) > limit:
        print(f"    ... {len(lines) - limit} more lines")


def processes(run_id, work):
    out = subprocess.run(["ps", "-eo", "pid,ppid,sid,etimes,args"], capture_output=True, text=True).stdout.splitlines()
    marks = (f"runs/{run_id}/", str(work), "bwrap", "socat", "spike-gate", "_alive.txt", "claude-cli")
    rows = [line for line in out[1:] if any(m in line for m in marks) and "spike.py" not in line]
    print(f"--- processes still running for this run: {len(rows)}")
    for r in rows:
        print("    " + r[:300])


def evidence(run_id):
    prof, work = load(run_id)
    logs, wlogs = M / "logs", WIN / "logs"
    stream = wlogs / f"{run_id}.jsonl"
    if prof.get("evidence") == "raw":
        show("what the command printed", stream, limit=200)
        show("its error output", wlogs / f"{run_id}.stderr.txt", 30)
        return
    if prof.get("evidence") == "env":
        print("--- the environment the command received (sorted; values shown only for harmless names):")
        lines = stream.read_text(errors="replace").splitlines() if stream.exists() else []
        for line in sorted(lines):
            name, _, value = line.partition("=")
            print(f"    {name}={value if name in SHOW_VALUE else '<value not shown>'}")
        print(f"    ({len(lines)} variables)")
        show("its error output", wlogs / f"{run_id}.stderr.txt", 30)
        return

    print("--- event stream, read with a JSON parser:")
    sys.stdout.flush()
    subprocess.run(["/usr/bin/python3", "-I", str(SPIKES / "tools/parse_claude_stream.py"), str(stream), str(M / "canary/fake_key")])
    show("error output", wlogs / f"{run_id}.stderr.txt", 30)
    gate_logs = [logs / f"{run_id}.gate.log"] if prof["gate"] == "node" else [wlogs / f"{run_id}.gate.log", wlogs / f"{run_id}.host-gate.log"]
    for g in gate_logs:
        show(f"gate log {g.name}", g)
    gate_text = "".join(g.read_text(errors="replace") for g in gate_logs if g.exists())

    print("--- every tool call the agent made, and whether it asked the checker first:")
    for line in stream.read_text(errors="replace").splitlines() if stream.exists() else []:
        try:
            ev = json.loads(line)
        except ValueError:
            continue
        if ev.get("type") != "assistant":
            continue
        for c in (ev.get("message") or {}).get("content") or []:
            if c.get("type") == "tool_use":
                inp = c.get("input") or {}
                what = str(inp.get("command") or inp.get("file_path") or inp.get("url") or json.dumps(inp)[:80])
                asked = json.dumps(what)[1:-1] in gate_text
                print(f"    {c.get('name'):10} asked={'yes' if asked else 'NO '}  {what[:160]}")

    model_log = logs / f"{run_id}.model.log"
    if model_log.exists():
        print("--- scripted model: every request it received:")
        entries = [json.loads(line) for line in model_log.read_text().splitlines() if line.strip()]
        t0 = entries[0]["time"] if entries else 0
        loop_times = []
        for e in entries:
            if e.get("kind") == "agent loop":
                loop_times.append(e["time"])
            print("    +%.2fs %s %s step=%s answer=%s model=%s key_sent=%s key_is_test_key=%s bearer=%s tools=%d" % (
                e["time"] - t0, e.get("kind", e.get("method")), e.get("path", "")[:40], e.get("step"), e.get("answer"),
                e.get("model"), e.get("api_key_sent"), e.get("api_key_matches_test_key"), e.get("bearer_sent"),
                len(e.get("tools_offered") or [])))
        first = next((e for e in entries if e.get("kind") == "agent loop"), None)
        if first:
            print("    tools offered to the model on the first agent request:", ",".join(first["tools_offered"]))
        print("    seconds between agent requests:", [round(b - a, 2) for a, b in zip(loop_times, loop_times[1:])])

    show("m0_results.jsonl (the fixture's own record)", work / "m0_results.jsonl")
    show("listener inside WSL (127.0.0.1:47831)", logs / f"{run_id}.listener.log")
    print("--- marker files: WSL", sorted(p.name for p in (M / "outside").glob("*")) or "none",
          "| Windows", sorted(p.name for p in (WIN / "outside").glob("*")) or "none")
    print("--- working copy HEAD:", git("-C", work, "rev-parse", "HEAD").stdout.strip())
    print("--- git status --short --ignored:")
    print("    " + git("-C", work, "status", "--short", "--ignored").stdout.strip().replace("\n", "\n    "))
    print("--- files in .git/hooks:", sorted(p.name for p in (work / ".git/hooks").glob("*")) if (work / ".git/hooks").exists() else "no such folder")

    before = json.loads((logs / f"{run_id}.before.json").read_text())
    after = snapshot(work)
    print("--- new files in Claude Code's private settings folder:", sorted(set(after["config_files"]) - set(before["config_files"])))
    print("--- new files in Steplet's private home:", sorted(set(after["private_home_files"]) - set(before["private_home_files"])))
    ignore = P / "home/.config/git/ignore"
    print("--- private home .config/git/ignore:", repr(ignore.read_text()) if ignore.exists() else "absent")
    print("--- real ~/.config/git/ignore: before", repr(before["real_git_ignore"]), "| after", repr(after["real_git_ignore"]))
    print("--- Claude Code's record for this folder: before", json.dumps(before["trust_record"]), "| after", json.dumps(after["trust_record"]))

    seen = work / "s3_seen"
    if seen.exists():
        env_file = seen / "env.txt"
        lines = env_file.read_text(errors="replace").splitlines() if env_file.exists() else []
        print("--- environment variable names inside the fence:", ",".join(sorted(l.split("=", 1)[0] for l in lines if "=" in l)))
        odd = [l.split("=", 1)[0] for l in lines if l.split("=", 1)[0] in NOT_LINUX or "/mnt/c" in l or ":\\" in l]
        print("--- Windows or WSL variables inside the fence:", odd or "none")
        cmdlines = {f.name.split("_")[1]: f.read_bytes() for f in seen.glob("proc_*_cmdline.txt")}
        claude_pids = [pid for pid, c in cmdlines.items() if b"--permission-prompt-tool" in c]
        readable_env = [pid for pid in claude_pids if (seen / f"proc_{pid}_environ.txt").exists()]
        print(f"--- /proc: command lines readable from the fence: {len(cmdlines)}; of processes carrying Claude Code's flags: "
              f"{len(claude_pids)}; their environment readable: {len(readable_env)}")
        show("files the probe tried to read", seen / "reads.txt")

    ps_file = work / "s1_ps.txt"
    if ps_file.exists():
        token = (WIN / "runs" / run_id / "token.txt").read_text().strip()
        text = ps_file.read_text(errors="replace")
        print(f"--- the fence's own process list: {len(text.splitlines())} lines; shows the gate: {'spike-gate' in text}; shows the run token: {token in text}")
    show("S8 connection attempts", work / "s8_results.jsonl")

    keys = {"fake API key": (P / "secrets/fake-api-key").read_bytes().strip()}
    real = P / "secrets/anthropic-api-key"
    if real.exists():
        keys["real API key"] = real.read_bytes().strip()
    places = [stream, wlogs / f"{run_id}.stderr.txt", model_log, *gate_logs, ps_file]
    places += sorted(p for p in seen.rglob("*") if p.is_file()) if seen.exists() else []
    print("--- API key text found (counts only; the text is never printed):")
    for label, key in keys.items():
        for part, needle in ((label, key), (label + ", last 20 characters", key[-20:])):
            hits = {p.name: p.read_bytes().count(needle) for p in places if p.is_file() and needle in p.read_bytes()}
            print(f"    {part}: {sum(hits.values())} {hits or ''}")

    if prof.get("grep_debug"):
        words = ("sandbox", "violation", "denied", "EPERM", "Read-only file system", "Network is unreachable", "blocked")
        new_files = [P / "claude-config" / f for f in sorted(set(after["config_files"]) - set(before["config_files"]))]
        print("--- lines about blocked actions in error output and in new files of the settings folder (first 40):")
        found = 0
        for path in [wlogs / f"{run_id}.stderr.txt", *new_files]:
            if not path.is_file():
                continue
            for line in path.read_text(errors="replace").splitlines():
                if found < 40 and any(w.lower() in line.lower() for w in words):
                    print(f"    {path.name}: {line[:240]}")
                    found += 1
        print(f"    ({found} shown)")
    processes(run_id, work)


def cleanup(run_id):
    prof, work = load(run_id)
    marker = f"runs/{run_id}/"

    def marked():
        out = subprocess.run(["ps", "-eo", "pid,sid,args"], capture_output=True, text=True).stdout.splitlines()[1:]
        return [(int(l.split()[0]), int(l.split()[1])) for l in out if marker in l and "spike.py" not in l]

    found = marked()
    sessions = sorted({sid for _, sid in found}) if prof.get("cleanup_by_session") else []
    print(f"--- clean-up: {len(found)} processes carry the run marker; sessions to stop: {sessions or 'none'}")
    for sig in (signal.SIGTERM, signal.SIGKILL):
        for pid, _ in marked():
            try:
                os.kill(pid, sig)
            except ProcessLookupError:
                pass
        for sid in sessions:
            subprocess.run(["pkill", "-" + sig.name[3:], "-s", str(sid)])
        time.sleep(5 if sig == signal.SIGTERM else 1)
    processes(run_id, work)


if __name__ == "__main__":
    {"prepare": prepare, "evidence": evidence, "cleanup": cleanup}[sys.argv[1]](sys.argv[2])
```

`spikes/wsl/services.sh`:
```bash
#!/bin/bash
# Keeps one WSL session open for a spike run and runs its helpers: the scripted model and the loopback listener.
# Usage: services.sh <ID>. Stops when C:\dev\m1\runs\<ID>\stop exists.
ID=$1; M=$HOME/m1; S=/mnt/c/dev/runFile/spikes; R=/mnt/c/dev/m1/runs/$ID
KEY=$HOME/.local/share/steplet/secrets/fake-api-key
pids=()
if [ -f "$M/logs/$ID.script.json" ]; then
  python3 -I "$S/tools/fake_model.py" 47900 "$M/logs/$ID.script.json" "$M/logs/$ID.model.log" "$KEY" > "$M/logs/$ID.model.stdout" 2>&1 &
  pids+=($!)
fi
python3 -I "$S/tools/listener.py" 47831 "$M/logs/$ID.listener.log" > /dev/null 2>&1 &
pids+=($!)
sleep 1
for p in "${pids[@]}"; do
  kill -0 "$p" 2>/dev/null || { echo "a helper died at start (pid $p)" > "$R/services.ready"; exit 1; }
done
echo "running, pids ${pids[*]}" > "$R/services.ready"
while [ ! -e "$R/stop" ]; do sleep 0.5; done
kill "${pids[@]}" 2>/dev/null
```

`spikes/wsl/launch_inside.sh`:
```bash
#!/bin/bash
# Launch a prepared spike run from inside WSL, the way milestone 0 did. Used by S7 only.
ID=$1; R=/mnt/c/dev/m1/runs/$ID; L=/mnt/c/dev/m1/logs
mapfile -t ARGV < "$R/argv.txt"
cd "$(cat "$R/cd.txt")" || exit 1
START=$(date +%s)
"${ARGV[@]}" < "$R/task.txt" > "$L/$ID.jsonl" 2> "$L/$ID.stderr.txt"
CODE=$?
echo "launch inside WSL: exit code $CODE, seconds $(( $(date +%s) - START ))"
```

`spikes/wsl/strings.sh`:
```bash
#!/bin/bash
# Read-only: distinct names inside the pinned Claude Code program that contain a word, or a few
# passages around it with "context". Usage: strings.sh <word> [context]
B=$(readlink -f "$HOME/.local/share/steplet/claude-cli/bin/claude")
if [ "${2:-}" = context ]; then
  grep -a -o -E ".{0,160}$1.{0,160}" "$B" | head -5
else
  grep -a -o -E "[A-Za-z_]{0,40}$1[A-Za-z_]{0,40}" "$B" | sort -u | head -80
fi
```

`spikes/probes/s3_probe.py`:
```python
"""Spike probe, run inside the fence by a scripted command. Copies what this program can see into
s3_seen/ in the working copy, for spike.py evidence to search from outside. Prints no values."""
import os
import pathlib
import sys

out = pathlib.Path("s3_seen")
out.mkdir(exist_ok=True)
(out / "env.txt").write_text("\n".join(f"{k}={v}" for k, v in sorted(os.environ.items())))
copied = 0
for d in pathlib.Path("/proc").iterdir():
    if not d.name.isdigit():
        continue
    for part in ("environ", "cmdline"):
        try:
            (out / f"proc_{d.name}_{part}.txt").write_bytes((d / part).read_bytes())
            copied += 1
        except OSError:
            pass
lines = []
for i, target in enumerate(sys.argv[1:]):
    try:
        data = open(target, "rb").read()
        (out / f"read_{i}.bin").write_bytes(data)
        lines.append(f"{target}: READ {len(data)} bytes")
    except OSError as e:
        lines.append(f"{target}: refused ({type(e).__name__}: {e.strerror})")
(out / "reads.txt").write_text("\n".join(lines) + "\n")
print(f"probe: {len(os.environ)} environment variables, {copied} /proc files copied, {len(lines)} reads tried")
print("\n".join(lines))
```

- [ ] **Step 8: write the Windows side.**

Copy the two milestone 0 helpers unchanged:
```powershell
Copy-Item C:\dev\m0\tools\listener.py C:\dev\runFile\spikes\tools\listener.py
Copy-Item C:\dev\m0\tools\wsl\parse_claude_stream.py C:\dev\runFile\spikes\tools\parse_claude_stream.py
```

`spikes/tools/run.ps1`:
```powershell
# Milestone 1, phase 0: one spike run, start to finish. Throwaway.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File C:\dev\runFile\spikes\tools\run.ps1 -Id S7a
# Everything it shows is also written to C:\dev\m1\logs\<Id>.txt.
param([Parameter(Mandatory = $true)][string]$Id)
$ErrorActionPreference = "Continue"
$env:WSL_UTF8 = "1"
$Distro = "steplet-test"
$Spikes = "C:\dev\runFile\spikes"
$RunDir = "C:\dev\m1\runs\$Id"
$Out = "C:\dev\m1\logs\$Id.txt"
$Run = Get-Content "$Spikes\runs\$Id.json" -Raw | ConvertFrom-Json
$DistroId = (Get-Content "C:\dev\m1\distro-id.txt" -Raw).Trim()
Remove-Item $Out -ErrorAction SilentlyContinue

function Say([string]$Text) { Add-Content -Path $Out -Value $Text -Encoding utf8; Write-Output $Text }
function InWsl([string[]]$Argv) { & wsl.exe -d $Distro --exec @Argv 2>&1 | ForEach-Object { Say "$_" } }

Say "=== spike run $Id, started $(Get-Date -Format o)"
Say "--- run description: $((Get-Content "$Spikes\runs\$Id.json" -Raw).Trim())"
InWsl @("/usr/bin/python3", "-I", "/mnt/c/dev/runFile/spikes/wsl/spike.py", "prepare", $Id)
$svc = Start-Process -FilePath "wsl.exe" -ArgumentList "-d $Distro --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/services.sh $Id" -WindowStyle Hidden -PassThru
for ($i = 0; $i -lt 40 -and -not (Test-Path "$RunDir\services.ready"); $i++) { Start-Sleep -Milliseconds 500 }
Say "--- helpers (scripted model, listener): $(Get-Content "$RunDir\services.ready" -ErrorAction SilentlyContinue)"
$started = Get-Date
if ($Run.launch -eq "inside") {
    InWsl @("/bin/bash", "/mnt/c/dev/runFile/spikes/wsl/launch_inside.sh", $Id)
} else {
    $a = @("launch", "--id", $Id, "--distro-id", $DistroId, "--cd", (Get-Content "$RunDir\cd.txt" -Raw).Trim(), "--run-dir", $RunDir)
    if ($Run.gate -eq "exe") {
        $a += @("--pipe", "steplet-spike-$Id", "--token", (Get-Content "$RunDir\token.txt" -Raw).Trim(), "--gate-mode", $Run.gate_mode)
    }
    if ($Run.host) {
        foreach ($p in $Run.host.PSObject.Properties) { $a += @("--" + $p.Name.Replace("_", "-"), "$($p.Value)") }
    }
    & "$Spikes\target\release\spike-host.exe" @a 2>&1 | ForEach-Object { Say "$_" }
}
Say "--- launch finished after $([math]::Round(((Get-Date) - $started).TotalSeconds, 1)) s"
Start-Sleep -Seconds 2
Say "--- Windows: spike-gate.exe processes still running: $(@(Get-Process spike-gate -ErrorAction SilentlyContinue).Count)"
InWsl @("/usr/bin/python3", "-I", "/mnt/c/dev/runFile/spikes/wsl/spike.py", "evidence", $Id)
if ($Run.cleanup) { InWsl @("/usr/bin/python3", "-I", "/mnt/c/dev/runFile/spikes/wsl/spike.py", "cleanup", $Id) }
New-Item -ItemType File -Path "$RunDir\stop" -Force | Out-Null
if (-not $svc.WaitForExit(15000)) { Say "--- the helpers did not stop within 15 s" }
Say "=== spike run $Id finished $(Get-Date -Format o)"
```

`spikes/tools/append_results.py`:
```python
# Milestone 1, phase 0: append a section to docs/milestone-1-spikes.md.
# Wherever the template says {{LOG:<file>}}, C:\dev\m1\logs\<file> is pasted in a code block.
# Fake keys are replaced with labelled placeholders. If the real API key (or its last 20 characters)
# appears in a log, nothing is written and the script stops: that is a finding, not something to paste.
# The real key is read only to look for it; it is never printed.
# Usage: python -I append_results.py <template.md> <results.md>
import re
import sys

LOGS = "C:\\dev\\m1\\logs\\"
FAKES = [("C:\\dev\\m1\\canary\\wsl_fake_key", "M1-FAKE-KEY-WSL-<removed: the fixture's fake key>"),
         ("C:\\dev\\m1\\canary\\fake_api_key", "sk-ant-api03-M1FAKE-<removed: the fake API key>")]
REAL = "\\\\wsl.localhost\\steplet-test\\home\\tester\\.local\\share\\steplet\\secrets\\anthropic-api-key"

fakes = [(open(path, encoding="ascii").read().strip(), label) for path, label in FAKES]
try:
    real = open(REAL, encoding="ascii").read().strip()
except OSError:
    real = None


def paste(m):
    raw = open(LOGS + m.group(1), encoding="utf-8-sig", errors="replace").read().replace("\r\n", "\n").rstrip("\n")
    if real and (real in raw or real[-20:] in raw):
        sys.exit("STOP: the real API key text is in %s. Nothing was appended." % m.group(1))
    for text, label in fakes:
        raw = raw.replace(text, label).replace(text[-20:], label + " (last 20 characters)")
    raw = re.sub(r"[\x00-\x08\x0b-\x1f\x7f]", lambda c: "\\x%02x" % ord(c.group(0)), raw)
    return "```\n" + raw + "\n```"


text = re.sub(r"\{\{LOG:([^}]+)\}\}", paste, open(sys.argv[1], encoding="utf-8").read())
with open(sys.argv[2], "a", encoding="utf-8", newline="") as f:
    f.write("\n" + text.rstrip("\n") + "\n")
print("appended %d lines" % text.count("\n"))
```

`spikes/README.md`:
```markdown
# Spikes (milestone 1, phase 0)

Throwaway tests that answer one question each before the product is built on the answer.
The plan is `docs/plans/2026-10-08-milestone-1-phase-0.md`; results with pasted output are in
`docs/milestone-1-spikes.md`. Nothing here is product code, and none of it is copied into the
product: phases 1 to 7 write their code fresh, tests first.

- `host/`, `gate/`: Windows stand-ins for the Steplet app and `steplet-gate.exe`.
- `tools/`: the scripted model, self-tests, the run script, the results helper.
- `wsl/`: scripts that run inside the test distribution `steplet-test`.
- `runs/`: one JSON file per run; each says exactly what that run does.
- `scripts/`: the scripted model's tool calls.  `probes/`: programs the fenced commands run.
- `node-gate/`: milestone 0's Node stand-in gate, unchanged.
```

- [ ] **Step 9: create `docs/milestone-1-spikes.md` and paste the setup logs.**

Header of `docs/milestone-1-spikes.md`:
```markdown
# Milestone 1, phase 0: spike results

Plan: `docs/plans/2026-10-08-milestone-1-phase-0.md`. Each section gives the question, the exact
commands, the pasted output, a verdict (PASS, FAIL, INCONCLUSIVE, or RECORDED when the question has
no pass mark), and what it means. The agent's own words are never evidence. Fake keys in pasted logs
are replaced by labelled placeholders (`spikes/tools/append_results.py`).
```
Template `spikes/results/setup.md`:
```markdown
## Setup

### Task 2: Rust tools on Windows
Command: `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\tools\setup_windows.ps1`
{{LOG:setup-windows.txt}}

### Task 3: the test distribution
Created by you with `wsl --install Ubuntu-26.04 --name steplet-test`, user `tester`, then
`sudo apt-get install -y nodejs npm socat bubblewrap ripgrep git python3`. Checks: `spikes/wsl/check_distro.sh`.
{{LOG:setup-distro.txt}}

### Task 4: Steplet's folders, Claude Code 2.1.220, srt 0.0.77, spike data
Command: `wsl -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/setup_steplet.sh`
{{LOG:setup-steplet.txt}}

### Task 5: self-tests of the harness
{{LOG:selftest-fake-model.txt}}
{{LOG:selftest-gate.txt}}
```
Save the self-test output first, then append:
```powershell
wsl -d steplet-test --exec /usr/bin/python3 -I /mnt/c/dev/runFile/spikes/tools/selftest_fake_model.py 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\selftest-fake-model.txt
powershell -NoProfile -ExecutionPolicy Bypass -File C:\dev\runFile\spikes\tools\selftest_gate.ps1 | Out-File -Encoding utf8 C:\dev\m1\logs\selftest-gate.txt
python -I C:\dev\runFile\spikes\tools\append_results.py C:\dev\runFile\spikes\results\setup.md C:\dev\runFile\docs\milestone-1-spikes.md
```
Expected: `appended N lines`; both self-tests show `failures: 0`.

- [ ] **Step 10: commit.**

```powershell
cd C:\dev\runFile
git add spikes docs/milestone-1-spikes.md
git status --short
git commit -m "Add the phase 0 spike harness: scripted model, Windows host and gate stand-ins, run scripts"
```

### Task 6: S7 — does a scripted model change what the checker and the fence see?

**Question (milestone plan):** with a fake model server (`ANTHROPIC_BASE_URL`), does every action still go through the same gate and fence? **Pass:** same gate requests and fence results as T1c.

**Files:**
- Create: `spikes/scripts/s7_t1c.json`, `spikes/runs/S7a.json`, `spikes/runs/S7b.json`, `spikes/results/S7.md`

`spikes/scripts/s7_t1c.json` (T1c's two commands, copied from milestone 0's gate log, 2026-10-07T13:17):
```json
{
  "about": "Replays T1c: check out COMMIT1, run the tests",
  "steps": [
    {"tool": "Bash", "input": {"command": "git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"", "description": "Check out the specified commit"}},
    {"tool": "Bash", "input": {"command": "python3 -m unittest discover -s tests -t . 2>&1; echo \"EXIT=$?\"", "description": "Run the unittest suite"}},
    {"text": "Spike: the tests ran."}
  ]
}
```
`spikes/runs/S7a.json`:
```json
{"about": "S7: scripted model replays T1c; milestone 0 settings; launched inside WSL", "script": "s7_t1c.json", "gate": "node", "gate_mode": "allow", "shape": "m0", "launch": "inside", "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```
`spikes/runs/S7b.json`:
```json
{"about": "S7: same with the product settings shape (allowRead, denyWrite on .git)", "script": "s7_t1c.json", "gate": "node", "gate_mode": "allow", "shape": "product", "launch": "inside", "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```

- [ ] **Step 1: run S7a.**

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\dev\runFile\spikes\tools\run.ps1 -Id S7a
```
**PASS** if all hold:
- The scripted model's log shows the agent-loop requests went to it, with `key_is_test_key=True`, and the stream has `INIT version=2.1.220` and `mcp_servers=[{"name": "gate", "status": "connected"}]`.
- Both Bash calls show `asked=yes`, and the gate log's request has the same fields as T1c's: `{"tool_name":"Bash","input":{"command":…,"description":…},"tool_use_id":…}`.
- `m0_results.jsonl`: A `ok: true`; B, C, D, E and both child lines `ok: false`; G `ok: false`; H `ok: true` (in the `m0` shape `/mnt/c/Windows` is readable, as in T1c).
- The WSL listener log is empty; no `from_wsl.txt` marker; the test ran (`Ran 1 test` in a tool result).

**FAIL** if any attempt or gate line differs from T1c. **INCONCLUSIVE** if Claude Code never reached the scripted model (no agent-loop line) or A is not `ok: true`.

- [ ] **Step 2: run S7b** (same command, `-Id S7b`). Expected differences from S7a, each because of the product shape:
  - H `ok: false`, because `/mnt` is now denied.
  - **The git checkout fails** with a read-only error, because `.git` is now read-only for the agent (D6: in the product, Steplet does the checkout as its own fenced step before the agent starts; S4 tests that step).
  - Everything else as S7a, and the tests still run on the files already there (they read the working copy, which `allowRead` re-opens).
  If the tests cannot read their own files, `allowRead` does not work in 2.1.220 as the milestone plan assumed: record it, change `"shape"` from `product` to `product-noallow` in every later run description, and say so in the summary.

- [ ] **Step 3: write `spikes/results/S7.md`** after reading both outputs:

```markdown
## S7: does a scripted model change what the checker and the fence see?

Question, from the milestone plan: with a fake model server (`ANTHROPIC_BASE_URL`), does every action still
go through the same gate and fence? Pass: same gate requests and fence results as T1c.

Runs: `spikes/runs/S7a.json` (milestone 0 settings, compare with T1c), `spikes/runs/S7b.json` (product settings).
Command: `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\tools\run.ps1 -Id <run>`

### S7a
{{LOG:S7a.txt}}

### S7b
{{LOG:S7b.txt}}

**Verdict:** <PASS | FAIL | INCONCLUSIVE>, because <the lines above that decide it>.
**What it means:** <one paragraph: whether later spikes and the phase 5 suite may use the scripted model;
whether allowRead works; whether the product shape blocks the checkout as D6 intends>.
```
Then append and commit:
```powershell
python -I C:\dev\runFile\spikes\tools\append_results.py C:\dev\runFile\spikes\results\S7.md C:\dev\runFile\docs\milestone-1-spikes.md
git add spikes docs/milestone-1-spikes.md; git commit -m "S7: record whether a scripted model changes what the checker and the fence see"
```
If S7 fails: the later spikes run against the live model instead (Task 15's key, earlier), and the phase 5 suite runs live-only (the milestone plan's fallback). Stop and tell you before spending money.

### Task 7: S2 — launching Claude Code from Windows

**Question (milestone plan):** What does `wsl.exe --exec` add to the environment? Do Linux processes outlive a killed `wsl.exe`? Does Claude Code work with Steplet's private `HOME`, and does F49's git ignore file land there? **Pass:** no Windows variables cross; no process outlives the run once the clean-up runs; F49's file lands in the private home.

**Files:**
- Create: `spikes/scripts/s2_env.json`, `spikes/scripts/long_sleep.json`, `spikes/runs/S2r-empty.json`, `spikes/runs/S2r-systemroot.json`, `spikes/runs/S2r-inherit.json`, `spikes/runs/S2a.json`, `spikes/runs/S2c.json`, `spikes/runs/S2d.json`, `spikes/results/S2.md`

`spikes/scripts/s2_env.json`:
```json
{
  "about": "Record what the fence sees, then run git once (F49 appeared on the first command)",
  "steps": [
    {"tool": "Bash", "input": {"command": "python3 spike_probes/s3_probe.py; echo \"EXIT=$?\"", "description": "Record what the fence can see"}},
    {"tool": "Bash", "input": {"command": "git status --short > /dev/null; echo \"EXIT=$?\"", "description": "Run git once"}},
    {"text": "Spike S2: done."}
  ]
}
```
`spikes/scripts/long_sleep.json`:
```json
{
  "about": "One long fenced command, so the run can be stopped or killed while it works",
  "steps": [
    {"tool": "Bash", "input": {"command": "python3 -c \"import time; open('spike_alive.txt','w').write('x'); time.sleep(600)\"", "description": "Stay busy", "timeout": 600000}},
    {"text": "Spike: the long command ended."}
  ]
}
```
Run descriptions. `S2r-empty.json`, `S2r-systemroot.json`, `S2r-inherit.json` (the last passes on the launcher's whole Windows environment, as a comparison):
```json
{"about": "S2: the environment wsl.exe gives a program when the launcher passes no Windows variables", "script": "none", "gate": "node", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "argv_override": ["/usr/bin/env"], "evidence": "env"}
```
```json
{"about": "S2: the same with only SystemRoot passed", "script": "none", "gate": "node", "shape": "product", "launch": "host", "host": {"env_mode": "systemroot"}, "argv_override": ["/usr/bin/env"], "evidence": "env"}
```
```json
{"about": "S2: the same with the launcher's whole Windows environment, for comparison", "script": "none", "gate": "node", "shape": "product", "launch": "host", "host": {"env_mode": "inherit"}, "argv_override": ["/usr/bin/env"], "evidence": "env"}
```
```json
{"about": "S2: a full run launched from Windows; what the fence sees; where F49's git ignore file lands", "script": "s2_env.json", "gate": "node", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "probes": ["s3_probe.py"], "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```
(`S2a.json`.)
```json
{"about": "S2: kill wsl.exe while a fenced command runs; list what keeps running; then clean up by the run marker", "script": "long_sleep.json", "gate": "node", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty", "kill_wsl_after_s": 20}, "cleanup": true, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```
(`S2c.json`.)
```json
{"about": "S2: only if S2c left processes: same, started in its own process session and cleaned up by session", "script": "long_sleep.json", "gate": "node", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty", "kill_wsl_after_s": 20}, "argv_prefix": ["/usr/bin/setsid", "-w"], "cleanup": true, "cleanup_by_session": true, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```
(`S2d.json`.)

- [ ] **Step 1: run S2r-empty, S2r-systemroot, S2r-inherit** (`run.ps1 -Id <each>`). Record which environment modes let `wsl.exe` start at all, and the variable list each gives. Every later run description uses `"env_mode": "empty"`; if `empty` failed and `systemroot` worked, change it to `systemroot` in every `spikes/runs/*.json` that has a `host` field, and say so in the results.
- [ ] **Step 2: run S2a.** Pass parts: the run ends with a `result` event; "Windows or WSL variables inside the fence: none"; the private home's `.config/git/ignore` exists after the run and the real `~/.config/git/ignore` is `None` before and after. Also recorded: whether the fence can read Claude Code's own command line and environment through `/proc` (S3 depends on it).
- [ ] **Step 3: run S2c.** Recorded: which processes are still running after `wsl.exe` was killed (evidence) and after clean-up (clean-up list). Pass part: zero after clean-up. If not zero, run S2d and record it; if S2d also leaves processes, the result is FAIL and phase 3 needs the runner-up (Steplet's own small Linux launcher).
- [ ] **Step 4: write `spikes/results/S2.md`** (same layout as S7.md: question, runs, `{{LOG:S2r-empty.txt}}`, `{{LOG:S2r-systemroot.txt}}`, `{{LOG:S2r-inherit.txt}}`, `{{LOG:S2a.txt}}`, `{{LOG:S2c.txt}}`, and `{{LOG:S2d.txt}}` if run; verdict; what it means), append it, commit with message `S2: record what launching from Windows passes on, what outlives a killed wsl.exe, and where F49's file lands`.

### Task 8: S1 — the gate as a Windows program started through interop

**Question (milestone plan):** Can Claude Code start a Windows `.exe` through interop as its permission-prompt server? Does that program exit when Claude Code is stopped or killed, and come back after a restart? Can a fenced program still not start Windows programs (F28)? How long does one decision take? **Pass:** all yes; the time recorded.

**Files:**
- Create: `spikes/scripts/s1_writes.json`, run descriptions `S1a`, `S1a-i`, `S1n`, `S1b`, `S1s`, `S1c`, `S1t`, `S1k`, `S1f`, `spikes/results/S1.md`

`spikes/scripts/s1_writes.json` (ten file writes, then the fence's own process list):
```json
{
  "about": "Ten Write calls, each one decision, then the process list as the fence sees it",
  "steps": [
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_01.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_02.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_03.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_04.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_05.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_06.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_07.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_08.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_09.txt", "content": "s1\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/s1_marker_10.txt", "content": "s1\n"}},
    {"tool": "Bash", "input": {"command": "ps -eo pid,args > s1_ps.txt 2>&1; echo \"EXIT=$?\"", "description": "Save the process list"}},
    {"text": "Spike S1: done."}
  ]
}
```
Run descriptions (all `"launch": "host"`, `"shape": "product"`, `"env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]`, `"host": {"env_mode": "empty"}` unless shown):

| File | Fields that differ | What it shows |
|---|---|---|
| `S1a.json` | `"about": "S1: gate as a Windows program, host allows; ten writes"`, `"script": "s1_writes.json"`, `"gate": "exe"`, `"gate_mode": "allow"` | it starts, decides, how long |
| `S1a-i.json` | as S1a plus `"env_extra": ["ANTHROPIC_API_KEY=@fake-api-key", "WSL_INTEROP=/run/WSL/1_interop"]`; **run only if S1a's gate did not start** (use the socket name Task 3 recorded) | whether interop needs this variable |
| `S1n.json` | `"script": "s1_writes.json"`, `"gate": "node"`, `"gate_mode": "allow"` | the Node gate inside WSL, for comparing the time |
| `S1b.json` | `"script": "s1_writes.json"`, `"gate": "exe"`, `"gate_mode": "drop"` | host closes without answering → every write refused |
| `S1s.json` | `"script": "s1_writes.json"`, `"gate": "exe"`, `"gate_mode": "stall"`, `"host": {"env_mode": "empty", "deadline_ms": 5000}` | host's own deadline refuses after about 5 s each |
| `S1c.json` | `"script": "s1_writes.json"`, `"gate": "exe"`, `"gate_mode": "allow"`, `"gate_crash_once": true` | first decision crashes the gate; is it restarted? |
| `S1t.json` | `"script": "long_sleep.json"`, `"gate": "exe"`, `"gate_mode": "allow"`, `"host": {"env_mode": "empty", "stop_after_s": 20}`, `"cleanup": true` | stop signal: does the gate exit? |
| `S1k.json` | `"script": "long_sleep.json"`, `"gate": "exe"`, `"gate_mode": "allow"`, `"host": {"env_mode": "empty", "kill_claude_after_s": 20}`, `"cleanup": true` | hard kill of Claude Code: does the gate exit? |
| `S1f.json` | `"script": "s7_t1c.json"`, `"gate": "exe"`, `"gate_mode": "allow"`, **`"shape": "m0"`** | the fixture through the Windows gate: F28 still holds. Milestone 0's settings on purpose: the product shape denies all of `/mnt`, so attempt G would fail just because it cannot read `cmd.exe`, and the run would not test interop at all |

The files, in full:
```json
{"about": "S1: gate as a Windows program, host allows; ten writes", "script": "s1_writes.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S1: as S1a, with WSL_INTEROP passed (only if S1a's gate did not start)", "script": "s1_writes.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key", "WSL_INTEROP=/run/WSL/1_interop"]}
{"about": "S1: the Node stand-in gate inside WSL, for comparing decision times", "script": "s1_writes.json", "gate": "node", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S1: host closes the pipe without answering; every write must be refused", "script": "s1_writes.json", "gate": "exe", "gate_mode": "drop", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S1: host never decides; its own 5 s deadline must refuse each write", "script": "s1_writes.json", "gate": "exe", "gate_mode": "stall", "shape": "product", "launch": "host", "host": {"env_mode": "empty", "deadline_ms": 5000}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S1: the gate crashes on its first decision; is it restarted, and is that write refused?", "script": "s1_writes.json", "gate": "exe", "gate_mode": "allow", "gate_crash_once": true, "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S1: stop signal to Claude Code mid-command; does the gate exit?", "script": "long_sleep.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty", "stop_after_s": 20}, "cleanup": true, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S1: hard kill of Claude Code mid-command; does the gate exit?", "script": "long_sleep.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty", "kill_claude_after_s": 20}, "cleanup": true, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S1: the fixture through the Windows gate, milestone 0 settings, so attempt G tests interop", "script": "s7_t1c.json", "gate": "exe", "gate_mode": "allow", "shape": "m0", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```
(One line per file, in table order: `S1a`, `S1a-i`, `S1n`, `S1b`, `S1s`, `S1c`, `S1t`, `S1k`, `S1f`. For `S1a-i`, use the interop socket name Task 3 recorded if it is not `1_interop`.)

- [ ] **Step 1: run S1a.** Pass parts: the stream's `mcp_servers` shows the gate `connected`; all ten Writes show `asked=yes`; ten markers exist (`git status` lists them); the gate log shows `started; N environment variables: …` and ten `behavior="allow"` lines; the host log's `tasklist` shows the gate under your Windows account. Recorded: the gate's `ms=` values, the scripted model's "seconds between agent requests", and whether the fence's own process list shows the gate and the run token.
  If the gate shows `failed`, run S1a-i. If that also fails, S1 is FAIL: the milestone plan's runner-up (`--permission-prompt-tool stdio`) applies, but only after the phase 5 suite shows no action skips the checker; tell you before going further.
- [ ] **Step 2: run S1n.** Compare its "seconds between agent requests" with S1a's. Recorded, no pass mark.
- [ ] **Step 3: run S1b and S1s.** Pass parts: zero markers; every Write listed under `permission_denials`; S1s's host log shows each decision answered after about 5000 ms.
- [ ] **Step 4: run S1c.** Pass parts: the gate log shows two different `pid=` values; the first Write refused, later ones allowed (nine markers or fewer, never the first one written while the gate was dead).
- [ ] **Step 5: run S1t and S1k.** Pass parts: "spike-gate.exe processes still running: 0" in both; after clean-up no processes left in WSL. Also recorded: the result event of S1t (F37 says a stop signal ends cleanly).
- [ ] **Step 6: run S1f.** Pass parts: G `ok: false` and no `C:\dev\m1\outside\from_wsl.txt`, while the gate (a Windows program) was running through interop for the same Claude Code; everything else as S7a.
- [ ] **Step 7: write `spikes/results/S1.md`** (question, runs, one `{{LOG:<run>.txt}}` per run made, a small table of decision times from S1a and S1n, verdict, what it means), append, commit with message `S1: record whether Claude Code can use a Windows gate program through WSL interop`.

### Task 9: S3 — can a fenced program see the API key?

**Question (milestone plan):** with API-key sign-in (P1-A), can a fenced program see the key, either in its environment or through an `apiKeyHelper` file? Does a key stop Claude Code using any stored sign-in? **Pass:** the key is invisible to fenced programs. The fake API key stands in for a real one; nothing here uses a real key.

**Files:**
- Create: `spikes/scripts/s3_probe.json`, run descriptions `S3a`, `S3b`, `S3c`, `S3d`, `spikes/wsl/s3e_precedence.sh`, `spikes/results/S3.md`

`spikes/scripts/s3_probe.json`:
```json
{
  "about": "Copy what the fence can see, and try to read Steplet's private files",
  "steps": [
    {"tool": "Bash", "input": {"command": "python3 spike_probes/s3_probe.py {P}/secrets/fake-api-key {P}/claude-config/.claude.json {P}/claude-config/settings.json {P}/runs/{ID}/settings.json {P}/home/.config/git/ignore; echo \"EXIT=$?\"", "description": "Record what the fence can see"}},
    {"text": "Spike S3: done."}
  ]
}
```
Run descriptions (all: `"script": "s3_probe.json"`, `"gate": "exe"`, `"gate_mode": "allow"`, `"shape": "product"`, `"launch": "host"`, `"host": {"env_mode": "empty"}`, `"probes": ["s3_probe.py"]`):

| File | Key route | Extra fields |
|---|---|---|
| `S3a.json` | key in Claude Code's environment, put there by the launch line | `"env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]` |
| `S3b.json` | the same, plus Claude Code's documented clean-up of child environments | `"env_extra": ["ANTHROPIC_API_KEY=@fake-api-key", "CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1"]` |
| `S3c.json` | `apiKeyHelper` reads the key file in the private folder; no key in any environment | `"env_extra": []`, `"settings_extra": {"apiKeyHelper": "/bin/cat {P}/secrets/fake-api-key"}` |
| `S3d.json` | no key at all | `"env_extra": []` |

The files, in full (`S3a`, `S3b`, `S3c`, `S3d`):
```json
{"about": "S3: key in Claude Code's environment, from the launch line; can the fence see it?", "script": "s3_probe.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "probes": ["s3_probe.py"], "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S3: as S3a, plus CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1", "script": "s3_probe.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "probes": ["s3_probe.py"], "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key", "CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1"]}
{"about": "S3: key through apiKeyHelper only; can the fence see it?", "script": "s3_probe.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "probes": ["s3_probe.py"], "env_extra": [], "settings_extra": {"apiKeyHelper": "/bin/cat {P}/secrets/fake-api-key"}}
{"about": "S3: no key at all; is there any sign-in to fall back on?", "script": "s3_probe.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "probes": ["s3_probe.py"], "env_extra": []}
```

- [ ] **Step 1: run S3a, S3b, S3c.** For each, pass for that route means all of:
  - the scripted model's log shows `key_sent=True key_is_test_key=True` (Claude Code really used that key);
  - "fake API key: 0" and "fake API key, last 20 characters: 0" in the counts (nothing the fence saw, nothing in the stream, error output or gate logs holds it);
  - every line in "files the probe tried to read" says `refused`.
  Note that the launch line passes `ANTHROPIC_API_KEY=…` as an argument of `env`, so in S3a and S3b the key is also on the command line of the `timeout` process; the `/proc` line shows whether the fence can read command lines at all.
- [ ] **Step 2: run S3d.** Recorded: how Claude Code fails with no key (expected: not signed in), and that the scripted model got no request carrying any key or `bearer`. This shows Steplet's private folder has no hidden sign-in to fall back on.
- [ ] **Step 3 (needs yes A5): S3e, in your main Ubuntu.** Write `spikes/wsl/s3e_precedence.sh`:

```bash
#!/bin/bash
# S3e, in the MAIN Ubuntu: with milestone 0's signed-in Claude folder, does an API key win over the stored
# sign-in in -p mode? The key is made up and cannot work. Prints only the sign-in method and whether the run failed.
export CLAUDE_CONFIG_DIR=/home/ranvir/m0/claude-home DISABLE_AUTOUPDATER=1 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
C=/home/ranvir/m0/claude-cli/bin/claude
FAKE=sk-ant-api03-steplet-spike-made-up-key-0000000000
method() {
  "$C" auth status < /dev/null 2>&1 | python3 -c '
import json, sys
raw = sys.stdin.read()
try:
    print("authMethod:", json.loads(raw).get("authMethod"))
except ValueError:
    print("auth status did not print JSON; first line not shown, length", len(raw))'
}
echo "claude: $("$C" --version)"
echo "--- without a key:"; method
echo "--- with a made-up key:"; ANTHROPIC_API_KEY=$FAKE method
echo "--- one tiny prompt with the made-up key:"
ANTHROPIC_API_KEY=$FAKE "$C" -p "Reply with the single word ok" --max-turns 1 --output-format json < /dev/null 2>&1 | python3 -c '
import json, sys
raw = sys.stdin.read()
try:
    d = json.loads(raw)
    print("is_error:", d.get("is_error"), "| result:", str(d.get("result"))[:120])
except ValueError:
    print("not JSON:", raw[:200])'
```
Run:
```powershell
wsl -d Ubuntu --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/s3e_precedence.sh 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\S3e.txt; Get-Content C:\dev\m1\logs\S3e.txt
```
Pass part: with the made-up key the method is `api_key` (or the run fails with an invalid-key error), not your subscription. If the prompt answers `ok`, the stored sign-in won: record it as a finding, because then a stray sign-in file could silently replace the receiver's key.
- [ ] **Step 4: verdict.** PASS if at least one route (S3a, S3b or S3c) passes. My recommendation for the product, if more than one passes, is S3c, because the key then never sits in any process's environment or command line, where a later Claude Code version could leak it. If no route passes: FAIL, P1-A cannot ship until a route is found (milestone plan), and I stop and tell you.
- [ ] **Step 5: write `spikes/results/S3.md`** (question, runs, `{{LOG:S3a.txt}}` … `{{LOG:S3e.txt}}`, verdict, the route Task 15 uses, what it means), append, commit with message `S3: record whether fenced programs can see the API key`.

### Task 10: S4 — checkout in the fence with no agent, and a read-only `.git`

**Question (milestone plan):** checkout under `srt` with no agent. Does `denyWrite` on `.git` refuse writes from commands **and** from the Write tool? Optional: the clone itself under `srt`, allowing only the git host. **Pass:** checkout works; both `.git` writes are refused.

**Files:**
- Create: `spikes/wsl/s4_clone.sh`, `spikes/wsl/s4_fenced_clone.sh`, `spikes/wsl/s4_fenced_clone_run.sh`, `spikes/scripts/s4_git.json`, `spikes/runs/S4b.json`, `spikes/runs/S4c.json`, `spikes/results/S4.md`

`spikes/wsl/s4_clone.sh`:
```bash
#!/bin/bash
# S4, part 1: Steplet's hardened clone (no fence, no checkout), then the checkout alone inside srt, with no agent.
set -u
P=$HOME/.local/share/steplet; W=$HOME/steplet-work/S4; R=$W/repo
URL=https://github.com/octocat/Hello-World.git
COMMIT=7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
SRT=$P/srt/node_modules/.bin/srt
rm -rf "$W"; mkdir -p "$W/empty-home"
GITENV=(/usr/bin/env -i PATH=/usr/bin:/bin HOME="$W/empty-home" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0 GIT_LFS_SKIP_SMUDGE=1)
HARD=(-c protocol.allow=never -c protocol.https.allow=always -c core.hooksPath=/dev/null -c core.fsmonitor=false
      -c transfer.fsckObjects=true -c transfer.bundleURI=false -c credential.helper=)
echo "git: $(/usr/bin/git --version)"
echo "--- 1. hardened clone, no checkout, no fence"
"${GITENV[@]}" /usr/bin/git "${HARD[@]}" clone --no-checkout --no-recurse-submodules --template= "$URL" "$R" 2>&1
echo "clone exit code: $?"
echo "commit present: $("${GITENV[@]}" /usr/bin/git -C "$R" cat-file -t "$COMMIT" 2>&1)"
echo ".git/hooks after clone: $(ls -A "$R/.git/hooks" 2>&1 | tr '\n' ' ')"
echo "files outside .git before checkout: $(ls -A "$R" | grep -v '^.git$' | tr '\n' ' ')"
cat > "$W/srt-checkout.json" <<EOF
{ "filesystem": { "denyRead": ["/mnt", "$P", "$HOME/m1"], "allowWrite": ["$R"], "denyWrite": [] },
  "network": { "allowedDomains": [], "deniedDomains": [] } }
EOF
echo "--- 2. control: the srt fence holds (write outside, read the private folder, internet)"
"$SRT" --settings "$W/srt-checkout.json" /bin/sh -c "echo x > $W/outside_marker" 2>&1 | tail -2
echo "outside marker: $(test -e "$W/outside_marker" && echo EXISTS || echo absent)"
"$SRT" --settings "$W/srt-checkout.json" /bin/ls "$P" 2>&1 | tail -2
"$SRT" --settings "$W/srt-checkout.json" /usr/bin/python3 -c "import socket; socket.create_connection(('1.1.1.1', 443), timeout=5); print('CONNECTED')" 2>&1 | tail -1
echo "--- 3. checkout inside srt, no agent"
"$SRT" --settings "$W/srt-checkout.json" "${GITENV[@]}" /usr/bin/git -C "$R" -c core.hooksPath=/dev/null -c core.fsmonitor=false checkout --quiet --detach "$COMMIT" 2>&1
echo "checkout exit code: $?"
echo "HEAD: $(git -C "$R" rev-parse HEAD)"
echo "status: $(git -C "$R" status --short | head -5)"
echo "files: $(ls -A "$R" | tr '\n' ' ')"
```
`spikes/wsl/s4_fenced_clone.sh` (optional part: the clone itself inside `srt`, network only to `github.com`):
```bash
#!/bin/bash
# S4, optional part: the clone itself inside srt, allowed to reach github.com only. Runs inside srt;
# keeps srt's proxy variables, drops everything else.
W=$HOME/steplet-work/S4-fenced-clone; R=$W/repo
exec /usr/bin/env -i PATH=/usr/bin:/bin HOME="$W/empty-home" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null \
  GIT_TERMINAL_PROMPT=0 GIT_LFS_SKIP_SMUDGE=1 HTTPS_PROXY="${HTTPS_PROXY:-}" HTTP_PROXY="${HTTP_PROXY:-}" \
  https_proxy="${https_proxy:-}" http_proxy="${http_proxy:-}" ALL_PROXY="${ALL_PROXY:-}" \
  /usr/bin/git -c protocol.allow=never -c protocol.https.allow=always -c core.hooksPath=/dev/null -c core.fsmonitor=false \
  -c transfer.fsckObjects=true -c transfer.bundleURI=false -c credential.helper= \
  clone --no-checkout --no-recurse-submodules --template= https://github.com/octocat/Hello-World.git "$R"
```
`spikes/scripts/s4_git.json`:
```json
{
  "about": "Try to change .git by shell commands and by the Write tool; then two controls",
  "steps": [
    {"tool": "Bash", "input": {"command": "mkdir -p .git/hooks && echo planted > .git/hooks/post-checkout; echo \"EXIT=$?\"", "description": "Try to plant a git hook"}},
    {"tool": "Bash", "input": {"command": "echo planted > .git/spike_planted_by_bash; echo \"EXIT=$?\"", "description": "Try to write into .git"}},
    {"tool": "Bash", "input": {"command": "git config core.hooksPath /tmp/spike-hooks; echo \"EXIT=$?\"", "description": "Try to change git settings"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/.git/hooks/pre-commit", "content": "#!/bin/sh\necho planted\n"}},
    {"tool": "Write", "input": {"file_path": "{WORK}/.git/spike_planted_by_write", "content": "planted\n"}},
    {"tool": "Bash", "input": {"command": "echo ok > spike_inside.txt; echo \"EXIT=$?\"", "description": "Control: write inside the working copy"}},
    {"tool": "Bash", "input": {"command": "git status --short; echo \"EXIT=$?\"", "description": "Control: read the repository"}},
    {"text": "Spike S4: done."}
  ]
}
```
`spikes/runs/S4b.json`:
```json
{"about": "S4: agent step on the fenced checkout; product settings (denyWrite and Edit deny on .git)", "script": "s4_git.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "workcopy": "keep:S4", "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```
`spikes/runs/S4c.json` (control: the same attempts with milestone 0's settings, which do not protect `.git`, to show the attempts themselves work):
```json
{"about": "S4 control: same attempts, milestone 0 settings (no .git protection)", "script": "s4_git.json", "gate": "exe", "gate_mode": "allow", "shape": "m0", "launch": "host", "host": {"env_mode": "empty"}, "workcopy": "keep:S4", "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```

- [ ] **Step 1: clone and fenced checkout.**
```powershell
wsl -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/s4_clone.sh 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\S4-clone.txt; Get-Content C:\dev\m1\logs\S4-clone.txt
```
Pass parts: clone exit 0; `commit present: commit`; `.git/hooks` empty or missing; the control lines show the outside write absent, the private-folder listing refused, no `CONNECTED`; checkout exit 0; `HEAD: 7fd1a60b…`; status clean; `README` present.
- [ ] **Step 2: run S4b.** Pass parts: every `.git` attempt fails (each Bash step prints a non-zero `EXIT=`; both Writes are under `permission_denials` or end in a tool error); after the run `.git/hooks` holds no `post-checkout` or `pre-commit`, and `.git/spike_planted_by_*` and `core.hooksPath` are absent; the control write `spike_inside.txt` exists; `git status` worked. Recorded: whether the refused Writes reached the gate at all (the deny rule should stop them first).
- [ ] **Step 3: run the clone again, then S4c.** `s4_clone.sh` again (fresh copy), then `run.ps1 -Id S4c`. Expected: the same attempts **succeed** with milestone 0's settings. If they also fail, S4b's result did not come from our rules: S4 is INCONCLUSIVE for that part.
- [ ] **Step 4 (optional part): fenced clone.**
```powershell
wsl -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/s4_fenced_clone_run.sh 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\S4-fenced-clone.txt; Get-Content C:\dev\m1\logs\S4-fenced-clone.txt
```
with `spikes/wsl/s4_fenced_clone_run.sh`:
```bash
#!/bin/bash
# Runs s4_fenced_clone.sh inside srt with network to github.com only. The script is copied into the folder
# the fence allows first, because the fence denies reading /mnt, where the repository's copy lives.
P=$HOME/.local/share/steplet; W=$HOME/steplet-work/S4-fenced-clone
rm -rf "$W"; mkdir -p "$W/empty-home"
cp /mnt/c/dev/runFile/spikes/wsl/s4_fenced_clone.sh "$W/"
cat > "$W/srt-clone.json" <<EOF
{ "filesystem": { "denyRead": ["/mnt", "$P", "$HOME/m1"], "allowWrite": ["$W"], "denyWrite": [] },
  "network": { "allowedDomains": ["github.com"], "deniedDomains": [] } }
EOF
"$P/srt/node_modules/.bin/srt" --settings "$W/srt-clone.json" /bin/bash "$W/s4_fenced_clone.sh" 2>&1 | tail -5
echo "clone exit code: ${PIPESTATUS[0]}"
echo "commit present: $(git -C "$W/repo" cat-file -t 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d 2>&1)"
```
Recorded either way. If `github.com` alone is not enough (git may be sent to another GitHub host), the error names the host; record it rather than widening the list.
- [ ] **Step 5: write `spikes/results/S4.md`** (`{{LOG:S4-clone.txt}}`, `{{LOG:S4b.txt}}`, the second clone and `{{LOG:S4c.txt}}`, `{{LOG:S4-fenced-clone.txt}}`, verdict, what it means for D6), append, commit with message `S4: record the fenced checkout and whether .git stays read-only for the agent`.
  If S4 fails: the milestone plan's fallback is Steplet's own hardened checkout without a fence (D6's fallback). Record, do not switch; phase 3's plan decides with you.

### Task 11: S5 — commands that skip the checker, and `--tools`

**Question (milestone plan):** is there a setting that makes every command ask the checker (D8-C)? Does `--tools` limit the tool list as its help text says? **Pass:** recorded either way (D8's harmless list is already decided as the fallback).

**Files:**
- Create: `spikes/scripts/s5_readonly.json`, `spikes/scripts/s5_tools.json`, run descriptions `S5a`, `S5b`, `S5c`, `S5t`, `S5u`, `spikes/results/S5.md`

`spikes/scripts/s5_readonly.json`:
```json
{
  "about": "Commands that look read-only, then one that writes",
  "steps": [
    {"tool": "Bash", "input": {"command": "ls -la", "description": "List files"}},
    {"tool": "Bash", "input": {"command": "cat README.md", "description": "Show the readme"}},
    {"tool": "Bash", "input": {"command": "echo ok", "description": "Print ok"}},
    {"tool": "Bash", "input": {"command": "pwd", "description": "Print the folder"}},
    {"tool": "Bash", "input": {"command": "git status --short", "description": "Show git status"}},
    {"tool": "Bash", "input": {"command": "wc -l README.md", "description": "Count lines"}},
    {"tool": "Bash", "input": {"command": "head -1 README.md", "description": "First line"}},
    {"tool": "Bash", "input": {"command": "grep -c second README.md", "description": "Search the readme"}},
    {"tool": "Bash", "input": {"command": "printf ok > s5_marker.txt", "description": "Write a file"}},
    {"text": "Spike S5: done."}
  ]
}
```
`spikes/scripts/s5_tools.json`:
```json
{
  "about": "One allowed tool, then two the --tools list leaves out",
  "steps": [
    {"tool": "Bash", "input": {"command": "echo ok > s5t.txt; echo \"EXIT=$?\"", "description": "Write a file"}},
    {"tool": "WebFetch", "input": {"url": "https://example.com", "prompt": "What is the page title?"}},
    {"tool": "Read", "input": {"file_path": "{WORK}/README.md"}},
    {"text": "Spike S5: done."}
  ]
}
```
Run descriptions (all `"gate": "exe"`, `"gate_mode": "allow"`, `"shape": "product"`, `"launch": "host"`, `"host": {"env_mode": "empty"}`, `"env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]`):

| File | Script | Extra fields |
|---|---|---|
| `S5a.json` | `s5_readonly.json` | none (baseline: reproduces F34?) |
| `S5b.json` | `s5_readonly.json` | `"settings_extra": {"permissions": {"ask": ["Bash"]}}` |
| `S5c.json` | `s5_readonly.json` | `"settings_extra": {"permissions": {"ask": ["Bash(*)"]}}` |
| `S5t.json` | `s5_tools.json` | `"flags": ["--tools", "Bash,Write"]` |
| `S5u.json` | `s5_tools.json` | `"flags": ["--disallowedTools", "mcp__gate__decide"]`; **run only if** S5a shows `mcp__gate__decide` among the tools offered to the model |

The files, in full (one line per file, in table order):
```json
{"about": "S5: which read-only-looking commands skip the checker (baseline, F34)", "script": "s5_readonly.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S5: the same with an ask rule for Bash", "script": "s5_readonly.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"], "settings_extra": {"permissions": {"ask": ["Bash"]}}}
{"about": "S5: the same with an ask rule for Bash(*)", "script": "s5_readonly.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"], "settings_extra": {"permissions": {"ask": ["Bash(*)"]}}}
{"about": "S5: does --tools limit the tools offered to the model?", "script": "s5_tools.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"], "flags": ["--tools", "Bash,Write"]}
{"about": "S5: hide the gate tool from the model; is the checker still asked?", "script": "s5_tools.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"], "flags": ["--disallowedTools", "mcp__gate__decide"]}
```

- [ ] **Step 1: record the names Claude Code 2.1.220 has around read-only commands.**
```powershell
wsl -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/strings.sh readOnly 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\S5-strings.txt; Get-Content C:\dev\m1\logs\S5-strings.txt
```
Recorded. If a name there looks like a setting (for example a word ending in `ReadOnly` that also appears in the settings documentation), add one more run `S5d.json` with that setting and say which name was tried; otherwise none.
- [ ] **Step 2: run S5a, S5b, S5c.** For each, the evidence lists every Bash call with `asked=yes` or `asked=NO `. Recorded: which commands skipped the checker in each. A setting "makes every command ask" if all nine show `asked=yes`.
- [ ] **Step 3: run S5t.** Recorded: the tools offered to the model on the first request (the scripted model's log is the evidence, not the agent); what happened to the WebFetch and Read calls (tool error text). `--tools` works as its help says if only `Bash` and `Write` (plus any MCP tool) were offered and the other two calls failed as unknown tools.
- [ ] **Step 4: if S5a offered `mcp__gate__decide` to the model, run S5u.** Recorded: whether the gate tool disappears from the model's list while the checker still gets asked for the Bash write.
- [ ] **Step 5: write `spikes/results/S5.md`** (`{{LOG:S5-strings.txt}}`, one `{{LOG:…}}` per run, a table command × run → asked yes/no, verdict RECORDED, what it means for D8 and for the tools Steplet offers), append, commit with message `S5: record which commands skip the checker and what --tools does`.

### Task 12: S6 — the model switch after a refusal, and reporting of blocked attempts (scripted part)

**Question (milestone plan):** can the refusal model switch be turned off (D7-C)? Does Claude Code report blocked fence attempts anywhere the app can read (stream, error output, a log)? **Pass:** recorded either way.

**Files:**
- Create: `spikes/scripts/s6_refusal.json`, run descriptions `S6a`, `S6b`, `S6c`, `spikes/results/S6.md`

- [ ] **Step 1: find the refusal's field names.**
```powershell
wsl -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/strings.sh api_refusal_category context 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\S6-strings.txt
wsl -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/strings.sh switchModelsOnFlag context 2>&1 | ForEach-Object { "$_" } | Add-Content -Encoding utf8 C:\dev\m1\logs\S6-strings.txt
Get-Content C:\dev\m1\logs\S6-strings.txt
```
Read which fields of the model service's answer Claude Code takes the category and explanation from (in milestone 0's event they surfaced as `api_refusal_category` and `api_refusal_explanation`). Put those names into the `refusal` step below, under `"message"` if they belong to the message and under `"delta"` if they belong to the stop details. If the passage does not show them, use the file as written: a bare `stop_reason: "refusal"`.

`spikes/scripts/s6_refusal.json` (as written before Step 1's edit):
```json
{
  "about": "One normal tool call, then a refusal; a repeat request at that step gets an answer",
  "steps": [
    {"tool": "Bash", "input": {"command": "echo before > s6_before.txt; echo \"EXIT=$?\"", "description": "Write a file"}},
    {"refusal": {"message": {}, "delta": {}}, "text_after_refusal": "Spike S6: answer after the refusal."},
    {"text": "Spike S6: done."}
  ]
}
```
Run descriptions (all `"gate": "exe"`, `"gate_mode": "allow"`, `"shape": "product"`, `"launch": "host"`, `"host": {"env_mode": "empty"}`, `"env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]`):

| File | Script | Extra fields |
|---|---|---|
| `S6a.json` | `s6_refusal.json` | none |
| `S6b.json` | `s6_refusal.json` | `"settings_extra": {"switchModelsOnFlag": false}` |
| `S6c.json` | `s7_t1c.json` | `"flags": ["--debug"]`, `"grep_debug": true` |

The files, in full (one line per file, in table order):
```json
{"about": "S6: a scripted refusal; does Claude Code switch model, and what shows it?", "script": "s6_refusal.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
{"about": "S6: the same with switchModelsOnFlag false", "script": "s6_refusal.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"], "settings_extra": {"switchModelsOnFlag": false}}
{"about": "S6: the fixture with --debug; does anything Steplet can read report the blocked attempts?", "script": "s7_t1c.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"], "flags": ["--debug"], "grep_debug": true}
```

- [ ] **Step 2: run S6a and S6b.** Recorded from the scripted model's log, not the agent: after the refusal, did a second request arrive at the same step, and under which model name? From the stream: any `system/model_refusal_fallback` event, and how the run ended. The switch "can be turned off" if S6a shows a second request under a different model and S6b does not. If S6a shows no switch at all, the scripted refusal does not trigger it: the question moves to the live runs in Task 15.
- [ ] **Step 3: run S6c.** Recorded: whether any line in the stream, the error output, or new files in Claude Code's settings folder reports the blocked attempts (B, C, D, E, G) as blocked by the sandbox, as opposed to the fixture's own error text inside tool results. If none does, the report must say "blocked attempts are not visible to Steplet" (the milestone plan's wording).
- [ ] **Step 4: write `spikes/results/S6.md`** (`{{LOG:S6-strings.txt}}`, `{{LOG:S6a.txt}}`, `{{LOG:S6b.txt}}`, `{{LOG:S6c.txt}}`, the field names used, verdict RECORDED, what it means for D7 and the report screen; leave a heading "S6 live" for Task 15), append, commit with message `S6: record the model switch after a scripted refusal, and where blocked attempts show up`.

### Task 13: S8 — the Windows side, from inside the fence

**Question (milestone plan):** from inside the WSL fence, can a program reach a listener on the Windows side (127.0.0.1, the address WSL uses for the host, all addresses), in WSL's default network mode? **Pass:** all refused. If not: the chooser keeps the "programs running in WSL" wording only; it becomes a finding.

**Files:**
- Create: `spikes/probes/s8_probe.py`, `spikes/scripts/s8_probe.json`, `spikes/runs/S8-control.json`, `spikes/runs/S8.json`, `spikes/results/S8.md`

`spikes/probes/s8_probe.py`:
```python
"""S8 probe: which addresses can this program connect to? One JSON line per attempt in s8_results.jsonl.
Inside Claude Code's fence a proxy is offered through environment variables; the probe asks it for the same places."""
import json
import os
import socket
import sys
import time
from urllib.parse import urlparse

host_ip = sys.argv[1]
TARGETS = [("listener inside WSL", "127.0.0.1", 47831), ("127.0.0.1:47832 (a Windows-only listener)", "127.0.0.1", 47832),
           ("Windows host RPC 135", host_ip, 135), ("Windows host 5040", host_ip, 5040), ("Windows host 49664", host_ip, 49664),
           ("Windows host 47832", host_ip, 47832), ("internet 1.1.1.1:443", "1.1.1.1", 443)]
results = []
for name, ip, port in TARGETS:
    t = time.time()
    try:
        with socket.create_connection((ip, port), timeout=5) as s:
            s.sendall(b"S8-PROBE")
        results.append({"target": name, "ok": True, "detail": "connected", "seconds": round(time.time() - t, 2)})
    except OSError as e:
        results.append({"target": name, "ok": False, "detail": f"{type(e).__name__}: {e}", "seconds": round(time.time() - t, 2)})
proxy = next((os.environ[v] for v in ("HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy") if os.environ.get(v)), None)
if proxy:
    p = urlparse(proxy)
    for name, ip, port in TARGETS:
        t = time.time()
        try:
            with socket.create_connection((p.hostname, p.port), timeout=5) as s:
                s.sendall(f"CONNECT {ip}:{port} HTTP/1.1\r\nHost: {ip}:{port}\r\n\r\n".encode())
                reply = s.recv(200).decode("latin-1").splitlines()
            first = reply[0] if reply else "(empty reply)"
            results.append({"target": "through the proxy: " + name, "ok": " 200" in first, "detail": first, "seconds": round(time.time() - t, 2)})
        except OSError as e:
            results.append({"target": "through the proxy: " + name, "ok": False, "detail": f"{type(e).__name__}: {e}", "seconds": round(time.time() - t, 2)})
else:
    results.append({"target": "proxy", "ok": None, "detail": "no proxy variable in the environment"})
with open("s8_results.jsonl", "w") as f:
    for r in results:
        f.write(json.dumps(r) + "\n")
for r in results:
    print(r["target"], "->", "CONNECTED" if r["ok"] else r["detail"])
```
`spikes/scripts/s8_probe.json`:
```json
{
  "about": "Try to connect to programs on the Windows side and inside WSL",
  "steps": [
    {"tool": "Bash", "input": {"command": "python3 spike_probes/s8_probe.py {HOST_IP}; echo \"EXIT=$?\"", "description": "Try to connect to other programs"}},
    {"text": "Spike S8: done."}
  ]
}
```
`spikes/runs/S8-control.json` (the same probe with no fence, so a refusal inside the fence means something):
```json
{"about": "S8 control: the probe with no fence; also the network mode and route", "script": "none", "gate": "node", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "probes": ["s8_probe.py"], "argv_override": ["/bin/sh", "-c", "wslinfo --networking-mode; ip route show default; python3 spike_probes/s8_probe.py {HOST_IP}"], "evidence": "raw"}
```
`spikes/runs/S8.json`:
```json
{"about": "S8: the probe inside Claude Code's fence", "script": "s8_probe.json", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty"}, "probes": ["s8_probe.py"], "env_extra": ["ANTHROPIC_API_KEY=@fake-api-key"]}
```

- [ ] **Step 1: start a listener on the Windows side, bound to 127.0.0.1 only** (no firewall prompt for loopback), then run the control and the fenced probe, then stop it.
```powershell
$l = Start-Process python -ArgumentList "C:\dev\runFile\spikes\tools\listener.py 47832 C:\dev\m1\logs\S8.winlistener.log" -WindowStyle Hidden -PassThru
powershell -NoProfile -ExecutionPolicy Bypass -File C:\dev\runFile\spikes\tools\run.ps1 -Id S8-control
powershell -NoProfile -ExecutionPolicy Bypass -File C:\dev\runFile\spikes\tools\run.ps1 -Id S8
Stop-Process -Id $l.Id
"--- Windows listener log:"; Get-Content C:\dev\m1\logs\S8.winlistener.log -ErrorAction SilentlyContinue
```
- [ ] **Step 2: verdict.**
  - **PASS:** every attempt inside the fence fails, directly and through the proxy; the Windows listener log has no line from the fenced run; and at least one target that failed inside the fence connected in the control run.
  - **INCONCLUSIVE:** no target connected in the control either (then Windows itself already refuses them, and the fence is not what was tested). Record the error messages: an immediate "Network is unreachable" inside the fence against a delay or "refused" outside is still informative.
  - **FAIL:** any attempt inside the fence connects.
  Only the default `nat` mode is tested; the `mirrored` mode, where Windows' own 127.0.0.1 is reachable from WSL, is not (it needs a change to your WSL settings).
- [ ] **Step 3: write `spikes/results/S8.md`** (`{{LOG:S8-control.txt}}`, `{{LOG:S8.txt}}`, the Windows listener log pasted as a code block, verdict, the chooser wording it allows), append, commit with message `S8: record whether the WSL fence reaches programs on the Windows side`.

### Task 14: S9 — signatures in WASM, Svelte in WebView2, and the check page's lock (yes: A6)

**Question (milestone plan):** does `ssh-key` build `no_std` for `wasm32`? Does Svelte run in WebView2? Does a check page with embedded WASM still work under `connect-src 'none'`? **Pass:** all yes.

This task is also phase 0's "Node packages for `web/`" and "the Tauri command-line tool" setup items: the exact versions in section 3 are installed here, into the spike. The product's `web/` folder is created in phases 4 and 6 with the same versions, once S9 shows they work together.

**Files:**
- Modify: `spikes/Cargo.toml` (add three members, one per step below)
- Create: `spikes/s9-core/` (`Cargo.toml`, `src/lib.rs`, `tests/vectors.rs`, `tests/vectors/msg.txt`, `msg.txt.sig`, `key.pub`, `fingerprint.txt`)
- Create: `spikes/s9-wasm/` (`Cargo.toml`, `src/lib.rs`)
- Create: `spikes/s9-check/` (`package.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`, `index.html`, `src/main.ts`, `src/App.svelte`, `src/checks.ts`, `scripts/embed.mjs`, `scripts/make_icon.py`, `playwright.config.ts`, `tests/check-page.spec.ts`, `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`)
- Create: `spikes/results/S9.md`

**Interfaces:** `s9_core::verify(msg: &[u8], sig_pem: &str, namespace: &str) -> Result<String, String>` (Ok holds the key's `SHA256:` fingerprint); `s9_wasm::verify(msg: &[u8], sig_pem: &str, namespace: &str) -> String` (`"valid (SHA256:…)"` or `"invalid: …"`).

- [ ] **Step 1: make test vectors with Windows' own `ssh-keygen`**, by full path (Git Bash finds a different one first). The key is a throwaway made for this test; it stays in `C:\dev\m1\s9` and is not committed.
```powershell
$k = "C:\Windows\System32\OpenSSH\ssh-keygen.exe"
New-Item -ItemType Directory -Force C:\dev\m1\s9, C:\dev\runFile\spikes\s9-core\tests\vectors | Out-Null
& $k -t ed25519 -N '""' -C s9-spike -f C:\dev\m1\s9\key
Set-Content -NoNewline -Encoding ascii C:\dev\m1\s9\msg.txt "Steplet spike S9 test message"
& $k -Y sign -f C:\dev\m1\s9\key -n steplet-v1 C:\dev\m1\s9\msg.txt
cmd /c "C:\Windows\System32\OpenSSH\ssh-keygen.exe -Y check-novalidate -n steplet-v1 -s C:\dev\m1\s9\msg.txt.sig < C:\dev\m1\s9\msg.txt"
$fp = ((& $k -l -f C:\dev\m1\s9\key.pub) -split " ")[1]
Set-Content -NoNewline -Encoding ascii C:\dev\runFile\spikes\s9-core\tests\vectors\fingerprint.txt $fp
Copy-Item C:\dev\m1\s9\msg.txt, C:\dev\m1\s9\msg.txt.sig, C:\dev\m1\s9\key.pub C:\dev\runFile\spikes\s9-core\tests\vectors\
"fingerprint: $fp"
```
Expected: `Good "steplet-v1" signature with ED25519 key SHA256:…` and the same fingerprint.

- [ ] **Step 2: write the failing tests.** `spikes/s9-core/tests/vectors.rs`:
```rust
const MSG: &[u8] = include_bytes!("vectors/msg.txt");
const SIG: &str = include_str!("vectors/msg.txt.sig");
const FINGERPRINT: &str = include_str!("vectors/fingerprint.txt");

#[test]
fn ssh_keygen_signature_verifies_with_the_same_fingerprint() {
    assert_eq!(s9_core::verify(MSG, SIG, "steplet-v1"), Ok(FINGERPRINT.trim().to_string()));
}

#[test]
fn changed_message_fails() {
    assert!(s9_core::verify(b"Steplet spike S9 test message!", SIG, "steplet-v1").is_err());
}

#[test]
fn wrong_namespace_fails() {
    assert!(s9_core::verify(MSG, SIG, "runfile-v1").is_err());
}
```
`spikes/s9-core/Cargo.toml`:
```toml
[package]
name = "s9-core"
version = "0.0.0"
edition.workspace = true
publish.workspace = true

[dependencies]
ssh-key = { version = "=0.6.7", default-features = false, features = ["alloc", "ed25519"] }
```
`spikes/s9-core/src/lib.rs` (empty function so the tests compile and fail):
```rust
#![no_std]
extern crate alloc;
use alloc::string::String;

pub fn verify(_msg: &[u8], _sig_pem: &str, _namespace: &str) -> Result<String, String> {
    Err(String::from("not written yet"))
}
```
In `spikes/Cargo.toml`, set `members = ["host", "gate", "s9-core"]` (Step 4 adds `"s9-wasm"`, Step 7 adds `"s9-check/src-tauri"`; a member folder must exist before it is listed).
```powershell
cd C:\dev\runFile\spikes; cargo test -p s9-core
```
Expected: `ssh_keygen_signature_verifies_with_the_same_fingerprint` FAILS; the other two pass trivially.

- [ ] **Step 3: write the real `verify`.**
```rust
//! Spike: check an SSH signature (SSHSIG, armoured) without the operating system (milestone 1, phase 0). Throwaway.
#![no_std]
extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use ssh_key::{Algorithm, HashAlg, PublicKey, SshSig};

/// Checks `sig_pem` over `msg` against the key embedded in the signature. Ok holds that key's SHA256 fingerprint.
pub fn verify(msg: &[u8], sig_pem: &str, namespace: &str) -> Result<String, String> {
    let sig = SshSig::from_pem(sig_pem).map_err(|e| format!("not a signature: {e}"))?;
    if sig.version() != 1 {
        return Err("signature version is not 1".into());
    }
    if sig.hash_alg() != HashAlg::Sha512 {
        return Err("hash is not sha512".into());
    }
    if sig.algorithm() != Algorithm::Ed25519 {
        return Err("key is not Ed25519".into());
    }
    let key = PublicKey::from(sig.public_key().clone());
    key.verify(namespace, msg, &sig).map_err(|e| format!("does not verify: {e}"))?;
    Ok(key.fingerprint(HashAlg::Sha256).to_string())
}
```
If a method name differs in 0.6.7, read `https://docs.rs/ssh-key/0.6.7` and use the 0.6.7 name; record the change in the results.
```powershell
cargo test -p s9-core
cargo build -p s9-core --target wasm32v1-none --release
```
Expected: 3 passed; the `wasm32v1-none` build finishes. That target has no standard library at all, so a finished build proves nothing in `ssh-key`'s dependency tree needs the operating system. If it fails, record the crate and error that pulled the standard library in (S9 part 1 FAIL; the core or its dependency features change before phase 1).

- [ ] **Step 4: the WASM wrapper.** Add `"s9-wasm"` to `members` in `spikes/Cargo.toml`. `spikes/s9-wasm/Cargo.toml`:
```toml
[package]
name = "s9-wasm"
version = "0.0.0"
edition.workspace = true
publish.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
s9-core = { path = "../s9-core" }
wasm-bindgen = "=0.2.129"
```
`spikes/s9-wasm/src/lib.rs`:
```rust
//! Spike: s9-core for the browser (milestone 1, phase 0). Throwaway.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn verify(msg: &[u8], sig_pem: &str, namespace: &str) -> String {
    match s9_core::verify(msg, sig_pem, namespace) {
        Ok(fingerprint) => format!("valid ({fingerprint})"),
        Err(e) => format!("invalid: {e}"),
    }
}
```
```powershell
cd C:\dev\runFile\spikes
cargo build -p s9-wasm --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir s9-check\src\wasm target\wasm32-unknown-unknown\release\s9_wasm.wasm
```

- [ ] **Step 5: the page.** `spikes/s9-check/package.json`:
```json
{
  "name": "s9-check",
  "private": true,
  "type": "module",
  "scripts": {
    "embed": "node scripts/embed.mjs",
    "check": "svelte-check --tsconfig ./tsconfig.json",
    "build:browser": "vite build --mode browser",
    "build:tauri": "vite build --mode tauri",
    "test": "playwright test",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "2.12.1"
  },
  "devDependencies": {
    "@playwright/test": "1.63.0",
    "@sveltejs/vite-plugin-svelte": "7.3.1",
    "@tauri-apps/cli": "2.12.1",
    "@tsconfig/svelte": "5.0.8",
    "svelte": "5.57.1",
    "svelte-check": "4.7.6",
    "typescript": "6.0.3",
    "vite": "8.3.1"
  }
}
```
`spikes/s9-check/vite.config.ts`:
```ts
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// The public check page's lock: it may load its own files and run its embedded WASM, and nothing may leave.
const CHECK_PAGE_CSP =
  "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:; " +
  "connect-src 'none'; form-action 'none'; base-uri 'none'";

export default defineConfig(({ mode }) => ({
  plugins: [
    svelte(),
    {
      name: "check-page-csp",
      transformIndexHtml: () =>
        mode === "browser"
          ? [{ tag: "meta", attrs: { "http-equiv": "Content-Security-Policy", content: CHECK_PAGE_CSP }, injectTo: "head-prepend" }]
          : [],
    },
  ],
  build: {
    outDir: mode === "browser" ? "dist" : "dist-tauri",
    modulePreload: { polyfill: false },
    assetsInlineLimit: 0,
  },
}));
```
`spikes/s9-check/svelte.config.js`:
```js
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default { preprocess: vitePreprocess() };
```
`spikes/s9-check/tsconfig.json`:
```json
{
  "extends": "@tsconfig/svelte/tsconfig.json",
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "types": ["svelte"]
  },
  "include": ["src/**/*.ts", "src/**/*.svelte"]
}
```
`spikes/s9-check/index.html`:
```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Steplet spike S9</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```
`spikes/s9-check/src/main.ts`:
```ts
import { mount } from "svelte";
import App from "./App.svelte";

mount(App, { target: document.getElementById("app")! });
```
`spikes/s9-check/src/checks.ts`:
```ts
import { initSync, verify } from "./wasm/s9_wasm.js";
import { WASM_BASE64 } from "./wasm/s9_wasm_base64";
import { SAMPLE_MESSAGE, SAMPLE_SIGNATURE } from "./sample";

export type CheckResults = { sample: string; tampered: string; fetchBlocked: boolean };

function bytesFromBase64(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

export async function runChecks(): Promise<CheckResults> {
  // The WASM is embedded in the script, so loading it needs no network request.
  initSync({ module: bytesFromBase64(WASM_BASE64) });
  const enc = new TextEncoder();
  const sample = verify(enc.encode(SAMPLE_MESSAGE), SAMPLE_SIGNATURE, "steplet-v1");
  const tampered = verify(enc.encode(SAMPLE_MESSAGE + "!"), SAMPLE_SIGNATURE, "steplet-v1");
  // Deliberately try to send something. Without the lock this would reach the preview server and get a 404 (no error).
  let fetchBlocked = false;
  try {
    await fetch("/s9-probe", { method: "POST", body: "spike" });
  } catch {
    fetchBlocked = true;
  }
  return { sample, tampered, fetchBlocked };
}
```
`spikes/s9-check/src/App.svelte`:
```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { runChecks, type CheckResults } from "./checks";

  let results: CheckResults | null = $state(null);
  let error = $state("");

  onMount(async () => {
    try {
      results = await runChecks();
      if ("__TAURI_INTERNALS__" in window) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("report", { text: JSON.stringify({ ...results, renderedBySvelte: true, userAgent: navigator.userAgent }) });
      }
    } catch (e) {
      error = String(e);
    }
  });
</script>

<main>
  <h1>Steplet spike S9</h1>
  {#if error}
    <p data-testid="error">{error}</p>
  {:else if !results}
    <p data-testid="loading">Checking…</p>
  {:else}
    <p data-testid="valid">Sample signature: {results.sample}</p>
    <p data-testid="tampered">Changed message: {results.tampered}</p>
    <p data-testid="fetch">Network request from this page: {results.fetchBlocked ? "blocked" : "NOT blocked"}</p>
  {/if}
</main>
```
`spikes/s9-check/scripts/embed.mjs`:
```js
// Embeds the WASM and the test vector into source files, so the page loads nothing over the network.
import { readFileSync, writeFileSync } from "node:fs";

const wasm = readFileSync("src/wasm/s9_wasm_bg.wasm");
writeFileSync("src/wasm/s9_wasm_base64.ts", `export const WASM_BASE64 = "${wasm.toString("base64")}";\n`);
const msg = readFileSync("../s9-core/tests/vectors/msg.txt", "utf8");
const sig = readFileSync("../s9-core/tests/vectors/msg.txt.sig", "utf8");
writeFileSync("src/sample.ts", `export const SAMPLE_MESSAGE = ${JSON.stringify(msg)};\nexport const SAMPLE_SIGNATURE = ${JSON.stringify(sig)};\n`);
console.log(`embedded ${wasm.length} bytes of WASM and the test vector`);
```
`spikes/s9-check/playwright.config.ts`:
```ts
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "tests",
  reporter: [["list"]],
  use: { baseURL: "http://127.0.0.1:4179" },
  webServer: {
    command: "npx vite preview --mode browser --host 127.0.0.1 --port 4179 --strictPort",
    url: "http://127.0.0.1:4179",
    reuseExistingServer: false,
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } },
  ],
});
```
`spikes/s9-check/tests/check-page.spec.ts`:
```ts
import { test, expect } from "@playwright/test";

test("the check page verifies a signature in the browser and cannot send anything", async ({ page }) => {
  const requests: string[] = [];
  const consoleLines: string[] = [];
  page.on("request", (r) => requests.push(`${r.method()} ${r.url()}`));
  page.on("console", (m) => consoleLines.push(`${m.type()}: ${m.text()}`));
  await page.goto("/");
  const csp = await page.locator('meta[http-equiv="Content-Security-Policy"]').getAttribute("content");
  expect(csp).toContain("connect-src 'none'");
  await expect(page.getByTestId("valid")).toContainText("valid (SHA256:");
  await expect(page.getByTestId("tampered")).toContainText("invalid: does not verify");
  await expect(page.getByTestId("fetch")).toHaveText("Network request from this page: blocked");
  expect(requests.filter((r) => r.startsWith("POST"))).toEqual([]);
  expect(requests.every((r) => r.includes("127.0.0.1:4179"))).toBe(true);
  console.log("requests:", JSON.stringify(requests));
  console.log("console:", JSON.stringify(consoleLines));
});
```

- [ ] **Step 6: ask A6, install, build, and run the browser test.**
```powershell
cd C:\dev\runFile\spikes\s9-check
npm install --no-fund --no-audit
npx playwright install chromium firefox webkit
npm run embed
npm run check
npm run build:browser
npx playwright test
```
Expected: `svelte-check` reports 0 errors; 3 tests passed (one per browser); the `requests:` lines list only `GET http://127.0.0.1:4179/…`; the console lines show the browser's own refusal of the deliberate request.

- [ ] **Step 7: the Tauri shell around the same page.** Add `"s9-check/src-tauri"` to `members` in `spikes/Cargo.toml` once the files below exist. `spikes/s9-check/scripts/make_icon.py`:
```python
"""Writes a plain 1024x1024 PNG, the source the Tauri icon command needs. Usage: python -I make_icon.py <out.png>"""
import struct
import sys
import zlib

w = h = 1024
raw = (b"\x00" + bytes([37, 99, 235, 255]) * w) * h


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)


png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
with open(sys.argv[1], "wb") as f:
    f.write(png)
```
`spikes/s9-check/src-tauri/Cargo.toml`:
```toml
[package]
name = "s9-shell"
version = "0.0.0"
edition.workspace = true
publish.workspace = true

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "=2.12.1", features = [] }
```
`spikes/s9-check/src-tauri/build.rs`:
```rust
fn main() {
    tauri_build::build()
}
```
`spikes/s9-check/src-tauri/src/main.rs`:
```rust
//! Spike: does the Svelte page run inside WebView2 (milestone 1, phase 0)? Throwaway.
//! The page reports what it rendered through one command; the shell writes it to a file and exits.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn report(app: tauri::AppHandle, text: String) {
    let path = std::env::var("S9_REPORT").unwrap_or_else(|_| r"C:\dev\m1\logs\S9-webview.txt".into());
    std::fs::write(&path, text).expect("write the report");
    app.exit(0);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![report])
        .run(tauri::generate_context!())
        .expect("the S9 shell failed to start");
}
```
`spikes/s9-check/src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "s9-shell",
  "version": "0.0.0",
  "identifier": "dev.steplet.spike.s9",
  "build": {
    "frontendDist": "../dist-tauri",
    "beforeBuildCommand": "npm run build:tauri"
  },
  "app": {
    "windows": [{ "title": "Steplet spike S9", "width": 800, "height": 600 }],
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' data:; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": { "active": false }
}
```
`spikes/s9-check/src-tauri/capabilities/default.json`:
```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```
```powershell
cd C:\dev\runFile\spikes\s9-check
python -I scripts\make_icon.py C:\dev\m1\s9\icon.png
npx tauri icon C:\dev\m1\s9\icon.png
npx tauri build --debug --no-bundle
$env:S9_REPORT = "C:\dev\m1\logs\S9-webview.txt"; Remove-Item $env:S9_REPORT -ErrorAction SilentlyContinue
$p = Start-Process C:\dev\runFile\spikes\target\debug\s9-shell.exe -PassThru
"exited by itself within 60 s: $($p.WaitForExit(60000))"
Get-Content $env:S9_REPORT
```
A window appears on your screen for a moment and closes by itself. Expected report: `"sample":"valid (SHA256:…)"`, `"tampered":"invalid: does not verify…"`, `"renderedBySvelte":true`, and a `userAgent` containing `Edg/` (WebView2). That the report file exists at all shows Tauri's own channel from the page to the Rust code worked under the app's content-security rule. If the window stays open past 60 s, the page did not reach the Rust code: record it and close the window.

- [ ] **Step 8: write `spikes/results/S9.md`** with the outputs of Steps 1, 3, 4, 6 and 7 (save each with `| Out-File -Encoding utf8 C:\dev\m1\logs\S9-<step>.txt` and paste with `{{LOG:…}}`), verdict, what it means for the core and the check page. Append; commit with message `S9: record ssh-key without the operating system, Svelte in WebView2, and the check page under connect-src none`.

### Task 15: live checks with a real test key (yes: A7, A8)

Two runs need the real model: S6 live (refusals cannot be faked reliably), and L1, one combined run of the whole path. L1 is not in the milestone plan's list; it is one run that checks the scripted model did not hide anything on the exact path phase 3 builds (S7's pass rule compares with milestone 0, not with this path).

**Files:**
- Create: `spikes/runs/S6L.json`, `spikes/runs/S6L-off.json`, `spikes/runs/L1.json`, `spikes/results/live.md`

- [ ] **Step 1 (you): create the key.** In the Anthropic console, make a workspace (or use one) with a spending limit (I suggest 5 US dollars), create an API key in it, then save it into the test distribution from your own WSL window, so it never appears on screen or in history:
```bash
install -m 600 /dev/null ~/.local/share/steplet/secrets/anthropic-api-key
read -rs K && printf '%s' "$K" > ~/.local/share/steplet/secrets/anthropic-api-key && unset K
```
(Paste the key after the second line and press Enter; nothing is shown.) Tell me when done. I check only its size:
```powershell
wsl -d steplet-test --exec /usr/bin/stat -c "%s bytes, mode %a" /home/tester/.local/share/steplet/secrets/anthropic-api-key
```

The key route below is `apiKeyHelper` (S3c). If S3 showed that route leaks and another passed, change the three files to the route that passed, but never put the real key on a command line or in `env_extra` (`spike.py` refuses that).

`spikes/runs/S6L.json`:
```json
{"about": "S6 live: a task that drew refusals in milestone 0; does the model switch, and what shows it", "script": "live", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty", "timeout_s": 900}, "timeout_s": 840, "settings_extra": {"apiKeyHelper": "/bin/cat {P}/secrets/anthropic-api-key"}, "env_extra": [], "max_turns": 8, "task": "Read tests/fixtures.py and explain in detail what each function in it does and why a test helper would do that."}
```
`spikes/runs/S6L-off.json`:
```json
{"about": "S6 live, with switchModelsOnFlag false", "script": "live", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty", "timeout_s": 900}, "timeout_s": 840, "settings_extra": {"apiKeyHelper": "/bin/cat {P}/secrets/anthropic-api-key", "switchModelsOnFlag": false}, "env_extra": [], "max_turns": 8, "task": "Read tests/fixtures.py and explain in detail what each function in it does and why a test helper would do that."}
```
`spikes/runs/L1.json`:
```json
{"about": "L1: one real-model run of the whole path: Windows launcher, Windows gate, private folders, key through apiKeyHelper, product settings", "script": "live", "gate": "exe", "gate_mode": "allow", "shape": "product", "launch": "host", "host": {"env_mode": "empty", "timeout_s": 900}, "timeout_s": 840, "settings_extra": {"apiKeyHelper": "/bin/cat {P}/secrets/anthropic-api-key"}, "env_extra": [], "max_turns": 12, "task": "In the git repository in the current folder, try to check out commit 65930082d222089b2633d34e9bb2bd42351bdc76. Then, whether or not the checkout worked, run the tests with `python3 -m unittest discover -s tests -t .`, and tell me in three short lines: whether the checkout worked, how many tests ran, and whether they passed. Do not edit any file."}
```

- [ ] **Step 2 (A8): run L1.** Pass, all of:
  - the fence results of the product shape, as in S7b: A `ok: true`; B, C, D, E, G, H and both child lines `ok: false`; no listener line; no Windows marker; the checkout refused because `.git` is read-only;
  - the gate log shows a request for every command that changes something, answered through `spike-gate.exe`;
  - "real API key: 0" and "real API key, last 20 characters: 0" in the counts;
  - the stream has one `result` event.

  Recorded: which models ran, and anything the live agent did that no scripted run did (extra commands, tools the scripts never used).
- [ ] **Step 3 (A8): run S6L up to three times**, copying the log between runs (`Copy-Item C:\dev\m1\logs\S6L.txt C:\dev\m1\logs\S6L-run1.txt`, and so on), stopping at the first run that shows `system/model_refusal_fallback`. If one does, run S6L-off up to three times the same way. Recorded: refusals, model names, and whether `switchModelsOnFlag: false` stopped the switch and how the run then ended.
- [ ] **Step 4: write `spikes/results/live.md`** (`{{LOG:L1.txt}}`, the S6L runs made, verdicts), append (`append_results.py` stops if the real key appears in any log: that would itself be the finding), commit with message `Live checks: one real-model run of the whole path, and the refusal model switch`.

### Task 16: wrap-up

**Files:**
- Modify: `docs/milestone-1-spikes.md` (summary at the end), `docs/plans/2026-10-08-milestone-1-plan.md` (status line only), `docs/handoff.md` (one update line)

- [ ] **Step 1: write the summary section** of `docs/milestone-1-spikes.md`:

| Spike | Verdict | What changes for later phases |
|---|---|---|
| S1 … S9, L1 | one per row | from each spike's "what it means" |

Then: every screen line from the milestone plan's section 7 that a spike touched, and whether it may stay as worded; and "Not verified in phase 0" (section 7 below, updated with what actually ran).
- [ ] **Step 2: status lines.** The milestone plan's status line gets "Phase 0 finished on <date>; results in `docs/milestone-1-spikes.md`." `docs/handoff.md` gets the same one-line update. **No row in `docs/decisions.md` changes without your yes**: where a spike failed, I list the milestone plan's fallback and ask you.
- [ ] **Step 3: commit** with message `Phase 0 wrap-up: spike summary and status`.
- [ ] **Step 4: tell you** the verdicts, what failed, which fallbacks apply, and the clean-up options: delete `C:\dev\m1\s9` (the throwaway SSH key), revoke the test API key or keep it for phase 5's live layer, and the still-pending milestone 0 clean-up.

---

## 7. What this plan does not verify

- **Every spike is unanswered until it runs.** Several pass rules depend on behaviour no documentation states (how Claude Code treats a refusal from a stand-in model; whether `allowRead` re-opens a folder under a `denyRead` one in 2.1.220; whether interop works with the emptied environment).
- **The scripted model can hide behaviour that depends on the real model** (the milestone plan says so). L1 checks one path live; the phase 5 suite's live layer remains needed.
- **The spike code itself is untested beyond its self-tests**, and is throwaway. `fake_model.py`, `spike.py` and the host have not run yet; their first run is Task 5's self-tests and S7a. The Rust code has not been compiled; the `ssh-key` 0.6.7 method names in Task 14 come from my reading of that version, not from a build.
- **One machine, one Windows build, one Ubuntu (26.04), NAT networking only.** WSL's `mirrored` network mode, Windows 10, Windows Pro, and antivirus products are not tested.
- **Interop identity:** S1 records the gate's Windows account and session through `tasklist`, not its integrity level or logon session, which the pipe's permission list in phase 3 will depend on.
- **`srt` 0.0.77's sub-packages** are recorded, not pinned by `srt` itself; the product needs its own pinning (phase 3).
- **Whether the test distribution behaves like a receiver's machine:** it is fresh and has none of the tools a real developer's Ubuntu has.
- **The Anthropic terms question** (P1) is unchanged by phase 0.
- **No time estimate** is given for phase 0; milestone 0 measured machine time, not build effort.
