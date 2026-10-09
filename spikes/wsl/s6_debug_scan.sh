#!/bin/bash
# S6c: a broader, independent look at Claude Code's --debug log for anything about the fence blocking an action.
# Usage: s6_debug_scan.sh <session id>
D=$HOME/.local/share/steplet/claude-config/debug/$1.txt
echo "debug log: $D ($(wc -l < "$D") lines, $(wc -c < "$D") bytes)"
echo "--- lines mentioning sandbox, bwrap, seccomp, violation, denied, blocked, EPERM, EACCES, read-only (first 30):"
grep -n -i -E "sandbox|bwrap|seccomp|violation|denied|blocked|EPERM|EACCES|read-only|readonly" "$D" | cut -c1-260 | head -30
echo "--- count of those lines: $(grep -c -i -E "sandbox|bwrap|seccomp|violation|denied|blocked|EPERM|EACCES|read-only|readonly" "$D")"
echo "--- mentions of the fixture's targets (canary, outside, 47831, 1.1.1.1, cmd.exe, win.ini):"
grep -n -E "canary|/outside|47831|1\.1\.1\.1|cmd\.exe|win\.ini" "$D" | cut -c1-260 | head -10
echo "--- count: $(grep -c -E "canary|/outside|47831|1\.1\.1\.1|cmd\.exe|win\.ini" "$D")"
