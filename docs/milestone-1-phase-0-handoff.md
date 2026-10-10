# Handoff: milestone 1, phase 0, finished (10 Oct 2026)

Read `docs/handoff.md` first, then this file, then the plan `docs/plans/2026-10-08-milestone-1-phase-0.md`.
The user approved the plan on 8 Oct 2026 and chose to run it **inline in one session, not through separate
agents**, because almost every task needs a yes or a step from them. Each install still gets its own yes.
Results so far, with pasted output: `docs/milestone-1-spikes.md`.

## Where things stand

| Task | Status |
|---|---|
| 1. git repository (P6) | **Done.** First commit `18bc9b0`. `keys.txt` and `sk.json` live in `C:\Users\RANVIR\runfile-secrets\`, never opened; both in `.gitignore`. |
| 2. Rust tools on Windows | **Done.** Log: `C:\dev\m1\logs\setup-windows.txt`. Fuzzing needs the AddressSanitizer folder on `PATH` (trap below). |
| 3. Test distribution (P5) | **Done.** `steplet-test`, user `tester`. GUID in `C:\dev\m1\distro-id.txt`. |
| 4. Steplet's folders, Claude Code, srt, spike data | **Done** (yes A4). Claude Code 2.1.220 (a Linux program, `177 E L F`), srt 0.0.77, fixture with COMMIT1, both fake keys, Node gate packages load. Log: `C:\dev\m1\logs\setup-steplet.txt`. |
| 5. Spike harness | **Done.** Self-tests 8 of 8 and 6 of 6. Setup logs appended to `docs/milestone-1-spikes.md` (`c505631`). |
| 6. S7, scripted model | **PASS** (`1e90e60`). `allowRead` works in 2.1.220, so the `product` shape stays. |
| 7. S2, launching from Windows | **PASS with `env_mode` `systemroot`** (`d4436ab`). See "Changes to the plan" below. |
| 8. S1, Windows gate through interop | **PASS** (`f74e0b8`). The runner-up is not needed. |
| 9. S3, can the fence see the API key | **PASS through route S3c (`apiKeyHelper`)** (`a2b61a8`). S3a fails, S3b inconclusive. S3e ran with yes A5. |
| 10. S4, fenced checkout, read-only `.git` | **PASS** (`2829563`), after three fixes to the plan's `srt` scripts (see "Changes"). Two parts inconclusive by the plan's rule: Claude Code's fence protects `.git/hooks` and `.git/config` from commands on its own. |
| 11. S5, commands that skip the checker | **RECORDED** (`1c853a5`). No setting makes every command ask; `ls`, `cat`, `wc`, `head`, `grep` on working-folder files always skip it. `--tools` limits the tools as documented. **Re-read on 10 Oct:** in S5a–c `ls`/`cat` were refused by the `product` shape's work-folder Read rule, not run; S5r (`product-readable`) shows they do run without asking. |
| 12. S6, refusal switch (scripted) | **RECORDED** (`3f15f99`). `switchModelsOnFlag: false` in the run's settings file stops the switch; blocked fence attempts are not visible to Steplet. Field names: `stop_details.category`, `stop_details.explanation`. |
| 13. S8, Windows side from the fence | **PASS** (`7f9770f`), `nat` mode only. The plan's probe had to send the proxy's login (it is in the proxy address). |
| 14. S9, WASM, WebView2, check page | **PASS** (`8d770b6`, `7015c67`), yes A6 used. Built in the worktree's `spikes/target`. |
| 15. Live checks | **Done** (`90c7ead`), yes A7, A8 used; about 0.94 USD. L1 **PASS**. S6L could not read its own working copy (the `product` shape's work-folder Read rule); the user chose a stand-in shape `product-readable` (checked by scripted S5r, S5x); on it S6R showed the live switch (`claude-opus-5[1m]` to `claude-opus-4-8`, `cyber`) and S6R-off showed `switchModelsOnFlag: false` stops it. A refused run reports `subtype=success`, `is_error=True`. Results: `spikes/results/live.md`. |
| 16. Wrap-up | **Done** (10 Oct): summary at the end of `docs/milestone-1-spikes.md` (`spikes/results/summary.md`), status lines in the milestone plan and `docs/handoff.md`. The user chose the recommended option for each decision: E1 to E3, C8, C9 in `docs/decisions.md` (`990e2cc`). `C:\dev\m1\s9` moved to the Recycle Bin. **Still the user's call:** merging (remove the copies in `C:\dev\runFile\spikes` first), revoking the test key or keeping it for phase 5, `npx playwright uninstall`, milestone 0 clean-up. |

## Changes to the plan made while running it

- **Every run description with a `host` field uses `"env_mode": "systemroot"`, not `"empty"`** (the plan's own
  fallback in Task 7, Step 1). `wsl.exe` started with an empty Windows environment fails with
  `The RPC call contains a handle that differs from the declared handle type` (`Wsl/Service/0x8007072c`); if that
  call is the one starting a stopped distro, the next call hangs until `wsl --terminate steplet-test`. `SystemRoot`
  alone works, also as the starting call. Only `S2r-empty.json` keeps `empty`, as the measurement. The plan's later
  run descriptions (Tasks 10 onward) still say `empty`: change them to `systemroot` when you create the files.
- **`spikes/tools/run.ps1` fixed:** it built host options as `"--" + $p.Name, "$($p.Value)"`; the comma binds tighter
  than `+`, so `--env-mode systemroot` arrived as one argument and the host fell back to Empty. Brackets added. Every
  host option (`deadline_ms`, `stop_after_s`, `kill_claude_after_s`, `kill_wsl_after_s`) depended on this.
- **Product route for the key (Task 15): `apiKeyHelper` only** (S3c). Never put the key in Claude Code's environment.
- **Every direct `srt` call (Tasks 10 onward):** (1) put `--` between `srt`'s options and the command, or `srt` takes
  a `-c` meant for the command as its own and runs the next argument as a raw shell line; (2) when the settings deny
  `~/.local/share/steplet`, add `"allowRead": ["$P/srt/node_modules/@anthropic-ai/sandbox-runtime/vendor/seccomp"]`,
  or every fenced command fails with `apply-seccomp: No such file or directory` (exit 127); (3) a script that strips
  the environment with `env -i` must keep `GIT_CONFIG_PARAMETERS`, or git fails `srt`'s proxy login
  (`Proxy CONNECT aborted`). The plan's later `srt` scripts do not have these yet: add them when you create the files.
- New tools: `spikes/tools/s2_env_min.py` (which Windows variables `wsl.exe` needs), `spikes/tools/s3e_session.py`
  (reads one saved session record).
- `spikes/tools/selftest_gate.ps1` hands the gate a file instead of a PowerShell pipe (8 Oct).
- **New shape `product-readable`** in `spikes/wsl/spike.py` (10 Oct, user's choice): the `product` fence without the `Read(...)` deny rule on the work folder, which refused the agent's own Read tool and `ls`/`cat` on its own copy. A stand-in for the checks, not the product layout: on it only the checker stops the Read tool from opening other runs' copies (S5x). New runs `S5r`, `S5x`, `S6R`, `S6R-off`; script `s5_reads.json`.

## Where the files live: this worktree and the main checkout

This session worked in the worktree `C:\dev\runFile\.claude\worktrees\phase-0-plan-continuation-775872`, branch
`claude/phase-0-plan-continuation-775872`; every commit above is on that branch, not on `main`. But `run.ps1`,
`spike.py` and the WSL scripts read from `C:\dev\runFile\spikes` (`/mnt/c/dev/runFile/spikes`), which is the main
checkout. So new run files, scripts and the fixed `run.ps1` were **copied** into `C:\dev\runFile\spikes` for the
runs. There they are untracked files plus one modified tracked file (`spikes/tools/run.ps1`), identical to the
branch. Before merging the branch into `main`, remove those copies (`git -C C:\dev\runFile status` lists them), or
git may refuse to overwrite them.

On 10 Oct a new session ran in the worktree `steplet-phase0-live-checks-15666e` (branch `claude/steplet-phase0-live-checks-15666e`), which a hook allows to write only its own files. Its branch was fast-forwarded to `82901a6` and Tasks 15 and 16 were committed there, so **that branch holds all of phase 0**; merge it, not `claude/phase-0-plan-continuation-775872`. Six more copies went into `C:\dev\runFile\spikes` (`wsl/spike.py` modified; `scripts/s5_reads.json`, `runs/S5r.json`, `runs/S5x.json`, `runs/S6R.json`, `runs/S6R-off.json` new).

## Traps hit (do not rediscover them)

- **Never start `wsl.exe` with an empty environment** (above). If WSL answers `Catastrophic failure` or
  `E_UNEXPECTED`, run `wsl --terminate steplet-test` (only the test distro; check `wsl -l -v` first).
- **PowerShell 5.1 puts a byte-order mark in front of text it pipes into a program.** Write a file and use
  `Start-Process -RedirectStandardInput` instead.
- **In PowerShell, `"a" + $x, "b"` is `"a" + ($x, "b")`.** Bracket each element of an array literal.
- **`cd` in the Bash tool or `Set-Location` in the PowerShell tool moves the session's working directory.** Use absolute paths.
- **Set `$env:WSL_UTF8 = "1"` before calling `wsl.exe`**, or its output comes back as UTF-16.
- **The harness cuts long lines** (for example `permission_denials`). Count from the raw stream
  `C:\dev\m1\logs\<run>.jsonl` with a JSON parser.
- **Fuzzing on Windows:** add `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64`
  to `PATH` for the fuzz command only.
- **The gate logs the names of the environment variables it inherited.** Never log values.
- `wsl: Failed to start the systemd user session for 'tester'` appears on most calls into `steplet-test`. Not
  investigated; no run has failed because of it.
- `python -m py_compile` leaves `__pycache__`; it is now in `.gitignore`.
- **Windows Python cannot open `/c/...` paths** from the Bash tool; pass Windows paths (`C:\...`) to it.
- **Clean-up options for the wrap-up:** `C:\dev\m1\s9` holds the throwaway SSH private key (delete when done);
  Playwright's browsers are in `C:\Users\RANVIR\AppData\Local\ms-playwright` (`npx playwright uninstall`).
- `claude` on the test distribution's `PATH` is the Windows wrapper; the scripts only call Steplet's pinned copy by full path.

## Findings to carry into later tasks

- S3a: anything in Claude Code's environment is readable by every fenced program (`/proc/<pid>/environ` inside the
  fence). Keep secrets out of that environment.
- S3b: `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1` breaks every fenced command in the `product` shape
  (`bwrap: Can't mkdir /mnt/c/Program Files/ClaudeCode`). Do not use it.
- S1: the gate inherits `wsl.exe`'s Windows environment. The fence's process list does not show the gate or the run token.
- S3e: `claude auth status` does not show which sign-in a run uses. An API key beats a stored subscription sign-in
  in `-p` mode. S3e left one session record in `/home/ranvir/m0/claude-home/projects/` (harmless, left in place).
