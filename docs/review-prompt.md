You are a senior product and security reviewer. I am a student building an open-source project and I want your honest, specific feedback on the plan below before I write any code. Do not flatter. Do not summarise the plan back to me. Attack it, then tell me what to change.

How to answer:
1. First list the three decisions you think are most likely to be wrong, in order, each with one concrete failure scenario (what happens, to whom, on which day).
2. Then go section by section (A to H) and give feedback only where you disagree, see a gap, or know a fact I am missing. Skip sections you agree with; say "agree" in one word.
3. Separate "I know this is wrong because X" from "I suspect this but am not sure". Label guesses as guesses.
4. If you know a product, paper, standard or incident I have not listed, name it with a date. Do not invent names.
5. End with: the single change that would most improve the odds, and the single thing you would cut.
6. Plain language. I am a student. Explain any technical term in one short line the first time you use it.

======================================================================
A. WHAT THE PRODUCT IS
======================================================================

Working name: Steplet. A "steplet" is one Markdown text file with a YAML header that carries a task for an AI agent instead of for a person: who sent it, the goal, the steps in plain words, what the agent may touch (folders, shell command patterns, website domains, logins needed, admin rights yes/no), what it must never do, how to undo it, and an expiry date.

The sender writes it once and sends it like a PDF (WhatsApp, email, download, or a link). The receiver double-clicks it on a Windows laptop. Our desktop app opens it and shows: who signed it, the whole plan with nothing cut off, a green/amber/red risk colour decided by code rules, what it will touch, and a choice of agents installed on the machine (or our built-in agent). The receiver approves. The agent then does the work on the receiver's own machine, and our app checks every action against the file's allowed list. Anything undeclared pauses for a yes/no. Red files cannot be run at all.

Goal for version 1: strangers run files we did not write for them. Revenue is not the goal yet. Later: a subscription for people who use our hosted model relay.

Who sends: anyone (friend, team lead, brand, dev tool). Who receives: first developers who have Claude Code or Codex installed; then non-technical people via our built-in agent.

Six launch use cases, chosen so each proves one different thing:
1. Developer to developer: set up this repository and run its tests, report. (green)
2. Child to parent: sort the files in this folder into subfolders by type, list what moved. (amber, has undo)
3. Brand to customer: connect our tool to your project. (amber, pauses for the receiver to type a key personally)
4. Friend to friend: open this site, log in yourself, download your file to Downloads. (amber, browser control, login pause)
5. Helper to colleague: back up these folders to an external drive and verify. (green, asks which drive first, resumes after failure)
6. Attacker to anyone: looks like 1 but also reaches an undeclared website and reads a private key. (red: rating catches it, guard blocks it; this is the pitch clip)

======================================================================
B. WHAT I FOUND WHEN I RESEARCHED (6 Oct 2026, about 100 searches, 70 primary pages)
======================================================================

