"""Makes command-splitting test vectors, checked by bash itself (decision G1).
Each line of commands.txt is: a command line, then each word bash makes of it, separated by the unit
separator (U+001F). The lines use only plain characters, spaces and quotes, never $ ` ; | & < > ( ) outside
quotes, so bash runs nothing when it reads them; it only splits them into words.
Run once from the repository root: python -I crates/steplet-core/tests/vectors/make_command_vectors.py
Needs the WSL distribution steplet-test (phase 0)."""
import pathlib
import random
import subprocess
import sys

OUT = pathlib.Path(__file__).resolve().parent / "commands.txt"
PLAIN = "abcxyzABCXYZ0189_-./,:@%+="
IN_SINGLE = PLAIN + " \"\\#*?[]{}~!;|&<>()"  # everything is literal inside single quotes
IN_DOUBLE = PLAIN + " '#*?[]{}~;|&<>()"  # no $ ` ! inside double quotes; " and \ are added escaped below
COUNT = 2000
US = "\x1f"
# Sent to `bash -s` as the first line of its input, ahead of the command lines (bash reads a script on a pipe one
# character at a time, so `read` gets the lines after it). Nothing passes through Windows' command-line quoting.
SCRIPT = r"""while IFS= read -r line; do eval "set -- $line"; for w in "$@"; do printf '%s\037' "$w"; done; printf '\n'; done"""

rng = random.Random(20261012)


def part():
    kind = rng.choice(["plain", "single", "double"])
    n = rng.randint(0, 6)
    if kind == "plain":
        text = "".join(rng.choice(PLAIN) for _ in range(max(n, 1)))
        return text, text
    if kind == "single":
        text = "".join(rng.choice(IN_SINGLE) for _ in range(n))
        return "'" + text + "'", text
    chars = [rng.choice(IN_DOUBLE + '"\\') for _ in range(n)]
    return '"' + "".join("\\" + c if c in '"\\' else c for c in chars) + '"', "".join(chars)


def word(first):
    while True:
        parts = [part() for _ in range(rng.randint(1, 3))]
        written, value = "".join(p[0] for p in parts), "".join(p[1] for p in parts)
        if not (first and "=" in value):  # Steplet refuses a first word that sets a variable
            return written, value


def line():
    words = [word(i == 0) for i in range(rng.randint(1, 5))]
    text = " " * rng.randint(0, 2)
    for i, (written, _) in enumerate(words):
        text += (" " * rng.randint(1, 3) if i else "") + written
    return text + " " * rng.randint(0, 2), [value for _, value in words]


def main():
    cases = [line() for _ in range(COUNT)]
    stdin = (SCRIPT + "\n" + "".join(text + "\n" for text, _ in cases)).encode("ascii")
    r = subprocess.run(["wsl.exe", "-d", "steplet-test", "--exec", "bash", "-s"], input=stdin, capture_output=True)
    if r.returncode != 0:
        sys.exit(f"bash stopped with exit code {r.returncode}: {r.stderr.decode(errors='replace')}")
    answers = r.stdout.decode("ascii").split("\n")[:-1]
    if len(answers) != COUNT:
        sys.exit(f"bash answered {len(answers)} lines for {COUNT}")
    for (text, words), answer in zip(cases, answers):
        if answer.split(US)[:-1] != words:
            sys.exit(f"bash and this script disagree on {text!r}: {answer.split(US)[:-1]!r} != {words!r}")
    OUT.write_bytes("".join(US.join([text] + words) + "\n" for text, words in cases).encode("ascii"))
    print(f"wrote {COUNT} command lines to {OUT}; bash made the same words for every one")


main()
