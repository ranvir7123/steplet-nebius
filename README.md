# Steplet

**Run an AI coding task from a stranger on your own Windows laptop, without letting the AI or the stranger's code near
your files, your keys or your network.**

A `.steplet` file is one text file that carries a task for an AI agent, such as "check out this repository at this
commit, run its tests and tell me what fails". It says who sent it, the steps, what the agent may touch and what it
must never do, and it can be signed. You double-click it, Steplet shows you the whole plan and a risk rating worked
out by fixed rules, you pick who does the work, and the work happens inside a fenced Linux that Steplet builds and
owns. Every move the AI asks for is checked by Steplet before it runs, and the report at the end shows what really
happened, not what the AI says happened.

This copy of Steplet is the entry for the **Nebius x NVIDIA Global AI Hackathon, Coding and Agentic Engineering
track**. What is new in it: **Steplet's own agent**, driven by **NVIDIA Nemotron 3 Super on Nebius Token Factory**.

<!-- TODO after Tasks 7 and 8: screenshots, dark mode, at 1100 px wide: preview, run chooser, live run, question,
report. Each picture checked by eye for keys before it is added (see docs/hackathon/secret-scan.md). -->

## The problem

Asking an AI agent to work on code you got from someone else means letting two strangers loose on your laptop: the
AI, which can be talked into things by text in the files it reads, and the code itself, which runs when its tests
run. Most agents run with your own permissions, see your home folder and your saved keys, can reach the internet and
this computer's other programs, and finish by telling you how it went in their own words.

Steplet's answer has three layers that do not trust each other:

1. **Fixed rules, not an AI, rate the file** before you agree to anything.
2. **A checker decides each move** the agent asks for, against what the file declared. Anything the file did not
   declare is refused, or you are asked.
3. **A fence contains whatever runs**, so a move the checker wrongly allowed still cannot reach your files, the
   internet or Windows.

## Steplet's own agent: Nemotron proposes, the checker decides, the fence runs

```
Windows: the Steplet app                         Steplet's Linux (one move at a time)
  Nemotron on Token Factory  <- HTTPS ->           root script: close shared folders, check layout
  (key from Credential Manager)                    -> steplet account, every power dropped
  checker decides each move   -- allowed move ->   -> fence (srt) -> the one move
  live view, report, cap      <- output + exit --  -> end everything still running
```

- **The loop runs on Windows.** Steplet sends the task and the conversation so far to Nemotron through Token
  Factory's chat-completions API, which follows the OpenAI shape, so no Nebius package was needed. Steplet uses
  Windows' own web library (WinHTTP). Nemotron answers with tool calls.
- **Seven tools**, each mapped to the tool Steplet's checker already knows from Claude Code: `run_command` (Bash),
  `read_file` (Read), `write_file` (Write), `edit_file` (Edit), `list_files` and `find_files` (Glob), `search_text`
  (Grep).
- **Each tool call goes through the checker first.** Allowed moves run in Steplet's Linux, one at a time, as an
  account with no powers, inside the run's fence, with a time limit. File tools run inside the fence too, not just
  commands, so a checker mistake is still caught by the fence. What a move leaves running is ended before its answer
  goes back to the model. A refused move goes back to the model as a refusal, and it can try another way.
