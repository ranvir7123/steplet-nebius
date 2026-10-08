# Milestone 1, phase 0: append a section to docs/milestone-1-spikes.md.
# Wherever the template says {{LOG:<file>}}, C:\dev\m1\logs\<file> is pasted in a code block.
# Fake keys are replaced with labelled placeholders. If the real API key (or its last 20 characters)
# appears in a log, nothing is written and the script stops: that is a finding, not something to paste.
# The real key is read only to look for it; it is never printed.
# Usage: python -I append_results.py <template.md> <results.md>
import re
import sys

LOGS = "C:\\dev\\m1\\logs\\"
FAKES = [("C:\\dev\\m1\\canary\\wsl_fake_key", "M1-FAKE-KEY-WSL-<removed: the fixture's fake key>"),
         ("C:\\dev\\m1\\canary\\fake_api_key", "sk-ant-api03-M1FAKE-<removed: the fake API key>")]
REAL = "\\\\wsl.localhost\\steplet-test\\home\\tester\\.local\\share\\steplet\\secrets\\anthropic-api-key"

fakes = [(open(path, encoding="ascii").read().strip(), label) for path, label in FAKES]
try:
    real = open(REAL, encoding="ascii").read().strip()
except OSError:
    real = None


def paste(m):
    raw = open(LOGS + m.group(1), encoding="utf-8-sig", errors="replace").read().replace("\r\n", "\n").rstrip("\n")
    if real and (real in raw or real[-20:] in raw):
        sys.exit("STOP: the real API key text is in %s. Nothing was appended." % m.group(1))
    for text, label in fakes:
        raw = raw.replace(text, label).replace(text[-20:], label + " (last 20 characters)")
    raw = re.sub(r"[\x00-\x08\x0b-\x1f\x7f]", lambda c: "\\x%02x" % ord(c.group(0)), raw)
    return "```\n" + raw + "\n```"


text = re.sub(r"\{\{LOG:([^}]+)\}\}", paste, open(sys.argv[1], encoding="utf-8").read())
with open(sys.argv[2], "a", encoding="utf-8", newline="") as f:
    f.write("\n" + text.rstrip("\n") + "\n")
print("appended %d lines" % text.count("\n"))
