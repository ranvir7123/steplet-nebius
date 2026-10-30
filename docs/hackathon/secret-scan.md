# Secret scan of the whole history

Before the history goes public (decision N6 keeps all of it), two scripts checked that no working key is anywhere in
it: not in today's files, not in an old version of a file, not in a commit message. This page has their real output.
No key, and no part of one, is written here.

- `tools/secret_scan.py` reads every version of every file and every commit message, and lists every string that is
  shaped like a key or looks random.
- `tools/count_saved_keys.py` (decision N8) reads the keys saved in Steplet on this laptop from Windows Credential
  Manager and counts how many times each one appears in the history. It prints counts only.

Both use Python 3.13's standard library only (nothing downloaded). Run on 30 Oct 2026 at 06:03 IST, at commit
`2b22b56`, from the worktree `docs-release-prep-be42e6`.

## Result in one line

**No hit is classed real, and the saved Anthropic key appears 0 times. The Nebius key was not saved yet, so its count
is still to do.**

| Class | Distinct hits | What they are |
|---|---|---|
| real | 0 | — |
| made-up marker | 16 | 14 Anthropic-shaped test keys and 2 Bearer lines, each with "MADE-UP", "made-up" or "FAKE" in it |
| test value | 98 | SSH signatures and public keys in the signing tests and samples, key fingerprints, GitHub's signature on one merge commit, one recorded model reply's thinking signature |

What the classes mean:

- **real**: anything the rules cannot explain. It counts as real until a person has looked at it.
- **made-up marker**: a string written to look like a key but that works nowhere. It says so in its own text, or it is
  a run of one repeated character.
- **test value**: public by design. A signature or a public key only proves who signed something; it opens nothing.
  A fingerprint is a short summary of a public key.

## 1. The scan of every file version and message

### Does the scan catch what it should?

Before trusting a "0 real", the scan was tested on keys planted on purpose. The self-test makes a throwaway git
repository, writes one random made-up key of each shape into a file and one into a commit message, deletes the file
so the keys live only in the history, and checks that each is found and classed real. It also plants a key marked
made-up (must be classed made-up marker) and a long folder path (must not be a hit). The planted keys are made fresh
from random letters each run, so they work nowhere, and the script's own text holds nothing key-shaped.

```
> py -3.13 -I tools/secret_scan.py --self-test --temp-dir <scratch folder>
PASS: Anthropic key (sk-ant-), line 1: expected real, got real
PASS: sk- key, line 2: expected real, got real
PASS: GitHub token (ghp_ and kin), line 3: expected real, got real
PASS: GitHub token (github_pat_), line 4: expected real, got real
PASS: AWS key id (AKIA, ASIA), line 5: expected real, got real
PASS: Google key (AIza), line 6: expected real, got real
PASS: Slack token (xox), line 7: expected real, got real
PASS: private key block, line 8: expected real, got real
PASS: long Bearer token, line 9: expected real, got real
PASS: JSON web token, line 10: expected real, got real
PASS: long random-looking string, line 11: expected real, got real
PASS: Anthropic key (sk-ant-), line 12: expected made-up marker, got made-up marker
PASS: long random-looking string, line 13: expected no hit, got no hit
PASS: GitHub token in a commit message: expected real, got real
PASS: no hit outside the planted lines (0 found)
Self-test: PASSED; the throwaway repository is deleted.
exit 0
```

A self-test that cannot fail proves nothing, so it was broken on purpose twice, in memory only (the file was not
changed), and run again:

```
--- BREAK 1: the GitHub token shape is dropped
FAIL: GitHub token (ghp_ and kin), line 3: expected real, got no hit
FAIL: GitHub token in a commit message: expected real, got no hit
Self-test: FAILED (2); the throwaway repository is deleted.
self-test exit 1
--- BREAK 2: every hit is called a test value
FAIL: Anthropic key (sk-ant-), line 1: expected real, got test value
  ... (one FAIL line for each of the 13 planted hits)
Self-test: FAILED (13); the throwaway repository is deleted.
self-test exit 1
```

### The real run

```
> py -3.13 -I tools/secret_scan.py
Refs scanned (everything `git rev-list --all` reaches):
  refs/heads/claude/docs-release-prep-be42e6 2b22b56
  refs/heads/claude/task7-report-ab48a6 2b22b56
  refs/heads/claude/task8-chooser-60e1f8 2b22b56
  refs/heads/nebius-hackathon 2b22b56
Objects read: 138 commits, 1018 folder lists, 2412 file versions, 0 tags; 102,425,479 bytes of file versions and messages.
File versions not read as text (not text, by git's own test): 1 .ico 446 .png 1 .pyc
Long strings not counted as hits: 8 character sets written out; 7318 hex only (hashes, commit ids, UUIDs); 550 names made of words (paths, folder and test names); 176 package hashes in lock files;

Hits: 114 distinct (the same text at the same path counts once, however many versions): 0 real, 16 made-up marker, 98 test value
  made-up marker  Anthropic key (sk-ant-)        14
  made-up marker  long Bearer token              2
  test value      long random-looking string     98
```

Exit code 0 (no hit classed real). The full list of 114 hits, one line each with its class, rule, first commit, path,
line, length and reason (never the text itself), is printed by the same command. The made-up markers:

