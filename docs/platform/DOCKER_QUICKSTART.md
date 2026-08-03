> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# SCMessenger Docker Quick Start

Status: Current
Last updated: 2026-07-25

This guide gets you up and running with SCMessenger in Docker in under 5 minutes.

> **Model note:** SCMessenger has no dedicated relays and no bootstrap node role.
> There are only nodes, and every node is a full relay. Peers are learned through
> local discovery (mDNS on the LAN) and through ledger exchange
> (`/sc/ledger-exchange/1.0.0`). No addresses are shipped in any build, so a
> container with an empty ledger and no LAN peers needs **one** address you
> supply. See `docs/BOOTSTRAP.md`.
>
> **Environment variable:** the only name the code reads is
> **`SC_BOOTSTRAP_NODES`**. Plain `BOOTSTRAP_NODES` is silently ignored --
> including in `docker-compose.yml` and some files under `docker/`, which still
> set the unprefixed name. That is a defect in those files, not a second
> supported spelling.
>
> **Addresses in this guide are placeholders.** `<NODE_IP>` and `<PEER_ID>` come
> from a node you or someone you know operates. There is no project-operated
> address to copy.

## [Current] Section Action Outcome (2026-02-23)

- `rewrite`: this guide remains operational only where commands match current docker files and image/runtime behavior.
- `move`: community-operated deployment policy belongs in `docs/UNIFIED_GLOBAL_APP_PLAN.md`.
- `move`: current validated runtime behavior belongs in `docs/CURRENT_STATE.md`.
- `delete/replace`: done 2026-07-25 -- hardcoded node IPs removed, `BOOTSTRAP_NODES` corrected to `SC_BOOTSTRAP_NODES`, and the shipped-defaults framing replaced with the ledger-exchange model.

## [Needs Revalidation] Prerequisites

- Docker and Docker Compose installed
- Ports 9000 and 9001 available (or configure different ports)

## [Needs Revalidation] Single Node Setup

The simplest way to run SCMessenger:

```bash
# 1. Build the image
docker compose build

# 2. Start the node
docker compose up -d

# 3. View logs
docker compose logs -f

# 4. Access the interactive CLI
docker compose exec scmessenger bash -c "scm identity"
```

## [Needs Revalidation] GCP or Cloud Deployment

### [Needs Revalidation] One-Command Deploy

```bash
# Build and run on a GCP VM or any cloud server
docker build -t scmessenger -f docker/Dockerfile .

# Run with persistent storage
docker run -d \
  --name scmessenger \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  -v ~/scm_config:/root/.config/scmessenger \
  -e LISTEN_PORT=9000 \
  scmessenger

# View your identity and Peer ID
docker logs scmessenger
```

### [Current] Supplying a Seed Peer Address

A node on a LAN with other SCMessenger nodes needs no address at all -- mDNS finds
them. Supply an address only for the cold-start case: empty ledger, no local
peers, and you want internet connectivity.

```bash
# Multiaddr format:
# /ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>

docker run -d \
  --name scmessenger \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  -e LISTEN_PORT=9000 \
  -e SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>" \
  scmessenger
```

This address is a one-time entry point, not a dependency. Once connected, the
node receives peer records over ledger exchange and persists them, so the seed
address stops mattering.

## [Needs Revalidation] Connect Two Nodes (Local + Cloud)

### [Needs Revalidation] Step 1: Start your cloud node (GCP)

```bash
# On your GCP VM
docker run -d \
  --name scmessenger \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data:/root/.local/share/scmessenger \
  scmessenger

# Get the Peer ID from logs
docker logs scmessenger | grep "Peer ID"
# Example output: [OK] Peer ID: 12D3KooW<redacted>

# Get your public IP
curl ifconfig.me
# Output: your VM's public IP -- call it <NODE_IP>
```

**Your cloud node's multiaddress:**
```
/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>
```

Both values are specific to your deployment. Record them; the next step uses them.

### [Needs Revalidation] Step 2: Start your local node (Mac/Linux)

```bash
# On your local machine
docker run -d \
  --name scmessenger-local \
  -p 9000:9000 \
  -p 9001:9001 \
  -v ~/scm_data_local:/root/.local/share/scmessenger \
  -e SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>" \
  scmessenger

# Watch the dial and connection
docker logs -f scmessenger-local
```

### [Needs Revalidation] Step 3: Verify Connection

