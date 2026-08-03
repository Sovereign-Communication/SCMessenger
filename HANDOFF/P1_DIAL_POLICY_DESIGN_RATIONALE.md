# P1 Graceful Dial Policy — Design Rationale & Edge Cases

## Design Decisions

### 1. Per-Peer Backoff as Separate Module

**Decision:** Create `dial_policy.rs` as a standalone module rather than embedding in `swarm.rs`.

**Rationale:**
- **Modularity:** Clear separation of concerns. Dial policy is independently testable.
- **Reusability:** Future platforms (iOS, Android) can import and test independently.
- **Maintainability:** Changes to backoff strategy don't require swarm.rs edits.
- **Testing:** Unit tests in dial_policy.rs are fast and don't require swarm runtime.

**Trade-off:** Slightly more indirection (2 module boundaries), but worth it for clarity.

---

### 2. Exponential Backoff: 1s → 30s (Capped)

**Decision:** Start at 1s, double each failure, cap at 30s.

**Rationale:**
- **1s initial:** Covers transient DNS timeouts and brief network hiccups.
- **2x multiplier:** Standard exponential backoff (not too aggressive, not too lenient).
- **30s cap:** Prevents sustained hammering. After 5 failed attempts, peer is at max backoff (16s → 30s).
  - At 3 failures, peer is marked dead (no further retries this session).
  - So effective: 1s → 2s → 4s → dead (3 attempts total).

**Math:**
```
Attempt 1 (fail): backoff = 1s
Attempt 2 (fail): backoff = 2s
Attempt 3 (fail): backoff = 4s → marked dead
```

**Alternative Considered:** 1s → 10s → 60s (faster ramp). **Rejected:** Too aggressive for transient failures (DNS cache delays, mobile handover).

---

### 3. Concurrent Dial Limit: 3 Per Peer

**Decision:** Max 3 in-flight dials to the same peer simultaneously.

**Rationale:**
- **3 is tunable:** Comment in code notes it's a parameter. Change if needed.
- **Covers ladder:** Direct address + last-good port + fallback timeout = ~2-3 candidates typical.
- **Prevents spiral:** Without limit, a misconfigured peer could spawn unlimited dials.

**Enforcement Points:**
1. **Register:** Check limit before `swarm.dial()` succeeds
2. **Complete:** Decrement when dial results in ConnectionEstablished or OutgoingConnectionError
3. **Timeout:** Also complete and apply backoff if dial never receives an event

**Example Timeline:**
```
T0: register_dial_attempt("peer-a") → OK (1/3)
T1: register_dial_attempt("peer-a") → OK (2/3)
T2: register_dial_attempt("peer-a") → OK (3/3)
T3: register_dial_attempt("peer-a") → REJECTED (3/3)
T4: ConnectionEstablished for one dial → complete_dial_attempt() → (2/3)
T5: register_dial_attempt("peer-a") → OK (3/3)
```

---

### 4. Dead Peer Tracking (Per-Session)

**Decision:** Mark peer as dead (no retry this boot) after 3 failed attempts.

**Rationale:**
- **Prevents spam:** Unreachable peers (wrong IP, offline) won't hammer logs.
- **Per-session only:** Not persisted to disk. On reboot, retry from scratch.
  - Rationale: Network conditions change. If peer was unreachable yesterday, it might be up now.
  
**Alternative Considered:** Persist dead list to disk. **Deferred to Phase 2** for safety review (could interfere with mobile roaming).

---

### 5. Circuit-Relay Ladder After Connection

**Decision:** On successful connection, add relay multiaddrs to future dial candidates.

**Rationale:**
- **Warm-start:** If we connected to peer A directly, we can use A as a relay to reach peer B.
- **Incremental discovery:** Each peer that connects becomes a potential relay. Network grows organically.
- **Ladder order:** Try direct first (fastest), then relay (NAT traversal), then timeout (fallback).

