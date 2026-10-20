#!/bin/bash
# Looks at Steplet's own Linux and changes nothing (phase 4 Task 6). Runs as root, sent on standard input by
# setup.rs. Arguments: the Claude Code version, the srt version, and the gate program's file name on the Linux disk.
# Prints one line for each check: "ok <what> <version>" (or "ok <what>"), or "missing <what>".
set -u

# "ok <what> <version>" when $2 is not empty, else "missing <what>".
say() {
    if [ -n "$2" ]; then echo "ok $1 $2"; else echo "missing $1"; fi
}

# The version of the npm package whose package.json is $1, read without starting it; empty when it is not there.
package_version() {
    [ -f "$1" ] && node -p "require('$1').version" 2> /dev/null
}

main() {
    local CLAUDE=$1 SRT=$2 GATE=$3 p v
    # The base tools (H4), installed by the package manager: every package setup.sh installs.
    for p in git python3 python3-venv build-essential ca-certificates bubblewrap socat ripgrep nodejs npm; do
        say "$p" "$(dpkg-query -W -f='${Status} ${Version}\n' "$p" 2> /dev/null | sed -n 's/^install ok installed //p')"
    done
    # Claude Code and srt, each at exactly the version this Steplet was tested with.
    v=$(package_version /opt/steplet/claude-cli/lib/node_modules/@anthropic-ai/claude-code/package.json)
    if [ "$v" = "$CLAUDE" ] && [ -x /opt/steplet/claude-cli/bin/claude ]; then say claude-code "$v"; else say claude-code ""; fi
    v=$(package_version /opt/steplet/srt/node_modules/@anthropic-ai/sandbox-runtime/package.json)
    if [ "$v" = "$SRT" ] && [ -x /opt/steplet/srt/node_modules/.bin/srt ]; then say srt "$v"; else say srt ""; fi
    # WSL's settings for this Linux: no Windows drives, no Windows PATH, systemd on, steplet the default user.
    v=yes
    for p in 'systemd=true' 'enabled=false' 'appendWindowsPath=false' 'default=steplet'; do
        grep -qx -- "$p" /etc/wsl.conf 2> /dev/null || v=
    done
    say wsl.conf "$v"
    # The steplet account: there, with its password locked.
    v=
    if id -u steplet > /dev/null 2>&1 && [ "$(passwd -S steplet 2> /dev/null | cut -d' ' -f2)" = L ]; then v=locked; fi
    say steplet-account "$v"
    # Steplet's folders, owned by root and not writable by the steplet account.
    v=yes
    for p in /opt/steplet /opt/steplet/gate /var/lib/steplet /var/lib/steplet/runs /work; do
        [ -d "$p" ] && [ "$(stat -c %U -- "$p")" = root ] || v=
        [ -d "$p" ] && setpriv --reuid=steplet --regid=steplet --init-groups -- /usr/bin/test -w "$p" && v=
    done
    say folders "$v"
    # The gate program of this build, named after its contents (H8).
    if [ -f "/opt/steplet/gate/$GATE" ]; then echo "ok gate"; else echo "missing gate"; fi
}

# As in Steplet's other scripts: the work is in one function, read whole before it runs, and nothing it starts reads
# the script.
main "$@" < /dev/null
