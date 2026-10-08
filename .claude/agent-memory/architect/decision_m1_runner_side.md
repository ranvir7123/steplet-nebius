---
name: decision-m1-runner-side
description: 2026-10-08 architect advice for milestone 1 runner side - WSL gate channel, launch/stop, storage layout, checkout git, Codex scope, hostile suite as code; rejected options and why
metadata:
  type: project
---

Advice given 2026-10-08 (not yet approved by the user; check docs/decisions.md for what was adopted). Confidence ~70%.

1. Gate channel (Claude Code in WSL -> Windows app): small Windows gate .exe started by Claude Code through WSL interop as the --permission-prompt-tool MCP server, talking to the app over a per-run named pipe (DACL = user SID only, deny RESTRICTED, PIPE_REJECT_REMOTE_CLIENTS, FIRST_PIPE_INSTANCE, both ends check each other's PID/image, per-run token). Runner-up: `--permission-prompt-tool stdio` control protocol over wsl.exe stdio (2.1.220 binary has it; SDK-internal, no spec; GitHub #34046 reported it skipping can_use_tool, closed not planned 2026-05-03). Rejected: TCP/HTTP listeners (D1 loopback, reachable by all local programs), hvsocket (HKLM registration needs admin), npiperelay (same interop dependency, more parts).
2. Launch: wsl.exe --distribution-id --cd --exec, Windows env cleared (no WSLENV), env set explicitly on Linux side, prompt via stdin file, coreutils timeout backstop, stop = pkill -TERM by --session-id, sweep at app start. Linux disk: private root (~/.local/share/steplet: pinned CLI, CLAUDE_CONFIG_DIR, per-run settings/mcp json) separate from work root; logs/journal/gate on Windows side; denyRead /mnt + private root + other runs (allowRead this run).
3. Checkout: receiver's WSL git, hardened -c/env list, URL(https)+full commit only in M1; fenced checkout via pinned srt (sandbox-runtime, beta); .git read-only via sandbox.filesystem.denyWrite + Edit deny rule. Runner-up gix (0.88). Distro git version numbers do not track fixes (Ubuntu backports), so no version gate.
4. Codex: trials only (mxc, WFP, D2 app-server tests) with a thin app-server test client; no adapter until a trial passes. app-server is "[experimental]" in 0.160.1 help.
5. Hostile suite: Rust harness in repo driving real adapter code; scripted-model tier via ANTHROPIC_BASE_URL mock + live-model tier per version; self-hosted Windows machine with WSL2; Windows-side listener on 127.0.0.1, WSL host IP, 0.0.0.0; NAT and mirrored modes.

**Why:** every promise must stay backed by a measured test; no listener reachable by other local programs (D1).
**How to apply:** if the user adopted these, do not re-litigate; if the interop spike fails, switch Q1 to the stdio runner-up.
Machine fact 2026-10-08: no C:\Users\RANVIR\.wslconfig, so WSL ran in default NAT mode during milestone 0.
