# Runfile: what exists, is it feasible, how to build it

Researched 6 Oct 2026. Five parallel investigations, about 100 searches, roughly 70 primary pages opened.
Tags: **[opened]** = the page itself was read. **[unverified]** = seen only in a search summary or a secondary site. Treat unverified items as leads, not facts.

Written in plain words. Jargon is explained the first time it appears.

---

## 1. The map: how this territory divides

1. **Instruction packs that live in a folder** (Agent Skills, plugins, custom agents, AGENTS.md). Huge, standardised, adopted by 40+ agents. Not designed to be *sent to a person*; the receiver must already have the folder.
2. **One-click install links** (Cursor and VS Code install links, Claude Desktop `.mcpb` bundles, Claude Code `claude-cli://` links). These are the only existing "send someone a thing that opens in their agent" primitives. They install a server or pre-fill a prompt. None carries a guarded task.
3. **Cloud task sharing** (Zapier agent templates, ChatGPT scheduled tasks). Preview-then-copy exists, but the agent runs in the vendor's cloud, not on your machine.
4. **Scanners** (Snyk agent-scan, Cisco skill-scanner, Invariant mcp-scan). Scan skills and MCP servers for injected instructions and malware. No runner, no file format, no receiver screen.
5. **Guards and sandboxes** (Claude Code permission rules and hooks, Codex's Windows sandbox, Microsoft's experimental sandbox API). The enforcement pieces exist, mostly inside each agent.
6. **Desktop agents for non-technical people** (Claude Cowork, Codex app, Windows Copilot Actions). They exist, but **none can be launched by an outside program with a task**. This is the single most important constraint found.

**Headline:** no product was found that combines (a) a task file sent to a person, (b) a preview with a risk rating, (c) a choice among agents installed on the receiver's machine, and (d) per-action enforcement. The pieces all exist separately. "Not found" after about 100 searches is a stronger claim than the brief's "about a dozen".

---

## 2. Entries

### 2a. Instruction packs in folders (the standard we should be compatible with)

| Thing | What it is | Why it matters to us | Date / source |
|---|---|---|---|
| **Agent Skills standard** (Anthropic, open standard) | A folder with `SKILL.md` (YAML header + Markdown body) plus optional `scripts/`, `references/`. Header has `name`, `description`, and an experimental `allowed-tools` field. | Adopted by Claude Code, Codex, Copilot, VS Code, Cursor, Gemini CLI, and 35+ others. `allowed-tools` is **pre-approval** (fewer prompts), **not a restriction**. If our file is a valid SKILL.md with extra header fields, every one of those agents can read it when dropped in a folder. | agentskills.io/specification [opened]. Launch date Dec 2025 [unverified]. |
| Claude Code skills | Same layout plus extra fields: `disallowed-tools` (really removes tools), `hooks`, `shell: powershell`. | Shows that a per-file "disallowed" list is already a concept in one agent. | code.claude.com/docs/en/skills [opened] |
| Claude Code sub-agents (`.claude/agents/*.md`) | A Markdown file with `tools` allowlist, `disallowedTools`, `permissionMode`, `maxTurns`, `hooks`. | **The one existing format that carries a real per-tool allowlist plus a permission mode.** But it is a repo or user file, not something you send. | code.claude.com/docs/en/sub-agents [opened] |
| Codex skills | Same folder layout; docs say skills "don't restrict permissions, those are configured at the host level". | Confirms: limits must come from the runner, not the file. | learn.chatgpt.com/docs/build-skills [opened] |
| Gemini CLI skills | `gemini skills install <git url>` shows a **consent screen** (name, purpose, path) before activating. | A shipped precedent for "preview before install", but at install time, not per action. | geminicli.com/docs/cli/skills [opened] |
| Vercel `npx skills add` + skills.sh | A CLI that writes a skill into 17 agents' folders at once; skills.sh is a leaderboard. | This is where the "install this dev tool" use case is going. It is replacing paste-a-prompt. | vercel.com/changelog Jan 20 2026 [opened] |
| AGENTS.md | Plain Markdown repo file read by 25+ agents, 60k+ repos. | Repo artifact only. | agents.md [opened] |
| VS Code `.prompt.md` | Prompt files with header (agent, model, tools). VS Code's own page says they are deprecated in favour of skills. | Even Microsoft is converging on skills. | code.visualstudio.com [opened, updated 30 Sep 2026] |

