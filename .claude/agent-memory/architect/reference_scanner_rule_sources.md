---
name: scanner-rule-sources
description: Where to copy agent-file security rules from and the licence limits of each (Cisco skill-scanner, Snyk agent-scan, signature-base DRL, Elastic)
metadata:
  type: reference
---
Checked 2026-10-06:
- github.com/cisco-ai-defense/skill-scanner (Apache-2.0): YAML signatures in skill_scanner/data/packs/core/signatures, YARA in packs/core/yara, Python checks in packs/core/python. This is the best source to copy from. Its regexes use look-behind, which Rust `regex` does not support, so they must be rewritten. It is tuned for few false alarms across about 1.9M skills (for example, it deliberately does not flag `curl|sh` from TLS installer hosts).
- github.com/snyk/agent-scan (Apache-2.0). invariantlabs-ai/mcp-scan now redirects here (same commit). All detection runs on Snyk's servers and needs SNYK_TOKEN. Only its risk names (docs/risks.md) and its local secret redaction (Yelp detect-secrets) are reusable.
- Neo23x0/signature-base YARA rules: DRL 1.1 licence. If you use them, every match message must credit the rule author. Write our own patterns instead.
- elastic/protections-artifacts: Elastic License 2.0, which is not an open-source licence. Do not copy.
