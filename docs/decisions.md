# Decisions log

Started 6 Oct 2026. One line per decision, with the reason. Pending items are marked. Nothing here is code yet.

Plain words throughout. A term is explained the first time it appears.

## Product

| # | Decision | Why |
|---|---|---|
| 1 | **Goal for version 1:** strangers run files we did not write for them. | Chosen over "a demo" because one stranger's run teaches more than any clip. |
| 2 | **First receivers:** developers with Claude Code. Non-technical people are the second phase and the bigger market. | Developer path is fully supported by documented flags today. No consumer desktop agent on Windows can yet be started with a task by an outside program. |
| 3 | **Windows first.** Every system-specific piece (file type registration, agent launch, path handling) lives in one small place so macOS is a port. | Can only verify on Windows. |
| 4 | **Open source**, permissive licence. | A trust product must be readable. Also needed for "Sign in with ChatGPT" open-source eligibility. |
| 5 | **Guard promise:** "the agent itself will not act outside the allowed list". Not "nothing bad can happen". A later strict mode may add a system sandbox. | Hooks and our own agent see the agent's actions, never what launched programs do. No sandbox on native Windows. |
| 6 | **Server:** full set eventually, built in this order: link page, then reports, then kill switch, then verified senders. | Link page is static hosting. The rest needs an API. Identity before users is wasted work. |
| 7 | **Six launch use cases, no overlap** (see table below). Approved 6 Oct 2026. | Each proves one thing the others do not. |
| 8 | **Name: Steplet.** Extension `.steplet`, link scheme `steplet://`. Chosen 6 Oct 2026 after collision checks on ten names. Known collisions: one Swiss national trademark (classes 9, 41, 42, registered 7 July 2026, owner an individual), a hackathon project with no site or code, a dormant 2000s Java page, and the GitHub username held by a private person. Free: steplet.dev, steplet.ai, steplet.io, npm and crates.io names. Rejected: Tasklet (funded AI-agent company plus live US/EU/UK trademarks), Runlet, Botlet, Agentlet, Asklet, Joblet (all live same-space products or software-class trademarks), Tasket (all domains taken, pending US software mark), Chorelet and Questlet (clean, not chosen). | "Send me the steplet" rolls off the tongue; no trademark in a major market. |
| 8b | **Free hosted runs need sign-in** with Google (everyone) or GitHub (developers). Daily cap per account, small budget per run, cheap models only. | No passwords of ours; throwaway identities are harder than with email links; GitHub sign-in later feeds the verified-sender tier. |
| 8c | **Build order inside version 1:** (1) file format, preview, rating, signing, Claude Code adapter, public check page; (2) built-in agent with files, terminal and fetch, hosted relay with sign-in, Codex adapter; (3) browser control, link page; (4) reports, kill switch, verified senders. Approved 6 Oct 2026. | Each milestone is usable on its own; the first stranger-usable release comes earliest. |

### The six use cases (pending approval)

| # | Who to whom | Task | Colour | Proves |
|---|---|---|---|---|
| 1 | Developer to developer | Set up this repository and run its tests, report | Green | Core developer path |
| 2 | Child to parent | Sort files in a folder into subfolders by type, list what moved | Amber | Non-technical receiver, undo |
| 3 | Brand to customer | Connect our tool to your project | Amber | Pause for the receiver to type a key personally |
| 4 | Friend to friend | Open this site, log in yourself, download your file to Downloads | Amber | Browser control, login pause (version 2) |
| 5 | Helper to colleague | Back up these folders to an external drive and verify | Green | Asks-first, resume after failure |
| 6 | Attacker to anyone | Looks like 1, also reaches an undeclared site and reads a private key | Red | Rating catches it, guard blocks it |

## File format

| # | Decision | Why |
|---|---|---|
| 9 | **One Markdown file with a YAML header**, valid as an Agent Skills `SKILL.md` underneath. | 40+ agents read that layout, so the no-app path is "drop it in your skills folder". |
| 10 | **Header layout:** all runfile fields under `metadata:` as strings; lists written as block strings. | The official validator allows only six top-level fields and string-only metadata. The maker and the preview hide the ugliness; the "40+ agents" claim stays honest. |
| 11 | **Command patterns in the allowed list:** plain words plus an optional trailing `*`. Nothing richer. Never compiled as a regex. | Patterns come from the sender, who may be hostile. Rust's regex library warns about untrusted patterns. Agents' own rule languages differ, and this subset maps to all of them. |
| 12 | **Size limits:** file 256 KiB, header 32 KiB, at most 25 commands, 15 domains, 10 folders per list, 30 steps, any command under 300 characters. Over the limit is red. | Everything must fit on one screen with nothing cut off (the Cursor "DeepJack" lesson). Starting values, to tune on a test corpus. |

