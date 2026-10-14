#!/bin/bash
# Starts the agent for one prepared run (plan Task 7: section 3's launch, with choice H1 as the user decided on
# 14 Oct 2026). As root it closes the folders WSL shares between distributions, keeps name lookup working and checks
# the layout; then it becomes the steplet account for good and starts Claude Code. Installed by launch.rs as
# /opt/steplet/bin/start-run before each run, and run with `wsl.exe -u root`. Its standard input is the task text,
# which only Claude Code reads. launch.rs has checked every argument.
# Arguments: run id, time limit in seconds, most turns, and (tests only) the scripted model's address.
# Exit codes of its own: 30 the shared folders, 31 the layout; after that, Claude Code's, or 124 at the time limit.
set -u

fail() {
    echo "steplet-start: $2" >&2
    exit "$1"
}

# True when the path reaches a mount of its own: it is on another device than its folder. WSL's /tmp/.X11-unix can be
# listed as a mount yet lie hidden under Ubuntu's own /tmp, mounted later on top; its path then reaches nothing of it.
reachable_mount() {
    [ -e "$1" ] && [ "$(stat -c %d -- "$1")" != "$(stat -c %d -- "$(dirname -- "$1")")" ]
}

close_shared_folders() {
    # /etc/resolv.conf, the name lookup file, is WSL's link to /mnt/wsl/resolv.conf, and WSL puts that link back
    # whenever a new session starts. So /mnt/wsl is not just closed but replaced, by a small folder of Steplet's own
    # holding a copy of that one file: the link keeps working, and nothing other systems share there is visible.
    local lookup m
    lookup=$(cat /etc/resolv.conf) || fail 30 "could not read the name lookup file"
    # WSL's own programs may still hold these, so each is detached now and freed when they let go.
    for m in /mnt/wslg /tmp/.X11-unix /mnt/wsl; do
        if reachable_mount "$m"; then umount -R -l "$m" 2> /dev/null; fi
        if reachable_mount "$m"; then fail 30 "could not close $m"; fi
    done
    mount -t tmpfs -o size=64k,mode=755 steplet-wsl /mnt/wsl &&
        printf '%s\n' "$lookup" > /mnt/wsl/resolv.conf ||
        fail 30 "could not keep name lookup working"
    [ "$(findmnt -rn -o TARGET,SOURCE | grep -E '^/mnt(/| )')" = "/mnt/wsl steplet-wsl" ] ||
        fail 30 "a folder under /mnt is still open"
    [ "$(ls -A /mnt/wsl)" = resolv.conf ] || fail 30 "/mnt/wsl holds more than the name lookup file"
    # srt lets every fence write to /tmp/claude if it exists (Task 6), so it must not exist.
    rm -rf -- /tmp/claude || fail 30 "could not remove /tmp/claude"
}

check_layout() {
    local ID=$1 R=/var/lib/steplet/runs/$1 f
    [ "$(ls -A /work)" = "$ID" ] || fail 31 "/work holds something besides this run"
    [ -d "/work/$ID/repo" ] || fail 31 "the run has no working copy"
    for f in "$R/settings.json" "$R/mcp.json"; do
        [ "$(stat -c '%U %a' -- "$f" 2> /dev/null)" = "root 644" ] || fail 31 "$f is missing or not Steplet's"
    done
}

ID=$1 LIMIT=$2 TURNS=$3 MODEL_URL=${4:-}
R=/var/lib/steplet/runs/$ID
# These steps must not read the task text, so their input is /dev/null; only Claude Code gets it.
close_shared_folders < /dev/null
check_layout "$ID" < /dev/null
cd "/work/$ID/repo" || fail 31 "the run has no working copy"
VARS=(HOME="$R/home" PATH=/usr/local/bin:/usr/bin:/bin LANG=C.UTF-8 CLAUDE_CONFIG_DIR="$R/config" TMPDIR="$R/tmp"
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
