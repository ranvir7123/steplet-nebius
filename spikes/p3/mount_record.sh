#!/bin/bash
# Record why the drvfs mount failed, then take the fstab line out (the plan's fallback needs no Windows mount). Root.
echo "--- kernel messages from the failed mounts (last 6):"
dmesg 2>&1 | grep -i -E '9p|drvfs|plan9|mount' | tail -6 | sed 's/^/    /'
echo "--- does drvfs work at all here? mount the drive root C: read-only at a temporary place, then unmount"
mkdir -p /tmp/drvfs-probe
mount -t drvfs 'C:' /tmp/drvfs-probe -o ro 2>&1 | tail -2 | sed 's/^/    /'
echo "    exit code: ${PIPESTATUS[0]}; mounted: $(findmnt -rn -o FSTYPE /tmp/drvfs-probe || echo no)"
umount /tmp/drvfs-probe 2>/dev/null; rmdir /tmp/drvfs-probe
echo "--- removing the fstab line"
grep -v ' /opt/steplet/gate ' /etc/fstab > /etc/fstab.new || true
mv /etc/fstab.new /etc/fstab
echo "    lines left that mention steplet: $(grep -c steplet /etc/fstab)"
systemctl daemon-reload
