> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

## [Current] Section Action Outcome (2026-02-23)

- `move`: current verified behavior and active priorities belong in `docs/CURRENT_STATE.md` and `REMAINING_WORK_TRACKING.md`.
- `move`: rollout and architecture-level decisions belong in `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and `docs/REPO_CONTEXT.md`.
- `rewrite`: operational commands/examples in this file require revalidation against current code/scripts before use.
- `keep`: retain this file as supporting context and workflow/reference detail.
- `delete/replace`: do not use this file alone as authoritative current-state truth; use canonical docs above.

# SCMessenger CLI — Windows Setup Guide

> Legacy OS-specific guide. Prefer `cli/README.md` for the current command surface.

Complete installation and usage guide for SCMessenger command-line interface on Windows.

## [Needs Revalidation] System Requirements

- **Windows Version**: Windows 10 (1809+) or Windows 11
- **Processor**: x86_64 (64-bit Intel/AMD)
- **RAM**: 512 MB minimum, 1 GB recommended
- **Disk Space**: 200 MB for development tools + 100 MB for application
- **Network**: Internet connection for P2P connectivity

## [Needs Revalidation] Installation

### [Needs Revalidation] Prerequisites

#### [Needs Revalidation] 1. Install Visual Studio Build Tools

SCMessenger requires C++ build tools for compilation.

**Option A: Visual Studio 2022 (Full IDE)**
1. Download from https://visualstudio.microsoft.com/downloads/
2. Run installer
3. Select "Desktop development with C++"
4. Install

**Option B: Build Tools Only (Recommended for CLI)**
1. Download from https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
2. Run installer
3. Select "C++ build tools"
4. Ensure these components are selected:
   - MSVC v143 - VS 2022 C++ x64/x86 build tools
   - Windows 10/11 SDK
5. Install

#### [Needs Revalidation] 2. Install Rust

```powershell
# Download and run rustup-init.exe from:
# https://www.rust-lang.org/tools/install

# Or use PowerShell to download and run:
Invoke-WebRequest -Uri https://win.rustup.rs -OutFile rustup-init.exe
.\rustup-init.exe

# Follow the prompts (press Enter for defaults)
# Close and reopen PowerShell/CMD after installation

# Verify installation
rustc --version
cargo --version
```

#### [Needs Revalidation] 3. Install Git (Optional but Recommended)

```powershell
# Download from https://git-scm.com/download/win
# Or install via winget (Windows 11 or Windows 10 with App Installer)
winget install Git.Git
```

### [Needs Revalidation] Building SCMessenger CLI

#### [Needs Revalidation] Method 1: Using PowerShell

```powershell
# Clone the repository
git clone https://github.com/yourusername/SCMessenger.git
cd SCMessenger

# Build release version
cargo build --release -p scmessenger-cli

# The binary will be at: target\release\scmessenger-cli.exe

# Copy to a location in your PATH
# Option 1: User profile bin
mkdir $HOME\bin -ErrorAction SilentlyContinue
copy target\release\scmessenger-cli.exe $HOME\bin\scm.exe

# Add to PATH if not already present
$env:Path += ";$HOME\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$HOME\bin", [EnvironmentVariableTarget]::User)

# Option 2: System-wide (requires admin)
copy target\release\scmessenger-cli.exe C:\Windows\System32\scm.exe

# Verify installation
scm --version
```

#### [Needs Revalidation] Method 2: Using Command Prompt

```cmd
:: Clone the repository
git clone https://github.com/yourusername/SCMessenger.git
cd SCMessenger

:: Build release version
cargo build --release -p scmessenger-cli

:: Copy to user directory
mkdir %USERPROFILE%\bin
copy target\release\scmessenger-cli.exe %USERPROFILE%\bin\scm.exe

:: Add to PATH (restart CMD after this)
setx PATH "%PATH%;%USERPROFILE%\bin"

:: Verify (in new CMD window)
scm --version
```

#### [Needs Revalidation] Method 3: Using Cargo Install

```powershell
# Install directly to cargo bin directory
cd SCMessenger
cargo install --path cli

# Cargo bin is usually already in PATH
# Default location: %USERPROFILE%\.cargo\bin

# Verify
scm --version
```

## [Needs Revalidation] First-Time Setup

### [Needs Revalidation] 1. Initialize Your Identity

```powershell
scm init
```

**What this does:**
- Creates `%APPDATA%\scmessenger\` directory
- Generates Ed25519 keypair
- Creates default configuration
- Initializes contact and message databases

**Output:**
```
 Configuration initialized
 Data directory: C:\Users\YourName\AppData\Roaming\scmessenger
 Identity created successfully

