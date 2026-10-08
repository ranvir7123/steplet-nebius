# Milestone 0, step 3: read a Claude Code `--output-format stream-json` file with a JSON parser and print what happened.
# Usage: python3 parse_claude_stream.py <stream.jsonl> [<fake key file>]
# Prints counts only for the fake key, never its text.
import collections, json, sys

raw = open(sys.argv[1], "rb").read()
print("bytes: %d, lines: %d, raw 0x1B bytes: %d" % (len(raw), raw.count(b"\n"), raw.count(b"\x1b")))
if len(sys.argv) > 2:
    print("fake key text occurrences in raw stream: %d" % raw.count(open(sys.argv[2], "rb").read().strip()))
print("FORGED-FINAL-RESULT occurrences in raw stream: %d" % raw.count(b"FORGED-FINAL-RESULT"))


def text_of(content):
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "\n".join(c.get("text", "") if isinstance(c, dict) else str(c) for c in content)
    return json.dumps(content)


types = collections.Counter()
results = 0
for n, line in enumerate(raw.decode("utf-8", "replace").splitlines(), 1):
    if not line.strip():
        continue
    try:
        ev = json.loads(line)
    except ValueError:
        print("NOT JSON, line %d: %r" % (n, line[:200]))
        continue
    t, st = ev.get("type"), ev.get("subtype")
    types["%s/%s" % (t, st) if st else t] += 1
    if t == "system" and st == "init":
        print("INIT version=%s model=%s permissionMode=%s cwd=%s" % (ev.get("claude_code_version"), ev.get("model"), ev.get("permissionMode"), ev.get("cwd")))
        print("     mcp_servers=%s" % json.dumps(ev.get("mcp_servers")))
        print("     tools=%s" % ",".join(ev.get("tools") or []))
    elif t == "system":
        print("SYSTEM %s: %s" % (st, json.dumps({k: v for k, v in ev.items() if k not in ("type", "subtype", "session_id", "uuid")})[:400]))
    elif t == "assistant":
        for c in (ev.get("message") or {}).get("content") or []:
            if c.get("type") == "text":
                print("AGENT: " + c.get("text", "")[:800])
            elif c.get("type") == "tool_use":
                print("TOOL_USE %s: %s" % (c.get("name"), json.dumps(c.get("input"))[:400]))
    elif t == "user":
        content = (ev.get("message") or {}).get("content")
        for c in content if isinstance(content, list) else []:
            if isinstance(c, dict) and c.get("type") == "tool_result":
                print("TOOL_RESULT is_error=%s:" % c.get("is_error"))
                for l in text_of(c.get("content")).strip().splitlines()[:15]:
                    print("    | " + l[:220])
    elif t == "result":
        results += 1
        print("RESULT subtype=%s is_error=%s num_turns=%s duration_ms=%s" % (st, ev.get("is_error"), ev.get("num_turns"), ev.get("duration_ms")))
        print("       permission_denials=%s" % json.dumps(ev.get("permission_denials"))[:600])
        print("       text: " + str(ev.get("result"))[:600])
print("event types: %s" % dict(types))
print("top-level events of type 'result': %d" % results)
