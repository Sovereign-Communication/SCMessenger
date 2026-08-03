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

# SCMessenger - Native Installation Guide

Status: Current
Last updated: 2026-07-25

Build and run SCMessenger directly on your system without Docker.

## [Needs Revalidation] Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/Sovereign-Communication/SCMessenger.git
cd SCMessenger

# 2. Build (requires Rust)
cargo build --release --bin scmessenger-cli

# 3. Run
./target/release/scmessenger-cli start
```

---

## [Needs Revalidation] Prerequisites

### [Needs Revalidation] Install Rust

SCMessenger is written in Rust. You need Rust 1.70+ installed.

#### [Needs Revalidation] Linux & macOS
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### [Needs Revalidation] Windows
Download and run: https://rustup.rs/

Or via PowerShell:
```powershell
winget install Rustlang.Rustup
```

Verify installation:
```bash
rustc --version
cargo --version
```

### [Needs Revalidation] System Dependencies

#### [Needs Revalidation] macOS
```bash
# Xcode Command Line Tools (usually already installed)
xcode-select --install
```

#### [Needs Revalidation] Linux (Debian/Ubuntu)
```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
```

#### [Needs Revalidation] Linux (Fedora/RHEL)
```bash
sudo dnf install -y gcc openssl-devel pkg-config
```

#### [Needs Revalidation] Linux (Arch)
```bash
sudo pacman -S base-devel openssl pkg-config
```

#### [Needs Revalidation] Windows
- Visual Studio Build Tools (required for Rust)
- Installs automatically via rustup

---

## [Needs Revalidation] Installation

### [Needs Revalidation] Option 1: Build from Source (Recommended)

```bash
# Clone repository
git clone https://github.com/Sovereign-Communication/SCMessenger.git
cd SCMessenger

# Build release binary (5-10 minutes first time)
cargo build --release --bin scmessenger-cli

# Binary location:
# - Linux/macOS: ./target/release/scmessenger-cli
# - Windows: .\target\release\scmessenger-cli.exe
```

### [Needs Revalidation] Option 2: Install Globally

```bash
# Build and install to ~/.cargo/bin (in your PATH)
cargo install --path cli

# Now you can run from anywhere:
scmessenger-cli start
```

### [Needs Revalidation] Option 3: Create Alias

Add to your shell config (`~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish`):

```bash
alias scm='/path/to/SCMessenger/target/release/scmessenger-cli'
```

Then reload:
```bash
source ~/.bashrc  # or ~/.zshrc
```

Now you can use:
```bash
scm start
scm identity
scm contact add ...
```

---

## [Needs Revalidation] First Run

### [Needs Revalidation] 1. Initialize Identity

```bash
./target/release/scmessenger-cli init
```

**Output:**
```
Initializing SCMessenger...

  [OK] Configuration
  [OK] Data directory: /Users/you/.local/share/scmessenger
  [OK] Identity created

Identity Information:
  ID:         <redacted-pubkey>
  Public Key: 8c7a3b1e...

Next steps:
  • Add contacts: scm contact add <peer-id> <public-key> --name <nickname>
  • Start node:   scm start
