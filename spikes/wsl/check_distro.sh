#!/bin/bash
# Milestone 1, phase 0, task 3: read-only checks of the test distribution. Changes nothing.
echo "distribution: $WSL_DISTRO_NAME, $(lsb_release -ds), kernel $(uname -r), user $(id -un)"
for p in nodejs npm socat bubblewrap ripgrep git python3; do
  echo "$p: $(dpkg-query -W -f='${Version}' "$p" 2>/dev/null || echo MISSING)"
done
echo "node: $(node --version 2>&1)"
echo "bwrap smoke test: $(bwrap --ro-bind / / --unshare-net --dev /dev --proc /proc true 2>&1 && echo BWRAP_OK)"
echo "AppArmor limit on user namespaces: $(cat /proc/sys/kernel/apparmor_restrict_unprivileged_userns 2>/dev/null || echo 'setting not present')"
echo "networking mode: $(wslinfo --networking-mode 2>&1)"
echo "interop sockets in /run/WSL: $(ls /run/WSL/ 2>&1 | tr '\n' ' ')"
echo "WSL_INTEROP in this session: ${WSL_INTEROP:-unset}"
echo "a Windows program from here: $(/mnt/c/Windows/System32/cmd.exe /c ver < /dev/null 2>&1 | tr -d '\r' | grep -m1 -i version)"
echo "claude on PATH: $(command -v claude || echo none)"
echo "~/.claude: $(test -e ~/.claude && echo exists || echo absent); ~/.config/git/ignore: $(test -e ~/.config/git/ignore && echo exists || echo absent)"
