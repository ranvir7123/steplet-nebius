#!/bin/bash
# Prepares one run inside Steplet's own Linux (plan Task 6, decision D6), in the layout of plan section 3. Runs as
# root, sent on standard input by prepare.rs, which has checked every argument; nothing here is installed.
# Two entries, named by the first argument:
#   prepare <run id> <repository address (plain https)> <commit> <settings.json> <mcp.json> (both in base64)
#   strip <run id>: only the removal of the agent settings files from that run's working copy (phase 4 Task 2),
#   which prepare also runs.
# Each agent settings file removed is said on standard output as "removed: <name>", and a ready copy as
# "steplet-prepare: ready at <commit>"; prepare.rs reads both. Nothing else this script starts writes to standard
# output: git's and the fence's output goes to the error output, so nothing the repository holds can add a line. A failed step exits with its own code, which
# prepare.rs turns into words: 10 clear, 11 clone, 12 commit, 13 checkout, 14 agent files, 15 run files.
set -u

fail() {
    echo "steplet-prepare: $2" >&2
    exit "$1"
}

# Ends every program of the steplet account and waits until none is left; one that starts another while being ended
# is caught on the next pass.
stop_steplet() {
    for _ in $(seq 50); do
        pkill -KILL -u steplet
        pkill -KILL -U steplet
        pgrep -u steplet > /dev/null || pgrep -U steplet > /dev/null || return 0
        sleep 0.1
    done
    return 1
}

main() {
    local ID=$1 URL=$2 COMMIT=$3 SETTINGS=$4 MCP=$5
    local W=/work/$ID R=/var/lib/steplet/runs/$ID T=/tmp/steplet-run-$ID
    local REPO=$W/repo C=$R/checkout
    local SRT=/opt/steplet/srt/node_modules/.bin/srt

    # Git runs as the steplet account with nobody's configuration: no system or personal settings file, an empty
    # home, no hooks, no file-system monitor, no password prompt, no large-file downloads.
    local AS=(setpriv --reuid=steplet --regid=steplet --init-groups --inh-caps=-all --no-new-privs --
        /usr/bin/env -i PATH=/usr/bin:/bin HOME="$C" TMPDIR="$C" LANG=C.UTF-8
        GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0 GIT_LFS_SKIP_SMUDGE=1)
    local GIT=(/usr/bin/git -c core.hooksPath=/dev/null -c core.fsmonitor=false)

    # 1. One run at a time: end whatever the steplet account left running, then empty /work and the runs folder, and
    #    remove every run's temporary folder under /tmp. With nothing of the steplet account running, nothing can
    #    make one again before it is made fresh below.
    stop_steplet || fail 10 "a program from the last run would not stop"
    find /work /var/lib/steplet/runs -mindepth 1 -maxdepth 1 -exec rm -rf -- {} + &&
        find /tmp -mindepth 1 -maxdepth 1 -name 'steplet-run-*' -exec rm -rf -- {} + ||
        fail 10 "could not clear the last run"
    install -d -o root -g root -m 755 "$W" "$R" &&
        install -d -o steplet -g steplet -m 755 "$REPO" &&
        install -d -o steplet -g steplet -m 700 "$R/home" "$R/config" "$T" "$C" ||
        fail 10 "could not make the run's folders"

    # 2. The hardened clone, outside the fence and without a checkout: https only, every object checked, no bundle
    #    addresses, no stored passwords, no submodules, no template (so no sample hooks). A stalled download gives up.
    cd "$C" || fail 11 "no folder to clone from"
    timeout -k 10 600 "${AS[@]}" "${GIT[@]}" -c protocol.allow=never -c protocol.https.allow=always \
        -c transfer.fsckObjects=true -c transfer.bundleURI=false -c credential.helper= \
        -c http.lowSpeedLimit=1000 -c http.lowSpeedTime=60 \
        clone --quiet --no-checkout --no-recurse-submodules --template= -- "$URL" "$REPO" >&2 ||
        fail 11 "could not clone $URL"

    # 3. The declared commit must be among what was fetched, and be a commit.
    [ "$("${AS[@]}" "${GIT[@]}" -C "$REPO" cat-file -t "$COMMIT" 2> /dev/null)" = commit ] ||
        fail 12 "$URL has no commit $COMMIT"

    # 4. The checkout, inside the fence with no agent (D6): it can write the working copy only and reach no network.
    #    srt guards some file names in the folder it starts in, so it starts in the empty folder, not the working
    #    copy; otherwise a commit that holds .gitmodules or .vscode/ could not be checked out. "--" comes before the
    #    command.
    cat > "$R/srt-checkout.json" << EOF || fail 13 "could not write the fence's settings"
{"filesystem": {"denyRead": ["/mnt", "/var/lib/steplet", "/work"], "allowRead": ["$REPO"],
                "allowWrite": ["$REPO"], "denyWrite": []},
 "network": {"allowedDomains": [], "deniedDomains": []}}
EOF
    timeout -k 10 300 "${AS[@]}" "$SRT" --settings "$R/srt-checkout.json" -- \
        "${GIT[@]}" -C "$REPO" checkout --quiet --detach "$COMMIT" >&2 ||
        fail 13 "the checkout of $COMMIT failed inside the fence"
    [ "$("${AS[@]}" "${GIT[@]}" -C "$REPO" rev-parse HEAD)" = "$COMMIT" ] ||
        fail 13 "the working copy is not at $COMMIT"
    # Nothing the clone or the checkout started may still be running while root works on their folders.
    stop_steplet || fail 13 "a program from the checkout would not stop"

    # 5. The agent settings files the repository carries are removed (D3), by the steplet account.
    strip "$ID"

    # 6. The run's settings file and MCP config, owned by root so the agent cannot change them.
    cd / && rm -rf -- "$C" "$R/srt-checkout.json" &&
        printf '%s' "$SETTINGS" | base64 -d > "$R/settings.json" &&
        printf '%s' "$MCP" | base64 -d > "$R/mcp.json" &&
        chmod 644 "$R/settings.json" "$R/mcp.json" ||
        fail 15 "could not write the run's settings"
    echo "steplet-prepare: ready at $COMMIT"
}

