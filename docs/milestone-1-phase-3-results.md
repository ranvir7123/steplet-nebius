# Milestone 1, phase 3: results

Run from 13 Oct 2026 in the worktree `.claude\worktrees\e3-layout-phase3-plan-8f4432` (branch `claude/e3-layout-phase3-plan-8f4432`), from `b5b2d3c` (the approved plan, `docs/plans/2026-10-13-milestone-1-phase-3.md`). Every command below was run as written, and its output is pasted unchanged unless a line says otherwise.

## Task 1: build Steplet's Linux and prove the layout holds

### The account this runs under

The session's token, from PowerShell. `Administrators` is present only as "deny only" and the level is Medium, so every step below that succeeds needed no administrator rights.

```
> whoami; whoami /groups | Select-String -Pattern "Mandatory Label|Administrators"
laptop-r1rg22o7\ranvir
NT AUTHORITY\Local account and member of Administrators group Well-known group S-1-5-114    Group used for deny only
BUILTIN\Administrators                                        Alias            S-1-5-32-544 Group used for deny only
Mandatory Label\Medium Mandatory Level                        Label            S-1-16-8192
```

### The phase 0 stand-ins (Checks 1 and 5 use them until Task 8 builds the real gate)

```
> cargo build --release --offline -p spike-gate -p spike-host     (in spikes\, last 3 lines)
   Compiling tokio v1.53.1
   Compiling spike-host v0.0.0 (C:\dev\runFile\.claude\worktrees\e3-layout-phase3-plan-8f4432\spikes\host)
    Finished `release` profile [optimized] target(s) in 14.51s
> Get-ChildItem target\release\spike-*.exe | Select-Object Name, Length
spike-gate.exe  297472
spike-host.exe  903680
```

### Installing Steplet's Linux (Z1), without administrator rights

```
> $S = "$env:LOCALAPPDATA\Steplet"; New-Item -ItemType Directory -Force "$S\gate", "$S\wsl", "$S\logs"; Copy-Item spike-gate.exe "$S\gate\"
> wsl.exe --install Ubuntu-26.04 --name steplet --location "$S\wsl" --no-launch
Downloading: Ubuntu 26.04 LTS
Installing: Ubuntu 26.04 LTS
Distribution successfully installed. It can be launched via 'wsl.exe -d steplet'
exit code: 0; seconds: 190
> wsl.exe -l -v
NAME            STATE           VERSION
* Ubuntu          Stopped         2
  steplet         Stopped         2
  steplet-test    Stopped         2
```

No administrator prompt appeared, and the session's token has `Administrators` as "deny only" (above). **Check 2: PASS** for an administrator account running unelevated; a true standard (non-administrator) Windows account was not tried.

### Setup as root inside it (`crates/steplet-run/tests/support/setup_steplet.sh`)

```
> wsl.exe -d steplet -u root --exec /bin/bash .../crates/steplet-run/tests/support/setup_steplet.sh C:/Users/RANVIR/AppData/Local/Steplet/gate
--- /etc/wsl.conf: Windows drives not mounted, Windows PATH not added, default user steplet
[boot]
systemd=true

[automount]
enabled=false
mountFsTab=true

[interop]
enabled=true
appendWindowsPath=false

[user]
default=steplet
--- /etc/fstab: the gate folder only, read-only
C:/Users/RANVIR/AppData/Local/Steplet/gate /opt/steplet/gate drvfs ro,noatime 0 0
--- the steplet account: no password, no sudo, no login shell
passwd: password changed.
uid=999(steplet) gid=986(steplet) groups=986(steplet)
--- base tools (choice H4-A)
Failed to connect to system scope bus via local transport: Connection refused

Processing triggers for man-db (2.13.1-1build1) ...
git: 1:2.53.0-1ubuntu1
python3: 3.14.3-0ubuntu2
build-essential: 12.12ubuntu2.26.04.2
bubblewrap: 0.11.1-1ubuntu0.3
socat: 1.8.1.1-1ubuntu0.1
nodejs: 22.22.1+dfsg+~cs22.19.15-1ubuntu1
npm: 9.2.0~ds3-1
--- Claude Code 2.1.220 into /opt/steplet/claude-cli

added 3 packages in 2m
--- srt 0.0.77 into /opt/steplet/srt

added 5 packages in 5s
--- Steplet's folders, owned by root; nothing under /opt/steplet writable by steplet
--- what was installed
claude runs: /opt/steplet/claude-cli/lib/node_modules/@anthropic-ai/claude-code/bin/claude.exe
claude sha256: 674f61f20ff306f3100cf9200e4c36c4b70278b5bef2884549819b942a89c863
claude version: 2.1.220 (Claude Code)
srt package: 0.0.77
drwxr-xr-x 5 root root 4096 Oct 13 12:33 /opt/steplet
drwxr-xr-x 4 root root 4096 Oct 13 12:33 /opt/steplet/claude-cli
drwxr-xr-x 2 root root 4096 Oct 13 12:26 /opt/steplet/gate
drwxr-xr-x 3 root root 4096 Oct 13 12:33 /opt/steplet/srt
drwxr-xr-x 3 root root 4096 Oct 13 12:33 /var/lib/steplet
drwxr-xr-x 2 root root 4096 Oct 13 12:33 /var/lib/steplet/runs
drwxr-xr-x 2 root root 4096 Oct 13 12:33 /work
--- setup finished
exit code: 0; seconds: 445
```

The "system scope bus" line comes from a package's install step: systemd was not running during this first start, which was made with `--exec` before any restart. Nothing failed (every version line printed, exit code 0).

### After a restart (`wsl --terminate steplet`), as the `steplet` account: Checks 3 and 4, and the gate folder

Script: `layout_checks.sh` (scratchpad; read-only), sent on standard input with `cmd /c "wsl.exe -d steplet -u steplet --exec /bin/bash -s < layout_checks.sh"`.

```
--- who: uid=999(steplet) gid=986(steplet) groups=986(steplet)
--- /etc/wsl.conf in effect: automount enabled=false
--- /mnt holds: c wsl wslg
--- /mnt/c: PRESENT
--- Windows-backed mounts (9p, drvfs, virtiofs):
    /usr/lib/wsl/drivers 9p ro,nosuid,nodev,noatime,aname=drivers;fmask=222;dmask=222,cache=5,access=client,msize=65536,trans=fd,rfd=8,wfd=8
--- gate folder:
    total 8
    drwxr-xr-x 2 root root 4096 Oct 13 12:26 .
    drwxr-xr-x 5 root root 4096 Oct 13 12:33 ..
--- write into the gate folder: touch: cannot touch '/opt/steplet/gate/probe': Permission denied
--- /home holds: []
--- /root readable: no
--- /mnt/wsl:
    total 8
    drwxrwxrwt 2 root root   60 Oct 13 17:02 .
    drwxr-xr-x 5 root root 4096 Oct 13 12:26 ..
    -rw-r--r-- 1 root root  200 Oct 13 17:02 resolv.conf
--- /mnt/wslg (first 12 lines):
    total 44
    drwxrwxrwt  7 root root   300 Oct 13 17:03 .
    drwxr-xr-x  5 root root  4096 Oct 13 12:26 ..
    drwxrwxrwx  2 root root    60 Oct 13 17:02 .X11-unix
    srwxrwxrwx  1 1000 1000     0 Oct 13 17:03 PulseAudioRDPSink
    srwxrwxrwx  1 1000 1000     0 Oct 13 17:02 PulseAudioRDPSource
    srwxrwxrwx  1 1000 1000     0 Oct 13 17:02 PulseServer
    drwxr-xr-x 20 root root  4096 Oct 13 17:02 distro
    drwxr-xr-x 65 root root  4096 Oct  6  2025 doc
    -rw-------  1 1000 1000  1371 Oct 13 17:03 pulseaudio.log
    drwxr-xr-x  3 root root    60 Oct 13 17:02 run
    drwxrwxrwx  3 1000 1000   100 Oct 13 17:02 runtime-dir
--- /mnt/wslg/distro readable: yes
--- a Windows program by the usual drive path: not reachable
--- interop: start the stand-in gate with no arguments (it should start on Windows and stop on the missing --pipe)
    timeout: failed to execute process: No such file or directory (os error 2)
    exit code: 127
--- bash outside the fence: 5.3.9(1)-release
```

`/mnt/c` is only the empty mount point left from the first start (`ls -A /mnt/c` printed nothing, below); no drive is mounted. **Check 3: PASS**: no Windows drive is mounted, `cmd.exe` is not reachable, `/home` is empty, and `/root` is closed to `steplet`. The only Windows-backed mount is WSL's own read-only GPU driver folder.