Your Peer ID: 12D3KooWABC123...
```

### [Needs Revalidation] 2. View Your Identity

```powershell
scm identity show
```

**Save this information** — your Peer ID is how others will contact you.

### [Current] 3. Supply a Seed Peer Address (Optional)

On a local network, peers are found automatically via mDNS -- nothing to
configure. Supply an address only for the cold-start case: an empty ledger and no
peers on your LAN. Get the multiaddr from the SCMessenger node you are joining.

```powershell
scm config set bootstrap_node_add /ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>
scm config get bootstrap_nodes
```

`config` takes only `set`, `get`, and `list`; there is no `config bootstrap`
subcommand. After the first connection, peers are learned and persisted via the
`/sc/ledger-exchange/1.0.0` protocol. See `docs/BOOTSTRAP.md`.

> [WARNING] Do not use the public libp2p/IPFS bootstrap nodes
> (`/dnsaddr/bootstrap.libp2p.io/...`), which earlier revisions of this guide
> suggested. They run a different network, host no SCMessenger peers, and are
> exactly the external infrastructure dependency SCMessenger is built to avoid.

## [Needs Revalidation] File Locations on Windows

SCMessenger uses standard Windows application data directories:

| Component | Path |
|-----------|------|
| Configuration | `%APPDATA%\scmessenger\config.json` |
| Data Directory | `%APPDATA%\scmessenger\` |
| Contacts Database | `%APPDATA%\scmessenger\contacts\` |
| Message History | `%APPDATA%\scmessenger\history\` |

**Typical full paths:**
- Config: `C:\Users\YourName\AppData\Roaming\scmessenger\config.json`
- Data: `C:\Users\YourName\AppData\Roaming\scmessenger\`

### [Needs Revalidation] Viewing Files

```powershell
# Open config directory in Explorer
explorer $env:APPDATA\scmessenger

# View config file
notepad $env:APPDATA\scmessenger\config.json

# View config in PowerShell
Get-Content $env:APPDATA\scmessenger\config.json

# View config with formatting (requires ConvertFrom-Json)
Get-Content $env:APPDATA\scmessenger\config.json | ConvertFrom-Json

# Check directory size
(Get-ChildItem -Recurse $env:APPDATA\scmessenger | Measure-Object -Property Length -Sum).Sum / 1MB
```

### [Needs Revalidation] Backing Up Your Identity

️ **IMPORTANT**: Back up your identity to prevent data loss!

```powershell
# Create backup directory
mkdir $HOME\Documents\SCMessenger-Backups -ErrorAction SilentlyContinue

# Export identity
scm identity export > $HOME\Documents\SCMessenger-Backups\identity.txt

# Backup entire config directory
$date = Get-Date -Format "yyyyMMdd"
Compress-Archive -Path $env:APPDATA\scmessenger -DestinationPath $HOME\Documents\SCMessenger-Backups\scm-backup-$date.zip

# Verify backup
Get-ChildItem $HOME\Documents\SCMessenger-Backups
```

## [Needs Revalidation] Network Configuration

### [Needs Revalidation] Windows Firewall

SCMessenger needs to accept incoming P2P connections.

#### [Needs Revalidation] Configure via PowerShell (Admin Required)

```powershell
# Run PowerShell as Administrator

# Allow SCMessenger through firewall
New-NetFirewallRule -DisplayName "SCMessenger CLI" -Direction Inbound -Program "$HOME\bin\scm.exe" -Action Allow

# Or allow specific port
scm config set listen_port 9999
New-NetFirewallRule -DisplayName "SCMessenger Port 9999" -Direction Inbound -LocalPort 9999 -Protocol TCP -Action Allow
New-NetFirewallRule -DisplayName "SCMessenger Port 9999 UDP" -Direction Inbound -LocalPort 9999 -Protocol UDP -Action Allow

# Verify rules
Get-NetFirewallRule -DisplayName "*SCMessenger*"
```

#### [Needs Revalidation] Configure via GUI

1. Open **Windows Security** → **Firewall & network protection**
2. Click **Allow an app through firewall**
3. Click **Change settings** (admin required)
4. Click **Allow another app...**
5. Browse to `scm.exe` location
6. Check both **Private** and **Public** networks
7. Click **Add**

### [Needs Revalidation] Port Forwarding (Optional)

For better connectivity behind NAT/router:

1. Set a fixed port: `scm config set listen_port 9999`
2. Log into your router admin panel
3. Navigate to Port Forwarding settings
4. Add rule:
   - **External Port**: 9999
   - **Internal Port**: 9999
   - **Internal IP**: Your PC's local IP (e.g., x.x.x.x)
   - **Protocol**: Both TCP and UDP
5. Save settings

## [Needs Revalidation] Running as Windows Service

### [Needs Revalidation] Using NSSM (Non-Sucking Service Manager)

NSSM allows running console applications as Windows services.

#### [Needs Revalidation] 1. Install NSSM

```powershell
# Download from https://nssm.cc/download
# Or via Chocolatey
choco install nssm