Nothing found that combines (a) a task file sent to a person, (b) a preview with a risk rating, (c) a choice of locally installed agents, (d) per-action enforcement. The pieces exist separately:
- Agent Skills open standard (Anthropic, Dec 2025): SKILL.md folders read by 40+ agents (Claude Code, Codex, Copilot, Cursor, Gemini CLI, ...). Its allowed-tools field is pre-approval, not a restriction. Vercel's "npx skills add" installs into 17 agents. VS Code deprecated its own prompt files in favour of skills. So "install this dev tool into your agent" is being absorbed by skills; we must not compete there.
- Claude Code deep links (claude-cli://open?q=...) pre-fill a prompt in a terminal with a warning banner; no guard, no file, no rating. Closest thing from an agent maker.
- Claude Desktop .mcpb bundles: double-click opens an install dialog; installs a server, not a task.
- Cursor MCP install links: a confirm dialog that was defeated ("DeepJack", reported Apr 2026, still working Jul 2026) by padding text so the dangerous part scrolled off a single-line box.
- Munder Difflin ".hire.json" (Jun 2026): a sendable agent ROLE file with an import preview and a human spawn gate. No task, no per-action guard.
- Zapier Agents template links: preview then copy, cloud only.
- Security: ClawHub (OpenClaw skills marketplace) had 341 of 2,857 skills malicious (Koi Security, Feb 2026), mostly real malware behind fake "prerequisite" download steps and curl|sh. Snyk ToxicSkills (Feb 2026): 76 confirmed malicious of 3,984, 13.4% with a critical issue. Pillar "Rules File Backdoor" (Mar 2025): invisible Unicode in instruction files. SafeDep (May 2026): npm packages shipping a project .claude/settings.json that registers a hook, so malware runs when the project opens. OWASP LLM06: "Implement authorization in downstream systems rather than relying on an LLM to decide if an action is allowed."
- Benchmarks: best agents finish about 85% of short desktop tasks (OSWorld-Verified) and 20.6% of long ones (OSWorld 2.0, Jul 2026, 318 tool calls per task on average). Short tasks only.
- Non-technical receivers: NO consumer desktop agent on Windows can be started with a task by an outside program today. Claude Cowork has no API; Claude Code's screen control is interactive-only and Windows-only inside the Desktop app; Windows Copilot Actions is still an Insider preview. So we must ship our own agent.
- Subscriptions: Anthropic forbids third-party apps from using claude.ai logins. Google says consumer plans do not cover API use. OpenAI, since 29 Sep 2026, lets open-source local apps use a ChatGPT Plus/Pro plan ("Sign in with ChatGPT") only while the app does not charge; paid apps need approval.
- Demand signal is thin: only two vendor pages confirmed saying "paste this prompt into your coding agent" (Arize, NVIDIA NemoClaw); 736 "llms-install.md" files on GitHub; no high-scoring Hacker News thread on the risk.
- Phones are out: Google Play bans autonomous use of the Accessibility API (clarified Oct 2025); iOS sandboxes all apps.

======================================================================
C. ARCHITECTURE DECISIONS (all made; tell me which are wrong)
======================================================================

Platform: Windows first, macOS later, every OS-specific piece in one module. Open source, permissive licence.

App shell: Tauri 2 (Rust backend + web front end). Reason: per-user file association and steplet:// link registration without admin, streams child-process output, 3 to 15 MB installer. Electron's file association requires an admin install.

File format: one Markdown file with a YAML header that is a VALID Agent Skills SKILL.md underneath, so the no-app fallback is "drop it in your skills folder" on 40+ agents. The official skill validator allows only six top-level header fields and string-only metadata, so ALL our fields (sender, allowed list, never list, undo, expiry, signature) sit under metadata: as strings, lists written as block strings. Command patterns in the allowed list are plain words plus an optional trailing *, nothing richer, never compiled as a regex (the sender may be hostile). Size limits: file 256 KiB, header 32 KiB, at most 25 commands, 15 domains, 10 folders per list, 30 steps.

Signing: OpenSSH signature format (SSHSIG), Ed25519 only, embedded in the header as a block string, namespace steplet-v1. The signed bytes are the whole file after a fixed clean-up (strict UTF-8, strip one BOM, CRLF to LF, strip trailing spaces, one final newline, no Unicode normalisation) minus the signature lines. Tested: browser-made signatures verify with Windows' built-in ssh-keygen and the reverse; edits to body, expiry or header order break it; Windows line endings and email artefacts do not. The header is parsed ONCE from the signed bytes; duplicate keys, anchors, aliases, tags, multiple documents are rejected. Identity in v1: trust on first use (app remembers each sender's key fingerprint, warns if it changes). Verified tier later: the signing key must appear on the sender's GitHub account (checked by numeric account id). Expiry required on signed files, default 14 days. Rejected: Sigstore (needs network to sign, publishes email to a permanent public log, JS library not browser-capable), minisign (no key directory anyone uses), PGP (its text format corrupts the opening --- line).

