import datetime, socket, sys
port, log = int(sys.argv[1]), sys.argv[2]
s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", port))
s.listen()
print("listening on 127.0.0.1:%d" % port, flush=True)
while True:
    c, addr = s.accept()
    data = c.recv(4096)
    c.close()
    with open(log, "a", encoding="utf-8") as f:
        f.write("%s from=%s bytes=%d first8=%r\n" % (datetime.datetime.now().isoformat(), addr, len(data), data[:8]))
