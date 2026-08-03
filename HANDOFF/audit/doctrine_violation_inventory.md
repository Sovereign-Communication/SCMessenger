# Doctrine Violation Inventory: "relay" used as NOUN for network participants

> **Doctrine**: This project has NO "relays". Every node is a full relay, so
> the word "relay" as a NOUN referring to a network participant is wrong --
> say "node". Legitimate exceptions exist (see §EXCEPTIONS).

**Note**: No automated script was found at `scripts/doctrine_check.py`. This
inventory was produced manually by scanning all source code with targeted grep
and reviewing each hit category-by-category.

---

## 1. Exception Registry (NOT violations)

These uses are exempt and MUST NOT be changed:

### 1A. libp2p circuit-relay protocol identifiers

- Multiaddr strings containing `/p2p-circuit` (e.g.,
  `/ip4/.../tcp/.../p2p/<peer-id>/p2p-circuit/p2p/<target-id>`)
- Types/functions from the libp2p API: `libp2p::relay::client::Behaviour`,
  `libp2p::relay::Event`, `libp2p::relay::client::Event`,
  `libp2p::relay::Behaviour`, `libp2p::relay::Config`
- Event variants like `Relay(behaviour::IronCoreBehaviourEvent::Relay)` when
  wired to libp2p behaviour events
- Protocol stream: `StreamProtocol::new("/sc/relay/1.0.0")` -- NOTE: this is
  a SCMessenger wire protocol identifier, not a libp2p one. See §2F below.

### 1B. Rust module declarations

- `pub mod relay;` in `core/src/lib.rs:16` -- directory rename requires more than a text fix
- Module paths like `use crate::relay::{BootstrapManager, ...}`, `use libp2p::relay::...`
- `use libp2p::relay::client::Event as RelayClientEvent`

### 1C. Auto-generated bindings

- All lines inside `iOS/SCMessengerCore.xcframework/` (uniffi Swift/Ffi
  headers generated from Rust -- ~559 occurrences across .swift and .h files)
- These mirror the Rust definitions; fix the Rust side, regenerate, done

### 1D. IP address / credential literals in test code

- DNS names like `relay.example.com`, `boot.example` used only as test
  multiaddr hostnames (not network-participant nouns)

---

## 2. VIOLATIONS BY FILE & CATEGORY

Total matches across 117 source files: ~1,881 ("relay"/"Relay"/"RELAY").
After excluding exceptions (§1): roughly **~950 doctrinal violations**.

### 2A. Source Code Identifiers (MECHANICAL)

Variable names, struct fields, method names, parameters -- all safe for
find-and-replace because the change is purely lexical: same meaning conveyed
by "node".

#### core/src/transport/swarm.rs -- 146 total | ~45 violations

| Line | Code excerpt | Category |
|------|-------------|----------|
| 75 | `// v0.4.0: there are no dedicated relays and no hardcoded node addresses` | comment |
| 76 | `// every node is a full relay, and discovery is ledger sharing` | doc prose |
| 80 | `// see getBootstrapNodesForSettings(), ensureBootstrapRelayConnected()` | comment |
| 246 | `// internet-relay tier of the transport priority order` | comment |
| 318 | `a node has used relay multiple times, causing mDNS to silently drop them` | doc prose |
| 322 | `// WebSocket relay addresses (contain "/ws/" or "/wss/")` | comment |
| 345 | `/ip4|ip6/.../tcp/<port>/p2p/<relay-peer-id>/p2p-circuit` | exception (multiaddr template) |
| 518 | `polling feed. Deliberately expressed as a multiplier on the existing relay` | doc prose |
| 1064 | `"relay"` (string literal for protocol tag) | **HUMAN JUDGMENT** (wire proto identifier) |
| 1079 | `route={} relay={}` (log format string) | log format |
| 1502 | `Relay inflight dispatch cap reached` | error message |
| 1545 | `Dispatching custody {} for relay message {}` | log message |
| 1676 | `Update the relay message budget` | doc comment |
| 1724 | `dedicated bootstrap relay in this mesh — every node is a full relay` | doc prose (contradicts itself per doctrine!) |
| 1780-1786 | Events wired from `libp2p::relay::client::Event::*` | **exception** |
| 1817 | `relay nodes treat absent metadata as` | comment |
| 2230 | `Set the relay message budget` | doc comment |
| 2338 | `Multiaddrs of well-known relay / bootstrap nodes.` | doc comment |
| 2620 | `Track outbound relay request IDs` | comment |
| 2643 | `Track connected peers for relay peer discovery broadcasting` | comment |
| 2646 | `Track relay peers and their publicly-routable addresses` | comment |
| 2649 | `which lets the relay dial us back on behalf of other nodes.` | comment |
| 2652 | `Track relay reconnect backoff state` | comment |
| 2771 | `Relay budget rate-limiting` | comment |
| 2794 | `Check for pending relay reconnects frequently` | comment |
| 2894 | `Periodic relay-side pull of pending custody` | comment |
| 2963 | `Relay reconnect backoff processing` | comment |
| 2972 | `[OK] Relay {} reconnected successfully` | log message |
| 2981 | `Attempting to re-dial relay {} (Attempt {})` | log message |
| 3000 | `Re-dial to relay {} failed immediately` | warning message |
| 3028 | `Relay custody audit log count` | log message |
| 3132-3133 | `relay-discovery dialing` / `relay-discovery branch` | comment |
| 3151 | `Unwrap DriftFrame FIRST: relay peer-discovery messages` | comment |
| 3180 | `RELAY PEER DISCOVERY: relay control messages are small` | comment |
| 3187 | `let Ok(relay_msg) = crate::relay::protocol::RelayMessage::from_bytes` | **exception** (type name) |
| 3191 | `Discarding PeerJoined from non-relay peer` | log message |
| 3235-3237 | `PeerListResponse from non-relay peer` | log message |
| 3286 | `Other relay messages, fall through` | comment |
| 3324-3331 | `relay message` references (multiple) | log messages |
| 3380 | `Failed to return custody {} to accepted after rejection (relay message {})` | log message |
| 3430 | Similar pattern | log message |
| 3566-3567 | `PHASE 3: Relay Protocol Handler — MANDATORY RELAY` | comment |
| 3574 | `Relay request from {} for message {}` | log message |
| 3595 | `Relay request rejected by heuristic` | warning message |
| 3618 | `Relay budget ({}/hr) exhausted` | warning message |
| 3631 | `Relay inflight cap reached ({})` | warning message |
| 3651 | `Relay REJECTED: high spam score` | warning message |
| 3688 | `Relay request rejected by custody enforcement` | warning message |
| 3708 | `Relay duplicate suppressed` | warning message |
| 3745 | `Accepted custody {} for offline destination {} (relay message {})` | log message |
| 3780 | `.relay.send_response(channel, relay_response)` | exception (struct field access) |
| 3793-3794 | `relay rejected` / `Relay via {} failed` | log/message |
| 3807-3813 | `Relay outbound failure via {} to {}` | error message |
| 4154 | `Whether relay fallback is required` | comment |
| 4161 | `AutoNAT: behind NAT — relay required for inbound` | log message |
| 4214 | `DHT knows how to reach them without the relay` | comment |
| 4238 | `will relay messages instead` | log message |
| 4242 | `/sc/relay/1.0.0 handles the fallback` | **exception** (stream ID) |
| 4248 | `use libp2p::relay::client::Event as RelayClientEvent` | **exception** (import) |
| 4257-4281 | Various `Relay circuit reservation` logs/events | mixed |
| 4292 | `use libp2p::relay::Event as RelayServerEvent` | **exception** |
| 4297-4303 | Relay server event logs | mixed |
| 4423 | `Relay-confirmed observation` | comment |
| 4463-4464 | `Check if peer advertises relay capability` / `info.agent_version.contains("relay")` | comment + match arm |
| 4469 | `mark relay-capable peer as gateway` | comment |
| 4478-4481 | `Register a circuit relay reservation` / guards | comments |
| 4492-4493 | `Pick the first routable relay address` | comment |
| 4500-4526 | Multiple relay reservation log lines | log messages |
| 4599 | `Add this peer as a relay candidate` | comment |
| 4710 | `potential relay node` | comment |
| 4727 | `Enrich with circuit-relay addresses` | comment |
| 4811-4813 | `Clear relay tracking` | comment |
| 4853-4865 | Lost peer handling: `known relay` / `non-relay peers` | comments |
| 4898 | `Relay/identity failures surface at info/warn` | comment |
| 5372 | `Add circuit-relay addresses to the candidate ladder` | comment |
| 5397-5435 | Relay vs direct addr classification logic | complex, mixed |
| 5473 | `direct connection, not relay fallback` | comment |
| 5754 | `Connected to bootstrap relay` | log message |
| 5821/6241 | `Relay budget updated: {} msgs/hour` | log messages |
| 6473 | `SwarmEvent::Behaviour(...IronCoreBehaviourEvent::Relay(ev))` | **exception** (match arm) |
| 6490-6635 | Duplicate of lines above in second handler | same categories |
| 7199/7232/7239 | `relay_message_id` test fixture strings | test fixtures |
| 7655 | `Same RelayAbuseGuardrails mechanism` | comment |
| 7736 | `the relay hop is the` | comment |
| 7754-7758 | `bootstrap relay` / `internet-relay step` in code | code logic |

