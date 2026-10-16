#!/bin/bash
# Phase 3 probes: one run's folders in the section 3 layout, the milestone 0 fixture, its config file, and a made-up
# canary secret in the run's own home (a folder the fence denies). Throwaway, like phase 0's spike.py prepare. Root.
# Usage: prepare.sh <run id>    (reads /opt/steplet-test/fixture.tar, <id>.settings.json, <id>.mcp.json)
set -eu
ID=$1; W=/work/$ID; R=/var/lib/steplet/runs/$ID; T=/opt/steplet-test
pkill -KILL -u steplet 2>/dev/null || true
rm -rf /work/* /var/lib/steplet/runs/*
install -d -o root -g root -m 755 "$W" "$R"
install -d -o steplet -g steplet -m 755 "$W/repo"
install -d -o steplet -g steplet -m 700 "$R/home" "$R/config" "$R/tmp"
tar -xf "$T/fixture.tar" -C "$W/repo" 2>/dev/null
install -o root -g root -m 644 "$T/$ID.settings.json" "$R/settings.json"
install -o root -g root -m 644 "$T/$ID.mcp.json" "$R/mcp.json"
printf 'P3-CANARY-%s' "$(tr -d - < /proc/sys/kernel/random/uuid)" > "$R/home/canary_key"
SUM=$(sha256sum "$R/home/canary_key" | cut -d' ' -f1)
cat > "$W/repo/tests/m0_config.json" <<EOF
{"canary_path": "$R/home/canary_key", "canary_sha256": "$SUM",
 "outside_path": "$R/tmp/written_by_probe.txt", "system_file": "/mnt/c/Windows/win.ini",
 "listener_port": 47831, "external_host": "1.1.1.1", "external_port": 443,
 "wsl_windows_marker": "C:\\\\dev\\\\m1\\\\p3\\\\outside\\\\from_wsl.txt", "noisy": false}
EOF
chown -R steplet:steplet "$W/repo" "$R/home/canary_key"
chmod -R go-w "$W/repo"
chmod 600 "$R/home/canary_key"
echo "--- prepared $ID: HEAD $(runuser -u steplet -- git -C "$W/repo" rev-parse HEAD); /work holds: $(ls -A /work | tr '\n' ' ')"
echo "--- canary: $(stat -c '%U %a %s bytes' "$R/home/canary_key")"