**Multiaddr Format:**
```
/ip4/x.x.x.x/tcp/4001/p2p/RELAY_ID/p2p-circuit/p2p/TARGET_ID
```
- Tells libp2p: "use peer RELAY_ID to reach TARGET_ID via circuit relay protocol"
- libp2p handles rest: dial RELAY_ID, establish relay reservation, proxy to TARGET_ID

**Why After Connection?**
- Can't use peer as relay until we know it's reachable and supports relay protocol
- ConnectionEstablished proves both
- Extract external addresses from `swarm.external_addresses()` at that moment

---

### 6. Thread Safety & Concurrency

**Decision:** Use `Arc<RwLock<HashMap>>` for backoff state and concurrent dial counts.

**Rationale:**
- **Arc:** Multiple tasks (swarm event loop, command handler) share same state
- **RwLock:** parking_lot RwLock (not std::sync) for fairness and Instant support
- **HashMap:** O(1) lookup, independent per-peer

**Lock Granularity:**
- Each call holds lock only for the duration of the operation (lookup, update, increment)
- No long-held locks that could block the event loop
- Read lock for `get_backoff_state()` (diagnostics)
- Write lock for state mutations

**No Deadlocks:**
- Single lock per manager. Can't deadlock between managers.
- Lock always released at end of method (never held across await)

---

## Edge Cases & Handling

### Edge Case 1: Dial Queued but Never Completed

**Scenario:** `swarm.dial()` succeeds (dial queued), but neither ConnectionEstablished nor OutgoingConnectionError ever fires.

**Handling:**
1. **Pending Dial Sweep:** Every 5 seconds, check for dials older than 10 seconds
2. **On Timeout:**
   - Call `complete_dial_attempt()` (decrement concurrent count)
   - Call `record_dial_failure()` (apply backoff)
   - Send error reply to caller

**Why?** Prevents backoff state from drifting if libp2p event gets lost.

---

### Edge Case 2: Dial Policy Rejects (Concurrent Limit)

**Scenario:** `register_dial_attempt()` returns false because peer is at limit.

**Handling:** Return error to caller immediately.
```rust
"Peer is at concurrent dial limit (3/3)"
```

**Why?** Caller can retry later. No need to queue in pending_dials.

---

### Edge Case 3: Connection Established for Multiple Concurrent Dials

**Scenario:** Dials A, B, C to same peer. All 3 resolve via different addresses. Which one matches ConnectionEstablished?

**Handling:**
```rust
// Only resolve a pending_dial if its candidate_addrs include the connected address
if entry.candidate_addrs.iter().any(|a| a == &remote_addr || a == &stripped_remote) {
    // Resolve this entry
}
```

**Why?** Prevents false resolution. Dial to different peer with same IP won't accidentally resolve another dial.

---

### Edge Case 4: Peer Connects via Relay, Then Adds Itself as Relay

**Scenario:**
1. Dial peer-B via relay through peer-A
2. Connection succeeds
3. peer-B registers itself as a relay

**Handling:** CircuitRelayLadder just adds it. No deduplication needed.

**Why?** Relay supports multiple addresses. No harm in redundancy. Circuit-relay protocol handles it.

---

### Edge Case 5: Very Fast Network Recovery

**Scenario:** Peer fails 2x in quick succession, then comes back up 1 second later.

**Timeline:**
```
T0: Attempt 1 → FAIL (backoff_duration = 2s, next_dial = T0 + 2s)
T1: Attempt 2 → FAIL (backoff_duration = 4s, next_dial = T1 + 4s)
T2: Peer comes back online, but we're backed off until T5
```

**Handling:** Caller can manually `reset_on_connection_established()` if they detect connection via another path (e.g., mDNS). Or wait until next backoff expires.

**Why?** Correct behavior. We don't want to hammer a flaky peer.

---

### Edge Case 6: Backoff Entries Never Pruned (Memory Leak)

**Scenario:** If pruning is disabled, dead peer entries accumulate forever.

**Handling:**
- Prune every 5 minutes (configurable interval in code)
- Remove entries not touched in 1 hour
- Rough bound: ~1000 peers * 1KB per entry = 1MB typical

**Why?** Bounded memory even in pathological scenarios.

