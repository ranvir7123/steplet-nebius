# What Steplet's two start scripts share (the Nebius plan, Task 5): start-run.sh, which starts Claude Code, and
# start-hands.sh, which runs one move of Steplet's own agent. Both run as root and read this file first. Installed by
# Steplet as /opt/steplet/bin/root-common.sh, owned by root, beside them. It only defines functions; `fail` names the
# script by `$WHO`, which each sets before reading this.

fail() {
    echo "${WHO:-steplet}: $2" >&2
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

# The run's layout, as prepare made it: /work holds this run alone, with its working copy; the run's temporary folder
# is the steplet account's alone; and each file named after the run id is root's, readable by all, writable by root
# alone. Arguments: the run id, then those files.
check_layout() {
    local ID=$1 T=/tmp/steplet-run-$1 f
    shift
    [ "$(ls -A /work)" = "$ID" ] || fail 31 "/work holds something besides this run"
    [ -d "/work/$ID/repo" ] || fail 31 "the run has no working copy"
    # The run's temporary folder, made by prepare: a folder (not a link) of the steplet account's, for it alone.
    [ -d "$T" ] && [ ! -L "$T" ] && [ "$(stat -c '%U %a' -- "$T")" = "steplet 700" ] ||
        fail 31 "the run's temporary folder is missing or not Steplet's"
    for f in "$@"; do
        [ "$(stat -c '%U %a' -- "$f" 2> /dev/null)" = "root 644" ] || fail 31 "$f is missing or not Steplet's"
    done
}

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
