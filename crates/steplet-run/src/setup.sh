#!/bin/bash
# Steplet's own Linux (decision E3, option D): first-time setup. Runs as root inside Steplet's Linux, right after
# `wsl --install`, sent on standard input by setup.rs (phase 4 Task 6; phase 3 ran it as a test script). Safe to run
# again: every step can be repeated. Arguments: the Claude Code version and the srt version to install.
# Nothing here asks for a password: WSL starts it as root inside Steplet's own Linux only.
# The gate program is copied in afterwards from Windows, on standard input, because WSL 2.6.3 cannot mount a single
# Windows folder (only whole drives), and mounting a drive would undo option D.
set -eu

main() {
CLAUDE_VERSION=$1
SRT_VERSION=$2
echo "--- /etc/wsl.conf: Windows drives not mounted, Windows PATH not added, default user steplet"
cat > /etc/wsl.conf <<'EOF'
[boot]
systemd=true

[automount]
enabled=false
mountFsTab=true

[interop]
enabled=true
appendWindowsPath=false

[user]
default=steplet
EOF
cat /etc/wsl.conf

echo "--- /opt/steplet/gate: the gate program's folder on the Linux disk, owned by root"
mkdir -p /opt/steplet/gate
grep -v ' /opt/steplet/gate ' /etc/fstab > /etc/fstab.new || true
mv /etc/fstab.new /etc/fstab

echo "--- the steplet account: no password, no sudo, no login shell"
id steplet > /dev/null 2>&1 || useradd --system --user-group --shell /usr/sbin/nologin --home-dir /nonexistent --no-create-home steplet
passwd -l steplet
id steplet

echo "--- base tools (choice H4-A; ripgrep because srt needs it, approved 14 Oct 2026)"
export DEBIAN_FRONTEND=noninteractive
apt-get update -q > /var/log/steplet-apt-update.txt 2>&1
apt-get install -y -q --no-install-recommends git python3 python3-venv build-essential ca-certificates \
    bubblewrap socat ripgrep nodejs npm > /var/log/steplet-apt-install.txt 2>&1
tail -2 /var/log/steplet-apt-install.txt
for p in git python3 build-essential bubblewrap socat ripgrep nodejs npm; do
  echo "$p: $(dpkg-query -W -f='${Version}' "$p")"
done

echo "--- Claude Code $CLAUDE_VERSION into /opt/steplet/claude-cli"
npm install -g --prefix /opt/steplet/claude-cli --no-fund --no-audit "@anthropic-ai/claude-code@$CLAUDE_VERSION" \
    > /var/log/steplet-npm-claude.txt 2>&1
tail -3 /var/log/steplet-npm-claude.txt
test -x /opt/steplet/claude-cli/bin/claude

echo "--- srt $SRT_VERSION into /opt/steplet/srt"
mkdir -p /opt/steplet/srt
cd /opt/steplet/srt
npm init -y > /dev/null
npm install --save-exact --no-fund --no-audit "@anthropic-ai/sandbox-runtime@$SRT_VERSION" > /var/log/steplet-npm-srt.txt 2>&1
tail -3 /var/log/steplet-npm-srt.txt
test -x /opt/steplet/srt/node_modules/.bin/srt

echo "--- Steplet's folders, owned by root; nothing under /opt/steplet writable by steplet"
mkdir -p /var/lib/steplet/runs /work
chown -R root:root /opt/steplet/claude-cli /opt/steplet/srt /var/lib/steplet /work
chmod -R go-w /opt/steplet/claude-cli /opt/steplet/srt
chmod 755 /opt/steplet /var/lib/steplet /var/lib/steplet/runs /work

echo "--- what was installed"
CLAUDE_FILE=$(readlink -f /opt/steplet/claude-cli/bin/claude)
echo "claude runs: $CLAUDE_FILE"
echo "claude sha256: $(sha256sum "$CLAUDE_FILE" | cut -d' ' -f1)"
echo "claude version: $(env -i HOME=/tmp PATH=/usr/bin:/bin DISABLE_AUTOUPDATER=1 /opt/steplet/claude-cli/bin/claude --version)"
echo "srt package: $(node -p "require('/opt/steplet/srt/node_modules/@anthropic-ai/sandbox-runtime/package.json').version")"
ls -ld /opt/steplet /opt/steplet/* /var/lib/steplet /var/lib/steplet/runs /work
echo "--- setup finished"
}

# The script comes on standard input, and bash reads it as it goes. All the work is in one function, which bash reads
# whole before it runs it, and every program it starts gets /dev/null as its input, so none can read the script.
main "$@" < /dev/null
