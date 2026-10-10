"""Milestone 1, phase 0 spike runs inside the test distribution. Throwaway.

  python3 -I spike.py prepare  <ID>   fresh working copy, settings, MCP config, launch files
  python3 -I spike.py evidence <ID>   what the disk, the logs and the event stream show
  python3 -I spike.py cleanup  <ID>   stop whatever is left of the run, then list what remains

A run is described by spikes/runs/<ID>.json. Nothing here trusts what the agent says.
"""
import hashlib
import json
import os
import pathlib
import shutil
import signal
import subprocess
import sys
import time
import uuid

SPIKES = pathlib.Path("/mnt/c/dev/runFile/spikes")
WIN = pathlib.Path("/mnt/c/dev/m1")
HOME = pathlib.Path.home()
P = HOME / ".local/share/steplet"  # Steplet's private folder (milestone plan, section 3)
M = HOME / "m1"                     # spike data: fixture source, fake key, logs, markers
CLAUDE = P / "claude-cli/bin/claude"
FAKE_MODEL_PORT = 47900
NOT_LINUX = {"Path", "PATHEXT", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "SystemRoot", "windir", "COMPUTERNAME",
             "USERNAME", "USERDOMAIN", "ProgramFiles", "ProgramData", "OneDrive", "WSLENV", "WT_SESSION",
             "WSL_INTEROP", "WSL_DISTRO_NAME", "PSModulePath", "HOMEDRIVE", "HOMEPATH", "NUMBER_OF_PROCESSORS"}
# Values safe to paste into the results; any other variable is shown by name only (it could hold a secret).
SHOW_VALUE = {"PATH", "WSLENV", "WSL_DISTRO_NAME", "WSL_INTEROP", "NAME", "HOSTTYPE", "TERM", "LANG", "SHELL", "HOME",
              "USER", "LOGNAME", "PWD", "SHLVL", "DISPLAY", "WAYLAND_DISPLAY", "XDG_RUNTIME_DIR", "PULSE_SERVER",
              "WSL2_GUI_APPS_ENABLED", "_"}


def load(run_id):
    prof = json.loads((SPIKES / "runs" / f"{run_id}.json").read_text(encoding="utf-8"))
    keep = prof.get("workcopy", "fixture")
    owner = run_id if keep == "fixture" else keep.split(":", 1)[1]
    return prof, HOME / "steplet-work" / owner / "repo"


def git(*args):
    return subprocess.run(["/usr/bin/git", *map(str, args)], capture_output=True, text=True)


def host_ip():
    words = subprocess.run(["ip", "route", "show", "default"], capture_output=True, text=True).stdout.split()
    return words[words.index("via") + 1] if "via" in words else "none"


def render(value, ctx):
    if isinstance(value, str):
        for k, v in ctx.items():
            value = value.replace("{" + k + "}", v)
        return value
    if isinstance(value, list):
        return [render(v, ctx) for v in value]
    if isinstance(value, dict):
        return {k: render(v, ctx) for k, v in value.items()}
    return value


def merge(base, extra):
    for k, v in extra.items():
        if isinstance(v, dict) and isinstance(base.get(k), dict):
            merge(base[k], v)
        elif isinstance(v, list) and isinstance(base.get(k), list):
            base[k] = base[k] + v
        else:
            base[k] = v
    return base


def snapshot(work):
    cfg, home = P / "claude-config", P / "home"
    try:
        trust = json.loads((cfg / ".claude.json").read_text()).get("projects", {}).get(str(work))
    except (OSError, ValueError):
        trust = None
    real_ignore = HOME / ".config/git/ignore"
    return {"config_files": sorted(str(p.relative_to(cfg)) for p in cfg.rglob("*") if p.is_file()),
            "private_home_files": sorted(str(p.relative_to(home)) for p in home.rglob("*") if p.is_file()),
            "real_git_ignore": real_ignore.read_text() if real_ignore.exists() else None,
            "trust_record": trust}


