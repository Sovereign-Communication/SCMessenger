# Freenet Lessons Learned — Handoff Document

**Date**: 2026-07-31
**Source**: Investigation of `freenet/freenet-core` (local dashboard at 127.0.0.1:7509, whitepaper, GitHub source)
**Target**: SCMessenger (`core/src/transport/nat.rs`, `core/src/transport/internet.rs`, libp2p behaviours)

---

## Executive Summary

Freenet has **production-grade NAT traversal** (64% success rate measured live: 38/59) with a **dual-sided simultaneous hole-punch** implementation. SCMessenger's NAT traversal (`core/src/transport/nat.rs`) has the framework but **the actual hole-punch logic is stubbed** — `send_hole_punch_probes()` at line 442 just logs and returns `Success`.

**Key insight**: Freenet's transport is a standalone UDP implementation with X25519 + AES-GCM. SCMessenger uses libp2p (TCP/QUIC/WS) with `dcutr` + `autonat` + circuit relay. The approaches are **compatible but different layers** — Freenet's techniques can harden SCMessenger's libp2p-based NAT traversal.

---

## Verified Freenet Implementation Details

| Component | File:Lines | Key Parameters | Status |
|-----------|------------|----------------|--------|
| NAT traversal state machine | `connection_handler.rs:2007-2400` | 3s deadline, 200ms cadence, 40 attempts | CHECK Production (64% success) |
| X25519 intro packets | `crypto.rs:140-200` | Static-ephemeral DH, ChaCha20Poly1305 | CHECK |
| Symmetric handshake | `symmetric_message.rs` | `AckConnection`, AES-128-GCM | CHECK |
| Packet reliability | `peer_connection.rs` | `SentPacketTracker`, `ReceivedPacketTracker` | CHECK |
| Fixed-rate congestion | `fixed_rate.rs`, `token_bucket.rs` | Token bucket pacing | CHECK Production default |
| Summary/delta sync | Whitepaper §4.4 | `summarize`/`getDelta`/`applyDelta` | CHECK |

---

## SCMessenger Current State (Verified)

### `core/src/transport/nat.rs` — **Hole-Punch is Stubbed**

```rust
// Lines 442-494: send_hole_punch_probes()
async fn send_hole_punch_probes(&self, attempt_key: &str) -> Result<(), NatTraversalError> {
    // ... logging ...
    // In production, this would:
    // 1. Create UDP socket bound to local external port
    // 2. Send probe packets to remote_external_addr
    // 3. Listen for incoming probe packets from remote
    // 4. Confirm bidirectional connectivity
    attempt.status = HolePunchStatus::Success;  // <-- STUB: always succeeds
    info!("UDP hole-punch successful with remote peer");
    Ok(())
}
```

**Config (line 310-323)**:
```rust
NatConfig {
    max_attempts: 5,          // Freenet: 40 (prod) / 10 (test)
    attempt_timeout: 10,      // Freenet: 3s total / 200ms cadence
    enable_hole_punch: true,
    enable_relay_fallback: true,
}
```

### libp2p Behaviours (`core/src/transport/behaviour.rs:35-75`)

```rust
// Actual NAT traversal via libp2p:
pub dcutr: dcutr::Behaviour,      // Direct connection upgrade (hole-punch via relay)
pub autonat: autonat::Behaviour,  // NAT status probing
pub relay_client: relay::client::Behaviour,  // Circuit relay v2 client
pub relay_server: relay::Behaviour,          // Circuit relay v2 server (all nodes mandatory relays)
pub upnp: upnp::tokio::Behaviour,            // UPnP port mapping (non-WASM, non-Android)
```

### Internet Relay (`core/src/transport/internet.rs`)

- `InternetRelay` manages relay connections (Client/Server/Both modes)
- `attempt_hole_punch()` at line 466: **also stubbed** — logs and returns `Ok(())`
- Bandwidth limiting per relay (1 Mbps default)

---

## Gap Analysis: What Freenet Has That SCMessenger Needs

###  HIGH PRIORITY — Integrate Immediately

| Freenet Technique | SCMessenger Gap | Integration Approach |
|-------------------|-----------------|----------------------|
| **Dual-sided simultaneous hole-punch** | SCMessenger relies on `dcutr` (relay-assisted) only | Add raw UDP hole-punch as **supplementary path** before/parallel to `dcutr` |
| **200ms cadence for 3s** (keeps NAT mappings alive) | SCMessenger: 5 attempts × 10s = 50s but no continuous send | Replace `max_attempts: 5, attempt_timeout: 10` with **time-bounded continuous send** |
| **X25519 static-ephemeral intro + symmetric key exchange** | SCMessenger uses libp2p Noise (transport encryption) | Add **application-layer handshake** inside hole-punch probes for faster symmetric key setup |
| **Rate-limited asymmetric decryption** (1s min interval) | No rate limit on intro packet processing | Add `ASYM_DECRYPTION_RATE_LIMIT` equivalent to prevent CPU exhaustion |
| **Connection state machine** (`NatTraversal` state tracking) | `HolePunchAttempt` struct exists but no state machine | Adopt Freenet's `HandshakePhase` enum (StartOutbound → RemoteInbound) |