- **The fence** is `srt` (Anthropic's sandbox-runtime, which uses bubblewrap), run inside Steplet's own WSL Linux.
- **The model sees** the first and last 4 KB of each move's output, and the exit code from Steplet's own record.
- **Limits per run:** $0.25 of spending, 40 steps, 30 minutes, 10 minutes per move, 4,096 tokens per answer. Before
  each call Steplet works out the most that call could cost (everything sent plus the longest answer allowed); if that
  could pass the cap, the run stops without sending. Busy or failing answers from Nebius are tried again after 2, 5
  and 10 seconds, and each try is shown.
- **The key** is kept in Windows Credential Manager, never shown on screen, and sent only as the `Authorization`
  header.

The models Steplet knows, with the prices it counts in (US dollars per million tokens, sent / received):

| Model | Name on Token Factory | Sent | Received |
|---|---|---|---|
| Nemotron 3 Super (the default) | `nvidia/nemotron-3-super-120b-a12b` | 0.30 | 0.90 |
| Nemotron 3 Ultra | `nvidia/Nemotron-3-Ultra-550b-a55b` | 1.00 | 3.00 |
| Nemotron 3 Nano | `nvidia/NVIDIA-Nemotron-3-Nano-30B-A3B` | 0.06 | 0.24 |

A model with no price in this table is never called.

**Where Token Factory helped:** because its API follows the OpenAI chat-completions shape, Steplet needed no SDK or
extra package. The Token Factory part is 109 lines (`crates/steplet-run/src/tokenfactory.rs`), on top of Steplet's
own small layer over Windows' web library (`winhttp.rs`) and the chat message shapes (`model.rs`), and it is tested
against a stand-in server. No other Nebius service is used.

On 30 Oct 2026 Token Factory's list of models offered all three under exactly these names, and each one answered a
real tool call correctly (the output is in [docs/nebius-hackathon-results.md](docs/nebius-hackathon-results.md),
"Task 4"). Two more Nemotron models are offered, `nvidia/Nemotron-3_5-Lightning` and `nvidia/Nemotron-3.5-Super-VL`;
they have no price here, so a run never uses them. The prices are the plan's starting prices: they were not checked
against the Nebius dashboard.

**Ask Nemotron what a file will do.** Before you run a `.steplet` file, the preview has a button that sends the file's
text to Nemotron 3 Super: one call, no tools, at most 1,500 tokens back. Nemotron is told to describe what running the
file would do and what a careful person would look at, in plain words, and not to judge whether it is safe. Its answer
is shown in a box marked as AI advice, with the model's name and what the call cost. It is advice only: the red, amber
or green rating above it comes from Steplet's fixed rules, and the answer has no way to change it. Tests:
`crates/steplet-app/tests/explain.rs` (7, with a stand-in model; among them `the_advice_cannot_change_the_rating` and
`the_file_text_goes_in_after_the_instruction`) and 6 in `web/app/tests/preview.spec.ts`. The key, the model and the
price table are the same as the agent's.

## What has been tested

Only what a passing test or a recorded run shows is listed here. Test names are in the code; the outputs are in
[docs/nebius-hackathon-results.md](docs/nebius-hackathon-results.md) and [docs/suite/](docs/suite/).

### Steplet's own agent

Talking to Token Factory (`crates/steplet-run/tests/tokenfactory.rs`, 15 tests against a stand-in server on this
laptop, no real calls):

- The key goes only in the `Authorization: Bearer` header, and never into an error message:
  `sends_the_key_as_a_bearer_header_and_nowhere_else`, `the_key_never_appears_in_an_error`.
- A refused key, an empty account, a busy service and a server fault are told apart, so the screen can say which:
  `a_refused_key_is_key_refused`, `no_credit_is_said_as_such`, `busy_and_server_errors_are_told_apart`.
- Only HTTPS addresses (or this laptop, for the tests) are accepted: `only_https_or_this_laptop_is_accepted`.
- Silent servers and oversized replies end the call: `a_server_that_goes_silent_is_too_slow`,
  `an_oversized_reply_is_refused`.
- Cost is counted both ways, and the worst case is never below a real reply's cost: `cost_counts_both_directions`,
  `worst_case_is_at_least_the_cost_of_any_reply_within_max_tokens`.

The moves, inside the fence (`crates/steplet-run/tests/hands.rs`, 9 tests run in Steplet's real Linux):

- `each_file_tool_works_inside_the_copy`, `a_command_runs_in_the_copy_and_its_exit_code_is_steplets`.
- A command cannot fake its exit code: `a_command_cannot_forge_its_exit_code`.
- The fence holds even with the checker taken away: `the_fence_holds_without_the_checker`.
- A command reaches only the sites the file declares: `a_command_reaches_only_declared_sites`.
- A program left running is ended before the next move, and a move past its time limit is ended:
  `a_program_left_running_is_ended_before_the_next_move`, `a_move_past_its_time_limit_is_ended`.
- Huge output is cut, and the run folder is out of the fence's reach: `huge_output_is_cut`,
  `the_run_folder_is_out_of_the_fences_reach`.
- The agent's fence follows the same rules and sites as the fence Steplet already uses for Claude Code:
  `the_hands_fence_matches_claude_codes_fence`.

The loop (`crates/steplet-run/tests/agent.rs`, 14 tests with a stand-in model and stand-in moves):

- What runs is exactly what the checker saw, and every call goes through the checker: `what_runs_is_what_was_checked`,
  `every_call_goes_through_the_checker`.
- A refusal goes back to the model; a malformed tool call is answered, not run: `a_refusal_goes_back_to_the_model`,
  `a_malformed_tool_call_is_answered_not_run`.
- The report shows Steplet's exit code, not the model's claim:
  `the_report_shows_steplets_exit_code_not_the_models_claim`.
- The spending cap ends the run before the call that could pass it; the step and time limits end the run:
  `the_spending_cap_ends_the_run_before_the_call_that_could_pass_it`, `the_step_limit_ends_the_run`,
  `the_time_limit_ends_the_run`. The app hands the loop $0.25 and 40 steps
  (`crates/steplet-app/tests/session.rs`, `start_run_passes_the_runner_and_model`).
- The Nebius key never appears in the report or the logs: `the_nebius_key_never_appears_in_the_report_or_logs`.
- Busy answers are tried again and shown; three unreadable replies in a row end the run; Stop works during a model
  call: `busy_nebius_is_retried_and_shown`, `three_unreadable_replies_in_a_row_end_the_run`,
  `stop_during_a_model_call_ends_the_run_promptly`.

The loop's code was also broken on purpose three times (a refused move run anyway, the cap never checked, the key
not hidden from the screen). Each time the tests above that guard it failed, and the code was then put back.

The attack tests, on Steplet's agent in Steplet's real Linux (`crates/steplet-run/tests/agent_linux.rs`, 10 tests,
each with a stand-in model playing a fixed script of moves, a made-up Nebius key, and a control that shows its check
can see a failure):

- The Nebius key never reaches Steplet's Linux, before, during or after a run. The test counts the key's text in every
  file and every program in Steplet's Linux, and finds it 0 times; its control plants it and finds it:
  `a2_the_key_never_reaches_linux`.
- Anything one move leaves running is ended before the next move; its control shows the same program keeps running
  outside the fence: `a7_a_program_left_running_is_ended_before_the_next_move`.
- The others: a command the file did not declare is refused and the agent is told; writes outside the working copy,
  links out of it, reads of Windows files, sites the file does not declare and starting Windows programs are all
  contained; Stop during a move ends it; use case 5's public test file runs and each escape is contained.

Steplet's attack suite replays these as ten cases (G14, F4, F5, Y5, N4, P2, I3, U3, K1, E1). On 30 Oct 2026 it ran
them with Claude Code's 39 cases: **49 of 49 blocked, nothing left behind**
([record](docs/suite/2026-10-30-claude-2.1.220-and-steplet-agent-scripted.txt)). The run chooser shows two lines that
are Steplet's agent's own, each naming the tests above:

- Your Nebius key stays on Windows: it never goes into Steplet's Linux, the report or the logs.
- Anything the agent leaves running is ended before its next move.

### The real runs with Nemotron (30 Oct 2026)

Three real calls to Nemotron 3 Super on Nebius Token Factory, with the real key read from Windows' Credential Manager,
the runs in Steplet's real Linux. A test program played the screen
(`crates/steplet-app/tests/nemotron_live.rs`): it drives the app's own session, the same code the **Run with
Steplet's agent** button starts, and answers No to any question. Nobody clicked through the app for these. The full
output of each is in [docs/real-runs](docs/real-runs).

| Run | What Nemotron did | What Steplet did | Turns, time, tokens, cost |
|---|---|---|---|
| **Ask Nemotron what this file will do**, on `examples/sample-run.steplet` | Said the file runs the one allowed command once and reports, and changes nothing, installs nothing and uses no network; listed the repository, the commit, the command and those limits as the things to look at | Shown as advice | 1 call, 3.6 s, $0.00057 |
| `examples/sample-run.steplet` | Asked to run `python3 -m unittest discover -s tests -t .`, then said "5 tests ran and all passed." | Allowed it, because the file declares it. Exit code 0; Python's own output says "Ran 5 tests" and "OK" | 2 turns, 40 s, 3,799 tokens sent and 100 received: about $0.0012 |
| A copy of that file, unsigned, whose text also asks for `python3 --version`, which it does not allow | Ran the tests, then asked for `python3 --version`. Told it was refused, it finished and said it could not report the Python version | Allowed the tests (exit code 0). Asked the person about `python3 --version` ("the file does not ask for this command"), got No, refused it and told the model | 3 turns, 19 s, 5,844 tokens sent and 562 received: about $0.0023 |

- The times run from start to report, cloning the repository included. The costs are worked out from the token
  counts at the price table's prices ($0.30 per million tokens sent, $0.90 per million received); the screen showed
  "less than $0.01 of $0.25" for both runs.
- The Nebius key was found 0 times in each run's logs.
- In both runs the report lists three new files: Python's own cache files (`__pycache__`), which Python writes when
  tests run, although the file says "Do not change any file". The report shows them; it does not hide them.
- Not run with real Nemotron: use case 5's public test file and the attack suite's ten agent cases, which use a
  scripted model.

### Claude Code, the other runner

The chooser also offers Claude Code (version 2.1.220, pinned), run inside the same Linux. Its attack suite ran 39
cases on 29 Oct 2026, on commit `a8ce633`, all blocked and nothing left behind
([record](docs/suite/2026-10-29-claude-2.1.220-scripted.txt)). Among the lines it backs, as they read on screen:

- Steplet checks the agent's commands that change something, its file writes and its web fetches before they happen.
- The agent can fetch only the web addresses the file declares, and another one only if you say yes to that exact
  address.
- The agent and the programs this task runs can change files only inside the working copy.
- Your own files are not inside Steplet's Linux, where this task runs.
- Programs this task runs cannot connect to the internet, or to other programs on this computer.
- Programs this task runs cannot start Windows programs.
- The report and the live log are safe to read.

That suite used a scripted stand-in for the model. In its two cases with the real model (28 and 29 Oct), the model
never ran the project's tests, so those cases show nothing either way.

## What you need

- **Windows 11** with **WSL 2** (Windows' built-in way to run Linux). If WSL is missing or too old, Setup says which
  command to type.
- **A Nebius Token Factory key** for Steplet's own agent. Setup has a box for it and says where to get one.
- For Claude Code instead: an Anthropic key.

- **Disk space:** on this laptop Steplet's Linux takes 4.74 GB, the size of its disk file
  (`%LOCALAPPDATA%\Steplet\wsl\ext4.vhdx`) on 30 Oct 2026, after two days of test runs in it. A fresh install was not
  measured, so take this as a rough guide, not a minimum.

## Install and run

The release is [v0.2.0](https://github.com/ranvir7123/steplet-nebius/releases/tag/v0.2.0), built from commit
`d994368` with `tools\release.ps1`:

```
2483f45940d29729e5bb602e2e1723fb29ee6de29ca67709e4cd1141de85bcb4  Steplet_0.2.0_x64-setup.exe
f5bf573280d9db4a0c78dc8cfa2a8e1f0fccc5d020e17a4cabad297386967773  Steplet_0.2.0_x64.zip
```

The zip holds the same two programs and the licence, without an installer.

1. Download `Steplet_0.2.0_x64-setup.exe` from the release and run it. It installs for your Windows account only,
   into `%LOCALAPPDATA%\Steplet`, without admin rights.
2. Windows will warn "Windows protected your PC" / unknown publisher, because the app is not signed with a paid
   certificate. Choose **More info**, then **Run anyway**.
3. Open Steplet. The first time, Setup builds Steplet's own Linux and asks for your Nebius key.
4. Choose **Open a file…** and pick a `.steplet` file. To try one: [examples/sample-run.steplet](examples/sample-run.steplet).
   To open them with a double-click instead, turn on **Open .steplet files with Steplet** in Setup (it starts off).

<!-- TODO after Task 12: steps 1 to 4 checked on this laptop from the installed app (use case 1). -->

## Build from source

You need Rust 1.98 or newer, Node.js with npm, and Windows 11 with WSL 2.

```
npm --prefix web ci
npm --prefix web run build
cargo build -p steplet-app -p steplet-gate --release
```

The page must be built before the Rust app, because the app carries it inside the program. The two programs
`steplet.exe` and `steplet-gate.exe` land in `target\release\`, and must stay side by side.

<!-- TODO after Task 12: run these three commands from a clean checkout and paste their result; the release build
(installer and zip) is Task 12's. -->

Tests: `cargo test --workspace --offline` and `npm --prefix web test`. The tests marked ignored need Steplet's Linux,
or a real key and money; they are run on purpose, one at a time.

## History

Steplet's first commit was 8 Oct 2026, after the hackathon opened on 26 Aug 2026. Its whole history is kept in this
repository, including what was built before the Nebius agent: the file format, the signing, the rating rules, the
checker, Steplet's Linux and the Claude Code runner. What was built for the hackathon is the Nebius agent and its
screens, from 29 Oct 2026; see [docs/plans/2026-10-29-nebius-hackathon.md](docs/plans/2026-10-29-nebius-hackathon.md).

## What is not verified

- **The real runs were started by a test program, not by a person in the app.** The two whole runs and the one
  "what will this file do" call above went through the app's own session code with real Nemotron, but nobody clicked
  through the screens for them. Use case 5's public test file and the attack cases have not been run with real
  Nemotron.
- **The prices are not checked against Nebius's bill.** Real calls were made on 30 Oct (six models, at most $0.0073
  counted, then about $0.004 more for the three real runs above), and the model names are confirmed, but nobody
  compared Steplet's count with the dashboard's charge. The cap is only as good as the price table.
- **Two faults found while building were repaired, with one gap left untried.** A Stop pressed in the first moments
  of a run could leave Steplet's Linux unable to look up web addresses until it restarted (a gap a few milliseconds
  wide, in Steplet before this copy); now Steplet keeps its own copy of the lookup file and puts it back at every
  start. One Linux test's time margin was too thin and failed once; it now has ten times the room. The tests break
  lookup the way that gap does, with the same commands, but a real Stop timed into those few milliseconds was not
  tried.
- **The build steps above** are taken from earlier results; they were not run again for this draft.
- **Only on this laptop:** Steplet has been tested on one Windows 11 laptop. A Windows without WSL has been tried
  only through a test of the version reader, not on a real machine.
- **Claude Code's live suite cases** with a real model showed nothing either way; the suite above is scripted.

<!-- TODO after Task 12: update this list. -->

## Licence

Apache License 2.0. See [LICENSE](LICENSE).