Risk rating: colour decided by code rules ONLY. An AI-written summary may be shown, labelled, below everything, never changes the colour, hidden for red. Colour = worst single finding. Unsigned does NOT cap at amber (anyone can sign with a fresh key for free; capping would teach people to ignore amber); identity is a separate line. Only "changed after signing" and "same sender, different key" force red. About 70 rules in groups: strict parsing, structure, consistency (a step mentions something the allowed list does not cover = amber, because "green" must mean the agent will not need to ask you mid-run), dangerous capability (admin rights = red; writes to home root, system folders, shell profiles, .claude/, .cursor/, .git/hooks = red; download piped into a shell = red EVEN from official sites; package installs and script runners like npm install = amber because the guard cannot see inside them; registry redirection = red), hiding tricks (any default-ignorable or bidi Unicode = red; look-alike domains; long base64; padding; HTML comments; link text vs real address), social tricks (asks for seed phrases = red; raw IP links = red; "don't tell the user" phrases = red; fake "prerequisite" near a download = red). An always-blocked list no file can unlock: .ssh, .aws, .kube, .gnupg, .npmrc, .netrc, browser credential stores, password managers, wallets, agent credential files. Red files cannot run, no override. Preview screen: nothing ever cut off, counts first, invisible characters drawn as visible badges, no network fetches during preview, text from the file never appears in the app's own window frame, Run button after the full list. One matcher function used by the preview, the guard, and the ask screen; built as a Rust library with a WebAssembly build for a public check page.

Guard tiers shown to the receiver before the run: A full guard (every action checked, undeclared actions pause for yes/no, live log); B rules only (enforced, cannot pause); C watched (we can stop it, the bad action may already have happened); D preview only (file copied into the agent's skills folder). Today: Claude Code A (headless -p, per-run --settings with permission rules where deny beats allow, PreToolUse hook, --permission-prompt-tool for the pause, stream-json events, --json-schema result, --resume, --max-budget-usd, --bare to skip the receiver's own hooks); Codex CLI A (runs natively on Windows with its own sandbox, app-server approval callbacks, PreToolUse hook, prefix rules, network domain rules; hosted web search turned off because it cannot be hooked); Copilot CLI B; Gemini CLI B; Cursor CLI C; everything else D. Claude Code settings generated from a file contain DENY rules only, never allow rules, because Claude Code strips wrappers before matching allow rules. One rule checker in the core; one small translator per agent; one gate helper program that hook-based agents run before each action, which asks the app over a local pipe and waits (that wait is the pause). If the event stream ever shows an action that skipped the gate, stop the run and mark the adapter broken.

What the guard honestly cannot enforce (shown on screen): programs started by an allowed command (npm install runs other people's scripts), data leaving through allowed paths, write-then-run (agent writes a script into an allowed folder then runs an allowed command that executes it; rated amber/high), plain-language "never" items.

Built-in agent (version 1): the agent loop runs in Rust inside the app, on the receiver's machine, using the genai crate behind our own provider interface (first two days: a test of streaming tool calls on five providers; if two or more fail, switch to a Node helper running the Vercel AI SDK). Tools: read/write files inside allowed folders (with a copy before overwrite for real undo), run commands matching allowed patterns (started directly, not via a shell; chaining and redirect symbols refused; our secrets stripped from the child's environment; Job Object for kill-all and limits), fetch allowed domains (https only, private/local addresses refused, redirects re-checked), request_user_action for pauses. Windows path hardening: resolve .., links and junctions, 8.3 short names, \\?\ and UNC paths, alternate data streams, device names; re-check the real path after opening. Logins: the receiver types secrets into OUR window; the model sees a placeholder swapped in at tool-run time.

Model sources, in the order offered: (1) our hosted relay (server holds provider keys, passes calls through with per-run budget, output-size cap, per-minute limit, cheap-model allowlist, usage logging without prompts, kill switch), free with daily caps behind Google or GitHub sign-in at first, subscription later; (2) Sign in with ChatGPT at zero cost to us while we do not charge; (3) pasted API key as a developer add-on; (4) local models via Ollama. Normal people never see a key field. We pay the provider bills at first.

