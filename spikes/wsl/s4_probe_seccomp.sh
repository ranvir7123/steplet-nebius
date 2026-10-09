#!/bin/bash
# Milestone 1, phase 0, S4 diagnostic (9 Oct 2026). Throwaway. Run inside steplet-test.
# Does denying Steplet's private folder hide srt's own seccomp helper from the fence?
P=$HOME/.local/share/steplet; T=$HOME/steplet-work/srt-probe
SRT=$P/srt/node_modules/.bin/srt
V=$P/srt/node_modules/@anthropic-ai/sandbox-runtime/vendor/seccomp
rm -rf "$T"; mkdir -p "$T"
echo "srt allowRead mentions in README: $(grep -c allowRead "$P/srt/node_modules/@anthropic-ai/sandbox-runtime/README.md")"
cat > "$T/1-deny-private.json" <<EOF
{ "filesystem": { "denyRead": ["/mnt", "$P", "$HOME/m1"], "allowWrite": ["$T"], "denyWrite": [] },
  "network": { "allowedDomains": [], "deniedDomains": [] } }
EOF
cat > "$T/2-private-not-denied.json" <<EOF
{ "filesystem": { "denyRead": ["/mnt", "$HOME/m1"], "allowWrite": ["$T"], "denyWrite": [] },
  "network": { "allowedDomains": [], "deniedDomains": [] } }
EOF
cat > "$T/3-deny-private-allow-helper.json" <<EOF
{ "filesystem": { "denyRead": ["/mnt", "$P", "$HOME/m1"], "allowRead": ["$V"], "allowWrite": ["$T"], "denyWrite": [] },
  "network": { "allowedDomains": [], "deniedDomains": [] } }
EOF
for s in 1-deny-private 2-private-not-denied 3-deny-private-allow-helper; do
  echo "=== $s"
  "$SRT" --settings "$T/$s.json" /bin/sh -c 'echo RAN_INSIDE; ls /home/tester/.local/share/steplet 2>&1 | head -3' 2>&1 | tail -4
  echo "exit: ${PIPESTATUS[0]}"
done
