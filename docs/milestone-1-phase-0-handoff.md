# Handoff: milestone 1, phase 0, tasks 1 to 3 done (8 Oct 2026, evening)

Read `docs/handoff.md` first, then this file, then the plan `docs/plans/2026-10-08-milestone-1-phase-0.md`.
The user approved the plan on 8 Oct 2026 and chose to run it **inline in one session, not through separate
agents**, because almost every task needs a yes or a step from them. Each install still gets its own yes.
The user asked to continue in a new session after Task 3.

## Where things stand

| Task | Status |
|---|---|
| 1. git repository (P6) | **Done.** First commit `18bc9b0`. At the user's request I moved `keys.txt` and `sk.json` to `C:\Users\RANVIR\runfile-secrets\` without opening them; both are also in `.gitignore`. |
| 2. Rust tools on Windows | **Done.** `wasm-bindgen 0.2.129`, `cargo-fuzz 0.13.2`, nightly `rustc 1.101.0-nightly (1d81eb4ad 2026-10-07)`, targets `wasm32-unknown-unknown` and `wasm32v1-none`. Log: `C:\dev\m1\logs\setup-windows.txt`. Fuzzing works only with the AddressSanitizer folder on `PATH` (trap below). |
| 3. Test distribution (P5) | **Done.** `steplet-test`, user `tester`, Ubuntu 26.04.1 LTS, Node 22.22.1, bubblewrap 0.11.1, socat, ripgrep, git 2.53.0, python 3.14.3. `BWRAP_OK`, networking `nat`, interop sockets `1_interop` and others. GUID `{7d77bf1b-4ab3-40e7-aa63-4241953a1a25}` in `C:\dev\m1\distro-id.txt`. Log: `C:\dev\m1\logs\setup-distro.txt`. |
| 4. Steplet's folders, Claude Code, srt, spike data | **Not run. The user said yes to A4 on 8 Oct 2026** and asked for it to run in the new session. |
| 5. Spike harness | **Code written and committed; self-tests pass** (scripted model 8 of 8 on Windows and in `steplet-test`; gate and pipe 6 of 6). Logs: `C:\dev\m1\logs\selftest-fake-model.txt`, `selftest-gate.txt`. **Left:** Step 9 (append `spikes/results/setup.md` to `docs/milestone-1-spikes.md`; it needs Task 4's fake keys) and Step 10's commit. |
| 6 to 16 | Not started. |

Never run yet: `spikes/wsl/spike.py`, `services.sh`, `launch_inside.sh`, `spikes/tools/run.ps1`, `append_results.py`,
and the `launch` path of `spike-host.exe`. Their first run is Task 5 Step 9 and S7a.

## Next, in order

1. **Task 4** (A4 already approved):
   ```powershell
   $env:WSL_UTF8 = "1"
   & wsl.exe -d steplet-test --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/setup_steplet.sh 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 C:\dev\m1\logs\setup-steplet.txt
   Get-Content C:\dev\m1\logs\setup-steplet.txt
   ```
   Check the expected lines in the plan (Task 4, Step 3).
2. **Task 5, Step 9 and 10:** `python -I C:\dev\runFile\spikes\tools\append_results.py C:\dev\runFile\spikes\results\setup.md C:\dev\runFile\docs\milestone-1-spikes.md`, then commit.
3. **Task 6 (S7):** create `spikes/scripts/s7_t1c.json`, `spikes/runs/S7a.json`, `spikes/runs/S7b.json` from the plan; run `run.ps1 -Id S7a`, then `S7b`.
4. Then Tasks 7 onward as the plan orders them. The next yes needed: A5 (S3e, Task 9), A6 (Task 14), A7 and A8 (Task 15).

## Traps hit in this session (do not rediscover them)

- **PowerShell 5.1 puts a byte-order mark (`EF BB BF`) in front of text it pipes into a program**, even when
  `$OutputEncoding` says US-ASCII with no preamble. Never feed data into the gate or any program through a
  PowerShell pipe; write a file and use `Start-Process -RedirectStandardInput` (as `selftest_gate.ps1` does).
- **`cd` in the Bash tool or `Set-Location` in the PowerShell tool moves the session's working directory.** Use absolute paths.
- **Set `$env:WSL_UTF8 = "1"` before calling `wsl.exe`**, or its own output comes back as UTF-16 with spaces between letters.
- **Fuzzing on Windows:** add `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64`
  to `PATH` for the fuzz command only. Without it: `0xc0000135, STATUS_DLL_NOT_FOUND`; with `--sanitizer none`
  it does not link (`__start___sancov_cntrs`).
- **The gate logs the names of the environment variables it inherited.** Started from this Claude session, that
  list holds this session's names (for example a GitHub token's name). The self-test now prints the count only.
  In spike runs, under interop, the names are the measurement; never log values.
- Git warns "LF will be replaced by CRLF" for documents (global `core.autocrlf`). Harmless; `.gitattributes`
  keeps `.sh`, `.py` and `.mjs` files on Linux line endings, and every spike script was checked (0 CR bytes).
- `claude` on the test distribution's `PATH` is the Windows wrapper (`/mnt/c/Users/RANVIR/AppData/Roaming/npm/claude`).
  The scripts only ever call Steplet's pinned copy by full path.

## Changes to the plan made while running it

- `spikes/tools/selftest_gate.ps1` hands the gate a file instead of a PowerShell pipe, and hides inherited
  variable names. The plan's copy was updated to match.
- The plan's status line records the approval and the inline choice.
