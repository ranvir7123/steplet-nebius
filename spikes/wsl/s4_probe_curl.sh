#!/bin/bash
# Milestone 1, phase 0, S4 diagnostic (9 Oct 2026). Throwaway. Run inside steplet-test.
# Through the same srt settings as the fenced clone: can curl reach github.com, and is another host refused?
P=$HOME/.local/share/steplet; W=$HOME/steplet-work/S4-fenced-clone
S=$W/srt-clone.json
SRT=$P/srt/node_modules/.bin/srt
echo "curl present: $(command -v curl || echo no)"
for u in https://github.com https://example.com; do
  echo "=== $u through the fence"
  "$SRT" --settings "$S" -- /usr/bin/curl -sS -o /dev/null -w "http_code=%{http_code}\n" --max-time 20 "$u" 2>&1 | tail -2
done
echo "=== https://github.com without the fence, for comparison"
/usr/bin/curl -sS -o /dev/null -w "http_code=%{http_code}\n" --max-time 20 https://github.com 2>&1 | tail -2
