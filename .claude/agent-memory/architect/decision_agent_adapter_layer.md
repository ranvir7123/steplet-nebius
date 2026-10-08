---
name: decision-agent-adapter-layer
description: 2026-10-06 decision on the agent adapter layer - core-owned policy engine, normalized Action, per-agent native gate; tiers A-D; v1 = Claude Code + Codex + Copilot CLI
metadata:
  type: project
---

Decided 2026-10-06 (architect, research cut short by coordinator; confidence ~75%).

Decision: the app core owns ONE policy engine (allowed folders / command prefixes / domains / never list -> allow|deny|ask, fail closed). Each agent adapter only translates: runfile -> launch plan, agent's pre-action hook/approval callback -> normalized Action, Decision -> agent's reply format, agent event stream -> normalized Events. Adapters never decide. A shared gate helper exe talks to the app over a local pipe so hook-based agents can block and wait for the receiver (that is the "pause").
Coverage rule: any tool call seen in the event stream with no prior gate decision stops the run.

Tiers: A full guard (gate fires on every action, can ask receiver, live log), B rules/deny only (asks become denies), C watch-and-stop (no injectable gate, kill on breach after the fact), D skill-drop only.
V1 adapters: Claude Code (A), Codex CLI (A, native Windows sandbox confirmed on learn.chatgpt.com docs; repo docs/install.md "WSL2 only" is stale), Copilot CLI (B confirmed, A if SDK permission handler verified). Gemini CLI next (B via per-run --admin-policy). Cursor, pi, others: D/C until researched.
Command patterns in runfiles: prefix tokens with trailing * only (narrowest grammar every agent's native rules can express).

Rejected: ACP-only adapter (agent decides when to ask, so not every action is gated; Codex/Copilot/Gemini ACP are adapter/preview/experimental) - keep ACP as a later generic C/B adapter for breadth. Separate per-agent policy logic (duplicated security logic drifts). Our own OS sandbox (Experimental_CreateProcessInSandbox not in public headers). Claude-only (fails "work with everything").

**Why:** founder wants every popular agent; guard must be honest per agent.
**How to apply:** new agent = one adapter file implementing detect/capabilities/prepare/start/events/stop/resume; do not move policy logic into adapters. Flip condition: if headless hooks turn out not to fire on every tool call, move enforcement to an OS-level sandbox we own.
Unresearched as of this date: OpenCode, Amp, Goose, Aider, Cline, Windsurf.