Browser control (version 1, built last): the receiver's own Edge/Chrome with a separate app-only profile (Chrome ignores remote control on the default profile); login happens with automation detached so the agent cannot see the password, then the receiver clicks Continue in OUR window; allowed sites enforced by our own local proxy, not by the browser tool (Playwright MCP's own allowed-origins option says it is not a security boundary); downloads forced into an allowed folder; password boxes blanked in screenshots. Likely controller: Playwright MCP as a helper attached to the browser we launch.

Server features, in build order: link page (static page showing the same preview, hands the file to the app via steplet://), then reports to the sender, then kill switch (a file reported harmful is switched off; app checks before every run), then verified senders.

Distribution: expect the SmartScreen warning at first; apply to SignPath Foundation (free code signing for open-source Windows apps); Azure Trusted Signing is not open to individuals in India; winget listing.

Build order inside version 1: (1) file format, preview, rating, signing, Claude Code adapter, public check page; (2) built-in agent (files, terminal, fetch), hosted relay with sign-in, Codex adapter; (3) browser control, link page; (4) reports, kill switch, verified senders.

======================================================================
D. THINGS I KNOW ARE UNVERIFIED
======================================================================

- Whether a third-party app may drive a receiver's own Claude Code subscription in headless mode (no terms page found either way).
- Whether Claude Code's --bare also drops hooks passed via --settings.
- Whether Codex hooks can be injected for one run without writing to the receiver's config.
- Whether WhatsApp, Gmail or Outlook re-wrap or alter pasted steplet text so a genuine file fails its signature.
- Whether the Rust signing and YAML crates compile cleanly to WebAssembly.
- OpenAI's exact per-app limits for Sign in with ChatGPT (help page returned 403).
- All effort estimates (built-in agent 6 to 9 weeks, browser control 6 to 10 more) are one reviewer's judgment.
- Nothing has been built or run except the signing prototype.

======================================================================
E. CONSTRAINTS ABOUT ME
======================================================================

Solo student builder on Windows. Can learn Rust but has not shipped Rust. No budget beyond small hosting and early model bills. No Mac. Quality, simplicity, robustness and sound engineering are weighted far above development time.

======================================================================
F. QUESTIONS I MOST WANT YOUR VIEW ON
======================================================================

1. Is the hosted relay (we pay for strangers' model use, behind Google/GitHub sign-in) a reasonable version-1 feature for a solo student, or is it the decision most likely to sink the project? What caps would you set on day one?
2. Browser control in version 1: keep, or cut to version 2? What is the most likely way it fails on a real non-technical person's laptop?
3. The guard tiers: is showing "rules only" and "watched" agents honest enough, or does offering any non-A tier undermine the whole trust story?
4. "Red cannot run, no override": right call, or will it block honest files so often that people stop using the app?
5. Is "a valid SKILL.md underneath, all our fields as strings under metadata" worth the ugliness, or should the format be our own and the skills compatibility be an export?
6. Signing with SSHSIG and GitHub keys: what attack or usability problem am I missing? Is trust-on-first-use defensible for a product whose pitch is trust?
7. The six use cases: which one would you replace, and with what?
8. Name "Steplet" (steplet.dev/.ai/.io free; one Swiss national trademark in software classes registered July 2026 by an individual; a hackathon project of that name): fine, or a problem?
9. What is the worst thing a GREEN-rated file could still do on a receiver's machine under this design?
10. What would make an agent maker (Anthropic, OpenAI) ship this natively, and what part of this design still matters if they do?

======================================================================
G. WHAT I DO NOT WANT
======================================================================

No restating the plan. No generic advice ("talk to users", "start small") unless tied to a specific decision above. No praise before criticism. No invented products or statistics; if unsure, say unsure.

======================================================================
H. OUTPUT FORMAT
======================================================================

1. Top three likely-wrong decisions, each with a concrete failure scenario.
2. Section-by-section feedback (A to F), "agree" where you agree.
3. Answers to the ten questions in F, numbered, short.
4. Facts or prior art I am missing, with dates, labelled verified or guess.
5. One change that most improves the odds. One thing to cut.
