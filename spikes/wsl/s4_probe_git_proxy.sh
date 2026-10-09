#!/bin/bash
# Milestone 1, phase 0, S4 diagnostic (9 Oct 2026). Throwaway. Run inside steplet-test.
# Variant 2 (stripped environment, no hardening options) and variant 3 (the script's exact command), traced.
P=$HOME/.local/share/steplet; W=$HOME/steplet-work/S4-fenced-clone
S=$W/srt-clone.json
SRT=$P/srt/node_modules/.bin/srt
cat > "$W/inner2.sh" <<'EOF'
#!/bin/bash
W=$HOME/steplet-work/S4-fenced-clone
KEEP=(PATH=/usr/bin:/bin HOME="$W/empty-home" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0
      HTTPS_PROXY="${HTTPS_PROXY:-}" HTTP_PROXY="${HTTP_PROXY:-}" https_proxy="${https_proxy:-}" http_proxy="${http_proxy:-}"
      ALL_PROXY="${ALL_PROXY:-}" GIT_TRACE_CURL=1 GIT_TRACE_CURL_NO_DATA=1)
F="Recv header: HTTP|Proxy auth|CONNECT phase|abort|fatal|Connected to github|SSL connection"
echo "=== 2. stripped environment, no hardening options"
rm -rf "$W/repo2"
/usr/bin/env -i "${KEEP[@]}" /usr/bin/git clone --no-checkout --quiet https://github.com/octocat/Hello-World.git "$W/repo2" > "$W/t2.txt" 2>&1
echo "exit: $?"; grep -E "$F" "$W/t2.txt" | sed -E 's/^[0-9:.]+ +http.c:[0-9]+ +//' | head -8
echo "=== 3. stripped environment plus the script's hardening options (the script's exact git command)"
rm -rf "$W/repo3"
/usr/bin/env -i "${KEEP[@]}" GIT_LFS_SKIP_SMUDGE=1 /usr/bin/git -c protocol.allow=never -c protocol.https.allow=always \
  -c core.hooksPath=/dev/null -c core.fsmonitor=false -c transfer.fsckObjects=true -c transfer.bundleURI=false \
  -c credential.helper= clone --no-checkout --no-recurse-submodules --template= https://github.com/octocat/Hello-World.git "$W/repo3" > "$W/t3.txt" 2>&1
echo "exit: $?"; grep -E "$F" "$W/t3.txt" | sed -E 's/^[0-9:.]+ +http.c:[0-9]+ +//' | head -8
for opt in "protocol.allow=never -c protocol.https.allow=always" "credential.helper=" "transfer.bundleURI=false" "transfer.fsckObjects=true"; do
  rm -rf "$W/repo4"
  /usr/bin/env -i "${KEEP[@]}" /usr/bin/git -c $opt clone --no-checkout --quiet https://github.com/octocat/Hello-World.git "$W/repo4" > /dev/null 2>&1
  echo "=== 4. only -c $opt: exit $?"
done
EOF
cp "$W/inner2.sh" "$W/inner2-run.sh"
"$SRT" --settings "$S" -- /bin/bash "$W/inner2-run.sh" 2>&1
