"""Secret scan over the whole git history (Nebius hackathon plan, Task 12, first step).

Reads every object reachable from any branch or tag (`git rev-list --all --objects`, then `git cat-file --batch`):
every version of every file, and every commit message. It looks for the shapes real keys have, and for long
random-looking strings. Each hit is listed with the commit that first added it, its path and line, and a class:

- made-up marker: a key-shaped string written to look like a key that works nowhere ("MADE-UP", "NOT-A-KEY",
  "EXAMPLE", a run of one repeated character, ...);
- test value: a value that is public by design and was made for the tests or the samples: an SSH signature, an SSH
  public key, or a key's fingerprint;
- real: anything the rules here cannot explain. It counts as real until a person has looked at it.

A hit is never printed whole: only its rule, its length and why it got its class. Files that are not text (pictures)
are counted and named by type, not read; the saved-keys count (`count_saved_keys.py`) reads those too.

Usage:  py -3.13 -I tools/secret_scan.py [--repo PATH]
        py -3.13 -I tools/secret_scan.py --self-test [--temp-dir FOLDER]
Exit code: 0 when no hit is classed real (or the self-test passed), 1 when at least one is (or it failed), 2 when
git fails.
"""

import argparse
import collections
import math
import re
import secrets
import string
import subprocess
import sys
import tempfile
import threading
from dataclasses import dataclass, field
from pathlib import Path

# The key shapes the plan lists, plus JSON web tokens, the shape of long signed keys.
KEY_SHAPES = [
    ("Anthropic key (sk-ant-)", re.compile(rb"sk-ant-[A-Za-z0-9_\-]{6,}")),
    ("sk- key", re.compile(rb"(?<![A-Za-z0-9_\-])sk-(?!ant-)[A-Za-z0-9_\-]{16,}")),
    ("GitHub token (ghp_ and kin)", re.compile(rb"(?<![A-Za-z0-9_])gh[pousr]_[A-Za-z0-9]{20,}")),
    ("GitHub token (github_pat_)", re.compile(rb"github_pat_[A-Za-z0-9_]{20,}")),
    ("AWS key id (AKIA, ASIA)", re.compile(rb"(?<![A-Z0-9])(?:AKIA|ASIA)[0-9A-Z]{16}(?![A-Z0-9])")),
    ("Google key (AIza)", re.compile(rb"AIza[0-9A-Za-z_\-]{35}")),
    ("Slack token (xox)", re.compile(rb"xox[abposr]-[A-Za-z0-9\-]{10,}")),
    ("private key block", re.compile(rb"-----BEGIN (?:[A-Z0-9]+ )*PRIVATE KEY(?: BLOCK)?-----")),
    ("long Bearer token", re.compile(rb"Bearer\s+[A-Za-z0-9._~+/=\-]{20,}")),
    ("JSON web token", re.compile(rb"eyJ[A-Za-z0-9_\-]{8,}\.eyJ[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}")),
]
RANDOM_RULE = "long random-looking string"
RANDOM_CANDIDATE = re.compile(rb"[A-Za-z0-9+/_\-]{32,}={0,2}")
RANDOM_MIN_ENTROPY = 4.2  # bits per character; random base64 or base62 of 32+ characters scores about 4.6 to 5
HEX_ONLY = re.compile(rb"[0-9a-fA-F\-]+")  # hashes, commit ids, UUIDs: listed as counts, not hits

# Strongest first: the reason names the first of these the hit contains.
MADE_UP_WORDS = [
    re.compile(word, re.IGNORECASE)
    for word in (
        rb"MADE[-_]?UP", rb"NOT[-_]?A[-_]?KEY", rb"FAKE", rb"EXAMPLE", rb"DUMMY", rb"PLACEHOLDER", rb"REDACTED",
        rb"YOUR[-_]?(?:API[-_]?)?KEY", rb"TEST",
    )
]
REPEATED_CHARACTER = re.compile(rb"(.)\1{5,}")