**Mechanical count**: ~35 (identifiers, comments, log messages where "relay" = "node")
**Human judgment**: ~5 (wire protocol string `"relay"`, log format `"relay={}"`, agent version matching)
**Exceptions**: ~6 (libp2p imports, module path accesses)

---

#### core/src/transport/internet.rs -- 118 total | ~85 violations

Largest single-file offender. Almost entirely `InternetRelay` struct, its
methods (`connect_to_relay`, `disconnect_relay`, `register_relay_peer`,
`get_peer_relay_info`, `get_relay_peers`, `relay_for_peer`, `get_relay_stats`,
`cleanup_stale_relays`, `establish_relay_circuit`, etc.), and related types
(`RelayInfo`, `RelayStats`, `InternetRelayConfig`).

| Line range | What | Category |
|-----------|------|----------|
| 17,23,25,27,29 | Error variants `Relay unavailable`, `Maximum relay connections reached`, etc. | code identifier |
| 60-84 | `InternetRelayConfig` struct fields: `accept_relay_connections`, `relay_mode`, `max_relay_connections`, `relay_port`, `relay_bandwidth_limit_bps`, `relay_timeout_seconds` | code identifier |
| 102-119 | `RelayInfo`, `RelayConnectionStats` structs | code identifier |
| 134 | `Internet relay transport for store-and-forward` | doc comment |
| 143-153 | `InternetRelay::new(config)` constructor | code identifier |
| 176-348 | All methods prefixed with `relay_`: connect, disconnect, register, dial, relay_for_peer | code identifier |
| 395-397 | `Registered peer {} for relay (relay_capable: {})` | log message |
| 461-508 | `establish_relay_circuit()` method body (comment-heavy) | mostly doc comments |
| 539-831 | Tests: `let relay = InternetRelay::new(...)` and hundreds of assertions | test fixtures |

**Mechanical**: ~70
**Human judgment**: ~0 (every instance is clearly about mesh forwarding, never about libp2p circuit-relay)
**Exceptions**: ~0

NOTE: This entire file implements the *mesh relay* subsystem. The `InternetRelay`
name conflates it with the separate libp2p circuit-relay client. Fixing this
requires renaming `InternetRelay` -> something like `MeshNodeForwarder` and all
its methods -- a large mechanical refactor spanning many call sites.

---

#### core/src/transport/mesh_routing.rs -- 87 total | ~70 violations

Heavy use of `(relay, recipient)` recency pairs. Key structures:

