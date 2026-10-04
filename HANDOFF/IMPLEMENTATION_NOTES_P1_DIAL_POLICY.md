# P1 Graceful Dial Policy Implementation — Items 3+4

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** Complete (Code + Tests + Documentation)  
**Date:** 2026-07-22  
**Scope:** Per-peer backoff state machine + Circuit-relay ladder preference

## Overview

This implementation adds graceful dial policy to SCMessenger v0.4.0:

### Item 3: Per-Peer Backoff State Machine (max 3 concurrent dials)
- Each peer maintains `attempt_count` (0-3), `last_attempt_ts`, `backoff_duration` (1s → 30s exponential)
- Global enforcer: never more than 3 concurrent outbound dials to any peer
- Dial loop filters: only dials peers where `attempt_count < 3` AND `now() >= last_attempt_ts + backoff_duration`
- On `ConnectionEstablished`: reset peer's backoff to `attempt_count = 0`
- On dial failure (timeout/reset): increment `attempt_count`, double backoff (1s → 2s → 4s → 8s → 16s → 30s capped)

### Item 4: Prefer Circuit-Relay After Connection Established
- Listen for `ConnectionEstablished` libp2p event
- Once connected, add circuit-relay multiaddr to dial candidate list
- Ladder order: direct addresses → circuit-relay → fallback timeout
- Circuit relay address format: `/ip4/<relay-ip>/tcp/<relay-port>/p2p/<relay-peer-id>/p2p-circuit/p2p/<target-peer-id>`

## Files Changed

### 1. New File: `core/src/transport/dial_policy.rs`
**Size:** ~650 lines (code + tests)

**Key Structures:**
- `PerPeerBackoffState` — tracks attempt_count, last_attempt_ts, backoff_duration per peer
- `DialPolicyManager` — global orchestrator for backoff + concurrent dial limits
- `CircuitRelayLadder` — builds circuit-relay multiaddrs for known relay peers

**Key Methods:**
- `PerPeerBackoffState::is_eligible()` — checks if peer is eligible for dial (not dead, under limit, backoff elapsed)
- `PerPeerBackoffState::on_dial_failure()` — increments attempt, doubles backoff, marks dead at 3 attempts
- `PerPeerBackoffState::on_connection_established()` — resets to attempt_count=0, backoff=1s
- `DialPolicyManager::register_dial_attempt()` — checks eligibility + concurrent limit, returns bool
- `DialPolicyManager::complete_dial_attempt()` — decrements concurrent count
- `DialPolicyManager::record_dial_failure()` — applies backoff
- `CircuitRelayLadder::add_relay()` — registers relay peer + external addrs
- `CircuitRelayLadder::build_relay_addresses()` — constructs circuit-relay multiaddrs for target

**Comprehensive Unit Tests:**
- Exponential backoff progression (1→2→4→8→16→30s)
- Backoff cap enforcement
- Concurrent dial limit (max 3)
- Connection reset behavior
- Permanent failure handling
- Multi-peer independence
- Circuit-relay address construction

### 2. Modified: `core/src/transport/mod.rs`
**Changes:**
- Added `pub mod dial_policy;` declaration
- Added public exports:
  ```rust
  pub use dial_policy::{
      CircuitRelayLadder, DialPolicyManager, PerPeerBackoffState, multiaddr_to_key,
  };
  ```

### 3. Modified: `core/src/transport/swarm.rs`
**Changes:**

#### a) Import statements (line ~24):
```rust
use super::dial_policy::{CircuitRelayLadder, DialPolicyManager, multiaddr_to_key};
```

#### b) Initialization in swarm task (line ~2368):
```rust
// P1 Item 3: Per-peer backoff state machine (max 3 concurrent dials)
let dial_policy_manager = DialPolicyManager::new();
let mut backoff_prune_interval = tokio::time::interval(Duration::from_secs(300));

// P1 Item 4: Circuit-relay preference after connection established
let circuit_relay_ladder = CircuitRelayLadder::new();
```

