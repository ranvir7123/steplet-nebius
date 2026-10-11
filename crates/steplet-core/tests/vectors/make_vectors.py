"""Makes phase 1's signature test files with Windows' own ssh-keygen, without Steplet's code.
Run once: python -I crates/steplet-core/tests/vectors/make_vectors.py
Keys are throwaway and stay in C:\\dev\\m1\\p1 (never committed); only their public halves are copied here."""
import pathlib
import shutil
import subprocess
import sys

KEYGEN = r"C:\Windows\System32\OpenSSH\ssh-keygen.exe"
KEYS = pathlib.Path(r"C:\dev\m1\p1")
OUT = pathlib.Path(__file__).resolve().parent
BASE = """---
steplet: 1
title: Count the lines of the README in Hello-World
sender: Steplet test vectors
expires: 2026-12-31
repository:
  url: https://github.com/octocat/Hello-World.git
  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
steps:
  - Count the lines in the README file.
  - Report the count.
allow:
  commands:
    - wc -l README
---
# Count the README lines

Run `wc -l README` and report the number.
"""


def run(*args, stdin=None):
    r = subprocess.run([KEYGEN, *args], input=stdin, capture_output=True)
    out = (r.stdout + r.stderr).decode(errors="replace").strip()
    print(f"$ ssh-keygen {' '.join(args)}\n  exit {r.returncode}: {out}")
    if r.returncode != 0:
        sys.exit("ssh-keygen failed")
    return out


def key(name, *kind):
    k = KEYS / name
    if not k.exists():
        run("-t", *kind, "-N", "", "-q", "-C", "steplet-p1-vectors", "-f", str(k))
    return k


def sign(text, k, *options):
    msg = KEYS / "msg.txt"
    msg.write_bytes(text.encode("utf-8"))
    sig = KEYS / "msg.txt.sig"
    sig.unlink(missing_ok=True)
    run("-Y", "sign", "-f", str(k), *options, str(msg))
    return sig.read_text(encoding="ascii")


def block(pem):
    lines = [l.strip() for l in pem.splitlines() if l.strip()]
    return "signature: |\n" + "".join("  " + l + "\n" for l in lines)


def embed(text, blk):
    head, _, body = text.partition("\n---\n")
    return head + "\n" + blk + "---\n" + body


KEYS.mkdir(parents=True, exist_ok=True)
ed = key("ed25519", "ed25519")
rsa = key("rsa", "rsa", "-b", "3072")
good = block(sign(BASE, ed, "-n", "steplet-v1", "-O", "hashalg=sha512"))
run("-Y", "check-novalidate", "-n", "steplet-v1", "-s", str(KEYS / "msg.txt.sig"), stdin=BASE.encode("utf-8"))
valid = embed(BASE, good)
files = {
    "unsigned.steplet": BASE,
    "valid.steplet": valid,
    "crlf.steplet": valid.replace("\n", "\r\n"),
    "tampered.steplet": valid.replace("Report the count.", "Report the count and push."),
    "wrong-namespace.steplet": embed(BASE, block(sign(BASE, ed, "-n", "runfile-v1"))),
    "sha256.steplet": embed(BASE, block(sign(BASE, ed, "-n", "steplet-v1", "-O", "hashalg=sha256"))),
    "rsa.steplet": embed(BASE, block(sign(BASE, rsa, "-n", "steplet-v1"))),
    "cut.steplet": valid.replace("  -----END SSH SIGNATURE-----\n", ""),
    "doubled.steplet": embed(BASE, good + good),
    "moved-in-header.steplet": BASE.replace("steps:\n", good + "steps:\n", 1),
    "moved-to-body.steplet": BASE + good,
}
for name, text in files.items():
    (OUT / name).write_bytes(text.encode("utf-8"))
shutil.copy(KEYS / "ed25519.pub", OUT / "ed25519.pub")
shutil.copy(KEYS / "rsa.pub", OUT / "rsa.pub")
fingerprint = run("-l", "-f", str(KEYS / "ed25519.pub")).split()[1]
(OUT / "fingerprint.txt").write_bytes(fingerprint.encode("ascii"))
print("fingerprint:", fingerprint)
