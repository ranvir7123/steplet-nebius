#!/bin/bash
# Milestone 1, phase 0, task 4: Steplet's private folder, pinned Claude Code and srt, spike data. No sudo.
# No "pipefail": a shortened listing (| head) must not stop the script. Each install is checked by a test -x instead.
set -eu
P=$HOME/.local/share/steplet; M=$HOME/m1; S=/mnt/c/dev/runFile/spikes
mkdir -p "$P"/{claude-cli,claude-config,home,runs,secrets,srt} "$M"/{canary,outside,logs,tools/gate,npm-cache} "$HOME/steplet-work"
chmod 700 "$P" "$P/secrets"

echo "--- Claude Code 2.1.220 into $P/claude-cli"
npm install -g --prefix "$P/claude-cli" --cache "$M/npm-cache" --no-fund --no-audit @anthropic-ai/claude-code@2.1.220 > "$M/logs/npm-claude.txt" 2>&1 || true
tail -5 "$M/logs/npm-claude.txt"
test -x "$P/claude-cli/bin/claude"
echo "version: $(env -i HOME="$P/home" PATH=/usr/bin:/bin CLAUDE_CONFIG_DIR="$P/claude-config" DISABLE_AUTOUPDATER=1 "$P/claude-cli/bin/claude" --version)"
echo "the file that runs: $(readlink -f "$P/claude-cli/bin/claude")"
head -c 4 "$(readlink -f "$P/claude-cli/bin/claude")" | od -c | head -1

echo "--- srt 0.0.77 into $P/srt"
cd "$P/srt"
npm init -y > /dev/null
npm install --save-exact --cache "$M/npm-cache" --no-fund --no-audit @anthropic-ai/sandbox-runtime@0.0.77 > "$M/logs/npm-srt.txt" 2>&1 || true
tail -3 "$M/logs/npm-srt.txt"
test -x "$P/srt/node_modules/.bin/srt"
npm ls --all 2>&1 | head -30 || true
"$P/srt/node_modules/.bin/srt" --help 2>&1 | head -20 || true

echo "--- fixture (milestone 0's test repository)"
rm -rf "$M/fixture"
git clone -q /mnt/c/dev/m0/repo "$M/fixture"
echo "fixture HEAD: $(git -C "$M/fixture" rev-parse HEAD); COMMIT1 present: $(git -C "$M/fixture" cat-file -t 65930082d222089b2633d34e9bb2bd42351bdc76)"

echo "--- fake keys (random, made for these tests, never real)"
printf '%s' "M1-FAKE-KEY-WSL-$(tr -d - < /proc/sys/kernel/random/uuid)" > "$M/canary/fake_key"
printf '%s' "sk-ant-api03-M1FAKE-$(tr -d - < /proc/sys/kernel/random/uuid)" > "$P/secrets/fake-api-key"
chmod 600 "$P/secrets/fake-api-key"
cp "$M/canary/fake_key" /mnt/c/dev/m1/canary/wsl_fake_key
cp "$P/secrets/fake-api-key" /mnt/c/dev/m1/canary/fake_api_key
echo "fixture fake key: $(stat -c %s "$M/canary/fake_key") bytes; fake API key: $(stat -c %s "$P/secrets/fake-api-key") bytes"

echo "--- Node stand-in gate"
cp "$S/node-gate/gate.mjs" "$S/node-gate/package.json" "$M/tools/gate/"
cd "$M/tools/gate"
npm install --cache "$M/npm-cache" --no-fund --no-audit 2>&1 | tail -2
node -e "import('@modelcontextprotocol/sdk/server/mcp.js').then(() => console.log('gate packages load: yes'))"

echo "--- folders"
ls -la "$P"
echo "~/.claude: $(test -e ~/.claude && echo exists || echo absent); ~/.claude.json: $(test -e ~/.claude.json && echo exists || echo absent)"
