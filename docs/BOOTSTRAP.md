# Joining the Mesh: Peer Discovery and Node Addresses

Status: Current
Last updated: 2026-10-06

> Scope note: this document replaces the former "Bootstrap Node Configuration"
> guidance, which described a privileged tier of shipped bootstrap nodes. That
> tier does not exist. Canonical architecture reference:
> `docs/TRANSPORT_ARCHITECTURE.md`. Governance/trust reference:
> `docs/BOOTSTRAP_GOVERNANCE.md`. Operating a well-connected node:
> `docs/RELAY_OPERATOR_GUIDE.md`.

> [SUPERSEDED IN PART 2026-10-06] Operator decision for issue #469
> (`docs/BOOTSTRAP_RENDEZVOUS_DECISION_469.md`): the ONLY seed source is an
> invite (QR or invite info). Static or operator-supplied seed lists -- the
> `SC_BOOTSTRAP_NODES` variable (runtime and build-time), `config set
> bootstrap_node_add`, and "supply one address manually" cold start -- are
> superseded and scheduled for removal (tasks T4, T10). Sections below that
> describe them stay for traceability until the removal lands. The invite
> sections and the "no dedicated relays / every node is a full relay" model
> remain current.

## [Current] The Model in One Paragraph

SCMessenger has no dedicated relays and no bootstrap node role. There are only
**nodes**, and **every node is a full relay**. Every build -- desktop, mobile,
headless -- starts both a libp2p relay server and a relay client
unconditionally (`core/src/transport/behaviour.rs`, field `relay_server`, plus
`.with_relay_client(...)` in every swarm build path), and every SCMessenger peer
advertises itself as a relay in its libp2p `identify` agent string. A node with
a public address is not a special class of node; it is an ordinary node that
happens to be reachable.

Peers are learned two ways:

1. **Local discovery** -- mDNS on the LAN, plus BLE, Wi-Fi Aware / Wi-Fi Direct
   on Android, and Multipeer on iOS. No configuration, no internet.
2. **Ledger exchange** -- the `/sc/ledger-exchange/1.0.0` protocol. Nodes gossip
   the peer records they already know to the peers they are connected to. This
   is what replaced static bootstrap lists for learning about *remote* peers.

Kademlia DHT lookups, libp2p `identify`, and peer broadcast also contribute once
a node has at least one live connection. None of these require a privileged
entry node.

## [Current] There Are No Shipped Default Addresses

All compiled-in address lists are empty, by design:

| Location | Constant / field | Value |
|----------|------------------|-------|
| `core/src/transport/bootstrap.rs` | `CORE_BOOTSTRAP_NODES` | `&[]` |
| `cli/src/bootstrap.rs` | `DEFAULT_BOOTSTRAP_NODES` | `&[]` |
| `cli/src/config.rs` | `bootstrap_nodes` config default | empty |

The Rust core and the CLI contain no hardcoded routable IP addresses. Any node
address in a running install got there because a **user or operator supplied
it**. The project does not accept contributed addresses into a shipped default
list, and there is no PR process for doing so.

The remaining `bootstrap_*` names in code and config are historical vocabulary
for one thing only: *the optional list of peer addresses to dial on startup
before any peers are known*. Treat "bootstrap node" in config keys as
"user-supplied seed peer address", not as a node role.

## [Current] Cold Start: The Only Case That Needs Manual Input

A node needs exactly one reachable peer address, once, and only when **both** of
these are true:

- its ledger is empty (first run, or data directory wiped), and
- there are no peers on its local network to find via mDNS/BLE/Wi-Fi.

In that case the user supplies one address. After that first connection the node
receives peer records over ledger exchange, persists them, and no longer depends
on the address it started from:

- CLI ledger: `<data_dir>/peers.json` (`cli/src/ledger.rs`)
- Core/mobile ledger: `ledger.json` via `LedgerManager`
  (`core/src/store/ledger_entry.rs`)

Entries are added from `PeerIdentified` and `LedgerReceived` events and shared
outward via `to_shared_entries()` / `share_ledger()`.

On a LAN -- two laptops on the same Wi-Fi, a phone and a desktop in the same
room -- no address is needed at all. Start both and they find each other.

## [Current] Invites Carry a Seed Ledger -- Routing Only, No Identity

An invite token (`core/src/relay/invite.rs`, `InviteToken`) carries a
`seed_ledger`: a snapshot of the inviter's connection ledger, with the
inviter's own dialable address always first and never evicted. It holds at
most `MAX_SEED_LEDGER_ENTRIES` (16) records.

