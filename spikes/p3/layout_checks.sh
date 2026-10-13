#!/bin/bash
# Phase 3, Task 1, Checks 3 and 4, and the interop half of Check 1. Read-only; run as the steplet account.
echo "--- who: $(id)"
echo "--- /etc/wsl.conf in effect: automount $(grep -A1 '^\[automount\]' /etc/wsl.conf | tail -1)"
echo "--- /mnt holds: $(ls -A /mnt 2>&1 | tr '\n' ' ')"
echo "--- /mnt/c: $(test -e /mnt/c && echo PRESENT || echo absent)"
echo "--- Windows-backed mounts (9p, drvfs, virtiofs):"
findmnt -rn -o TARGET,FSTYPE,OPTIONS | grep -E ' (9p|drvfs|virtiofs) ' | sed 's/^/    /'
echo "--- gate folder:"
ls -la /opt/steplet/gate | sed 's/^/    /'
echo "--- write into the gate folder: $(touch /opt/steplet/gate/probe 2>&1 && echo WRITTEN || true)"
echo "--- /home holds: [$(ls -A /home 2>&1 | tr '\n' ' ')]"
echo "--- /root readable: $(ls /root > /dev/null 2>&1 && echo yes || echo no)"
echo "--- /mnt/wsl:"
ls -la /mnt/wsl 2>&1 | sed 's/^/    /'
echo "--- /mnt/wslg (first 12 lines):"
ls -la /mnt/wslg 2>&1 | head -12 | sed 's/^/    /'
echo "--- /mnt/wslg/distro readable: $(ls /mnt/wslg/distro > /dev/null 2>&1 && echo yes || echo no)"
echo "--- a Windows program by the usual drive path: $(test -e /mnt/c/Windows/System32/cmd.exe && echo reachable || echo 'not reachable')"
echo "--- interop: start the stand-in gate with no arguments (it should start on Windows and stop on the missing --pipe)"
timeout 20 /opt/steplet/gate/spike-gate.exe < /dev/null 2>&1 | head -3 | sed 's/^/    /'
echo "    exit code: ${PIPESTATUS[0]}"
echo "--- bash outside the fence: $BASH_VERSION"