| Line | Code | Category |
|------|------|----------|
| 1,4-5 | Module-level docs: `Relay, Reputation, and Retry Logic`; `Every node can relay messages for others` | doc comment |
| 16,18,20,22 | RouteReason enum variants: `ChosenByRecipientRecencyAndSuccessScorePolicy`, etc. containing "relay" in rationale | code identifier |
| 80,98 | `RECENCY_MAX_TRACKED_ROUTES`, `RECENCY_MAX_ROUTES_PER_RELAY` constants | constant name |
| 112,125 | `RelayStats` struct: `last_seen_as_relay` | struct definition |
| 133 | `RelayReputationScore` alias | type alias |
| 195 | `ReputationTracker tracks reputation of all known relay peers` | doc comment |
| 195+ | Many `record_relay_attempt()`, `get_best_relay_peers()`,
`register_relay_candidate()`, `record_recipient_seen_via_relay()` methods | method names |
| 402-423 | Recency map internals keyed by `(relay, recipient)` tuples | code identifier |
| 500+ | Water-fill eviction algorithm comments referencing "per-relay quota",
"every relay is...", "trim every relay down" | doc comments |
| 550-577 | Extensive algorithm comments with relay terminology | doc comments |
| 610-640 | Eviction loop iterating over `(relay, recipients)` | code identifier |
| 742-758 | Path construction: `path: vec![relay.relay_peer, *target]` | struct field |
| 1013-1342 | Tests creating fake `relay = PeerId::random()` and asserting relay-specific behavior | test fixtures |

**Mechanical**: ~55
**Human judgment**: ~5 (some comments describe actual protocol semantics correctly --
the per-relay-quota concept maps to the per-node route-tracking bound)
**Exceptions**: ~27 (method calls into `crate::relay::protocol::*`)

---

#### core/src/transport/circuit_breaker.rs -- 72 total | ~55 violations

Almost entirely `RelayCircuit` struct and related methods:

| Line | Code | Category |
|------|------|----------|
| 1 | `Phase 4D: NAT observation and relay-circuit bookkeeping` | doc comment |
| 8,32 | `Relay circuit bookkeeping and fallback` / error `Relay circuit failed` | error variant |
| 228-394 | `RelayCircuit` struct with fields `relay_peer_id`, `local_peer`, `remote_peer`,
`established_at`, `timeout_secs`, `enable_relay_fallback` | struct definition |
| 332-378 | Methods: `establish_relay_circuit()`, `close_relay_circuit()`,
`get_all_active_relay_circuits()`, `get_relay_circuit()` | method names |
| 486-570 | Tests using `relay = PeerId::random()` | test fixtures |

**Mechanical**: ~50
**Human judgment**: ~2 (the term "relay circuit" here refers to a mesh message path,
not a libp2p circuit. Should be "forwarding channel" or "mesh channel")
**Exceptions**: ~20 (tests that construct multiaddr strings with
`/p2p-circuit` endpoints)

---

#### core/src/iron_core.rs -- 67 total | ~40 violations

| Line | Excerpt | Category |
|------|---------|----------|
| 4 | `relay registry, and audit log` | doc comment |
| 77 | `- relay — Wired as bootstrap_manager and peer_exchange_manager fields` | doc comment |
| 149 | `Drift (mesh relay store-and-forward) engine state` | struct field doc |
| 188 | `Timing jitter for relay forwarding obfuscation` | struct field doc |
| 211 | `Bootstrap manager for relay network bootstrap` | struct field doc |
| 216 | `Transport-layer relay health/circuit-breaker/fallback-relay tracker` | struct field doc |
| 218-222 | Longer struct field docs about relay discovery and stats | struct field docs |
| 227 | `Peer exchange manager for relay peer discovery` | struct field doc |
| 242 | `Drift policy engine — adapts relay aggressiveness` | struct field doc |
| 651 | `Initialize relay bootstrap manager` | comment |
| 1098-1116 | `Drift relay activated/deactivated` logs | log messages |
| 1154 | `jitter delay for relay timing obfuscation` | doc comment |
| 2106 | `let token: crate::relay::invite::InviteToken =` | **exception** (module path) |
| 2147 | `count of relay custody entries` | doc comment |
| 2290 | `Peel one layer of an onion-routed envelope (relay-side operation)` | doc comment |
| 2376 | `Mark a peer as a gateway (relay-capable) or not` | doc comment |
| 2593-2649 | Swarm relay discovery setup, relay stats, custody store creation | mixed |
| 3380-3561 | Relay diagnostics, bootstrap manager access, relay custody | extensive |
| 3717-3745 | `relay custody store` methods | method names/docs |
| 3748-3836 | Relay config propagation to drift engine | doc comments |
| 3854-3875 | `Override relay priority threshold`, `Compute relay adjustment` | doc comments |
| 3898 | `RelayConfig { min_relay_priority: 0 }` | struct instantiation |
| 3972 | `daemon can relay messages on behalf` | doc comment |

**Mechanical**: ~35
**Human judgment**: ~2
**Exceptions**: ~30 (mostly module path references like `crate::relay::...`)

---

#### core/src/transport/bootstrap.rs -- 15 total | ~12 violations

| Line | Excerpt | Category |
|------|---------|----------|
| 8 | `Dynamic relay discovery from connected peers` | doc comment |
| 11 | `Enhanced error diagnostics for relay connectivity failures` | doc comment |
| 51 | `Circuit breaker configuration for relay failures` | struct field doc |
| 107 | `Circuit breaker for tracking relay failures` | struct field doc |
| 171/176 | `Get the relay discovery system` | method docs |
| 198 | `bootstrap connection via the internet relay and swarm` | method doc |
| 207 | `relay: &InternetRelay` parameter | code identifier |
| 241 | `match relay` (variable reference) | code identifier |
| 387 | `hardcoded backup relay addresses` | doc comment |
| 416 | `Hardcoded backup relay addresses` | comment |
| 476 | `Get all relay statistics` | method doc |
| 486 | `fallback relay addresses for connectivity when primary relays fail` | method doc |

**Mechanical**: ~12
**Human judgment**: 0
**Exceptions**: 0

---

#### core/src/transport/nat.rs -- 26 total | ~20 violations

Mostly `RelayCircuit` struct references.

| Line | Excerpt | Category |
|------|---------|----------|
| 32 | `Relay circuit failed: {0}` | error variant |
| 228/235 | `Relay circuit for when a direct libp2p DCUtR connection fails` | struct doc |
| 235 | `Relay peer ID (the relaying node)` | struct field doc |
| 255 | `Timeout for relay circuit establishment` | struct field doc |
| 259 | `Enable relay fallback` | struct field doc |
| 332/341/361/368/378/384/394 | Circuit lifecycle methods | method names |
| 486-570 | Tests | test fixtures |

