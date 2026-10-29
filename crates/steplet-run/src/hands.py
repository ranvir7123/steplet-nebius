"""The file tools of Steplet's own agent (the Nebius plan, Task 5): read, write, edit, list, find and search. Run by
start-hands inside the run's fence, as the steplet account, in the working copy, with the move's JSON on standard
input. What it found or did goes to standard output; a refusal goes to the error output, with exit code 1. It judges
nothing: Steplet's checker decided before the move was sent, and the fence holds whatever this does. Installed by
Steplet as /opt/steplet/bin/hands.py."""

import fnmatch
import json
import os
import re
import stat
import sys

# The most lines a read gives unless it asks for fewer, and the most characters of any line shown.
MOST_LINES = 2000
LONGEST_LINE = 2000
# The most entries a list or a find gives, and the most lines a search gives.
MOST_ENTRIES = 500
MOST_MATCHES = 200
# How far down a list goes, and the folders no list, find or search goes into.
DEEPEST = 3
SKIPPED = {".git"}
# A file whose first bytes hold a zero byte is taken as not text.
SNIFF = 8192


def fail(why):
    print(why, file=sys.stderr)
    sys.exit(1)


def contents(path):
    """The file's bytes, and whether they are text."""
    with open(path, "rb") as f:
        data = f.read()
    return data, b"\0" not in data[:SNIFF]


def number(args, name, default):
    value = args.get(name)
    if value is None:
        return default
    if not isinstance(value, int) or isinstance(value, bool) or value < 1:
        fail(f"{name} must be a whole number of at least 1.")
    return value


def text_arg(args, name):
    value = args.get(name)
    if not isinstance(value, str):
        fail(f"{name} is missing; it must be text.")
    return value


def read(args):
    path = text_arg(args, "path")
    data, is_text = contents(path)
    if not is_text:
        return f"{path} is not text ({len(data)} bytes); it is not shown."
    lines = data.decode("utf-8", errors="replace").splitlines()
    if not lines:
        return f"{path} is empty."
    start, limit = number(args, "offset", 1), number(args, "limit", MOST_LINES)
    shown = lines[start - 1 : start - 1 + limit]
    if not shown:
        return f"{path} has {len(lines)} lines; there is nothing from line {start}."
    out = [f"{n:6}\t{line[:LONGEST_LINE]}" for n, line in enumerate(shown, start)]
    if start - 1 + len(shown) < len(lines):
        out.append(f"(The file has {len(lines)} lines; read on from line {start + len(shown)}.)")
    return "\n".join(out)


def write(args):
    path, content = text_arg(args, "path"), text_arg(args, "content")
    folder = os.path.dirname(path)
    if folder:
        os.makedirs(folder, exist_ok=True)
    data = content.encode("utf-8")
    with open(path, "wb") as f:
        f.write(data)
    return f"Wrote {len(data)} bytes to {path}."


def edit(args):
    path, old, new = text_arg(args, "path"), text_arg(args, "old"), text_arg(args, "new")
    if not old:
        fail("The old text is empty; give the exact text to replace.")
    data, is_text = contents(path)
    if not is_text:
        fail(f"{path} is not text; it was not changed.")
    old_bytes = old.encode("utf-8")
    count = data.count(old_bytes)
    if count == 0:
        fail(f"The old text is not in {path}; it was not changed.")
    if count > 1:
        fail(f"The old text is in {path} {count} times; give more of it, so it matches once. It was not changed.")
    with open(path, "wb") as f:
        f.write(data.replace(old_bytes, new.encode("utf-8"), 1))
    return f"Edited {path}: the one match was replaced."


def is_device(path):
    """Whether path is a device file. No repository holds one (git cannot), and inside the fence the names no move may
    make are held by /dev/null, so list, find and search leave device files out."""
    try:
        mode = os.lstat(path).st_mode
    except OSError:
        return False
    return stat.S_ISCHR(mode) or stat.S_ISBLK(mode)


def files_under(root):
    """Every file below the folder root, as a path relative to it, in a fixed order, skipping SKIPPED folders and
    device files."""
    for folder, folders, names in os.walk(root):
        folders[:] = sorted(f for f in folders if f not in SKIPPED)
        for name in sorted(names):
            full = os.path.join(folder, name)
            if not is_device(full):
                yield os.path.relpath(full, root)


