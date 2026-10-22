"""Test data for Steplet's runner tests (milestone 1, phase 5, Task 3). Inert: it only looks, and prints one JSON line.

Run in the working copy, /work/<run id>/repo. Writes down what a program the task runs sees at the run's own folder,
at the two notes the test put outside the working copy, and at the links in notes/: for a folder, the names in it; for
a file, only whether it holds a made-up marker word (never the file's text).
"""
import json
import os

RUN = os.getcwd().split("/")[2]
PLACES = [
    "/var/lib/steplet/runs/%s" % RUN,
    "/var/lib/steplet/runs/%s/home/run-home-note.txt" % RUN,
    "/tmp/steplet-run-%s/run-tmp-note.txt" % RUN,
    "notes/run-home.txt",
    "notes/run-tmp.txt",
    "notes/run-folder",
]

seen = {}
for place in PLACES:
    entry = {"exists": os.path.exists(place)}
    try:
        if os.path.isdir(place):
            entry["names"] = sorted(os.listdir(place))
        else:
            with open(place, encoding="utf-8", errors="replace") as f:
                entry["holds_a_marker_word"] = "steplet-marker-" in f.read(200)
    except OSError as e:
        entry["error"] = "%s: %s" % (type(e).__name__, e.strerror)
    seen[place] = entry
print(json.dumps(seen))