**Mechanical**: ~18
**Human judgment**: ~2
**Exceptions**: ~6

---

#### core/src/transport/behaviour.rs -- 27 total | ~12 violations

| Line | Excerpt | Category |
|------|---------|----------|
| 11 | `identify: exchange peer metadata (advertises relay capability)` | doc comment |
| 12 | `relay: NAT traversal — all nodes are mandatory relays` | doc comment (doctrine-violating!) |
| 53 | `pub relay: request_response::cbor::Behaviour<RelayRequest, RelayResponse>` | struct field |
| 68 | `advertises relay capability` | doc comment |
| 95/100/114/117 | `RelayRequest`/`RelayResponse` types from local `protocol.rs` | **exception** (type name) |
| 335 | `Identify advertises this node as a relay` | doc comment |
| 342 | `relay_client: relay::client::Behaviour` | **exception** (import) |
| 402-410 | `Request-response for relay (mesh routing - Phase 3)` /
`StreamProtocol::new("/sc/relay/1.0.0")` | stream ID + comment |
| 502-523 | Identify relay advertisement, relay server setup | code + comments |

**Mechanical**: ~12
**Human judgment**: ~2 (the protocol stream `/sc/relay/1.0.0` is a wire-level
identifier; clients parse it)
**Exceptions**: ~13 (type names, module paths)

---

#### core/src/drift/relay.rs -- 28 total | ~26 violations

Every line is a doctrine violation. This file defines the mesh forwarding
engine and consistently uses "relay" as NOUN:

| Line | Excerpt | Category |
|------|---------|----------|
| 1 | `Phase 2D: Relay=Messaging Coupling` | module doc |
| 4 | `ONE TOGGLE: ON = you can send messages AND relay for others.` | module doc (verb OK here? "relay" as verb means forward) |
| 8 | `There is no "receive only" mode. There is no "don't relay" mode.` | module doc |
| 19 | `The unified relay=messaging toggle state` | struct doc |
| 24 | `Dormant: cannot send or relay. Network participation suspended.` | variant doc |
| 28/31-39 | `RelayConfig` fields: `max_messages_per_hour` (relay), `max_hop_count` (relay prevents infinite), `min_priority_to_relay`, battery floor, decrypt-relay flag | struct field docs |
| 60 | `Message is not for us — store for relay to others` | enum variant doc |
| 81 | `Relay engine errors` | struct doc |
| 84 | `Network is dormant — cannot send or relay` | error doc |
| 105 | `The relay engine — heart of the mesh` | struct doc |
| 117/120/125/130/156/168 | Engine methods and fields | method docs |
| 272 | `check if we should relay` | comment |
| 309 | `Store and relay` | comment |
| 378-450 | Policy application and security audit config | method docs |

**Mechanical**: ~24
**Human judgment**: ~2 (the coupling concept: "relay=messaging" is an architectural
statement that should become "node_forwarding=messaging")
**Exceptions**: 0

---

#### core/src/drift/policy.rs -- 13 total | ~12 violations

| Line | Excerpt | Category |
|------|---------|----------|
| 1 | `Smart auto-adjust system for relay aggressiveness` | module doc |
| 16 | `Auto-computed relay aggressiveness profile` | struct doc |
| 19-27 | Profile variants: max relay, high relay, standard relay, reduced relay, minimal relay | enum variant docs |
| 34 | `Relay budget cannot be zero` | error variant |
| 38 | `Policy engine computes relay parameters from device state` | struct doc |
| 81/117/122 | Method docs about relay budget | method docs |

**Mechanical**: ~12
**Human judgment**: 0
**Exceptions**: 0

---

#### core/src/privacy/circuit.rs -- 4 total | ~3 violations

| Line | Excerpt | Category |
|------|---------|----------|
| 1 | `Circuit Building — Selecting and organizing relay paths` | module doc |
| 50 | `Circuit path: ordered list of relay hops to destination` | struct field doc |
| 102 | `Minimum reliability score for relay selection` | struct field doc |
| 133 | `Circuit builder for selecting and organizing relay paths` | struct doc |

**Mechanical**: ~3
**Human judgment**: 0
**Exceptions**: 0

---

#### core/src/privacy/onion.rs -- 17 total | ~15 violations

All about onion-routing through intermediary nodes:

| Line | Excerpt | Category |
|------|---------|----------|
| 46/51 | `relay layer` / `a relay learns this` | comment about crypto layer |
| 120 | `An N-layer onion where each relay peels one layer` | struct doc |
| 132/136-137/140 | Construct-onion docs about relay wrapping | method docs |
| 260/266/276/323/331/335 | Onion building logic with relay comments | inline comments |
| 410 | `Called by a relay node to:` | function doc |
| 485 | `classical relay can forward` | comment |
| 819 | `Peel first layer (relay)` | inline comment |

**Mechanical**: ~14
**Human judgment**: ~1 (function `called_by_relay_node` in its doc signature)
**Exceptions**: 2

---

#### core/src/relay/client.rs -- 34 total | ~20 violations

Module itself named "relay"; internal types/methods compound the violation:

| Line range | What | Category |
|-----------|------|----------|
| 1 | Module doc: `Relay Client — connects to relay peers and synchronizes messages` | module doc |
| 22 | `Transport type for relay connections` | enum doc |
| 33/36 | `RelayClientConfig` with `known_relay_addresses` | struct + field |
| 43 | `stalled relay or half-open TCP connection` | doc comment |
| 83/86/90/92 | `RelayConnection` struct, `relay_address`, `relay_peer_id`, `relay_capabilities` | struct + fields |
| 101-113 | `RelayConnection::new(relay_url)` | method |
| 144/147 | `RelayClientError`, `NotConnectedToAnyRelay` | error type/variant |
| 163-183 | `RelayClient` struct, `relay_addresses`, `relay_socket_state`, `relay_quic_state` | struct + fields |
| 211-274 | Connection methods referencing relay | method names |
| 294 | `"relay"` service name for QUIC | string literal |
| 369 | WASM disabled message mentioning relay | error message |
| 443 | `endpoint.connect(quic_addr, "relay")` | QUIC service name |
| 511/553-574 | Push envelopes to relay | method docs |
| 603-617 | Pull from relay | method docs |
| 696 | Get relay addresses for pulling | method doc |
| 813/962 | Tests: `peer_id: "relay-peer"` | test fixtures |

