# Milestone 1, phase 0, S2: which Windows environment variables wsl.exe itself needs to start. Throwaway.
# Prints variable names and outcomes only, never values.
# Usage: python -I s2_env_min.py <case> [<case> ...]   where a case is "full", "none", or NAME,NAME,...
# Prefix a case with "-" to mean "full environment minus these names", e.g. -ProgramFiles,SystemDrive
import os
import subprocess
import sys
import tempfile

WSL = r"C:\Windows\System32\wsl.exe"
GUID = open(r"C:\dev\m1\distro-id.txt", encoding="ascii").read().strip()


def reset():
    subprocess.run([WSL, "--terminate", "steplet-test"], capture_output=True, timeout=60)
    r = subprocess.run([WSL, "-d", "steplet-test", "--exec", "/bin/echo", "ok"], capture_output=True, timeout=60)
    return r.stdout.strip() == b"ok"


def build(case):
    full = dict(os.environ)
    if case == "full":
        return full
    if case == "none":
        return {}
    if case.startswith("-"):
        drop = {n.upper() for n in case[1:].split(",")}
        return {k: v for k, v in full.items() if k.upper() not in drop}
    keep = {n.upper() for n in case.split(",")}
    return {k: v for k, v in full.items() if k.upper() in keep}


def run(case):
    env = build(case)
    with tempfile.TemporaryFile() as out:
        p = subprocess.Popen([WSL, "--distribution-id", GUID, "--exec", "/bin/echo", "STARTED"],
                             env=env, stdin=subprocess.PIPE, stdout=out, stderr=subprocess.STDOUT)
        p.stdin.close()
        try:
            code = p.wait(timeout=15)
            hung = False
        except subprocess.TimeoutExpired:
            subprocess.run(["taskkill", "/T", "/F", "/PID", str(p.pid)], capture_output=True)
            code, hung = None, True
        out.seek(0)
        text = out.read().replace(b"\x00", b"").decode("utf-8", "replace").strip()
    ok = (code == 0 and text == "STARTED")
    names = "(full environment)" if case == "full" else ",".join(sorted(env)) or "(none)"
    if case.startswith("-"):
        names = "full minus " + case[1:]
    first = text.splitlines()[0][:100] if text else ""
    print(f"{'OK  ' if ok else 'FAIL'} vars={len(env):3d} hung={hung} exit={code} first_line={first!r}")
    print(f"     names: {names}")
    if not ok and RESET:
        print(f"     distro reset: {'ok' if reset() else 'STILL BROKEN'}")
    return ok


RESET = "--no-reset" not in sys.argv

if __name__ == "__main__":
    for c in sys.argv[1:]:
        if c != "--no-reset":
            run(c)
    if not RESET:
        print(f"final distro reset: {'ok' if reset() else 'STILL BROKEN'}")