**A seed entry is a bare multiaddr and nothing else.** No peer id, no public
key, no nickname, no topics, no success/failure counters, no `last_seen`. An
invite says *where to knock*, not *who lives there*: all of those fields are
identity or behavioural metadata about a third party who never agreed to
appear in someone else's invite. The invitee dials the bare address, completes
the Noise handshake, and learns the peer's identity from Identify at connect
time -- the same path used for any mDNS- or gossip-learned peer.
`LedgerManager::annotate_identity()` attaches that identity locally afterwards.

This is safe because transport peer identity is not what secures messages.
Confidentiality is per-contact X25519 / XChaCha20-Poly1305 established out of
band from public keys, so reaching an unintended node at a given address leaks
nothing and decrypts nothing. Dropping the peer id does forgo dial-time
identity pinning, which is an availability property, not a confidentiality one.

The encoded invite is `SCI1:` + base64(token), checked against the 2953-byte QR
byte-mode budget at encode time; a token that would not scan is rejected rather
than silently truncated. No compression: 16 bare multiaddrs are roughly 500
bytes, so a compressor would only add attack surface.

The seed ledger is inside `get_signable_data()`, so it is covered by the
inviter's signature. Tampering with it -- adding, rewriting or removing an
entry -- invalidates the invite. This is what stops anyone who intercepts or
forwards an invite from injecting addresses that a fresh node would dial on
first launch.

**Residual privacy note (reduced, not eliminated).** An invite still discloses
the inviter's IP address and up to 15 other node IPs to whoever holds it. QR
codes get photographed, forwarded and posted publicly. Treat an invite as
though its address list were public:

- Do not post an invite QR code anywhere you would not post your home IP.
- Prefer short expiry windows for invites you share outside a room.
- A revoked or expired invite does not un-disclose addresses that were already
  read out of it.

That is unavoidable if the invite is to be useful at all -- seed delivery is
invite/QR only, there is no DNS seed and no shipped node list -- and it is now
bare routing data with no identities attached.

Imported seed addresses start unproven: `LedgerManager::import_seed_entries()`
adds them with `success_count = 0` and no identity fields, which keeps them out
of `dialable_addresses()` and `get_preferred_relays()`. They are surfaced
through `seed_addresses()` until a real connection promotes them. An existing
ledger entry is never touched by seed data -- counters, `last_seen` and any
known peer id are all left exactly as they are.

## [Current] Supplying a Seed Peer Address

Address format is a libp2p multiaddr:

```
/ip4/<NODE_IP>/tcp/<P2P_PORT>/p2p/<PEER_ID>
```

`<NODE_IP>`, `<P2P_PORT>` and `<PEER_ID>` come from the node you are joining.
Its operator can read them off that node with `scm identity` and the node's own
"Listening on" log lines. Never copy an address out of documentation -- addresses
are deployment-specific and there are no project-operated ones to copy.

### [Current] CLI

The `config` subcommand takes `set` / `get` / `list` only
(`cli/src/cli.rs`, `ConfigAction`). Seed addresses are managed through `set`
with a pseudo-key:

```bash
# Add a seed peer address
scmessenger-cli config set bootstrap_node_add /ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>

# Remove one
scmessenger-cli config set bootstrap_node_remove /ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>

# Inspect
scmessenger-cli config get bootstrap_nodes
scmessenger-cli config list
```

> [WARNING] Earlier revisions of this document showed
> `scm config bootstrap add|list|remove <addr>`. **That command form does not
> exist** and never did -- there is no `bootstrap` subcommand under `config`.
> Use the `config set bootstrap_node_add` / `config set bootstrap_node_remove`
> forms above.

### [Current] Environment Variable

The only environment variable the code reads is **`SC_BOOTSTRAP_NODES`**
(`cli/src/bootstrap.rs`, `core/src/transport/bootstrap.rs`). It takes a
comma-separated multiaddr list and, when set and non-empty, is the only source
used.

```bash
export SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>"
scmessenger-cli start
```

```bash
docker run -d \
  --name scmessenger \
  -p 9000:9000 -p 9001:9001 \
  -e SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>" \
  testbotz/scmessenger:latest
```

> [WARNING] A plain `BOOTSTRAP_NODES` variable is **silently ignored** -- nothing
> in the codebase reads it. Some compose files under `docker/` still set the
> unprefixed name; that is a known defect in those files, not a second supported
> spelling. Always use `SC_BOOTSTRAP_NODES`.

### [Current] Mobile

Android and iOS discover peers on the local network with no configuration. For
internet reachability, the Join Mesh flow ingests a join bundle by QR scan
(Android: `android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt`,
"Scan QR Code"). The bundle carries the seed peer addresses, so the address is
still user-supplied -- it is just transported as a QR code rather than typed.

### [Current] Private Networks: Build-Time Seeding

For a closed deployment you can compile a seed list in, via the same variable
read through `option_env!` at build time (`cli/src/bootstrap.rs`):