## Signing and identity

| # | Decision | Why |
|---|---|---|
| 13 | **Sign with the OpenSSH signature format ("SSHSIG"), Ed25519 keys only, signature embedded in the header as a block string.** Namespace `runfile-v1`. | Proven both directions on this machine: browser-made signatures verify with Windows' own `ssh-keygen`, and the reverse. Every Windows 11 machine has OpenSSH. GitHub publishes keys in this exact format. |
| 14 | **Signed payload:** the whole file after a fixed clean-up (strict UTF-8, strip one byte-order mark, CRLF to LF, strip trailing spaces, one final newline, no Unicode normalisation), minus the signature lines. Everything else, including expiry and body, is signed. | Edits to body, expiry, or header order all break the signature. Windows line endings and email artefacts do not. |
| 15 | **Parse the header once from the signed bytes.** Reject duplicate keys, anchors, aliases, tags, multiple documents. The body handed to the agent is the canonical signed body. | Two parsers disagreeing on a duplicate `expires:` is the real attack. |
| 16 | **Version 1 identity:** trust on first use. The app remembers each sender's key fingerprint and warns if a sender's key changes. | No server, no accounts. |
| 17 | **Verified tier:** the signing key must appear on the sender's GitHub account (checked by numeric account id, not name, since names can be reclaimed). Cached 24 hours, works offline after. | Developers already have GitHub and SSH keys. No password system of ours. |
| 18 | **Rejected:** Sigstore (needs network and login to sign, publishes sender email to a permanent public log, JS library not browser-capable), minisign (no key directory anyone uses), PGP (its text format corrupts the opening `---`), JWS. | |
| 19 | **Expiry required on signed files.** Default 14 days, cap 365. | Limits damage from a stolen key. |

## Risk rating

| # | Decision | Why |
|---|---|---|
| 20 | **Colour decided by code rules only. An AI summary may be shown, labelled, below everything, and never changes the colour.** Hidden for red files. | A file can be written to trick an AI reviewer. OWASP: enforce in code, not by asking the model. |
| 21 | **Colour = worst single finding.** Green: nothing above a note. Amber: continue only if you trust the sender. Red: cannot be run from the app, no override in version 1. | Scores that add up are gameable and hard to explain. |
| 22 | **Unsigned does not cap at amber.** Identity is a separate line beside the colour. Only "changed after signing" and "same sender, different key" force red. | Anyone can sign with a fresh key for free. Capping would teach people to ignore amber. |
| 23 | **Rule groups:** reading the file (strict), structure, consistency (steps vs allowed list), dangerous capability, hiding tricks, social tricks. About 70 rules, each with a fixed id, a severity, and a plain-words reason. Copied patterns credited to Cisco skill-scanner and Snyk agent-scan (Apache-2.0). | Full table in the architect memo `.claude/agent-memory/architect/project_risk_rating_decision.md`. |
| 24 | **Always-blocked list** no file can unlock: `.ssh`, `.aws`, `.kube`, `.gnupg`, `.npmrc`, `.netrc`, `.git-credentials`, browser credential stores, password managers, wallets, agent credential files. Asking for them is red. | |
| 25 | **Red even from official sites:** download piped into a shell. | ClawHub malware hid behind exactly this. We check one file with a person watching, so we can be stricter than mass scanners. |
| 26 | **Amber:** package installs and script runners (`npm install`, `pip`, `npm run`, `make`). | The guard cannot see inside them. Most developer files will be amber. To be tuned on the corpus. |
| 27 | **"Green" means the agent will not need to ask the receiver for anything risky mid-run.** Steps that mention something the allowed list does not cover are amber. | Asking under time pressure after showing green is a trap. |
| 28 | **Preview screen rules:** nothing ever cut off, counts first, one item per row, invisible characters drawn as visible badges, links show their real address, no network fetches during preview, text from the file never appears in the app's own frame, the Run button sits after the full list. | Against DeepJack-style tricks. |
| 29 | **One matcher** (`Policy::check`) used by the preview, the guard, and the ask screen. Never two implementations. Built as a Rust library with a WebAssembly build for the public check page. | Preview and enforcement must never disagree. |
| 30 | **Re-checked at run time:** the exact command, the resolved real path (links, junctions, short names, streams, UNC), expiry, the file's hash against the one approved, and the receiver's own agent setup (project hooks, mods, bypass modes). | |
| 31 | **Claude Code settings generated from a file contain deny rules only, never allow rules.** The gate hook decides allow. | Claude Code strips wrappers before matching allow rules, so a generated allow rule could pass something our matcher would ask about. |