**LoC Impact**: ~800 LoC (Freenet's `traverse_nat` function) → adapt to SCMessenger's async patterns

---

###  MEDIUM PRIORITY — Evaluate After High Priority

| Freenet Technique | SCMessenger Status | Decision Needed |
|-------------------|-------------------|-----------------|
| **Fixed-rate + token bucket congestion** | libp2p handles transport congestion; app-layer has `SyncRateLimiter` | Only if message throughput issues appear |
| **Summary/delta sync for messages** | Drift protocol has IBLT sketch + CRDT mesh store | Already have efficient sync (IBLT); Freenet's generic interface is a pattern, not code |
| **Per-neighbor performance model (isotonic regression)** | Mycorrhizal routing has reputation + latency tracking | Freenet's approach is novel but complex; current routing may suffice |
| **Deterministic simulation (Turmoil)** | Integration tests marked `#[ignore]` requiring SwarmHandle | Adopt Turmoil or similar for CI — high value |

---

###  LOW PRIORITY / NOT APPLICABLE

| Freenet Feature | Why Not Applicable |
|-----------------|-------------------|
| Small-world ring routing | SCMessenger uses libp2p Kademlia (DHT) + mycorrhizal routing |
| Contract/WASM application model | SCMessenger is native Rust + UniFFI bindings |
| Delegate pattern for private keys | Keys in `core/src/identity/keys.rs` — native, not WASM |
| Group chat / invite chains | 1:1 messaging only |
| Subscription trees (8-min leases) | Direct push via request-response |
| UI bundles as contract state | Native mobile/desktop apps |

---

## Concrete Integration Plan for SCMessenger

### Phase 1: Harden NAT Traversal (Week 1-2)

**File**: `core/src/transport/nat.rs`

1. **Replace `send_hole_punch_probes` stub** with actual UDP hole-punch:
   ```rust
   // Freenet pattern:
   // - Bind UDP socket to local external port (reuse from libp2p if possible)
   // - Send probe packets every 200ms for 3s (not 5 attempts × 10s)
   // - Probe format: magic + transaction_id + timestamp + signature
   // - Success on bidirectional probe receipt + RTT < 500ms
   ```

2. **Update `NatConfig`** to match Freenet's proven params:
   ```rust
   // FROM: max_attempts: 5, attempt_timeout: 10
   // TO:   max_attempts: 40 (or time-bounded), cadence: 200ms, deadline: 3s
   ```

3. **Add state machine** (from Freenet's `HandshakePhase`):
   ```rust
   enum HolePunchPhase {
       StartOutbound,    // Sent intro, waiting for remote intro
       RemoteInbound,    // Got remote intro, sending symmetric ACK
   }
   ```

4. **Add asymmetric decryption rate limit** (Freenet: 1s min interval per address):
   ```rust
   // In nat.rs or a new transport security module
   const ASYM_DECRYPTION_RATE_LIMIT: Duration = Duration::from_secs(1);
   ```

### Phase 2: Supplement libp2p with Raw UDP Path (Week 2-3)

**Files**: `core/src/transport/nat.rs`, `core/src/transport/internet.rs`, `core/src/transport/swarm.rs`

- Freenet's transport is **pure UDP** with custom reliability
- SCMessenger has **libp2p TCP/QUIC/WS** + `dcutr` for hole-punch
- **Strategy**: Run both in parallel
  - libp2p `dcutr` for relay-assisted hole-punch (current)
  - Raw UDP hole-punch for direct path (new, from Freenet)
  - First to succeed wins

**Integration point**: `NatTraversal::start_hole_punch()` → spawn both strategies

### Phase 3: Handshake Optimization (Week 3)

- Freenet exchanges symmetric keys **during** hole-punch (intro packet carries AES key)
- SCMessenger establishes Noise session **after** connection
- **Win**: Encrypt first application message in hole-punch ACK, avoid 1-RTT

---

## Code References to Port/Adapt

| Freenet Source | Lines | SCMessenger Target |
|----------------|-------|-------------------|
| `connection_handler.rs` | 2007-2400 (`traverse_nat`) | `nat.rs` `send_hole_punch_probes` |
| `crypto.rs` | 140-200 (`encrypt`/`decrypt`) | New `crypto/hole_punch.rs` or extend `crypto/negotiation.rs` |
| `symmetric_message.rs` | `AckConnection`, `ack_ok` | `nat.rs` handshake payload types |
| `peer_connection.rs` | `SentPacketTracker`, `ReceivedPacketTracker` | `drift/frame.rs` + `drift/sync.rs` (already have) |
| `fixed_rate.rs` + `token_bucket.rs` | Congestion control | `drift/rate_limit.rs` (already have `SyncRateLimiter`) |

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| UDP socket conflicts with libp2p | Medium | libp2p can expose UDP socket; or bind separate port |
| Dual hole-punch races | Low | First success wins; both use same external addresses |
| Increased complexity | Medium | Isolate in `nat.rs`; feature-flag behind `enable_raw_udp_hole_punch` |
| Mobile battery (UDP keepalive) | Low | 200ms × 3s = 15 packets per attempt; negligible |

---

## Verification Checklist (Post-Integration)

- [ ] NAT traversal success rate improves from baseline (measure via dashboard)
- [ ] Symmetric NAT pairs connect (currently fail with `dcutr` alone)
- [ ] Hole-punch completes in <3s (vs current 10-50s)
- [ ] No regression in relay fallback path
- [ ] Mobile (Android/iOS) UDP binding works
- [ ] Integration tests pass in `tests/integration_nat.rs`

---

## Appendix: Freenet Dashboard Metrics (Live)

```
Peer count: 33 connected
NAT hole punching: 38/59 successful (64%) — 12/20 recent
Data transferred: 32.6 MB up / 41.2 MB down
Ring location: 0.7334
Peer diversity: 20+ countries
```

These are **real production numbers** from the running Freenet node at `127.0.0.1:7509`.

---

## Decision Required

1. **Approve Phase 1** (harden `nat.rs` with Freenet's proven params + state machine)?
2. **Approve Phase 2** (add raw UDP hole-punch as parallel path to libp2p `dcutr`)?
3. **Target timeline**: 2 weeks for Phase 1, 1 week for Phase 2?

The Freenet code is **verified, production-tested, and directly applicable** to SCMessenger's #1 connectivity challenge.