# Removes the agent settings files and folders from the working copy of run $1, /work/<run id>/repo, in a fixed order,
# saying each one it removed (D3). It takes the run id, as discard.sh and after.sh do, so it can touch no other folder.
# Every program of the steplet account is ended first, so none can put a file back after its removal is said; and
# all five are checked again at the end. The removal runs as the steplet account with only a PATH (it starts no git,
# so it needs none of main's git settings).
strip() {
    local ID=${1-} f
    [[ $ID =~ ^[a-z0-9][a-z0-9-]{0,63}$ ]] || fail 14 "not a run id: $ID"
    local REPO=/work/$ID/repo
    [ -d "$REPO" ] && [ ! -L "$REPO" ] || fail 14 "run $ID has no working copy"
    stop_steplet || fail 14 "a program of the steplet account would not stop"
    for f in .claude .codex .cursor .gemini .mcp.json; do
        [ -e "$REPO/$f" ] || [ -L "$REPO/$f" ] || continue
        setpriv --reuid=steplet --regid=steplet --init-groups --inh-caps=-all --no-new-privs -- \
            /usr/bin/env -i PATH=/usr/bin:/bin /bin/rm -rf -- "$REPO/$f"
        if [ -e "$REPO/$f" ] || [ -L "$REPO/$f" ]; then fail 14 "could not remove $f from the working copy"; fi
        echo "removed: $f"
    done
    for f in .claude .codex .cursor .gemini .mcp.json; do
        if [ -e "$REPO/$f" ] || [ -L "$REPO/$f" ]; then fail 14 "$f came back in the working copy"; fi
    done
}

entry() {
    case "${1-}" in
        prepare) shift && main "$@" ;;
        strip) shift && strip "$@" ;;
        *) fail 10 "no such entry: ${1-}" ;;
    esac
}

# The script comes on standard input, and bash reads it as it goes. All the work is in functions, which bash reads
# whole before it runs them, and every program they start gets /dev/null as its input, so none can read the script.
entry "$@" < /dev/null