#### c) Dial command handler (line ~4618):
**Before:** Simple dial attempt without policy checks  
**After:**
```rust
SwarmCommand::Dial { addr, reply } => {
    // ... existing port ladder logic ...

    // P1 Item 3: Check dial policy (backoff + concurrent limit)
    let addr_key = multiaddr_to_key(&addr);
    if !dial_policy_manager.register_dial_attempt(&addr_key, target_peer_id) {
        // Return error with reason (dead, attempt_count, or concurrent limit)
        let policy_error = /* details */;
        debug!("[DIAL-REJECTED] {}: {}", addr_key, policy_error);
        let _ = reply.send(Err(policy_error)).await;
        continue;
    }

    // ... perform actual dial ...

    // P1 Item 4: Add circuit-relay addresses to candidate ladder
    let relay_addrs = circuit_relay_ladder.build_relay_addresses(pid);
    for relay_addr in relay_addrs {
        if !candidates.contains(&relay_addr) {
            candidates.push(relay_addr);
        }
    }

    match dial_res {
        Ok(_) => { /* register pending dial */ }
        Err(e) => {
            // Complete dial attempt since it failed to queue
            dial_policy_manager.complete_dial_attempt(&addr_key);
            let _ = reply.send(Err(err_msg)).await;
        }
    }
}
```

#### d) ConnectionEstablished event handler (line ~3987):
**Before:** Simple connection logging  
**After:**
```rust
SwarmEvent::ConnectionEstablished { peer_id, endpoint, connection_id, .. } => {
    let remote_addr = endpoint.get_remote_address().clone();

    // P1 Item 3: Reset backoff state on successful connection
    let addr_key = multiaddr_to_key(&remote_addr);
    dial_policy_manager.reset_on_connection_established(&addr_key, Some(peer_id));
    dial_policy_manager.complete_dial_attempt(&addr_key);

    // P1 Item 4: Add this peer as relay candidate for circuit-relay addresses
    let external_addrs: Vec<Multiaddr> = swarm.external_addresses().cloned().collect();
    if !external_addrs.is_empty() {
        circuit_relay_ladder.add_relay(peer_id, external_addrs);
    }

    // ... rest of connection handling ...
}
```

#### e) OutgoingConnectionError event handler (line ~4246):
**Before:** Error logging and bootstrap backoff only  
**After:**
```rust
SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
    // ... existing error logging ...

    if let libp2p::swarm::DialError::Transport(ref errors) = error {
        for (failed_addr, _) in errors {
            let stripped_failed: Multiaddr = /* strip /p2p/ */;

            // P1 Item 3: Apply backoff on transient dial failure
            let addr_key = multiaddr_to_key(&stripped_failed);
            dial_policy_manager.record_dial_failure(&addr_key, peer_id);
            dial_policy_manager.complete_dial_attempt(&addr_key);

            // ... resolve pending dials ...
        }
    }

    // ... bootstrap backoff handling ...
}
```

#### f) Pending dial sweep interval (line ~2512):
**Before:** Timeout handling only  
**After:**
```rust
_ = pending_dial_sweep_interval.tick() => {
    let timed_out: Vec<Multiaddr> = /* timeout check */;
    for key in timed_out {
        if let Some(entry) = pending_dials.remove(&key) {
            // P1 Item 3: Complete and apply backoff on timeout
            let key_str = key.to_string();
            dial_policy_manager.complete_dial_attempt(&key_str);
            dial_policy_manager.record_dial_failure(&key_str, None);

            // ... send error reply ...
        }
    }
}

// P1 Item 3: Add backoff prune interval
_ = backoff_prune_interval.tick() => {
    dial_policy_manager.prune_old_entries(Duration::from_secs(3600));
    debug!("[DIAL-POLICY] Pruned stale backoff entries");
}
```

### 4. New File: `core/tests/integration_dial_policy.rs`
**Size:** ~650 lines

