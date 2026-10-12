# The `.steplet` file: a guide for senders

A `.steplet` file is a task you send to someone else's AI agent: "check out my repository at this commit and do these steps". It has a short header that says exactly what the agent may and must never do, and a body with the instructions in Markdown. You sign it with your own SSH key, so the receiver can see that nobody changed it on the way.

## Start here

1. Copy `examples/run-tests.steplet` and give the copy your own name.
2. Change the header and the body (every field is described below).
3. Check it. It prints `ok`, or each problem with its line number, what is allowed, and an example. It also prints the rating the receiver will see (`rating:`) with each finding and what to do; a blocked finding makes `check` fail like a problem:
   ```
   steplet-sign check my-task.steplet
   ```
4. Sign it with your Ed25519 key. `ssh-keygen` asks for your passphrase as usual:
   ```
   steplet-sign sign my-task.steplet --key C:\Users\you\.ssh\id_ed25519
   ```
5. Send the file. Do not edit it after signing; if you must, sign it again.

## A complete file

```yaml
---
steplet: 1
title: Run the tests of Hello-World
sender: Ana Lee
expires: 2026-10-25
repository:
  url: https://github.com/octocat/Hello-World.git
  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
steps:
  - Count the lines in the README file.
  - Report the count.
allow:
  commands:
    - wc -l README
  domains:
    - crates.io
  read:
    - .
  write:
    - target
never:
  - Do not push to any remote.
signature: |
  -----BEGIN SSH SIGNATURE-----
  …
  -----END SSH SIGNATURE-----
---
# Body: instructions for the agent, in Markdown
```

The header is everything between the first two `---` lines. The body is everything after the second one. The `signature` block is added by `steplet-sign sign`; you never write it yourself.

## The fields

Any name not in this table is refused, at every level.

| Field | Required | What is allowed | Limit | Example |
|---|---|---|---|---|
| `steplet` | yes, and it must be the first line of the header | exactly `1` (the format version) | | `steplet: 1` |
| `title` | yes | one line of text, not empty | at most 120 characters | `title: Run the tests` |
| `sender` | yes | one line of text, not empty. This is who *says* they sent the file; the signature line shows whose key really signed it. | at most 80 characters | `sender: Ana Lee` |
| `expires` | yes | a real date written `YYYY-MM-DD`. The file is expired once that day ends (UTC). | from today to 365 days ahead; 14 days is usual | `expires: 2026-10-25` |
| `repository` | no (the format allows leaving it out) | two named fields below it, `url` and `commit`, both required | | see the next two rows |
| `repository` → `url` | yes, inside `repository` | `https://`, then a lowercase host name with at least one dot (the same rules as `domains`), then optionally a path of letters, digits and `. _ ~ / -`. No user name, port, `?` or `#`. | under 300 characters | `  url: https://github.com/octocat/Hello-World.git` |
| `repository` → `commit` | yes, inside `repository` | the full commit hash: 40 (or 64) lowercase hexadecimal characters | | `  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d` |
| `steps` | yes | a list of one-line steps, shown to the receiver one per row | 1 to 30 steps; each under 300 characters | `steps:` then `  - Report the count.` |
| `allow` | no | named lists below it: `commands`, `domains`, `read`, `write`; each one optional | | |
| `allow` → `commands` | no | a list of commands, kept exactly as written. Each line is matched word for word (decision G1): plain words and simple quotes only. Pipes, `;`, `&&`, `$`, redirections, wildcards and `~` make Steplet unable to read the command, so the agent must ask the receiver every time. Put a longer sequence in a script in the repository and allow `bash scripts/x.sh`. | 25 commands; each under 300 characters | `    - cargo test` |
| `allow` → `domains` | no | exact lowercase host names with at least one dot. No `*`, scheme (`https://`), port or path; the last part must have a letter (so no IP addresses). | 15 | `    - crates.io` |
| `allow` → `read` | no | folders inside the repository: `.` (all of it), or a path like `src/tests`. Parts are letters, digits, `.`, `_`, `-`, joined by `/`; no `..`, no `.` part, no leading or trailing `/`. | 10; each under 300 characters | `    - .` |
| `allow` → `write` | no | folders, same rules as `read` | 10; each under 300 characters | `    - target/debug` |
| `never` | no | a list of one-line things the agent must never do | 25; each under 300 characters | `  - Do not push to any remote.` |
| `signature` | added by `steplet-sign` | one block, the last thing in the header | | |

