# Spikes (milestone 1, phase 0)

Throwaway tests that answer one question each before the product is built on the answer.
The plan is `docs/plans/2026-10-08-milestone-1-phase-0.md`; results with pasted output are in
`docs/milestone-1-spikes.md`. Nothing here is product code, and none of it is copied into the
product: phases 1 to 7 write their code fresh, tests first.

- `host/`, `gate/`: Windows stand-ins for the Steplet app and `steplet-gate.exe`.
- `tools/`: the scripted model, self-tests, the run script, the results helper.
- `wsl/`: scripts that run inside the test distribution `steplet-test`.
- `runs/`: one JSON file per run; each says exactly what that run does.
- `scripts/`: the scripted model's tool calls.  `probes/`: programs the fenced commands run.
- `node-gate/`: milestone 0's Node stand-in gate, unchanged.
