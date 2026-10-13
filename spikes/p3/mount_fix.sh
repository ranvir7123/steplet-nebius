#!/bin/bash
# Mount the gate folder with the Windows path in backslash form; if that works, write the same into /etc/fstab
# (fstab writes a backslash as \134). Run as root.
SRC='C:\Users\RANVIR\AppData\Local\Steplet\gate'
echo "--- manual mount with backslashes"
mount -t drvfs "$SRC" /opt/steplet/gate -o ro,noatime 2>&1 | sed 's/^/    /'
echo "    exit code: ${PIPESTATUS[0]}"
findmnt -rn -o TARGET,SOURCE,FSTYPE,OPTIONS /opt/steplet/gate | sed 's/^/    now: /'
ls -la /opt/steplet/gate | sed 's/^/    /'
umount /opt/steplet/gate && echo "    unmounted again, to test the fstab line next"
echo "--- fstab line with \\134 for each backslash"
FSTAB_SRC=$(printf '%s' "$SRC" | sed 's/\\/\\134/g')
grep -v ' /opt/steplet/gate ' /etc/fstab > /etc/fstab.new || true
echo "$FSTAB_SRC /opt/steplet/gate drvfs ro,noatime 0 0" >> /etc/fstab.new
mv /etc/fstab.new /etc/fstab
grep steplet /etc/fstab | sed 's/^/    /'
systemctl daemon-reload
mount /opt/steplet/gate 2>&1 | sed 's/^/    /'
echo "    mount by the fstab line, exit code: ${PIPESTATUS[0]}"
findmnt -rn -o TARGET,SOURCE,FSTYPE,OPTIONS /opt/steplet/gate | sed 's/^/    now: /'
