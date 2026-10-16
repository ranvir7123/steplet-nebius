#!/bin/bash
# Phase 3, Task 1, Check 1: one run's folders in the section 3 layout, from files already copied into
# /opt/steplet-test (root). Throwaway probe, like phase 0's spike.py prepare. Run as root.
set -eu
ID=p1; W=/work/$ID; R=/var/lib/steplet/runs/$ID; T=/opt/steplet-test
pkill -KILL -u steplet 2>/dev/null || true
rm -rf /work/* /var/lib/steplet/runs/*
install -d -o root -g root -m 755 "$W" "$R"
install -d -o steplet -g steplet -m 755 "$W/repo"
install -d -o steplet -g steplet -m 700 "$R/home" "$R/config" "$R/tmp"
tar -xf "$T/fixture.tar" -C "$W/repo"
chown -R steplet:steplet "$W/repo"
chmod -R go-w "$W/repo"   # Windows tar writes every file as world-writable; a real clone inside Linux does not
# H7 reverted after run 1: with .git owned by root, the sandbox could not create its lock placeholder inside .git
# and every sandboxed command failed. .git stays with steplet; denyWrite and the Edit deny rule guard it (phase 0 S4).
install -o root -g root -m 644 "$T/p1.settings.json" "$R/settings.json"
install -o root -g root -m 644 "$T/p1.mcp.json" "$R/mcp.json"
echo "--- prepared: HEAD $(runuser -u steplet -- git -C "$W/repo" rev-parse HEAD)"
ls -la "$W" "$W/repo" "$R" | sed 's/^/    /'
echo "--- /work holds: $(ls -A /work | tr '\n' ' ')"