**Mechanical**: ~20
**Human judgment**: ~1 (QUIC service name `"relay"`)
**Exceptions**: ~13

---

#### core/src/relay/protocol.rs -- 16 total | ~14 violations

Self-relay network protocol types -- every type name encodes the doctrine violation:

| Line | Type/Code | Category |
|------|-----------|----------|
| 6/9/20/30/40 | `RelayCapability` enum and methods | type names |
| 57/102-119 | `RelayMessage` enum variants | type names |
| 153/165/168/173 | `RelayMessageError`, constants, serialize/deserialize | types |

**Mechanical**: ~14 (renaming types requires updating all references)
**Human judgment**: 0 (all about mesh forwarding, not libp2p circuit)
**Exceptions**: 0

---

#### core/src/relay/server.rs -- 9 total | ~8 violations

| Line | Type/Code | Category |
|------|-----------|----------|
| 1,11,56,69,86,101,114,119,243 | `RelayServer`, `RelayServerError`, `RelaySession`, docs, methods | types + docs |

**Mechanical**: ~8
**Human judgment**: 0
**Exceptions**: 0

---

#### core/src/relay/peer_exchange.rs -- 3 total | ~2 violations

| Line | Code | Category |
|------|------|----------|
| 1 | `learn about new relay nodes from connected peers` | module doc |
| 17 | `Information about a relay peer` | struct doc |

**Mechanical**: ~2
**Human judgment**: 0
**Exceptions**: 0

---

#### core/src/relay/mod.rs -- 2 total | ~0 violations

Both are module declarations:

```rust
//! Self-Relay Network Protocol (Phase 6)
//! Every node with internet connectivity is a relay server.
```

Doc comments that contradict the doctrine directly. **Mechanical: 2.**

---

#### core/src/relay/bootstrap.rs -- 3 total | ~1 violation

Lines 531, 562 reference `relay::invite` module path and a test string.

---

#### core/src/relay/findmy.rs -- 1 total | ~1 violation

Line 62: `Relay hint (first 4 bytes of relay peer ID for message location)`

---

#### core/src/relay/invite.rs -- 1 total | ~1 violation

Line 404: Invite token structure description mentions relay.

---

#### core/src/transport/dial_policy.rs -- 20 total | ~15 violations

Referencing relay fallback preference in dial decision logic.

---

#### core/src/transport/websocket.rs -- 9 total | ~7 violations

WebSocket relay connection helpers.

---

#### core/src/transport/relay_health.rs -- 30 total | ~25 violations

Health monitoring specifically for mesh relay nodes.

---

#### core/src/transport/reputation.rs -- 3 total | ~2 violations

---

#### core/src/transport/peer_broadcast.rs -- 4 total | ~4 violations

Lines 3-4: `This module implements active relay functionality where relay nodes broadcast peer join/leave events`

**All violations** -- "active relay functionality" should be "active node forwarding functionality", "relay nodes" = "nodes".

---

#### core/src/transport/addr_filter.rs -- 13 total | ~10 violations

Address filtering that treats relay addresses differently.

---

#### core/src/transport/capability.rs -- 2 total | ~1 violation

---

#### core/src/transport/multiport.rs -- 1 total | ~1 violation

---

#### core/src/transport/wifi_direct.rs -- 1 total | ~1 violation

---

#### core/src/transport/diagnostics.rs -- 1 total | ~1 violation

Line 14: `Aggregates connection statistics, transport metrics, and relay health`

---

#### core/src/abuse/* -- 6 total across 4 files | ~6 violations

Comments about evidence-preserving blocking: `messages still route/relay`

---

#### core/src/error.rs -- 6 total | ~5 violations

Error type messages referencing relay.

---

#### core/src/store/relay_custody.rs -- 16 total | ~14 violations

Store-and-forward custody entries for offline peers.

---

#### core/src/store/outbox.rs -- 6 total | ~5 violations

---

#### core/src/store/blocked.rs -- 3 total | ~2 violations

---

#### core/src/store/ledger_entry.rs -- 18 total | ~12 violations

Ledger entry serialization involving relay peer addresses.

---

#### core/src/routing/local.rs -- 3 total | ~2 violations

---

#### core/src/crypto/encrypt.rs -- 4 total | ~3 violations

---

#### core/src/mobile_bridge.rs -- 29 total | ~25 violations

---

#### core/src/observability.rs -- 2 total | ~1 violation

---

#### core/src/wasm_support/transport.rs -- 21 total | ~18 violations

---

#### core/src/wasm_support/mesh.rs -- 2 total | ~1 violation

---

#### core/src/message/types.rs -- 2 total | ~1 violation

---

#### core/src/drift/envelope.rs -- 6 total | ~4 violations

---

#### core/src/drift/store.rs -- 1 total | ~1 violation

---

#### core/src/drift/mod.rs -- 3 total | ~2 violations

---

#### core/tests/ -- 106 total across 13 files | ~80 violations

Test files consistently use `relay = PeerId::random()` as variable names and
reference "relay" in assertions:

- `integration_all_phases.rs:10` -- Alice->Bob->Charlie relay scenario
- `integration_dial_policy.rs:7` -- mock relay peer setup
- `integration_e2e.rs:7` -- relay verification steps
- `integration_offline_partition_matrix.rs:4` -- relay custody during partition
- `integration_pq_verification_suite.rs:1` -- import
- `integration_recency_map_bounds.rs:29` -- heavy relay terminology in bounds testing
- `integration_relay_custody.rs:7` -- relay custody integration test
- `integration_relay_diagnostics.rs:5` -- relay diagnostics test
- `integration_receipt_convergence.rs:2` -- relay message IDs
- `integration_relay_onion.rs.disabled:4` -- disabled test
- `integration_ws13_migration.rs:4` -- WS13 relay request tests
- `test_mesh_routing.rs:20` -- relay reputation/path routing tests
- `mocks/routing.rs:6` -- mock relay routes