---

### Edge Case 7: Dial via Circuit-Relay with Dead Peer

**Scenario:**
1. Peer-B is marked dead (3 failed attempts)
2. New dial request arrives: "dial peer-B via relay"

**Handling:**
```rust
if !dial_policy_manager.register_dial_attempt(&addr_key, ...) {
    return Err("Peer marked as dead...");
}
```

**Why?** Reject immediately. Don't queue relay dials to dead peers.

---

### Edge Case 8: Backoff State Lost on Panic

**Scenario:** Swarm task panics and restarts.

**Handling:**
- Backoff state is in memory only (Arc<RwLock> on heap)
- On panic/restart, state is lost
- Next boot: peers start fresh with attempt_count=0

**Why?** Acceptable trade-off. Backoff is per-session for safety. Full recovery on reboot is fine.

---

### Edge Case 9: Rapid Register/Complete Cycling

**Scenario:** Caller calls `register_dial_attempt()`, then immediately `complete_dial_attempt()` without actual dial.

**Handling:** Allowed. Concurrent count goes up then down. State is consistent.

**Why?** No invariant violations. Could happen if `swarm.dial()` fails immediately (rare).

---

### Edge Case 10: Multiple Peers with Same Address Key

**Scenario:** Two peers with same IP but different ports get the same address key somehow.

**Handling:** This can't happen. `multiaddr_to_key()` includes TCP/UDP port.

**Why?** Addresses are fully normalized: `/ip4/X/tcp/P` → unique key per (IP, port) pair.

---

## Performance Analysis

### O(1) Operations
- `register_dial_attempt()` — HashMap lookup + insert
- `complete_dial_attempt()` — HashMap decrement
- `record_dial_failure()` — HashMap update
- `reset_on_connection_established()` — HashMap update
- `get_backoff_state()` — HashMap clone

### O(n) Operations
- `prune_old_entries()` — Full scan of backoff map (every 5 min, n~1000 peers max)

### Lock Contention
- RwLock acquired per operation (very brief)
- No long-held locks
- Read lock for diagnostics; write lock for mutations

**Estimated Impact:**
- Dial path latency: +1-2 μs per dial attempt (HashMap lookup)
- Memory: ~1KB per dead peer, ~1000 max = 1MB typical

---

## Security Considerations

### DoS Protection
- Concurrent dial limit prevents malicious caller from spawning unbounded dials
- Backoff prevents hammering flaky peers
- Dead peer tracking stops spam after 3 failures

### Timing Side Channels
- Backoff duration is not secret (network observable anyway)
- No cryptographic operations in dial_policy

### No Unsafe Code
- All Rust safe abstractions (Arc, RwLock, HashMap)
- No manual memory management

---

## Testing Strategy

### Unit Tests (dial_policy.rs)
- State transitions (failure → failure → dead)
- Exponential progression + cap
- Connection reset behavior
- Permanent failure marking
- Concurrent limit enforcement

### Integration Tests (integration_dial_policy.rs)
- Multi-peer independence
- Circuit-relay address construction
- Address key normalization
- Timing-sensitive eligibility checks
- Pruning functionality

### Manual Testing (via swarm.rs)
- Run end-to-end test with flaky peer
- Verify backoff in logs
- Verify relay addresses added after connection
- Verify concurrent limit via parallel dials

---

## Future Enhancements

### Phase 2: Persistent Dead Peer List
- Serialize dead peers to disk on session exit
- Load on startup, but allow manual re-enable
- **Safety:** Only after careful mobile-roaming testing

### Phase 3: Adaptive Backoff
- Adjust multiplier based on network type (WiFi vs cellular)
- Faster retry on mobile (roaming expected)
- Slower on stable networks

### Phase 4: Dead-Letter Queue
- Move permanently-failed peers to inspection queue
- Analytics: "Why did we give up on this peer?"

### Phase 5: Per-Transport Backoff
- Separate backoff for TCP, QUIC, relay
- Allow TCP to fail fast while relay retries

---

**End of Design Rationale**
