---
name: decision-builtin-agent
description: 2026-10-06 decision on the built-in agent - loop in Rust via genai behind own trait, guard in Rust, v1 scope = files+commands+fetch, browser and sponsored runs deferred to v2
metadata:
  type: project
---

Decided 2026-10-06 (architect). No app code existed in C:\dev\runFile then; it was grounded in research/2026-10-06-landscape-and-feasibility.md and the founder's brief.

**Decision**
- The built-in agent loop runs in the Tauri **Rust backend**, in the same process as the guard and the Claude Code adapter. Both runners share one Runner interface, one event stream and one policy engine.
- Provider library: `genai` crate (0.7, Apache-2.0), pinned, behind our own `ModelProvider` trait so it can be swapped. Sign in with ChatGPT gets its own adapter: Responses API, store=false, stream=true, banned parameters stripped. See [[provider-access-facts]].
- v1 = adapters + built-in agent with file, command and fetch tools. Browser control and sponsored runs go to v2.
- Browser in v2: drive the receiver's installed Edge or Chrome with a **dedicated persistent profile**, never the default one. Enforce allowed sites with our own local allowlisting proxy, because Playwright MCP's `--allowed-origins` "does not serve as a security boundary". For login, open the browser without automation attached, let the receiver log in, then relaunch with automation.
- Sponsored runs in v2. Step A: the sender mints a capped, expiring OpenRouter key; no server. Step B: our own proxy, only if demand is proven.

**Rejected**
- Vercel AI SDK in the webview: the loop dies with the window, hidden webviews get throttled, and it shares a process with the screen that shows hostile content.
- Node sidecar with AI SDK (runner-up): a second runtime and binary to sign, plus API keys crossing a process boundary. Revisit if genai fails.
- LiteLLM: needs a Python runtime, and its PyPI package was itself compromised on 2026-03-24.
- rig: breaking changes every month; its `chatgpt` provider calls the private chatgpt.com/backend-api endpoint, which OpenAI's docs say not to use.
- Bundled or WebView2 browser: Google blocks sign-in inside embedded webviews, and macOS WKWebView has no CDP (the Chrome control protocol).
- Browser extension: Web Store review, and the agent would see all of the receiver's real sessions.

**Flip condition:** if a 2-day spike shows genai streaming tool calls are unreliable on Anthropic, OpenAI, Gemini or Ollama, or genai's single maintainer stops, move the loop to a Node sidecar running the AI SDK. The guard stays in Rust either way.

**How to apply:** do not re-argue where the loop runs or the v1 scope unless the flip condition happened. Any new tool must go through the Rust policy engine.