## App and agents

| # | Decision | Why |
|---|---|---|
| 32 | **Tauri 2** (Rust backend, web front end). | Per-user file association and `runfile://` link registration without admin; streams child output; 3 to 15 MB installer; same code on macOS. Electron's file association requires an admin install. |
| 33 | **Built-in agent in version 1:** files, terminal commands, web fetch, **and browser control**. Confirmed 6 Oct 2026. The agent itself always runs on the receiver's machine; only the model calls can be hosted. | Strongest guard, since we own every action. No agent to install. Browser control is the highest-risk piece (architect estimate: six to ten extra weeks, a judgment not a measurement) and the user chose to include it. |
| 34 | **Model sources, in the order the app offers them:** (1) our hosted relay, where we pay the providers and cap each run, free at first with caps, subscription later; (2) Sign in with ChatGPT for Plus or Pro users at no cost to us (allowed for open-source local apps since 29 Sep 2026 only while we do not charge; charging needs OpenAI's approval); (3) pasted API key, shown as an add-on for developers; (4) local models via Ollama. Normal people never see a key field. Google and Anthropic consumer plans cannot be used by third-party apps. | User decision: no key pasting for normal people; we bear early costs, then a subscription. |
| 35 | **Agent loop in Rust** using the `genai` crate behind our own provider interface. First two days: a test of streaming tool calls on five providers. If two or more fail, switch to a Node helper running the Vercel AI SDK. | One trusted process, keys never leave Rust, one program to sign. The library is mostly one maintainer, hence the interface and the test. |
| 36 | **Guard tiers shown to the receiver before the run:** A full guard (every action checked, undeclared actions pause for yes/no, live log), B rules only (enforced, cannot pause), C watched (we can stop it but the bad action may have happened), D preview only (file copied into the agent's skills folder). | Each agent exposes different switches. Honest labels instead of pretending. |
| 37 | **Tier assignments today:** Claude Code A, built-in agent A, Codex CLI A (hosted web search turned off since it cannot be hooked), Copilot CLI B, Gemini CLI B, Cursor CLI C, everything else D until researched. Codex runs natively on Windows; its install page saying "WSL2 only" is stale. | From official docs; not yet run on a real machine. |
| 38 | **Adapters:** one rule checker in the core; one small translator file per agent; one gate helper program that hook-based agents run before each action, which asks the app over a local pipe and waits (that wait is the pause). If the event stream ever shows an action that skipped the gate, stop the run and mark the adapter broken. | |
| 39 | **Version 1 adapters:** Claude Code and Codex. Copilot and Gemini in 1.1. All others as tier D. | Both A-tier, both native on Windows. |
| 40 | **Run the external agent with the receiver's own hooks, skills, plugins and project settings switched off** where the agent allows it (Claude `--bare`, Codex `--ignore-user-config --ignore-rules`), dropping a tier if it cannot. | May 2026 npm attack shipped malware as a project settings file registering a hook. |
| 41 | **Logins:** the receiver types secrets into our window only. The model sees a placeholder and the real value is swapped in at the moment the tool runs. | |
| 42 | **Undo:** copy each file before overwriting it. Undo covers file writes only, not command side effects, and the screen says so. | |
| 43 | **Browser control (version 1, after the file and terminal path works):** the receiver's own Edge or Chrome with a separate app-only profile; login happens with automation detached so the agent cannot see the password; allowed sites enforced by our own local relay, not by the browser tool; downloads forced into an allowed folder. | Chrome ignores remote control on the default profile. The browser tool's own site filter says it is not a security boundary. |
| 44 | **Hosted model relay in version 1:** our server holds the provider keys and passes model calls through with a per-run budget, output-size cap, per-minute limit, turn and time limits, a model allowlist of cheap models, usage logging (token counts and cost, not prompts), and a kill switch. Free tier with daily caps at first; subscription later. Who may use the free tier is pending (Q24). | User decision. This replaces the earlier "sender-sponsored OpenRouter key" idea. Costs: hosting, provider bills, abuse handling, and later payments, all before the first stranger runs a file. The user chose this knowingly. |
| 44b | **Red files cannot run, no override.** | Red means a known attack trick or power the app cannot limit. An override would only need one persuaded receiver. |

## Distribution

| # | Decision | Why |
|---|---|---|
| 45 | **Expect the SmartScreen warning at first.** Apply to SignPath Foundation (free signing for open-source Windows apps). Azure Trusted Signing is not open to individuals in India. | Smart App Control hard-blocks unsigned installers with no bypass. |
| 46 | **winget listing** once there is an installer; `.exe` needs no signing to be listed. | |

## Open items to verify before launch

- Whether a third-party app may drive the receiver's own Claude Code subscription in headless mode (no terms page found either way).
- Whether `--bare` also drops hooks passed via `--settings`.
- Whether Codex hooks can be supplied for one run without writing to the receiver's config, and whether `--ignore-user-config` skips repo hooks.
- Copilot CLI: `--allow-url`/`--deny-url`, SDK permission handler.
- Whether chat apps and email re-wrap or alter pasted runfile text (test with WhatsApp, Gmail, Outlook).
- Whether the Rust signing and YAML crates compile to WebAssembly.
- OpenAI's help-centre page on using a ChatGPT plan in other apps (returned 403); whether free accounts are excluded.
- The name and extension, once chosen: collisions with existing file types and link schemes, domain availability.

## Review round 1 (7 Oct 2026): two outside AI reviews, claims verified against official docs

Rows above that these supersede are noted in brackets.

### Verified facts that changed decisions

| # | Fact | Source |
|---|---|---|
| V1 | In `--bare` mode "Claude Code never reads OAuth credentials or the system keychain"; it needs an API key. So `--bare` cannot be used with a receiver's subscription. | code.claude.com/docs/en/headless.md |
| V2 | In headless mode, hooks in a project's settings only run after that folder's trust prompt was accepted, and headless mode shows no trust prompt. Deny rules apply regardless. Hooks merge across settings levels rather than replace. | hooks.md, settings-reference.md |
| V3 | "A timed-out command, http, or mcp_tool hook doesn't block the tool call" and "use the permission system rather than a hook to enforce a hard allow or deny." | hooks.md |
| V4 | Claude Code has no containment of any kind on native Windows. Only WSL2, a container or a VM contains child programs. | sandboxing.md |
| V5 | Codex: "Treat tool hooks as a useful guardrail, not a complete enforcement boundary." `write_stdin` does not re-run PreToolUse; some tool paths opt out of hooks. | learn.chatgpt.com/docs/hooks |
| V6 | Codex's native Windows sandbox, both modes: "prevent filesystem writes beyond your working folder, network access without explicit approval, and child processes from operating outside sandbox boundaries." Elevated mode uses dedicated low-privilege users and firewall rules; unelevated uses restricted tokens and ACLs. | learn.chatgpt.com/docs/windows/windows-sandbox |
| V7 | Electron itself does not need an admin install for file associations; the limit is electron-builder's NSIS setting. Tauri stays, for size and the Rust core, but row 32's justification was too broad. | reviewer; consistent with Windows per-user registration verified earlier |
| V8 | Prior art confirmed: Saltzer and Schroeder 1975 "complete mediation"; DSSE signing envelope "avoids canonicalization to reduce attack surface"; arXiv 2510.26328 (30 Oct 2025) on skills as trivial prompt-injection carriers; "TrustFall" one-click RCE via MCP installs (7 May 2026). | github.com/secure-systems-lab/dsse, arxiv.org, adversa.ai |
| V9 | Sign in with ChatGPT was announced 29 Sep 2026 (DevDay); the article page itself shows no date. | engadget, simonwillison.net |
| V10 | Not verified either way: whether Codex hooks can be injected for one run via `-c`, and whether `--ignore-user-config` skips a repository's `.codex/` files. Test on a machine. | learn.chatgpt.com/docs/non-interactive-mode |

### Decisions changed

| # | Change | Why |
|---|---|---|
| R1 | **The security promise is two separate promises, shown separately:** (a) the agent's proposed actions are checked against the list; (b) the operating system restricts what the resulting programs can touch. "Every action checked" and "cannot see inside programs" were contradictory labels. [replaces rows 5, 36] | Both reviews; V4, V5, V6. |
| R2 | **Two execution modes.** *Contained*: child programs are OS-restricted (Codex's native Windows sandbox; Claude Code inside WSL2; later Microsoft's sandbox API). *Native*: no containment. **A file that allows any install, build, test, script runner or arbitrary command can only run in contained mode.** On a machine with no contained runner, such a file is shown as "cannot run safely on this computer", not amber. [replaces rows 26, 37] | An amber warning does not repair missing containment. |
| R3 | **Built-in agent, version 1: file tools and web fetch run natively; shell commands only through a contained runner (WSL2) or not at all.** For folder sorting and backup, the agent proposes the mapping and the app's own deterministic code moves and verifies files. [amends rows 33, 35] | Review: deterministic code already does the core operation; the agent adds uncertainty without benefit there. |
| R4 | **Claude Code gate:** no `--bare`. Deny rules for the never list and always-blocked paths; **the permission-prompt tool is the decision and pause point** for everything else; the PreToolUse hook is for logging only. Project-folder hooks are inert in headless mode without trust (V2). Never generate allow rules. The prompt tool's timeout behaviour must be tested. [replaces row 38's hook-based gate] | V1, V2, V3. |
| R5 | **Codex adapter:** enforcement = its sandbox plus approval callbacks over app-server plus `forbidden` prefix rules. Hooks are logging only. Hosted web search off. [amends row 37] | V5, V6. |
| R6 | **Run chooser shows only agents with real enforcement, listed as concrete enforced capabilities** ("blocks writes outside these folders", "blocks network except these hosts", "asks you before new actions"), not letter tiers. "Watched" (old tier C) is removed. "Export to your agent's skills folder" exists outside the chooser, labelled unguarded. [replaces rows 36, 37] | Both reviews: a letter ranking misleads; inability to pause is not weaker security. |
| R7 | **Permission is the intersection of three records:** the sender's request, the receiver's grants, the app's policy. Each mid-run approval is bound to the exact request (hashed); a changed request invalidates it. The prompt is a concrete change ("add write access to this folder for this run"), not yes/no. | Review. |
| R8 | **Command permission model:** structured rules per operation (executable resolved to an absolute path from an allowlist, argument constraints), a small supported set in version 1. Anything else requires contained mode or is blocked. "Words plus a trailing *" is retired. [replaces row 11] | Arguments can select interpreters, plugins and output paths without shell chaining; search-path resolution can pick the wrong program. |
| R9 | **Rating outcomes split into three:** BLOCKED, no override (invalid signature, admin rights, always-blocked paths, registry redirection, download piped to shell, hidden characters in machine fields, enforcement unsupported on this machine); RED FINDINGS, run only after typing a confirmation sentence (heuristics: hidden characters in prose, suspicious phrases, long encoded blocks, HTML comments); AMBER/GREEN as before. The Unicode rule becomes: blocked in identifiers and machine fields, warning in prose. [amends rows 21, 23, 44b] | Review 1: static heuristics and hard policy are different evidence. Review 2: no override drives copy-paste bypass. Verified: default-ignorable characters appear in ordinary writing. |
| R10 | **The public check page reports file findings only.** Final run status depends on the machine and the runner and is computed in the app. [amends row 29] | The same file has different access on different machines. |
| R11 | **Green wording:** "nothing in this file asks for more than it declares, and everything declared is low risk." Not "no mid-run questions". [amends row 27] | Code rules cannot prove prose mentions every needed resource. |
| R12 | **File format: our own typed YAML (fields have kinds: list, boolean, date). "Export as SKILL.md" is a labelled, unguarded export.** [replaces rows 9, 10] | Both reviews; conforming to the skill syntax does not make another agent enforce anything, and string-only metadata was ugly for nothing. |
| R13 | **Signing stays text-based (SSHSIG), with the clean-up rule, but documented precisely:** whitespace-only edits are by design not authenticated; the signature block is delimited by fixed markers, never found by heuristics; DSSE's "no canonicalization" position is noted and the clean-up is kept minimal. Optional `pin:` field binds a repository revision or content hash. **Transit test through WhatsApp, Gmail and Outlook is a launch blocker; the link is the robust channel.** Rejected: signing the parsed structure, because that puts the parser in the trust path. [amends rows 13, 14] | V8 (DSSE); review 2's transit-mutation scenario. |
| R14 | **Identity wording:** "signature valid, sender unknown" until identity is established. Dedicated signing keys (GitHub distinguishes signing from authentication keys). Key rotation and lost-key recovery flows specified. GitHub control proves account control, not employment or brand authority. [amends rows 16, 17] | Review. |
| R15 | **Child environment reduced to an explicit minimum allowlist**, not "strip our secrets". App settings, signer keys, gate, logs and recovery store are outside every agent-writable area. | Credentials live in project files, env vars and backups too. |
| R16 | **Preview shows the model provider and which categories of data will leave the laptop.** Reading a file and sending it to the model is disclosure. | Review. |
| R17 | **Secret placeholders authorise one specific use** (an auth header to one named host), never substitution into commands, URLs or files. Tool output is scrubbed. | Review. |
| R18 | **Undo redesigned:** a durable change journal written before each change; recovery data outside the agent-writable area; restore only when the current file still matches the version our run produced, else a conflict; external actions marked irreversible. [replaces row 42] | Copy-before-overwrite missed deletion, moves, subprocess changes and later user edits. |
| R19 | **Path checks happen on an open handle before any mutation, and the same handle is used for the write.** Tests include hard links and concurrent directory changes. [amends row 30] | Rechecking after opening is too late if opening truncated. |
| R20 | **A hostile test suite gates every adapter and every supported agent version:** (1) an approved hostile program tries to read a forbidden file and open a forbidden connection; (2) crashed gate, disconnected pipe, invalid hook response, unknown tool all stop the run; (3) repository config, plugins or an agent update cannot weaken policy; (4) an interrupted run resumes without duplicate effects; (5) hostile output cannot forge app messages or leak secrets into logs. Supported agent versions are pinned. | Review 1, section D. |
| R21 | **Milestone 0 added before any UI:** prove the execution boundary with one runtime and one task on this machine, with test output. | Review 1, section E: too many simultaneous unknowns otherwise. |
| R22 | **Reports** become a separately approved, previewable export. **Kill switch** spec must cover offline behaviour, stale data, compromised signing keys, signed revocations, and the fact that a pre-run check cannot stop a running task. [amends row 6] | Review. |
| R23 | **If browser control ships:** the proxy must be enforced at the operating-system boundary, not by browser configuration, and the screen must say that domain filtering cannot tell "download my statement" from "change my settings" on the same site. [amends row 43] | Review. |

### Kept against the reviews, with reasons

- Steplet name: one Swiss national mark by an individual; review 2 judged it fine, review 1 could not verify. Kept; re-check before any commercial step.
- Review 2's guess that WebView2 might block `steplet://` registration: dismissed. Registration is a per-user registry entry, verified earlier, unrelated to WebView2.
- Review 2's "sign the parsed structure": rejected, see R13.
- Tauri: kept (R7 corrects only the justification).

### Product scope after review round 1 (7 Oct 2026)

| # | Decision | Why |
|---|---|---|
| R24 | **Browser control: version 2 by default** (the release after the first public one). In version 1 a browser steplet can be written, sent and previewed; the app says it needs browser control, coming next release. The user may still move it into version 1 as the last milestone; if so, R23 applies. | Both reviews and the verified same-site limit. |
| R25 | **Hosted relay: public free tier with a fixed total budget to burn**, per-account daily cap, per-run cap, one running task per account, cheap models only, caps enforced server-side, hard stop when the budget is spent, then paid only. Google or GitHub sign-in. Fake-account risk is bounded by the total budget, not eliminated. Built in milestone 2, not earlier. [replaces row 44 and the earlier "invited pilot" option] | User decision: burn a set amount for popularity, then charge. |
| R26 | **Revenue candidates, none decided:** monthly subscription for hosted runs; sender-paid runs (a brand or team pre-pays so receivers run free); team plans with verified-sender badge, link-page branding, result reports; pay-as-you-go credits. | User asked for options. |
| R27 | **Use cases, final for version 1:** (1) developer to developer: check out this repository at an exact commit, run its tests, return a report, contained; (2) maintainer to contributor: run this triage script to reproduce a bug, return the log, contained; (3) child to parent: sort this folder by type, agent proposes, app code moves and verifies, native; (4) helper to colleague: back up these folders to a chosen drive and verify, asks first, resumes, native; (5) attacker: innocent-looking repository task whose test fixture tries to read a private key and phone home, contained, the pitch clip; (6) friend to friend: browser download with personal login, version 2, previewable only. Security tests, not demos: a permitted website serving injected instructions; a permitted destination that is attacker-controlled; a permitted command trying to modify the gate or recovery store; a crashed gate mid-run. [replaces row 7 and its table] | Approved by the user. |

## After milestone 0 (8 Oct 2026)

Measured on one Windows 11 Home laptop with Codex 0.160.1 and Claude Code 2.1.220. Evidence, options and reasons: `docs/milestone-0-wrap-up.md`; raw output: `docs/milestone-0-results.md` (test IDs and F-numbers point there). The user chose the recommended option for every item on 8 Oct 2026.

### Decisions taken

| # | Decision | Why |
|---|---|---|
| D1 | **Codex admin mode does not count as contained in version 1**, because loopback stays open (F7). It is not offered for files that run code. Milestone 1 runs two measured trials to close loopback: Steplet's own Windows Filtering Platform block for the sandbox accounts, and Codex's `mxc` engine. If either passes T1b with loopback blocked, Codex admin becomes contained with no change to the screen wording. The suite also gets a listener on the Windows side for the WSL runner. | Only option where every promise on screen is backed by a passed test today. A warning does not repair missing containment (R2). |
| D2 | **Codex and a received repository's `.codex/` files: two layers.** (1) Steplet owns a private Codex settings folder, writes an explicit `untrusted` entry for the run folder in Codex's own form (lower-case path), restores the file from a known copy before every run, watches it during the run and stops the run if it changes. (2) Every run gets a fresh working copy without `.codex/`. T3c to T3f and the `untrusted` follow-up join the hostile suite for every Codex version, run through the app-server route. Milestone 1 tests whether permission profiles passed on the command line survive `--ignore-user-config`; if so, that may replace layer 1. The private folder and its sign-in sit where fenced programs cannot read them and are on the always-blocked list. | F16, F17, F20, F21: the first run switched the fence off. One mitigation measured once is too thin alone. |
| D3 | **Claude Code and a received repository's settings: three layers.** (1) `--setting-sources user`, with the private Claude folder, `--strict-mcp-config` and Steplet's own `--settings`. (2) Every run gets a fresh working copy without `.claude/` and `.mcp.json`. (3) Any hook event in a contained run's stream stops the run and marks the adapter broken. Plus a preview line listing the agent settings files the repo carries, saying Steplet tells the agent not to load them (never "will not be loaded"), and a check after each run that no trust was recorded for the run folder. | F41, F44, F45: repo hooks ran outside the fence in a never-trusted folder. |
| D4 | **No agent resume for contained runs in version 1.** After an interruption, Steplet discards the working copy, prepares a fresh one, and offers "run again from the start". Resume stays where Steplet's own code makes the changes (use cases 3 and 4). | F46: resume repeated a step 3 of 3. A checker sees commands, not effects, so a resume guard would look like a guarantee and not be one. |
| D5 | **Codex admin mode needs tools installed for all users.** Steplet changes no permissions in the receiver's folder. Before each run it checks that the fence can start every needed program (from R8's absolute-path list) and says why not if it cannot. Revisit if Codex admin becomes contained and receivers hit this often. | F10. Opening the receiver's folder to the sandbox accounts trades a usability problem for a security one (F8, F9). |
| D6 | **Checkout: Steplet clones without checking out (hardened git, unfenced, fresh clone every run), then runs the checkout inside the fence as its own step with no agent; the agent's step follows with `.git` read-only.** Where a runner cannot do the fenced checkout step (Codex, until a checkout-only profile is shown to work), Steplet's own code does the checkout with hardened git. A working copy whose `.git` was writable is never handed back as a folder to run git in. The git-hardening list is checked in milestone 1. | F12, F30. A program that can write `.git` can plant hooks that later run unfenced. |
| D7 | **A model switch after a refusal is allowed and never silent:** the live log and report name every model used and show the refusal. Milestone 1 tests whether the switch can be turned off. | F32: 6 of 20 runs switched model with only one stream event to show it. |
| D8 | **Commands that skip the checker:** a shell command that ran without asking the checker is accepted only if it is on Steplet's own fixed list of harmless commands; any other one stops the run and marks the adapter broken (row 38). Milestone 1 tests whether a setting can make every command ask; if one exists, use it and drop the list. | F34: read-only commands skip the checker routinely, so row 38 as written would stop almost every run. |