```bash
export SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>"
cargo build --release

docker build \
  --build-arg SC_BOOTSTRAP_NODES="/ip4/<NODE_IP>/tcp/9001/p2p/<PEER_ID>" \
  -t my-private-build \
  -f docker/Dockerfile .
```

This is for private networks, test infrastructure, and regional deployments you
control. It is not a mechanism for adding addresses to public builds.

## [Current] Running a Reachable Node

Any node with a stable public address helps others cold-start -- not because it
holds a special role, but because it is easy to reach. Requirements:

1. Stable public IP or DNS name
2. Inbound TCP/UDP open on the P2P port (9001 by default; 9000 for the
   WebSocket/API interface)
3. Persistent data directory, so the PeerId stays stable across restarts
4. Reasonable uptime

Read the node's own identity and addresses:

```bash
# Docker
docker exec scmessenger scm identity

# Native
scmessenger-cli identity
```

Full operational guidance -- systemd unit, cloud firewall rules, health checks,
monitoring -- is in `docs/RELAY_OPERATOR_GUIDE.md`.

Sensible topology for a deployment you run: a couple of geographically separate
reachable nodes so a single outage does not isolate new joiners, across more than
one hosting provider. This is redundancy advice for *your* infrastructure. It is
not a project-wide bootstrap tier, and there is no list to enroll in.

## [Current] What a Relaying Node Can and Cannot See

Every node relays, so this applies to every node, not to a special class:

- **Cannot** read message contents -- everything is end-to-end encrypted.
- **Cannot** impersonate a peer -- identities are cryptographic.
- **Can** observe transport metadata: which PeerIds connected, message sizes,
  timing.
- **Can** misbehave -- refuse circuits, or gossip junk peer records over ledger
  exchange. Mitigation is structural: multiple independent paths, reputation
  tracking on relay performance, and no node being load-bearing for entry.
- **Publicly reachable nodes attract DDoS.** Mitigate with rate limits,
  connection caps, and the relay budget cap (`max_relay_budget` in settings,
  applied via `set_relay_budget`).

## [Current] Verifying Peer Discovery

```bash
# What seed addresses does this install have?
scmessenger-cli config get bootstrap_nodes

# Full config dump
scmessenger-cli config list

# Watch discovery with verbose logging
RUST_LOG=debug scmessenger-cli start

# Peer count and connection state
scmessenger-cli status
```

The persisted ledger is the real evidence that discovery is working. Check that
`peers.json` (CLI) or `ledger.json` (core/mobile) is growing in the data
directory across sessions.

## [Current] Troubleshooting

### Peer count stays at 0 on a LAN

On desktop, libp2p mDNS should just work; it degrades gracefully to disabled in
containers and cloud VMs without multicast. On Android the libp2p mDNS behaviour
is compiled out and platform `NsdManager` discovery is used instead. Check:

```bash
# Are we listening at all?
scmessenger-cli status
docker logs scmessenger | grep "Listening on"
```

Then confirm the two hosts are on the same L2 segment and that mDNS/UDP 5353 is
not blocked by a client-isolation setting on the access point. Guest Wi-Fi
networks commonly block peer-to-peer traffic entirely.

### Peer count stays at 0 with no local peers and an empty ledger

Expected. This is the cold-start case -- supply one seed peer address (see
above). There is no shipped default to fall back on, so a node in this state
stays at 0 until a user provides an address or a local peer appears.

### A supplied seed address does not connect

```bash
# Reachability
nc -zv <NODE_IP> 9001

# Format check -- must be /ip4/<IP>/tcp/<PORT>/p2p/<PEER_ID>
scmessenger-cli config get bootstrap_nodes

# Firewall
# Linux: sudo ufw status
# macOS: /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
```

Causes, in order of likelihood: the address is stale (the operator's IP or
PeerId changed), the port is not open inbound on the remote host, or the
multiaddr is malformed.

### Our own node is unreachable from outside

```bash
# From a different machine
nc -zv <YOUR_PUBLIC_IP> 9001
```

Open inbound on both ports. GCP example:

```bash
gcloud compute firewall-rules create allow-scmessenger \
  --allow tcp:9000,tcp:9001,udp:9001 \
  --direction=INGRESS
```

A node behind strict NAT with no UPnP can still participate: it reaches others
through relay circuits provided by whichever peers it can reach. It just cannot
serve as an entry point for anyone else.

### PeerId changed after a restart

The data directory was not persisted. In Docker, check the volume mount; the
network keypair lives under the data directory and must survive restarts, or
every previously-shared ledger entry pointing at this node goes stale.

---

**Key point:** the mesh has no entry tier. Every node relays; peers propagate by
ledger exchange and local discovery; the single manual step is one user-supplied
address on a cold start with no neighbours.
