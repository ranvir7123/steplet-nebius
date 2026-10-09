#!/bin/bash
# S4, optional part: the clone itself inside srt, allowed to reach github.com only. Runs inside srt;
# keeps srt's proxy variables and srt's GIT_CONFIG_PARAMETERS (without it git fails srt's proxy login:
# "Proxy CONNECT aborted"), drops everything else. The -c options below still win over srt's.
W=$HOME/steplet-work/S4-fenced-clone; R=$W/repo
exec /usr/bin/env -i PATH=/usr/bin:/bin HOME="$W/empty-home" GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null \
  GIT_CONFIG_PARAMETERS="${GIT_CONFIG_PARAMETERS:-}" \
  GIT_TERMINAL_PROMPT=0 GIT_LFS_SKIP_SMUDGE=1 HTTPS_PROXY="${HTTPS_PROXY:-}" HTTP_PROXY="${HTTP_PROXY:-}" \
  https_proxy="${https_proxy:-}" http_proxy="${http_proxy:-}" ALL_PROXY="${ALL_PROXY:-}" \
  /usr/bin/git -c protocol.allow=never -c protocol.https.allow=always -c core.hooksPath=/dev/null -c core.fsmonitor=false \
  -c transfer.fsckObjects=true -c transfer.bundleURI=false -c credential.helper= \
  clone --no-checkout --no-recurse-submodules --template= https://github.com/octocat/Hello-World.git "$R"
