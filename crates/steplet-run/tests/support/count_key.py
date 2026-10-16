"""Count where a key's text appears, without printing it. Test support only (phase 0's S3 method).

Usage: python3 -I count_key.py <file or folder>...      (the key arrives on standard input)
       python3 -I count_key.py --whole-disk            (every file on the root file system, inside Linux)

Prints, for the whole key and for its last 20 characters, the total count and the files it was found in.
The key's text is never printed.
"""
import os
import sys

SKIP = ("/proc", "/sys", "/dev", "/run", "/mnt")


def files_under(paths):
    for p in paths:
        if os.path.isfile(p):
            yield p
        for root, dirs, names in os.walk(p):
            if root in SKIP:
                dirs[:] = []
                continue
            for n in names:
                full = os.path.join(root, n)
                if os.path.isfile(full) and not os.path.islink(full):
                    yield full


def main():
    key = sys.stdin.buffer.read().strip()
    if len(key) < 20:
        print("no key on standard input")
        return 2
    paths = ["/"] if sys.argv[1:] == ["--whole-disk"] else sys.argv[1:]
    root_dev = os.stat("/").st_dev if paths == ["/"] else None
    for label, needle in (("key", key), ("key, last 20 characters", key[-20:])):
        hits = {}
        scanned = 0
        for f in files_under(paths):
            try:
                if root_dev is not None and os.stat(f).st_dev != root_dev:
                    continue
                data = open(f, "rb").read()
            except OSError:
                continue
            scanned += 1
            n = data.count(needle)
            if n:
                hits[f] = n
        print("%s: %d %s (files read: %d)" % (label, sum(hits.values()), hits or "", scanned))
    return 0


if __name__ == "__main__":
    sys.exit(main())