| First added | Where | Why it is made-up |
|---|---|---|
| d06ac91, 8 Oct | `docs/plans/2026-10-08-milestone-1-phase-0.md` lines 412, 2081 | contains "FAKE" / "made-up" |
| d06ac91, 8 Oct | `spikes/tools/append_results.py:12`, `spikes/wsl/setup_steplet.sh:33` | contains "FAKE" |
| a2b61a8, 9 Oct | `spikes/wsl/s3e_precedence.sh:6` | contains "made-up" |
| 2a75fd0, 13 Oct | `spikes/p3/check2.ps1:22` | contains "FAKE" |
| 921eb44 to 2b22b56, 21–29 Oct | the test files `session_live.rs`, `gate.rs`, `support/mod.rs`, `key.rs`, `save.rs`, `session.rs`, `views.rs`, `setup.spec.ts` | contains "MADE-UP" |
| 2b22b56, 29 Oct | `docs/nebius-hackathon-results.md` lines 174, 175 (two Bearer lines) | contains "MADE-UP" |

The test values, by kind: 11 lines of GitHub's PGP signature in merge commit `56d0433`'s message; SSH signatures in
the signing test files (`crates/steplet-core/tests/vectors/*.steplet`, one of them cut short on purpose), the two
samples in `examples/`, the early spike's test files, and one results document; 3 SSH public keys (the test keys'
`.pub` files); key fingerprints in tests, test data and results documents; one thinking signature in a recorded
model reply (`crates/steplet-run/tests/support/recorded_stream.jsonl`).

### The 19 known hits

The plan says "the 19 known hits are included". No list of them is written down anywhere: not in `docs/`, not in the
private repository's files, not in the saved notes. So they could not be checked one by one against this scan.
The scan's own list has 16 key-shaped hits (all made-up markers) and 98 random-looking test values. If the 19 come
from an earlier search, that search's list is needed to match them up.

## 2. The saved-keys count (decision N8)

This one matters most: the scan above only finds strings that look like keys. A real key pasted into a file would
most likely look key-shaped or random and be caught, but the count checks for it exactly. It reads the key saved in Steplet (the same
place Steplet reads it from) and looks for that exact key in every file version, pictures included, and every commit
message: as plain text, as UTF-16 (the way some Windows programs write text), and as base64 (a way of writing bytes as
letters), in every alignment.

Self-test: a random made-up key is planted in a throwaway repository as plain text, UTF-16, base64 and in a commit
message; a second made-up key is never planted.

```
> py -3.13 -I tools/count_saved_keys.py --self-test --temp-dir <scratch folder>
PASS: planted key, plain text: expected 2, found 2
PASS: planted key, UTF-16: expected 1, found 1
PASS: planted key, base64: expected 1, found 1
PASS: never planted key, plain text: expected 0, found 0
PASS: never planted key, UTF-16: expected 0, found 0
PASS: never planted key, base64: expected 0, found 0
Objects read: 7.
Self-test: PASSED; the throwaway repository is deleted.
exit 0
```

The real run:

```
> py -3.13 -I tools/count_saved_keys.py
Anthropic key (Steplet/anthropic-api-key): saved; found 0 times in 3568 objects (plain text 0, UTF-16 0, base64 0).
Nebius key (Steplet/nebius-api-key): not saved, so nothing to count.
count exit 0
```

3568 objects = 138 commits + 1018 folder lists + 2412 file versions, the same set the scan read.

<!-- TODO after Task 4: the Nebius key is saved through Setup in Task 4. Rerun `py -3.13 -I tools/count_saved_keys.py`
then, and again just before the push in Task 12, and paste both lines here. -->

## What these checks do not cover

- **The Nebius key**: not saved when this ran, so not counted (see the note above).
- **Later commits**: this ran at `2b22b56`. Everything committed after it (Tasks 4 and 7 to 12) must be scanned again
  just before the push.
- **Pictures**: the shape scan does not read the 446 `.png`, 1 `.ico` and 1 `.pyc` file versions. The saved-keys
  count does read them, but only for the two saved keys. A key shown in a screenshot is a picture of text, and
  neither script can read it: screenshots added for the README and the video must be looked at by eye.
- **Keys the scan has no shape for**: other services' keys that are short, or not random-looking, or written with
  spaces or line breaks in the middle, would not be found by the shape scan. Only the two saved keys are checked
  exactly.
- **The two saved keys only**: any other key Ranvir has ever used (for example in another project) is not counted.
- **Other places**: GitHub issues, pull request text, the Devpost page and the video are not part of the history and
  were not checked.

## Again before the push (30 Oct 2026, 18:13 and 18:45 IST, at commit `1fa7f1f`)

`py -3.13 -I tools/secret_scan.py`, over every ref (`nebius-hackathon` and the merged task branches):

```
Objects read: 148 commits, 1128 folder lists, 2925 file versions, 0 tags; 148,896,844 bytes of file versions and messages.
Hits: 128 distinct (the same text at the same path counts once, however many versions): 14 real, 16 made-up marker, 98 test value
```

The 14 hits classed real are all in the attack suite's records of 30 Oct (`docs/suite/2026-10-30-*.txt`, first in
`429d8fe`). Each one is looked at: it is the fence's own login for the local web proxy of one run, in the shape
`http://srt.<one-run token>:<one-run password>@localhost:3128`, printed by a test that shows which sites a program in the
fence could reach. That login is made fresh for each run, works only on `localhost` inside that run's fence, and ends
with the run. It opens nothing now and nothing outside this laptop. So: **0 working keys**. They stay in the history
(decision N6 keeps all of it), recorded here.

`py -3.13 -I tools/count_saved_keys.py`, with both keys saved in Steplet on this laptop:

```
Anthropic key (Steplet/anthropic-api-key): saved; found 0 times in 4201 objects (plain text 0, UTF-16 0, base64 0).
Nebius key (Steplet/nebius-api-key): saved; found 0 times in 4201 objects (plain text 0, UTF-16 0, base64 0).
```