```bash
# On local machine - check peer count
docker exec scmessenger-local scm status

# You should see "Peers: 1" or more
```

## [Needs Revalidation] Without Docker (Native Binary)

If you prefer to run the binary directly:

```bash
# Build from source
cargo build --release --bin scmessenger-cli

# Start node
./target/release/scmessenger-cli start --port 9000

# Add a seed peer address (only needed for cold start with no LAN peers)
./target/release/scmessenger-cli config set bootstrap_node_add \
  /ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>

# Restart to connect
./target/release/scmessenger-cli start --port 9000
```

The `config` subcommand takes `set`, `get`, and `list` only. There is no
`config bootstrap` subcommand.

## [Needs Revalidation] Port Configuration

SCMessenger uses **two ports** by default:

- **Port 9000**: WebSocket interface (for web UI and API)
- **Port 9001**: P2P network communication (automatically set to `--port + 1`)

When you specify `--port 9000`, the P2P port becomes 9001.

**Both ports must be open in your firewall for internet connectivity.**

### [Needs Revalidation] GCP Firewall Example

```bash
gcloud compute firewall-rules create allow-scmessenger \
  --allow tcp:9000,udp:9000,tcp:9001,udp:9001 \
  --description="SCMessenger P2P and WebSocket traffic" \
  --direction=INGRESS
```

## [Needs Revalidation] Data Persistence

Your identity and messages are stored in:
- **Linux/Mac**: `~/.local/share/scmessenger/storage/`
- **Docker**: Mounted volume (e.g., `~/scm_data/`)

**Important**: The network keypair (which determines your Peer ID) is now persisted in:
- `network_keypair.dat` - Your Peer ID will remain constant across restarts

## [Needs Revalidation] Troubleshooting

### [Needs Revalidation] Peer Count Stays at 0

**Check 1**: Verify both nodes show "Listening on" messages:
```bash
docker logs scmessenger | grep "Listening on"
```

**Check 2**: Ensure firewall ports are open:
```bash
# Test from another machine
nc -zv <NODE_IP> 9001
```

**Check 3**: Verify the seed address format:
```
/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>
```
All three components must be correct.

**Check 4**: Check dial logs:
```bash
docker logs scmessenger | grep -i bootstrap
```

**Check 5**: Confirm the variable name. If you set `BOOTSTRAP_NODES` instead of
`SC_BOOTSTRAP_NODES`, the container starts fine and dials nothing -- the variable
is never read. This is the most common cause of a silent 0-peer container.

**Check 6**: If the ledger is empty and there are no LAN peers, 0 peers is the
correct behaviour until an address is supplied. Nothing is shipped to fall back
on.

### [Needs Revalidation] Identity Changes on Restart

This should be fixed now! The network keypair is persisted. If you still see changing Peer IDs:
- Ensure your volume mount is working (`-v ~/scm_data:/root/.local/share/scmessenger`)
- Check that `network_keypair.dat` exists in the data directory

### [Needs Revalidation] Port Already in Use

```bash
# Find what's using the port
lsof -i :9000
lsof -i :9001

# Kill the process or change ports
docker run -p 8000:9000 -p 8001:9001 -e LISTEN_PORT=9000 scmessenger
```

## [Needs Revalidation] Commands Reference

### [Needs Revalidation] Docker

```bash
# Start node
docker compose up -d

# Stop node
docker compose down

# View logs
docker compose logs -f

# Restart node
docker compose restart

# Remove everything (including data)
docker compose down -v
```

### [Needs Revalidation] CLI (inside container)

```bash
# Show identity
docker exec scmessenger scm identity

# Add contact
docker exec scmessenger scm contact add <peer_id> <public_key> --name "Alice"

# List contacts
docker exec scmessenger scm contact list

# Check status
docker exec scmessenger scm status

# View history
docker exec scmessenger scm history

# Seed peer address management
docker exec scmessenger scm config get bootstrap_nodes
docker exec scmessenger scm config set bootstrap_node_add <multiaddr>
docker exec scmessenger scm config set bootstrap_node_remove <multiaddr>
docker exec scmessenger scm config list
```

## [Needs Revalidation] Next Steps

1. Open the web UI: `http://localhost:9000` (UI implementation in progress)
2. Add contacts via CLI
3. Send messages!

## [Needs Revalidation] Support

- Report issues: https://github.com/Sovereign-Communication/SCMessenger/issues
- See main README.md for architecture details