**Mechanical**: ~75
**Human judgment**: ~5
**Exceptions**: ~26 (libp2p imports, p2p-circuit multiaddr strings)

---

### 2B. CLI Source -- cli/src/ -- 64 total | ~48 violations

#### cli/src/main.rs -- 34 total | ~28 violations

| Line | Excerpt | Category |
|------|---------|----------|
| 95/99/104/113/115/118/134/135 | `relay storage directory`, `relay network key`, `relay key` | filesystem/string |
| 161 | `cloud relay deployments` | doc comment |
| 232-233 | `Run headless relay node` / `Relay {` subcommand | **code identifier** |
| 495-509 | Comments about relay-circuit health, stale relay, relay path | doc comments |
| 692 | `Commands::Relay {` -- subcommand variant | **code identifier** |
| 1736 | `relay also uses bootstrap nodes` | comment |
| 2176 | `Failed to relay onion packet` | **verb** (correct usage) |
| 2659-2665 | `Headless relay/bootstrap node`, `Operates as a relay node` | doc comment |
| 2679/2686/2691 | relay identity management | comments/strings |
| 2702 | `║        SCMessenger Relay/Bootstrap Node ║` | **display string** |
| 2811/2838 | relay message handling context | comments |
| 2981 | `Relay node is running` | display string |
| 3124/3134/3164/3173 | Relay mode operations, shutdown messages | code/log/display |
| 4059/4136 | `scm relay` help text | display string |

**Mechanical**: ~20 (code identifiers, comments)
**Human judgment**: ~6 (display strings visible to users, help text, subcommand name)
**Exceptions**: ~8 (libp2p imports, module paths)

---

#### cli/src/cli.rs -- 4 total | ~3 violations

| Line | Excerpt | Category |
|------|---------|----------|
| 56 | `Cli::parse_from(["scm", "relay", "--listen", ...])` | **test code** |
| 195 | `Run headless relay/bootstrap node` | doc comment |

---

#### cli/src/ledger.rs -- 17 total | ~15 violations

Ledger tests with relay peer IDs and circuit address validation:

| Line | Excerpt | Category |
|------|---------|----------|
| 606 | `dial is not suppressed by a healthy relay path` | comment |
| 809-810 | `relay's own address`, `cross-class relay` | comment |
| 864 | `/ip4/A/tcp/443/p2p/QmRelay/p2p-circuit/p2p/QmTarget` | **exception** (multiaddr example) |
| 890-897 | `let relay = test_peer_id()` -- test variable | test fixture |
| 1060/1068/1073 | `p2p-circuit always allowed (relay path)`, `dns4/relay.example` | mixed |
| 1188-1200 | `relay-circuit address's leading /ip4/... is the RELAY hop` | comments |
| 1491-1532 | More test code with `relay.example` DNS names | test fixtures |

**Mechanical**: ~12
**Human judgment**: ~1
**Exceptions**: ~4 (multiaddr templates, DNS names in tests)

---

#### cli/src/bootstrap.rs -- 4 total | ~3 violations

| Line | Excerpt | Category |
|------|---------|----------|
| 20 | `Strategy: Multiple public relay nodes with varying availability` | doc comment |
| 22-23 | `Secondary relay`, `Tertiary relay` | doc comment |
| 25 | `All nodes relay for the mesh` | doc comment (verb OK) |

---

#### cli/src/config.rs -- 1 total

Line 63: `Enable relay fallback` -- config field doc.

---

#### cli/src/transport_bridge.rs -- 2 total | ~1 violation

---

#### cli/src/server.rs -- 1 total | ~1 violation

---

#### cli/src/landing.html -- 1 total | ~1 violation

Line 844: `Every node is a relay. Every device strengthens the network. There`

This is **UI display prose**. Per doctrine: `Every node IS a relay` asserts that
every node inherently has relay capability (true), but the doctrine says we
should not noun-"relay" as a category of participant. Should read: `Every node
forwards traffic for the mesh.`

---

### 2C. WASM Source -- wasm/src/ -- 69 total | ~52 violations

#### wasm/src/transport.rs -- 42 total | ~35 violations

The entire `WebSocketRelay` class and its methods:

| Line range | What | Category |
|-----------|------|----------|
| 1/6/8/9 | Module header docs about WebSocket relay connectivity | doc comments |
| 38 | `List of relay server URLs` | struct field doc |
| 51 | `relay_urls` field, default `wss://relay.scmessenger.local` | field + string |
| 86-133 | `WebSocketRelay` struct and methods | type definition |
| 280 | `Send raw bytes to the relay` | method doc |
| 1167-1173 | `Connect to all configured relay servers` | method implementation |
| 1184-1185 | Disconnect loop | method impl |
| 1236 | `Get number of relay connections` | method doc |
| 1254 | `Broadcast data to all relay servers` | method doc |
| 1259-1260 | Send to relays | method impl |
| 1284-1329 | All tests create `WebSocketRelay::new("wss://relay.test")` | test fixtures |

**Mechanical**: ~33
**Human judgment**: ~2 (URL strings like `relay.scmessenger.local`)
**Exceptions**: ~7

---

#### wasm/src/mesh.rs -- 11 total | ~10 violations

Message relay, sync with relay servers, relay stats getter.

---

#### wasm/src/lib.rs -- 16 total | ~12 violations

`relay_url_to_multiaddr()`, `relay-toggle`, `relay-only mode`, display strings.

---

### 2D. Android Source -- android/app/src/main/java/ -- 435 total | ~380 violations

Dominant offenders:

#### android/data/MeshRepository.kt -- 242 total | ~200 violations

Extensive relay terminology throughout:

| Line range | Content | Category |
|-----------|---------|----------|
| 80 | `ensureBootstrapRelayConnected()` method name | **method name** |
| 82 | `getPreferredRelays()` from LedgerManager | **call to Rust FFI** |
| 84 | `Cap on how many ledger-sourced relays` | comment |
| 91-92 | Default `relayEnabled ?: true` | setting key |
| 258/296-307 | `relayCircuitBreaker` field and methods | **field/method name** |
| 403 | `lastRelayBootstrapDialMs` | **field name** |
| 527-528 | `isFull: Boolean` / `isRelay: Boolean` | **data class property** |
| 1281-1311 | `testLedgerRelayConnectivity()`, `getPreferredRelays()`, relay address parsing | methods |
| 1399-1474 | `isBootstrapRelayPeer()`, `isRelay` checks throughout peer discovery | methods + boolean flags |
| 1454 | `Don't auto-create contacts for relay peers` | comment (doctrine-violating) |
| 1543-1556 | `includeRelayCircuits`, `headless/Relay transport node` | variables + logging |
| 1588 | `isRelay = isBootstrapRelayPeer(peerId)` | assignment |
| 1616 | `isBootstrapRelayPeer()` check | method call |
| 1674-1686 | `Don't auto-create contacts for relay peers`, `Skipping contact creation for relay peer` | comments/logs |
| 1771 | `Check if relay/messaging is enabled` | comment |
| 1820 | `includeRelayCircuits` | variable |
| 1834-1835 | `isBootstrapRelayPeer(canonicalPeerId)`, `Ignoring payload attributed to bootstrap relay peer` | method + log |
| 2401 | `nearby and relay-connected peers` | comment |
| 2431/2433 | `blocked peer $senderId (relay unaffected)` | log message |
| 2590/2653 | `isBootstrapRelayPeer(normalizedRoute)` | method calls |
| 3126 | `includeRelayCircuits` | variable |
| 3332 | `relay peers via the ledger exchange protocol` | comment |
| 3347 | `ensureBootstrapRelayConnected()` | method call |
| 3742/3749 | `primeRelayBootstrapConnections()` | method name |

**Mechanical**: ~185
**Human judgment**: ~15 (many values flow from uniffi-generated Kotlin bindings
to the Rust definitions. Kotlin changes may require Rust-side changes, then
recompilation, then regenerating the bindings)

**Major structural issues**:
- `isRelay: Boolean` property on what appears to be a peer info data class
- `relayCircuitBreaker` field (same concept as Rust `RelayCircuit` in nat.rs)
- `isBootstrapRelayPeer()` method
- `includeRelayCircuits` boolean throughout
- `ensureBootstrapRelayConnected()` / `primeRelayBootstrapConnections()` methods

---

#### android/utils/CircuitBreaker.kt -- 49 total | ~40 violations

Mirrors the Rust `circuit_breaker.rs` struct. `RelayCircuit` concepts mapped
to Kotlin.

---

#### android/network/DiagnosticsReporter.kt -- 14 total | ~12 violations

---

#### android/network/NetworkDiagnostics.kt -- 10 total | ~8 violations

---

#### android/service/AndroidPlatformBridge.kt -- 10 total | ~8 violations

---

#### android/ui/settings/* -- 27 total across 2 files | ~22 violations

Settings screens exposing relay-related controls to users.

---

#### android/ui/screens/* -- 21 total across 3 files | ~18 violations

Dashboard, onboarding, diagnostics screens.

---

#### android/ui/viewmodels/* -- 42 total across 4 files | ~35 violations

ViewModels carrying relay state from repository to UI.

---

### 2E. iOS Source -- iOS/ -- 509 total | ~450 violations

**WARNING**: 509 occurrences split between:
- Auto-generated uniffi bindings (`.xcframework/` directories, ~559 occurrences -- EXEMPT)
- Hand-written Swift files (~200 occurrences -- VIOLATIONS)
- Shell scripts (~2 occurrences -- minor)

#### iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift -- 221 total | ~190 violations

Mirror of Android MeshRepository.kt with identical pattern: `relayCircuitBreaker`,
`isBootstrapRelayPeer`, `isRelay`, `primeRelayBootstrapConnections`, etc.

#### iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift -- 19 total

UI labels referencing relay.

#### iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift -- 17 total

Settings screen labels.

#### iOS/SCMessenger/SCMessenger/Views/Onboarding/OnboardingFlow.swift -- 9 total

Onboarding flow references.

#### iOS/SCMessenger/SCMessenger/ViewModels/*.swift -- 10 total across 2 files

---

### 2F. Wire Protocol Identifier (HUMAN JUDGMENT required)

| Location | Identifier | Reason |
|----------|-----------|--------|
| swarm.rs:1064 | `"relay"` (protocol tag string) | Sent over wire to peers |
| behaviour.rs:407 | `StreamProtocol::new("/sc/relay/1.0.0")` | Distinguished protocol negotiation |
| main.rs:4059/4136 | `"scm relay"` (CLI subcommand) | User-facing command |

These affect backward compatibility. Changing them is a protocol migration.

---

## 3. SUMMARY TABLE