# A signature block: the begin line, then at most 40 lines of base64 (indented, quoted or blank), then the matching
# end line. A block cut short (a test file's on purpose) stops at the first line that is not base64.
SIGNATURE_BLOCK = re.compile(
    rb"-----BEGIN (SSH|PGP) SIGNATURE-----\r?\n(?:[ \t>]*[A-Za-z0-9+/=]*\r?\n){0,40}"
    rb"(?:[ \t>]*-----END (\1) SIGNATURE-----|[ \t>]*[A-Za-z0-9+/=]+\Z)?"
)
SSH_PUBLIC_KEY = re.compile(rb"ssh-(?:ed25519|rsa|ecdsa[\w-]*) AAAA[A-Za-z0-9+/]+={0,2}")
FINGERPRINT = re.compile(rb"SHA256:[A-Za-z0-9+/]{43}")
# A recorded model reply's thinking block ends with a signature the model's maker checks; it opens nothing.
THINKING_SIGNATURE = re.compile(rb"\"type\":\"thinking\",\"thinking\":\"[^\"]*\",\"signature\":\"[A-Za-z0-9+/=]+\"")
LOCK_HASH = re.compile(rb"\"integrity\": \"sha(?:1|256|384|512)-[A-Za-z0-9+/]+={0,2}\"")
# Names made of words: file paths, folder names, test names. Split at separators, then into chunks of one kind
# ("AppData" is "App" and "Data"; "Python312" is "Python" and "312"). Words make long chunks; a random key's chunks
# average about 2 characters ("BlvNj6YK" is "Blv", "Nj", "6", "YK").
SEPARATORS = re.compile(rb"[/_.:\-]")
CHUNK = re.compile(rb"[A-Z]?[a-z]+|[A-Z]+(?![a-z])|[0-9]+")
HEX_PIECE = re.compile(rb"[0-9a-f]+")
ALPHABET_RUN = 20  # this many characters in a row in alphabet order: a character set written out, not a key


def git(repo, *args, data=None):
    out = subprocess.run(["git", "-C", str(repo), *args], input=data, capture_output=True, check=False)
    if out.returncode != 0:
        sys.stderr.write(f"git {' '.join(args)} failed: {out.stderr.decode(errors='replace').strip()}\n")
        sys.exit(2)
    return out.stdout


