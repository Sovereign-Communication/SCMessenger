# P1-10 — Adaptive Port Selection: Design Note (mechanism decision)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Task:** P1-10 [OPUS+] from `HANDOFF/V1_0_0_EXECUTION_PLAN.md` Stage C.
**Author:** Claude (native Cowork session), on operator direction (Lucas).
**Date:** 2026-07-04.
**Status:** DESIGN — read-only pass, code-verified this session. Decomposes into P1-11/P1-12/P1-13 (tickets in `HANDOFF/todo/`).
**Verification legend:** [V-READ] = confirmed by reading source this session. No commands run (sandbox has no toolchain).

## 0. Goal restated (settled input 3)

Adaptive port selection is a **deliverability goal, mechanism open**: whatever port lands traffic in a given network is the right port (443, 80, ephemeral, port 0, negotiated — any combination). Move off the hardcoded 9001/9002 defaults. The bar is worst-case (firewall blocks 9001/9002; only 443 or only 80 gets out). Second contact on a hostile network must be fast.

This note refines the plan four-part sketch (listen / advertise / dial / remember) against the **actual** current code and hands three implementation slices to P1-11/12/13.

## 1. Ground-truth corrections to the sketch (what the code actually says)

The sketch was a prior read-only hypothesis. Verified this session; corrections in **bold**:

### 1.1 Listen — CONFIRMED, with one correction
- `core/src/transport/multiport.rs` exists and is complete: `COMMON_PORTS = [443, 80, 8080, 9090]`, `MultiPortConfig` (default enables common ports + random port), `generate_listen_addresses` (appends port 0 last), `requires_elevated_privileges` (Unix <1024), `analyze_bind_results`, `ConnectivityStatus`. All unit-tested. [V-READ]
- `start_swarm_with_config` (`swarm.rs:1837`) already has the multi-port listen branch (`swarm.rs:1876-1901`): if `multiport_config` is `Some`, it iterates `generate_listen_addresses` and `listen_on`s each, records `BindResult`s, and bails only if **zero** binds succeed. The single-port else-branch (`swarm.rs:1902-1910`) defaults to `/ip4/0.0.0.0/tcp/0` (ephemeral) if `listen_addr` is `None`. [V-READ]
- **CORRECTION — the mechanism is present but dormant.** BOTH call sites pass `None` for `multiport_config`: CLI at `cli/src/main.rs:1395`, mobile at `core/src/mobile_bridge.rs:725`. So today every node runs single-port. The CLI *forces* a fixed port: `main.rs:1373` builds `/ip4/0.0.0.0/tcp/{p2p_port}` where `p2p_port = ws_port + 1` and `ws_port` defaults to 9000 (`main.rs:723-728`) — i.e. **9001 is a derived default, not a literal**, but the effect is a hardcode. The `Commands::Relay` clap default is the literal `/ip4/0.0.0.0/tcp/9001` (`cli/src/cli.rs:189`, mirrored `cli/src/main.rs:184`).
- QUIC: `swarm.rs:1928` binds `/ip4/0.0.0.0/udp/0/quic-v1` — already ephemeral, no change. [V-READ]
- **WS listener is bound TWICE and one is a literal hardcode.** (a) `swarm.rs:1938` unconditionally binds `/ip4/0.0.0.0/tcp/9002/ws` inside `start_swarm_with_config` (literal 9002). (b) `main.rs:1406-1408` *additionally* binds `/ip4/0.0.0.0/tcp/{p2p_port+1}/ws` via `swarm_handle.listen`. On the CLI default these collide on 9002. The literal at `swarm.rs:1938` is the one to kill. [V-READ]