Size: the whole file at most 256 KiB (262,144 bytes); the header at most 32 KiB (32,768 bytes). A list over its limit is reported at the line of the first item over the limit.

## Writing values

The header is a small, strict part of YAML. Anything outside it is refused with its line number, never guessed.

- **One line per value.** No `|` or `>` blocks and no values that continue on the next line.
- **Indent with spaces, two per level.** No tab characters anywhere in the header.
- **Lists** are one `- item` per line, indented two spaces more than the list's name. No `[a, b]` lists.
- **Comments** only on a line of their own, starting with `#`. Not after a value.
- **Plain text** is fine for most values: `title: Run the tests`, `title: a#b`, `url: https://x.org/a:b`.
- **Use double quotes** when the value:
  - starts with any of `` - ? : , [ ] { } # & * ! | > ' " % @ ` ``
  - contains a colon followed by a space (`: `), or a space followed by `#` (` #`)
  - ends with a colon

  For example `title: "Note: run the tests"` and `title: "Fix issue #12"`.
- **Inside double quotes**, the only escapes are `\"` (a quote) and `\\` (a backslash). Anything else after a backslash is refused. Nothing may follow the closing quote, not even a comment.
- **Single quotes** work too: `title: 'it''s done'`. Two single quotes stand for one.
- **Empty text** (`title: ""`) is refused wherever text is expected; leave out a field you do not need instead.
- **Not allowed in the header:** control characters, byte-order marks, and the Unicode line and paragraph separators.
- **Invisible characters** (decision G7): the receiver's preview shows every one as a visible badge, and the rating treats them by where they are. A hiding character (a zero-width space or joiner, a direction control, a tag, a control character) in the `title` or the `sender` makes the file **blocked**; any invisible character in a command makes it **blocked** (the shell would run something other than what is shown); in steps, never lines and the body a hiding character makes the file **red**. Space-like characters (a no-break space, an ideographic space and the other Unicode space separators) are only drawn as badges, so French spacing before `:` or `!` is fine.
- Anchors (`&`), aliases (`*`), tags (`!!str`), duplicate names at one level, and a second YAML document are all refused.

## Signing

- **Ed25519 keys only.** If you do not have one: `ssh-keygen -t ed25519`. An RSA key (or any other kind) is refused before anything is written, with the message "the signing key is not an Ed25519 key".
- `steplet-sign` calls Windows' own `C:\Windows\System32\OpenSSH\ssh-keygen.exe`. To use a different one: `--ssh-keygen <path>`.
- **What `sign` changes in your file:** it adds the signature block as the last thing in the header, turns Windows line endings into Unix ones, removes spaces and tabs at the ends of lines, and leaves exactly one newline at the end. It writes a temporary file next to yours and then replaces yours, so a crash cannot leave half a file. It changes nothing if any step fails.
- **Before writing, `sign` checks the header, the size limits and the expiry, as `check` does,** then signs, then verifies the new signature with Steplet's own code.
- **`sign` does not rate.** It signs a file that `check` would fail for a blocked finding (a `sudo` command, a path like `~/.ssh` in the body). Run `check` first, and send only a file that says `ok`.
- **Editing after signing breaks the signature.** `check` then says "the file was changed after it was signed". Run `sign` again: it replaces the old signature.
- **What the signature does not cover, by design (decision R13):** spaces and tabs at the ends of lines, Windows versus Unix line endings, and extra empty lines at the very end of the file. These are cleaned up before the signature is checked, because email and chat programs often change them on the way.
- **Do not edit or re-wrap the signature block.** It must stay exactly as `steplet-sign` wrote it; otherwise the file is refused.
- **Expiry** is required. `check` reports, and `sign` refuses, a date in the past or more than 365 days ahead, and shows a date 14 days from today as an example. The example file `examples/run-tests.steplet` expires on 2027-09-30; move its date forward before then.

## How receivers will see your file

Steplet rates every file with fixed code rules (decision G3; the full list is in `docs/rules.md`) and shows the receiver one colour, the reasons for it, and what to do. The colour is the **worst single finding**:

