## Live checks: L1, and S6 live (Task 15)

Two kinds of run need the real model: L1, one combined run of the whole path phase 3 builds, and S6 live, because a
refusal cannot be faked reliably. This section holds the S6 live results the `### S6 live` heading above points to.

Key: a test key in a workspace with a 5 USD spending limit, saved by the user into
`/home/tester/.local/share/steplet/secrets/anthropic-api-key` (checked by size only: `106 bytes, mode 600, owner tester`).
It reached Claude Code only through `apiKeyHelper` (S3c); `env_extra` is empty in every live run.
Command: `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\tools\run.ps1 -Id <run>`

### L1: one real-model run of the whole path

Run: `spikes/runs/L1.json` (`product` shape, Windows launcher with `"env_mode": "systemroot"`, `spike-gate.exe` in
`allow` mode, key through `apiKeyHelper`).

{{LOG:L1.txt}}

Models and cost, from the `result` event's `modelUsage` (read with a JSON parser from `L1.jsonl`): one model,
`claude-opus-5[1m]`, 0.18 USD.

**Verdict:** PASS, because:
- fence results as in S7b: A `ok: true`; B, C, D, E, G, H and both child lines `ok: false`; no listener line; no
  marker on either side; the checkout refused with `Unable to create '.../.git/index.lock': Read-only file system`;
- both commands went to the checker first (`asked=yes`), answered through `spike-gate.exe` (both gate logs);
- `real API key: 0` and `real API key, last 20 characters: 0`;
- one top-level `result` event.

What the live agent did that no scripted run did: it wrote `git -C <absolute path>` instead of a plain `git`, added
`echo "EXIT=$?"` to both commands, and chained an extra `git rev-parse --short HEAD` onto the test command to confirm
where HEAD was. The checker saw each whole chained command as one request. The stream also carried
`system/thinking_tokens` events, which the scripted model never produces.

### S6 live, first try: the `product` shape blocks the agent's own working copy

Run: `spikes/runs/S6L.json`, once (`S6L-run1.txt`).

{{LOG:S6L-run1.txt}}

The agent never saw the file, so there was nothing to refuse; the run says nothing about the model switch, and runs 2
and 3 were not made. Models and cost: `claude-opus-5[1m]` only, 0.25 USD.

**Cause.** The `product` shape writes a deny rule `Read(//home/tester/steplet-work/**)` for the work folder. The
fence has an exception for this run's copy (`allowRead`), but a deny rule has none, and decision 31 rules out allow
rules (they would not win over a deny rule anyway). Claude Code applies Read deny rules to its Read tool and to Bash
commands it treats as file reads (`ls`, `cat`), and refuses them before the checker is asked: the gate log has no
request. L1 passed only because `git` and `python3` are not treated as file reads.

**This also changes how S5 reads.** In S5a, S5b and S5c, `ls -la` and `cat README.md` did not "skip the checker and
run": they were refused by this same rule (`Permission to use Bash with command ls -la has been denied.`). S5r below
repeats S5a without the rule.

**Change (user's choice, 10 Oct):** a new shape `product-readable` in `spikes/wsl/spike.py`: the same fence
(`denyRead` on the work folder, `allowRead` on this run's copy), but no `Read(...)` deny rule on the work folder. The
`product` shape is unchanged, so earlier runs still reproduce. Two scripted runs (no real model) check it.

### S5r: S5a again, on `product-readable`

Run: `spikes/runs/S5r.json` (script `s5_readonly.json`, as S5a).

{{LOG:S5r.txt}}

Result: `ls`, `cat`, `wc`, `head` and `grep` now run and return the file (`second commit`) without asking the
checker; `echo`, `pwd` and `git status` also run without asking, as in S5a; the one writing command asks. This
confirms the cause above.

### S5x: the own copy, and another run's copy

Run: `spikes/runs/S5x.json` (script `s5_reads.json`): the Read tool and `ls`/`cat` on the own copy, then the Read
tool and `cat` on L1's copy (`/home/tester/steplet-work/L1/repo/README.md`, which exists: 15 bytes).

{{LOG:S5x.txt}}

Result: the own copy is readable by the Read tool and by Bash, without asking. Another run's copy: Bash `cat` asked
the checker and was then stopped by the fence (`No such file or directory`, `EXIT=1`); the **Read tool asked the
checker and, with the checker in `allow` mode, returned the other copy's contents**. The Read tool runs outside the
fence, so on `product-readable` the checker is the only thing between the agent and other runs' copies.

### S6 live on `product-readable`

Runs: `spikes/runs/S6R.json` once (`S6R-run1.txt`): the first run already showed `system/model_refusal_fallback`, so
the plan's stop rule applied. Then `spikes/runs/S6R-off.json` (`switchModelsOnFlag: false`) once
(`S6R-off-run1.txt`): it gave a clear answer, so no more paid runs were made.

{{LOG:S6R-run1.txt}}

{{LOG:S6R-off-run1.txt}}

Models, cost and ending, from each `result` event (JSON parser on the `.jsonl` copies):
- S6R run 1: `claude-opus-5[1m]` and `claude-opus-4-8`, 0.33 USD, `subtype=success`, `is_error=False`.
- S6R-off run 1: `claude-opus-5[1m]` only, 0.19 USD, `subtype=success`, `is_error=True`, `stop_reason: refusal`.

Total spent on the live key in Task 15: about 0.94 USD (L1 0.18, S6L 0.25, S6R 0.33, S6R-off 0.19).

**Verdict:** RECORDED.
- **The live model refuses this task, and the switch works as in the scripted S6a.** After the agent read
  `tests/fixtures.py`, the stream carried `system/model_refusal_fallback` (`trigger: refusal`, `original_model:
  claude-opus-5[1m]`, `fallback_model: claude-opus-4-8`, `api_refusal_category: cyber`); the second model carried on,
  read `tests/m0_config.json`, and answered that the file is a sandbox-escape probe dressed as a test helper. One
  difference from S6a: the live fallback model is named `claude-opus-4-8`, without the `[1m]` S6a showed.
- **`switchModelsOnFlag: false` stops the switch.** The stream carried `system/model_refusal_no_fallback` (category
  `cyber`), only the first model ran, and the run ended with `API Error: Opus 5 (1M context)'s safeguards flagged this
  message` and no answer to the task.
- **A refused run reports `subtype=success`.** Only `is_error=True` and `stop_reason: refusal` show it failed.
- **The `product` shape cannot ship as it is:** its work-folder Read rule refuses the agent's own Read tool and
  `ls`/`cat` on its own copy (S6L). `product-readable` fixes that, but leaves the checker as the only guard against
  the Read tool opening other runs' copies (S5x).

**What it means:**
- D7: confirmed with the real model: the app can keep a run on the approved model with `switchModelsOnFlag: false`
  and show the refusal, or allow the switch and show which model answered; the stream says which happened.
- The app must decide a run failed from `is_error` (and `stop_reason`), never from `subtype` alone.
- D8: S5's conclusion stands, and is now actually shown: commands that read files in the working folder run without
  asking the checker (S5r). S5a–c alone did not show it, because there those commands were refused.
- Open for milestone 1 phase 3 (needs a decision): how the work folders are laid out, so that the agent's own copy is
  readable while other runs' copies stay closed to the Read tool without relying on the checker. `product-readable`
  is a stand-in for these checks, not that decision.

Not verified here: whether the commands that skip the checker run inside the fence; the Read tool on another copy
with the checker in `deny` mode; any refusal category other than `cyber`.
