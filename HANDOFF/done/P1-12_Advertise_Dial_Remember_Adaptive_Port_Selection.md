# TASK: P1-12  Advertise + dial + remember (adaptive port selection)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET] [AUDIT-GATE]
**Phase:** v1.0.0 Phase 1, Stage C (deliverability workstream).
**Design source:** `HANDOFF/plans/P1-10_adaptive_port_selection_design.md` 1.2, 1.3, 1.4, 3.2, 4 (read it first  the operator-sign-off items live in 4 and gate parts of this ticket).
**Depends on / BLOCKED BY:**
- **P1-04** (transport negotiation root-cause) MUST land first  this ticket touches the `transport/` hotspot lane, which P1-04 owns until root cause lands (plan 1.4). DO NOT start before the lane frees.
- **P1-11** must land first  this ticket consumes the bound-addr accessor P1-11 adds.
- **Operator sign-off** on P1-10 4 items 13 (see below) before implementing the wire/schema pieces.

## Source

`HANDOFF/V1_0_0_EXECUTION_PLAN.md` P1-12 (Stage C). Ground truth verified in P1-10 design note this session ([V-READ]).

## Problem (exact, verified)

Three sub-problems the design confirmed against real code:

**(A) Advertise  self bound-addrs never propagate.** The peer_exchange path works as a *third-peer relay-gossip* mechanism (producer `transport/peer_broadcast.rs` seeds addresses from an observed remote addr + circuit-relay enrichment at `swarm.rs:3712-3729`; consumer dials them at `swarm.rs:2450-2483`). But NO surface advertises *this node own actually-bound listen addrs* to peers. `build_mdns_advertised_addrs` (`swarm.rs:127`) exists but is **dead code** (only its own test references it, `swarm.rs:5596`). libp2p identify + mDNS auto-share listen addrs, but the peer_exchange self-advertisement and the mDNS TXT-size filter are unwired.

**(B) Dial  no same-host port ladder.** `SwarmCommand::Dial` (`swarm.rs:4104`) dials exactly the addresses it is given. When advertised addrs fail, nothing retries 443/80/8080/last-known-good on the same host. There is no candidate-ladder synthesis and no fast-fallback race across ports.

**(C) Remember  no persistence.** No per-(peer, network) last-successful (transport, port) store exists. `routing/negative_cache.rs` (bloom, TTL) and `transport/smart_retry.rs` exist and should be *fed* by the new store, not replaced. No "network fingerprint" concept exists anywhere in the tree.

## Root Cause

The plumbing carries multiaddr strings but no producer publishes self bound-addrs; the dialer has no ladder synthesis; there is no memory store to make second contact fast.

## Scope / What to do

1. **Advertise self bound-addrs** [REQUIRES SIGN-OFF item 1]. Feed the bound-addr set from P1-11 into the peer_exchange producer so the node advertises its own reachable addrs. Wire `build_mdns_advertised_addrs` (or record an explicit decision that libp2p mDNS default suffices). Confirm identify already shares listen addrs (it does at the libp2p layer  record the confirmation, do not rebuild).
2. **Dial candidate ladder + race** [no wire change]. Build per-peer candidates = advertised addrs, THEN same-host {443,80,8080,last-known-good}. Dial with the repo fast-fallback race (<500ms pattern). Keep the WSS-on-443 relay path (`relay/client.rs:257-284`) as the carrier-filter escape hatch.
3. **Remember store** [REQUIRES SIGN-OFF item 3]. New `core/src/store/transport_memory.rs` backed by `SledStorage` (`store/backend.rs:103`). Key `tmem:{peer_id}:{network_fingerprint}`, value JSON `{transport, port, last_success_unix, ladder_rank}`. Read before dialing (promote last-good to front of ladder); write on `ConnectionEstablished`. Define the network-fingerprint (proposed: hash of default-gateway MAC + local /24). Feed results into `NegativeCache`/`smart_retry` as an input.
4. **WiFi Direct GO port** [REQUIRES SIGN-OFF item 2]. Either add a `port` field to `GroupInfo` (`transport/wifi_direct.rs:49`) + coordinated Kotlin change so `mobile_bridge.rs:1398` dials the negotiated port, OR (no-wire-change alternative) have the client run the port ladder against `group_owner_ip`. Operator picks in 4 item 2.

## Blast Radius

`core/src/transport/swarm.rs` + `peer_broadcast.rs` + `wifi_direct.rs` (hotspot lane), `core/src/routing/` (negative cache / smart retry integration  widens audit gate), new `core/src/store/transport_memory.rs`, `core/src/mobile_bridge.rs`, and the Kotlin `GroupInfo` if 4 item 2 option (a) is chosen. Wire-format and on-disk-schema changes  see sign-off gate.

## Adversarial Review Requirement

**[AUDIT-GATE].** Touches `core/src/transport/` AND `core/src/routing/`. Mandatory `crypto-security-auditor` pass before done, `release-gatekeeper` before merge. The network-fingerprint is a privacy-adjacent local identifier  call this out explicitly in the audit request.

## Files to Touch

- `core/src/transport/swarm.rs`  self-addr into peer_exchange producer (near 3712-3754); wire `build_mdns_advertised_addrs` (127); dial ladder + race (near `SwarmCommand::Dial`, 4104).
- `core/src/transport/peer_broadcast.rs`  accept self bound-addrs.
- `core/src/transport/wifi_direct.rs`  `GroupInfo.port` (if option a).
- `core/src/mobile_bridge.rs`  WiFi Direct client dial (1394-1400) uses negotiated port or ladder.
- `core/src/store/transport_memory.rs`  NEW sled-backed store.
- `core/src/store/mod.rs`  register the new module.
- `core/src/routing/negative_cache.rs` / `core/src/transport/smart_retry.rs`  consume last-good.

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core --lib transport_memory
cargo test --workspace --no-run
```
Manual (device, feeds P1-14): with 9001 firewalled and 443 allowed, phone dials Windows and lands via the ladder; restart and confirm second contact hits the last-good rung first.

## Acceptance Tests (per-slice)

1. A peer learns this node real bound port(s) from advertisement  not the 9001 assumption. (integration: two local nodes, non-default ports)
2. Dial ladder reaches a peer whose advertised port is firewalled by falling through to 443. (integration/manual)
3. `transport_memory` round-trips: write `(peer, fp) -> (tcp, 443, t)`, read it back, and the dialer promotes 443 to rung 0 for that (peer, fp). (unit)
4. WiFi Direct client no longer hardcodes 9001 (dials negotiated port or ladder). (unit/manual)
5. `build_mdns_advertised_addrs` is either wired (referenced outside tests) or its removal/keep is recorded with rationale. (code + note)

## Do NOT

- Do NOT replace `NegativeCache`/`smart_retry`  the store feeds them.
- Do NOT invent a new wire protocol  reuse the existing `addresses: Vec<String>` field and identify/mDNS (P1-10 rejected a new sub-protocol).
- Do NOT implement the 4 sign-off items (self-advertise semantics, `GroupInfo.port`, sled schema/fingerprint) before the operator signs off.
- Do NOT start before P1-04 releases the lane and P1-11 lands.
- Do NOT skip the audit gate.