| Colour | What the receiver sees | What it means for you |
|---|---|---|
| green | "nothing in this file asks for more than it declares, and everything declared is low risk" | the file runs after one click |
| amber | "continue only if you trust the sender" | the receiver decides on trust; most test tasks land here |
| red | "run only after typing a confirmation sentence" | the receiver must type a sentence to run it; avoid this |
| blocked | "this file cannot run"; there is no Run button | fix the file |

`steplet-sign check` prints the same rating (`rating: amber`) and every finding with its line, the text it is about and what to do, so you see it before the receiver does. A blocked finding makes `check` fail.

**Common reasons for amber, and how to avoid the avoidable ones:**

- *A command runs the repository's own code* (`cargo test`, `npm test`, a script): unavoidable for a test task, and fine. Only reading commands (`ls`, `cat`, `wc`, `grep`, `git status`, …) are green.
- *A command downloads and installs packages* (`npm install`, `pip install`): unavoidable when the task needs it.
- *The file lets the agent reach a web site*: every site in `domains` except the package registries Steplet knows (crates.io, npm, PyPI, Go, Maven, Gradle, RubyGems, NuGet, Packagist, GitHub). List only the sites the task really needs.
- *The instructions mention a command the file does not ask for*: the body or a step names a command in backticks or a code block that is not in `allow: commands`. Add it, or take it out of the text.
- *Steplet cannot read a command exactly*: the command uses a pipe, `;`, `&&`, `$`, a redirection, a wildcard or `~`. Write one plain command, or put the sequence in a script in the repository.
- *A command names a path outside the repository*: `/etc/...`, `~/...`, `../...`. Keep the task inside the repository.

**Red** comes from things that look like tricks: invisible characters in the text, an HTML comment, a long block of encoded characters, many empty lines in a row, a link whose text names one site and opens another, a `xn--` address, a web address one letter away from a well-known site, a paste or webhook site in `domains`, and sentences that tell the agent to ignore its instructions, hide something, switch off its safety rules, claim approval, or ask for passwords, keys, administrator rights or for protection to be switched off.

**Blocked** comes from a `sudo`-like command, a command that downloads a script and runs it straight away, a command that points a package manager at another registry, `allow: write` naming `.git` or an agent's settings folder, an invisible character in the title, the sender or a command, a signature that does not match, an expired file, and any mention of a place that holds keys or passwords (`~/.ssh`, `~/.aws`, a browser profile, a wallet) outside the repository.

**Warnings to the agent belong in `never`.** "Do not read `~/.ssh`" in the body makes the file blocked, because the body is instructions; the same line under `never` is a warning and is fine.

## What the messages mean

`steplet-sign` ends with one of three exit codes:

| Exit code | Meaning |
|---|---|
| 0 | Fine: the file is ok and nothing in its rating is blocked (`check`), or it was signed (`sign`). |
| 1 | Problems or blocked findings were found (`check`), or signing failed (`sign`). The file was not changed. |
| 2 | The command was used wrongly, or the file could not be read. The usage is printed. |

A file that is ok (real output on 13 Oct 2026):

```
examples/run-tests.steplet: ok
  3 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 2 never lines
  repository: https://github.com/octocat/Hello-World.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
  expires: 2027-09-30 (in 353 days)
  signature: not signed
  rating: green
```

A signed file that was changed afterwards (real output on 13 Oct 2026). The rating says `blocked` with nothing under it, because the signature problem is already printed as a problem line:

```
crates/steplet-core/tests/vectors/tampered.steplet: 1 problem
  2 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 0 never lines
  repository: https://github.com/octocat/Hello-World.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
  expires: 2026-12-31 (in 80 days)
  rating: blocked
  problem: signature: the file was changed after it was signed, or the signature belongs to another file
    allowed: a signature made by steplet-sign over the file exactly as it is now
    for example:
      steplet-sign sign my-task.steplet --key C:\Users\you\.ssh\id_ed25519
```

Each problem has the same three parts: the line and what is wrong, what is allowed instead, and an example you can copy.

A rating finding, printed under `rating:`, is different: the rule's severity and the line, a title, the text from your file it is about (invisible characters written as `<U+200B>`), and what to do. `docs/rules.md` lists every rule with its id, severity, title and advice. For example:

```
  rating: amber
    amber, line 14: A command runs the repository's own code
      cargo test
      Tests, builds and scripts run code from the repository. Steplet runs them only inside the fence.
```