def settings_for(shape, work):
    if shape == "m0":
        deny = [str(M / "canary"), str(WIN / "canary")]
        fs = {"denyRead": deny}
        rules = [f"Read(/{p}/**)" for p in deny]
    elif shape == "product":
        deny = ["/mnt", str(P), str(HOME / "steplet-work"), str(M)]
        fs = {"denyRead": deny, "allowRead": [str(work)], "denyWrite": [str(work / ".git")]}
        rules = [f"Read(/{p}/**)" for p in deny] + [f"Edit(/{work}/.git/**)"]
    elif shape == "product-readable":
        # As product, but no Read rule on the work folder: a deny rule has no allowRead-style exception,
        # so that rule also refused the agent's own Read and ls/cat on its working copy (S6L run 1).
        deny = ["/mnt", str(P), str(HOME / "steplet-work"), str(M)]
        fs = {"denyRead": deny, "allowRead": [str(work)], "denyWrite": [str(work / ".git")]}
        rules = [f"Read(/{p}/**)" for p in deny if p != str(HOME / "steplet-work")] + [f"Edit(/{work}/.git/**)"]
    elif shape == "product-noallow":
        deny = ["/mnt", str(P), str(M)]
        fs = {"denyRead": deny, "denyWrite": [str(work / ".git")]}
        rules = [f"Read(/{p}/**)" for p in deny] + [f"Edit(/{work}/.git/**)"]
    else:
        raise SystemExit("unknown shape " + shape)
    return {"sandbox": {"enabled": True, "failIfUnavailable": True, "allowUnsandboxedCommands": False,
                        "autoAllowBashIfSandboxed": False, "filesystem": fs,
                        "network": {"allowedDomains": [], "strictAllowlist": True}},
            "permissions": {"deny": rules}}


def prepare(run_id):
    prof, work = load(run_id)
    run, win_run = P / "runs" / run_id, WIN / "runs" / run_id
    token = uuid.uuid4().hex
    ctx = {"WORK": str(work), "P": str(P), "M": str(M), "ID": run_id, "HOME": str(HOME), "HOST_IP": host_ip()}
    live = prof["script"] == "live"
    if live and any("API_KEY" in e for e in prof.get("env_extra", [])):
        raise SystemExit("live runs take the real key only through apiKeyHelper, never on a command line")

    if prof.get("workcopy", "fixture") == "fixture":
        shutil.rmtree(work.parent, ignore_errors=True)
        work.parent.mkdir(parents=True)
        r = git("clone", "--quiet", M / "fixture", work)
        if r.returncode:
            raise SystemExit(r.stderr)
        git("-C", work, "checkout", "--quiet", "main")
        key = (M / "canary/fake_key").read_bytes()
        (work / "tests/m0_config.json").write_text(json.dumps({
            "canary_path": str(M / "canary/fake_key"), "canary_sha256": hashlib.sha256(key).hexdigest(),
            "outside_path": str(M / "outside/written_by_probe.txt"), "system_file": "/mnt/c/Windows/win.ini",
            "listener_port": 47831, "external_host": "1.1.1.1", "external_port": 443,
            "wsl_windows_marker": "C:\\dev\\m1\\outside\\from_wsl.txt", "noisy": False}, indent=2))
    for name in prof.get("probes", []):
        (work / "spike_probes").mkdir(exist_ok=True)
        shutil.copy(SPIKES / "probes" / name, work / "spike_probes" / name)

    for folder in (M / "outside", WIN / "outside"):
        for f in folder.glob("*"):
            f.unlink()
    old = list((M / "logs").glob(f"{run_id}.*"))
    old += [WIN / "logs" / f"{run_id}{s}" for s in (".jsonl", ".stderr.txt", ".gate.log", ".gate.log.crashed", ".host-gate.log")]
    for f in old:
        f.unlink(missing_ok=True)

    settings = merge(settings_for(prof["shape"], work), render(prof.get("settings_extra", {}), ctx))
    if prof["gate"] == "node":
        server = {"type": "stdio", "command": "/usr/bin/node", "args": [str(M / "tools/gate/gate.mjs")],
                  "env": {"GATE_MODE": prof.get("gate_mode", "allow"), "GATE_LOG": str(M / "logs" / f"{run_id}.gate.log")}}
    else:
        server = {"type": "stdio", "command": str(SPIKES / "target/release/spike-gate.exe"),
                  "args": ["--pipe", f"steplet-spike-{run_id}", "--token", token, "--log", f"C:\\dev\\m1\\logs\\{run_id}.gate.log"]
                  + (["--crash-once"] if prof.get("gate_crash_once") else [])}

    fake_key = (P / "secrets/fake-api-key").read_text().strip()
    env = [f"HOME={P / 'home'}", "PATH=/usr/local/bin:/usr/bin:/bin", "LANG=C.UTF-8", f"CLAUDE_CONFIG_DIR={P / 'claude-config'}",
           "DISABLE_AUTOUPDATER=1", "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1"]
    if not live:
        env.append(f"ANTHROPIC_BASE_URL=http://127.0.0.1:{FAKE_MODEL_PORT}")
    env += [render(e, ctx).replace("@fake-api-key", fake_key) for e in prof.get("env_extra", [])]
    argv = render(prof.get("argv_override"), ctx) or (
        prof.get("argv_prefix", [])
        + ["/usr/bin/timeout", "-k", "10", str(prof.get("timeout_s", 600)), "/usr/bin/env", "-i", *env, str(CLAUDE),
           "-p", "--output-format", "stream-json", "--verbose", "--include-hook-events",
           "--max-turns", str(prof.get("max_turns", 12)), "--setting-sources", "user", "--strict-mcp-config",
           "--settings", str(run / "settings.json"), "--mcp-config", str(run / "mcp.json"),
           "--permission-prompt-tool", "mcp__gate__decide", *render(prof.get("flags", []), ctx)])

    run.mkdir(parents=True, exist_ok=True)
    (run / "settings.json").write_text(json.dumps(settings, indent=2))
    (run / "mcp.json").write_text(json.dumps({"mcpServers": {"gate": server}}, indent=2))
    if prof["script"] not in ("live", "none"):
        script = render(json.loads((SPIKES / "scripts" / prof["script"]).read_text(encoding="utf-8")), ctx)
        (M / "logs" / f"{run_id}.script.json").write_text(json.dumps(script, indent=2))
    win_run.mkdir(parents=True, exist_ok=True)
    for f in ("services.ready", "stop"):
        (win_run / f).unlink(missing_ok=True)
    (win_run / "argv.txt").write_text("\n".join(argv) + "\n")
    (win_run / "task.txt").write_text(render(prof.get("task", "Follow the spike script."), ctx))
    (win_run / "token.txt").write_text(token)
    (win_run / "cd.txt").write_text(str(work))
    (M / "logs" / f"{run_id}.before.json").write_text(json.dumps(snapshot(work), indent=2))

    def masked(text):
        return text.replace(fake_key, "<fake API key>").replace(token, "<run token>")
    print("--- working copy:", work, "HEAD", git("-C", work, "rev-parse", "HEAD").stdout.strip())
    print("--- settings.json:")
    print(json.dumps(settings, indent=2))
    print("--- MCP server:", masked(json.dumps(server)))
    print("--- launch command, one argument per line:")
    print(masked("\n".join(argv)))
    print("--- task text:", (win_run / "task.txt").read_text())


