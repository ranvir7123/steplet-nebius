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
