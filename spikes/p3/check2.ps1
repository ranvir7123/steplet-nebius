# Phase 3, Task 2: the key reaches Claude Code only through the gate's --key mode (apiKeyHelper), so it is never
# written inside Steplet's Linux; then the milestone 0 fixture runs, scripted or with the real model.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File spikes\p3\check2.ps1 -Mode scripted|live
param([Parameter(Mandatory = $true)][ValidateSet("scripted", "live")][string]$Mode)
$ErrorActionPreference = "Continue"
$env:WSL_UTF8 = "1"
$Here = Split-Path -Parent $MyInvocation.MyCommand.Path
$Wt = Split-Path -Parent (Split-Path -Parent $Here)
$Support = "$Wt\crates\steplet-run\tests\support"
$P3 = "C:\dev\m1\p3"; $Id = if ($Mode -eq "live") { "P3-2L" } else { "P3-2s" }; $RunDir = "$P3\runs\$Id"
$GateLog = "$env:LOCALAPPDATA\Steplet\logs\$Id.gate.log"
$TestKeyWsl = "steplet-test:/home/tester/.local/share/steplet/secrets/anthropic-api-key"
$FakeKey = "$P3\fake-api-key.txt"
New-Item -ItemType Directory -Force $RunDir, "$P3\outside" | Out-Null
Remove-Item "C:\dev\m1\logs\$Id.*", $GateLog, "$P3\outside\*" -ErrorAction SilentlyContinue
$Distro = (Get-ChildItem HKCU:\Software\Microsoft\Windows\CurrentVersion\Lxss | Where-Object { (Get-ItemProperty $_.PSPath).DistributionName -eq "steplet" }).PSChildName
$Token = [guid]::NewGuid().ToString("N")
$NoBom = New-Object System.Text.UTF8Encoding($false)
function Put([string]$Src, [string]$Dst) { cmd /c "wsl.exe -d steplet -u root --exec /usr/bin/dd of=$Dst status=none < `"$Src`"" }

"=== $Id started $(Get-Date -Format o); distribution steplet $Distro"
if (-not (Test-Path $FakeKey)) { [IO.File]::WriteAllText($FakeKey, "sk-ant-api03-P3FAKE-" + [guid]::NewGuid().ToString("N") + [guid]::NewGuid().ToString("N"), $NoBom) }
Remove-Item -Recurse -Force "$P3\fixture" -ErrorAction SilentlyContinue
git -c core.autocrlf=false clone -q C:\dev\m0\repo "$P3\fixture"   # Unix line endings, as a clone inside Linux has
tar -cf "$P3\fixture.tar" -C "$P3\fixture" . 2>$null
[IO.File]::WriteAllText("$P3\p2.settings.json", ((Get-Content "$Here\p2.settings.json" -Raw) -replace "RUN_TOKEN", $Token), $NoBom)
$mcp = @{ mcpServers = @{ gate = @{ type = "stdio"; command = "/opt/steplet/gate/spike-gate.exe"; args = @("--pipe", "steplet-p3-p2", "--token", $Token, "--log", $GateLog) } } }
[IO.File]::WriteAllText("$P3\p2.mcp.json", ($mcp | ConvertTo-Json -Depth 5), $NoBom)
$argv = (Get-Content "$Here\argv.txt") -replace "/p1/", "/p2/" | Where-Object { $Mode -eq "scripted" -or $_ -notlike "ANTHROPIC_BASE_URL=*" }
[IO.File]::WriteAllText("$RunDir\argv.txt", (($argv -join "`n") + "`n"), $NoBom)
$task = if ($Mode -eq "live") { (Get-Content "$Wt\spikes\runs\L1.json" -Raw | ConvertFrom-Json).task } else { "Follow the script." }
[IO.File]::WriteAllText("$RunDir\task.txt", $task, $NoBom)
$sha = (Get-FileHash $FakeKey -Algorithm SHA256).Hash.ToLower()
[IO.File]::WriteAllText("$P3\p2.keysha", $sha, $NoBom)

wsl.exe -d steplet -u root --exec /usr/bin/install -d -m 755 /opt/steplet-test
Put "$P3\fixture.tar" /opt/steplet-test/fixture.tar
Put "$P3\p2.settings.json" /opt/steplet-test/p2.settings.json
Put "$P3\p2.mcp.json" /opt/steplet-test/p2.mcp.json
Put "$Here\p2.script.json" /opt/steplet-test/p2.script.json
Put "$P3\p2.keysha" /opt/steplet-test/p2.keysha
Put "$Support\fake_model.py" /opt/steplet-test/fake_model.py
Put "$Support\listener.py" /opt/steplet-test/listener.py
Put "$Support\count_key.py" /opt/steplet-test/count_key.py
Put "$Here\prepare.sh" /opt/steplet-test/prepare.sh
Put "$Wt\spikes\target\release\spike-gate.exe" /opt/steplet/gate/spike-gate.exe
wsl.exe -d steplet -u root --exec /usr/bin/chmod 755 /opt/steplet/gate/spike-gate.exe
wsl.exe -d steplet -u root --exec /bin/bash /opt/steplet-test/prepare.sh p2

"--- helpers inside Steplet's Linux: listener on 127.0.0.1:47831$(if ($Mode -eq 'scripted') { ', scripted model on 47900' })"
wsl.exe -d steplet -u root --exec /usr/bin/rm -f /opt/steplet-test/p2.model.log /opt/steplet-test/p2.listener.log
$helpers = @()
$helpers += Start-Process -FilePath wsl.exe -ArgumentList "-d steplet -u root --exec /usr/bin/python3 -I /opt/steplet-test/listener.py 47831 /opt/steplet-test/p2.listener.log" -WindowStyle Hidden -PassThru
if ($Mode -eq "scripted") {
    $helpers += Start-Process -FilePath wsl.exe -ArgumentList "-d steplet -u root --exec /usr/bin/python3 -I /opt/steplet-test/fake_model.py 47900 /opt/steplet-test/p2.script.json /opt/steplet-test/p2.model.log /opt/steplet-test/p2.keysha" -WindowStyle Hidden -PassThru
}
Start-Sleep -Seconds 3
wsl.exe -d steplet -u root --exec /usr/bin/pgrep -af "listener.py|fake_model.py"

"--- launch through the host stand-in (wsl.exe gets SystemRoot only; the key is served over the pipe)"
$keyArgs = if ($Mode -eq "live") { @("--key-wsl", $TestKeyWsl) } else { @("--key-file", $FakeKey) }
& "$Wt\spikes\target\release\spike-host.exe" launch --id $Id --distro-id $Distro --cd /work/p2/repo --run-dir $RunDir --pipe steplet-p3-p2 --token $Token --gate-mode allow --env-mode systemroot --timeout-s 900 @keyArgs

Start-Sleep -Seconds 2
"--- Windows: spike-gate.exe processes still running: $(@(Get-Process spike-gate -ErrorAction SilentlyContinue).Count)"
"--- event stream, read with a JSON parser:"
python -I "$Wt\spikes\tools\parse_claude_stream.py" "C:\dev\m1\logs\$Id.jsonl"
"--- models used, from the result event:"
python -I -c "import json,sys; [print('   ', m, round(u.get('costUSD', 0), 4), 'USD') for l in open(sys.argv[1],encoding='utf-8') if l.strip() for e in [json.loads(l)] if e.get('type')=='result' for m,u in (e.get('modelUsage') or {}).items()]" "C:\dev\m1\logs\$Id.jsonl"
"--- error output:"
Get-Content "C:\dev\m1\logs\$Id.stderr.txt" -ErrorAction SilentlyContinue | Select-Object -First 10
"--- gate log:"
Get-Content $GateLog -ErrorAction SilentlyContinue | ForEach-Object { $_.Substring(0, [Math]::Min(200, $_.Length)) }
"--- host gate log (key requests and decisions; the key itself is never logged):"
Get-Content "C:\dev\m1\logs\$Id.host-gate.log" -ErrorAction SilentlyContinue | Where-Object { $_ -notmatch '^"|tasklist' }
"--- every tool call, and whether it asked the gate:"
python -I -c "import json,sys; g=open(sys.argv[2],encoding='utf-8',errors='replace').read(); [print('    %-6s asked=%s  %s' % (c['name'], 'yes' if json.dumps(str(c['input'].get('file_path') or c['input'].get('command')))[1:-1] in g else 'NO ', str(c['input'].get('file_path') or c['input'].get('command'))[:110])) for l in open(sys.argv[1],encoding='utf-8') if l.strip() for e in [json.loads(l)] if e.get('type')=='assistant' for c in e['message']['content'] if c.get('type')=='tool_use']" "C:\dev\m1\logs\$Id.jsonl" $GateLog
"--- m0_results.jsonl (the fixture's own record):"
wsl.exe -d steplet -u root --exec /usr/bin/cat /work/p2/repo/m0_results.jsonl
"--- listener inside Steplet's Linux (127.0.0.1:47831): $(wsl.exe -d steplet -u root --exec /usr/bin/sh -c 'cat /opt/steplet-test/p2.listener.log 2>/dev/null || echo none')"
"--- outside write: $(wsl.exe -d steplet -u root --exec /usr/bin/sh -c 'ls /var/lib/steplet/runs/p2/tmp/written_by_probe.txt 2>/dev/null || echo absent')"
"--- Windows marker: $(if (Test-Path "$P3\outside\from_wsl.txt") { 'EXISTS' } else { 'absent' })"
"--- working copy HEAD and status:"
wsl.exe -d steplet -u steplet --cd /work/p2/repo --exec /usr/bin/git -c core.fsmonitor=false status --short --branch
if ($Mode -eq "scripted") {
    "--- scripted model: requests, and whether the key it got matches the expected key's SHA-256:"
    wsl.exe -d steplet -u root --exec /usr/bin/python3 -I -c "import json; [print('   ', e.get('kind'), e.get('step'), e.get('answer'), 'key_sent=%s matches=%s' % (e.get('api_key_sent'), e.get('api_key_matches_expected'))) for e in map(json.loads, open('/opt/steplet-test/p2.model.log'))]"
}
"--- where the key's text appears (counts only):"
$keyCmd = if ($Mode -eq "live") { "wsl.exe -d steplet-test -u tester --exec /bin/cat /home/tester/.local/share/steplet/secrets/anthropic-api-key" } else { "type `"$FakeKey`"" }
"    the run's folders inside Steplet's Linux:"
cmd /c "$keyCmd | wsl.exe -d steplet -u root --exec /usr/bin/python3 -I /opt/steplet-test/count_key.py /var/lib/steplet/runs/p2 /work/p2"
"    every file on Steplet's Linux disk:"
cmd /c "$keyCmd | wsl.exe -d steplet -u root --exec /usr/bin/python3 -I /opt/steplet-test/count_key.py --whole-disk"
"    the Windows-side logs of this run:"
cmd /c "$keyCmd | python -I `"$Support\count_key.py`" C:\dev\m1\logs\$Id.jsonl C:\dev\m1\logs\$Id.stderr.txt C:\dev\m1\logs\$Id.host-gate.log `"$GateLog`""
wsl.exe -d steplet -u root --exec /usr/bin/pkill -f "listener.py|fake_model.py"
foreach ($h in $helpers) { if (-not $h.HasExited) { Stop-Process -Id $h.Id -ErrorAction SilentlyContinue } }
"=== $Id finished $(Get-Date -Format o)"
