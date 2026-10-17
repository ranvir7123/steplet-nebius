#!/bin/bash
# Steplet's own look at a run after it ends and before it is removed (plan Task 10): the files the task changed, and
# the run's Claude Code config for the trust check (D3). Runs as root, sent on standard input by prepare.rs, which has
# checked the run id. Arguments: the run id, then "status" or "config".
# Exit codes: 30 a program of the run (or of the listing) would not stop, 31 the listing inside the fence failed,
# 32 the config could not be read.
set -u

# As in prepare.sh: every program of the steplet account, caught again on the next pass if it started another.
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
    local ID=$1 WHAT=$2
    local R=/var/lib/steplet/runs/$ID
    local REPO=/work/$ID/repo CONFIG=$R/config/.claude.json A=$R/after
    local SRT=/opt/steplet/srt/node_modules/.bin/srt
    # Nothing the run left running may change what Steplet is looking at.
    stop_steplet || { echo "steplet-after: a program from the run would not stop" >&2; exit 30; }
    # As the steplet account, with nobody's configuration (as prepare.sh runs git), and a home that holds nothing the
    # run wrote, so nothing the run wrote is read as settings.
    local AS=(setpriv --reuid=steplet --regid=steplet --init-groups --inh-caps=-all --no-new-privs --
        /usr/bin/env -i PATH=/usr/bin:/bin HOME="$A" TMPDIR="$A" LANG=C.UTF-8
        GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0 GIT_OPTIONAL_LOCKS=0)
    case $WHAT in
        status)
            # The agent wrote this working copy, so git may find in it things that make it start a program (a
            # submodule's own config can name a filter). Steplet's listing therefore runs inside the fence, as the
            # checkout in prepare.sh does: no network, nothing readable but the working copy, nothing writable. Like
            # the checkout, it starts in an empty folder of its own, because srt guards some file names in the folder
            # it starts in. Without hooks or a file-system monitor, and without writing the index.
            install -d -o steplet -g steplet -m 700 "$A" && cd "$A" ||
                { echo "steplet-after: could not make the listing's folder" >&2; exit 31; }
            cat > "$R/srt-after.json" << EOF || { echo "steplet-after: could not write the fence's settings" >&2; exit 31; }
{"filesystem": {"denyRead": ["/mnt", "/var/lib/steplet", "/work"], "allowRead": ["$REPO"],
                "allowWrite": [], "denyWrite": []},
 "network": {"allowedDomains": [], "deniedDomains": []}}
EOF
            local status=0
            timeout -k 10 90 "${AS[@]}" "$SRT" --settings "$R/srt-after.json" -- \
                /usr/bin/git -c core.hooksPath=/dev/null -c core.fsmonitor=false --no-optional-locks \
                -C "$REPO" status --porcelain=v1 -z --untracked-files=all --no-renames || status=$?
            # Nothing the listing started may still be running while root removes its folder.
            stop_steplet || { echo "steplet-after: a program from the listing would not stop" >&2; exit 30; }
            cd / && rm -rf -- "$A" "$R/srt-after.json"
            [ "$status" = 0 ] || { echo "steplet-after: git status failed inside the fence ($status)" >&2; exit 31; }
            ;;
        config)
            # Read as the steplet account, so a link the run left there cannot make root read another file; a plain
            # file only, at most 4 MiB.
            [ -f "$CONFIG" ] || { echo "steplet-after: the run's config file is missing" >&2; exit 32; }
            "${AS[@]}" /usr/bin/head -c 4194304 -- "$CONFIG" ||
                { echo "steplet-after: the run's config file could not be read" >&2; exit 32; }
            ;;
        *) exit 2 ;;
    esac
}

# As in prepare.sh: the work is in one function, read whole before it runs, and nothing it starts reads the script.
main "$@" < /dev/null
