## Phase 0 summary (Task 16, 10 Oct 2026)

Every verdict below comes from the sections above, where the real output is pasted. Nothing new was run for this
summary.

### Verdicts

| Spike | Verdict | What changes for later phases |
|---|---|---|
| S1: the gate as a Windows program through interop | PASS | The checker can be a Windows program (`steplet-gate.exe`) that Claude Code starts through interop; the runner-up (`--permission-prompt-tool stdio`) is not needed. A crashed gate comes back and its first request is refused; a stalled one is refused at the deadline. The gate sees only the Windows environment the app gives `wsl.exe`. |
| S2: launching Claude Code from Windows | PASS, with `SystemRoot` instead of an empty environment | **The app starts `wsl.exe` with `SystemRoot` only, never an empty environment**: an empty one fails (`Wsl/Service/0x8007072c`) and can leave a stopped distro hanging until `wsl --terminate`. Killing `wsl.exe` ends the fenced command too. Never pass the whole Windows environment. |
| S3: can a fenced program see the API key? | PASS, through `apiKeyHelper` only | The key reaches Claude Code only through `apiKeyHelper`, never through the environment or a command line (S3a: every fenced program could read an environment key). The real service accepted the key sent this way (L1). `claude auth status` cannot tell the app which sign-in a run uses. |
| S4: checkout in the fence, and a read-only `.git` | PASS (two parts inconclusive by the plan's rule) | D6 works as intended: hardened clone outside the fence, checkout inside `srt` with no agent, then the agent with `.git` read-only. Every `srt` call puts `--` before the command. If `srt` lives in the private folder, the fence must re-allow its seccomp helper folder (phase 3 choice). |
| S5: commands that skip the checker, and `--tools` | RECORDED | No setting makes every command ask the checker, so D8's harmless list stays. Without the work-folder Read rule (S5r), `ls`, `cat`, `wc`, `head`, `grep`, `echo`, `pwd` and `git status` all run without asking; the live agent also ran `ls … && find …` without asking (S6R-off). S5a–c's `ls`/`cat` were refused by that rule, not run. `--tools` limits the tools offered. |
| S6: the model switch after a refusal | RECORDED (scripted and live) | `switchModelsOnFlag: false` stops the switch, scripted and live; the stream says which happened (`model_refusal_fallback` / `model_refusal_no_fallback`, with category). A refused run reports `subtype=success`: the app must judge failure from `is_error` and `stop_reason`. Blocked fence attempts are not reported anywhere Steplet can read. |
| S7: does a scripted model change what the checker and fence see? | PASS | The phase 5 suite may use the scripted model for these actions. `allowRead` re-opens the run's copy inside a `denyRead` folder for programs in the fence. The fence blocks the agent's own checkout, so Steplet does the checkout as its own step (D6). |
| S8: the Windows side, from inside the fence | PASS, in WSL's `nat` mode only | The chooser may say "on this computer" for connections, with the condition kept in the design notes: tested in `nat` mode only; a computer in `mirrored` mode needs its own check first. |
| S9: signatures in WASM, Svelte in WebView2, the check page | PASS | Signature checking can live in a `no_std` crate with `ssh-key` 0.6.7; phase 1 can start from `s9-core`. The check page works locked with `connect-src 'none'` in three browser engines. The pinned web versions work together. |
| L1: one real-model run of the whole path | PASS | The scripted model hid nothing on this path: same fence results, every changing command asked the checker, key not leaked, one `result` event. **But the `product` folder layout cannot ship as it is:** its `Read(...)` deny rule on the work folder refuses the agent's own Read tool and `ls`/`cat` on its own copy (S6L). The stand-in `product-readable` fixes that, and leaves the checker as the only guard against the Read tool opening other runs' copies (S5x). The layout is open for phase 3. |

### Lines on screen that a spike touched (milestone plan, section 7)

| Line on screen | Spikes | May it stay as worded? |
|---|---|---|
| "Steplet checks the agent's shell commands that change something, its file writes and its web fetches before they happen. If Steplet's checker fails, the action is refused." | S1, S4, S5, L1 | **Yes**, with D8's harmless list behind it: every command that changed something asked (S5, L1); a crashed or stalled checker meant refusal (S1); Write-tool writes into `.git` were refused (S4). The commands that skip the checker are reads (S5r). Web fetches were not tested in phase 0. |
| "Programs this task runs can change files only inside this folder." | S7, S4, L1 | **Yes.** C is blocked in every product-shape run; `.git` inside the folder is also read-only (S4, L1). |
| "Programs this task runs cannot read these paths: …" | S3, S7, L1, S5x | **Yes**, for programs in the fence: B and H blocked, the key and the private folder unreadable (S3). It does not cover the agent's own Read tool, which runs outside the fence (S5x); that gap belongs to the folder layout decision, not to this line. |
| "Programs this task runs cannot connect to the internet, or to other programs running in WSL." Extended to "on this computer" only if S8 and its suite test pass. | S8, S7, L1 | **May be extended** to "on this computer", in `nat` mode only (S8), once the suite test exists. |
| "Programs this task runs cannot start Windows programs." | S1, S7, L1 | **Yes.** G stays blocked even while the Windows gate runs for the same Claude Code (S1f). |
| "Interrupted; the folder was reset; run again?" | S2 | Not settled by phase 0: S2c shows stopping `wsl.exe` ends the fenced command; the restart-clean test is not written yet. |
| Report: every model used, and refusals | S6, S6 live | **Yes.** Models come from the `result` event's `modelUsage` (L1, S6R), refusals from the two stream events. |
| New, from S6: the report must say blocked attempts are not visible to Steplet | S6 | A line to add: the fence reports nothing Steplet can read, so the report must not claim the fence stopped something. |

Not touched in phase 0: the web-address line (T2e), the prepare screen's agent-settings line (D3), "report and live log are safe to read" (T5), and the Codex line.

### Not verified in phase 0

- **One machine, once.** Windows 11 Home (build 26200), Ubuntu 26.04 in WSL 2.6.3, `nat` networking only. WSL's
  `mirrored` mode, Windows 10, Windows Pro and antivirus products are not tested. Most spikes ran once.
- **The live model, a handful of times.** L1 once; S6 live once with the switch on and once with it off; only the
  `cyber` refusal category was seen. The phase 5 suite's live layer is still needed.
- **The folder layout.** `product-readable` was checked only by S5r and S5x. Not tested: the Read tool on another
  run's copy with the checker in `deny` mode, and any layout that closes other runs' copies to the Read tool without
  the checker.
- **Whether the commands that skip the checker run inside the fence** (S5). Also whether `find` with `-exec` or
  `-delete` skips it.
- **Interop identity:** S1 records the gate's account and session through `tasklist`, not its integrity level or
  logon session, which the pipe's permission list in phase 3 depends on. How fast the gate exits when Claude Code is
  stopped was not measured.
- **`srt`:** its sub-packages are recorded, not pinned; whether a fenced command can write the default paths `srt`
  adds (`~/.npm/_logs`) was not tested; S3b (`CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1`) could not run in this layout.
- **S4's two inconclusive parts** (hook planting by command, `git config`), where milestone 0's control failed too.
- **S9:** an installed (bundled) app, another WebView2, the page served from a real web host.
- **The warning `wsl: Failed to start the systemd user session for 'tester'`** appears on most calls and never broke
  a run; not investigated.
- **The spike code is throwaway**, tested by its self-tests and the runs above, not by unit tests.
- **The test distribution is fresh**, without the tools a real developer's Ubuntu has.
- **The Anthropic terms question (P1)** is unchanged by phase 0.
