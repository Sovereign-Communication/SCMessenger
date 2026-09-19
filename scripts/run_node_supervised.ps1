# run_node_supervised.ps1 - bounded supervised restart for the SCMessenger CLI node.
#
# Why this exists: the log-silence watchdog exits 1 when a node wedges (no log
# output past SCM_LOG_SILENCE_TIMEOUT_SECS, default 600s). This host has no
# supervisor, so a detected wedge used to leave the node DOWN until a human
# noticed: the 2026-09-16 incident ran 2h06m (silent from 03:19:17Z, node
# stopped manually at 05:25Z) while the operator's message never moved.
#
# This is the smallest thing that closes that gap: run the node, restart it on a
# NON-ZERO exit with exponential backoff, and give up loudly after a bounded
# budget so a crash-looping node cannot spin forever.
#
# Trade-off (deliberate): no service framework, no install, no elevation. It
# supervises only while this wrapper process lives, so a logoff or reboot ends
# supervision. That is strictly better than nothing on an always-on desktop, but
# it is NOT release-grade supervision - a Windows service or a Task Scheduler
# entry with a restart policy is the real answer for the always-on/cloud role.
#
# Ticket: HANDOFF/todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run_node_supervised.ps1
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\run_node_supervised.ps1 -NodeArgs 'start -p 9001'
#
# Behaviour:
# - Exit 0 (Ctrl+C / graceful shutdown in the node) stops the wrapper: only
#   failures are restarted.
# - A child that ran >= HealthyRuntimeSec resets the restart budget, so a wedge
#   reached hours into a healthy run does not count against a crash loop.
# - Ctrl+C in this console does not forward to the node; stop the node with
#   `Stop-Process -Name scmessenger-cli`.

param(
    [string]$NodeArgs = 'start -p 9001',
    [string]$ExePath = '',
    [int]$MaxRestarts = 5,
    [int]$WindowMinutes = 60,
    [int]$BaseBackoffSec = 5,
    [int]$MaxBackoffSec = 60,
    [int]$HealthyRuntimeSec = 600,
    [string]$LogFile = ''
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($ExePath)) {
    $ExePath = Join-Path $repoRoot 'target\release\scmessenger-cli.exe'
}
if ([string]::IsNullOrWhiteSpace($LogFile)) {
    $LogFile = Join-Path $repoRoot 'tmp\node-supervisor.log'
}

if (-not (Test-Path $ExePath)) {
    Write-Host "[FAIL] node binary not found at $ExePath - build it with: cargo build --release -p scmessenger-cli"
    exit 2
}

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $LogFile) | Out-Null

function Write-Sup {
    param([string]$Level, [string]$Message)
    $stamp = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    $line = "$stamp  $Level $Message"
    # Explicit UTF8: Tee-Object/Out-File default to UTF-16 on PowerShell 5.1,
    # which makes the log unreadable to grep/tail.
    Add-Content -Path $LogFile -Value $line -Encoding UTF8
    Write-Host $line
}

Write-Sup '[INFO]' "supervisor start: exe=$ExePath args='$NodeArgs' max_restarts=$MaxRestarts window_min=$WindowMinutes"

# Pre-flight: a stale node holding the ports makes every launch fail instantly,
# which the wrapper would read as a crash loop and burn its restart budget.
$existing = @(Get-Process scmessenger-cli -ErrorAction SilentlyContinue)
if ($existing.Count -gt 0) {
    Write-Sup '[FAIL]' "a scmessenger-cli process is already running (PID $($existing[0].Id)); refusing to double-start. Stop it first: powershell -NoProfile -ExecutionPolicy Bypass -File scripts\stop_node.ps1"
    exit 3
}

Set-Location $repoRoot
$nodeArgList = @($NodeArgs -split '\s+' | Where-Object { $_ -ne '' })
$restarts = New-Object System.Collections.Generic.List[datetime]
$attempt = 0

while ($true) {
    $startedAt = Get-Date
    Write-Sup '[INFO]' "launching node (attempt $($attempt + 1))"
    # Call operator rather than Start-Process: $LASTEXITCODE is reliable here,
    # while Start-Process -PassThru returned an EMPTY ExitCode on this host and
    # the wrapper read that as failure. Observed 2026-09-16 05:33Z: bogus
    # restarts while the real cause was a stale node still holding the ports.
    $LASTEXITCODE = 0
    & $ExePath @nodeArgList
    $exitCode = $LASTEXITCODE
    if ($null -eq $exitCode) {
        $exitCode = 1
        Write-Sup '[WARNING]' 'exit code unavailable - treating as a failure'
    }
    $ranSec = [int]((Get-Date) - $startedAt).TotalSeconds
    Write-Sup '[WARNING]' "node exited code=$exitCode after ${ranSec}s"

    if ($exitCode -eq 0) {
        Write-Sup '[OK]' 'clean exit (code 0) - not restarting'
        break
    }

    if ($ranSec -ge $HealthyRuntimeSec) {
        Write-Sup '[INFO]' "runtime ${ranSec}s >= ${HealthyRuntimeSec}s - restart budget reset"
        $restarts.Clear()
        $attempt = 0
    }

    $cutoff = (Get-Date).AddMinutes(-1 * $WindowMinutes)
    $recent = @($restarts | Where-Object { $_ -gt $cutoff })
    $restarts.Clear()
    foreach ($r in $recent) { $restarts.Add($r) }

    if ($restarts.Count -ge $MaxRestarts) {
        Write-Sup '[FAIL]' "restart budget exhausted ($MaxRestarts restarts within ${WindowMinutes}m) - leaving the node down; investigate before restarting"
        exit 1
    }

    $restarts.Add((Get-Date))
    $attempt++
    $backoff = [Math]::Min($BaseBackoffSec * [Math]::Pow(2, $attempt - 1), $MaxBackoffSec)
    Write-Sup '[WARNING]' "restart $($restarts.Count)/$MaxRestarts in ${backoff}s"
    Start-Sleep -Seconds $backoff
}