### 2b. One-click install links (the "send" primitives that exist)

| Thing | What happens on click | Lesson for us | Source |
|---|---|---|---|
| **Claude Code deep links** `claude-cli://open?q=<prompt>` | Opens a terminal, pre-fills the prompt, does **not** send it until Enter, shows a "Prompt from an external link" warning. Registered per user on Windows under `HKCU\Software\Classes\claude-cli`. Max 5,000 characters. GitHub strips the scheme from Markdown. | **Anthropic already ships "send someone a link with a task."** No file, no risk rating, no agent choice, no guard beyond normal permissions. This is the closest thing from an agent maker, and the thing they could extend. | code.claude.com/docs/en/deep-links [opened] |
| Claude Desktop `.mcpb` bundles | Zip with `manifest.json`; double-click opens Claude Desktop with an install dialog listing required permissions and configuration. | The double-click-to-preview pattern on Windows, proven. Installs a server, not a task. | github.com/modelcontextprotocol/mcpb MANIFEST.md v0.3 (2 Dec 2025), anthropic.com/engineering/desktop-extensions [opened] |
| Cursor MCP install links `cursor://anysphere.cursor-deeplink/mcp/install?...` | Cursor shows a confirm dialog with server name and args. | **"DeepJack" (reported 27 Apr 2026, still working in Cursor 3.9.8 per Adversa, 15 Jul 2026):** the dialog shows args in a single-line box, so padded text scrolls the dangerous part off-screen. A confirm screen that can be gamed is worse than none. Our preview must show the whole allowed list, wrapped, nothing truncated. | adversa.ai blog [opened]; Cursor doc page [unverified] |
| VS Code MCP install links | `https://insiders.vscode.dev/redirect/mcp/install?...` badges on GitHub READMEs. | Same pattern, web redirect into the app. | github.com/github/github-mcp-server [opened] |
| Munder Difflin `.hire.json` | A file describing an agent *role* (provider, model, flags, goal, token budget). Opened via `munderdifflin://hire?src=` or file import into an "Add agent" screen with an "imported" banner. Importing never spawns anything. | **Closest prior art found.** Sendable file + preview + human gate. But it describes a role, not a task, and has no per-action guard. | munderdiffl.in/blog/shareable-agent-roles, 15 Jun 2026 [opened] |

### 2c. Cloud task sharing

| Thing | Can a task be sent to another person? | Source |
|---|---|---|
| Zapier Agents | Yes. "Generate template link" gives a public link; the receiver sees a preview of instructions, actions and knowledge, then clicks "Use this template" to make their own copy. Cloud only. | help.zapier.com, updated 1 Oct 2026 [opened] |
| ChatGPT scheduled tasks | Share link is a snapshot; receiver reviews and creates a copy. | [unverified, help pages returned 403] |
| Claude Cowork | "Sessions can't be shared with others." Scheduled tasks are not shareable. Plugins are the shareable unit. | support.claude.com article 13345190 [opened] |
| Microsoft Copilot Studio agent flows | "can't be copied, shared, have co-owners, or give run-only permissions." | learn.microsoft.com, 26 May 2026 [opened] |
| Google Gemini scheduled actions | No share or export. | support.google.com [opened] |
| Relay, Lindy, Bardeen, Tango, Rabbit teach mode | Template or lesson links exist; cloud or browser only. | all [unverified] |

### 2d. Security: what actually happened to shared agent instructions

