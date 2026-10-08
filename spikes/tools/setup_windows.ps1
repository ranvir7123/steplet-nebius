# Milestone 1, phase 0, task 2: Rust tools on Windows. Logs everything to C:\dev\m1\logs\setup-windows.txt.
$ErrorActionPreference = "Continue"
$Out = "C:\dev\m1\logs\setup-windows.txt"
New-Item -ItemType Directory -Force C:\dev\m1\logs | Out-Null
Remove-Item $Out -ErrorAction SilentlyContinue
function Say([string]$Text) { Add-Content -Path $Out -Value $Text -Encoding utf8; Write-Output $Text }
function Run([string]$Title, [scriptblock]$Block) {
    Say "--- $Title"
    & $Block 2>&1 | ForEach-Object { Say "$_" }
    Say "exit code: $LASTEXITCODE"
}
Run "rustup target add wasm32-unknown-unknown wasm32v1-none" { rustup target add wasm32-unknown-unknown wasm32v1-none }
Run "rustup toolchain install nightly --profile minimal" { rustup toolchain install nightly --profile minimal }
Run "cargo install wasm-bindgen-cli --version 0.2.129 --locked" { cargo install wasm-bindgen-cli --version 0.2.129 --locked }
Run "cargo install cargo-fuzz --version 0.13.2 --locked" { cargo install cargo-fuzz --version 0.13.2 --locked }
Run "versions" { rustc --version; rustc +nightly --version; wasm-bindgen --version; cargo fuzz --version; rustup target list --installed }
Run "fuzz smoke test: new crate" { cargo new --lib C:\dev\m1\fuzz-smoke --vcs none }
Push-Location C:\dev\m1\fuzz-smoke
Run "cargo +nightly fuzz init" { cargo +nightly fuzz init }
Run "cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=10" { cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=10 }
if ($LASTEXITCODE -ne 0) {
    Run "again without AddressSanitizer" { cargo +nightly fuzz run --sanitizer none fuzz_target_1 -- -max_total_time=10 }
}
Pop-Location