**Test Coverage:**
1. `test_dial_policy_exponential_backoff()` — backoff doubling from 1→2→4→8s
2. `test_backoff_cap_at_30_seconds()` — enforce 30s maximum
3. `test_connection_resets_backoff()` — reset on success
4. `test_permanent_failure_marks_dead()` — immediate dead marking
5. `test_dial_policy_manager_concurrent_limit()` — max 3 concurrent dials
6. `test_dial_policy_manager_backoff_rejection()` — reject backed-off dials
7. `test_dial_policy_manager_dead_peer()` — persistent dead peer rejection
8. `test_dial_policy_multiple_peers()` — per-peer independence
9. `test_circuit_relay_ladder_construction()` — relay address building
10. `test_circuit_relay_ladder_multiple_relays()` — multiple relay support
11. `test_multiaddr_to_key_strips_peer_id()` — address key normalization
12. `test_backoff_eligibility_timing()` — time-based eligibility
13. `test_dial_policy_concurrent_limit_per_peer()` — independent limits
14. `test_backoff_progression_sequence()` — complete progression test
15. `test_circuit_relay_invalid_addresses_skipped()` — robustness test
16. `test_dial_policy_prune_old_entries()` — memory hygiene

## Logging

### DEBUG Level (`tracing::debug`)
- `[DIAL-POLICY]` — Dial policy decision logs (eligibility checks, concurrent limits)
- `[DIAL-BACKOFF]` — Backoff state updates (attempt count, backoff duration)
- `[DIAL-REJECTED]` — Dial rejections with reason
- `[CIRCUIT-RELAY]` — Relay ladder construction and updates

### INFO Level (`tracing::info`)
- `[DIAL-BACKOFF]` — Peer marked dead after 3 failures
- `[CIRCUIT-RELAY]` — Relay registration

## Architecture Notes

### State Isolation
- `DialPolicyManager` holds Arc<RwLock<HashMap>> for thread-safe, lock-protected state
- Per-peer state is completely independent (no cross-peer blocking)
- Backoff state is **ephemeral** (per-session, not persisted across reboots)

### Hot-Path Considerations
- `register_dial_attempt()` and `complete_dial_attempt()` are O(1) hash lookups
- `record_dial_failure()` is O(1) with lock held briefly
- Backoff pruning happens every 5 minutes (low frequency)

### Dial Ladder Priority
After peer connection, candidate ladder is:
1. Direct addresses (original + last-known-good + port fallbacks)
2. Circuit-relay addresses (via known relays)
3. Fallback timeout (libp2p handles)

This provides warm-start benefit: new peers use existing relay connections for faster discovery.

## Acceptance Criteria

- [x] Code compiles: `cargo check --workspace`
- [x] Lint passes: `cargo clippy --workspace -- -D warnings`
- [x] Backoff state machine test passes
- [x] Concurrent limit test passes
- [x] Circuit-relay ladder test passes
- [x] Ready for adversarial audit (transport/ audit gate applies)

## Review Checklist

- [x] Backoff logic is exponential (1s → 30s, capped)
- [x] Concurrent dial limit enforced per-peer (max 3)
- [x] Reset on successful connection
- [x] Circuit-relay addresses added after direct attempts
- [x] Verbose logging at key decision points
- [x] Comprehensive test coverage
- [x] Memory-safe: Arc<RwLock> for shared state
- [x] No unsafe code in dial_policy.rs
- [x] Thread-safe under tokio async context

## Future Enhancements

1. **Persistent Backoff** (Phase 2): Save dead-peer list to disk across reboots
2. **Adaptive Tuning**: Adjust backoff based on network conditions
3. **Dead-Letter Queue**: Move permanently-failed peers to inspection queue
4. **Metrics Export**: Expose dial policy stats to observability system
5. **Per-Transport Backoff**: Different strategies for TCP vs QUIC vs relay

## Dependencies Added

- None (uses existing `parking_lot`, `web_time`, `libp2p`, `tracing`)

---

**Implementation Complete**  
Ready for security audit and merge to main branch.
