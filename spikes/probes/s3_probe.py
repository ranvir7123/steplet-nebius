"""Spike probe, run inside the fence by a scripted command. Copies what this program can see into
s3_seen/ in the working copy, for spike.py evidence to search from outside. Prints no values."""
import os
import pathlib
import sys

out = pathlib.Path("s3_seen")
out.mkdir(exist_ok=True)
(out / "env.txt").write_text("\n".join(f"{k}={v}" for k, v in sorted(os.environ.items())))
copied = 0
for d in pathlib.Path("/proc").iterdir():
    if not d.name.isdigit():
        continue
    for part in ("environ", "cmdline"):
        try:
            (out / f"proc_{d.name}_{part}.txt").write_bytes((d / part).read_bytes())
            copied += 1
        except OSError:
            pass
lines = []
for i, target in enumerate(sys.argv[1:]):
    try:
        data = open(target, "rb").read()
        (out / f"read_{i}.bin").write_bytes(data)
        lines.append(f"{target}: READ {len(data)} bytes")
    except OSError as e:
        lines.append(f"{target}: refused ({type(e).__name__}: {e.strerror})")
(out / "reads.txt").write_text("\n".join(lines) + "\n")
print(f"probe: {len(os.environ)} environment variables, {copied} /proc files copied, {len(lines)} reads tried")
print("\n".join(lines))
