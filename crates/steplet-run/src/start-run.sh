#!/bin/bash
# Starts the agent for one prepared run (plan Task 7: section 3's launch, with choice H1 as the user decided on
# 14 Oct 2026). As root it closes the folders WSL shares between distributions, keeps name lookup working and checks
# the layout; then it becomes the steplet account for good and starts Claude Code. Installed by launch.rs as
# /opt/steplet/bin/start-run before each run, with root-common.sh beside it, and run with `wsl.exe -u root`. Its
# standard input is the task text, which only Claude Code reads. launch.rs has checked every argument.
# Arguments: run id, time limit in seconds, most turns, and (tests only) the scripted model's address.
# Exit codes of its own: 30 the shared folders, 31 the layout; after that, Claude Code's, or 124 at the time limit.
set -u

WHO=steplet-start
# fail, close_shared_folders and check_layout, shared with start-hands.sh.
. /opt/steplet/bin/root-common.sh || { echo "$WHO: could not read root-common.sh" >&2; exit 31; }

ID=$1 LIMIT=$2 TURNS=$3 MODEL_URL=${4:-}
R=/var/lib/steplet/runs/$ID
# These steps must not read the task text, so their input is /dev/null; only Claude Code gets it.
close_shared_folders < /dev/null
check_layout "$ID" "$R/settings.json" "$R/mcp.json" < /dev/null
cd "/work/$ID/repo" || fail 31 "the run has no working copy"
VARS=(HOME="$R/home" PATH=/usr/local/bin:/usr/bin:/bin LANG=C.UTF-8 CLAUDE_CONFIG_DIR="$R/config" TMPDIR="/tmp/steplet-run-$ID"
    DISABLE_AUTOUPDATER=1 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1 CLAUDE_CODE_DISABLE_AUTO_MEMORY=1)
if [ -n "$MODEL_URL" ]; then VARS+=("ANTHROPIC_BASE_URL=$MODEL_URL"); fi
# From here on nothing runs as root: setpriv becomes steplet, drops every power, empties the set any power could ever
# come back from, and forbids gaining any back. (A new user namespace, as the fence makes, starts its own full set.)
exec setpriv --reuid=steplet --regid=steplet --init-groups --inh-caps=-all --bounding-set=-all --no-new-privs -- \
    /usr/bin/timeout -k 10 "$LIMIT" /usr/bin/env -i "${VARS[@]}" \
    /opt/steplet/claude-cli/bin/claude -p --output-format stream-json --verbose --include-hook-events \
    --max-turns "$TURNS" --setting-sources user --strict-mcp-config \
    --settings "$R/settings.json" --mcp-config "$R/mcp.json" \
    --permission-prompt-tool mcp__gate__decide --tools Bash,Read,Write,Edit,Glob,Grep,WebFetch
