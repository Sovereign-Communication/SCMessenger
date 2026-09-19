# stop_node.ps1 - stops the supervised node and its supervisor, deterministically.
#
# The documented recovery command when a node must be replaced or a supervisor
# must be re-armed (run_node_supervised.ps1 refuses to double-start):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\stop_node.ps1
#
# Ticket: HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md
$sup = Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" |
    Where-Object { $_.CommandLine -like '*run_node_supervised*' }
foreach ($s in $sup) {
    Write-Host "stopping supervisor PID $($s.ProcessId)"
    Stop-Process -Id $s.ProcessId -Force -ErrorAction SilentlyContinue
}

$nodes = Get-CimInstance Win32_Process -Filter "Name='scmessenger-cli.exe'"
foreach ($n in $nodes) {
    Write-Host "stopping node PID $($n.ProcessId)"
    Stop-Process -Id $n.ProcessId -Force -ErrorAction SilentlyContinue
}

Start-Sleep -Seconds 3

$left = @(Get-Process scmessenger-cli -ErrorAction SilentlyContinue).Count
Write-Host "remaining scmessenger-cli processes: $left"
$listeners = @(netstat -ano | Select-String -Pattern 'LISTENING' |
    Select-String -Pattern ':(80|443|8080|9002|9090|9001)\s')
Write-Host "remaining node listeners: $($listeners.Count)"
