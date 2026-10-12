# Handoff: Steplet, start of build planning

Written 7 Oct 2026 at the end of the design sessions. Read this first in a new session, then the two files it points to. Nothing in the repository is code yet.

**Update, 13 Oct 2026: phase 2 finished on 13 Oct 2026; results in `docs/milestone-1-phase-2-results.md`** (summary at its end: all four "done means" pass; 36 rating rules in `docs/rules.md`, each with a hit and a near-miss in `corpus/`; the run-time checker `Policy::check`; `steplet-wasm` giving byte-for-byte the native answer on all 134 corpus files; decisions G1 to G14 in `docs/decisions.md`). The ordinary corpus comes out 1 green, 59 amber, 2 red, 0 blocked: the amber tuning question is for the user. Not merged into `main` yet. Next: phase 3's build plan, which first needs E3 (the folder layout) decided.

**Update, 11 Oct 2026, later: phase 1 finished on 11 Oct 2026; results in `docs/milestone-1-phase-1-results.md`** (summary at its end: all five "done means" pass; the sender's guide is `docs/steplet-format.md`). The property tests found that `ssh-key` 0.6.7 accepts a signature whose inner lengths were edited; `sig::verify` now requires the exact text OpenSSH writes. Decisions F11 (that check) and F12 (the 14-day rule for every crate in every lock file) were confirmed by the user the same day. Phase 1 is merged into `main` by fast-forward. Next: phase 2's build plan.

**Update, 11 Oct 2026: phase 1's build plan is approved** (`docs/plans/2026-10-11-milestone-1-phase-1.md`, F1 to F3 as recommended; decisions F1 to F10). The file's header fields are now fixed (decisions F1). Next: run the plan from Task 1, inline.

**Update, 10 Oct 2026: phase 0 finished on 10 Oct 2026; results in `docs/milestone-1-spikes.md`** (summary at its end: S1, S2, S3, S4, S7, S8, S9 and L1 pass; S5, S6 recorded; the work-folder Read rule in the planned folder layout blocks the agent's own working copy, so the layout is open for phase 3; decisions E1 to E3 and corrections C8, C9 recorded in `docs/decisions.md`). Next: the phase 1 build plan.

**Update, 9 Oct 2026: phase 0 tasks 4 to 14 are done** (S7, S2, S1, S3, S4, S8, S9 pass; S5, S6 recorded; Task 15 waits for the test key; the app must give `wsl.exe` `SystemRoot`, never an empty environment; the key goes through `apiKeyHelper` only). Where to pick up: `docs/milestone-1-phase-0-handoff.md`.

**Update, 8 Oct 2026, evening: milestone 1, phase 0 has started.** Tasks 1 to 3 of `docs/plans/2026-10-08-milestone-1-phase-0.md` are done (git repository, Rust tools, the test distribution `steplet-test`). Where to pick up, and traps already hit: `docs/milestone-1-phase-0-handoff.md`.

**Update, 8 Oct 2026: milestone 0 is finished.** Read `docs/milestone-0-wrap-up.md` (what each runner may promise, the measured numbers, what is still unverified) and the "After milestone 0" section of `docs/decisions.md` (D1 to D8, corrections C1 to C7). Facts 1 and 3 below were corrected by measurement. Next: the milestone 1 plan.

**Update, 7 Oct 2026 evening: milestone 0 is in progress.** Plan: `docs/plans/2026-10-07-milestone-0-execution-boundary.md`. Results with real output: `docs/milestone-0-results.md`. Where to pick up, and traps already hit: `docs/milestone-0-handoff.md`.

## What Steplet is, in four sentences

A steplet is one text file (YAML header plus Markdown steps) that carries a task for an AI agent: who sent it, the goal, the steps, what the agent may touch, what it must never do, how to undo, when it expires. A sender writes it once and sends it like a PDF or a link. The receiver double-clicks it on Windows; our desktop app (Tauri 2, Rust core) shows the whole plan, a risk rating decided by code rules, who signed it, and a choice of runners; the receiver approves. The runner does the work on the receiver's own machine while the app checks every proposed action and, for anything that executes other people's code, an operating-system sandbox contains the programs.

## Where everything is

- `docs/decisions.md`: every decision with its reason, in order. Rows 1 to 46 are the original design; the "Review round 1" section (V1 to V10, R1 to R27) supersedes several of them and says which. **When a row and an R-row disagree, the R-row wins.**
- `research/2026-10-06-landscape-and-feasibility.md`: what exists, what was verified, what could not be.
- `.claude/agent-memory/architect/`: the four architect memos (adapter layer, built-in agent, provider access, signing, SKILL.md constraint, risk rating, scanner sources). Written before review round 1; read them with the R-rows in mind.
- `docs/review-prompt.md`: the prompt sent to two outside AI reviewers; their feedback shaped the R-rows.

## The ten facts a planner must not forget

1. Claude Code has no containment on native Windows. Claude Code contains only inside WSL2 (measured in milestone 0: writes, listed reads, internet, loopback, Windows programs all blocked). **Corrected 8 Oct 2026:** Codex's native Windows sandbox in non-admin mode blocks writes only; in admin mode it blocks writes, listed reads and the internet but not loopback, so it is not counted as contained (decisions D1, C1).
2. Hooks are logging, not enforcement, in both Claude Code and Codex (their own docs say so). Claude Code's hard gate is the permission system plus `--permission-prompt-tool`. Codex's is its sandbox plus app-server approval callbacks plus `forbidden` prefix rules.
3. `--bare` disables subscription login in Claude Code. Do not use it. **Corrected 8 Oct 2026:** in headless mode a never-trusted project folder's hooks **do** run, outside the fence (F41). Every contained run uses `--setting-sources user` plus the layers in decision D3.
4. Two execution modes: contained and native. A file that installs, builds, tests or runs scripts may only run contained. Without a contained runner, the app says "cannot run safely on this computer".
5. The built-in agent runs file tools and web fetch natively; shell commands only through a contained runner. For sorting and backup, the agent proposes and the app's own code moves and verifies.
6. The file format is our own typed YAML. "Export as SKILL.md" is a labelled, unguarded export, not a fallback we call safe.
7. Signing: OpenSSH signature format, Ed25519, embedded in the header, over the cleaned-up text minus the signature block. Whitespace-only edits are not authenticated, by design. Transit through WhatsApp, Gmail and Outlook must be tested before launch.
8. Rating has three outcomes: blocked (no override), red findings (typed confirmation), amber/green. Colour is decided by code rules only. The public check page reports file findings; final run status is per machine.
9. One rule checker (`Policy::check`) serves preview, guard and ask screen. Never two matchers.
10. Every adapter and every supported agent version must pass the hostile test suite (R20) before it is offered in the run chooser.

## Build order

- **Milestone 0 (first, no UI):** prove the execution boundary on this Windows machine with one runtime and one task, with real command output. Concretely: run use case 1 (check out a repository at a commit, run tests, report) through Codex's native Windows sandbox, and through Claude Code inside WSL2 if WSL2 is available, with the hostile fixture from use case 5 inside the repository. Show that the forbidden read and the forbidden connection fail, and that a crashed gate stops the run. Also test: the permission-prompt tool's timeout behaviour; whether Codex hooks can be supplied per run; whether `--ignore-user-config` skips repository `.codex/` files. Record every command and its output.
- **Milestone 1:** file format and parser (strict, parse-once), signing and verification with test vectors, rating library (Rust, with a WebAssembly build), preview screen, Claude Code adapter, Codex adapter, public check page. Use cases 1, 2 and 5 work for developers.
- **Milestone 2:** built-in agent (files, fetch; commands only via contained runner), hosted relay with Google or GitHub sign-in and the burn budget (R25), use cases 3 and 4.
- **Milestone 3:** link page (static, hands the file to the app via `steplet://`), reports as a separately approved export.
- **Milestone 4:** kill switch with signed revocations, verified senders via GitHub keys.
- **Version 2:** browser control (R23, R24).

## Open items, with who decides

- Browser control in version 1 or 2: default is version 2 (R24). Only the user can move it.
- Whether a third-party app may drive a receiver's Claude Code subscription headlessly: not found in any terms page. Needs checking before launch; a planner should flag where it bites.
- Name "Steplet": one Swiss national trademark by an individual; re-check before any commercial step.
- Effort estimates in the architect memos are judgments, not measurements. Milestone 0 produces the first measured data.

## Working rules carried over from the sessions

- Plain language; the owner is a student. Explain a term the first time it appears.
- Never claim something works without the command and its real output.
- Technical decisions: dispatch one or two architect agents, then decide from their evidence. Product decisions: ask the user with a recommendation first.
- Plan before touching more than one file; wait for approval.
- No attribution lines in commits or pull requests.