def iter_objects(repo):
    """Yields (sha, type, content) for every object `git rev-list --all --objects` lists."""
    shas = [line.split(b" ", 1)[0] for line in git(repo, "rev-list", "--all", "--objects").splitlines() if line]
    proc = subprocess.Popen(
        ["git", "-C", str(repo), "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE
    )

    def feed():
        proc.stdin.write(b"".join(sha + b"\n" for sha in shas))
        proc.stdin.close()

    feeder = threading.Thread(target=feed, daemon=True)
    feeder.start()
    for _ in shas:
        header = proc.stdout.readline().split()
        if len(header) != 3:
            sys.stderr.write("git cat-file gave an answer this script cannot read\n")
            sys.exit(2)
        sha, kind, size = header[0].decode(), header[1].decode(), int(header[2])
        content = proc.stdout.read(size)
        proc.stdout.read(1)  # the newline after each object
        yield sha, kind, content
    feeder.join()
    if proc.wait() != 0:
        sys.stderr.write("git cat-file failed\n")
        sys.exit(2)


def first_commits(repo):
    """Maps each file version (blob id) to the oldest commit that added it, with its path and the commit's date."""
    log = git(
        repo, "log", "--all", "--reverse", "--root", "-m", "--raw", "--no-abbrev", "--no-renames",
        "--format=commit %H %ad", "--date=short",
    )
    seen = {}
    commit = None
    for line in log.decode("utf-8", errors="replace").splitlines():
        if line.startswith("commit "):
            _, sha, date = line.split(" ", 2)
            commit = (sha, date)
        elif line.startswith(":") and "\t" in line:
            meta, path = line.split("\t", 1)
            blob = meta.split()[3]
            if set(blob) != {"0"} and blob not in seen:
                seen[blob] = (commit[0], commit[1], path)
    return seen


def entropy(text):
    counts = collections.Counter(text)
    return -sum(n / len(text) * math.log2(n / len(text)) for n in counts.values())


def looks_random(token):
    body = token.rstrip(b"=")
    has = lambda pattern: re.search(pattern, body) is not None  # noqa: E731
    return has(rb"[a-z]") and has(rb"[A-Z]") and has(rb"[0-9]") and entropy(body) >= RANDOM_MIN_ENTROPY


def word_like(piece):
    if len(piece) <= 4 or HEX_PIECE.fullmatch(piece):
        return True
    chunks = CHUNK.findall(piece)
    return sum(map(len, chunks)) == len(piece) and len(piece) >= 3 * len(chunks)


def made_of_words(token):
    """True when at least 60% of the token's characters are in word-like pieces."""
    pieces = [p for p in SEPARATORS.split(token) if p]
    return sum(len(p) for p in pieces if word_like(p)) >= 0.6 * len(token)


def has_alphabet_run(token):
    run = 1
    for a, b in zip(token, token[1:]):
        run = run + 1 if b == a + 1 else 1
        if run >= ALPHABET_RUN:
            return True
    return False


def inside(spans, start, end):
    return any(a <= start and end <= b for a, b in spans)


def classify(match, start, end, public_spans):
    """The class of one hit and the reason for it."""
    for (a, b), what in public_spans:
        if a <= start and end <= b:
            return "test value", f"{what}: public by design"
    for word in MADE_UP_WORDS:
        found = word.search(match)
        if found:
            return "made-up marker", f'contains "{found.group().decode()}"'
    if REPEATED_CHARACTER.search(match):
        return "made-up marker", "a run of one repeated character"
    return "real", "not explained by any rule: look at it"


def hits_in(content, skipped):
    """Every hit in one file version or message, as (rule, match, class, reason). Long strings that are not hits are
    counted in `skipped` by why."""
    public = [
        ((m.start(), m.end()), f"an {m.group(1).decode()} signature" + ("" if m.group(2) else ", cut short"))
        for m in SIGNATURE_BLOCK.finditer(content)
    ]
    public += [((m.start(), m.end()), "an SSH public key") for m in SSH_PUBLIC_KEY.finditer(content)]
    public += [((m.start(), m.end()), "a key fingerprint") for m in FINGERPRINT.finditer(content)]
    public += [
        ((m.start(), m.end()), "a recorded model reply's thinking signature")
        for m in THINKING_SIGNATURE.finditer(content)
    ]
    lock_hashes = [(m.start(), m.end()) for m in LOCK_HASH.finditer(content)]

    hits = []
    for rule, pattern in KEY_SHAPES:
        hits += [(rule, m) for m in pattern.finditer(content)]
    shaped = [(m.start(), m.end()) for _, m in hits]
    for m in RANDOM_CANDIDATE.finditer(content):
        token = m.group()
        if any(a <= m.start() < b for a, b in shaped):
            continue  # already a key-shape hit
        if HEX_ONLY.fullmatch(token):
            skipped["hex only (hashes, commit ids, UUIDs)"] += 1
        elif inside(lock_hashes, m.start(), m.end()):
            skipped["package hashes in lock files"] += 1
        elif not looks_random(token):
            continue
        elif made_of_words(token):
            skipped["names made of words (paths, folder and test names)"] += 1
        elif has_alphabet_run(token):
            skipped["character sets written out"] += 1
        else:
            hits.append((RANDOM_RULE, m))
    return [(rule, m, *classify(m.group(), m.start(), m.end(), public)) for rule, m in hits]


@dataclass
class Finding:
    cls: str
    rule: str
    reason: str
    commit: str
    date: str
    path: str
    line: int
    length: int
    versions: int = 1
    blobs: set = field(default_factory=set)


def scan(repo):
    commits = first_commits(repo)
    findings = {}
    stats = collections.Counter()
    binary_types = collections.Counter()
    skipped = collections.Counter()

    for sha, kind, content in iter_objects(repo):
        stats[kind] += 1
        if kind not in ("blob", "commit", "tag"):
            continue
        stats["bytes"] += len(content)
        if kind == "blob":
            if b"\0" in content[:8000]:
                where = commits.get(sha)
                binary_types[(Path(where[2]).suffix.lower() or "(no ending)") if where else "(unknown)"] += 1
                continue
            commit, date, path = commits.get(sha, ("?", "?", "(not in any commit's changes)"))
        else:
            commit, date, path = sha, "", f"({kind} message)"

        for rule, m, cls, reason in hits_in(content, skipped):
            key = (path, rule, m.group())
            if key in findings:
                if sha not in findings[key].blobs:
                    findings[key].versions += 1
                    findings[key].blobs.add(sha)
                continue
            line = content.count(b"\n", 0, m.start()) + 1
            findings[key] = Finding(cls, rule, reason, commit, date, path, line, len(m.group()), blobs={sha})
    return list(findings.values()), stats, binary_types, skipped


def throwaway_repo(parent, files, message):
    """A new git repository in a temporary folder under `parent` (the system's when None): one commit adds `files`
    (name to bytes) with `message`, a second removes them, so what they hold is only in the history. Returns the
    `TemporaryDirectory`; the repository is at its `.name`, and it is deleted on `.cleanup()`."""
    folder = tempfile.TemporaryDirectory(prefix="steplet-self-test-", dir=parent, ignore_cleanup_errors=True)
    repo = Path(folder.name)
    who = ["-c", "user.name=steplet-self-test", "-c", "user.email=self-test@steplet.invalid"]
    git(repo, "init", "-q")
    for name, data in files.items():
        (repo / name).write_bytes(data)
    git(repo, "add", "--", *files)
    git(repo, *who, "commit", "-q", "-m", message)
    git(repo, "rm", "-q", "--", *files)
    git(repo, *who, "commit", "-q", "-m", "removed")
    return folder


def made_up_random(n, chars=string.ascii_letters + string.digits):
    """`n` random characters from `chars` that work as no key anywhere, with no made-up word or repeated run in them
    and every kind of character `chars` has (so the self-test never fails by chance)."""
    kinds = [k for k in (string.ascii_lowercase, string.ascii_uppercase, string.digits) if set(k) & set(chars)]
    while True:
        s = "".join(secrets.choice(chars) for _ in range(n))
        b = s.encode()
        if (
            all(set(s) & set(k) for k in kinds)
            and not any(w.search(b) for w in MADE_UP_WORDS)
            and not REPEATED_CHARACTER.search(b)
        ):
            return s


def self_test(parent):
    """Plants random keys of every shape in a throwaway repository, removes them, and checks that the scan finds each
    one and classes it real, classes a made-up one as made up, and lets a file path pass. The pieces are joined at
    run time so this file holds no key-shaped text; the keys are made here and work nowhere."""
    r = made_up_random
    upper = string.ascii_uppercase + string.digits
    planted = [  # (rule, the line, the class expected; None: no hit expected)
        ("Anthropic key (sk-ant-)", "sk-" + "ant-api03-" + r(80), "real"),
        ("sk- key", "sk-" + "proj-" + r(48), "real"),
        ("GitHub token (ghp_ and kin)", "gh" + "p_" + r(36), "real"),
        ("GitHub token (github_pat_)", "github" + "_pat_" + r(22) + "_" + r(59), "real"),
        ("AWS key id (AKIA, ASIA)", "AK" + "IA" + r(16, upper), "real"),
        ("Google key (AIza)", "AI" + "za" + r(35), "real"),
        ("Slack token (xox)", "xo" + "xb-" + r(12, string.digits) + "-" + r(24), "real"),
        ("private key block", "-----BEGIN " + "OPENSSH PRIVATE KEY-----", "real"),
        ("long Bearer token", "Authorization: Bear" + "er " + r(40), "real"),
        ("JSON web token", "ey" + "J" + r(20) + ".ey" + "J" + r(30) + "." + r(43), "real"),
        (RANDOM_RULE, "value = " + r(48), "real"),
        ("Anthropic key (sk-ant-)", "sk-" + "ant-api03-STEPLET-MADE-UP-NOT-A-KEY", "made-up marker"),
        (RANDOM_RULE, "path = /home/someone/AppData/Local/Programs/Python/Python312/Scripts/", None),
    ]
    in_message = "planted " + "gh" + "p_" + r(36)
    text = "".join(line + "\n" for _, line, _ in planted).encode()
    folder = throwaway_repo(parent, {"planted.txt": text}, in_message)
    try:
        findings = scan(Path(folder.name))[0]
    finally:
        folder.cleanup()

    failures = 0
    for number, (rule, _, expected) in enumerate(planted, start=1):
        found = [f for f in findings if f.path == "planted.txt" and f.line == number and f.rule == rule]
        got = found[0].cls if found else None
        ok = got == expected
        failures += not ok
        print(
            f"{'PASS' if ok else 'FAIL'}: {rule}, line {number}: expected "
            f"{expected or 'no hit'}, got {got or 'no hit'}"
        )
    found = [f for f in findings if f.path == "(commit message)" and f.rule == "GitHub token (ghp_ and kin)"]
    ok = len(found) == 1 and found[0].cls == "real"
    failures += not ok
    print(f"{'PASS' if ok else 'FAIL'}: GitHub token in a commit message: expected real, got {found[0].cls if found else 'no hit'}")
    extra = [f for f in findings if f.path == "planted.txt" and f.line > len(planted)]
    failures += bool(extra)
    print(f"{'PASS' if not extra else 'FAIL'}: no hit outside the planted lines ({len(extra)} found)")
    print(f"Self-test: {'PASSED' if not failures else f'FAILED ({failures})'}; the throwaway repository is deleted.")
    return 1 if failures else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--repo", default=".", help="the repository to scan (default: this folder)")
    parser.add_argument("--self-test", action="store_true", help="check the scan on planted keys, then stop")
    parser.add_argument("--temp-dir", help="where the self-test makes its throwaway repository")
    args = parser.parse_args()
    repo = Path(args.repo)
    sys.stdout.reconfigure(encoding="utf-8")
    if args.self_test:
        return self_test(args.temp_dir)

    refs = git(repo, "for-each-ref", "--format=%(refname) %(objectname:short)").decode().split("\n")
    findings, stats, binary_types, skipped = scan(repo)
    order = {"real": 0, "made-up marker": 1, "test value": 2}
    findings.sort(key=lambda f: (order[f.cls], f.date, f.path, f.line))

    print("Refs scanned (everything `git rev-list --all` reaches):")
    for ref in filter(None, refs):
        print(f"  {ref}")
    print(
        f"Objects read: {stats['commit']} commits, {stats['tree']} folder lists, {stats['blob']} file versions, "
        f"{stats['tag']} tags; {stats['bytes']:,} bytes of file versions and messages."
    )
    print(
        "File versions not read as text (not text, by git's own test):"
        + "".join(f" {n} {t}" for t, n in sorted(binary_types.items()))
    )
    print("Long strings not counted as hits:" + "".join(f" {n} {what};" for what, n in sorted(skipped.items())))
    print()
    by_class = collections.Counter(f.cls for f in findings)
    print(
        f"Hits: {len(findings)} distinct (the same text at the same path counts once, however many versions): "
        + ", ".join(f"{by_class[c]} {c}" for c in order)
    )
    by_rule = collections.Counter((f.cls, f.rule) for f in findings)
    for (cls, rule), n in sorted(by_rule.items(), key=lambda kv: (order[kv[0][0]], kv[0][1])):
        print(f"  {cls:15} {rule:30} {n}")
    print()
    print("Each hit: class | rule | first commit (date) | path:line | length | versions | why")
    for f in findings:
        print(
            f"{f.cls} | {f.rule} | {f.commit[:7]} ({f.date}) | {f.path}:{f.line} | {f.length} chars | "
            f"{f.versions} | {f.reason}"
        )
    return 1 if by_class["real"] else 0


if __name__ == "__main__":
    sys.exit(main())