def show(title, path, limit=60):
    print(f"--- {title}:")
    if not (path.exists() and path.stat().st_size):
        print("    (none)")
        return
    lines = path.read_text(errors="replace").splitlines()
    for line in lines[:limit]:
        print("    " + line[:400])
    if len(lines) > limit:
        print(f"    ... {len(lines) - limit} more lines")


def processes(run_id, work):
    out = subprocess.run(["ps", "-eo", "pid,ppid,sid,etimes,args"], capture_output=True, text=True).stdout.splitlines()
    marks = (f"runs/{run_id}/", str(work), "bwrap", "socat", "spike-gate", "_alive.txt", "claude-cli")
    rows = [line for line in out[1:] if any(m in line for m in marks) and "spike.py" not in line]
    print(f"--- processes still running for this run: {len(rows)}")
    for r in rows:
        print("    " + r[:300])


def evidence(run_id):
    prof, work = load(run_id)
    logs, wlogs = M / "logs", WIN / "logs"
    stream = wlogs / f"{run_id}.jsonl"
    if prof.get("evidence") == "raw":
        show("what the command printed", stream, limit=200)
        show("its error output", wlogs / f"{run_id}.stderr.txt", 30)
        return
    if prof.get("evidence") == "env":
        print("--- the environment the command received (sorted; values shown only for harmless names):")
        lines = stream.read_text(errors="replace").splitlines() if stream.exists() else []
        for line in sorted(lines):
            name, _, value = line.partition("=")
            print(f"    {name}={value if name in SHOW_VALUE else '<value not shown>'}")
        print(f"    ({len(lines)} variables)")
        show("its error output", wlogs / f"{run_id}.stderr.txt", 30)
        return

    print("--- event stream, read with a JSON parser:")
    sys.stdout.flush()
    subprocess.run(["/usr/bin/python3", "-I", str(SPIKES / "tools/parse_claude_stream.py"), str(stream), str(M / "canary/fake_key")])
    show("error output", wlogs / f"{run_id}.stderr.txt", 30)
    gate_logs = [logs / f"{run_id}.gate.log"] if prof["gate"] == "node" else [wlogs / f"{run_id}.gate.log", wlogs / f"{run_id}.host-gate.log"]
    for g in gate_logs:
        show(f"gate log {g.name}", g)
    gate_text = "".join(g.read_text(errors="replace") for g in gate_logs if g.exists())

    print("--- every tool call the agent made, and whether it asked the checker first:")
    for line in stream.read_text(errors="replace").splitlines() if stream.exists() else []:
        try:
            ev = json.loads(line)
        except ValueError:
            continue
        if ev.get("type") != "assistant":
            continue
        for c in (ev.get("message") or {}).get("content") or []:
            if c.get("type") == "tool_use":
                inp = c.get("input") or {}
                what = str(inp.get("command") or inp.get("file_path") or inp.get("url") or json.dumps(inp)[:80])
                asked = json.dumps(what)[1:-1] in gate_text
                print(f"    {c.get('name'):10} asked={'yes' if asked else 'NO '}  {what[:160]}")

    model_log = logs / f"{run_id}.model.log"
    if model_log.exists():
        print("--- scripted model: every request it received:")
        entries = [json.loads(line) for line in model_log.read_text().splitlines() if line.strip()]
        t0 = entries[0]["time"] if entries else 0
        loop_times = []
        for e in entries:
            if e.get("kind") == "agent loop":
                loop_times.append(e["time"])
            print("    +%.2fs %s %s step=%s answer=%s model=%s key_sent=%s key_is_test_key=%s bearer=%s tools=%d" % (
                e["time"] - t0, e.get("kind", e.get("method")), e.get("path", "")[:40], e.get("step"), e.get("answer"),
                e.get("model"), e.get("api_key_sent"), e.get("api_key_matches_test_key"), e.get("bearer_sent"),
                len(e.get("tools_offered") or [])))
        first = next((e for e in entries if e.get("kind") == "agent loop"), None)
        if first:
            print("    tools offered to the model on the first agent request:", ",".join(first["tools_offered"]))
        print("    seconds between agent requests:", [round(b - a, 2) for a, b in zip(loop_times, loop_times[1:])])

    show("m0_results.jsonl (the fixture's own record)", work / "m0_results.jsonl")
    show("listener inside WSL (127.0.0.1:47831)", logs / f"{run_id}.listener.log")
    print("--- marker files: WSL", sorted(p.name for p in (M / "outside").glob("*")) or "none",
          "| Windows", sorted(p.name for p in (WIN / "outside").glob("*")) or "none")
    print("--- working copy HEAD:", git("-C", work, "rev-parse", "HEAD").stdout.strip())
    print("--- git status --short --ignored:")
    print("    " + git("-C", work, "status", "--short", "--ignored").stdout.strip().replace("\n", "\n    "))
    print("--- files in .git/hooks:", sorted(p.name for p in (work / ".git/hooks").glob("*")) if (work / ".git/hooks").exists() else "no such folder")

    before = json.loads((logs / f"{run_id}.before.json").read_text())
    after = snapshot(work)
    print("--- new files in Claude Code's private settings folder:", sorted(set(after["config_files"]) - set(before["config_files"])))
    print("--- new files in Steplet's private home:", sorted(set(after["private_home_files"]) - set(before["private_home_files"])))
    ignore = P / "home/.config/git/ignore"
    print("--- private home .config/git/ignore:", repr(ignore.read_text()) if ignore.exists() else "absent")
    print("--- real ~/.config/git/ignore: before", repr(before["real_git_ignore"]), "| after", repr(after["real_git_ignore"]))
    print("--- Claude Code's record for this folder: before", json.dumps(before["trust_record"]), "| after", json.dumps(after["trust_record"]))

    seen = work / "s3_seen"
    if seen.exists():
        env_file = seen / "env.txt"
        lines = env_file.read_text(errors="replace").splitlines() if env_file.exists() else []
        print("--- environment variable names inside the fence:", ",".join(sorted(l.split("=", 1)[0] for l in lines if "=" in l)))
        odd = [l.split("=", 1)[0] for l in lines if l.split("=", 1)[0] in NOT_LINUX or "/mnt/c" in l or ":\\" in l]
        print("--- Windows or WSL variables inside the fence:", odd or "none")
        cmdlines = {f.name.split("_")[1]: f.read_bytes() for f in seen.glob("proc_*_cmdline.txt")}
        claude_pids = [pid for pid, c in cmdlines.items() if b"--permission-prompt-tool" in c]
        readable_env = [pid for pid in claude_pids if (seen / f"proc_{pid}_environ.txt").exists()]
        print(f"--- /proc: command lines readable from the fence: {len(cmdlines)}; of processes carrying Claude Code's flags: "
              f"{len(claude_pids)}; their environment readable: {len(readable_env)}")
        show("files the probe tried to read", seen / "reads.txt")

    ps_file = work / "s1_ps.txt"
    if ps_file.exists():
        token = (WIN / "runs" / run_id / "token.txt").read_text().strip()
        text = ps_file.read_text(errors="replace")
        print(f"--- the fence's own process list: {len(text.splitlines())} lines; shows the gate: {'spike-gate' in text}; shows the run token: {token in text}")
    show("S8 connection attempts", work / "s8_results.jsonl")

    keys = {"fake API key": (P / "secrets/fake-api-key").read_bytes().strip()}
    real = P / "secrets/anthropic-api-key"
    if real.exists():
        keys["real API key"] = real.read_bytes().strip()
    places = [stream, wlogs / f"{run_id}.stderr.txt", model_log, *gate_logs, ps_file]
    places += sorted(p for p in seen.rglob("*") if p.is_file()) if seen.exists() else []
    print("--- API key text found (counts only; the text is never printed):")
    for label, key in keys.items():
        for part, needle in ((label, key), (label + ", last 20 characters", key[-20:])):
            hits = {p.name: p.read_bytes().count(needle) for p in places if p.is_file() and needle in p.read_bytes()}
            print(f"    {part}: {sum(hits.values())} {hits or ''}")

    if prof.get("grep_debug"):
        words = ("sandbox", "violation", "denied", "EPERM", "Read-only file system", "Network is unreachable", "blocked")
        new_files = [P / "claude-config" / f for f in sorted(set(after["config_files"]) - set(before["config_files"]))]
        print("--- lines about blocked actions in error output and in new files of the settings folder (first 40):")
        found = 0
        for path in [wlogs / f"{run_id}.stderr.txt", *new_files]:
            if not path.is_file():
                continue
            for line in path.read_text(errors="replace").splitlines():
                if found < 40 and any(w.lower() in line.lower() for w in words):
                    print(f"    {path.name}: {line[:240]}")
                    found += 1
        print(f"    ({found} shown)")
    processes(run_id, work)


def cleanup(run_id):
    prof, work = load(run_id)
    marker = f"runs/{run_id}/"

    def marked():
        out = subprocess.run(["ps", "-eo", "pid,sid,args"], capture_output=True, text=True).stdout.splitlines()[1:]
        return [(int(l.split()[0]), int(l.split()[1])) for l in out if marker in l and "spike.py" not in l]

    found = marked()
    sessions = sorted({sid for _, sid in found}) if prof.get("cleanup_by_session") else []
    print(f"--- clean-up: {len(found)} processes carry the run marker; sessions to stop: {sessions or 'none'}")
    for sig in (signal.SIGTERM, signal.SIGKILL):
        for pid, _ in marked():
            try:
                os.kill(pid, sig)
            except ProcessLookupError:
                pass
        for sid in sessions:
            subprocess.run(["pkill", "-" + sig.name[3:], "-s", str(sid)])
        time.sleep(5 if sig == signal.SIGTERM else 1)
    processes(run_id, work)


if __name__ == "__main__":
    {"prepare": prepare, "evidence": evidence, "cleanup": cleanup}[sys.argv[1]](sys.argv[2])