```

### [Needs Revalidation] 2. Start the Node

```bash
./target/release/scmessenger-cli start
```

Or with custom port:
```bash
./target/release/scmessenger-cli start --port 9000
```

**Default Ports:**
- **9000**: WebSocket UI / API
- **9001**: P2P Network (automatically 9000 + 1)

**Both ports must be open in your firewall for internet connectivity.**

### [Needs Revalidation] 3. View Your Identity

```bash
./target/release/scmessenger-cli identity
```

---

## [Current] Connecting to Other Nodes

There are no dedicated relays and no bootstrap node role in SCMessenger -- only
nodes, and every node is a full relay. Peers are learned automatically:

- **On a local network:** mDNS finds other SCMessenger nodes with no
  configuration. If you are testing two machines on the same Wi-Fi, just start
  both.
- **Remote peers:** learned over the `/sc/ledger-exchange/1.0.0` protocol from
  peers you are already connected to, and persisted locally.

No addresses are compiled into any build. You only need to supply one manually in
the cold-start case: empty ledger *and* no peers on your local network.

Full model and troubleshooting: `docs/BOOTSTRAP.md`.

### [Current] Supply a Seed Peer Address

Get the multiaddr from the node you are joining -- a friend's node, or your own
server. Never reuse an address from documentation; there is no project-operated
address to reuse.

```bash
# Format: /ip4/<NODE_IP>/tcp/<P2P_PORT>/p2p/<PEER_ID>
./target/release/scmessenger-cli config set bootstrap_node_add \
  /ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>
```

Or per-run, without persisting:

```bash
export SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>"
```

`SC_BOOTSTRAP_NODES` is the only environment variable name the code reads; a
plain `BOOTSTRAP_NODES` is silently ignored.

**Restart to connect:**
```bash
./target/release/scmessenger-cli start
```

Startup logs the dial attempt and the resulting connection, then your own PeerId.

### [Current] View and Remove Seed Addresses

The `config` subcommand takes `set`, `get`, and `list` only -- there is no
`config bootstrap` subcommand.

```bash
# View
./target/release/scmessenger-cli config get bootstrap_nodes
./target/release/scmessenger-cli config list

# Remove
./target/release/scmessenger-cli config set bootstrap_node_remove <multiaddr>
```

---

## [Needs Revalidation] Adding Contacts

Before you can send messages, add contacts:

```bash
./target/release/scmessenger-cli contact add \
  <peer-id> \
  <public-key-hex> \
  --name "Alice"
```

**Example:**
```bash
./target/release/scmessenger-cli contact add \
  12D3KooW<redacted> \
  8c7a3b1e4f9d2a6c5e8b7a4d3f6e9c2b5a8d7e6f9c3a6b5e8d7f \
  --name "Bob"
```

### [Needs Revalidation] List Contacts

```bash
./target/release/scmessenger-cli contact list
```

### [Needs Revalidation] View Contact Details

```bash
./target/release/scmessenger-cli contact show Bob
```

---

## [Needs Revalidation] Sending Messages

### [Needs Revalidation] While Node is Running

Type in the interactive prompt:
```
> send Bob "Hello from SCMessenger!"
```

### [Needs Revalidation] Offline Mode (Encrypt Without Sending)

```bash
./target/release/scmessenger-cli send Bob "This will be encrypted and queued"
```

The message will be encrypted and stored. It will be sent automatically next time you start the node and connect to Bob's peer.

---

## [Needs Revalidation] Configuration

### [Needs Revalidation] View All Settings

```bash
./target/release/scmessenger-cli config list
```

### [Needs Revalidation] Change Settings

```bash
# Change default port
./target/release/scmessenger-cli config set listen_port 8000

# Disable mDNS (local network discovery)
./target/release/scmessenger-cli config set enable_mdns false

