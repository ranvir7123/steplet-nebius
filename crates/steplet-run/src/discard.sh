#!/bin/bash
# Removes one run after it ends (D4, D6): ends whatever the steplet account still runs, then deletes the run's working
# copy and private folders. Runs as root, sent on standard input by prepare.rs. Argument: the run id. Exit code 20
# when something is left.
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
    local ID=$1
    stop_steplet
    rm -rf -- "/work/$ID" "/var/lib/steplet/runs/$ID"
    if pgrep -u steplet > /dev/null || [ -e "/work/$ID" ] || [ -e "/var/lib/steplet/runs/$ID" ]; then
        echo "steplet-discard: run $ID could not be removed completely" >&2
        exit 20
    fi
}

# As in prepare.sh: the work is in one function, read whole before it runs, and nothing it starts reads the script.
main "$@" < /dev/null
