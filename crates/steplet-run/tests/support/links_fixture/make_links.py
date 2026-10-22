"""Test data for Steplet's runner tests (milestone 1, phase 5, Task 3). Inert: it only makes links; it reads and
writes nothing else.

Run in the working copy, /work/<run id>/repo. Makes, in notes/, a link to each place the test checks: a note in the
run's own folder, a note in the run's temporary folder, the public file /etc/hostname, a Windows path that does not
exist in Steplet's Linux, the run's own folder, and the run's temporary folder. The notes are harmless files holding
made-up words, which the test puts there itself.
"""
import os

RUN = os.getcwd().split("/")[2]
LINKS = {
    "run-home.txt": "/var/lib/steplet/runs/%s/home/run-home-note.txt" % RUN,
    "run-tmp.txt": "/tmp/steplet-run-%s/run-tmp-note.txt" % RUN,
    "hostname.txt": "/etc/hostname",
    "windows.txt": "/mnt/c/Windows/win.ini",
    "run-folder": "/var/lib/steplet/runs/%s" % RUN,
    "tmp-folder": "/tmp/steplet-run-%s" % RUN,
}

for name, target in LINKS.items():
    os.symlink(target, os.path.join("notes", name))
print("made %d links in notes/" % len(LINKS))
