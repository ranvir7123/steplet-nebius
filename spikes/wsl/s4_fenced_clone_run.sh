#!/bin/bash
# Runs s4_fenced_clone.sh inside srt with network to github.com only. The script is copied into the folder
# the fence allows first, because the fence denies reading /mnt, where the repository's copy lives.
P=$HOME/.local/share/steplet; W=$HOME/steplet-work/S4-fenced-clone
# srt's own seccomp helper sits inside the denied private folder; re-allow just that folder (see s4_clone.sh).
HELPER=$P/srt/node_modules/@anthropic-ai/sandbox-runtime/vendor/seccomp
rm -rf "$W"; mkdir -p "$W/empty-home"
cp /mnt/c/dev/runFile/spikes/wsl/s4_fenced_clone.sh "$W/"
cat > "$W/srt-clone.json" <<EOF
{ "filesystem": { "denyRead": ["/mnt", "$P", "$HOME/m1"], "allowRead": ["$HELPER"], "allowWrite": ["$W"], "denyWrite": [] },
  "network": { "allowedDomains": ["github.com"], "deniedDomains": [] } }
EOF
"$P/srt/node_modules/.bin/srt" --settings "$W/srt-clone.json" -- /bin/bash "$W/s4_fenced_clone.sh" 2>&1 | tail -5
echo "clone exit code: ${PIPESTATUS[0]}"
echo "commit present: $(git -C "$W/repo" cat-file -t 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d 2>&1)"
