# Builds Steplet's Windows release (the Nebius plan, Task 12) into target\release\dist: the installer, a zip of the
# two programs and the licence, and SHA-256 sums of both. Nothing is downloaded except Tauri's installer tools
# (NSIS), the first time. Run it from anywhere: powershell -ExecutionPolicy Bypass -File tools\release.ps1
#
# The release settings live in tauri.release.conf.json, so an ordinary build or test run does not need the gate
# program built first.
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$env:CARGO_NET_OFFLINE = "true"

function Run($what, [scriptblock]$step) {
    & $step
    if ($LASTEXITCODE) { throw "$what failed (exit code $LASTEXITCODE)" }
}

# The screens Steplet's window loads, then the gate program the installer puts beside Steplet's own. The gate is
# built in a target folder of its own: Tauri copies it next to steplet.exe, which must not be the file it reads.
Run "the web build" { npm --prefix "$root\web" run build }
Run "the gate build" { cargo build --release -p steplet-gate --manifest-path "$root\Cargo.toml" --target-dir "$root\target\gate" }

# Steplet itself, and its installer.
Push-Location "$root\crates\steplet-app"
try { Run "the Tauri build" { & "$root\web\node_modules\.bin\tauri.cmd" build --config tauri.release.conf.json } }
finally { Pop-Location }

$version = (Get-Content "$root\crates\steplet-app\tauri.conf.json" -Raw | ConvertFrom-Json).version
$out = "$root\target\release\dist"
New-Item -ItemType Directory -Force $out | Out-Null
$setup = Get-ChildItem "$root\target\release\bundle\nsis\*_${version}_x64-setup.exe"
if (@($setup).Count -ne 1) { throw "expected one installer for $version, found $(@($setup).Count)" }
Copy-Item -Force $setup.FullName "$out\Steplet_${version}_x64-setup.exe"
$zip = "$out\Steplet_${version}_x64.zip"
Compress-Archive -Force -DestinationPath $zip -Path "$root\target\release\steplet.exe", "$root\target\gate\release\steplet-gate.exe", "$root\LICENSE"
Get-FileHash -Algorithm SHA256 "$out\Steplet_${version}_x64-setup.exe", $zip |
    ForEach-Object { "{0}  {1}" -f $_.Hash.ToLower(), (Split-Path -Leaf $_.Path) } |
    Set-Content -Encoding ascii "$out\SHA256SUMS.txt"
Get-ChildItem $out | Select-Object Name, Length
Get-Content "$out\SHA256SUMS.txt"
