# Demo video script (at most 2:45)

For the Devpost entry. Recorded on Fri 30 Oct between 16:00 and 18:00 IST, by one person, on this laptop, from the
real app in dark mode. No music. Captions on every spoken line (the caption text is the spoken text below).

The spoken lines come to 270 words (counted with a script, taking the longer choice wherever there are two). At an
easy 130 words a minute that is about 2:05, which leaves room for pauses while the screen shows something. No shot
needs faster than 130 words a minute. If a take runs long, cut the words in [square brackets] first.

## Before you record

- **The key must never be on screen.** Save the Nebius key in Setup before recording. Setup then shows only that a key
  is saved. Do not open Credential Manager, a terminal, the Nebius dashboard's key page, or any file with a key in it.
- Turn on Do not disturb, so no notification shows up in the recording. Close every other window.
- Set the Steplet window to 1100 pixels wide, so the text is easy to read in a small video.
- Have ready: the sample file `examples/sample-run.steplet`, saved in Downloads as if it came from someone else; and a
  Nebius account with credit.
- Record the screen first, one shot at a time, with Windows' own recorder (the Snipping Tool's record button, or the
  Xbox Game Bar with Win + Alt + R). Record the voice afterwards, reading this script, while watching the clips.
  Then put the clips and the voice together and add the captions, for example in Clipchamp, which comes with
  Windows 11.
- After editing, watch the whole video once, frame by frame where text is on screen, to check no key, email address
  or private file name shows.
- The rules require: under 3 minutes, uploaded to YouTube and publicly visible (set it to Public), footage of the
  app working on the device it was built for, and no third-party trademarks or copyrighted music without permission.

## Shot list

The times are where each shot starts and ends in the finished video.

### 1. A file from a stranger (0:00 to 0:15)

**Screen:** the Downloads folder with `sample-run.steplet` in it. The mouse moves to it.

**Say:** "Someone sends you a file. It asks an AI to run their code's tests and tell you what fails. That means an
AI and a stranger's code, both on your laptop."

### 2. The preview (0:15 to 0:40)

**Screen:** double-click the file. Steplet's preview opens. Move slowly over: who signed it, the rating, the steps,
what it may touch. *Then, only if Task 11 shipped:* click "Ask Nemotron what this file will do", and show its answer
in the box marked as AI advice, next to the rating, which does not change.

**Say:** "Steplet opens it first. Who signed it, a rating from fixed rules, the steps, and what it may touch.
*(If Task 11 shipped:)* Nemotron can explain the file too, but its answer is marked as advice, and it can't change the
rating."

*If Task 11 did not ship,* hold the preview longer and say instead: "Fixed rules decide the rating, not an AI, so text
in the file can't talk its way to a better one."

### 3. The chooser (0:40 to 0:55)

**Screen:** click to continue. The run chooser: "Steplet's agent", "Nemotron 3 Super, on Nebius Token Factory",
"Stops at $0.25 or 40 steps", and the lines under it. Point at the cap line. Click the main button. Frame or crop
the shot so only Steplet's agent's card shows: the second card names another company's product, and the rules ask
for no third-party trademarks in the video.

**Say:** "I choose Steplet's own agent: Nemotron 3 Super, on Nebius Token Factory. It stops at twenty-five cents or
forty steps, whichever comes first."

*Depends on Task 8 (the chooser).* Read the card's words off the real screen, and change this line if they differ.

### 4. The live run, and a refusal (0:55 to 1:40)

**Screen:** the live list fills up: what the model said, each tool it asked for, Steplet's verdict, the output, the
exit code. At the top: the model, money spent of $0.25, steps used of 40. Pause the video on a refused move, then on
the model's next move, which reaches the goal another way.

**Say:** "Every move Nemotron asks for goes through Steplet's checker first. Allowed moves run inside a fence, in
Steplet's own Linux, one at a time, with no access to my files or the internet. Here the model asked for something
the file never declared. Steplet refused, and told the model. So it found another way, inside the rules. [Up top: the
money spent, and the steps used.]"

*Depends on Task 10 (real runs).* Use a refusal that really happened in a real run. If the run you record has no
refusal, do not stage one: use the recorded run from Task 10 that has one, or cut the three refusal sentences and
say "In this run, every move was inside the rules." instead.

### 5. A question (1:40 to 1:55)

**Screen:** a command the file did not declare. The question appears: "Allow / Refuse", with the 5-minute timer.
Click Refuse.

**Say:** "When a command is outside what the file declared, Steplet asks me. No answer in five minutes means no."

*Depends on Tasks 7 to 10.* Only record this if it happens in a real run; otherwise cut the shot and give its 15
seconds to shot 6.

### 6. The report (1:55 to 2:20)

**Screen:** the report. Show, in order: finished or not, and why; each command with Steplet's verdict and the exit
code from Steplet's own record; the files changed; the model; tokens and money spent; the agent's summary, marked as
its claim.

**Say:** "The report shows what really happened. Every command, Steplet's verdict, and the exit code from Steplet's
own record, right beside what the model claims. So if it says 'all tests pass' when they didn't, you see it.
[And here is the cost of the whole run.]"

*Depends on Task 7 (the report).* Read the spend off the real report, and do not round it down.

### 7. How Token Factory is used (2:20 to 2:40)

**Screen:** the picture from the README ("Nemotron proposes, the checker decides, the fence runs"), shown full
screen for the whole shot, as a plain picture (the README file open in a plain editor in dark mode, zoomed in), not a
web page with another company's logo on it.

**Say:** "Under the hood: Steplet talks to Token Factory's OpenAI-style API straight from Windows. Nemotron answers
with tool calls, seven tools in all. Before every call, Steplet works out the most it could cost, and stops before
the cap."

### 8. The end (2:40 to 2:45)

**Screen:** the repository's address and the words "Steplet, Apache 2.0".

**Say:** "Steplet. Open source, on GitHub."

## What this video must not say

The README and the video say only what was tested (see the README's "What has been tested" and "What is not
verified"). So:

- Do not say Steplet is "safe" or "secure" in general, or that it stops "any attack". Say what it did in this run.
- Do not say the cap can never be passed. Say it stops before a call that could pass it.
- Do not show or describe any attack test from the suite, only the real run.
- Do not say it works on every Windows laptop. It has been tested on one.