### 1.2 Advertise — PARTIALLY CORRECT; two real gaps found
- `relay/peer_exchange.rs` `RelayPeerInfo.addresses: Vec<String>` does carry full multiaddr strings, and the **consumer** side works: `swarm.rs:2450-2483` handles `PeerJoined` / `PeerListResponse`, parses each address as a `Multiaddr`, filters with `is_discoverable_multiaddr`, and dials. [V-READ]
- The **producer** is `PeerBroadcaster` (`transport/peer_broadcast.rs`). At `swarm.rs:3712-3729`, on `ConnectionEstablished`, it seeds `addresses` from the **observed remote address** of the *connecting third peer* (`remote_addr.to_string()`) plus circuit-relay-enriched `external_addresses`, then gossips `PeerJoined` to other peers. **CORRECTION:** this is a peer-relay-gossip mechanism (peer X is reachable at the address I observed / through me), NOT a self-advertisement of the local node own *actually-bound listen addrs*. It works for what it does, but it is not the surface that tells a remote peer "I now also listen on 443/80/ephemeral."
- **GAP A (mDNS):** `swarm.rs:127` defines `build_mdns_advertised_addrs(all_listeners)` — filters circuit/ws addrs, keeps direct ip4/ip6. It is **dead code**: the only reference is its own unit test (`swarm.rs:5596`). [V-READ] libp2p mDNS behaviour auto-advertises the swarm *listen* addrs, so ladder listens will propagate over mDNS automatically once P1-11 lands — but the intended filter (drop over-long TXT records >1300 bytes) is not applied. P1-12 must wire it or explicitly decide libp2p default is sufficient.
- **GAP B (self bound-addr registry):** `NewListenAddr` (`swarm.rs:3674-3677`) only logs and emits `SwarmEvent2::ListeningOn(address)`. There is **no canonical "my bound addrs" set** collected and fed to the advertise surfaces (peer_exchange producer, identify, relay registry). Identify auto-shares listen addrs at the libp2p layer, so identify/observed-address propagation is largely free; but the peer_exchange producer and any relay-registry advertisement need an explicit bound-addr accessor. P1-11 exports the set; P1-12 consumes it.
- identify/observed-address path: `observation.rs`/`reflection.rs`/`nat.rs` exist; `swarm.rs` already calls `add_external_address` from reflection/identify results (`swarm.rs:2720, 3541, 3658`) and `GetExternalAddresses` (`swarm.rs:4099`) exposes them. This path is functional. [V-READ]

### 1.3 Dial — CONFIRMED direction; race pattern is per-address, not per-port-ladder
- `SwarmCommand::Dial` (`swarm.rs:4104`) is promiscuous (`swarm.dial(addr)`, any PeerId). The peer-exchange consumer already dials every advertised discoverable addr (see 1.2). [V-READ]
- **CORRECTION:** there is no existing "same-host port ladder" dialer. When advertised addrs fail, nothing re-tries 443/80/8080/last-known-good on the same host. P1-12 adds this candidate-ladder synthesis (advertised first, then same-host ladder) with the repo fast-fallback race.
- WiFi Direct escape hatch: `relay/client.rs` `connect_websocket` (`client.rs:257-284`) builds a `ws://` (or `wss://` on 443) URL to bypass carrier port filtering; rationale comment confirms intent. `quic_port` defaults to **9002** (`client.rs:52, 65`). [V-READ] The WSS-on-443 relay path is real but the 9002 QUIC default is a hardcode P1-13 sweeps.
- **WiFi Direct dial hardcode is on the CLIENT side, not the group-owner listen side (sketch was imprecise).** `mobile_bridge.rs:1394-1400`: the **non-**group-owner spawns a dial to `/ip4/{group_owner_ip}/tcp/9001`. The `GroupInfo` struct (`transport/wifi_direct.rs:49-52`) carries `group_owner_ip: Option<String>` but **no port field** — there is no channel to negotiate the GO actual port, so the client assumes 9001. Fixing this requires either (a) the GO advertising its bound port through the WiFi Direct exchange, or (b) the client trying the port ladder against `group_owner_ip`. See P1-12/P1-13.

