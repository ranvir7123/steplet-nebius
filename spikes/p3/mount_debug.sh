#!/bin/bash
# Why did the fstab line for the gate folder not mount? Read-only probes, then one manual mount attempt. Run as root.
echo "--- /mnt/c is an empty folder: [$(ls -A /mnt/c 2>&1 | tr '\n' ' ')]"
echo "--- fstab as findmnt reads it:"
findmnt --fstab -rn | sed 's/^/    /'
echo "--- mount helper: $(ls -l /usr/sbin/mount.drvfs /sbin/mount.drvfs 2>&1 | tr '\n' ';')"
echo "--- systemd's unit for that mount:"
systemctl status opt-steplet-gate.mount --no-pager 2>&1 | head -12 | sed 's/^/    /'
echo "--- journal lines about it:"
journalctl -b --no-pager 2>/dev/null | grep -i -E 'steplet/gate|drvfs' | tail -8 | sed 's/^/    /'
echo "--- manual: mount /opt/steplet/gate"
mount /opt/steplet/gate 2>&1 | sed 's/^/    /'
echo "    exit code: ${PIPESTATUS[0]}"
findmnt -rn /opt/steplet/gate | sed 's/^/    now: /'
ls -la /opt/steplet/gate | sed 's/^/    /'
