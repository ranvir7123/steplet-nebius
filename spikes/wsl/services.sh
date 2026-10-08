#!/bin/bash
# Keeps one WSL session open for a spike run and runs its helpers: the scripted model and the loopback listener.
# Usage: services.sh <ID>. Stops when C:\dev\m1\runs\<ID>\stop exists.
ID=$1; M=$HOME/m1; S=/mnt/c/dev/runFile/spikes; R=/mnt/c/dev/m1/runs/$ID
KEY=$HOME/.local/share/steplet/secrets/fake-api-key
pids=()
if [ -f "$M/logs/$ID.script.json" ]; then
  python3 -I "$S/tools/fake_model.py" 47900 "$M/logs/$ID.script.json" "$M/logs/$ID.model.log" "$KEY" > "$M/logs/$ID.model.stdout" 2>&1 &
  pids+=($!)
fi
python3 -I "$S/tools/listener.py" 47831 "$M/logs/$ID.listener.log" > /dev/null 2>&1 &
pids+=($!)
sleep 1
for p in "${pids[@]}"; do
  kill -0 "$p" 2>/dev/null || { echo "a helper died at start (pid $p)" > "$R/services.ready"; exit 1; }
done
echo "running, pids ${pids[*]}" > "$R/services.ready"
while [ ! -e "$R/stop" ]; do sleep 0.5; done
kill "${pids[@]}" 2>/dev/null