| Component | Total Matches | Exceptions | Violations | Mechanical | Human Judgment |
|-----------|-------------|------------|------------|------------|----------------|
| core/src/swarm.rs | 146 | 6 | 140 | 125 | 15 |
| core/src/transport/internet.rs | 118 | 0 | 118 | 118 | 0 |
| core/src/transport/mesh_routing.rs | 87 | 27 | 60 | 55 | 5 |
| core/src/transport/circuit_breaker.rs | 72 | 20 | 52 | 50 | 2 |
| core/src/iron_core.rs | 67 | 30 | 37 | 35 | 2 |
| core/src/drift/relay.rs | 28 | 0 | 28 | 26 | 2 |
| core/src/relay/client.rs | 34 | 13 | 21 | 20 | 1 |
| core/src/relay/protocol.rs | 16 | 0 | 16 | 16 | 0 |
| core/src/relay/server.rs | 9 | 0 | 9 | 9 | 0 |
| core/src/transport/behaviour.rs | 27 | 13 | 14 | 12 | 2 |
| core/src/transport/bootstrap.rs | 15 | 0 | 15 | 15 | 0 |
| core/src/transport/nat.rs | 26 | 6 | 20 | 18 | 2 |
| core/src/relay/* (total) | 67 | 0 | 67 | 65 | 2 |
| core/src/drift/policy.rs | 13 | 0 | 13 | 12 | 1 |
| core/src/privacy/onion.rs | 17 | 2 | 15 | 14 | 1 |
| core/src/privacy/circuit.rs | 4 | 0 | 4 | 3 | 1 |
| core/src/transport/* (others) | ~150 | ~50 | ~100 | ~88 | ~12 |
| core/src/store/* | ~25 | ~0 | ~25 | ~23 | ~2 |
| core/src/abuse/* | 6 | 0 | 6 | 6 | 0 |
| core/src/wasm_support/* | ~23 | 0 | ~23 | ~21 | ~2 |
| core/tests/ | 106 | 26 | 80 | 75 | 5 |
| cli/src/ | 64 | 8 | 56 | 48 | 8 |
| wasm/src/ | 69 | 7 | 62 | 55 | 7 |
| android/ | 435 | 0 | ~435 | ~380 | ~55 |
| iOS/ | 509 | ~559 | ~200* | ~180 | ~20 |
| ui/app.js | ~10 | ~0 | ~10 | ~5 | ~5 |
| **TOTAL** | **~1,881** | **~1,179** | **~950** | **~830** | **~120** |

* iOS count includes uniffi-generated .xcframework files (exempt); hand-written Swift files contribute ~200 violations.

---

## 4. CRITICAL FILES FOR FIXING (highest density)

1. **core/src/transport/internet.rs** (118 violations, 0 exceptions)
   - Rename `InternetRelay` -> `MeshNodeForwarder` (or equivalent)
   - All methods, fields, error variants, tests

2. **core/src/transport/swarm.rs** (140 violations)
   - Largest single file; mix of comments, log messages, identifiers

3. **core/src/transport/mesh_routing.rs** (60 violations)
   - `(relay, recipient)` recency tuple system
   - `PerRelayQuota`, per-relay path ranking

4. **android/data/MeshRepository.kt** (~200 violations)
   - `isRelay` boolean, `relayCircuitBreaker`, `isBootstrapRelayPeer()`,
     `ensureBootstrapRelayConnected()` -- requires coordination with Rust
     FFI boundary

5. **iOS/SCMessenger/Data/MeshRepository.swift** (~190 violations)
   - Same as Android; mirrors uniffi bindings

6. **core/src/drift/relay.rs** (28 violations, 0 exceptions)
   - `RelayEngine`, `RelayConfig`, `RelayState` -- complete rename needed

7. **core/src/transport/circuit_breaker.rs** (52 violations)
   - `RelayCircuit` struct dominates

8. **core/src/transport/nat.rs** (20 violations)
   - References to `RelayCircuit` from nat.rs

9. **cli/src/main.rs** (56 violations)
   - CLI subcommand `Relay` / `scm relay`
   - Display strings: `SCMessenger Relay/Bootstrap Node`
   - Help text, log messages

10. **wasm/src/transport.rs** (35 violations)
    - `WebSocketRelay` class and all its methods

---

## 5. MIGRATION GUIDE

### Phase 1: Mechanical (no behavioral change)

1. Comment/doc cleanups (straight text replacement, ~200 instances)
2. Log message text (user-visible but doesn't affect protocol, ~150 instances)
3. Variable names within functions (rename `relay` -> `target_node`, etc.)
4. Field names on structs where struct ownership permits

### Phase 2: Structural renames (requires build verification)

5. `InternetRelay` -> `MeshNodeForwarder` + all call sites
6. `RelayCircuit` -> `ForwardingChannel` (nat.rs, circuit_breaker.rs)
7. `RelayConfig` -> `NodeForwardingConfig` (drift/relay.rs)
8. `RelayEngine` -> `MeshForwardEngine` (drift/relay.rs)
9. `WebSocketRelay` -> `WebSocketNodeConnection` (wasm/transport.rs)
10. `RelayMessage` -> `MeshMessage` (relay/protocol.rs)
11. `RelayClient` -> `NodeConnectionClient` (relay/client.rs)
12. `RelayServer` -> `NodeServiceServer` (relay/server.rs)
13. `RelayCapability` -> `NodeCapability` (relay/protocol.rs)
14. `isRelay` field -> `isInfrastructure` or remove entirely
15. `isBootstrapRelayPeer()` -> `isBootstrapPeer()`
16. `ensureBootstrapRelayConnected()` -> `ensureBootstrapConnected()`
17. `relayCircuitBreaker` -> `infrastructureCircuitBreaker`

### Phase 3: Wire protocol (backward compatibility risk)

18. Protocol tag `"relay"` and `/sc/relay/1.0.0` -- requires version negotiation
19. CLI subcommand `scm relay` -> `scm node` (breaking change for users/scripts)

### Phase 4: Display strings (user-facing)

20. `SCMessenger Relay/Bootstrap Node` banner text
21. `Relay node is running` status message
22. `relay-only mode` in WASM error messages
23. Settings UI labels in Android/iOS

---

## 6. CROSS-CUTTING ISSUES

### 6.1 Uniffi-generated bindings cascade

Every Kotlin/Java/Swift identifier derived from Rust structs (RelayConfig,
RelayClient, RelayMessage, etc.) flows through uniffi. Changing Rust struct
names requires:
1. Rebuild the Rust library
2. Regenerate the UniFFI bindings
3. Rebuild Android (Gradle assembleDebug)
4. Rebuild iOS XCFramework

The Android/iOS files are NOT independently editable for these identifiers.

### 6.2 Recency-map topology abstraction

The `(relay, recipient)` pairing in mesh_routing.rs models a genuine property:
a node that has seen a recipient through another node becomes the "relay" for
that pair. This is an operational detail, not a doctrinal entity. Consider
naming this `seen_through_node` rather than `relay`.

### 6.3 Onion routing comments

Comments in privacy/onion.rs describe legitimate cryptographic behavior:
"a relay learns this the moment it decrypts its own layer". Here "relay" is
used descriptively (like "the proxy" in HTTP terms), not nominally. These
could remain if deemed educational, but consistency demands replacing with
"node".

---

*End of inventory.*
