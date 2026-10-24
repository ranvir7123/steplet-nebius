"""A listener on the Windows side, for the attack suite's case N3 (phase 5 Task 4): it listens on one port on each
address it is given (127.0.0.1, and the address WSL uses for Windows), and writes one line for every connection that
reaches it, with the first bytes the connection sent. It only logs; it answers nothing. A program the task runs must
never reach it. Test support only, run with Windows' own Python. It ends when its standard input closes, so it never
outlives the test that started it, even one that was killed.

Usage: python -I windows_listener.py <port> <log> <address>...
"""
import datetime
import json
import os
import platform
import selectors
import socket
import sys
import threading


def end_with_the_test():
    sys.stdin.buffer.read()
    os._exit(0)


threading.Thread(target=end_with_the_test, daemon=True).start()
port, log, addresses = int(sys.argv[1]), sys.argv[2], sys.argv[3:]
waiting = selectors.DefaultSelector()
for address in addresses:
    s = socket.socket()
    s.bind((address, port))
    s.listen()
    s.setblocking(False)
    waiting.register(s, selectors.EVENT_READ, address)
print("listening on %s, with Python %s" % (", ".join("%s:%d" % (a, port) for a in addresses), platform.python_version()),
      flush=True)
while True:
    for key, _ in waiting.select():
        c, peer = key.fileobj.accept()
        c.settimeout(2)
        try:
            first = c.recv(64)
        except OSError:
            first = b""
        c.close()
        line = {"at": datetime.datetime.now().isoformat(), "on": key.data, "from": peer[0], "first": first.decode("latin-1")}
        with open(log, "a", encoding="utf-8") as f:
            f.write(json.dumps(line) + "\n")
