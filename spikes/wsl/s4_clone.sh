#!/bin/bash
# S4, part 1: Steplet's hardened clone (no fence, no checkout), then the checkout alone inside srt, with no agent.
set -u
P=$HOME/.local/share/steplet; W=$HOME/steplet-work/S4; R=$W/repo
URL=https://github.com/octocat/Hello-World.git
COMMIT=7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
SRT=$P/srt/node_modules/.bin/srt
# Every srt call puts "--" before the command: without it srt takes a "-c" meant for the command as its own
# option and runs the next argument as a raw shell line.
# srt lives inside the private folder the fence denies, so its own seccomp helper must be re-allowed,
# or every fenced command fails with "apply-seccomp: No such file or directory" (exit 127).
HELPER=$P/srt/node_modules/@anthropic-ai/sandbox-runtime/vendor/seccomp
rm -rf "$W"; mkdir -p "$W/empty-home"
GITENV=(/usr/bin/env -i PATH=/usr/bin:/bin HOME="$W/empty-home" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0 GIT_LFS_SKIP_SMUDGE=1)
HARD=(-c protocol.allow=never -c protocol.https.allow=always -c core.hooksPath=/dev/null -c core.fsmonitor=false
      -c transfer.fsckObjects=true -c transfer.bundleURI=false -c credential.helper=)
echo "git: $(/usr/bin/git --version)"
echo "--- 1. hardened clone, no checkout, no fence"
"${GITENV[@]}" /usr/bin/git "${HARD[@]}" clone --no-checkout --no-recurse-submodules --template= "$URL" "$R" 2>&1
echo "clone exit code: $?"
echo "commit present: $("${GITENV[@]}" /usr/bin/git -C "$R" cat-file -t "$COMMIT" 2>&1)"
echo ".git/hooks after clone: $(ls -A "$R/.git/hooks" 2>&1 | tr '\n' ' ')"
echo "files outside .git before checkout: $(ls -A "$R" | grep -v '^.git$' | tr '\n' ' ')"
cat > "$W/srt-checkout.json" <<EOF
{ "filesystem": { "denyRead": ["/mnt", "$P", "$HOME/m1"], "allowRead": ["$HELPER"], "allowWrite": ["$R"], "denyWrite": [] },
  "network": { "allowedDomains": [], "deniedDomains": [] } }
EOF
echo "--- 2. control: the srt fence holds (write outside, read the private folder, internet)"
"$SRT" --settings "$W/srt-checkout.json" -- /bin/sh -c "echo x > $W/outside_marker" 2>&1 | tail -2
echo "outside marker: $(test -e "$W/outside_marker" && echo EXISTS || echo absent)"
"$SRT" --settings "$W/srt-checkout.json" -- /bin/ls "$P" 2>&1 | tail -2
"$SRT" --settings "$W/srt-checkout.json" -- /usr/bin/python3 -c "import socket; socket.create_connection(('1.1.1.1', 443), timeout=5); print('CONNECTED')" 2>&1 | tail -1
echo "--- 3. checkout inside srt, no agent"
"$SRT" --settings "$W/srt-checkout.json" -- "${GITENV[@]}" /usr/bin/git -C "$R" -c core.hooksPath=/dev/null -c core.fsmonitor=false checkout --quiet --detach "$COMMIT" 2>&1
echo "checkout exit code: $?"
echo "HEAD: $(git -C "$R" rev-parse HEAD)"
echo "status: $(git -C "$R" status --short | head -5)"
echo "files: $(ls -A "$R" | tr '\n' ' ')"