### 1.4 Remember — NEW BUILD; infrastructure exists to build on
- Sled backend exists: `store/backend.rs:103` `SledStorage` implements `StorageBackend` (`put/get/remove/scan_prefix/count_prefix/flush`). Serde JSON serialization is the house style (`store/history.rs:373`, `store/dedup.rs`). [V-READ]
- Routing already has a `NegativeCache` (`routing/negative_cache.rs`, bloom-filter, local-only, TTL-expiring) and `smart_retry` (`transport/mod.rs:76` `calculate_next_attempt, BackoffStrategy, DeliveryTrigger`). The "remember" store feeds these as an input, it does not replace them. [V-READ]
- **No "network fingerprint" concept exists anywhere.** [V-READ] P1-12 defines one (see §3.2 and §4). This is the largest genuinely-new piece and the source of the wire/schema items in §4.

## 2. Architecture decision (the final call)

**Extend `MultiPortConfig`, do not reinvent.** The four mechanisms map cleanly onto existing seams:

1. **Listen (P1-11):** flip both spawn call sites to `Some(MultiPortConfig::default())` (or a config-derived variant), delete the literal WS hardcode, make WS bind laddered, and add a `SwarmHandle` accessor that returns the *actually-bound* addr set (the missing self-registry). Preferred-stable-port support: extend `MultiPortConfig` with an optional `preferred_port: Option<u16>` prepended to the ladder (so a user who wants 9001 keeps it as *first* try, not *only* try).

2. **Advertise (P1-12):** feed the bound-addr set from P1-11 into (a) the peer_exchange producer as *self* addresses (new message semantics — see §4), (b) wire `build_mdns_advertised_addrs` or ratify libp2p default, (c) the WiFi Direct GO exchange (add a port to `GroupInfo`).

3. **Dial (P1-12):** candidate-ladder synthesis per peer = advertised addrs plus same-host {443,80,8080,last-known-good}, dialed with the repo fast-fallback race; keep WSS-on-443 relay as the escape hatch.

4. **Remember (P1-12):** a new `store/transport_memory.rs` sled tree keyed by `(peer_id, network_fingerprint)` -> `(transport, port, last_success_unix)`, read before dialing (promote last-good to front of ladder) and written on `ConnectionEstablished`. Feeds — does not replace — `NegativeCache`/`smart_retry`.

**Rejected alternative:** a brand-new port-negotiation sub-protocol on the wire. Rejected because libp2p identify + mDNS already propagate listen addrs for free, and the peer_exchange channel already carries multiaddr strings; a new protocol is unnecessary surface and would itself require operator sign-off. The only new wire content is *self-address semantics on an existing message* and *one new field on `GroupInfo`* (§4).

## 3. Work decomposition (slices + per-slice acceptance tests)

### 3.1 P1-11 [SONNET][AUDIT-GATE] Listen-side — see ticket
Default-on ladder in CLI + mobile spawn; kill literal 9002 WS; laddered WS bind; export bound-addr set from `SwarmHandle`. **Queues behind P1-04** (transport/ hotspot lane).
Acceptance: node with 9001 firewalled still binds and reports 443/80/ephemeral; `--port N` preserved as *preferred first*; bound-addr accessor returns the real set; no double-WS-bind.

### 3.2 P1-12 [SONNET][AUDIT-GATE] Advertise + dial + remember — see ticket
Self-addr propagation (peer_exchange + mDNS + identify confirmed); candidate port ladder + race; sled `transport_memory` store; `GroupInfo.port`. **Queues behind P1-04.** Touches `transport/` + `routing/` -> audit gate.
Acceptance: peer learns real bound port from advertisement (not 9001 assumption); dial ladder reaches a firewalled peer via 443; second contact reads last-good and connects on first attempt of the ladder; unit test round-trips the sled store.

### 3.3 P1-13 [HAIKU] Hardcode sweep — see ticket
`mobile_bridge.rs:1398` 9001 -> negotiated/laddered; `relay/client.rs` `quic_port: 9002` default; repo-wide grep for `9001|9002|9010` outside tests/docs; update `docs/` references.
Acceptance: grep clean; workspace builds; WiFi Direct client no longer assumes 9001.

## 4. REQUIRES OPERATOR SIGN-OFF (wire-format / API-contract changes)

Per the repo escalation rule, the following are **contract changes** and must be signed off by the operator before P1-12/P1-13 implement them. Flagged separately, not buried:

