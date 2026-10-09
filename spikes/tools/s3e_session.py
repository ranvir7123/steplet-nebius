# Milestone 1, phase 0, S3e: summarise the session record Claude Code saved for the one S3e prompt. Throwaway.
# Prints each record's type and, for assistant or result records, whether it was an error and the first
# 160 characters of its text. Read-only.
# Usage: python3 -I s3e_session.py <session.jsonl>
import json
import sys

for n, line in enumerate(open(sys.argv[1], encoding="utf-8"), 1):
    try:
        e = json.loads(line)
    except ValueError:
        print(n, "not JSON")
        continue
    kind = e.get("type")
    msg = e.get("message") or {}
    text = ""
    content = msg.get("content") if isinstance(msg, dict) else None
    if isinstance(content, list):
        text = " ".join(c.get("text", "") for c in content if isinstance(c, dict))
    elif isinstance(content, str):
        text = content
    extra = {k: e[k] for k in ("isApiErrorMessage", "error", "subtype", "is_error") if k in e}
    model = msg.get("model") if isinstance(msg, dict) else None
    print(n, kind, f"model={model}" if model else "", extra or "", repr(text[:160]) if text else "")
