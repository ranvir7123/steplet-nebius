# Milestone 1, phase 0: one spike run, start to finish. Throwaway.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File C:\dev\runFile\spikes\tools\run.ps1 -Id S7a
# Everything it shows is also written to C:\dev\m1\logs\<Id>.txt.
param([Parameter(Mandatory = $true)][string]$Id)
$ErrorActionPreference = "Continue"
$env:WSL_UTF8 = "1"
$Distro = "steplet-test"
$Spikes = "C:\dev\runFile\spikes"
$RunDir = "C:\dev\m1\runs\$Id"
$Out = "C:\dev\m1\logs\$Id.txt"
$Run = Get-Content "$Spikes\runs\$Id.json" -Raw | ConvertFrom-Json
$DistroId = (Get-Content "C:\dev\m1\distro-id.txt" -Raw).Trim()
Remove-Item $Out -ErrorAction SilentlyContinue

function Say([string]$Text) { Add-Content -Path $Out -Value $Text -Encoding utf8; Write-Output $Text }
function InWsl([string[]]$Argv) { & wsl.exe -d $Distro --exec @Argv 2>&1 | ForEach-Object { Say "$_" } }

Say "=== spike run $Id, started $(Get-Date -Format o)"
Say "--- run description: $((Get-Content "$Spikes\runs\$Id.json" -Raw).Trim())"
InWsl @("/usr/bin/python3", "-I", "/mnt/c/dev/runFile/spikes/wsl/spike.py", "prepare", $Id)
$svc = Start-Process -FilePath "wsl.exe" -ArgumentList "-d $Distro --exec /bin/bash /mnt/c/dev/runFile/spikes/wsl/services.sh $Id" -WindowStyle Hidden -PassThru
for ($i = 0; $i -lt 40 -and -not (Test-Path "$RunDir\services.ready"); $i++) { Start-Sleep -Milliseconds 500 }
Say "--- helpers (scripted model, listener): $(Get-Content "$RunDir\services.ready" -ErrorAction SilentlyContinue)"
$started = Get-Date
if ($Run.launch -eq "inside") {
    InWsl @("/bin/bash", "/mnt/c/dev/runFile/spikes/wsl/launch_inside.sh", $Id)
} else {
    $a = @("launch", "--id", $Id, "--distro-id", $DistroId, "--cd", (Get-Content "$RunDir\cd.txt" -Raw).Trim(), "--run-dir", $RunDir)
    if ($Run.gate -eq "exe") {
        $a += @("--pipe", "steplet-spike-$Id", "--token", (Get-Content "$RunDir\token.txt" -Raw).Trim(), "--gate-mode", $Run.gate_mode)
    }
    if ($Run.host) {
        # Brackets matter: the comma binds tighter than +, which glued "--name value" into one argument.
        foreach ($p in $Run.host.PSObject.Properties) { $a += @(("--" + $p.Name.Replace("_", "-")), "$($p.Value)") }
    }
    & "$Spikes\target\release\spike-host.exe" @a 2>&1 | ForEach-Object { Say "$_" }
}
Say "--- launch finished after $([math]::Round(((Get-Date) - $started).TotalSeconds, 1)) s"
Start-Sleep -Seconds 2
Say "--- Windows: spike-gate.exe processes still running: $(@(Get-Process spike-gate -ErrorAction SilentlyContinue).Count)"
InWsl @("/usr/bin/python3", "-I", "/mnt/c/dev/runFile/spikes/wsl/spike.py", "evidence", $Id)
if ($Run.cleanup) { InWsl @("/usr/bin/python3", "-I", "/mnt/c/dev/runFile/spikes/wsl/spike.py", "cleanup", $Id) }
New-Item -ItemType File -Path "$RunDir\stop" -Force | Out-Null
if (-not $svc.WaitForExit(15000)) { Say "--- the helpers did not stop within 15 s" }
Say "=== spike run $Id finished $(Get-Date -Format o)"
