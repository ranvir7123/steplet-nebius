# The `.steplet` file: a guide for senders

A `.steplet` file is a task you send to someone else's AI agent: "check out my repository at this commit and do these steps". It has a short header that says exactly what the agent may and must never do, and a body with the instructions in Markdown. You sign it with your own SSH key, so the receiver can see that nobody changed it on the way.

## Start here

1. Copy `examples/run-tests.steplet` and give the copy your own name.
2. Change the header and the body (every field is described below).
3. Check it. It prints `ok`, or each problem with its line number, what is allowed, and an example:
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
| `title` | yes | one line of text, not empty | | `title: Run the tests` |
| `sender` | yes | one line of text, not empty. This is who *says* they sent the file; the signature line shows whose key really signed it. | | `sender: Ana Lee` |
| `expires` | yes | a real date written `YYYY-MM-DD`. The file is expired once that day ends (UTC). | from today to 365 days ahead; 14 days is usual | `expires: 2026-10-25` |
| `repository` | no (the format allows leaving it out) | two named fields below it, `url` and `commit`, both required | | see the next two rows |
| `repository` → `url` | yes, inside `repository` | `https://`, then a lowercase host name with at least one dot (the same rules as `domains`), then optionally a path of letters, digits and `. _ ~ / -`. No user name, port, `?` or `#`. | | `  url: https://github.com/octocat/Hello-World.git` |
| `repository` → `commit` | yes, inside `repository` | the full commit hash: 40 (or 64) lowercase hexadecimal characters | | `  commit: 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d` |
| `steps` | yes | a list of one-line steps, shown to the receiver one per row | 1 to 30 steps | `steps:` then `  - Report the count.` |
| `allow` | no | named lists below it: `commands`, `domains`, `read`, `write`; each one optional | | |
| `allow` → `commands` | no | a list of commands, kept exactly as written | 25 commands; each under 300 characters | `    - cargo test` |
| `allow` → `domains` | no | exact lowercase host names with at least one dot. No `*`, scheme (`https://`), port or path; the last part must have a letter (so no IP addresses). | 15 | `    - crates.io` |
| `allow` → `read` | no | folders inside the repository: `.` (all of it), or a path like `src/tests`. Parts are letters, digits, `.`, `_`, `-`, joined by `/`; no `..`, no `.` part, no leading or trailing `/`. | 10 | `    - .` |
| `allow` → `write` | no | folders, same rules as `read` | 10 | `    - target/debug` |
| `never` | no | a list of one-line things the agent must never do | 25 | `  - Do not push to any remote.` |
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
- **Not allowed in the header:** control characters, byte-order marks, and the Unicode line and paragraph separators. Invisible characters such as a zero-width space are allowed, but the receiver's preview shows each one as a visible badge.
- Anchors (`&`), aliases (`*`), tags (`!!str`), duplicate names at one level, and a second YAML document are all refused.

## Signing

- **Ed25519 keys only.** If you do not have one: `ssh-keygen -t ed25519`. An RSA key (or any other kind) is refused before anything is written, with the message "the signing key is not an Ed25519 key".
- `steplet-sign` calls Windows' own `C:\Windows\System32\OpenSSH\ssh-keygen.exe`. To use a different one: `--ssh-keygen <path>`.
- **What `sign` changes in your file:** it adds the signature block as the last thing in the header, turns Windows line endings into Unix ones, removes spaces and tabs at the ends of lines, and leaves exactly one newline at the end. It writes a temporary file next to yours and then replaces yours, so a crash cannot leave half a file. It changes nothing if any step fails.
- **Before writing, `sign` checks the header, the size limits and the expiry, as `check` does,** then signs, then verifies the new signature with Steplet's own code.
- **Editing after signing breaks the signature.** `check` then says "the file was changed after it was signed". Run `sign` again: it replaces the old signature.
- **What the signature does not cover, by design (decision R13):** spaces and tabs at the ends of lines, Windows versus Unix line endings, and extra empty lines at the very end of the file. These are cleaned up before the signature is checked, because email and chat programs often change them on the way.
- **Do not edit or re-wrap the signature block.** It must stay exactly as `steplet-sign` wrote it; otherwise the file is refused.
- **Expiry** is required. `check` reports, and `sign` refuses, a date in the past or more than 365 days ahead, and shows a date 14 days from today as an example. The example file `examples/run-tests.steplet` expires on 2027-09-30; move its date forward before then.

## What the messages mean

`steplet-sign` ends with one of three exit codes:

| Exit code | Meaning |
|---|---|
| 0 | Fine: the file is ok (`check`), or it was signed (`sign`). |
| 1 | Problems were found (`check`), or signing failed (`sign`). The file was not changed. |
| 2 | The command was used wrongly, or the file could not be read. The usage is printed. |

A file that is ok (real output on 11 Oct 2026):

```
examples/run-tests.steplet: ok
  3 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 2 never lines
  repository: https://github.com/octocat/Hello-World.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
  expires: 2027-09-30 (in 354 days)
  signature: not signed
```

A signed file that was changed afterwards (real output on 11 Oct 2026):

```
crates/steplet-core/tests/vectors/tampered.steplet: 1 problem
  2 steps; allowed: 1 command, 0 domains, 0 read folders, 0 write folders; 0 never lines
  repository: https://github.com/octocat/Hello-World.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d
  expires: 2026-12-31 (in 81 days)
  problem: signature: the file was changed after it was signed, or the signature belongs to another file
    allowed: a signature made by steplet-sign over the file exactly as it is now
    for example:
      steplet-sign sign my-task.steplet --key C:\Users\you\.ssh\id_ed25519
```

Each problem has the same three parts: the line and what is wrong, what is allowed instead, and an example you can copy.