# Change max peers
./target/release/scmessenger-cli config set max_peers 100
```

### [Needs Revalidation] Configuration Location

- **Linux**: `~/.config/scmessenger/config.json`
- **macOS**: `~/.config/scmessenger/config.json`
- **Windows**: `%APPDATA%\scmessenger\config.json`

### [Needs Revalidation] Data Location

- **Linux**: `~/.local/share/scmessenger/`
- **macOS**: `~/.local/share/scmessenger/`
- **Windows**: `%LOCALAPPDATA%\scmessenger\`

---

## [Needs Revalidation] Firewall Configuration

### [Needs Revalidation] macOS

macOS will prompt you the first time. If you denied access:

1. Go to **System Settings** → **Privacy & Security** → **Firewall**
2. Click **Firewall Options**
3. Add `scmessenger-cli` and allow incoming connections

Or via command line:
```bash
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /path/to/scmessenger-cli
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp /path/to/scmessenger-cli
```

### [Needs Revalidation] Linux (UFW)

```bash
sudo ufw allow 9000/tcp
sudo ufw allow 9001/tcp
sudo ufw allow 9001/udp
```

### [Needs Revalidation] Linux (firewalld)

```bash
sudo firewall-cmd --add-port=9000/tcp --permanent
sudo firewall-cmd --add-port=9001/tcp --permanent
sudo firewall-cmd --add-port=9001/udp --permanent
sudo firewall-cmd --reload
```

### [Needs Revalidation] Windows

```powershell
# Run PowerShell as Administrator
New-NetFirewallRule -DisplayName "SCMessenger P2P" -Direction Inbound -Protocol TCP -LocalPort 9001 -Action Allow
New-NetFirewallRule -DisplayName "SCMessenger UI" -Direction Inbound -Protocol TCP -LocalPort 9000 -Action Allow
```

Or use Windows Defender Firewall GUI:
1. Open **Windows Defender Firewall**
2. **Advanced Settings** → **Inbound Rules** → **New Rule**
3. Port → TCP → Specific local ports: `9000,9001`
4. Allow the connection → Apply to all profiles

---

## [Needs Revalidation] Running as a Service

### [Needs Revalidation] Linux (systemd)

Create `/etc/systemd/system/scmessenger.service`:

```ini
[Unit]
Description=SCMessenger P2P Node
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/home/youruser
ExecStart=/home/youruser/SCMessenger/target/release/scmessenger-cli start
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable scmessenger
sudo systemctl start scmessenger

# Check status
sudo systemctl status scmessenger

# View logs
journalctl -u scmessenger -f
```

### [Needs Revalidation] macOS (launchd)

Create `~/Library/LaunchAgents/com.scmessenger.node.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.scmessenger.node</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/youruser/SCMessenger/target/release/scmessenger-cli</string>
        <string>start</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/Users/youruser/scmessenger.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/youruser/scmessenger.error.log</string>
</dict>
</plist>
```

Load and start:
```bash
launchctl load ~/Library/LaunchAgents/com.scmessenger.node.plist
launchctl start com.scmessenger.node

# Stop
launchctl stop com.scmessenger.node

# Unload
launchctl unload ~/Library/LaunchAgents/com.scmessenger.node.plist
```

### [Needs Revalidation] Windows (NSSM - Non-Sucking Service Manager)

Download NSSM: https://nssm.cc/download

```powershell
# Install service
nssm install SCMessenger "C:\Path\To\SCMessenger\target\release\scmessenger-cli.exe" start

# Start service
nssm start SCMessenger

# Stop service
nssm stop SCMessenger

# Remove service
nssm remove SCMessenger confirm
```

---

## [Needs Revalidation] Commands Reference

```bash
# Identity
scmessenger-cli init                    # Create new identity
scmessenger-cli identity                # Show your identity
scmessenger-cli identity export         # Export identity (backup)

# Node
scmessenger-cli start [--port 9000]     # Start P2P node
scmessenger-cli status                  # Show node status

# Contacts
scmessenger-cli contact add <peer-id> <pubkey> [--name "Name"]
scmessenger-cli contact list
scmessenger-cli contact show <name-or-peer-id>
scmessenger-cli contact remove <name-or-peer-id>
scmessenger-cli contact search <query>

# Messages
scmessenger-cli send <contact> <message>
scmessenger-cli history [--peer <peer-id>] [--limit 20]
scmessenger-cli history --search "keyword"

# Configuration
scmessenger-cli config list
scmessenger-cli config get <key>
scmessenger-cli config set <key> <value>

# Seed peer addresses (there is no `config bootstrap` subcommand)
scmessenger-cli config set bootstrap_node_add <multiaddr>
scmessenger-cli config set bootstrap_node_remove <multiaddr>
scmessenger-cli config get bootstrap_nodes

