# Phase 3, Task 1, Checks 1 and 5: Claude Code as `steplet` in Steplet's Linux starts the gate through interop,
# and every write reaches it. Throwaway driver, like phase 0's run.ps1.
$ErrorActionPreference = "Continue"
$env:WSL_UTF8 = "1"
$Here = Split-Path -Parent $MyInvocation.MyCommand.Path
$Wt = "C:\dev\runFile\.claude\worktrees\e3-layout-phase3-plan-8f4432"
$P3 = "C:\dev\m1\p3"; $RunDir = "$P3\runs\p1"; $Id = "P3-1"
$GateLog = "$env:LOCALAPPDATA\Steplet\logs\p1.gate.log"
New-Item -ItemType Directory -Force $RunDir | Out-Null
Remove-Item "C:\dev\m1\logs\$Id.*", $GateLog -ErrorAction SilentlyContinue
$Distro = (Get-ChildItem HKCU:\Software\Microsoft\Windows\CurrentVersion\Lxss | Where-Object { (Get-ItemProperty $_.PSPath).DistributionName -eq "steplet" }).PSChildName
$Token = [guid]::NewGuid().ToString("N")
$NoBom = New-Object System.Text.UTF8Encoding($false)
function Put([string]$Src, [string]$Dst) { cmd /c "wsl.exe -d steplet -u root --exec /usr/bin/dd of=$Dst status=none < `"$Src`"" }

"=== $Id started $(Get-Date -Format o); distribution steplet $Distro"
Remove-Item -Recurse -Force "$P3\fixture" -ErrorAction SilentlyContinue
git clone -q C:\dev\m0\repo "$P3\fixture"
tar -cf "$P3\fixture.tar" -C "$P3\fixture" .
$mcp = @{ mcpServers = @{ gate = @{ type = "stdio"; command = "/opt/steplet/gate/spike-gate.exe"; args = @("--pipe", "steplet-p3-p1", "--token", $Token, "--log", $GateLog) } } }
[IO.File]::WriteAllText("$P3\p1.mcp.json", ($mcp | ConvertTo-Json -Depth 5), $NoBom)
[IO.File]::WriteAllText("$RunDir\task.txt", "Follow the spike script.", $NoBom)
Copy-Item "$Here\argv.txt" "$RunDir\argv.txt"

wsl.exe -d steplet -u root --exec /usr/bin/install -d -m 755 /opt/steplet-test
Put "$P3\fixture.tar" /opt/steplet-test/fixture.tar
Put "$Here\p1.settings.json" /opt/steplet-test/p1.settings.json
Put "$P3\p1.mcp.json" /opt/steplet-test/p1.mcp.json
Put "$Here\p1.script.json" /opt/steplet-test/p1.script.json
Put "$Wt\spikes\tools\fake_model.py" /opt/steplet-test/fake_model.py
Put "$Here\prepare_p1.sh" /opt/steplet-test/prepare_p1.sh
wsl.exe -d steplet -u root --exec /bin/bash /opt/steplet-test/prepare_p1.sh

"--- scripted model"
wsl.exe -d steplet -u root --exec /usr/bin/rm -f /opt/steplet-test/p1.model.log
$model = Start-Process -FilePath wsl.exe -ArgumentList "-d steplet -u root --exec /usr/bin/python3 -I /opt/steplet-test/fake_model.py 47900 /opt/steplet-test/p1.script.json /opt/steplet-test/p1.model.log" -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 3
wsl.exe -d steplet -u root --exec /usr/bin/pgrep -af fake_model.py

"--- launch through the host stand-in (wsl.exe gets SystemRoot only)"
& "$Wt\spikes\target\release\spike-host.exe" launch --id $Id --distro-id $Distro --cd /work/p1/repo --run-dir $RunDir --pipe steplet-p3-p1 --token $Token --gate-mode allow --env-mode systemroot --timeout-s 300

Start-Sleep -Seconds 2
"--- Windows: spike-gate.exe processes still running: $(@(Get-Process spike-gate -ErrorAction SilentlyContinue).Count)"
"--- event stream, read with a JSON parser:"
python -I "$Wt\spikes\tools\parse_claude_stream.py" "C:\dev\m1\logs\$Id.jsonl"
"--- error output:"
Get-Content "C:\dev\m1\logs\$Id.stderr.txt" -ErrorAction SilentlyContinue | Select-Object -First 10
"--- gate log (the stand-in gate, a Windows program started inside Linux):"
Get-Content $GateLog -ErrorAction SilentlyContinue | ForEach-Object { $_.Substring(0, [Math]::Min(220, $_.Length)) }
"--- host gate log:"
Get-Content "C:\dev\m1\logs\$Id.host-gate.log" -ErrorAction SilentlyContinue | ForEach-Object { $_.Substring(0, [Math]::Min(220, $_.Length)) }
"--- every tool call, and whether it asked the gate:"
python -I -c "import json,sys; g=open(sys.argv[2],encoding='utf-8',errors='replace').read(); [print('    %-6s asked=%s  %s' % (c['name'], 'yes' if json.dumps(str(c['input'].get('file_path') or c['input'].get('command')))[1:-1] in g else 'NO ', str(c['input'].get('file_path') or c['input'].get('command'))[:110])) for l in open(sys.argv[1],encoding='utf-8') if l.strip() for e in [json.loads(l)] if e.get('type')=='assistant' for c in e['message']['content'] if c.get('type')=='tool_use']" "C:\dev\m1\logs\$Id.jsonl" $GateLog
"--- markers in the working copy, and who owns .git:"
wsl.exe -d steplet -u root --exec /usr/bin/stat -c "%U %n" /work/p1/repo/.git /work/p1/repo/p3_marker_01.txt /work/p1/repo/p3_marker_05.txt
"--- scripted model: requests (kind, step, answer):"
wsl.exe -d steplet -u root --exec /usr/bin/python3 -I -c "import json; [print('   ', e.get('kind'), e.get('step'), e.get('answer'), 'tools=%d' % len(e.get('tools_offered') or [])) for e in map(json.loads, open('/opt/steplet-test/p1.model.log'))]"
wsl.exe -d steplet -u root --exec /usr/bin/pkill -f fake_model.py
if (-not $model.HasExited) { Stop-Process -Id $model.Id -ErrorAction SilentlyContinue }
"=== $Id finished $(Get-Date -Format o)"
