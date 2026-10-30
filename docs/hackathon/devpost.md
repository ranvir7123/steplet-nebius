# Devpost entry text

Text to paste into the Devpost submission form for the Nebius x NVIDIA Global AI Hackathon. Each heading below is one
field or one section of the "About the project" box. Fill the `TODO` parts from the real results before pasting, and
delete the `<!-- -->` notes. Every claim here must also be in the README's "What has been tested"; if a test result
changes, change both.

Submission closes **Fri 30 Oct 2026, 10:00 am Pacific time = 22:30 India time** (official rules, section 1). Aim to
submit by 22:00.

---

## Project name

Steplet

## Tagline

Run an AI coding task from a stranger on your own laptop: Nemotron proposes each move, Steplet's checker decides, a
fence runs it.

## Track

Coding and Agentic Engineering

<!-- Read the open question about this track in docs/hackathon/requirements-checklist.md before choosing. -->

## Built with

Rust, Tauri 2, Svelte 5, TypeScript, Vite, Python, WSL 2, Ubuntu 26.04, sandbox-runtime (srt), bubblewrap, WinHTTP,
Windows Credential Manager, Playwright, NVIDIA Nemotron 3 Super, Nebius Token Factory

## Links

- Code: https://github.com/ranvir7123/steplet-nebius
- Test build (the installer): https://github.com/ranvir7123/steplet-nebius/releases/tag/v0.2.0
- Video: <!-- TODO after the video is uploaded: the public YouTube link -->

---

## About the project

### What it does

A `.steplet` file carries a coding task for an AI agent: "check out this repository at this commit, run its tests,
tell me what fails". It says who sent it, the steps, what the agent may touch and what it must never do, and it can
be signed. Double-click it on Windows and Steplet shows the whole plan, who signed it, and a risk rating worked out by
fixed rules (not by an AI, so text in the file can't argue its way to a better rating). Then you pick who does the
work.

New for this hackathon: **Steplet's own agent, NVIDIA Nemotron 3 Super on Nebius Token Factory.** Nemotron proposes
each move as a tool call. Steplet's checker decides each one against what the file declared, and refuses or asks you
about anything else. Allowed moves run one at a time inside a fenced Linux that Steplet builds and owns, with no
access to your files, the internet or Windows. A refused move goes back to the model, which can try another way
inside the rules. The live view shows every move, every verdict, the exit code, the money spent and the steps used.
The report shows the exit codes from Steplet's own record next to what the model claims, so "all tests pass" can't
hide a failure.

### Why we built it

Asking an AI agent to work on someone else's code means letting two strangers loose on your laptop: the AI, which can
be talked into things by text in the files it reads, and the code, which runs when its tests run. Most coding agents
run with your own permissions, can see your home folder and saved keys, and finish by telling you how it went in
their own words. Steplet is for the person who wants the work done anyway, and wants a true account of it.

### How it uses Nemotron and Token Factory

- **The agent loop runs on Windows** and calls Token Factory's chat-completions API through Windows' own web library
  (WinHTTP). The API follows the OpenAI shape, so no SDK was needed: the Token Factory part of Steplet is 109 lines
  of its own code, tested against a stand-in server.
- **Nemotron 3 Super** (`nvidia/nemotron-3-super-120b-a12b`) is the default. Steplet also knows Nemotron 3 Ultra and
  Nano, and never calls a model it has no price for.
- **Seven tools** (`run_command`, `read_file`, `write_file`, `edit_file`, `list_files`, `find_files`, `search_text`),
  each mapped onto the tool Steplet's checker already understood, so one set of rules judges every runner.
- **Spending is capped before each call, not after.** Steplet works out the most the next call could cost
  (everything sent plus the longest answer allowed) and stops before sending if that could pass the run's $0.25 cap.
  A run also stops at 40 steps or 30 minutes.
- **Busy or failing answers** are tried again after 2, 5 and 10 seconds, each shown in the live view. A refused key or
  an empty account ends the run with plain words.
- **The key** lives in Windows Credential Manager and goes only into the `Authorization` header, never onto the screen
  or into a log or report.

- **Checked with real calls on 30 Oct:** Token Factory offers all three models under exactly the names Steplet uses,
  and each one, given a tool, called it correctly on the first try and then answered from its result.
- **"What will this file do?"** Before you run a file, the preview can ask Nemotron 3 Super to describe, in plain
  words, what running it would do and what a careful person would look at. The answer is shown as AI advice next to
  the rating; it cannot change the rating, which comes from fixed rules.

### How we built it

Steplet is a Tauri 2 app: a Rust core, with screens in Svelte. Steplet installs its own WSL 2 Linux (Ubuntu 26.04)
and runs each allowed move there as an account with no powers, inside a fence (`srt`, Anthropic's sandbox-runtime,
which uses bubblewrap), ending anything a move leaves running before the model sees its answer. Even file reads and writes run inside the fence, so a mistake
by the checker is still contained.

Steplet's first commit was 8 Oct 2026, after the hackathon opened on 26 Aug 2026. Before Nebius, it ran Claude Code
inside the same fence. For this hackathon we built Steplet's own agent: the Token Factory client, the agent loop, the
moves inside the fence, the spending cap, and the screens for them.

### What is tested

Only what a passing test or a recorded run shows. In short:

- 15 tests of the Token Factory client against a stand-in server (the key only in its header and never in an error,
  refused keys, empty accounts, busy servers, oversized replies, cost counted both ways).
- 9 tests of the moves in Steplet's real Linux (the fence holds even with the checker taken away, a command cannot
  fake its exit code, only declared sites are reachable, programs left running are ended).