# Testing
scmessenger-cli test                    # Run self-tests
```

---

## [Needs Revalidation] Troubleshooting

### [Needs Revalidation] "Permission denied" on macOS

macOS blocks unsigned binaries by default:

1. Try to run the binary
2. Go to **System Settings** → **Privacy & Security**
3. Click **Allow Anyway** next to the blocked app
4. Run again and click **Open**

Or disable Gatekeeper temporarily:
```bash
sudo spctl --master-disable
# Run the app
sudo spctl --master-enable
```

### [Needs Revalidation] "No such file or directory" - Shared Library

If you see `error while loading shared libraries: libssl.so.3`:

**Linux:**
```bash
# Debian/Ubuntu
sudo apt-get install libssl3

# Fedora
sudo dnf install openssl-libs
```

**macOS:**
```bash
brew install openssl@3
```

### [Needs Revalidation] Port Already in Use

```bash
# Find what's using the port
# Linux/macOS
lsof -i :9000
lsof -i :9001

# Windows
netstat -ano | findstr :9000

# Kill the process or change ports
scmessenger-cli start --port 8000
```

### [Current] Peer Count Stays at 0

1. **On a LAN, check mDNS** - Another SCMessenger node on the same network should
   be found automatically. Client isolation on guest Wi-Fi blocks this entirely.
2. **Cold start with no local peers** - Expected. Supply one seed peer address
   (see "Connecting to Other Nodes" above). Nothing is compiled in to fall back
   on.
3. **Check the variable name** - `SC_BOOTSTRAP_NODES`, not `BOOTSTRAP_NODES`.
4. **Check firewall** - Ensure ports 9000 and 9001 are open.
5. **Check logs** - Look for dial and connection errors.
6. **Test connectivity:**
   ```bash
   # From another machine, test if port is reachable
   nc -zv <NODE_IP> 9001
   ```

### [Needs Revalidation] Build Fails on Low Memory

If compilation crashes:

```bash
# Reduce parallel jobs
cargo build --release --bin scmessenger-cli -j 2

# Or add swap space (Linux)
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

---

## [Needs Revalidation] Updating

```bash
# Pull latest code
cd SCMessenger
git pull origin main

# Rebuild
cargo build --release --bin scmessenger-cli

# Your identity and data are preserved
```

---

## [Needs Revalidation] Uninstalling

```bash
# Remove binary
rm -rf /path/to/SCMessenger

# Remove data (WARNING: Deletes your identity and messages)
# Linux/macOS
rm -rf ~/.local/share/scmessenger
rm -rf ~/.config/scmessenger

# Windows
rmdir /s "%LOCALAPPDATA%\scmessenger"
rmdir /s "%APPDATA%\scmessenger"
```

---

## [Needs Revalidation] Next Steps

- See [docs/platform/DOCKER_QUICKSTART.md](platform/DOCKER_QUICKSTART.md) for Docker deployment
- See [README.md](README.md) for architecture details
- Report issues: https://github.com/Sovereign-Communication/SCMessenger/issues

---

## [Needs Revalidation] Platform-Specific Notes

### [Needs Revalidation] macOS M1/M2/M3 (Apple Silicon)

SCMessenger works natively on ARM64. No Rosetta required.

### [Needs Revalidation] Windows

- Use PowerShell or Windows Terminal (not CMD)
- Paths use backslashes: `.\target\release\scmessenger-cli.exe`
- Windows Defender may flag the binary - add exclusion if needed

### [Needs Revalidation] Linux ARM (Raspberry Pi)

```bash
# Install Rust for ARM
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build (may take 20-30 minutes on Pi 4)
cargo build --release --bin scmessenger-cli
```

---

**Ready to connect!** [LAUNCH]

Your Peer ID is your identity on the network. Share it with others to allow them to add you as a contact and send you messages.
