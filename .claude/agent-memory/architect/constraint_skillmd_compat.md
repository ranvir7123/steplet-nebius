---
name: constraint-skillmd-compat
description: Runfile must stay a valid SKILL.md; the reference validator (skills-ref) rejects unknown top-level frontmatter fields, so runfile fields must live under metadata as string values
metadata:
  type: project
---

The 2026-10-06 landscape research chose "runfile = superset of SKILL.md" so the file works as a skill in 40+ agents with no app.
The reference validator `skills-ref` (agentskills/agentskills, `validator.py`, `ALLOWED_FIELDS`) rejects any top-level key other than name, description, license, allowed-tools, metadata, compatibility. The spec defines `metadata` as a map of string keys to **string** values.

**Why:** extra top-level keys (sender, allowed, never, expires, signature) fail strict validation, and lists are not allowed inside metadata.

**How to apply:** put every runfile field under `metadata:` with a `runfile-` prefix and string values. List-valued fields (allowed list, never list) need an encoding inside a string, or the project must accept failing strict validation. Raise this with whoever designs the header schema. Related: [[decision-signing-v1]].