def folder_arg(args):
    root = args.get("path") or "."
    if not isinstance(root, str):
        fail("path must be text.")
    if not os.path.isdir(root):
        fail(f"{root} is not a folder.")
    return root


def list_files(args):
    root = folder_arg(args)
    depth = min(number(args, "depth", 2), DEEPEST)
    out, more = [], False

    def walk(folder, level):
        nonlocal more
        try:
            names = sorted(os.listdir(folder))
        except OSError as e:
            out.append(f"{os.path.relpath(folder, root)}/ (could not be listed: {e.strerror})")
            return
        for name in names:
            if len(out) >= MOST_ENTRIES:
                more = True
                return
            full = os.path.join(folder, name)
            if is_device(full):
                continue
            is_folder = os.path.isdir(full) and not os.path.islink(full)
            out.append(os.path.relpath(full, root) + ("/" if is_folder else ""))
            if is_folder and level < depth and name not in SKIPPED:
                walk(full, level + 1)

    walk(root, 1)
    if not out:
        return f"{root} is empty."
    if more:
        out.append(f"(Stopped at {MOST_ENTRIES} entries; list a smaller folder.)")
    return "\n".join(out)


def matches(relative, pattern):
    """Whether the file at relative matches the glob pattern. A pattern without "/" is matched against the file's
    name; one with "/" against its whole path, where fnmatch's * also crosses folders, and a leading "**/" also
    stands for no folder at all."""
    if "/" not in pattern:
        return fnmatch.fnmatchcase(os.path.basename(relative), pattern)
    tries = [pattern] + ([pattern[3:]] if pattern.startswith("**/") else [])
    return any(fnmatch.fnmatchcase(relative, p) for p in tries)


def find(args):
    root = folder_arg(args)
    pattern = text_arg(args, "pattern")
    out = []
    for relative in files_under(root):
        if matches(relative, pattern):
            if len(out) >= MOST_ENTRIES:
                out.append(f"(Stopped at {MOST_ENTRIES} files; give a narrower pattern.)")
                break
            out.append(relative)
    return "\n".join(out) if out else f"No file under {root} matches {pattern}."


def search(args):
    root = args.get("path") or "."
    regex, glob = text_arg(args, "regex"), args.get("glob")
    if glob is not None and not isinstance(glob, str):
        fail("glob must be text.")
    try:
        wanted = re.compile(regex)
    except re.error as e:
        fail(f"The pattern is not a regular expression Python reads: {e}.")
    if os.path.isfile(root):
        candidates = [(root, root)]
    else:
        folder_arg(args)
        candidates = [(os.path.join(root, r), r) for r in files_under(root) if not glob or matches(r, glob)]
    out = []
    for full, shown in candidates:
        try:
            data, is_text = contents(full)
        except OSError:
            continue
        if not is_text:
            continue
        for n, line in enumerate(data.decode("utf-8", errors="replace").splitlines(), 1):
            if wanted.search(line):
                if len(out) >= MOST_MATCHES:
                    out.append(f"(Stopped at {MOST_MATCHES} lines; give a narrower pattern or path.)")
                    return "\n".join(out)
                out.append(f"{shown}:{n}: {line[:LONGEST_LINE]}")
    return "\n".join(out) if out else f"No line under {root} matches {regex}."


TOOLS = {"read": read, "write": write, "edit": edit, "list": list_files, "find": find, "search": search}


def main():
    if len(sys.argv) != 2 or sys.argv[1] not in TOOLS:
        fail("hands.py takes one tool: " + ", ".join(TOOLS))
    try:
        args = json.load(sys.stdin)
    except ValueError as e:
        fail(f"The move's input is not JSON: {e}.")
    if not isinstance(args, dict):
        fail("The move's input is not a JSON object.")
    try:
        said = TOOLS[sys.argv[1]](args)
    except FileNotFoundError as e:
        fail(f"There is no {e.filename}.")
    except IsADirectoryError as e:
        fail(f"{e.filename} is a folder.")
    except NotADirectoryError as e:
        fail(f"A part of {e.filename} is not a folder.")
    except PermissionError as e:
        fail(f"Not allowed: {e.filename}.")
    except OSError as e:
        fail(f"{e.filename or 'The file'}: {e.strerror}.")
    sys.stdout.write(said + "\n")


main()