### Corrections to earlier rows

| # | Row | Correction |
|---|---|---|
| C1 | R2 | Contained on Windows today: Claude Code in WSL2 (measured). Codex admin mode only after D1's condition is met. Codex non-admin mode is not contained: it blocks writes only (F4). |
| C2 | R4, V2 | Project-folder hooks **do** run in headless mode without trust, outside the fence (F41, 2.1.220). Replaced by D3. |
| C3 | V6 | Measured: Codex non-admin blocks writes only; its "network off" stops only programs that obey proxy settings (F4, F5). Codex admin blocks writes, declared reads and the internet, not loopback (F7). |
| C4 | 38 | The stop rule stands, with D8's exception for Steplet's fixed list of harmless commands. |
| C5 | 40 | Replaced by D2 (Codex) and D3 (Claude Code). `--bare` was already replaced by R4; `--ignore-rules` drops Steplet's own `forbidden` rules, which R5 needs. |
| C6 | R20, test 4 | For agent runs: "an interrupted run starts again clean from a fresh working copy, with no duplicate effects inside the folder" (D4). For runs where Steplet's own code makes the changes: unchanged. |
| C7 | 8c, 39 | Codex is not offered for contained files in version 1 until D1's condition is met. What milestone 1 builds for Codex is set in the milestone 1 plan (P2 below). |

