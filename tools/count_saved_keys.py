"""Counts how often the keys saved in Steplet appear in the git history (decision N8).

Reads `Steplet/anthropic-api-key` and `Steplet/nebius-api-key` from Windows Credential Manager (`CredReadW`, as
Steplet itself does), then looks for each key in every object `git rev-list --all --objects` lists: every version of
every file, pictures included, and every commit message. Each key is looked for as plain text, as UTF-16 (the form
Windows programs often write), and as base64 in all three alignments, both alphabets.

It prints only whether each key is saved and how many times it was found. It never prints, logs or writes a key, any
part of it, its length, or anything worked out from it; an error while a key is held is reported by its kind only.

Usage:  py -3.13 -I tools/count_saved_keys.py [--repo PATH]
        py -3.13 -I tools/count_saved_keys.py --self-test [--temp-dir FOLDER]
Exit code: 0 when every saved key was found 0 times (or the self-test passed), 1 when one was found (or it failed),
2 when a key or git could not be read.
"""

import argparse
import base64
import ctypes
import importlib.util
import secrets
import sys
from ctypes import wintypes
from pathlib import Path

ENTRIES = [("Anthropic", "Steplet/anthropic-api-key"), ("Nebius", "Steplet/nebius-api-key")]
CRED_TYPE_GENERIC = 1
ERROR_NOT_FOUND = 1168


