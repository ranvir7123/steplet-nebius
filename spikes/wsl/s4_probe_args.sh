#!/bin/bash
# Milestone 1, phase 0, S4 diagnostic (9 Oct 2026). Throwaway. Run inside steplet-test.
# Does srt take a "-c" meant for the inner program as its own option? Does "--" stop that?
P=$HOME/.local/share/steplet; T=$HOME/steplet-work/srt-probe
SRT=$P/srt/node_modules/.bin/srt
S=$T/3-deny-private-allow-helper.json
echo "=== A. no separator: srt --settings S /usr/bin/python3 -c \"print('A ran', 1+1)\""
"$SRT" --settings "$S" /usr/bin/python3 -c "print('A ran', 1+1)" 2>&1 | tail -2; echo "exit: ${PIPESTATUS[0]}"
echo "=== B. with --: srt --settings S -- /usr/bin/python3 -c \"print('B ran', 1+1)\""
"$SRT" --settings "$S" -- /usr/bin/python3 -c "print('B ran', 1+1)" 2>&1 | tail -2; echo "exit: ${PIPESTATUS[0]}"
echo "=== C. with --, an argument with spaces, quotes and a semicolon: printf '[%s]\n' \"a b\" \"it's; echo INJECTED\""
"$SRT" --settings "$S" -- /usr/bin/printf '[%s]\n' "a b" "it's; echo INJECTED" 2>&1 | tail -3; echo "exit: ${PIPESTATUS[0]}"
echo "=== D. with --, git with two -c options"
"$SRT" --settings "$S" -- /usr/bin/git -c core.hooksPath=/dev/null -c core.fsmonitor=false --version 2>&1 | tail -2; echo "exit: ${PIPESTATUS[0]}"
