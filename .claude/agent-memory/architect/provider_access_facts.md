---
name: provider-access-facts
description: Which consumer AI subscriptions a third-party app may use as of 2026-10 - ChatGPT yes (Sign in with ChatGPT, open-source/local apps), Claude.ai no, Gemini consumer plans no
metadata:
  type: project
---

Checked 2026-10-06. These policies move fast, so re-check before relying on them.

- **OpenAI: allowed since DevDay, 2026-09-29.** "Sign in with ChatGPT" lets Plus and Pro users spend their plan allowance in **open-source or locally run apps** (developers.openai.com/siwc). Paid or hosted apps must apply through an interest form.
  - The OAuth flow uses a 127.0.0.1 loopback redirect with PKCE (a one-time proof code) and dynamic client registration.
  - Calls go to the public Responses API: `store:false` and `stream:true` are required. temperature, max_output_tokens and several other parameters are banned.
  - Function tools must be grouped in namespaces. Native computer use, file search and hosted MCP are not supported.
  - Do NOT call chatgpt.com/backend-api.
- **Anthropic: not allowed.** Claude.ai login cannot be used by third-party apps; API key only.
- **Google: not allowed.** AI Pro and Ultra benefits apply "only within the Google AI Studio web interface". Using Gemini CLI or Antigravity OAuth from third-party tools got paying accounts suspended (enforced from 2026-03-25). Pro and Ultra include monthly Cloud credits ($10 and $100) that can pay for API use.

**Why it matters:** ChatGPT sign-in is the only way a non-technical receiver can run the built-in agent with no API key and no sponsor. See [[decision-builtin-agent]].
**How to apply:** treat ChatGPT plan usage as a preview that may change. Keep API-key providers as the fallback.