1. **`RelayPeerInfoMessage` / peer_exchange self-address semantics.** Today the producer (`peer_broadcast.rs`) only advertises *observed remote addresses of third peers*. P1-12 adds the local node advertising *its own bound listen addrs*. The struct shape does not change (`addresses: Vec<String>` already), but the **semantic contract** of what a peer is asserting changes, and it increases the address count / TXT + gossip payload size. Sign-off needed: confirm self-advertisement is acceptable and bounded (cap N addresses, drop over-1300-byte circuit addrs via `build_mdns_advertised_addrs`).

2. **`GroupInfo` gains a `port` field** (`transport/wifi_direct.rs:49`). This is consumed by the Kotlin WiFi Direct bridge over the FFI/JNI boundary and by `mobile_bridge.rs:1398`. Adding a field to a struct crossing the platform boundary is an API-contract change (Kotlin `GroupInfo` equivalent must gain the field). Sign-off + coordinated Android-side change needed. **Alternative requiring no wire change:** client tries the port ladder against `group_owner_ip` instead of reading a negotiated port — no `GroupInfo` change, at the cost of extra dial attempts. Operator picks.

3. **New sled schema: `transport_memory` tree.** Key = `tmem:{peer_id}:{network_fingerprint}`; value = JSON `{transport, port, last_success_unix, ladder_rank}`. New on-disk schema in the node sled DB. Sign-off needed on: (a) key layout and the **network-fingerprint definition** (proposed: hash of default-gateway MAC + local subnet /24, coarse enough to be stable, specific enough to distinguish hostile-vs-home network — but this is a privacy-adjacent local identifier, so it needs an explicit call), (b) retention/TTL, (c) migration (new tree, no migration of existing data — additive only). Because it touches `routing/` behavior it is inside [AUDIT-GATE].

4. **`MultiPortConfig` gains `preferred_port: Option<u16>`** (public struct in `core/src/transport/multiport.rs`). Public-API addition to a `transport/` type. Additive and backward-compatible (Default = None), but flagged because it is a published core API surface and inside the audit-gated tree.

Items 1–3 are behavioral/wire/schema and are the real sign-off asks. Item 4 is a courtesy flag (additive, low risk) but named because the tree is audit-gated.

## 5. Blocking dependencies (do not imply these can start now)

- **P1-11 and P1-12 both queue behind P1-04** (the transport/ hotspot-lane owner) per plan §1.4: "P1-04 owns transport/ until root cause lands. Adaptive-port implementation (P1-11/12) queues behind P1-04 even if the design (P1-10) finishes earlier." P1-10 (this note) can complete now; the implementation cannot start until P1-04 frees the lane.
- **P1-13 queues behind P1-11/P1-12** (it sweeps the hardcodes the negotiated-port machinery replaces) per plan §2.5.
- Both P1-11 and P1-12 carry **[AUDIT-GATE]** (touch `transport/`, P1-12 also `routing/`): mandatory `crypto-security-auditor` review before done, `release-gatekeeper` before merge.

## 6. Critical files (for the implementer)

- `core/src/transport/multiport.rs` — the ladder engine (extend with `preferred_port`).
- `core/src/transport/swarm.rs` — `start_swarm_with_config` listen block (1876-1943), WS hardcode (1938), `NewListenAddr` (3674), peer_exchange producer (3712-3754) + consumer (2450-2483), `build_mdns_advertised_addrs` (127), `SwarmHandle` accessors.
- `cli/src/main.rs` — spawn call site (1391-1403), WS bind (1406-1416), port derivation (723-728).
- `core/src/mobile_bridge.rs` — spawn call site (721-732), WiFi Direct client dial (1394-1400).
- `core/src/relay/peer_exchange.rs` + `core/src/transport/peer_broadcast.rs` — advertise producers.
- `core/src/transport/wifi_direct.rs` — `GroupInfo` (49).
- `core/src/store/backend.rs` (`SledStorage`) + new `core/src/store/transport_memory.rs`.
- `core/src/routing/negative_cache.rs` + `core/src/transport/smart_retry.rs` — the "remember" store consumers.