# Or manually:
# 1. Download nssm-2.24.zip
# 2. Extract to C:\nssm
# 3. Add C:\nssm\win64 to PATH
```

#### [Needs Revalidation] 2. Create Service

```powershell
# Run PowerShell as Administrator

# Create service
nssm install SCMessenger "$HOME\.cargo\bin\scm.exe" start

# Configure service
nssm set SCMessenger AppDirectory "$env:APPDATA\scmessenger"
nssm set SCMessenger DisplayName "SCMessenger P2P Messaging"
nssm set SCMessenger Description "Decentralized encrypted messaging node"
nssm set SCMessenger Start SERVICE_AUTO_START

# Start service
nssm start SCMessenger

# Check status
nssm status SCMessenger

# View logs
Get-Content "$env:APPDATA\scmessenger\service.log" -Tail 50 -Wait
```

#### [Needs Revalidation] 3. Manage Service

```powershell
# Start service
nssm start SCMessenger

# Stop service
nssm stop SCMessenger

# Restart service
nssm restart SCMessenger

# Remove service
nssm remove SCMessenger confirm

# Or use standard Windows service commands
sc start SCMessenger
sc stop SCMessenger
sc query SCMessenger
```

### [Needs Revalidation] Using Windows Task Scheduler (Alternative)

```powershell
# Create a scheduled task to run at startup
$action = New-ScheduledTaskAction -Execute "$HOME\.cargo\bin\scm.exe" -Argument "start"
$trigger = New-ScheduledTaskTrigger -AtStartup
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType S4U
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries

Register-ScheduledTask -TaskName "SCMessenger" -Action $action -Trigger $trigger -Principal $principal -Settings $settings

# Manually run task
Start-ScheduledTask -TaskName "SCMessenger"

# Check task status
Get-ScheduledTask -TaskName "SCMessenger"

# Remove task
Unregister-ScheduledTask -TaskName "SCMessenger" -Confirm:$false
```

## [Needs Revalidation] PowerShell Integration

### [Needs Revalidation] Tab Completion (PowerShell 5.0+)

```powershell
# Generate completion script
scm --generate-completion powershell | Out-File -FilePath $PROFILE\..\Completions\scm.ps1

# Or manually add to profile
notepad $PROFILE

# Add this line:
# Register-ArgumentCompleter -CommandName scm -ScriptBlock { ... }

# Reload profile
. $PROFILE
```

### [Needs Revalidation] PowerShell Aliases

Add to your PowerShell profile:

```powershell
# Edit profile
notepad $PROFILE

# Add these aliases:
Set-Alias scmsg scm
function scms { scm status }
function scmh { scm history @args }
function scmc { scm contact @args }
function scmsend { param($contact, $message) scm send $contact $message }

# Backup function
function scmbackup {
    $date = Get-Date -Format "yyyyMMdd"
    $backupPath = "$HOME\Documents\SCMessenger-Backups"
    New-Item -ItemType Directory -Force -Path $backupPath | Out-Null
    Compress-Archive -Path $env:APPDATA\scmessenger -DestinationPath "$backupPath\scm-backup-$date.zip" -Force
    Write-Host "Backup created: $backupPath\scm-backup-$date.zip" -ForegroundColor Green
}

# Save and reload
. $PROFILE
```

## [Needs Revalidation] Common Usage Examples

### [Needs Revalidation] Example 1: Basic Messaging

```powershell
# Add a contact
scm contact add 12D3KooWXYZ... abc123... --name Alice

# Start messaging node
scm start

# In interactive mode:
# > send Alice Hello from Windows!
#  Message sent to Alice

# Wait for reply
#  Alice: Hi! Nice to hear from you!

# > quit
```

### [Needs Revalidation] Example 2: Scripted Messaging

```powershell
# Send automated message
$recipient = "Alice"
$message = "Automated backup completed at $(Get-Date)"
scm send $recipient $message
```

### [Needs Revalidation] Example 3: Message History Export

```powershell
# Export history to CSV
scm history --limit 1000 | Out-File -FilePath "$HOME\Documents\scm-history.txt"