| Incident | Numbers and mechanism | Source |
|---|---|---|
| **ClawHavoc** (Koi Security, about 1-2 Feb 2026) | **341 of 2,857** ClawHub skills malicious; 335 in one campaign delivering the Atomic Stealer on macOS; Windows variant was a keylogging trojan in a fake "prerequisite" download. Credential theft from `~/.clawdbot/.env` to webhook.site. Mostly real malware behind fake install steps, not clever prompt injection. | The Hacker News 2 Feb 2026 [opened]; Koi's own post now redirects to Palo Alto Networks [primary unverified] |
| **ToxicSkills** (Snyk, 5 Feb 2026) | 3,984 skills from ClawHub and skills.sh: **534 (13.4%)** with at least one critical issue, **76 confirmed malicious** after human review, 10.9% with hard-coded secrets. The "1,467 malicious" number in reposts is wrong; 1,467 is "any flaw". | snyk.io/blog/toxicskills [opened] |
| The brief's "1,103 of 14,706" | **Not found anywhere.** Possible origin: arXiv 2604.06550 "SkillSieve" [unverified]. Do not cite it until checked. | |
| Rules File Backdoor (Pillar Security, 18 Mar 2025) | Invisible Unicode characters in `.cursor/rules` and Copilot instruction files steer the model. Cursor's reply: "falls under the users' responsibility". GitHub's reply: users are responsible for reviewing suggestions. | pillar.security blog [opened] |
| Malicious npm packages that install Claude Code hooks (SafeDep, 13 May 2026) | Packages shipped a project `.claude/settings.json` registering a `SessionStart` hook, so the payload runs every time the project opens. | safedep.io [opened]. **Direct lesson: our runner must not let the receiver's working folder inject hooks into the run.** |
| CopyPasta (HiddenLayer, Sept 2025) | Malicious instructions disguised as a licence header, copied by agents into every repo they touched. | hiddenlayer.com follow-up 28 Jul 2026 [opened]; original [unverified] |
| GitHub MCP injection via public issue (Invariant, May 2025) | Agent reads a malicious issue, leaks private repo data into a public PR. | [unverified] |

**Scanners that exist** (all open source): Snyk agent-scan (prompt injection, malware, credential handling; needs a Snyk API token), Cisco skill-scanner (seven analysers incl. YARA and LLM; `--fail-on-severity high`), Invariant mcp-scan (tool poisoning, rug pulls via hashing). Several unrelated repos use the name "skillscan". None is a runner or a file format.

**Anthropic's own guidance** [opened, platform.claude.com skills overview]: "Use Skills only from trusted sources", "Treat like installing software", skills "have the same network access as any other program on the user's computer". Enterprise-only skill scanning exists for claude.ai and Cowork uploads.

**OWASP** [opened, LLM06:2025 Excessive Agency]: "Implement authorization in downstream systems rather than relying on an LLM to decide if an action is allowed or not." and "require a human to approve high-impact actions before they are taken." This is the exact principle behind a code-enforced allowed list. The 2026 Agentic Top 10 (published 9 Dec 2025) lists Goal Hijack, Tool Misuse, Supply Chain, Unexpected Code Execution, etc. [list unverified against the PDF].

### 2e. Guards and sandboxes: what we can actually enforce

**Claude Code, all [opened] at code.claude.com/docs:**

- `claude -p "..."` runs headless (no screen). Prompt can come from stdin (10 MB cap).
- `--settings '<json>'` passes **per-run settings including permission rules and hooks** without touching the receiver's own config.
- Permission rules: `Bash(npm install *)`, `Read(~/.ssh/**)`, `Edit(path)`, `Write(path)`, `WebFetch(domain:example.com)`. Order: **deny beats ask beats allow**; a deny cannot be overridden by a more specific allow.
- Hooks before each action (`PreToolUse`) see the exact command, file path or URL, and can return allow or deny. In `-p` mode a hook **cannot** return "ask".
- **`--permission-prompt-tool <mcp tool>`**: in `-p` mode, every permission question is sent to a tool our app provides, which answers allow or deny. **This is how "pause and show the receiver" works without the Agent SDK.**
- `--output-format stream-json --include-hook-events` gives a live event stream for the log. Final `json` result has `total_cost_usd`, `session_id`, `is_error`, `stop_reason`, and with `--json-schema` a `structured_output` object (for example `{status, failed_step}`).
- `--resume <session_id>` works in `-p` mode: resume after a failed step is supported.
- `--max-turns`, `--max-budget-usd` cap a run. `--add-dir` loads skills from any folder for one run. `--bare` skips hooks, skills, plugins and CLAUDE.md (defence against the SafeDep attack above; whether `--bare` also drops hooks passed via `--settings` is **not established**).
- `claude --version` prints the version; native install lives at `%USERPROFILE%\.local\bin\claude.exe`.
- Hooks see only the agent's own actions, never what a launched program does. **No sandbox on native Windows**; the sandbox needs WSL2, covers shell commands only, and "MCP servers and command hooks run unconstrained on the host".
- **Auth constraint:** the Agent SDK is API-key only, and the docs say "Anthropic does not allow third party developers to offer claude.ai login or rate limits for their products". Launching the receiver's own installed `claude` binary is a different thing from embedding the SDK, and `claude-cli://` links are Anthropic's own "external program hands Claude Code a task" feature, but **whether a third-party app may drive `claude -p` on the receiver's subscription is not confirmed from a terms page. Must be checked before launch.**

