---
name: risk-rating-decision
description: 2026-10-06 decision on how runfile green/amber/red is computed, colour semantics, signing axis, and the single shared Rust policy crate used by preview AND guard
metadata:
  type: project
---
Decided 2026-10-06 (architect, design memo; not yet built).

Decision:
- One pure Rust crate (`runfile-policy`) does strict parsing, rating AND the allow/deny/ask matcher. It is built natively into the Tauri Rust core (which is the policy engine of [[decision-agent-adapter-layer]]; the thin gate helper exe forwards each action to it over the local pipe) and to WASM for a browser check page. Preview and guard call the same `Policy::check`, so they cannot disagree.
- Colour = the worst finding (Note < Amber < Red). No adding up of scores. Any combined-risk cases are written as explicit rules.
- Red = "outside what the app can protect you from". You cannot run a red file from the app in v1. There is no override.
- Signature is a SEPARATE "who sent this" line and does not raise or lower the colour. Exceptions that force red: an invalid signature, or a known sender name signed with a different key. Reason: an attacker can make a new signing key for free.
- Pipe-to-shell (`curl|sh`, `irm|iex`) is red even from official installer hosts. Accepted cost: one-line installers like uv and rustup are blocked, so authors use winget or other package managers instead.
- The file's command patterns are UNTRUSTED. Never compile them as regexes. Use our own word-based matcher that refuses anything it can't understand: if it can't parse a command, the answer is "ask the human".
- The guard returns an explicit allow for matches. Never generate Claude Code `allow` rules from the file, because Claude Code's own matcher strips wrappers and env vars, so the two would disagree. Generate deny rules only, as a second safety net.
- Domain age (WHOIS/RDAP) is never an input to the colour, because it needs the network and would make the result non-deterministic.

Rejected: using Snyk agent-scan or Cisco skill-scanner directly (cloud-only or Python, tuned for few false alarms, unaware of the runfile schema, a second matcher); a TypeScript module plus a separate guard (two matchers); a Rego/CEL policy engine (an extra language, and parsing still needs host code); YARA-X as the core engine (works on raw bytes, not fields; licence attribution burdens). YARA-X is OK later as an optional pack for the body text.

**Why:** OWASP LLM06 says to enforce in code, not in the model. DeepJack shows that a confirm screen which disagrees with what is actually enforced is worse than no screen.
**How to apply:** Don't re-litigate these points without new evidence. The rule thresholds (size limits, blob lengths) are explicitly provisional and should be tuned on the test corpus. See [[scanner-rule-sources]], [[decision-builtin-agent]].