# Or with PowerShell processing
$history = scm history --limit 100
$history | Select-String "Alice" | Out-File "$HOME\Documents\alice-conversation.txt"
```

## [Needs Revalidation] Advanced Configuration

### [Needs Revalidation] Custom Data Location

```powershell
# Use different drive for storage (e.g., D:)
scm config set storage_path "D:\SCMessenger-Data"

# Verify
scm config get storage_path
```

### [Needs Revalidation] Performance Tuning

```powershell
# Increase connection limit
scm config set max_connections 200

# Set specific port
scm config set listen_port 9999

# Disable DHT for faster startup
scm config set enable_dht false
```

### [Needs Revalidation] Multiple Profiles

```powershell
# Create different config directories for multiple identities
$env:APPDATA = "C:\Users\YourName\AppData\Roaming\SCMessenger-Profile2"
scm init

# Switch back to default
Remove-Item Env:\APPDATA
```

## [Needs Revalidation] Troubleshooting

### [Needs Revalidation] Issue: Command not found

```powershell
# Check if scm is in PATH
where.exe scm

# If not found, add cargo bin to PATH
$env:Path += ";$HOME\.cargo\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::User)

# Restart PowerShell and try again
```

### [Needs Revalidation] Issue: "OpenSSL not found" during build

```powershell
# Option 1: Install vcpkg and OpenSSL
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg
C:\vcpkg\bootstrap-vcpkg.bat
C:\vcpkg\vcpkg integrate install
C:\vcpkg\vcpkg install openssl:x64-windows

# Option 2: Use Chocolatey
choco install openssl

# Option 3: Use Rust's vendored OpenSSL
$env:OPENSSL_STATIC = "1"
cargo build --release -p scmessenger-cli
```

### [Needs Revalidation] Issue: Firewall blocking connections

```powershell
# Check Windows Defender Firewall status
Get-NetFirewallProfile | Select-Object Name, Enabled

# Temporarily disable for testing (admin required)
Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled False

# Re-enable
Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True

# Or add exception (see Network Configuration section above)
```

### [Needs Revalidation] Issue: Permission denied errors

```powershell
# Check file permissions
icacls $env:APPDATA\scmessenger

# Fix permissions (give yourself full control)
icacls $env:APPDATA\scmessenger /grant "${env:USERNAME}:F" /T

# Reset ACLs to defaults
icacls $env:APPDATA\scmessenger /reset /T
```

### [Needs Revalidation] Issue: Cannot connect to peers

```powershell
# Check network connectivity
Test-NetConnection 8.8.8.8

# Check if port is open (if using specific port)
Test-NetConnection -ComputerName localhost -Port 9999

# Check firewall rules
Get-NetFirewallRule -DisplayName "*SCMessenger*"

# Enable verbose logging
$env:RUST_LOG = "debug"
scm start
```

### [Needs Revalidation] Issue: High CPU usage

```powershell
# Check process resource usage
Get-Process scm | Format-Table Name, CPU, PM -AutoSize

# View detailed process info
Get-Process scm | Select-Object *

# Kill process if needed
Stop-Process -Name scm -Force
```

## [Needs Revalidation] Windows Terminal Integration

### [Needs Revalidation] Custom Profile

Add SCMessenger profile to Windows Terminal:

1. Open Windows Terminal
2. Press `Ctrl+,` for settings
3. Click **Add a new profile**
4. Configure:
   - **Name**: SCMessenger
   - **Command line**: `scm start`
   - **Starting directory**: `%APPDATA%\scmessenger`
   - **Icon**: (optional custom icon)
5. Save

### [Needs Revalidation] Keyboard Shortcut

Add to Windows Terminal settings.json:

```json
{
    "command": {
        "action": "newTab",
        "commandline": "scm start",
        "profile": "SCMessenger"
    },
    "keys": "ctrl+shift+m"
}
```

## [Needs Revalidation] Security Best Practices

### [Needs Revalidation] Windows Defender

SCMessenger should not trigger Windows Defender, but if it does:

```powershell
# Add exclusion (admin required)
Add-MpPreference -ExclusionPath "$HOME\.cargo\bin\scm.exe"
Add-MpPreference -ExclusionPath "$env:APPDATA\scmessenger"

# View exclusions
Get-MpPreference | Select-Object -ExpandProperty ExclusionPath
```

### [Needs Revalidation] User Account Control (UAC)

SCMessenger does not require admin privileges. If prompted for admin access, decline and check the installation path.

### [Needs Revalidation] File Encryption

Enable BitLocker or use EFS to encrypt your identity:

```powershell
# Use Encrypting File System (EFS)
cipher /e $env:APPDATA\scmessenger