**Codex:** official install page says Windows 11 **via WSL2 only** [opened github.com/openai/codex docs/install.md]. But a Codex *desktop app* for Windows with "a native agent sandbox (restricted tokens, filesystem ACLs, dedicated sandbox users)" was announced 4 Mar 2026 [opened community.openai.com]. Non-interactive `codex exec` with `--sandbox read-only|workspace-write|danger-full-access` [opened config reference]. Codex hooks: a `PreToolUse`-style hook that can deny but not ask [unverified, third-party source only]. The two Windows statements conflict; needs a fresh check if Codex is adopted.

**Windows isolation options:**
- Windows Sandbox: a virtual machine, Pro/Enterprise/Education only, not Home. Pointless for "install on my real machine" tasks.
- WSL2 + Claude Code's own sandbox: real file and network limits, but tasks then run in Linux, not Windows.
- AppContainer and Job Objects: lightweight, but AppContainer needs capability setup and is not a path allowlist on its own.
- **`Experimental_CreateProcessInSandbox`** (Microsoft, page dated 1 Jun 2026): a first-party Windows 11 API that spawns a child with `fs_read_write` / `fs_read_only` path lists and a network proxy policy, but only when `app_container` is true. Experimental, "subject to change", not in public headers. The right thing for a future strict mode, not for version 1. [opened learn.microsoft.com]
- How OpenAI did it: two dedicated local users and firewall rules bound to them, because "firewall rules cannot match a restricted token's synthetic identity"; ACL setup "is slow on big workspace trees". [secondary, bex.co 26 Sep 2026; OpenAI's post returned 403]

### 2f. Desktop agents for non-technical receivers (phase 2)

| Agent | Runs locally on Windows? | Can an outside program hand it a task? | Source |
|---|---|---|---|
| Claude Cowork + computer use | Yes, Pro/Max, beta. | **No.** No API, CLI or deep link documented. "Sessions can't be shared." | support.claude.com 13345190 and 14128542, claude.com blog 23 Mar 2026 [opened] |
| Claude Code computer use | Desktop app only on Windows; CLI version is macOS only and "not available in non-interactive mode with the -p flag". | **No** for screen-control tasks. | code.claude.com/docs/en/computer-use and /desktop [opened] |
| Claude in Chrome | Browser only; Pro and up. Can be driven from Claude Code with `--chrome`. | Partly, via Claude Code, interactive. | support.claude.com 12012173 [opened] |
| Anthropic computer use via API | Needs an API key and "a sandboxed computing environment" you provide. | Yes, but not a consumer path. | platform.claude.com computer-use-tool [opened] |
| OpenAI Codex app (Windows, 4 Mar 2026) | Yes, with native sandbox; "Windows support for Computer Use" added June 2026. | Not documented. | community.openai.com [opened]; help pages 403 |
| Windows 11 Copilot Actions / agent workspace | Insider preview only (pages dated Oct-Dec 2025), off by default, agents "must be signed by a trusted source". | Not yet; Microsoft says the platform "will be available for other developers in preview soon". | support.microsoft.com, learn.microsoft.com [opened] |
| UI-TARS Desktop (ByteDance) | Yes, open source, 39k stars. Needs its own model endpoint. | Not documented. | github [opened] |
| Browser Use | Python library, 117k stars. | Yes, by design, but developers only. | github [opened] |

**Finding:** for a non-technical person on Windows today, there is no agent our app could start with a task. Phase 2 is blocked on the vendors, not on us.

### 2g. Benchmarks (how often tasks succeed)

- OSWorld (v1) human baseline: **72.36%** of tasks [opened, osworld-v1.xlang.ai abstract].
- OSWorld-Verified best scores: **86.1%** (Qwen3.8 Max), 85% (Claude Fable 5 and Mythos 5), per aggregator benchlm.ai updated 5 Oct 2026 [unverified against the official page, which loads by script]. **No 90% figure found.** The brief's "86-90%" should be "about 85%".
- **OSWorld 2.0**: 108 long tasks, average 318 tool calls each, humans take a median 1.6 hours. Best agent (Claude Opus 4.8, 500 steps) completes **20.6%**, 54.8% partial credit. [opened arxiv.org/abs/2606.29537, July 2026]. The brief's number is right.

### 2h. Demand signals (is anyone asking for this?)

- "Paste this prompt into your coding agent" pages confirmed first-hand: **2** (Arize AX, NVIDIA NemoClaw). Tigris uses plugin install commands, not a prompt. Stripe and Supabase use `npx skills add` / `claude plugin install` [unverified].
- Cline's `llms-install.md` convention: **736 files** on GitHub by code search today (Metabase, Repomix, Microsoft MCP, Snyk, etc.) [opened]. Agent-readable install instructions are a real, if niche, practice.
- **No high-scoring Hacker News thread about paste-and-go risk was found.** The debate the brief assumes is not visible in HN data. Reddit was not searched.
- **Attention calibration** (GitHub, 6 Oct 2026): openclaw 391,440 stars (created Nov 2025); browser-use 117,207; open-interpreter 68,511; UI-TARS-desktop 39,205; Agent-S 12,540. A Windows desktop-control CLI ("Hands", Show HN 23 Aug 2026) got 5 points.

### 2i. Building it on Windows

| Choice | Fact | Source |
|---|---|---|
| **Tauri 2** | `bundle.fileAssociations` in config; deep-link plugin registers the URL scheme per user under HKCU (no admin); single-instance plugin delivers the file path or URL as `args`; shell plugin streams a child's stdout line by line; NSIS installer `installMode: "currentUser"` needs no admin; installers 3-15 MB. | v2.tauri.app config, deep-linking, single-instance, shell, windows-installer [opened] |
| Electron | File associations in electron-builder **only work with `nsis.perMachine = true`** (admin install). `open-url` is not supported on Windows; parse `process.argv`. Installers 85-250 MB. | electron.build, electronjs.org [opened] |
| .NET WinUI 3 / WPF | Windows only. Not a path to macOS. | [opened] |
| SmartScreen | Unsigned installer shows "Windows protected your PC" with a "Run anyway" path. **Smart App Control hard-blocks unsigned files with no bypass.** Reputation resets per version if unsigned. | learn.microsoft.com smartscreen-reputation, west-wind blog 1 Jun 2026 [opened] |
| Signing routes | Azure Trusted Signing $9.99/month, individuals in **US, Canada, EU, UK** only (per Microsoft blog). **SignPath Foundation signs open-source Windows apps for free** after maintainer identity check. OV and EV certs now build reputation equally (since Mar 2024). | techcommunity.microsoft.com, signpath.org, todesktop blog [opened] |
| winget | `.exe` and `.msi` installers do not need signing to be listed; MSIX does. | github.com/microsoft/winget-pkgs [opened] |
| Signing the file itself | **minisign**: Ed25519, 64-byte signature, trusted comment can carry metadata, Rust crates `minisign` and `minisign-verify` (zero deps). **OpenSSH `ssh-keygen -Y sign/verify`** with an `allowed_signers` file, Rust crate `ssh-key`. **Sigstore keyless**: binds a signature to a GitHub, Google or Microsoft login via a public transparency log, no key to manage. MCPB signs with X.509 PKCS#7 appended after the zip; whether Claude Desktop verifies it is not stated. No skills ecosystem signs packages. | jedisct1.github.io/minisign, man.openbsd.org/ssh-keygen, docs.sigstore.dev, mcpb CLI.md [opened] |
| Phones | Google Play (clarified 30 Oct 2025): "Any use of the Accessibility API that enables an app to autonomously initiate, plan, and execute actions or decisions is strictly prohibited." iOS: "All third-party apps are sandboxed... prevent apps from gathering or modifying information stored by other apps." Phones are out. | support.google.com 10964491 and 16550159; support.apple.com sec15bfe098e [opened] |

---

## 3. What is actually changing (last 12-18 months)

**Structural shifts**
- **Skills became the standard unit.** One spec, 40+ agents, a CLI that installs into all of them, and VS Code deprecating its own prompt files in favour of it. The "install this dev tool into your agent" use case is being absorbed by skills and plugins. A runfile that competes with that loses. A runfile that is *compatible* with it (a valid SKILL.md underneath) rides it.
- **Agent makers now ship send-a-task primitives**, but thin ones: `claude-cli://` links (prompt only, inert until Enter), `.mcpb` (servers), Cowork Dispatch (your own phone to your own desktop). None adds a guard or a rating.
- **Marketplaces got attacked within weeks of existing** (ClawHub, Jan-Feb 2026). Scanners appeared within days. Trust-by-source plus scanning is the vendors' answer; signing is not yet anyone's answer.
- **Per-action enforcement is now a first-class feature inside Claude Code** (permission rules, hooks, permission-prompt tool, per-run settings). Two years ago this would have required patching the agent.
- **Windows is getting first-party sandbox primitives** (OpenAI's sandbox users approach, Microsoft's experimental API). Still early.

**Hype to discount**
- "Agentic OS" on Windows: preview since Oct 2025, no general release, no developer platform yet.
- Long desktop workflows: 20.6% success is the real number. Short tasks only, for a long time.

---

## 4. Contested or uncertain

- **Is the paste-and-go problem big?** Only two vendor pages confirmed, no visible HN debate, and the ecosystem is moving to `npx skills add`. The *problem* the brief leads with is real but small and shrinking. The *mechanism* (a guarded, sendable task) is unoccupied.
- **May a third-party app drive the receiver's own Claude Code?** Not confirmed either way from a terms page. The deep-link feature suggests Anthropic expects external programs to hand tasks to Claude Code, but `-p` mode under another app's control is a different shape. Needs checking.
- **Codex on Windows:** install docs say WSL2 only; the Codex app announcement says native Windows with a sandbox. Both from OpenAI, four months apart.
- **Does a confirm screen protect anyone?** Cursor's DeepJack says a badly built one does not. Gemini CLI and MCPB ship them anyway. Our design must assume the file is hostile to the screen, not just to the agent.

---

## 5. Relevance to this project

Filtered through the answers so far (developers first, Windows first, full app, open source, honest guard, non-technical people later):

1. **Phase 1 is feasible with high confidence (about 85%).** Every piece needed to run a guarded task through the receiver's own Claude Code exists today in documented flags: per-run permission rules where deny wins, a hook for logging, a permission-prompt tool for pausing, a live event stream, a structured result, resume, and budget caps. The remaining risk is the terms question above.
2. **Phase 2 is not feasible today (about 90% confidence).** No consumer desktop agent on Windows can be started with a task by an outside program. Design the adapter boundary so it slots in, and say so publicly rather than promising it.
3. **The format should be a superset of SKILL.md.** Then the no-app path is "drop it in your skills folder" on 40+ agents, which is better than copy-paste, and the install-a-dev-tool crowd is not alienated.
4. **The demo task must not be "install a dev tool".** Plugins and skills already own that. The unoccupied ground is *one-off tasks sent between people*: "set up and test this repo for me", "organise this folder", "migrate these files". The tamper clip (reading a private key, blocked) works on any of these.
5. **Tauri 2** is the app shell: per-user file association and URL scheme without admin, streaming child output, small installer, same code on macOS later. Electron's file association needs an admin install, which is a real problem for a "double-click a file a friend sent you" product.
6. **Expect SmartScreen.** Apply to SignPath Foundation early; Azure Trusted Signing is not open to individuals in India per Microsoft's eligibility list.

---

## 6. What could not be established

- The "1,103 of 14,706" ClawHub figure (not found; SkillSieve paper not opened).
- Koi Security's original ClawHavoc post (now redirects to Palo Alto Networks).
- Whether a third-party app may drive `claude -p` on a receiver's subscription (terms page not found).
- Whether `--bare` drops hooks supplied via `--settings`.
- Whether Claude Desktop verifies `.mcpb` signatures at install.
- Codex: native Windows vs WSL2-only, and whether its hooks exist in official docs.
- Official OSWorld-Verified leaderboard numbers (page loads by script); human success rate on OSWorld 2.0.
- General-release status and editions for Windows Copilot Actions in 2026 (all primary pages are 2025 previews).
- Whether `npx skills add` previews before writing.
- Reddit threads on paste-and-go risk (not searched; web tool limit hit near the end of two investigations).
