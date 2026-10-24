"""Test data for Steplet's runner tests (milestone 1, phase 5, Task 5). Inert: it only writes agent settings files and
says which writes worked.

Run in the working copy, /work/<run id>/repo. Writes the two agent settings files a repository can carry,
.claude/settings.json and .claude/settings.local.json, and the settings file in the run's own config folder, the one
Claude Code reads for its user. Each holds one hook: before each tool call, it would write a file in the working copy
named after its settings file, holding a made-up word. The word is put together only when the hook runs, so no
settings file holds it whole. Then it waits 5 seconds before it ends: Claude Code loads a settings file that changes
during a run a few seconds later, so the next call comes after it would have.
"""
import json
import os
import time

REPO = os.getcwd()
RUN = REPO.split("/")[2]
FILES = {
    "project": ".claude/settings.json",
    "local": ".claude/settings.local.json",
    "user": "/var/lib/steplet/runs/%s/config/settings.json" % RUN,
}

try:
    os.makedirs(".claude", exist_ok=True)
except OSError as e:
    print("could not make .claude: %s" % e.strerror)
for name, path in FILES.items():
    command = "printf '%%s-%%s\n' steplet-marker hook-ran > %s/hook-ran-%s.txt" % (REPO, name)
    hook = {"hooks": {"PreToolUse": [{"matcher": "*", "hooks": [{"type": "command", "command": command}]}]}}
    try:
        with open(path, "w") as f:
            json.dump(hook, f)
        print("wrote %s" % path)
    except OSError as e:
        print("could not write %s: %s" % (path, e.strerror))
time.sleep(5)
