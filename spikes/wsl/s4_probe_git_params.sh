#!/bin/bash
# Milestone 1, phase 0, S4 diagnostic (9 Oct 2026). Throwaway. Run inside steplet-test.
# Is srt's GIT_CONFIG_PARAMETERS what the stripped clone is missing?
P=$HOME/.local/share/steplet; W=$HOME/steplet-work/S4-fenced-clone
S=$W/srt-clone.json
SRT=$P/srt/node_modules/.bin/srt
cat > "$W/inner3.sh" <<'EOF'
#!/bin/bash
W=$HOME/steplet-work/S4-fenced-clone
echo "git settings named in GIT_CONFIG_PARAMETERS: $(printf '%s' "${GIT_CONFIG_PARAMETERS:-}" | grep -oE "'[a-zA-Z0-9_.-]+'=" | tr -d "'=" | tr '\n' ' ')"
KEEP=(PATH=/usr/bin:/bin HOME="$W/empty-home" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0
      HTTPS_PROXY="${HTTPS_PROXY:-}" HTTP_PROXY="${HTTP_PROXY:-}" https_proxy="${https_proxy:-}" http_proxy="${http_proxy:-}"
      ALL_PROXY="${ALL_PROXY:-}")
rm -rf "$W/repo5"
/usr/bin/env -i "${KEEP[@]}" GIT_CONFIG_PARAMETERS="${GIT_CONFIG_PARAMETERS:-}" /usr/bin/git clone --no-checkout --quiet https://github.com/octocat/Hello-World.git "$W/repo5" 2>&1 | tail -1
echo "=== 5. stripped environment plus GIT_CONFIG_PARAMETERS: exit ${PIPESTATUS[0]}"
EOF
cp "$W/inner3.sh" "$W/inner3-run.sh"
"$SRT" --settings "$S" -- /bin/bash "$W/inner3-run.sh" 2>&1
