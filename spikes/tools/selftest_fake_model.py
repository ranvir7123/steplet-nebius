"""Self-test for fake_model.py: start it, send the kinds of requests Claude Code sends, check each answer."""
import json
import os
import subprocess
import sys
import tempfile
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
PORT = 47999
KEY = "SELFTEST-KEY-0123456789"
tmp = tempfile.mkdtemp()
script, keyfile, log = (os.path.join(tmp, n) for n in ("script.json", "key", "log.jsonl"))
with open(script, "w") as f:
    json.dump({"steps": [{"tool": "Bash", "input": {"command": "echo one"}},
                         {"refusal": {"delta": {"spike_marker": True}}, "text_after_refusal": "after"},
                         {"text": "end"}]}, f)
with open(keyfile, "w") as f:
    f.write(KEY)
server = subprocess.Popen([sys.executable, "-I", os.path.join(HERE, "fake_model.py"), str(PORT), script, log, keyfile])
time.sleep(1.5)
failures = 0


def check(name, ok):
    global failures
    failures += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + name)


def post(path, body):
    req = urllib.request.Request("http://127.0.0.1:%d%s" % (PORT, path), json.dumps(body).encode(),
                                 {"content-type": "application/json", "x-api-key": KEY})
    return urllib.request.urlopen(req, timeout=10).read().decode()


def events(raw):
    return [json.loads(line[6:]) for line in raw.splitlines() if line.startswith("data: ")]


TOOLS = [{"name": "Bash", "input_schema": {"type": "object"}}]
GO = {"role": "user", "content": "go"}
RESULT = {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "x", "content": "one"}]}
try:
    m = json.loads(post("/v1/messages?beta=true", {"model": "m", "messages": [GO], "tools": TOOLS}))
    check("step 0 answers with a Bash tool call", m["stop_reason"] == "tool_use" and m["content"][0]["name"] == "Bash")
    ev = events(post("/v1/messages?beta=true", {"model": "m", "stream": True, "messages": [GO], "tools": TOOLS}))
    partial = "".join(e["delta"].get("partial_json", "") for e in ev if e["type"] == "content_block_delta")
    check("streamed step 0 rebuilds the same tool input", json.loads(partial) == {"command": "echo one"} and ev[-1]["type"] == "message_stop")
    ev = events(post("/v1/messages", {"model": "m", "stream": True, "messages": [GO, RESULT], "tools": TOOLS}))
    stop = [e for e in ev if e["type"] == "message_delta"][0]["delta"]
    check("step 1 refuses, with the extra field merged", stop["stop_reason"] == "refusal" and stop.get("spike_marker") is True)
    m = json.loads(post("/v1/messages", {"model": "m2", "messages": [GO, RESULT], "tools": TOOLS}))
    check("a repeat at step 1 gets the text after the refusal, under the model asked for", m["content"][0]["text"] == "after" and m["model"] == "m2")
    m = json.loads(post("/v1/messages", {"model": "m", "messages": [{"role": "user", "content": "title?"}]}))
    check("a request without tools gets 'ok'", m["content"][0]["text"] == "ok")
    check("count_tokens answers", json.loads(post("/v1/messages/count_tokens", {}))["input_tokens"] == 1)
    text = open(log).read()
    entries = [json.loads(line) for line in text.splitlines()]
    check("six requests logged", len(entries) == 6)
    check("the key matched and its text is not in the log", all(e.get("api_key_matches_test_key") is True for e in entries) and KEY not in text)
finally:
    server.terminate()
print("failures: %d" % failures)
sys.exit(1 if failures else 0)