def load_scan():
    """`secret_scan.py`, beside this file, for its history reader (loaded by path, so `python -I` works)."""
    spec = importlib.util.spec_from_file_location("secret_scan", Path(__file__).with_name("secret_scan.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class FILETIME(ctypes.Structure):
    _fields_ = [("dwLowDateTime", wintypes.DWORD), ("dwHighDateTime", wintypes.DWORD)]


class CREDENTIALW(ctypes.Structure):
    _fields_ = [
        ("Flags", wintypes.DWORD),
        ("Type", wintypes.DWORD),
        ("TargetName", wintypes.LPWSTR),
        ("Comment", wintypes.LPWSTR),
        ("LastWritten", FILETIME),
        ("CredentialBlobSize", wintypes.DWORD),
        ("CredentialBlob", ctypes.POINTER(ctypes.c_ubyte)),
        ("Persist", wintypes.DWORD),
        ("AttributeCount", wintypes.DWORD),
        ("Attributes", ctypes.c_void_p),
        ("TargetAlias", wintypes.LPWSTR),
        ("UserName", wintypes.LPWSTR),
    ]


def read_saved(target):
    """The bytes saved under `target`, or None when there is no such entry. Raises OSError with only the Windows
    error number on any other failure."""
    advapi32 = ctypes.WinDLL("advapi32", use_last_error=True)
    advapi32.CredReadW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD,
                                   ctypes.POINTER(ctypes.POINTER(CREDENTIALW))]
    advapi32.CredReadW.restype = wintypes.BOOL
    advapi32.CredFree.argtypes = [ctypes.c_void_p]
    advapi32.CredFree.restype = None
    entry = ctypes.POINTER(CREDENTIALW)()
    if not advapi32.CredReadW(target, CRED_TYPE_GENERIC, 0, ctypes.byref(entry)):
        error = ctypes.get_last_error()
        if error == ERROR_NOT_FOUND:
            return None
        raise OSError(f"Windows error {error}")
    try:
        cred = entry.contents
        # A wrong structure layout would read the wrong bytes and count a false 0: check what can be checked.
        if (cred.TargetName or "").lower() != target.lower():
            raise OSError("the entry's name did not read back as asked")
        if not cred.CredentialBlob or cred.CredentialBlobSize == 0:
            return None
        value = ctypes.string_at(cred.CredentialBlob, cred.CredentialBlobSize)
        if not all(0x21 <= b < 0x7F for b in value):
            raise OSError("the saved key is not plain text without spaces, as Steplet saves keys")
        return value
    finally:
        advapi32.CredFree(entry)


def forms(key):
    """The byte strings a key could take in a file: plain, UTF-16 little-endian, and base64 (standard and URL-safe)
    at each of the three alignments, with the edge characters that depend on neighbouring bytes cut off."""
    out = {"plain text": [key]}
    try:
        out["UTF-16"] = [key.decode("utf-8").encode("utf-16-le")]
    except UnicodeError:
        out["UTF-16"] = []
    b64 = []
    for shift in range(3):
        encoded = base64.b64encode(bytes(shift) + key)[4:-4]
        if len(encoded) >= 16:
            b64 += [encoded, encoded.replace(b"+", b"-").replace(b"/", b"_")]
    out["base64"] = list(dict.fromkeys(b64))
    return out


def count(repo, keys, scan):
    """For each name in `keys` (name to key bytes): how many times each form was found, summed over every object
    the history holds, and the number of objects read."""
    needles = {name: forms(key) for name, key in keys.items()}
    totals = {name: {form: 0 for form in fs} for name, fs in needles.items()}
    objects = 0
    for _, _, content in scan.iter_objects(repo):
        objects += 1
        for name, fs in needles.items():
            for form, patterns in fs.items():
                totals[name][form] += sum(content.count(p) for p in patterns)
    return totals, objects


def self_test(parent, scan):
    """Plants a made-up key in a throwaway repository (as plain text in one file, UTF-16 in another, base64 in a
    third, and in a commit message), removes the files, and checks that `count` finds each form, and finds a second
    made-up key that was never planted 0 times. Both keys are random, made here, and work nowhere."""
    planted = ("steplet-made-up-" + secrets.token_urlsafe(32)).encode()
    never = ("steplet-made-up-" + secrets.token_urlsafe(32)).encode()
    files = {
        "plain.txt": b"before " + planted + b" after\n",
        "wide.bin": b"\xff\xfe" + planted.decode().encode("utf-16-le"),
        "b64.txt": base64.b64encode(b"x" + planted + b"y") + b"\n",
    }
    folder = scan.throwaway_repo(parent, files, "planted " + planted.decode())
    try:
        totals, objects = count(Path(folder.name), {"planted": planted, "never planted": never}, scan)
    finally:
        folder.cleanup()
    expected = {  # plain text: the file and the commit message
        "planted": {"plain text": 2, "UTF-16": 1, "base64": 1},
        "never planted": {"plain text": 0, "UTF-16": 0, "base64": 0},
    }
    failures = 0
    for name, wanted in expected.items():
        for form, n in wanted.items():
            got = totals[name][form]
            failures += got != n
            print(f"{'PASS' if got == n else 'FAIL'}: {name} key, {form}: expected {n}, found {got}")
    print(f"Objects read: {objects}.")
    print(f"Self-test: {'PASSED' if not failures else f'FAILED ({failures})'}; the throwaway repository is deleted.")
    return 1 if failures else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--repo", default=".", help="the repository to look in (default: this folder)")
    parser.add_argument("--self-test", action="store_true", help="check the count on a planted made-up key, then stop")
    parser.add_argument("--temp-dir", help="where the self-test makes its throwaway repository")
    args = parser.parse_args()
    scan = load_scan()
    if args.self_test:
        return self_test(args.temp_dir, scan)

    keys, saved = {}, {}
    for name, target in ENTRIES:
        try:
            value = read_saved(target)
        except OSError as e:
            print(f"{name} key ({target}): could not be read ({e}).")
            return 2
        saved[name] = value is not None
        if value is not None:
            keys[name] = value
    try:
        totals, objects = count(Path(args.repo), keys, scan)
    except Exception as e:  # never let a message that might hold key bytes out: report the kind of error only
        print(f"The count failed: {type(e).__name__}.")
        return 2
    finally:
        keys.clear()

    found_any = False
    for name, target in ENTRIES:
        if not saved[name]:
            print(f"{name} key ({target}): not saved, so nothing to count.")
            continue
        n = sum(totals[name].values())
        found_any |= n > 0
        detail = ", ".join(f"{form} {k}" for form, k in totals[name].items())
        print(f"{name} key ({target}): saved; found {n} times in {objects} objects ({detail}).")
    return 1 if found_any else 0


if __name__ == "__main__":
    sys.exit(main())
