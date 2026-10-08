# Self-test for spike-gate.exe and spike-host's pipe: the gate must speak MCP, and must refuse
# whenever the host does not clearly allow (wrong token, host denies, closes, stalls, or is absent).
# The MCP lines go in through a file, not a PowerShell pipe: PowerShell 5.1 puts a byte-order mark
# (EF BB BF) in front of text it pipes into a program, which no real MCP client sends.
$ErrorActionPreference = "Continue"
$bin = "C:\dev\runFile\spikes\target\release"
$log = "C:\dev\m1\logs\selftest.gate.log"
$in = "C:\dev\m1\logs\selftest.mcp-in.jsonl"
$out = "C:\dev\m1\logs\selftest.mcp-out.jsonl"
Remove-Item $log, "$log.crashed", "C:\dev\m1\logs\selftest.host-gate.log" -ErrorAction SilentlyContinue
$mcp = @(
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"selftest","version":"0"}}}',
    '{"jsonrpc":"2.0","method":"notifications/initialized"}',
    '{"jsonrpc":"2.0","id":2,"method":"tools/list"}',
    '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"decide","arguments":{"tool_name":"Bash","input":{"command":"echo hi"},"tool_use_id":"t1"}}}'
)
[IO.File]::WriteAllText($in, ($mcp -join "`n") + "`n")  # UTF-8, no byte-order mark
$script:failures = 0
function Case([string]$Name, [string]$Mode, [string]$GateToken, [string]$Expect) {
    $srv = $null
    if ($Mode) {
        $srv = Start-Process "$bin\spike-host.exe" -PassThru -NoNewWindow -ArgumentList "serve --pipe steplet-spike-selftest --token good --gate-mode $Mode --deadline-ms 1500 --seconds 5"
        Start-Sleep -Milliseconds 700
    }
    $t = Measure-Command {
        Start-Process "$bin\spike-gate.exe" -ArgumentList "--pipe steplet-spike-selftest --token $GateToken --log $log" `
            -RedirectStandardInput $in -RedirectStandardOutput $out -NoNewWindow -Wait
    }
    $replies = @(Get-Content $out)
    if ($srv) { $srv.WaitForExit(10000) | Out-Null }
    $call = [string](@($replies | Where-Object { $_.Contains('"id":3') })[0])
    $ok = ($replies.Count -eq 3) -and $call.Contains($Expect)
    if (-not $ok) { $script:failures++ }
    "{0} {1} ({2:N1} s, {3} replies)`n    {4}" -f $(if ($ok) { "PASS" } else { "FAIL" }), $Name, $t.TotalSeconds, $replies.Count, $call
}
Case "host allows -> allow" "allow" "good" '\"behavior\":\"allow\"'
Case "wrong run token -> deny" "allow" "bad" 'wrong run token'
Case "host denies -> deny" "deny" "good" '\"behavior\":\"deny\"'
Case "host closes without answering -> deny" "drop" "good" 'no answer'
Case "host stalls -> deny at the host's deadline" "stall" "good" 'deadline'
Case "no host at all -> deny" "" "good" 'cannot reach Steplet'
# The gate logs the names of the variables it inherited. Here they are this shell's, not a run's: count only.
"--- gate log:"; Get-Content $log | ForEach-Object { $_ -replace '(environment variables): .*', '$1 (names not shown in the self-test)' }
"--- host log:"; Get-Content C:\dev\m1\logs\selftest.host-gate.log -ErrorAction SilentlyContinue
"failures: $script:failures"
