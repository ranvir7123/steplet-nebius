"""Scripted stand-in for the Anthropic Messages API, for Steplet's phase 0 spikes. Not a model. Throwaway.

Usage: python3 -I fake_model.py <port> <script.json> <log.jsonl> [<file holding the expected test key>]

Requests that offer tools are the agent loop. Each gets the script step whose number is the count of
tool results already in the conversation, so retries and side requests never move the script along.
A step is one of:
  {"tool": "<name>", "input": {...}}                 the model asks for one tool call
  {"text": "..."}                                     the model answers and stops
  {"refusal": {"message": {...}, "delta": {...}},    the model refuses: stop reason "refusal", with the given
   "text_after_refusal": "..."}                       fields merged in; a repeat request at the same step
                                                      gets text_after_refusal instead
Requests without tools (titles, summaries) get the text "ok". One JSON line is logged per request.
Keys are never logged: only whether one was sent, and whether it equals the expected test key.
"""
import json
import sys
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

PORT = int(sys.argv[1])
STEPS = json.load(open(sys.argv[2], encoding="utf-8"))["steps"]
LOG = sys.argv[3]
EXPECTED_KEY = open(sys.argv[4], encoding="ascii").read().strip() if len(sys.argv) > 4 else None
LOCK = threading.Lock()
ANSWERED = {}  # step number -> requests already answered at that step


def log(entry):
    entry["time"] = round(time.time(), 3)
    with LOCK, open(LOG, "a", encoding="utf-8") as f:
        f.write(json.dumps(entry) + "\n")


def tool_results_in(messages):
    count = 0
    for m in messages:
        if m.get("role") == "user" and isinstance(m.get("content"), list):
            count += sum(1 for b in m["content"] if isinstance(b, dict) and b.get("type") == "tool_result")
    return count


def step_for(number):
    with LOCK:
        earlier = ANSWERED.get(number, 0)
        ANSWERED[number] = earlier + 1
    if number >= len(STEPS):
        return {"text": "Spike script finished."}
    step = STEPS[number]
    if "refusal" in step and earlier > 0:
        return {"text": step.get("text_after_refusal", "Answer after the refusal.")}
    return step


def reply_parts(step):
    """(content blocks, stop reason, fields to merge) for one step."""
    if "tool" in step:
        block = {"type": "tool_use", "id": "toolu_spike_" + uuid.uuid4().hex[:20], "name": step["tool"], "input": step["input"]}
        return [block], "tool_use", {}
    if "refusal" in step:
        return [], "refusal", step["refusal"]
    return [{"type": "text", "text": step["text"]}], "end_turn", {}


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"  # one response per connection; a stream ends when the connection closes

    def log_message(self, *args):
        pass

    def send_json(self, status, obj):
        data = json.dumps(obj).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        log({"method": "GET", "path": self.path})
        self.send_json(404, {"type": "error", "error": {"type": "not_found_error", "message": "spike server"}})

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers.get("content-length") or 0)) or b"{}")
        path = self.path.split("?")[0]
        key = self.headers.get("x-api-key")
        tools = [t.get("name") for t in body.get("tools") or []]
        entry = {"method": "POST", "path": self.path, "model": body.get("model"), "stream": bool(body.get("stream")),
                 "tools_offered": tools, "api_key_sent": key is not None,
                 "api_key_matches_test_key": (key == EXPECTED_KEY) if (EXPECTED_KEY and key) else None,
                 "bearer_sent": self.headers.get("authorization") is not None}
        if path.endswith("/v1/messages/count_tokens"):
            log(entry)
            return self.send_json(200, {"input_tokens": 1})
        if not path.endswith("/v1/messages"):
            log(entry)
            return self.send_json(404, {"type": "error", "error": {"type": "not_found_error", "message": "spike server"}})
        if tools:
            number = tool_results_in(body.get("messages") or [])
            step = step_for(number)
            entry.update(kind="agent loop", step=number, answer=next(iter(step)))
        else:
            step = {"text": "ok"}
            entry.update(kind="side request")
        log(entry)
        content, stop, extra = reply_parts(step)
        message = {"id": "msg_spike_" + uuid.uuid4().hex[:20], "type": "message", "role": "assistant",
                   "model": body.get("model"), "content": content, "stop_reason": stop, "stop_sequence": None,
                   "usage": {"input_tokens": 1, "output_tokens": 1}}
        message.update(extra.get("message", {}))
        if not body.get("stream"):
            return self.send_json(200, message)
        self.send_response(200)
        self.send_header("content-type", "text/event-stream")
        self.send_header("cache-control", "no-cache")
        self.end_headers()

        def event(name, data):
            self.wfile.write(("event: %s\ndata: %s\n\n" % (name, json.dumps(data))).encode())
            self.wfile.flush()

        event("message_start", {"type": "message_start", "message": dict(message, content=[], stop_reason=None)})
        for i, block in enumerate(content):
            if block["type"] == "tool_use":
                event("content_block_start", {"type": "content_block_start", "index": i, "content_block": dict(block, input={})})
                delta = {"type": "input_json_delta", "partial_json": json.dumps(block["input"])}
            else:
                event("content_block_start", {"type": "content_block_start", "index": i, "content_block": {"type": "text", "text": ""}})
                delta = {"type": "text_delta", "text": block["text"]}
            event("content_block_delta", {"type": "content_block_delta", "index": i, "delta": delta})
            event("content_block_stop", {"type": "content_block_stop", "index": i})
        stop_delta = {"stop_reason": stop, "stop_sequence": None}
        stop_delta.update(extra.get("delta", {}))
        event("message_delta", {"type": "message_delta", "delta": stop_delta, "usage": {"output_tokens": 1}})
        event("message_stop", {"type": "message_stop"})


if __name__ == "__main__":
    server = ThreadingHTTPServer(("127.0.0.1", PORT), Handler)
    print("fake model on 127.0.0.1:%d with %d steps" % (PORT, len(STEPS)), flush=True)
    server.serve_forever()