# Verify encryption
cipher /c $env:APPDATA\scmessenger\config.json
```

### [Needs Revalidation] Regular Backups

```powershell
# Create automatic backup script
$script = @'
$date = Get-Date -Format "yyyyMMdd-HHmm"
$backupPath = "$env:USERPROFILE\Documents\SCMessenger-Backups"
New-Item -ItemType Directory -Force -Path $backupPath | Out-Null
Compress-Archive -Path $env:APPDATA\scmessenger -DestinationPath "$backupPath\scm-backup-$date.zip" -Force
# Keep only last 30 days
Get-ChildItem $backupPath -Filter "scm-backup-*.zip" | Where-Object {$_.LastWriteTime -lt (Get-Date).AddDays(-30)} | Remove-Item
'@

# Save script
$script | Out-File -FilePath "$HOME\Documents\scm-backup.ps1"

# Create scheduled task to run daily
$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-File `"$HOME\Documents\scm-backup.ps1`""
$trigger = New-ScheduledTaskTrigger -Daily -At 2am
Register-ScheduledTask -TaskName "SCMessenger Backup" -Action $action -Trigger $trigger
```

## [Needs Revalidation] Performance Notes

### [Needs Revalidation] Resource Usage on Windows

Typical resource usage:
- **Memory**: 30-60 MB idle, 80-200 MB active
- **CPU**: < 1% idle, 2-8% active
- **Disk**: < 1 MB/s during normal operation
- **Network**: < 1 KB/s idle, 5-20 KB/s active

### [Needs Revalidation] Monitoring

```powershell
# Monitor in real-time
while ($true) {
    Clear-Host
    Get-Process scm | Format-Table Name, CPU, WS -AutoSize
    Start-Sleep -Seconds 2
}

# Network connections
netstat -ano | findstr scm

# Detailed process info
Get-Process scm | Select-Object *
```

## [Needs Revalidation] Uninstallation

```powershell
# Stop service (if running)
nssm stop SCMessenger
nssm remove SCMessenger confirm

# Or stop scheduled task
Stop-ScheduledTask -TaskName "SCMessenger"
Unregister-ScheduledTask -TaskName "SCMessenger" -Confirm:$false

# Remove binary
Remove-Item $HOME\.cargo\bin\scm.exe
# Or
Remove-Item C:\Windows\System32\scm.exe

# Remove data (backup first!)
Remove-Item -Recurse -Force $env:APPDATA\scmessenger

# Remove from PATH
$path = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
$newPath = ($path.Split(';') | Where-Object { $_ -ne "$HOME\bin" }) -join ';'
[Environment]::SetEnvironmentVariable("Path", $newPath, [EnvironmentVariableTarget]::User)

# Remove firewall rules
Remove-NetFirewallRule -DisplayName "*SCMessenger*"
```

## [Needs Revalidation] Package Managers

### [Needs Revalidation] Chocolatey (Future)

```powershell
# Not yet available, but planned:
# choco install scmessenger-cli
```

### [Needs Revalidation] Scoop (Future)

```powershell
# Not yet available, but planned:
# scoop install scmessenger-cli
```

### [Needs Revalidation] Winget (Future)

```powershell
# Not yet available, but planned:
# winget install SCMessenger.CLI
```

## [Needs Revalidation] Getting Help

```powershell
# Built-in help
scm --help
scm <command> --help

# Run diagnostics
scm test

# Check status
scm status

# View Windows Event Viewer (if running as service)
Get-EventLog -LogName Application -Source SCMessenger -Newest 50
```

## [Needs Revalidation] Next Steps

1. Share your Peer ID: `scm identity show`
2. Add contacts: `scm contact add <peer-id> <public-key> --name <nickname>`
3. Start messaging: `scm start`

For more information, see the [main CLI README](../cli/README.md).

## [Needs Revalidation] Windows-Specific Tips

### [Needs Revalidation] Run at Startup (Simple)

1. Press `Win+R`
2. Type `shell:startup`
3. Create shortcut to `scm.exe start`

### [Needs Revalidation] Desktop Shortcut

```powershell
# Create desktop shortcut
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$Home\Desktop\SCMessenger.lnk")
$Shortcut.TargetPath = "$HOME\.cargo\bin\scm.exe"
$Shortcut.Arguments = "start"
$Shortcut.WorkingDirectory = "$env:APPDATA\scmessenger"
$Shortcut.Save()
```

### [Needs Revalidation] Context Menu Integration

Add "Send with SCMessenger" to right-click menu (advanced, requires registry editing).

---

For issues, questions, or contributions, visit:
- **GitHub**: https://github.com/yourusername/SCMessenger
- **Issues**: https://github.com/yourusername/SCMessenger/issues