**Check 4: recorded.** `/mnt/wsl` holds only `resolv.conf`. `/mnt/wslg` (WSL's graphics and sound bridge) is readable by `steplet`: its sockets are world-writable, and its `distro` folder is WSLg's own system files. No receiver data was seen there. Per H1, Steplet unmounts both inside its Linux at start-up; that is built and measured with the run preparation (Task 6).

### The gate folder did not mount: WSL 2.6.3 mounts drive roots only

```
--- /mnt/c is an empty folder: []
--- fstab as findmnt reads it:
    /opt/steplet/gate C:/Users/RANVIR/AppData/Local/Steplet/gate drvfs ro,noatime
--- mount helper: lrwxrwxrwx 1 root root 5 Oct 13 12:26 /sbin/mount.drvfs -> /init;lrwxrwxrwx 1 root root 5 Oct 13 12:26 /usr/sbin/mount.drvfs -> /init;
--- journal lines about it:
    Oct 13 17:04:03 LAPTOP-R1RG22O7 mount[162]: mount: /opt/steplet/gate: special device C:/Users/RANVIR/AppData/Local/Steplet/gate does not exist.
    Oct 13 17:04:03 LAPTOP-R1RG22O7 systemd[1]: Failed to mount opt-steplet-gate.mount - /opt/steplet/gate.
--- manual: mount /opt/steplet/gate
    mount: /opt/steplet/gate: special device C:/Users/RANVIR/AppData/Local/Steplet/gate does not exist.
    <3>WSL (447 - ) ERROR: UtilCreateProcessAndWait:709: /bin/mount failed with status 0x2000
    exit code: 32
```

The same with backslashes, by hand and as an fstab line written with `\134`:

```
--- manual mount with backslashes
    mount: /opt/steplet/gate: special device C:\Users\RANVIR\AppData\Local\Steplet\gate does not exist.
    <3>WSL (430 - ) ERROR: UtilCreateProcessAndWait:709: /bin/mount failed with status 0x2000
    exit code: 32
--- fstab line with \134 for each backslash
    C:\134Users\134RANVIR\134AppData\134Local\134Steplet\134gate /opt/steplet/gate drvfs ro,noatime 0 0
    mount: /opt/steplet/gate: special device C:\Users\RANVIR\AppData\Local\Steplet\gate does not exist.
    mount by the fstab line, exit code: 32
```

Whether drvfs works at all here, then the fstab line removed:

```
--- kernel messages from the failed mounts (last 6):
    [    1.043124] WSL (4 - ) ERROR: UtilCreateProcessAndWait:709: /bin/mount failed with status 0x2000
    [    1.045050] WSL (2 - init-systemd(steplet)) ERROR: ConfigMountFsTab:2145: Processing /etc/fstab with mount -a failed.
--- does drvfs work at all here? mount the drive root C: read-only at a temporary place, then unmount
    exit code: 0; mounted: 9p
--- removing the fstab line
    lines left that mention steplet: 0
```

A whole drive mounts; a single folder does not. Mounting all of `C:` would undo option D, so the plan's fallback was taken: **the gate program lives on the Linux disk, owned by root.** Copied with `dd` on standard input (no Windows folder mounted):

```
> cmd /c "wsl.exe -d steplet -u root --exec /usr/bin/dd of=/opt/steplet/gate/spike-gate.exe status=none < %LOCALAPPDATA%\Steplet\gate\spike-gate.exe"
copy exit code: 0
> wsl.exe -d steplet -u root --exec /usr/bin/chmod 755 /opt/steplet/gate/spike-gate.exe
> (Get-FileHash spike-gate.exe -Algorithm SHA256).Hash; wsl.exe ... sha256sum /opt/steplet/gate/spike-gate.exe; ls -l
8d3cf58e5f3f528a9323c4f9a1acee64c844671a9d399ecdd410684481d25cf8
8d3cf58e5f3f528a9323c4f9a1acee64c844671a9d399ecdd410684481d25cf8  /opt/steplet/gate/spike-gate.exe
-rwxr-xr-x 1 root root 297472 Oct 13 17:06 spike-gate.exe
```

Interop starts it from there, as `steplet`, after a restart. The Windows program runs and stops on its missing `--pipe` option, exit code 101 (a Rust panic on Windows):

```
> wsl.exe --terminate steplet
terminate exit code: 0
> cmd /c "wsl.exe -d steplet -u steplet --cd /tmp --exec /usr/bin/timeout 20 /opt/steplet/gate/spike-gate.exe < NUL"
wsl: Failed to start the systemd user session for 'steplet'. See journalctl for more details.

thread 'main' (32968) panicked at gate\src\main.rs:48:54:
--pipe
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
gate exit code: 101
```

The "systemd user session" warning is the one phase 0 saw on most calls (`steplet` has no home folder); it never stopped a run there. The "/etc/fstab" warning is gone after the restart.

**Change from the plan (recorded):** the gate is reached as a root-owned copy on the Linux disk, not a read-only mount. This is stricter: no Windows folder at all is visible inside Steplet's Linux. Setup copies the gate program in with `dd`, and before each run it compares the copy's SHA-256 with the Windows file.

### Check 1: Claude Code as `steplet` starts the gate, and every write reaches it

Probe files: `spikes/p3/` (`check1.ps1` drives it; `prepare_p1.sh` makes the run's folders in the section 3 layout as root; `p1.settings.json` is section 3's settings for run `p1`; `argv.txt` is section 3's launch command plus `ANTHROPIC_BASE_URL` for the scripted model; `p1.script.json` is five writes and one shell command). The host stand-in creates the pipe and starts `wsl.exe` with `SystemRoot` only; the scripted model runs as root inside Steplet's Linux. Run with `powershell -NoProfile -ExecutionPolicy Bypass -File spikes\p3\check1.ps1`.

**Run 1** (as the plan wrote it, `.git` handed to root after the copy was made). Five writes asked the gate and were written; the shell command asked the gate, then failed inside the fence:

```
--- prepared: HEAD
    drwxr-xr-x 7 root    root    4096 Oct 13 17:10 .git
--- launch through the host stand-in (wsl.exe gets SystemRoot only)
host: pipe \\.\pipe\steplet-p3-p1 created, gate mode allow
host: wsl.exe started (pid Some(15212), environment SystemRoot); task sent on its input, 24 bytes
host: wsl.exe ended: Ok(Some(0)), after 12.2 s
INIT version=2.1.220 model=claude-opus-5[1m] permissionMode=default cwd=/work/p1/repo
     mcp_servers=[{"name": "gate", "status": "connected"}]
     tools=Bash,Edit,Glob,Grep,Read,WebFetch,Write
...
TOOL_USE Bash: {"command": "echo \"shell=$(ps -o comm= -p $$) bash=$BASH_VERSION\"", "description": "Show the shell"}
TOOL_RESULT is_error=True:
    | Exit code 1
    | bwrap: Can't create file at /work/p1/repo/.git/config.lock: Permission denied
```

(`HEAD` is empty in that line because root ran `git` on a copy owned by `steplet`, and git refused it as "dubious ownership"; the probe now runs that `git` as `steplet`. Windows' `tar` also printed `Ignoring unknown extended header keyword 'SCHILY.fflags'` once per file; harmless.)

**Change from the plan (recorded): choice H7 is reverted.** The fence places a guard file inside `.git` before each command, so with `.git` owned by root every sandboxed command fails. `.git` stays owned by `steplet` and is guarded as phase 0 S4 measured: the fence's `denyWrite` for commands, and the `Edit(//…/.git/**)` deny rule for the agent's own tools. Copies are deleted after every run and Steplet's own `git` calls are hardened, so a changed `.git` is never used again (D6).

**Run 2** (`.git` owned by `steplet`):

```
=== P3-1 started 2026-10-13T22:40:59.7120429+05:30; distribution steplet {44f36236-99b1-42c0-84bd-7ff8392d51ba}
--- prepared: HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50
    /var/lib/steplet/runs/p1:
    drwx------ 2 steplet steplet 4096 Oct 13 17:11 config
    drwx------ 2 steplet steplet 4096 Oct 13 17:11 home
    -rw-r--r-- 1 root    root     801 Oct 13 17:11 mcp.json
    -rw-r--r-- 1 root    root     603 Oct 13 17:11 settings.json
    drwx------ 2 steplet steplet 4096 Oct 13 17:11 tmp
    /work/p1:
    drwxr-xr-x 3 root    root    4096 Oct 13 17:11 .
    drwxr-xr-x 5 steplet steplet 4096 Oct 13 17:10 repo
--- /work holds: p1
--- scripted model
798 /usr/bin/python3 -I /opt/steplet-test/fake_model.py 47900 /opt/steplet-test/p1.script.json /opt/steplet-test/p1.model.log
--- launch through the host stand-in (wsl.exe gets SystemRoot only)
host: pipe \\.\pipe\steplet-p3-p1 created, gate mode allow
host: wsl.exe started (pid Some(4040), environment SystemRoot); task sent on its input, 24 bytes
host: wsl.exe ended: Ok(Some(0)), after 1.2 s
--- Windows: spike-gate.exe processes still running: 0
--- event stream, read with a JSON parser:
bytes: 9789, lines: 15, raw 0x1B bytes: 0
INIT version=2.1.220 model=claude-opus-5[1m] permissionMode=default cwd=/work/p1/repo
     mcp_servers=[{"name": "gate", "status": "connected"}]
     tools=Bash,Edit,Glob,Grep,Read,WebFetch,Write
TOOL_USE Write: {"file_path": "/work/p1/repo/p3_marker_01.txt", "content": "p3\n"}
TOOL_RESULT is_error=None:
    | File created successfully at: /work/p1/repo/p3_marker_01.txt (file state is current in your context - no need to Read it back)
    (the same for markers 02 to 05)
TOOL_USE Bash: {"command": "echo \"shell=$(ps -o comm= -p $$) bash=$BASH_VERSION\"", "description": "Show the shell"}
TOOL_RESULT is_error=False:
    | shell=bash bash=5.3.9(1)-release
AGENT: P3 check 1: done.
RESULT subtype=success is_error=False num_turns=7 duration_ms=267
       permission_denials=[]
event types: {'system/init': 1, 'assistant': 7, 'user': 6, 'result/success': 1}
top-level events of type 'result': 1
--- error output:
--- gate log (the stand-in gate, a Windows program started inside Linux):
1791911466.495 pid=46684 started; 1 environment variables: SystemRoot
1791911466.611 pid=46684 tool="Write" behavior="allow" ms=1 input={"content":"p3\n","file_path":"/work/p1/repo/p3_marker_01.txt"}
1791911466.637 pid=46684 tool="Write" behavior="allow" ms=0 input={"content":"p3\n","file_path":"/work/p1/repo/p3_marker_02.txt"}
1791911466.656 pid=46684 tool="Write" behavior="allow" ms=0 input={"content":"p3\n","file_path":"/work/p1/repo/p3_marker_03.txt"}
1791911466.674 pid=46684 tool="Write" behavior="allow" ms=0 input={"content":"p3\n","file_path":"/work/p1/repo/p3_marker_04.txt"}
1791911466.691 pid=46684 tool="Write" behavior="allow" ms=0 input={"content":"p3\n","file_path":"/work/p1/repo/p3_marker_05.txt"}
1791911466.722 pid=46684 tool="Bash" behavior="allow" ms=0 input={"command":"echo \"shell=$(ps -o comm= -p $$) bash=$BASH_VERSION\"","description":"Show the shell"}
--- host gate log:
1791911466.611 tool="Write" token_ok=true mode=allow behavior="allow" ms=0
(five more lines: Write x4, Bash, all token_ok=true behavior="allow")
1791911466.876 tasklist:
"Image Name","PID","Session Name","Session#","Mem Usage","Status","User Name","CPU Time","Window Title"
"spike-gate.exe","46684","Console","1","3,996 K","Unknown","LAPTOP-R1RG22O7\RANVIR","0:00:00","N/A"
--- every tool call, and whether it asked the gate:
    Write  asked=yes  /work/p1/repo/p3_marker_01.txt
    Write  asked=yes  /work/p1/repo/p3_marker_02.txt
    Write  asked=yes  /work/p1/repo/p3_marker_03.txt
    Write  asked=yes  /work/p1/repo/p3_marker_04.txt
    Write  asked=yes  /work/p1/repo/p3_marker_05.txt
    Bash   asked=yes  echo "shell=$(ps -o comm= -p $$) bash=$BASH_VERSION"
--- markers in the working copy, and who owns .git:
steplet /work/p1/repo/.git
steplet /work/p1/repo/p3_marker_01.txt
steplet /work/p1/repo/p3_marker_05.txt
--- scripted model: requests (kind, step, answer):
    agent loop 0 tool tools=7
    (steps 1 to 5: tool, tools=7)
    agent loop 6 text tools=7
=== P3-1 finished 2026-10-13T22:41:10.0286174+05:30
```

The lines marked "(…)" stand for lines identical to the one above them except for the marker number; the full output is in `C:\dev\m1\logs\P3-1.*`.

**Check 1: PASS.** In Steplet's own Linux, with no Windows drive mounted, Claude Code running as `steplet` started the gate (a Windows program on the Linux disk) through interop: `mcp_servers` shows `gate` `connected`, all six actions asked the gate, the five markers exist, and the gate saw one Windows variable, `SystemRoot`. `--tools` gave the model exactly the seven tools asked for. The gate runs as `LAPTOP-R1RG22O7\RANVIR` in the console session (`tasklist`); its integrity level and logon session are **not** recorded, since `tasklist` does not show them. Task 8's pipe service reads them from the client's token and records them there.

**Check 5: PASS.** Inside the fence the agent's commands run in `bash` 5.3.9(1)-release, the version phase 2's command vectors agree with.

### Task 1 summary

| Check | Result |
|---|---|
| 1. The gate starts with the drives unmounted, and writes reach it | **PASS**, with the gate on the Linux disk (the plan's fallback; a single-folder mount is not possible in WSL 2.6.3) |
| 2. No administrator rights needed | **PASS** for an administrator account running unelevated; a standard account not tried |
| 3. The receiver's files are absent | **PASS** |
| 4. Shared folders | Recorded: `/mnt/wsl` holds `resolv.conf`; `/mnt/wslg` is readable (WSLg's sockets and system files). Unmounted at start-up per H1, built in Task 6 |
| 5. `bash` is the shell | **PASS**: 5.3.9 |

Changes from the plan, both recorded above: the gate is a root-owned copy on the Linux disk instead of a mounted folder, and H7 is reverted (`.git` stays with `steplet`). The stop condition Z4 was not reached.

## Task 2: the key path, and one real-model run

What was added for this task:
- **The stand-ins got a key mode (throwaway spike code).** `spike-gate.exe --key --pipe … --token …` asks the host for the key over the pipe and prints it with no line ending. `spike-host` answers a key request only with the run's token, and logs "served" or not, never the key. Its key comes from `--key-file` (a made-up key) or `--key-wsl steplet-test:<path>` (the test key you saved in phase 0, read straight into memory with `cat`).
- **Settings:** `apiKeyHelper` is `/opt/steplet/gate/spike-gate.exe --key --pipe steplet-p3-p2 --token <run token>` (`spikes/p3/p2.settings.json`).
- **Test support, ported** to `crates/steplet-run/tests/support/`: `fake_model.py` (the scripted model; it now takes the expected key as a SHA-256, so the test never puts the key on the Linux disk), `listener.py`, and `count_key.py` (S3's method: counts the key's text and its last 20 characters, never prints it; `--whole-disk` reads every file on Steplet's Linux root file system, skipping `/proc`, `/sys`, `/dev`, `/run` and `/mnt`).
- **The run:** `spikes/p3/check2.ps1 -Mode scripted|live`. `spikes/p3/prepare.sh` makes the section 3 folders, copies in the milestone 0 fixture, writes its `tests/m0_config.json`, and plants a made-up canary secret in the run's own home (a folder the fence denies). The listener runs as root on 127.0.0.1:47831 inside Steplet's Linux.

### Scripted run (`check2.ps1 -Mode scripted`)

```
=== P3-2s started 2026-10-13T22:45:39.0055377+05:30; distribution steplet {44f36236-99b1-42c0-84bd-7ff8392d51ba}
--- prepared p2: HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50; /work holds: p2
--- canary: steplet 600 42 bytes
--- helpers inside Steplet's Linux: listener on 127.0.0.1:47831, scripted model on 47900
--- launch through the host stand-in (wsl.exe gets SystemRoot only; the key is served over the pipe)
host: pipe \\.\pipe\steplet-p3-p2 created, gate mode allow
host: wsl.exe started (pid Some(43068), environment SystemRoot); task sent on its input, 18 bytes
host: wsl.exe ended: Ok(Some(0)), after 1.4 s
--- Windows: spike-gate.exe processes still running: 0
INIT version=2.1.220 model=claude-opus-5[1m] permissionMode=default cwd=/work/p2/repo
     mcp_servers=[{"name": "gate", "status": "connected"}]
     tools=Bash,Edit,Glob,Grep,Read,WebFetch,Write
TOOL_USE Bash: {"command": "git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"", "description": "Check out the specified commit"}
TOOL_RESULT is_error=False:
    | fatal: Unable to create '/work/p2/repo/.git/index.lock': Read-only file system
    | EXIT=128
TOOL_USE Bash: {"command": "python3 -m unittest discover -s tests -t . 2>&1; echo \"EXIT=$?\"", "description": "Run the unittest suite"}
TOOL_RESULT is_error=False:
    | .
    | Ran 1 test in 0.034s
    | OK
    | EXIT=0
RESULT subtype=success is_error=False num_turns=3 duration_ms=361
top-level events of type 'result': 1
--- host gate log (key requests and decisions; the key itself is never logged):
1791911748.003 key request token_ok=true served=true
1791911748.285 tool="Bash" token_ok=true mode=allow behavior="allow" ms=0
1791911748.410 tool="Bash" token_ok=true mode=allow behavior="allow" ms=0
--- every tool call, and whether it asked the gate:
    Bash   asked=yes  git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo "EXIT=$?"
    Bash   asked=yes  python3 -m unittest discover -s tests -t . 2>&1; echo "EXIT=$?"
--- scripted model: requests, and whether the key it got matches the expected key's SHA-256:
    agent loop 0 tool key_sent=True matches=True
    agent loop 1 tool key_sent=True matches=True
    agent loop 2 text key_sent=True matches=True
--- where the key's text appears (counts only):
    the run's folders inside Steplet's Linux:
key: 0  (files read: 58)
key, last 20 characters: 0  (files read: 58)
    every file on Steplet's Linux disk:
key: 0  (files read: 69141)
key, last 20 characters: 0  (files read: 69141)
    the Windows-side logs of this run:
key: 0  (files read: 4)
key, last 20 characters: 0  (files read: 4)
```

The fixture's own record and the checks made outside the fence were the same in the scripted and the live run; they are pasted once, under the live run below. In this scripted run, `git status` from outside the fence showed four tracked files as modified. My probe had copied the fixture from a clone made on Windows, where git converts line endings. `check2.ps1` now clones with `core.autocrlf=false`, and the live run's copy is clean. A real run clones inside Linux, so this cannot happen there.

### Live run (`check2.ps1 -Mode live`, the real model and the phase 0 test key, Z2)

The task is phase 0's L1 text (`spikes/runs/L1.json`): try the checkout, run the tests either way, answer in three lines, edit nothing.

```
=== P3-2L started 2026-10-13T22:47:45.2650620+05:30; distribution steplet {44f36236-99b1-42c0-84bd-7ff8392d51ba}
--- prepared p2: HEAD 7f4b72d7981454558a3631310426dcff4d4ebd50; /work holds: p2
--- canary: steplet 600 42 bytes
--- helpers inside Steplet's Linux: listener on 127.0.0.1:47831
--- launch through the host stand-in (wsl.exe gets SystemRoot only; the key is served over the pipe)
host: pipe \\.\pipe\steplet-p3-p2 created, gate mode allow
host: wsl.exe started (pid Some(46076), environment SystemRoot); task sent on its input, 347 bytes
host: wsl.exe ended: Ok(Some(0)), after 10.4 s
--- Windows: spike-gate.exe processes still running: 0
--- event stream, read with a JSON parser:
bytes: 9424, lines: 9, raw 0x1B bytes: 0
INIT version=2.1.220 model=claude-opus-5[1m] permissionMode=default cwd=/work/p2/repo
     mcp_servers=[{"name": "gate", "status": "connected"}]
     tools=Bash,Edit,Glob,Grep,Read,WebFetch,Write
AGENT: I'll check out the commit, then run the tests either way.
TOOL_USE Bash: {"command": "git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"", "description": "Check out the specified commit"}
TOOL_RESULT is_error=False:
    | fatal: Unable to create '/work/p2/repo/.git/index.lock': Read-only file system
    | EXIT=128
TOOL_USE Bash: {"command": "git status --short 2>&1 | head -5; git log --oneline -1; python3 -m unittest discover -s tests -t . 2>&1; echo \"EXIT=$?\"", "description": "Run the test suite"}
TOOL_RESULT is_error=False:
    | ?? .bash_profile
    | ?? .bashrc
    | ?? .claude/
    | ?? .gitconfig
    | ?? .gitmodules
    | 7f4b72d m0: readme
    | .
    | Ran 1 test in 0.035s
    | OK
    | EXIT=0
RESULT subtype=success is_error=False num_turns=3 duration_ms=9248
       permission_denials=[]
event types: {'system/init': 1, 'assistant': 5, 'user': 2, 'result/success': 1}
top-level events of type 'result': 1
--- models used, from the result event:
    claude-opus-5[1m] 0.0786 USD
--- error output:
--- gate log:
1791911873.881 pid=43160 started; 1 environment variables: SystemRoot
1791911877.060 pid=43160 tool="Bash" behavior="allow" ms=0 input={"command":"git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo \"EXIT=$?\"",...
1791911880.234 pid=43160 tool="Bash" behavior="allow" ms=0 input={"command":"git status --short 2>&1 | head -5; git log --oneline -1; python3 -m unittest discover -s tests -t . 2>&1; echo \"EXIT=$?\""
--- host gate log (key requests and decisions; the key itself is never logged):
1791911873.766 key request token_ok=true served=true
1791911877.060 tool="Bash" token_ok=true mode=allow behavior="allow" ms=0
1791911880.234 tool="Bash" token_ok=true mode=allow behavior="allow" ms=0
--- every tool call, and whether it asked the gate:
    Bash   asked=yes  git checkout 65930082d222089b2633d34e9bb2bd42351bdc76 2>&1; echo "EXIT=$?"
    Bash   asked=yes  git status --short 2>&1 | head -5; git log --oneline -1; python3 -m unittest discover -s tests -t . 2>&1; echo
--- m0_results.jsonl (the fixture's own record):
{"attempt": "A_write_inside_workspace", "ok": true, "detail": ""}
{"attempt": "B_read_canary", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/var/lib/steplet/runs/p2/home/canary_key'"}
{"attempt": "H_read_system_file", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/mnt/c/Windows/win.ini'"}
{"attempt": "C_write_outside_workspace", "ok": true, "detail": ""}
{"attempt": "D_connect_listener", "ok": false, "detail": "ConnectionRefusedError: [Errno 111] Connection refused"}
{"attempt": "E_connect_external", "ok": false, "detail": "OSError: [Errno 101] Network is unreachable"}
{"attempt": "child_read_canary", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/var/lib/steplet/runs/p2/home/canary_key'"}
{"attempt": "child_connect_listener", "ok": false, "detail": "ConnectionRefusedError: [Errno 111] Connection refused"}
{"attempt": "F_child_process", "ok": true, "detail": "child ran"}
{"attempt": "G_launch_windows_program", "ok": false, "detail": "FileNotFoundError: [Errno 2] No such file or directory: '/mnt/c/Windows/System32/cmd.exe'"}
--- listener inside Steplet's Linux (127.0.0.1:47831): none
--- outside write: absent
--- Windows marker: absent
--- working copy HEAD and status:
## main...origin/main
--- where the key's text appears (counts only):
    the run's folders inside Steplet's Linux:
key: 0  (files read: 59)
key, last 20 characters: 0  (files read: 59)
    every file on Steplet's Linux disk:
key: 0  (files read: 69141)
key, last 20 characters: 0  (files read: 69141)
    the Windows-side logs of this run:
key: 0  (files read: 4)
key, last 20 characters: 0  (files read: 4)
=== P3-2L finished 2026-10-13T22:49:01.7098793+05:30
```

(The agent's three-line answer is left out on purpose: the agent's words are never evidence, F47. It is in `C:\dev\m1\logs\P3-2L.jsonl`.)

**What it shows:**
- **The key path: PASS.** The real service accepted the key (the run completed with the real model) and the scripted model saw the expected key. Yet the key's text appears **0 times** in the run's folders, in every file on Steplet's Linux disk (69,141 files), and in the run's Windows-side logs. The key never touches the Linux disk: it lives in the host's memory and reaches Claude Code only through the gate's `--key` mode. The fallback file of choice H2 is not needed.
- **The real model matches the scripted one:** the same tool, both commands asked the gate first, the fixture's own record is line for line the same, and there is one `result` event. Cost: 0.0786 USD.
- **The fence, from the fixture's own record and the checks outside it:**
  - The canary could not be read, by the test or by its child process. The fence hides the run's home, so the file "does not exist" from inside.
  - No Windows file could be read, and no Windows program could be started: none exist in Steplet's Linux.
  - Both the listener connection and the internet connection failed; the listener logged nothing.
  - `.git` stayed read-only, so the agent's own checkout failed.
  - Nothing was written outside the working copy.
- **One line needs a closer look: `C_write_outside_workspace` says `ok: true`** in both runs, yet the target file (`/var/lib/steplet/runs/p2/tmp/written_by_probe.txt`) does not exist when root looks after the run. Phase 0 got "Read-only file system" here, because its target was not inside a denied folder. My reading is that the fence covers a denied folder with an empty, temporary one, so the write landed there and was thrown away. **Not yet proven**; Task 3 looks at the fence's mounts directly.
- **Inside the fence, `git status` shows five files that do not exist** (`.bash_profile`, `.bashrc`, `.claude/`, `.gitconfig`, `.gitmodules`). From outside, after the run, the copy is clean (`## main...origin/main` and nothing else). These are the fence's own guard placeholders; phase 0 S4b saw the same. For the report (Task 10): changed files come from Steplet's own `git status` after the run, never from what the agent saw inside the fence.

## Task 3: not run yet

Task 3 tests the agent's own file tools against shortcut files that point out of the working copy. While it was being prepared, a safety filter on the assistant's side stopped the work twice. Task 3 is left for later, on the user's word; the tasks after it do not depend on its result to be built. Until it runs, the C line above and the "receiver's files are not even present" promise rest on Task 1 Check 3 (nothing of the receiver's is in Steplet's Linux) and Task 2 (the key is on no Linux disk), not on a direct test of the file tools.

## Task 4: the runner crate, and the checker mapping

**Changes from the plan, both recorded here:**
- **The gate program's crate (`steplet-gate`) is created in Task 8, not now.** Until Task 8 it would be an empty stub.
- **Each shared type is added by the task that first uses it**, with the names and fields of plan section 6: `GateRequest` and `GateAnswer` now, `RunPaths` in Task 5, `RunRequest` in Task 6, `Event` in Task 7, the report types in Task 10. This avoids types with no code behind them.

**One addition: searches.** `decide` refuses a Glob whose `pattern`, or a Grep whose `glob`, starts at `/`, `\` or `~` or contains `..`. A search names a folder (which the checker sees), and its pattern could otherwise reach outside that folder. A search without a folder is refused too, since there is no path to check.

Step 2, the test before the code:

```
> cargo test -p steplet-run --offline --test decide
     Locking 7 packages to latest Rust 1.98 compatible versions
      Adding itoa v1.0.18
      Adding memchr v2.8.3
      Adding serde v1.0.229
      Adding serde_core v1.0.229
      Adding serde_derive v1.0.229
      Adding serde_json v1.0.151
      Adding zmij v1.0.23
error[E0432]: unresolved imports `steplet_run::GateAnswer`, `steplet_run::GateRequest`, `steplet_run::decide`
error: could not compile `steplet-run` (test "decide") due to 1 previous error
```

Every package came from cargo's cache (no download). The 14-day check of the lock file runs at the wrap-up (Task 12).

Step 4, with `src/types.rs`, `src/decide.rs` and `src/lib.rs` written:

```
> cargo test -p steplet-run --offline --test decide
test an_allow_hands_back_exactly_the_request ... ok
test maps_requests_to_the_checker ... ok
test anything_unclear_is_refused_with_a_reason ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
> cargo fmt --all -- --check
Diff in \\?\C:\dev\runFile\.claude\worktrees\e3-layout-phase3-plan-8f4432\crates\steplet-run\tests\decide.rs:36:
(one long line in the test wrapped into three)
fmt exit code: 1
> cargo fmt --all
> cargo fmt --all -- --check
fmt exit code: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings     (last line)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s
> cargo test -p steplet-run --offline
     Running tests\decide.rs (target\debug\deps\decide-081efc8f1b1a45b2.exe)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Task 5: the settings file and MCP config

**Changes from the plan, recorded here:**
- **Two settings beyond section 3's sketch.** Decisions already require both, and both were in the file the Task 2 runs used: `switchModelsOnFlag: false` (E1: the run stays on the model the receiver approved) and `apiKeyHelper` set to the gate's key mode (H2: the key is never written to the Linux disk).
- **The writers take a `GateLink`** (the gate program's Linux path, the pipe name, the run token, the gate's Windows log). `apiKeyHelper` needs the pipe and token, so the interface is `settings_json(s, paths, gate)` and `mcp_json(gate)`. `mcp_json` takes no run paths, because it uses none.
- **`RunPaths::new(distro_id, run_id)` builds section 3's paths and is the only way to make a `RunPaths`.** Both types are read-only after `new` (private fields, read-only accessors with the names of plan section 6). `new` refuses a run id that is not 1 to 64 lower-case letters, digits and `-` (the id becomes part of commands run as root), and a distribution id that is not a braced GUID (it goes on `wsl.exe`'s command line). `GateLink::new` refuses a program, pipe name or token that a shell could misread, because `apiKeyHelper` is a line Claude Code hands to `/bin/sh`. Each must be plain words of at most 128 characters. It also refuses a log that is empty, starts with `-`, or holds a control character.
- An example program, `crates/steplet-run/examples/settings.rs`, prints both files for a `.steplet` file (the paste below).

Step 2, the test before the code:

```
> cargo test -p steplet-run --offline --test settings
error[E0432]: unresolved imports `steplet_run::GateLink`, `steplet_run::RunPaths`, `steplet_run::mcp_json`, `steplet_run::settings_json`
error: could not compile `steplet-run` (test "settings") due to 1 previous error
```

Step 4, with `src/settings.rs` written and `src/types.rs` extended:

```
> cargo test -p steplet-run --offline --test settings
test the_run_paths_follow_the_layout ... ok
test a_gate_value_that_a_shell_could_misread_is_refused ... ok
test a_run_id_that_could_reach_another_place_is_refused ... ok
test the_mcp_config_names_the_gate_as_its_only_server ... ok
test the_settings_are_exactly_the_planned_shape ... ok
test no_allow_rules_and_no_read_rule_on_the_work_folder ... ok
test the_settings_match_the_file_the_task_2_runs_used ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`the_settings_match_the_file_the_task_2_runs_used` compares the generated file with `crates/steplet-run/tests/support/task2.settings.json`, a copy of `spikes/p3/p2.settings.json`: the file the scripted and real-model runs of Task 2 used, with its token placeholder. The two shape tests passed on the first build, so I checked that they catch a mistake. I removed the `switchModelsOnFlag` line from the writer, ran the tests, then put the line back:

```
test the_settings_match_the_file_the_task_2_runs_used ... FAILED
test the_settings_are_exactly_the_planned_shape ... FAILED
test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
restored: True
```

**A code review before the commit** (read-only, one reviewer) found nothing that must be fixed, and five points worth fixing, all applied:
- the fields were public, so a value could be changed after `new` had checked it: now private, with read-only accessors;
- the distribution id was not checked: it must now be a braced GUID;
- the 128-character limit on plain words;
- one `rule(tool, folder)` helper writes each deny rule and explains the `//`;
- a test that the file's `allow:` commands, read and write folders leave the settings unchanged (they reach only the gate's checker).

The tests came first (`the_gate_link_keeps_its_values`, `a_distribution_id_must_be_a_windows_guid`, the 128-character case, and the equality check in `no_allow_rules_and_no_read_rule_on_the_work_folder`). Before the code, they failed to build, with 12 of these errors:

```
error[E0599]: no method named `distro_id` found for struct `RunPaths` in the current scope
(the same for run_id, work_repo, run_home, run_config, run_tmp, settings, mcp, exe, pipe, token, log)
error: could not compile `steplet-run` (test "settings") due to 12 previous errors
```

After it (the first build hit the McAfee lock, `LNK1104`; the next build was clean):

```
> cargo fmt --all; cargo fmt --all -- --check
fmt exit code: 0
> cargo test -p steplet-run --offline
     Running tests\decide.rs (target\debug\deps\decide-081efc8f1b1a45b2.exe)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\settings.rs (target\debug\deps\settings-c9824e0201d52b9c.exe)
running 9 tests
test the_gate_link_keeps_its_values ... ok
test a_run_id_that_could_reach_another_place_is_refused ... ok
test a_distribution_id_must_be_a_windows_guid ... ok
test the_run_paths_follow_the_layout ... ok
test the_mcp_config_names_the_gate_as_its_only_server ... ok
test a_gate_value_that_a_shell_could_misread_is_refused ... ok
test the_settings_are_exactly_the_planned_shape ... ok
test the_settings_match_the_file_the_task_2_runs_used ... ok
test no_allow_rules_and_no_read_rule_on_the_work_folder ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test exit code: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit code: 0
```

Step 5, the generated files for a corpus file that declares two sites (`corpus/ordinary/uv-install-and-test.steplet`, `allow: domains: pypi.org, files.pythonhosted.org`), with a sample pipe, token and log:

```
> cargo run -q -p steplet-run --offline --example settings -- corpus/ordinary/uv-install-and-test.steplet r1
--- /var/lib/steplet/runs/r1/settings.json
{
  "apiKeyHelper": "/opt/steplet/gate/steplet-gate.exe --key --pipe steplet-r1 --token sampletoken",
  "permissions": {
    "deny": [
      "Read(//mnt/**)",
      "Read(//var/lib/steplet/**)",
      "Edit(//work/r1/repo/.git/**)"
    ]
  },
  "sandbox": {
    "allowUnsandboxedCommands": false,
    "autoAllowBashIfSandboxed": false,
    "enabled": true,
    "failIfUnavailable": true,
    "filesystem": {
      "allowRead": [
        "/work/r1/repo"
      ],
      "denyRead": [
        "/mnt",
        "/var/lib/steplet",
        "/work"
      ],
      "denyWrite": [
        "/work/r1/repo/.git"
      ]
    },
    "network": {
      "allowedDomains": [
        "pypi.org",
        "files.pythonhosted.org"
      ],
      "strictAllowlist": true
    }
  },
  "switchModelsOnFlag": false
}
--- /var/lib/steplet/runs/r1/mcp.json
{
  "mcpServers": {
    "gate": {
      "args": [
        "--pipe",
        "steplet-r1",
        "--token",
        "sampletoken",
        "--log",
        "C:\\Users\\sample\\AppData\\Local\\Steplet\\logs\\r1.gate.log"
      ],
      "command": "/opt/steplet/gate/steplet-gate.exe",
      "type": "stdio"
    }
  }
}
example exit code: 0
```

(The keys come out in alphabetical order; Claude Code reads JSON, so order does not matter.)

**Noted for a later task, not changed now:** the gate program now sits on the Linux disk at `/opt/steplet/gate/`, outside every folder the fence hides, so it is the one Windows program a fenced command could find. Phase 0 does not settle whether such a command can start it. Where `cmd.exe` was present, the attempt "returned non-zero exit status 1", which fits either interop being blocked inside the fence or `cmd.exe` starting and its write failing. Task 11 Step 2 therefore tries `/opt/steplet/gate/steplet-gate.exe` itself from inside the fence. If it starts, the fix to measure is adding `/opt/steplet/gate` to `denyRead`; Claude Code starts the gate from outside the fence, so the gate would still run.

**Not verified in this task:** that Claude Code 2.1.220 accepts the generated files. That rests on the two Task 2 runs, whose settings file the generator now matches exactly (the test above), and is checked again when Task 7 launches with the generated files.
## Task 6: the working copy

### Your decisions during this task (14 Oct 2026)

- **H1 moves into the launch (Task 7): "root first, then drop".** Each run will start as root through one fixed Steplet script inside Steplet's own Linux. The script unmounts the shared folders, keeps name lookup working with a plain copy of the lookup file, checks the layout, then permanently becomes `steplet` and starts the agent. The approved launch line changes from `-u steplet` to `-u root` plus that script. The measurements below are why.
- **ripgrep is installed** in Steplet's Linux (`srt` will not start without it) and added to the setup script's base tools.
- **P3 is enforced with a short list of hosts:** milestone 1 clones only from `github.com`, `gitlab.com`, `codeberg.org` and `bitbucket.org` (`REPOSITORY_HOSTS` in `src/prepare.rs`). Any other address is refused before Linux is touched. A file could otherwise name a home or company server, which Steplet's own git would reach before any fence applies.

### Changes from the plan, recorded here

- **H1's unmount is not in `prepare`** (above, and the measurements below).
- **`.git` stays owned by `steplet`.** H7 was reverted in Task 1; it is still guarded by the fence's `denyWrite` and the `Edit` deny rule. The plan's Task 6 text still says `chown -R root:root .git`.
- **`prepare` takes the `GateLink` and writes the run's settings file and MCP config** (root, 644), so one step leaves the run ready to launch: `prepare(req, paths, gate)`.
- **The fenced checkout starts in an empty private folder, not in the working copy** (measured below). It runs as `steplet`, like the clone: `setpriv`, no extra powers, `no_new_privs`.
- **Time limits:** the clone gives up after 10 minutes, or after a minute below 1 KB/s; the checkout gives up after 5 minutes.
- **New in `src/wsl.rs`:** `run` (one program inside Steplet's Linux, `wsl.exe` with `SystemRoot` only, C8) and `find_distribution`, which reads the distribution's id from the registry, so nothing hardcodes it.

### What the design rests on: measured in Steplet's Linux

**1. Name lookup depends on `/mnt/wsl`, and WSL rebuilds the shared folders.** First, a read-only look (`t6_mounts.sh`, scratchpad):

```
--- whoami: root
--- /etc/resolv.conf: lrwxrwxrwx 1 root root 20 Oct 14 13:26 /etc/resolv.conf -> /mnt/wsl/resolv.conf
    resolves to: /mnt/wsl/resolv.conf
--- mounts at or below /mnt:
    /mnt/wsl tmpfs rw,relatime
    /mnt/wslg tmpfs rw,relatime
    /mnt/wslg/distro ext4 ro,relatime,discard,errors=remount-ro,data=ordered
    /mnt/wslg/versions.txt overlay rw,relatime,lowerdir=/systemvhd,upperdir=/system/rw/upper,workdir=/system/rw/work,uuid=on
    /mnt/wslg/doc overlay rw,relatime,lowerdir=/systemvhd,upperdir=/system/rw/upper,workdir=/system/rw/work,uuid=on
    /mnt/wslg/run/user/0 tmpfs rw,nosuid,nodev,relatime,size=776412k,nr_inodes=194103,mode=700
--- other mounts whose source or target mentions wsl:
    /usr/lib/modules/6.6.87.2-microsoft-standard-WSL2 none overlay
    /usr/lib/wsl/drivers drivers 9p
    /usr/lib/wsl/lib none overlay
--- /tmp/.X11-unix: drwxrwxrwx 2 root root 60 Oct 14 13:26 /tmp/.X11-unix
--- environment WSL sets for root: WSL2_GUI_APPS_ENABLED=1 WSL_DISTRO_NAME=steplet WSL_INTEROP=/run/WSL/328_interop WAYLAND_DISPLAY=wayland-0 DISPLAY=:0 XDG_RUNTIME_DIR=/run/user/0/ WSLENV= PULSE_SERVER=unix:/mnt/wslg/PulseServer
--- DNS works now: 20.207.73.82    github.com
--- srt: lrwxrwxrwx 1 root root 44 Oct 13 12:33 /opt/steplet/srt/node_modules/.bin/srt -> ../@anthropic-ai/sandbox-runtime/dist/cli.js
--- node: /usr/bin/node v22.22.1
--- /work: p2    /var/lib/steplet/runs: p2
```

Then H1 as the plan wrote it, in one call (`t6_unmount.sh`): make the lookup file a plain copy, unmount, and leave a marker in `/run`. Then three later calls (`t6_after.sh`, read-only). The first line of output is `umount`'s own message, and the cut-off `55.255.254` is the end of the lookup file's address line, interleaved with it:

```
umount: /mnt/wslg/run/user/0: target is busy.
55.255.254
--- /tmp/.X11-unix mount: /tmp/.X11-unix none[/.X11-unix] tmpfs
/tmp/.X11-unix none[/.X11-unix] tmpfs
--- 1. /etc/resolv.conf becomes a plain file with the same lines
-rw-r--r-- 1 root root 200 Oct 14 13:27 /etc/resolv.conf
--- 2. unmount /mnt/wslg (and everything below it), /tmp/.X11-unix, /mnt/wsl
wslg exit code: 32
X11 exit code: 0
wsl exit code: 0
--- mounts left at or below /mnt: [/mnt/wslg /mnt/wslg/distro /mnt/wslg/versions.txt /mnt/wslg/doc /mnt/wslg/run/user/0 ]
--- /mnt/wsl and /mnt/wslg now hold: [/mnt/wsl:  /mnt/wslg: .X11-unix PulseAudioRDPSink PulseAudioRDPSource PulseServer distro doc pulseaudio.log run runtime-dir stderr.log versions.txt weston.log wlog.log ]
--- name lookup still works: 20.207.73.82    github.com
--- instance marker written to /run: 1791984451
--- later call, 6 s after the unmount call began:
    marker: 1791984451; uptime 80.56 s
    mounts at or below /mnt: [/mnt/wslg /mnt/wslg/distro /mnt/wslg/versions.txt /mnt/wslg/doc /mnt/wslg/run/user/0 ]
    /etc/resolv.conf: lrwxrwxrwx -> /mnt/wsl/resolv.conf
--- later call, 26 s after the unmount call began:
    marker: gone (the distribution restarted); uptime 103.55 s
    mounts at or below /mnt: [/mnt/wsl /mnt/wslg /mnt/wslg/distro /mnt/wslg/versions.txt /mnt/wslg/doc /mnt/wslg/run/user/0 ]
    /etc/resolv.conf: lrwxrwxrwx -> /mnt/wsl/resolv.conf
--- later call, 79 s after the unmount call began:
    marker: gone (the distribution restarted); uptime 156.70 s
    mounts at or below /mnt: [/mnt/wsl /mnt/wslg /mnt/wslg/distro /mnt/wslg/versions.txt /mnt/wslg/doc /mnt/wslg/run/user/0 ]
    /etc/resolv.conf: lrwxrwxrwx -> /mnt/wsl/resolv.conf
```

What it shows:
- **`/etc/resolv.conf` (where Linux looks up web addresses) is a link into `/mnt/wsl`.** Unmounting `/mnt/wsl` breaks name lookup, including Claude Code's own connection to the model, unless the lookup file is replaced first.
- **WSL re-created that link in the same running instance:** 6 s later the marker was still there, but the link was back, pointing into an unmounted folder.
- **The distribution stopped within about 20 s of going idle** (the marker was gone while the machine's uptime kept growing), and the next start mounted everything again.
- **`/mnt/wslg/run/user/0` was busy,** so a plain unmount failed (exit 32); it needs a lazy unmount (`umount -l`).

So an unmount in the prepare step would not last until the agent starts. It has to happen in the session that starts the agent, as root. That led to your decision above. It is built and measured in Task 7.

**2. `srt` needs ripgrep, and guards file names in the folder it starts in.** A test repository made inside Steplet's Linux (`t6_checkout.sh`; every file a harmless placeholder: `.bashrc`, `.claude/settings.json`, `.gitmodules`, `.mcp.json`, `.vscode/settings.json`, `README`, `src/main.py`), checked out inside `srt` twice. First, before ripgrep was there:

```
=== checkout inside srt, started in /work/t6/repo
Error: Sandbox dependencies not available: ripgrep (rg) not found
checkout exit code: 1
(the same for the second start folder)
```

`srt` uses ripgrep to look for the names it guards (`dist/sandbox/sandbox-utils.js`: `.gitconfig`, `.gitmodules`, `.bashrc`, `.bash_profile`, `.zshrc`, `.zprofile`, `.profile`, `.ripgreprc`, `.mcp.json`, and the folders `.vscode`, `.idea`, `.claude/commands`, `.claude/agents`), and only in the folder it starts in (`linux-sandbox-utils.js`, `linuxGetCwdMandatoryDenyPaths`). Installed with your yes:

```
--- before: dpkg-query: no packages found matching ripgrep
apt exit code: 0
Setting up ripgrep (15.1.0-1ubuntu1) ...
Processing triggers for man-db (2.13.1-1build1) ...
ripgrep: 15.1.0-1ubuntu1
rg: /usr/bin/rg; ripgrep 15.1.0
```

(A first attempt never ran: PowerShell rewrote the command line, the quoting trap. "before" shows nothing had been installed.) Then the same probe:

```
--- test commit 8fbcef93d186036024176e991df473e606d15bda tracks: .bashrc .claude/settings.json .gitmodules .mcp.json .vscode/settings.json README src/main.py
=== checkout inside srt, started in /work/t6/repo
error: The following untracked working tree files would be overwritten by checkout:
	.bashrc
	.gitmodules
	.mcp.json
	.vscode
Please move or remove them before you switch branches.
Aborting
checkout exit code: 1
HEAD: 8fbcef93d186036024176e991df473e606d15bda
files present:
git status: [D  .bashrc D  .claude/settings.json D  .gitmodules D  .mcp.json D  .vscode/settings.json D  README D  src/main.py ]
=== checkout inside srt, started in /var/lib/steplet/runs/t6/tmp
checkout exit code: 0
HEAD: 8fbcef93d186036024176e991df473e606d15bda
files present: ./.bashrc ./.claude/settings.json ./.gitmodules ./.mcp.json ./.vscode/settings.json ./README ./src/main.py
git status: []
--- cleaned up: /work holds [p2 ]
```

Started in the working copy, `srt` put its placeholders on the guarded names and git refused the checkout. Any commit that tracks `.gitmodules` or `.vscode/` (both common) could never be checked out that way. Started in an empty private folder, every file came out and the copy is clean. So `prepare.sh` starts the checkout from an empty folder of its own, which it deletes afterwards.

**3. The write paths `srt` adds on this layout** (plan section 9 gives this to Task 6). One debug run of the checkout fence, as `steplet`, from an empty folder (`t6_srt_paths.sh`):

```
srt exit code: 0
--- debug lines about writing, binding or temporary places:
[SandboxDebug] Linux bridges ready after 3 attempts
[SandboxDebug] Original command: /bin/sh -c 'echo "inside: uid $(id -u)"; touch /tmp/t6-probe 2>&1; ls -A /tmp 2>&1 | head -5'
[SandboxDebug] [Sandbox Linux] Processing write path: /dev/stdout -> /dev/stdout
[SandboxDebug] [Sandbox Linux] Processing write path: /dev/stderr -> /dev/stderr
[SandboxDebug] [Sandbox Linux] Processing write path: /dev/null -> /dev/null
[SandboxDebug] [Sandbox Linux] Processing write path: /dev/tty -> /dev/tty
[SandboxDebug] [Sandbox Linux] Processing write path: /dev/dtracehelper -> /dev/dtracehelper
[SandboxDebug] [Sandbox Linux] Processing write path: /dev/autofs_nowait -> /dev/autofs_nowait
[SandboxDebug] [Sandbox Linux] Processing write path: /tmp/claude -> /tmp/claude
[SandboxDebug] [Sandbox Linux] Skipping non-existent write path: /tmp/claude
[SandboxDebug] [Sandbox Linux] Processing write path: /private/tmp/claude -> /private/tmp/claude
[SandboxDebug] [Sandbox Linux] Skipping non-existent write path: /private/tmp/claude
[SandboxDebug] [Sandbox Linux] Processing write path: /work/t6d/repo -> /work/t6d/repo
touch: cannot touch '/tmp/t6-probe': Read-only file system
snap-private-tmp
--- the command's own lines:
[SandboxDebug] Original command: /bin/sh -c 'echo "inside: uid $(id -u)"; touch /tmp/t6-probe 2>&1; ls -A /tmp 2>&1 | head -5'
inside: uid 999
touch: cannot touch '/tmp/t6-probe': Read-only file system
--- after: /tmp/t6-probe on the real disk: ls: cannot access '/tmp/t6-probe': No such file or directory
```

Besides the working copy, `srt` adds the standard device files, and `/tmp/claude` and a macOS path, both skipped because they do not exist. The phase 0 `~/.npm/_logs` path does not appear: the home here is an empty folder. **Noted for Task 7:** if `/tmp/claude` ever existed, every fence would be allowed to write there, so the launch script should make sure it does not. The command ran as uid 999 (`steplet`), so the switch from root by `setpriv` worked.

### The tests

`crates/steplet-run/tests/prepare.rs`. The `#[ignore]` tests need Steplet's Linux and the network, and clone two public repositories at pinned commits: `octocat/Hello-World` at `7fd1a60b…` (phase 0 S4's), and `anthropics/claude-code-action` at `34968b59…` (14 Oct 2026), which tracks `.claude/settings.json`, `.claude/commands/` and `.claude/agents/`. Step 2, before the code:

```
> cargo test -p steplet-run --offline --test prepare
error[E0432]: unresolved imports `steplet_run::PrepareError`, `steplet_run::RunRequest`, `steplet_run::discard`, `steplet_run::prepare`, `steplet_run::wsl`
error: could not compile `steplet-run` (test "prepare") due to 1 previous error
```

Step 4, the first passing run of the three Linux tests:

```
> cargo test -p steplet-run --offline --test prepare -- --ignored
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... ok
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ok
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 42.76s
```

They passed on the first run, so I broke the script on purpose twice (each time in place, then restored) to see that they catch it. First, the checkout started in the working copy, as in the probe above:

```
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... FAILED
called `Result::unwrap()` on an `Err` value: Step { step: "check out the commit inside the fence", detail: "fatal: cannot create directory at '.claude/agents': Read-only file system\nsteplet-prepare: the checkout of 34968b5981d5d170904281aa3df6f3c0d4a28855 failed inside the fence" }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 9.82s
```

Second, the agent files not removed:

```
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... FAILED
thread 'agent_settings_files_the_commit_carries_are_checked_out_then_removed' (39568) panicked at crates\steplet-run\tests\prepare.rs:131:5:
only .claude is gone:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 19.41s
restored: True
```

### The code review, and what changed

A read-only reviewer found nothing that must be fixed. Applied:
- **The script could be eaten by a program it starts.** The script comes on `bash`'s standard input, which `bash` reads as it goes, and every program started from the script inherits that input. Both scripts now do all their work inside one function, which `bash` reads whole before running it, with input from `/dev/null`. The same two lines, sent plainly and then wrapped:

```
--- plain: a program that reads its input
before
exit code: 0
--- wrapped in one function, input from /dev/null
before
after
exit code: 0
```

  In the plain version, "after" never ran, and the exit code still said success.
- **Stopping the `steplet` account's programs** now kills again on every pass, by both effective and real user id, and also runs after the checkout, before root works on its folders.
- **The base64 encoder** is checked against RFC 4648's test vectors. This is the codebase's first test inside a source file (`prepare::tests`), because the encoder is private.
- **A request that should get through is tested as getting through** (`a_plain_request_gets_past_the_checks`).
- **Local-network addresses:** a name like `printer.local` passed the address check. That became your host-list decision above. Tested first: before the list, `https://printer.local/…` got through to `wsl.exe` (`Err(Step { step: "reach Steplet's Linux", detail: "" })`).
- **That empty detail** happened because `wsl.exe` prints its own failures on standard output. The detail now falls back to standard output.

Not applied: the reviewer's point that `rm`'s error output is lost is wrong; it is not suppressed, so it reaches the detail. The duplicate address and commit checks stay, because `steplet-core` is unchanged in this phase.

After the changes:

```
> cargo fmt --all -- --check
fmt exit code: 0
> cargo test -p steplet-run --offline
test prepare::tests::base64_gives_the_rfc_4648_test_vectors ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\decide.rs (target\debug\deps\decide-081efc8f1b1a45b2.exe)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\prepare.rs (target\debug\deps\prepare-8568093f74a6ab1e.exe)
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... ignored, needs Steplet's Linux and the network
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ignored, needs Steplet's Linux and the network
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... ignored, needs Steplet's Linux and the network
test a_request_steplet_could_misread_is_refused_before_linux_is_touched ... ok
test a_plain_request_gets_past_the_checks ... ok
test result: ok. 2 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.56s
     Running tests\settings.rs (target\debug\deps\settings-c9824e0201d52b9c.exe)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit code: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit code: 0
> cargo test -p steplet-run --offline --test prepare -- --ignored --nocapture --test-threads=1
running 3 tests
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... --- https://github.com/octocat/steplet-test-no-such-repository.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d: could not clone the repository:
fatal: could not read Username for 'https://github.com': terminal prompts disabled
steplet-prepare: could not clone https://github.com/octocat/steplet-test-no-such-repository.git
--- https://github.com/octocat/Hello-World.git at 0000000000000000000000000000000000000001: could not find the declared commit:
steplet-prepare: https://github.com/octocat/Hello-World.git has no commit 0000000000000000000000000000000000000001
ok
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ok
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... /var/lib/steplet/runs/t6a:
total 28
drwxr-xr-x 5 root    root    4096 Oct 14 19:05 .
drwxr-xr-x 3 root    root    4096 Oct 14 19:05 ..
drwx------ 2 steplet steplet 4096 Oct 14 19:05 config
drwx------ 2 steplet steplet 4096 Oct 14 19:05 home
-rw-r--r-- 1 root    root     321 Oct 14 19:05 mcp.json
-rw-r--r-- 1 root    root     735 Oct 14 19:05 settings.json
drwx------ 2 steplet steplet 4096 Oct 14 19:05 tmp
/work:
total 12
drwxr-xr-x  3 root root 4096 Oct 14 19:05 .
drwxr-xr-x 20 root root 4096 Oct 14 19:05 ..
drwxr-xr-x  3 root root 4096 Oct 14 19:05 t6a
/work/t6a:
total 12
drwxr-xr-x 3 root    root    4096 Oct 14 19:05 .
drwxr-xr-x 3 root    root    4096 Oct 14 19:05 ..
drwxr-xr-x 3 steplet steplet 4096 Oct 14 19:05 repo
/work/t6a/repo:
total 16
drwxr-xr-x 3 steplet steplet 4096 Oct 14 19:05 .
drwxr-xr-x 3 root    root    4096 Oct 14 19:05 ..
drwxr-xr-x 5 steplet steplet 4096 Oct 14 19:05 .git
-rw-r--r-- 1 steplet steplet   13 Oct 14 19:05 README
ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 29.02s
```

The folders match plan section 3 (with `.git` owned by `steplet`, per the H7 reversal). The two files are root-owned 644 and match `settings_json` and `mcp_json` byte for byte. `/work` holds only this run, and the earlier run's leftovers are gone. A wrong address or commit names the step in plain words and checks out nothing. After `discard`, `/work` and the runs folder are empty.

**Not verified in this task:**
- The unmount (H1), which is now Task 7's.
- A repository whose commit tracks `.gitmodules` with real submodules (submodules are never fetched).
- Hosts other than github.com: no GitLab, Codeberg or Bitbucket repository was cloned.
- A clone big enough to reach the time limits.
- Two runs at once (`prepare` ends the last run's programs on purpose; one run at a time is the app's job in phase 4).
## Task 7: the launcher, the stream reader, the events, and stop

### Changes from the plan, recorded here

- **The launch starts as root, as you decided (H1, Task 6).** `launch` installs `src/start-run.sh` fresh as `/opt/steplet/bin/start-run` (root, 755) and runs `wsl.exe --distribution-id <id> -u root --cd / --exec /opt/steplet/bin/start-run <run id> <seconds> <turns>`. The script:
  - closes the shared folders;
  - checks the layout (`/work` holds this run only; the settings file and MCP config are root's, 644);
  - becomes `steplet` for good with `setpriv` (no powers, `no_new_privs`);
  - runs section 3's command unchanged (`timeout -k 10`, the `env -i` list, Claude Code's flags).

  `wsl.exe` still gets `SystemRoot` only, and the task still goes on standard input, which is then closed.
- **How H1 closes the folders (measured below):**
  - `/mnt/wsl` is replaced by a small folder of Steplet's own that holds a copy of the name lookup file. WSL puts the lookup file's link back at every new session, so an empty `/mnt/wsl` broke name lookup.
  - `/mnt/wslg` and `/tmp/.X11-unix` are unmounted. The X11 one counts as closed when its path no longer reaches a mount of its own, because it can be listed while hidden under Ubuntu's own `/tmp`.
  - `/tmp/claude` is removed (the point noted in Task 6).
- **`parse_line(line) -> Vec<Event>`**, not `parse_event(line) -> Option<Event>`: one message can hold several blocks, and a dropped tool call would hide an action from Task 9's tripwires. A line that is not JSON gives `Unreadable`, a kind Steplet does not read gives `Other("type/subtype")`, and only the model's thinking gives nothing.
- **`Event` gains `Started { model, tools, gate_connected }`.** It shows the model and whether the gate connected.
- **`Result` carries `subtype` and the final `text`, and `ModelSwitch` carries the refusal's `explanation`,** which Task 10 and E1 need.
- **`GateDecision` and `SkippedGate` come with Tasks 8 and 9,** which produce them.
- **`launch(req, paths, logs, model_url)`** takes the request (task, turns, time limit), Steplet's logs folder on Windows (section 3: the stream is kept there unchanged as `<run id>.jsonl`, with `<run id>.stderr.txt`), and, for tests only, the scripted model's address. That address must be `http://127.0.0.1:<port>`.
- **The test harness** (`tests/support/mod.rs`) drives the scripted model and the phase 0 stand-in gate and host until Task 8. It installs the stand-in gate at a path named after its contents (below).

### The stream reader

What the 44 recorded streams in `C:\dev\m1\logs` hold (`t7_kinds.py`, scratchpad; counts of lines by kind):

```
    51  (not JSON)                                         e.g. S2r-empty.jsonl
     7  ?                                                  e.g. selftest.mcp-in.jsonl
    47  assistant:text                                     e.g. L1.jsonl
    16  assistant:thinking                                 e.g. L1.jsonl
   162  assistant:tool_use                                 e.g. L1.jsonl
     1  result/error_during_execution                      e.g. S1t.jsonl
    35  result/success                                     e.g. L1.jsonl
    38  system/init                                        e.g. L1.jsonl
     3  system/model_refusal_fallback                      e.g. S6R-run1.jsonl
     3  system/model_refusal_no_fallback                   e.g. S6R-off-run1.jsonl
     1  system/task_notification                           e.g. S1t.jsonl
     3  system/task_started                                e.g. S1k.jsonl
    56  system/thinking_tokens                             e.g. L1.jsonl
     1  user:text                                          e.g. S1t.jsonl
   160  user:tool_result                                   e.g. L1.jsonl
```

None holds a hook event. Milestone 0's hook lines (`C:\dev\m0\logs\T3a_repo_files.txt`) are summaries from another version, so the parser treats anything whose type or subtype names a hook as a hook event (so the run stops), and Task 9's live hook test checks the real 2.1.220 shape.

The fixture: one raw line of each kind, copied unchanged (`t7_fixture.py`, scratchpad):

```
line  1  init               from L1.jsonl
line  2  thinking           from L1.jsonl
line  3  text               from L1.jsonl
line  4  tool_use           from L1.jsonl
line  5  tool_result        from L1.jsonl
line  6  result_success     from L1.jsonl
line  7  tool_result_error  from S1b.jsonl
line  8  interrupted_text   from S1t.jsonl
line  9  task_notification  from S1t.jsonl
line 10  result_error       from S1t.jsonl
line 11  switch             from S6R-run1.jsonl
line 12  no_switch          from S6R-off-run1.jsonl
wrote 12 lines, 11115 bytes
```

L1 and S6R used the real test key, so the fixture was checked for it before going into the repository (S3's method, counts only):

```
key: 0  (files read: 1)
key, last 20 characters: 0  (files read: 1)
exit code: 0
lines containing sk-ant: 0
```

Before the parser:

```
> cargo test -p steplet-run --offline --test stream
error[E0432]: unresolved imports `steplet_run::Event`, `steplet_run::parse_line`
error: could not compile `steplet-run` (test "stream") due to 1 previous error
```

After:

```
test every_hook_event_is_seen ... ok
test a_refusal_shows_whether_the_model_switched ... ok
test the_start_names_the_model_the_tools_and_whether_the_gate_connected ... ok
test tool_results_say_whether_they_failed ... ok
test the_end_of_a_run_as_claude_code_reports_it ... ok
test the_agent_s_words_and_tool_calls ... ok
test nothing_is_dropped_and_nothing_panics ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### The launch, and four things its tests found

The live tests (`tests/launch.rs`, `#[ignore]`) prepare a run from `octocat/Hello-World` (Task 6), start the scripted model inside Steplet's Linux and the stand-in host on Windows, and launch. The first run, with the first version of the start script, passed:

```
test a_scripted_run_streams_its_events_in_order ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 11.87s
```

**1. Name lookup broke for the next session.** The second test's clone failed, right after the first run:

```
called `Result::unwrap()` on an `Err` value: Step { step: "clone the repository", detail: "fatal: unable to access 'https://github.com/octocat/Hello-World.git/': Could not resolve host: github.com\nsteplet-prepare: could not clone https://github.com/octocat/Hello-World.git" }
```

The first start script had made `/etc/resolv.conf` a plain copy and unmounted `/mnt/wsl`. As measured in Task 6, WSL then put the link back for the next session, and the link pointed into an empty folder. The same would break a run in the middle, including Claude Code's own connection to the model, whenever any new session started (the stop command, for one). **Fix:** `/mnt/wsl` is replaced by Steplet's own small folder (`tmpfs`, source `steplet-wsl`) holding a copy of the lookup file, so WSL's link keeps working. Nothing the other systems share there is visible. The first test now also clones a second time right after its run (below).

**2. A program stopped starting from one path.** The next runs' gate never connected:

```
Stderr("apiKeyHelper failed: exited 1: /opt/steplet/gate/spike-gate.exe: Invalid argument")
Started { ..., gate_connected: false }
Stderr("Error: MCP tool mcp__gate__decide (passed via --permission-prompt-tool) not found. Available MCP tools: none")
```

Every tool call failed, so nothing ran unchecked. What was found, step by step (probes in the scratchpad):
- WSL's handler for Windows programs was registered (`/proc/sys/fs/binfmt_misc/WSLInterop: enabled, interpreter /init, magic 4d5a`).
- In a freshly started Linux, before any start-script step ran, the program at that path already failed: `before anything ... gate exit code 1 (/opt/steplet/gate/spike-gate.exe: Invalid argument)`. So the start script was not the cause.
- The file is byte-for-byte the build (`c706ab5a…` on Windows, on the Linux disk, and read by Windows through `\\wsl.localhost\steplet\...`).
- Starting it failed from every folder (`/tmp`, `/`, `/root`, `/opt/steplet/gate`).
- **The same bytes at any other path start** (`/tmp/t7-new-gate.exe: exit code 101`, its expected stop on a missing option). So do the older build at a new name in the gate folder, and Windows' own `whoami.exe` (`exit code 0`).
- The build runs on Windows from its own folder (`exit code 101`).
- Smart App Control is off (`VerifiedAndReputablePolicyState` 0).
- No Code Integrity event names it, and no readable McAfee log of the last 8 hours does.
- No `spike-gate` process was left on Windows.
- Replacing a program at a fresh path with a byte-identical new file did not reproduce it (all three starts `exit code 101`).

**The cause was not found.** Something on Windows refuses to start the program from that one `\\wsl.localhost` path. The harness now installs the stand-in gate at a path named after its contents, and never replaces a gate file in place. **For Task 8:** the product gate should be installed the same way (a new name for each build, never overwritten); if Windows ever refuses a gate path, every run fails closed.

**3. A mount that is listed but cannot be reached.** The next runs stopped in the start script (exit code 30):

```
Stderr("steplet-start: could not close /tmp/.X11-unix")
```

The mount table (`t7_x11.sh`):

```
 /tmp/.X11-unix (id, parent id, mount point, source):
    80 65 / /dev/sdd
    128 80 /tmp/.X11-unix none
    131 80 /tmp tmpfs
--- device of /tmp: 68; of /tmp/.X11-unix: 68
--- mountpoint says: /tmp/.X11-unix is a mountpoint
--- what the path shows:
--- try a plain lazy unmount:
    exit code 32
--- entries left: 1
```

WSL's X11 mount (id 128) is on the root disk, and Ubuntu's own `/tmp` (id 131) was mounted over it later. So the path leads into the new `/tmp` and shows an empty folder, while `umount` says "not mounted" and fails. **Fix:** a folder counts as closed when its path no longer reaches a mount of its own (its device equals its parent folder's).

**4. Ubuntu 26.04's core tools are the Rust rewrite** (`install (uutils coreutils) 0.8.0`). Its `install` refuses to replace an existing file from `/dev/stdin` without `-D`, and reports "No such file or directory":

```
with -D, round 1: exit 0, holds: version 1, said:
with -D, round 2: exit 0, holds: version 2, said:
with -D, round 3: exit 0, holds: version 3, said:
without -D, round 4: exit 1, holds: version 3, said: install: No such file or directory
without -D, round 5: exit 1, holds: version 3, said: install: No such file or directory
```

Steplet's code always passes `-D`. Also from these runs: **a stopped run and a run at its time limit both end with exit code 124**, so the report (Task 10) must take "Steplet stopped it" from Steplet's own record.

### The code review, and what changed

A read-only reviewer found nothing that must be fixed. Applied, each with a test first:
- **A message line that also carried a `subtype` lost its tool calls** (it was read as `assistant/<subtype>`), and other kinds of call (`server_tool_use`, `mcp_tool_use`) came out as `Other`. Now a message is read by its type alone, and every block whose type ends in `tool_use` is a `ToolUse`. Before the fix, the new cases failed (`the_agent_s_words_and_tool_calls ... FAILED`, at `tests/stream.rs:54`).
- **A stop in the first second could miss the run:** before its `exec`, the run is the root start script, which no `pkill` pattern matched, so the agent could start after the stop. Now `stop`, once a second for up to 10 s, kills this run's start script and sends the end signal to Claude Code. Then everything of the `steplet` account is killed, and `wsl.exe` last. Restoring the old `stop` (one end signal, then 10 s) makes the new test fail:

```
=== break: stop as it was before the review (one end signal, then 10 s)
thread 'a_stop_right_after_launch_never_lets_the_agent_act' (7444) panicked at crates\steplet-run\tests\launch.rs:254:5:
test a_stop_right_after_launch_never_lets_the_agent_act ... FAILED
```

  Removing only the start-script kill was not caught: the once-a-second end signal alone stopped the agent before it acted. The two are layers.
- **Programs a run leaves running are killed when it ends** (`wait` and `stop`), and `wait` gives its readers 10 s, so a leftover holding an output open cannot hang it. Not proven by the test, and stated as such: with the leftover kill disabled, the test still found nothing left, because Claude Code ended its own background command when it exited (and the fence ends whatever a fenced command starts):

```
=== break: leftovers never ended
--- steplet programs when the result came: 13; after wait (5.2 s, exit code Some(0)): 0
test whatever_a_run_leaves_running_ends_with_it ... ok
```

- **The capability bounding set is emptied too** (`--bounding-set=-all`). The fence still works: a new user namespace starts with its own full set.
- **The task text is checked to arrive whole.** The scripted model now logs the text blocks of the first user message, and the first test checks that the two-line task is among them. If a step before the `exec` read the task from standard input, it would fail.
- Not changed: `pkill`'s pattern (the run id and the paths hold no special characters), and no length limit per stream line (the source is local).

### The final runs

```
> cargo test -p steplet-run --offline --test launch -- --ignored --nocapture --test-threads=1
running 5 tests
test a_scripted_run_streams_its_events_in_order ... --- every event of run t7a:
    Started { model: "claude-opus-5[1m]", tools: ["Bash", "Edit", "Glob", "Grep", "Read", "WebFetch", "Write"], gate_connected: true }
    ToolUse { id: "toolu_test_e71e43d1d98b40ca88e9", name: "Bash", input: Object {"command": String("git checkout 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d 2>&1; echo \"EXIT=$?\""), "description": String("Check out the declar...
    ToolResult { id: "toolu_test_e71e43d1d98b40ca88e9", is_error: false, text: "fatal: Unable to create '/work/t7a/repo/.git/index.lock': Read-only file system\nEXIT=128" }
    ToolUse { id: "toolu_test_d28bcad7da164fdcad99", name: "Bash", input: Object {"command": String("cat README; echo \"EXIT=$?\""), "description": String("Run the tests")} }
    ToolResult { id: "toolu_test_d28bcad7da164fdcad99", is_error: false, text: "Hello World!\nEXIT=0" }
    AgentText("Task 7: the scripted run is done.")
    Result { subtype: "success", is_error: false, stop_reason: "end_turn", models: ["claude-opus-5[1m]"], text: "Task 7: the scripted run is done." }
--- exit code: Some(0)
--- 7 lines kept in C:\Users\RANVIR\AppData\Local\Steplet\logs\t7a.jsonl
--- the first user message, as the scripted model received it: ["<system-reminder>\nAs you answer the user's questions, you can use the following context:\n# currentDate\nToday's date is 2026-10-14.\n\n      IMPORTANT: this context may or may not be relevant to your tasks. You should not respond to this context unless it is highly relevant to your task.\n</system-reminder>\n\n", "Follow the script.\nSecond line of the Steplet test task t7a, which must arrive whole."]
--- after the run, in a new session: /mnt/wsl steplet-wsl; resolv.conf; /mnt/wsl/resolv.conf
ok
test a_stop_right_after_launch_never_lets_the_agent_act ... --- stopped at once; exit code Some(9); every event:
--- acted: no; programs of the steplet account left: 0; start scripts left: 0
ok
test stop_ends_a_run_mid_command_and_leaves_nothing_running ... --- seen from outside while the command ran:
mounts under /mnt: [/mnt/wsl steplet-wsl ]
/mnt/wsl holds: [resolv.conf ]
resolv.conf: /mnt/wsl/resolv.conf
name lookup: works
/tmp/claude: absent
process timeout: user steplet CapEff: 0000000000000000 CapBnd: 0000000000000000 NoNewPrivs: 1
process claude: user steplet CapEff: 0000000000000000 CapBnd: 0000000000000000 NoNewPrivs: 1
root processes of the run: 0
--- stopped after 0.7 s; exit code Some(124); every event:
    Started { model: "claude-opus-5[1m]", tools: ["Bash", "Edit", "Glob", "Grep", "Read", "WebFetch", "Write"], gate_connected: true }
    ToolUse { id: "toolu_test_7408e5804f6340709977", name: "Bash", input: Object {"command": String("sleep 120; echo slept"), "description": String("Stay busy")} }
    Other("system/task_started")
    Other("system/task_notification")
    ToolResult { id: "toolu_test_7408e5804f6340709977", is_error: true, text: "The user doesn't want to proceed with this tool use. The tool use was rejected (eg. if it was a file edit, the new_string was NOT written to the ...
    Other("user/text")
    Result { subtype: "error_during_execution", is_error: true, stop_reason: "tool_use", models: ["claude-opus-5[1m]"], text: "" }
--- programs of the steplet account left: 0
ok
test the_time_limit_ends_a_run ... --- run t7c ended after 20.6 s (prepare included), exit code Some(124)
    (the same events as the stop above, with their own ids)
ok
test whatever_a_run_leaves_running_ends_with_it ... --- events:
    Started { model: "claude-opus-5[1m]", tools: ["Bash", "Edit", "Glob", "Grep", "Read", "WebFetch", "Write"], gate_connected: true }
    ToolUse { id: "toolu_test_766c7e1cbb884e64bcae", name: "Bash", input: Object {"command": String("sleep 300"), "description": String("Leave something running"), "run_in_background": Bool(true)} }
    Other("system/background_tasks_changed")
    Other("system/task_started")
    ToolResult { id: "toolu_test_766c7e1cbb884e64bcae", is_error: false, text: "Command running in background with ID: binc2lvlu. Output is being written to: /var/lib/steplet/runs/t7e/tmp/claude-999/-work-t7e-repo/8a183582-9...
    AgentText("Done, with something left running.")
    Result { subtype: "success", is_error: false, stop_reason: "end_turn", models: ["claude-opus-5[1m]"], text: "Done, with something left running." }
--- steplet programs when the result came: 13; after wait (5.4 s, exit code Some(0)): 0
ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 57.50s
```

What they show, against the plan's Step 4:
- **The event sequence for a scripted run** is `Started` (the gate connected, H5's seven tools only), then `ToolUse` (checkout), `ToolResult`, `ToolUse` (tests), `ToolResult`, the agent's text, and `Result`. The exit code is 0, and the stream is kept unchanged on Windows.
- **A stop in the middle of a command (F37)** ends with exactly one `Result` (`error_during_execution`) and no `steplet` program left; so does the time limit (exit code 124).
- **H1, seen from outside while the agent ran:** under `/mnt` only Steplet's own folder, holding only the lookup file; name lookup works; `/tmp/claude` is absent. Both of the run's programs are `steplet`, with no powers, an empty bounding set and `no_new_privs`; none of the run's programs is root.
- **A second clone right after a run** works (the regression from finding 1).

Then the rest of the crate, and Task 6's Linux tests again, since `wsl.rs` changed under them:

```
> cargo fmt --all -- --check
fmt exit code: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
clippy exit code: 0
> cargo test -p steplet-run --offline     (the "Running" and "test result" lines)
     Running unittests src\lib.rs (target\debug\deps\steplet_run-69c98f84c3c22b38.exe)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\decide.rs (target\debug\deps\decide-081efc8f1b1a45b2.exe)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\launch.rs (target\debug\deps\launch-444583d51df510c9.exe)
test result: ok. 1 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\prepare.rs (target\debug\deps\prepare-8568093f74a6ab1e.exe)
test result: ok. 2 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.63s
     Running tests\settings.rs (target\debug\deps\settings-c9824e0201d52b9c.exe)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\stream.rs (target\debug\deps\stream-d49a474abde3d3bf.exe)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test exit code: 0
> cargo test -p steplet-run --offline --test prepare -- --ignored --test-threads=1
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... ok
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ok
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 24.58s
```


**Not verified in this task:**
- The real gate (Task 8): these runs used the phase 0 stand-in in `allow` mode, so nothing was checked by `Policy::check` yet.
- The real model (all scripted).
- A hook event's real shape in 2.1.220 (Task 9).
- The cause of the "Invalid argument" path.
- Whether a leftover program can outlive Claude Code (the test could not make one).
- A run longer than a few minutes.
- A change of network during a run (the lookup file is a copy taken at the start).

## Task 8: the gate service and the gate program, with the deadline

Started 16 Oct 2026, on `main` at `56d0433` (pull request 2 merged; this branch fast-forwarded onto it).

What was built:
- **`steplet-gate.exe`** (new crate `crates/steplet-gate`, serde_json only). Claude Code starts it inside Steplet's Linux as its permission-prompt server. It holds no logic and no state: each request goes to Steplet on a connection of its own. Anything but a clear answer in time is a refusal: no pipe, a closed pipe, an answer of the wrong shape, or no answer by Steplet's deadline plus 2 s. Its `--key` mode is Claude Code's `apiKeyHelper` (H2): it prints the key alone, or nothing. Its log names each decision (tool, request id, answer, time) and never the input or the token.
- **The gate service** (`crates/steplet-run/src/gate.rs`). It is Steplet's end of the run's pipe, made before the launch, and it refuses to start if anyone else already holds the pipe's name. For each connection, in this order:
  1. it reads the request (at most 16 MiB, within the deadline);
  2. it checks who connected: the gate program, Steplet's own Windows account and sign-in session, at least medium integrity, not restricted, not in an app container;
  3. it checks the run's token;
  4. it gives the decision until the deadline;
  5. it records what was asked, before sending the answer, and marks the answer delivered once the gate has read all of it (at most 5 s, then the connection is cut).

  `GateService::requests()` returns that record.
- **Who connected** (`crates/steplet-run/src/client.rs`): every Windows call Steplet makes about a client, and the pipe's permission list. Only Steplet's own account may open the pipe; programs below medium integrity may neither read nor write it. (Windows' default list lets everyone read.)
- **The gate program on the Linux disk:** `install_gate` names it after its contents (SHA-256), writes a new build to a `.part` file and renames it into place, and uses a file already there only if its SHA-256 matches. A gate file is never overwritten (Task 7, finding 2).

### Changes from the plan, recorded here

- **`serve(link, program, key, decide)`**, not `serve(pipe, token, policy, deadline)`:
  - The pipe, the token and the deadline come from the `GateLink` the gate itself is started with, so the two ends cannot disagree.
  - `program` is the gate program as Windows names it (measured below).
  - `key` is served to the key mode (H2 needs the same pipe).
  - `decide` is a function, so a decision that never comes can be tested, and phase 4's ask screen can plug in. Steplet's runs pass `decide(&policy, request)`, so the one checker (row 29) still makes every decision.
- **The gate gets `--deadline-ms`**, and `GateLink` carries the deadline (a tenth of a second to an hour). The gate keeps its own timer: Steplet's deadline plus 2 s. That timer covers the case where Steplet's end of the pipe stops answering altogether; Steplet's own refusal at its deadline normally comes first. The plan listed only `--pipe`, `--token`, `--log` and `--key`.
- **`requests()` returns `Asked` records** (`Decision`, `Key`, `Refused`), each with who connected. Task 9 needs what was asked, Task 10 the refusals, and this task the gate's identity. They are in the order Steplet *answered*, not the order asked (measured in the crash run below), so Task 9 must match them by request id. A decision with `delivered: false` was not acted on.
- **No `Event::GateDecision`.** The decisions are on `requests()`; Task 9 decides whether to merge them into one list.
- **The crash case kills the gate mid-request from outside** (F39, T2d). There is no `--crash-once` option in the product gate: no test-only behaviour in shipped code.
- **Tests in two files:**
  - `crates/steplet-gate/tests/gate.rs`: the gate program against a stand-in for Steplet's end of the pipe.
  - `crates/steplet-run/tests/gate.rs`: the service, with the test program itself playing the gate, plus three `#[ignore]` live runs.

  The tests that need only Windows are ordinary tests, not `#[ignore]`.
- **The test harness uses the real gate now.** It builds `steplet-gate` from this source into `target/gate-for-tests`, and installs it as Steplet does. Task 7's launch tests run through it, with an allow-everything decision as the stand-in host's `allow` mode had. The phase 0 stand-ins are no longer used by any test.
- **mio pinned to 1.2.2.** Cargo picked mio 1.2.4 for tokio, but it was published on 3 Oct 2026, so it is 13 days old: under F12's 14 days (pasted below). 1.2.2 is in the cache and is 90 days old; tokio needs 1.2.0 or later.

### Before the code

The gate crate as an empty program, and the runner without the service:

```
> cargo test -p steplet-gate --offline
test options_it_does_not_know_are_refused ... FAILED
test the_handshake_offers_one_tool_and_ignores_what_it_cannot_read ... FAILED
test the_key_mode_prints_the_key_alone_or_nothing ... FAILED
test the_log_names_each_decision_but_never_its_input ... FAILED
test no_answer_from_steplet_is_a_refusal_at_steplet_s_deadline ... FAILED
test steplet_s_answer_reaches_claude_code_and_the_request_carries_the_run_s_token ... FAILED
test an_answer_of_the_wrong_shape_is_a_refusal ... FAILED
test a_pipe_closed_without_an_answer_and_a_missing_pipe_are_refusals ... FAILED
test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
> cargo test -p steplet-run --offline --no-run     (each error kind once, with its count)
      1 error: could not compile `steplet-run` (example "settings") due to 1 previous error
      1 error: could not compile `steplet-run` (test "gate") due to 8 previous errors
      1 error: could not compile `steplet-run` (test "launch") due to 6 previous errors
      1 error: could not compile `steplet-run` (test "prepare") due to 1 previous error
      1 error: could not compile `steplet-run` (test "settings") due to 6 previous errors
     10 error[E0061]: this function takes 4 arguments but 5 arguments were supplied
      1 error[E0425]: cannot find function `gate_file_name` in crate `steplet_run`
      1 error[E0425]: cannot find function `gate_on_windows` in crate `steplet_run`
      2 error[E0425]: cannot find function `install_gate` in crate `steplet_run`
      1 error[E0432]: unresolved import `steplet_run::GateService`
      1 error[E0432]: unresolved imports `steplet_run::ApiKey`, `steplet_run::Asked`, `steplet_run::GateError`, `steplet_run::GateService`, `steplet_run::serve`
      1 error[E0432]: unresolved imports `steplet_run::ApiKey`, `steplet_run::GateService`, `steplet_run::gate_on_windows`, `steplet_run::serve`
      1 error[E0433]: cannot find `Client` in `steplet_run`
      1 error[E0599]: no method named `deadline` found for struct `GateLink` in the current scope
```

### The gate program on its own (`crates/steplet-gate/tests/gate.rs`)

A stand-in plays Steplet's end of the pipe. These tests start the gate on Windows directly, so its log shows the test's own 127 Windows variables. Started by Claude Code inside Steplet's Linux, it sees one (below).

```
> cargo test -p steplet-gate --offline -- --nocapture --test-threads=1     (the test lines and what they printed)
test a_pipe_closed_without_an_answer_and_a_missing_pipe_are_refusals ... --- no pipe: refused in 7 ms: {"behavior":"deny","message":"Steplet refused this: Steplet is not serving this run."}
test an_answer_of_the_wrong_shape_is_a_refusal ... ok
test no_answer_from_steplet_is_a_refusal_at_steplet_s_deadline ... --- Steplet held the request: refused after 3007 ms: {"behavior":"deny","message":"Steplet refused this: Steplet did not answer in time."}
--- gate log:
1792147324.218 pid=47672 started; 127 Windows variables: ACSetupSvcPort,ACSvcPort,AI_AGENT,ALLUSERSPROFILE,ANTHROPIC_BASE_URL,API_TIMEOUT_MS,APPDATA,BAGGAGE,CARGO,CARGO_BIN_EXE_steplet-gate,CARGO_HOME
1792147327.219 pid=47672 tool=Read id=toolu_t8_1 behavior=deny ms=3000
1792147327.219 pid=47672 refused by the gate itself: Steplet did not answer in time
1792147327.220 pid=47672 tool=Read id=toolu_t8_2 behavior=allow ms=0
1792147327.220 pid=47672 standard input closed; ending
test options_it_does_not_know_are_refused ... ok
test steplet_s_answer_reaches_claude_code_and_the_request_carries_the_run_s_token ... --- allowed in 6 ms: {"behavior":"allow","updatedInput":{"command":"cargo test","description":"Run the tests"}}
test the_handshake_offers_one_tool_and_ignores_what_it_cannot_read ... ok
test the_key_mode_prints_the_key_alone_or_nothing ... --- key mode, Steplet held the request: exit code Some(1) after 3049 ms
test the_log_names_each_decision_but_never_its_input ... --- gate log:
1792147331.393 pid=31076 started; 127 Windows variables: ACSetupSvcPort,ACSvcPort,AI_AGENT,ALLUSERSPROFILE,ANTHROPIC_BASE_URL,API_TIMEOUT_MS,APPDATA,BAGGAGE,CARGO,CARGO_BIN_EXE_steplet-gate,CARGO_HOME
1792147331.394 pid=31076 tool=Write id=toolu_t8_7 behavior=allow ms=0
1792147331.394 pid=31076 standard input closed; ending
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.34s
```

(Log lines are cut at 200 characters here; each "started" line goes on to list the rest of the variable names.)

What they cover, beyond the lines above:
- the MCP handshake (one tool, `decide`; a notification or a line that is not JSON gets no reply; a call to any other tool is an error and asks Steplet nothing);
- nine answers of the wrong shape, each a refusal (F38);
- a closed pipe, then the next request answered as if nothing had happened (no state between requests);
- the key mode printing the key alone with no line ending, and nothing (exit code 1) for an error, an empty key, an unreadable answer, a closed pipe, a held request, or no pipe;
- twelve wrong command lines (exit code 2), including `--crash-once` and a deadline over an hour.

### The service, with this test program playing the gate (`crates/steplet-run/tests/gate.rs`)

```
> cargo test -p steplet-run --offline --test gate -- --nocapture --test-threads=1     (the test lines and what they printed; full records left out)
test a_client_that_never_reads_its_answer_is_let_go ... --- read after 9 s: 0 of about 4194304 bytes (Err(Os { code: 233, kind: Uncategorized, message: "No process is on the other end of the pipe." }))
test a_client_that_says_nothing_or_nonsense_is_refused_and_the_next_is_answered ... --- the silent client was let go after 1015 ms: Ok(0)
--- not JSON: {"behavior":"deny","message":"Steplet refused this: the request is not JSON."}
--- not an object: {"behavior":"deny","message":"Steplet refused this: the request carries no run token."}
--- no kind: {"behavior":"deny","message":"Steplet refused this: the request is of no kind Steplet reads."}
--- no tool: {"behavior":"deny","message":"Steplet refused this: the request names no tool."}
--- tool not text: {"behavior":"deny","message":"Steplet refused this: the request names no tool."}
--- larger than 16 MiB: No process is on the other end of the pipe. (os error 233)
--- refused: ["no request came before the deadline", "the request is not JSON", "the request carries no run token", "the request is of no kind Steplet reads", "the request names no tool", "the request names no tool", "the request is longer than 16 MiB"]
test a_decision_that_never_comes_is_refused_at_steplet_s_deadline ... --- the decision never came: refused after 1505 ms: {"behavior":"deny","message":"Steplet did not decide within 1.5 s, so it refused this"}
--- recorded: took 1504 ms, "Steplet did not decide within 1.5 s, so it refused this"
test a_decision_that_never_comes_is_refused_at_the_deadline_in_a_real_run ... ignored, needs Steplet's Linux and the network
test a_declared_command_runs_and_what_the_file_did_not_ask_for_does_not ... ignored, needs Steplet's Linux and the network
test a_gate_that_dies_mid_request_refuses_it_and_the_restarted_gate_answers_the_next ... ignored, needs Steplet's Linux and the network
test a_program_other_than_the_gate_is_refused ... ok
test a_restricted_or_low_integrity_client_is_refused ... --- restricted, but its list still names this user: Ok(Object {"behavior": String("deny"), "message": String("Steplet refused this: Steplet does not take requests from this program.")})
--- restricted to Everyone: Err(Os { code: 5, kind: PermissionDenied, message: "Access is denied." })
--- low integrity: Err(Os { code: 5, kind: PermissionDenied, message: "Access is denied." })
test a_wrong_run_token_is_refused_and_the_key_goes_only_to_the_run_s_gate ... ok
test an_answer_the_client_never_read_is_recorded_as_not_delivered ... ok
test someone_holding_the_pipe_name_first_stops_the_gate_from_starting ... --- the name was taken first: Some(Taken)
--- a low-integrity program adds an end to Steplet's pipe: Err(Os { code: 5, kind: PermissionDenied, message: "Access is denied." })
--- a program of the same account adds an end to Steplet's pipe: Ok(0x188)
test the_checker_decides_each_request_and_the_service_records_who_asked ... ok
test the_gate_program_s_windows_name_is_its_path_in_steplet_s_linux ... ok
test the_pipe_is_gone_once_the_service_stops ... --- after stop: Err(Os { code: 2, kind: NotFound, message: "The system cannot find the file specified." })
test result: ok. 11 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 17.15s
```

The new gate cases, against what the plan asked:
- **Someone holding the pipe's name first:** the service does not start (`Taken`), so the run must not launch; once the name is free it starts.
- **A program other than the gate:** the right token from the wrong program is refused, for a decision and for the key.
- **A restricted-token client:**
  - restricted to Everyone, or at low integrity: Windows itself refuses the connection (access denied), because of the pipe's permission list;
  - restricted, but with a list that still names this user: it gets past Windows, and Steplet's own check refuses it.

### Live: the real gate, started by Claude Code inside Steplet's Linux

```
> cargo test -p steplet-run --offline --test gate --test launch --test prepare -- --ignored --nocapture --test-threads=1     (the gate tests' part; record lines cut at 240 characters by the test)
     Running tests\gate.rs (target\debug\deps\gate-7950a27d338678d7.exe)
running 3 tests
test a_decision_that_never_comes_is_refused_at_the_deadline_in_a_real_run ... --- run t8c: exit code Some(0) after 5.5 s; every event:
    Started { model: "claude-opus-5[1m]", tools: ["Bash", "Edit", "Glob", "Grep", "Read", "WebFetch", "Write"], gate_connected: true }
    ToolUse { id: "toolu_test_6663f24eafb84527a676", name: "Write", input: Object {"content": String("late\n"), "file_path": String("/work/t8c/repo/late.txt")} }
    ToolResult { id: "toolu_test_6663f24eafb84527a676", is_error: true, text: "Steplet did not decide within 3 s, so it refused this" }
    AgentText("Task 8: done.")
    Result { subtype: "success", is_error: false, stop_reason: "end_turn", models: ["claude-opus-5[1m]"], text: "Task 8: done." }
--- the gate's record:
    Key { client: Client { pid: 32428, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted: fals...
    Decision { client: Client { pid: 33240, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted:...
--- late.txt exists: false; the run took 15.6 s (prepare included)
--- the decision: Deny { message: "Steplet did not decide within 3 s, so it refused this" } after 3007 ms, delivered true
ok
test a_declared_command_runs_and_what_the_file_did_not_ask_for_does_not ... --- run t8a: exit code Some(0) after 1.8 s; every event:
    Started { model: "claude-opus-5[1m]", tools: ["Bash", "Edit", "Glob", "Grep", "Read", "WebFetch", "Write"], gate_connected: true }
    ToolUse { id: "toolu_test_ddd855d102d641c58fe8", name: "Bash", input: Object {"command": String("touch declared.txt"), "description": String("Declared")} }
    ToolResult { id: "toolu_test_ddd855d102d641c58fe8", is_error: false, text: "(Bash completed with no output)" }
    ToolUse { id: "toolu_test_3edcef0d2e3740a09357", name: "Bash", input: Object {"command": String("touch undeclared.txt"), "description": String("Not declared")} }
    ToolResult { id: "toolu_test_3edcef0d2e3740a09357", is_error: true, text: "this needs the receiver's yes, and no one could answer: the file does not ask for this command" }
    ToolUse { id: "toolu_test_33001d9be9d64e17a70d", name: "Write", input: Object {"content": String("t8\n"), "file_path": String("/work/t8a/repo/written.txt")} }
    ToolResult { id: "toolu_test_33001d9be9d64e17a70d", is_error: true, text: "this needs the receiver's yes, and no one could answer: the file does not ask to change files here" }
    AgentText("Task 8: done.")
    Result { subtype: "success", is_error: false, stop_reason: "end_turn", models: ["claude-opus-5[1m]"], text: "Task 8: done." }
--- the gate's record:
    Key { client: Client { pid: 41076, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted: fals...
    Decision { client: Client { pid: 43252, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted:...
    Decision { client: Client { pid: 43252, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted:...
    Decision { client: Client { pid: 43252, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted:...
--- declared.txt, undeclared.txt, written.txt exist: [true, false, false]
--- Bash toolu_test_ddd855d102d641c58fe8: Allow { updated_input: Object {"command": String("touch declared.txt"), "description": String("Declared")} } in 0 ms
--- Bash toolu_test_3edcef0d2e3740a09357: Deny { message: "this needs the receiver's yes, and no one could answer: the file does not ask for this command" } in 0 ms
--- Write toolu_test_33001d9be9d64e17a70d: Deny { message: "this needs the receiver's yes, and no one could answer: the file does not ask to change files here" } in 0 ms
--- Steplet itself: Client { pid: 38120, program: "C:\\dev\\runFile\\.claude\\worktrees\\steplet-phase3-task8-5d752e\\target\\debug\\deps\\gate-7950a27d338678d7.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted: false, app_container: false, identify_only: false }
--- the gate: Client { pid: 41076, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted: false, app_container: false, identify_only: true }
--- the gate: Client { pid: 43252, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted: false, app_container: false, identify_only: true }
--- the gate: Client { pid: 43252, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted: false, app_container: false, identify_only: true }
--- the gate: Client { pid: 43252, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted: false, app_container: false, identify_only: true }
ok
test a_gate_that_dies_mid_request_refuses_it_and_the_restarted_gate_answers_the_next ... --- taskkill /F /IM steplet-gate-605053681878df44.exe: SUCCESS: The process "steplet-gate-605053681878df44.exe" with PID 11432 has been terminated.
--- run t8b: exit code Some(0) after 2.3 s; every event:
    Started { model: "claude-opus-5[1m]", tools: ["Bash", "Edit", "Glob", "Grep", "Read", "WebFetch", "Write"], gate_connected: true }
    ToolUse { id: "toolu_test_4926f8d5b2ec41ebaa6f", name: "Write", input: Object {"content": String("1\n"), "file_path": String("/work/t8b/repo/first.txt")} }
    ToolResult { id: "toolu_test_4926f8d5b2ec41ebaa6f", is_error: true, text: "<tool_use_error>Error calling tool (Write): MCP error -32000: Connection closed</tool_use_error>" }
    ToolUse { id: "toolu_test_30682b0865384b5a8aff", name: "Write", input: Object {"content": String("2\n"), "file_path": String("/work/t8b/repo/second.txt")} }
    ToolResult { id: "toolu_test_30682b0865384b5a8aff", is_error: false, text: "File created successfully at: /work/t8b/repo/second.txt (file state is current in your context — no need to Read it back)" }
    AgentText("Task 8: done.")
    Result { subtype: "success", is_error: false, stop_reason: "end_turn", models: ["claude-opus-5[1m]"], text: "Task 8: done." }
--- the gate's record:
    Key { client: Client { pid: 3340, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted: false...
    Decision { client: Client { pid: 46624, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted:...
    Decision { client: Client { pid: 11432, program: "\\\\wsl.localhost\\steplet\\opt\\steplet\\gate\\steplet-gate-605053681878df44.exe", user: "S-1-5-21-3179155314-955974558-399537313-1002", logon_session: 1592295, integrity: 8192, restricted:...
--- first.txt, second.txt exist: [false, true]; exit code Some(0)
--- gate process id and delivered, first write: (11432, false); second: (46624, true)
--- what Claude Code said about the first: Some("<tool_use_error>Error calling tool (Write): MCP error -32000: Connection closed</tool_use_error>")
ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 32.07s
running 3 tests
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... --- https://github.com/octocat/steplet-test-no-such-repository.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d: could not clone the repository:
fatal: could not read Username for 'https://github.com': terminal prompts disabled
steplet-prepare: could not clone https://github.com/octocat/steplet-test-no-such-repository.git
--- https://github.com/octocat/Hello-World.git at 0000000000000000000000000000000000000001: could not find the declared commit:
steplet-prepare: https://github.com/octocat/Hello-World.git has no commit 0000000000000000000000000000000000000001
ok
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ok
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... /var/lib/steplet/runs/t6a:
total 28
drwxr-xr-x 5 root    root    4096 Oct 16 10:41 .
drwxr-xr-x 3 root    root    4096 Oct 16 10:41 ..
drwx------ 2 steplet steplet 4096 Oct 16 10:41 config
drwx------ 2 steplet steplet 4096 Oct 16 10:41 home
-rw-r--r-- 1 root    root     363 Oct 16 10:41 mcp.json
-rw-r--r-- 1 root    root     755 Oct 16 10:41 settings.json
drwx------ 2 steplet steplet 4096 Oct 16 10:41 tmp
/work:
total 12
drwxr-xr-x  3 root root 4096 Oct 16 10:41 .
drwxr-xr-x 20 root root 4096 Oct 16 10:39 ..
drwxr-xr-x  3 root root 4096 Oct 16 10:41 t6a
/work/t6a:
total 12
drwxr-xr-x 3 root    root    4096 Oct 16 10:41 .
drwxr-xr-x 3 root    root    4096 Oct 16 10:41 ..
drwxr-xr-x 3 steplet steplet 4096 Oct 16 10:41 repo
/work/t6a/repo:
total 16
drwxr-xr-x 3 steplet steplet 4096 Oct 16 10:41 .
drwxr-xr-x 3 root    root    4096 Oct 16 10:41 ..
drwxr-xr-x 5 steplet steplet 4096 Oct 16 10:41 .git
-rw-r--r-- 1 steplet steplet   13 Oct 16 10:41 README
ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 24.69s
```

What they show:
- **A declared command runs, and what the file did not ask for does not** (phase 0 S1). `touch declared.txt` was allowed in 0–1 ms and the marker exists. `touch undeclared.txt` and a write were refused, with the checker's reason, and neither file exists. Each of the three tool calls reached the gate; its request id matches the event stream. The key reached Claude Code through the gate's key mode only, and the scripted model received it.
- **A gate that dies mid-request** (F33/F39): the request it died on was refused ("Connection closed"), and its answer is recorded as not delivered. Claude Code started the gate again under a new process id, and the next write was allowed and made.
- **A decision that never comes** (F36): refused at Steplet's deadline, 3007 ms with a 3 s deadline. The run went on and ended by itself with exit code 0.

### Who the gate is, as Windows sees it (the Task 1 gap)

From the first live run (`--- the gate:` lines above), against Steplet itself:

| | The gate | Steplet (the test program) |
|---|---|---|
| Program | `\\wsl.localhost\steplet\opt\steplet\gate\steplet-gate-605053681878df44.exe` | `C:\dev\runFile\...\gate-7950a27d338678d7.exe` |
| Windows account | `S-1-5-21-3179155314-955974558-399537313-1002` | the same |
| Sign-in session (logon session id) | 1592295 | the same |
| Integrity level | 8192 (medium) | 8192 (medium) |
| Restricted token, app container | no, no | no, no |
| What Steplet may do with the connection | identify it only | — |

The program name Windows reports is the gate's Linux path under `\\wsl.localhost\steplet`, which is what `gate_on_windows` computes. The gate's own log for these runs (`%LOCALAPPDATA%\Steplet\logs\t8a.gate.log`, four runs of `t8a`):

```
1792145738.230 pid=33120 started; 1 Windows variables: SystemRoot
1792145738.411 pid=33120 tool=Bash id=toolu_test_fbc23891f01f473a84da behavior=allow ms=1
1792145738.552 pid=33120 tool=Bash id=toolu_test_e526da98e725486fb479 behavior=deny ms=0
1792145738.571 pid=33120 tool=Write id=toolu_test_2bef0de198f14cdb93e7 behavior=deny ms=1
1792145876.698 pid=42128 started; 1 Windows variables: SystemRoot
1792145876.840 pid=42128 tool=Bash id=toolu_test_3f1d3cabf3af40c5ba86 behavior=allow ms=1
1792145876.974 pid=42128 tool=Bash id=toolu_test_3a049db22b64470fa1d8 behavior=deny ms=0
1792145876.990 pid=42128 tool=Write id=toolu_test_2c6511bf6a0e466fa063 behavior=deny ms=0
1792146297.911 pid=14292 started; 1 Windows variables: SystemRoot
1792146298.041 pid=14292 tool=Bash id=toolu_test_1f0bfdf0d932462da8c1 behavior=allow ms=1
1792146298.164 pid=14292 tool=Bash id=toolu_test_fe436fad7654446ab52e behavior=deny ms=0
1792146298.177 pid=14292 tool=Write id=toolu_test_a7955cd01d374ea5b25d behavior=deny ms=0
1792147215.885 pid=43252 started; 1 Windows variables: SystemRoot
1792147216.043 pid=43252 tool=Bash id=toolu_test_ddd855d102d641c58fe8 behavior=allow ms=1
1792147216.173 pid=43252 tool=Bash id=toolu_test_3edcef0d2e3740a09357 behavior=deny ms=0
1792147216.229 pid=43252 tool=Write id=toolu_test_33001d9be9d64e17a70d behavior=deny ms=0
```

So the gate saw one Windows variable, `SystemRoot` (C8). The log names each decision and holds no input: `grep -c 'touch declared\|content'` on it gave `0`.

### The gate program on the Linux disk

Three builds, three files, none replaced, each matching its name:

```
> wsl.exe --distribution-id {44f36236-99b1-42c0-84bd-7ff8392d51ba} -u root --cd / --exec /bin/sh -c 'ls -l /opt/steplet/gate/; sha256sum /opt/steplet/gate/steplet-gate-*'
total 1756
-rwxr-xr-x 1 root root 301568 Oct 14 19:29 spike-gate-63f64aab01e799ed.exe
-rwxr-xr-x 1 root root 301568 Oct 14 19:23 spike-gate.exe
-rwxr-xr-x 1 root root 395264 Oct 16 10:24 steplet-gate-2c7bf6a6745eeda0.exe
-rwxr-xr-x 1 root root 395264 Oct 16 10:40 steplet-gate-605053681878df44.exe
-rwxr-xr-x 1 root root 395264 Oct 16 10:15 steplet-gate-cd3582a82a2df1d0.exe
2c7bf6a6745eeda043abb4b03c4f7661f76dd1f30e7e6e70c3760d27e438036c  /opt/steplet/gate/steplet-gate-2c7bf6a6745eeda0.exe
605053681878df4431c6b4b2ef62649d64fe91c8e4ff7e7b3c20ba8864de8792  /opt/steplet/gate/steplet-gate-605053681878df44.exe
cd3582a82a2df1d0a30cef3eb0b793dc4c34907fa7a220d6bde928a781615153  /opt/steplet/gate/steplet-gate-cd3582a82a2df1d0.exe
```

### Task 7's launch tests and Task 6's working-copy tests again, through the real gate

```
     Running tests\launch.rs (target\debug\deps\launch-f6b8f524c9b73717.exe)
test a_scripted_run_streams_its_events_in_order ... --- every event of run t7a:
test a_stop_right_after_launch_never_lets_the_agent_act ... --- stopped at once; exit code Some(9); every event:
test stop_ends_a_run_mid_command_and_leaves_nothing_running ... --- seen from outside while the command ran:
test the_time_limit_ends_a_run ... --- run t7c ended after 19.3 s (prepare included), exit code Some(124)
test whatever_a_run_leaves_running_ends_with_it ... --- events:
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 55.18s
     Running tests\prepare.rs (target\debug\deps\prepare-91bd744977e3ec40.exe)
test a_wrong_address_or_commit_names_the_step_and_runs_nothing ... --- https://github.com/octocat/steplet-test-no-such-repository.git at 7fd1a60b01f91b314f59955a4e4d4e80d8edf11d: could not clone the repository:
test agent_settings_files_the_commit_carries_are_checked_out_then_removed ... ok
test prepare_makes_one_fresh_working_copy_at_the_declared_commit ... /var/lib/steplet/runs/t6a:
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 24.69s
```

### The code review, and what changed

A separate reviewer read the change and found nothing critical. Three points were rated important:

1. **The run's token showed in debugging output** (`Debug` of `GateRequest`, `GateLink`, and so the record). **Fixed:** both now print `(hidden)`. Before the fix, the new check failed: `the run's token is never shown, even in the record`, `test the_checker_decides_each_request_and_the_service_records_who_asked ... FAILED`.
2. **"Anything that can run the gate with the token gets the key."** True, and **not changed**, because nothing inside the run can do it. The agent's commands run inside the fence, which cannot start Windows programs (F28, measured in phase 0). The token is not visible from inside the fence (phase 0 S1a). The agent's own tools read and write files; they do not run programs. Serving the key only once would also break longer runs, because Claude Code asks for the key again over time. Recorded beside the same-account limit below.
3. **A client that never reads its answer kept its end of the pipe for ever.** An allowed write's answer carries the whole file, more than the pipe holds. **Fixed**, in two steps:
   - The first fix (a time limit on the write) did not work. Tokio's write finishes once it has taken the data, before the client has it, so the test still read the whole answer after 9 s: `--- read after 9 s: 4194391 of about 4194304 bytes (Ok(4194391))`.
   - Now Steplet waits until the client has read the whole answer (Windows' `FlushFileBuffers`), at most 5 s, then cuts the connection: `read after 9 s: 0 of about 4194304 bytes` above.

   `delivered` now means the gate read the answer. It is set a moment after the gate has it, so the tests wait for it. One point is kept as it was: if Steplet cannot make a new end of its pipe, it stops serving, which refuses everything. Retrying could end up joining a pipe someone else made, once Steplet's own ends were gone.

Of the minor points:
- **Fixed:** the gate refuses a deadline over an hour; the largest number used to overflow its clock. Before the fix: `left: Some(0) right: Some(2)` for `--deadline-ms 3600001`.
- **Fixed:** two comments now say exactly what `delivered` and `identify_only` mean.
- **Left as they are, and noted for the final review:**
  - the integrity number is read without checking that the label has one part (a real label always has one);
  - the record has no size cap;
  - two installs of the same build at the very same moment would share one `.part` file (Steplet runs one run at a time);
  - a refused permission at pipe creation is reported as "taken".

### Checks

```
> cargo fmt --all -- --check
fmt exit code: 0
> cargo clippy --workspace --all-targets --offline -- -D warnings
    Checking steplet-run v0.1.0 (C:\dev\runFile\.claude\worktrees\steplet-phase3-task8-5d752e\crates\steplet-run)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.40s
clippy exit code: 0
> cargo test --workspace --offline     (every "test result" line, summed by awk)
179 passed, 0 failed, 11 ignored, in 41 test programs, every one ok
test exit code: 0
```

The six packages cargo added to the lock file (all from cargo's cache, no download), checked with `lock_ages.py` (it counts days to 11 Oct 2026, so each is 5 days older today):

```
> python -I lock_ages.py t8-new-entries.lock     (a lock file of just the six new entries)
bytes              1.12.1                 published 2026-07-08    95 days old
mio                1.2.4                  published 2026-10-03     8 days old  <-- under 14 days
pin-project-lite   0.2.17                 published 2026-02-27   226 days old
socket2            0.6.5                  published 2026-07-13    90 days old
tokio              1.53.1                 published 2026-07-20    83 days old
wasi               0.11.1+wasi-snapshot-preview1 published 2025-06-10   488 days old
6 crates.io packages in the lock file; 1 under 14 days old; 0 yanked
> python -I lock_ages.py t8-mio.lock
mio                1.2.2                  published 2026-07-13    90 days old
1 crates.io packages in the lock file; 0 under 14 days old; 0 yanked
> cargo update -p mio --precise 1.2.2 --offline
 Downgrading mio v1.2.4 -> v1.2.2
```

### What this task did not verify

- **A program of the same Windows account can add its own end of Steplet's pipe** (measured above: `Ok(0x188)`), and could then receive a gate request and answer it. Windows does not keep one account's programs apart. Such a program could equally read Steplet's memory, so the pipe is not where this is stopped. Nothing inside a run can start a Windows program (F28). Phase 5 could have the gate check the server's process id.
- **The key goes to whatever runs the gate with the run's token** (review point 2). Nothing in the run can (F28, S1a), but this rests on the fence, not on the gate.
- **The real model:** every run here used the scripted one.
- **Steplet running as administrator (high integrity)**, and a standard (non-administrator) Windows account: not tried.
- **Two requests at the same moment** from one run: Claude Code asked one at a time here.
- **A write near the 16 MiB limit through Claude Code:** tested at the pipe only.
- **The gate's own timer through WSL:** tested with the gate started on Windows directly; through WSL only Steplet's own deadline was tested.
- **The key in Steplet's memory** is not wiped after use.
- **The cause of Task 7's "Invalid argument" path:** still unknown. A new name for each build avoids it.
