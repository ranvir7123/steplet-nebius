#!/bin/bash
# Launch a prepared spike run from inside WSL, the way milestone 0 did. Used by S7 only.
ID=$1; R=/mnt/c/dev/m1/runs/$ID; L=/mnt/c/dev/m1/logs
mapfile -t ARGV < "$R/argv.txt"
cd "$(cat "$R/cd.txt")" || exit 1
START=$(date +%s)
"${ARGV[@]}" < "$R/task.txt" > "$L/$ID.jsonl" 2> "$L/$ID.stderr.txt"
CODE=$?
echo "launch inside WSL: exit code $CODE, seconds $(( $(date +%s) - START ))"