- 14 tests of the loop with a stand-in model (what runs is what was checked, refusals go back to the model, the cap
  stops the run before a call that could pass it, the key never reaches the report or logs). The code was broken on
  purpose three times and these tests caught each break.

The README lists every test by name, and what is **not** verified yet.

- 10 attack tests of Steplet's agent in Steplet's real Linux, each with a control that shows the test can fail. The
  attack suite replays them as ten cases next to the 39 cases it runs for Claude Code: on 30 Oct, **49 of 49 blocked,
  nothing left behind**, with a scripted model (the record is in the repository).
<!-- TODO after Task 10: the real runs with Nemotron: what happened, refusals, exit codes, what each run cost. -->

### Challenges

<!-- TODO after Task 10: fill from what actually happened in Tasks 4 to 10. Candidates already recorded:
- A Stop in the first milliseconds of a run could leave Steplet's Linux unable to look up web addresses (a fault
  older than this copy, found while testing the agent; see docs/nebius-hackathon-results.md, Task 6).
- The fence program keeps some folders read-only, so a move's temporary folder had to be the run's own
  (docs/decisions.md, N17 and the rows after it). -->

### What's next

<!-- TODO: Ranvir's words. Keep to what is planned, not promised. -->

---

## Feedback on Nebius Token Factory and NVIDIA Nemotron

Written from [docs/nebius-feedback.md](../nebius-feedback.md), which records only what was actually seen, with the
date.

**Nebius Token Factory**

- (29 Oct 2026, helped) The API follows the OpenAI chat-completions shape (`/v1/chat/completions`, `tools`,
  `tool_choice`, `usage`). Steplet talks to it with Windows' own web library and its own code (the Token Factory
  part is 109 lines); no Nebius-specific package or SDK was needed.

- (30 Oct 2026, helped) A wrong or missing key comes back as HTTP 401 with a short reason, so Steplet can tell "the
  key is wrong" apart from every other failure and say so in plain words.
- (30 Oct 2026, got in the way, a little) The list of models also needs a key, so an app cannot check a model name is
  offered before its owner has typed a key in.
- (30 Oct 2026, got in the way) Applying the hackathon's promo code failed at first with "Act for contract(...) and
  period(...) not found", which says nothing the person can act on.
- (30 Oct 2026, helped) The models list showed 30 models, 5 of them Nemotron, under exactly the documented names.
  Every reply carried token counts in `usage`, so Steplet counts each call's cost as it arrives. Short calls on Super,
  Ultra and Lightning answered in 0.6 to 2.2 seconds.

**NVIDIA Nemotron**

- (30 Oct 2026, helped) Given one tool and asked to list a folder, all five Nemotron models on Token Factory called
  it correctly on the first try, then answered from its result without calling it again. The tool-call shape is the
  standard one, so Steplet's loop needed nothing model-specific.
- (30 Oct 2026, helped) Nemotron 3 Super's thinking comes back in its own field, apart from the answer, so Steplet
  shows only the answer and the moves.
- (30 Oct 2026, got in the way, a little) Nemotron 3 Nano used 1,721 tokens, most of them thinking, for a one-line
  answer (6.5 seconds). Still the cheapest, but a cap counted in tokens has to leave room for thinking that long.
