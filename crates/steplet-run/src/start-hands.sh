#!/bin/bash
# Runs one move of Steplet's own agent (the Nebius plan, Task 5): a command, or one of hands.py's file tools. As root
# it closes the folders WSL shares between distributions and checks the layout, as start-run.sh does; then the move
# runs as the steplet account, with no powers, inside the run's fence, and whatever it left running is ended. Installed
# by hands.rs as /opt/steplet/bin/start-hands, with root-common.sh and hands.py beside it, and run with
# `wsl.exe -u root` once per move. hands.rs has checked every argument.
# Arguments: run id, time limit in seconds, and the move's kind: "command", or a file tool of hands.py.
# Standard input: the move's input (the command line, or the file tool's JSON), which root keeps where only it can
# read it.
# Standard output, from root alone: one line "steplet-hands exit=<n> timed_out=<0|1> bytes=<total>", then at most
# 1 MiB of what the move printed, its output and error output together. The move never holds root's standard output,
# so it cannot write that line.
# Exit codes of its own: 30 the shared folders, 31 the layout, 32 the move could not be started, 33 a program of the
# steplet account would not stop.
set -u

WHO=steplet-hands
# fail, close_shared_folders, check_layout and stop_steplet, shared with start-run.sh.
. /opt/steplet/bin/root-common.sh || { echo "$WHO: could not read root-common.sh" >&2; exit 31; }

ID=$1 SECS=$2 KIND=$3
R=/var/lib/steplet/runs/$ID H=/var/lib/steplet/runs/$ID/hands
SRT=/opt/steplet/srt/node_modules/.bin/srt
# The most of a move's output kept; the rest is only counted.
MOST=1048576

close_shared_folders < /dev/null
check_layout "$ID" "$R/srt-hands.json" < /dev/null
# The moves' own folder, made by prepare: root's alone, so nothing the moves run can read or change what is in it.
[ -d "$H" ] && [ ! -L "$H" ] && [ "$(stat -c '%U %a' -- "$H")" = "root 700" ] ||
    fail 31 "the moves' folder is missing or not Steplet's"
# The last move's end already ended what it left; this is the backstop, before anything of this move is kept.
stop_steplet < /dev/null || fail 33 "a program from an earlier move would not stop"
rm -f -- "$H/in" "$H/out" "$H/rest" "$H/pipe" && cat > "$H/in" || fail 32 "could not keep the move's input"
case $KIND in
    command) MOVE=(/bin/bash --noprofile --norc -c "$(cat -- "$H/in")") INPUT=/dev/null ;;
    read | write | edit | list | find | search)
        MOVE=(/usr/bin/python3 -I /opt/steplet/bin/hands.py "$KIND") INPUT=$H/in ;;
    *) fail 32 "no such move: $KIND" ;;
esac
# srt guards some file names in the folder it starts in, and in this working copy it would put one inside .git, which
# the fence keeps read-only, and refuse to start. So, as for prepare's checkout, it starts in the run's temporary
# folder, and the move steps into the working copy inside the fence.
cd "/tmp/steplet-run-$ID" || fail 31 "the run has no temporary folder"

# Root reads what the move prints through a pipe of its own: the first MOST bytes are kept and the rest only counted,
# so a move that prints without end cannot fill the disk. head reads no more of a pipe than it keeps, so wc counts
# exactly the rest. The reader holds nothing of root's own output.
# The move may open its output again to write, as `> /dev/stdout` does, so others may write the pipe; only root can
# read it, and only root and the move's own programs (through /proc) can reach it, in root's folder.
mkfifo "$H/pipe" && chmod 602 "$H/pipe" || fail 32 "could not make the move's output pipe"
{ head -c "$MOST" > "$H/out"; wc -c > "$H/rest"; } < "$H/pipe" > /dev/null 2>&1 &
READER=$!
# Opened by root before the move starts, so the reader always sees the pipe close, even if the move cannot start.
exec 3> "$H/pipe" || fail 32 "could not open the move's output pipe"
# srt sets the move's TMPDIR to CLAUDE_CODE_TMPDIR, or to /tmp/claude, which the fence keeps read-only.
VARS=(HOME="/tmp/steplet-run-$ID" PATH=/usr/local/bin:/usr/bin:/bin LANG=C.UTF-8 TMPDIR="/tmp/steplet-run-$ID"
    CLAUDE_CODE_TMPDIR="/tmp/steplet-run-$ID")
START=$(date +%s%N)
# As in start-run.sh: setpriv becomes steplet, drops every power, empties the set any power could come back from and
# forbids gaining any back; the move gets only the files root opened for it.
setpriv --reuid=steplet --regid=steplet --init-groups --inh-caps=-all --bounding-set=-all --no-new-privs -- \
    /usr/bin/timeout -k 5 "$SECS" /usr/bin/env -i "${VARS[@]}" \
    "$SRT" --settings "$R/srt-hands.json" -- /usr/bin/env -C "/work/$ID/repo" "${MOVE[@]}" < "$INPUT" >&3 2>&3 3>&-
STATUS=$?
END=$(date +%s%N)
exec 3>&-
# timeout says 124 when it ended the move, or 137 when the move would not end until killed. A move that exits 124 on
# its own before its time is up has not timed out.
TIMED=0
if { [ "$STATUS" = 124 ] || [ "$STATUS" = 137 ]; } && [ $(((END - START) / 1000000000)) -ge "$SECS" ]; then
    TIMED=1
fi
# Whatever the move left running is ended now, which also closes every copy of the pipe it held.
stop_steplet < /dev/null || fail 33 "a program the move left would not stop"
wait "$READER"
# The fence keeps a move from making the agent settings files (the names prepare's strip removes) by holding each
# name with an empty file while the move runs, and srt removes them as it ends. A move that had to be killed ends
# srt with it, so any still there are removed here: no move can make one, so an empty file there is only srt's.
for f in .claude .codex .cursor .gemini .mcp.json; do
    P=/work/$ID/repo/$f
    if [ -f "$P" ] && [ ! -L "$P" ] && [ ! -s "$P" ]; then rm -f -- "$P" || fail 31 "could not remove the fence's $f"; fi
done
TOTAL=$(($(stat -c %s -- "$H/out") + $(cat -- "$H/rest")))
printf 'steplet-hands exit=%s timed_out=%s bytes=%s\n' "$STATUS" "$TIMED" "$TOTAL"
cat -- "$H/out"
rm -f -- "$H/in" "$H/out" "$H/rest" "$H/pipe"
