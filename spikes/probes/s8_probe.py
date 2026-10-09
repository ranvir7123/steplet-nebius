"""S8 probe: which addresses can this program connect to? One JSON line per attempt in s8_results.jsonl.
Inside Claude Code's fence a proxy is offered through environment variables; the probe asks it for the same places."""
import base64
import json
import os
import socket
import sys
import time
from urllib.parse import unquote, urlparse

host_ip = sys.argv[1]
TARGETS = [("listener inside WSL", "127.0.0.1", 47831), ("127.0.0.1:47832 (a Windows-only listener)", "127.0.0.1", 47832),
           ("Windows host RPC 135", host_ip, 135), ("Windows host 5040", host_ip, 5040), ("Windows host 49664", host_ip, 49664),
           ("Windows host 47832", host_ip, 47832), ("internet 1.1.1.1:443", "1.1.1.1", 443)]
results = []
for name, ip, port in TARGETS:
    t = time.time()
    try:
        with socket.create_connection((ip, port), timeout=5) as s:
            s.sendall(b"S8-PROBE")
        results.append({"target": name, "ok": True, "detail": "connected", "seconds": round(time.time() - t, 2)})
    except OSError as e:
        results.append({"target": name, "ok": False, "detail": f"{type(e).__name__}: {e}", "seconds": round(time.time() - t, 2)})
proxy = next((os.environ[v] for v in ("HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy") if os.environ.get(v)), None)
if proxy:
    p = urlparse(proxy)
    # The fence's proxy wants the login carried in its address (user and password); without it every CONNECT
    # gets "407 Proxy Authentication Required", which says nothing about the target.
    auth = ""
    if p.username:
        token = base64.b64encode(f"{unquote(p.username)}:{unquote(p.password or '')}".encode()).decode()
        auth = f"Proxy-Authorization: Basic {token}\r\n"
    results.append({"target": "proxy login sent", "ok": None, "detail": "yes" if auth else "no login in the proxy address"})
    for name, ip, port in TARGETS:
        t = time.time()
        try:
            with socket.create_connection((p.hostname, p.port), timeout=5) as s:
                s.sendall(f"CONNECT {ip}:{port} HTTP/1.1\r\nHost: {ip}:{port}\r\n{auth}\r\n".encode())
                reply = s.recv(200).decode("latin-1").splitlines()
            first = reply[0] if reply else "(empty reply)"
            results.append({"target": "through the proxy: " + name, "ok": " 200" in first, "detail": first, "seconds": round(time.time() - t, 2)})
        except OSError as e:
            results.append({"target": "through the proxy: " + name, "ok": False, "detail": f"{type(e).__name__}: {e}", "seconds": round(time.time() - t, 2)})
else:
    results.append({"target": "proxy", "ok": None, "detail": "no proxy variable in the environment"})
with open("s8_results.jsonl", "w") as f:
    for r in results:
        f.write(json.dumps(r) + "\n")
for r in results:
    print(r["target"], "->", "CONNECTED" if r["ok"] else r["detail"])  # never prints the login itself
