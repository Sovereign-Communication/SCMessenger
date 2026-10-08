# Joining the Mesh: Invites, Ledger Gossip and Discovery

Status: Current
Last updated: 2026-10-07

> Scope note: this document describes the decided join model for issue #469
> (`docs/BOOTSTRAP_RENDEZVOUS_DECISION_469.md`, operator decisions 2026-10-06).
> Canonical architecture reference: `docs/TRANSPORT_ARCHITECTURE.md`.
> Governance/trust reference: `docs/BOOTSTRAP_GOVERNANCE.md`. Node model:
> `docs/rules/NODE_MODEL.md`. Operating an always-on node:
> `docs/RELAY_OPERATOR_GUIDE.md`.

> [INFO] Implementation status. The invite machinery (`InviteToken`,
> `SeedLedgerEntry`, `import_seed_entries`) exists in core. FFI/CLI/mobile
> wiring (T1-T3), removal of static seed intake (T4, #485), the discovery
> scheduler (T5-T8) and infra changes (T10) are tracked in the decision record.
> Until T4 lands the legacy static-seed mechanisms still exist in code; they are
> documented only in the superseded section at the end of this file and must not
> be used or relied on.

## The Model in One Paragraph

There are only **nodes**. Every node is a full relay and is functionally
identical for transport; the only distinction is whether an identity is loaded.
Nothing is static: no shipped addresses, no seed list, no environment variable,
no config key, no node class. A node that is always on (for example an AWS host)
is an ordinary node that earns a good reputation through uptime and recency and
therefore tends to become predominant for store-and-forward -- by behaviour,
never by flag. Say "always-on node", never "bootstrap node".

## How a Node Joins

1. **Invite (QR or pasted info)** -- the only seed source. Any node can mint a
   `SCI1:` invite; the invitee redeems it (signature verified), which imports a
   small signed seed ledger of bare multiaddrs as unproven entries.
2. **Local discovery** -- mDNS/LAN, BLE, Wi-Fi Aware/Direct (Android), Multipeer
   (iOS). No configuration, no internet, no invite needed on a shared LAN or in
   Bluetooth range.
3. **Ledger gossip** -- once one connection exists, nodes exchange peer records
   over `/sc/ledger-exchange/1.0.0`. An exchanged pair reaches the DHT only when
   `ledger_verified_pair` holds (address locally dialed and bound to that peer
   id); wire data is never promoted without a successful local dial.
4. **Routing planning** -- discovery after first contact feeds the mycorrhizal
   routing layers (`core/src/routing/`, `docs/NATURE_INSPIRED_MESH_PHILOSOPHY.md`);
   there is no external directory.

A stock build starts with an empty ledger and logs
`[DISCOVERY] cold: awaiting invite or LAN/BLE` (T4). The UI never shows a
"no network peers" message; the existing node indicators are the only
connectivity presentation and show an active/probing state when nothing is
connected.

## Reputation and Predominance

Which known node is preferred is decided by observed behaviour only:

- `LedgerManager::get_preferred_relays` -- failure count ascending, then
  `last_seen` descending.
- `core/src/transport/reputation.rs` -- 0-100 score with decay.
- `core/src/transport/relay_health.rs` `priority_score` -- uptime 0.4,
  latency 0.3, stability 0.3. The `headless_bonus` node-class term is scheduled
  for removal (T11); scoring must use behaviour only.

## Event-Driven Discovery (no give-up, no fixed ceiling)

Retry is dynamic and event-driven (decision record section 6). One scheduler
per transport class (`ble`, `lan`, `ledger_dial`):

- Any affecting event (BLE on/off, Wi-Fi/cellular/LAN change, app foreground,
  invite redeemed, ledger received, peer lost) resets that transport to an
  aggressive floor interval and fires an immediate attempt.
- Without events the interval decays by a growth factor in [1.5, 2.0] with full
  jitter, reset whenever a new peer is added.
- There is no give-up state and no literal ceiling: the ceiling is computed from
  observed peer density and power state, so an isolated, charging node keeps
  probing at a short steady interval instead of going quiet.
- Log contract: `[DISCOVERY] event=<kind> transport=<ble|lan|ledger>
  phase=<aggressive|decay> interval_ms=<n> attempt=<n> peers=<n>`.
- Per-candidate dial eligibility remains with `DialPolicyManager`; the
  scheduler decides when to sweep.

The former fixed `5/15/45/120s` ladder (`cli/src/seed_dial.rs`) is superseded by
this scheduler (T5, T6).

## Invites Carry a Seed Ledger -- Routing Only, No Identity

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

## Running an Always-On Node

Any node with a stable public address is easy to reach and, through uptime,
tends to earn reputation. It holds no special role and appears in invites like
any other node. Requirements:

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

Redundancy across a few geographically separate always-on nodes is advice for
your own deployment, not a project-wide tier; there is no list to enroll in.

## What a Relaying Node Can and Cannot See

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

## Verifying Peer Discovery

```bash
# Watch discovery with verbose logging
RUST_LOG=debug scmessenger-cli start

# Connection state
scmessenger-cli status
```

The persisted ledger is the real evidence: `peers.json` (CLI) or `ledger.json`
(core/mobile) should grow across sessions, and ledger exchange log lines
(`Ledger exchange response from <peer>: they learned <x> new peers`) should
appear after the first connection. Score on receiver-side persisted entries,
not on transport acknowledgements.

## Troubleshooting

- **Nothing connects on a LAN.** Desktop mDNS degrades to disabled in containers
  and cloud VMs without multicast; on Android platform `NsdManager` discovery is
  used. Confirm both hosts share an L2 segment and client isolation on the
  access point is off (guest Wi-Fi commonly blocks peer-to-peer traffic).
- **Isolated node with an empty ledger.** Expected: redeem an invite or come
  into LAN/BLE range. The scheduler keeps probing; there is no give-up.
- **A redeemed invite does not connect.** The addresses may be stale, the port
  closed inbound, or the invite expired. Mint a fresh one.
- **Our own node is unreachable from outside.** Open inbound on the P2P port
  (9001) and API port (9000). A node behind strict NAT still participates
  through relay circuits provided by peers it can reach; it just cannot accept
  inbound dials.
- **PeerId changed after restart.** The data directory was not persisted; the
  network keypair lives there and must survive restarts.

---

**Key point:** the mesh has no entry tier and nothing static. Every node relays;
the only seed is an invite; peers propagate by ledger gossip and local
discovery; reputation decides predominance.

---

# Superseded Content (historical record)

[SUPERSEDED 2026-10-06] Everything below describes the pre-decision static-seed
model. It is retained as a record of earlier decisions and is not guidance.
Superseded by `docs/BOOTSTRAP_RENDEZVOUS_DECISION_469.md`; the mechanisms are
removed by T4 (#485) and T10. `SC_BOOTSTRAP_NODES`, `config set
bootstrap_node_add`, the mobile JSON join bundle, build-time seeding and
user-supplied cold-start addresses must not be used.

## [Superseded] There Are No Shipped Default Addresses

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

## [Superseded] Cold Start: The Only Case That Needs Manual Input

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

## [Superseded] Supplying a Seed Peer Address

Address format is a libp2p multiaddr:

```
/ip4/<NODE_IP>/tcp/<P2P_PORT>/p2p/<PEER_ID>
```

`<NODE_IP>`, `<P2P_PORT>` and `<PEER_ID>` come from the node you are joining.
Its operator can read them off that node with `scm identity` and the node's own
"Listening on" log lines. Never copy an address out of documentation -- addresses
are deployment-specific and there are no project-operated ones to copy.

### [Superseded] CLI

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

### [Superseded] Environment Variable

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

### [Superseded] Mobile

Android and iOS discover peers on the local network with no configuration. For
internet reachability, the Join Mesh flow ingests a join bundle by QR scan
(Android: `android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt`,
"Scan QR Code"). The bundle carries the seed peer addresses, so the address is
still user-supplied -- it is just transported as a QR code rather than typed.

### [Superseded] Private Networks: Build-Time Seeding

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
