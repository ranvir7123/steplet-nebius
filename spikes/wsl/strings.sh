#!/bin/bash
# Read-only: distinct names inside the pinned Claude Code program that contain a word, or a few
# passages around it with "context". Usage: strings.sh <word> [context]
B=$(readlink -f "$HOME/.local/share/steplet/claude-cli/bin/claude")
if [ "${2:-}" = context ]; then
  grep -a -o -E ".{0,160}$1.{0,160}" "$B" | head -5
else
  grep -a -o -E "[A-Za-z_]{0,40}$1[A-Za-z_]{0,40}" "$B" | sort -u | head -80
fi