## Milestone 1 plan choices (8 Oct 2026)

Plan: `docs/plans/2026-10-08-milestone-1-plan.md`, approved by the user on 8 Oct 2026 together with its technical choices (section 3 of the plan). The user chose the recommended option for P1 to P6.

| # | Decision | Why |
|---|---|---|
| P1 | **Claude Code signs in with the receiver's own Anthropic API key in milestone 1.** Subscription sign-in only after a written yes from Anthropic, which the user asks for. The key must be invisible to fenced programs (spike S3) before this ships. | Anthropic's legal page forbids third-party developers to "route requests through Free, Pro, or Max plan credentials on behalf of their users" or to "store, or intermediate Claude.ai credentials"; the API key route is documented for `-p` mode. |
| P2 | **Codex in milestone 1: trials only** (`mxc`, then Steplet's own loopback block, then D2's tests through a thin `app-server` test client). The Codex adapter is built only if a trial passes, in milestone 2. | Weeks of work on something that cannot be offered (C7); `app-server` is marked experimental in 0.160.1. |
| P3 | **Repository sources in milestone 1: a public `https` address plus the full commit hash only.** No local folders, no private repositories. | Local clones bring back git's local-clone holes and planted hooks; private ones need credentials near the run (row 24, R15). |
| P4 | **Signature name space `steplet-v1`.** Replaces row 13's `runfile-v1`. | Nothing has been signed for real yet; cheapest moment to change. |
| P5 | **Hostile suite runs on this laptop in a separate WSL distribution made only for tests, with a test-only Anthropic API key that has a spending cap.** A second machine before launch. | Keeps real sign-ins away from hostile test code; Windows 11 Home has no Hyper-V for a local virtual machine. |
| P6 | **`C:\dev\runFile` becomes a git repository** after the user moves `keys.txt` and `sk.json` out of it. Licence decided before the first public push. | Commits need a repository; the two files must never enter history. |

## Background

- Research: `research/2026-10-06-landscape-and-feasibility.md`
- Architect memos: `.claude/agent-memory/architect/` (adapter layer, built-in agent, provider access, signing, SKILL.md constraint, risk rating, scanner sources)
