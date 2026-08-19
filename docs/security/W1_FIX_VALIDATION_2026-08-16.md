# W1 Fix Validation Report

**Date:** 2026-08-16
**Validator:** CRITICAL_VALIDATOR
**Target:** PR #172 (`fix/w1-failover-cooldown-persist` into `tracking/pre-v040-tag-work`)

## V1. Is the bypass actually closed?
**Yes.** The bypass is closed. 
- `core/src/transport/swarm.rs:833` (in base): The only function that cleared the cooldown, `forget_peer`, was removed entirely along with its invocation sites in the swarm event loop (`core/src/transport/swarm.rs:5397` and `7736`).
- `RelayAbuseGuardrails` is instantiated once outside the main event loop (`core/src/transport/swarm.rs:3050` and `6674`) and is never recreated during normal swarm operation.
- The map `failover_reexchange_at` is only modified internally by `allow_failover_reexchange`. The only way to sidestep the cooldown is by presenting a fresh `PeerId`, which costs a new Ed25519 identity keypair generation and noise handshake per attempt—an acceptable cryptographic cost.

## V2. Did removing `forget_peer` break anything?
**No.** 
- `forget_peer` previously contained only a single instruction: `self.failover_reexchange_at.remove(peer_id);`. 
- Other per-peer states in `RelayAbuseGuardrails` (like `per_peer_buckets`) are cleaned up asynchronously via time-based sweeps (e.g., `prune_peer_buckets`) and did not rely on `forget_peer` for connection cleanup. No silent memory leaks were introduced.

## V3. Is there a behavioural regression on legitimate reconnects?
**No.** Topology convergence is fully preserved.
- The failover cooldown (`allow_failover_reexchange`) is **only** checked during a partial connection close (when `num_established > 0`).
- On a full disconnect (`num_established == 0`), `ledger_exchanged_peers.remove(&peer_id)` is still unconditionally called (`core/src/transport/swarm.rs:5398` and `7737`).
- When the honest peer reconnects, `ConnectionEstablished` checks `!ledger_exchanged_peers.contains(&peer_id)` and immediately triggers a new `LedgerExchangeRequest` (`core/src/transport/swarm.rs:5249`, `6922`, `7643`), completely bypassing the failover rate limit. 

## V4. Is the memory argument sound?
**Yes.** 
- `FAILOVER_REEXCHANGE_RETENTION` defines a 300-second window. The memory size is bounded by the number of *distinct* `PeerId`s successfully connecting and triggering failovers within 5 minutes.
- Because establishing a connection with a unique `PeerId` is computationally bounded by cryptographic handshakes, an attacker cannot arbitrarily inflate the map to unbounded sizes. 
- Furthermore, the `retain` sweep runs synchronously on *every* call to `allow_failover_reexchange` (`core/src/transport/swarm.rs:818`), meaning the map cannot grow unbounded "between sweeps" since the sweep frequency matches the insertion attempt frequency perfectly.

## V5. Is the replacement test adequate?
**No.** The CTO's assessment is correct; the test is weaker than requested.
- The test `failover_ledger_reexchange_cooldown_persists_across_full_disconnect` (`core/src/transport/swarm.rs:8754`) merely calls `allow_failover_reexchange(peer)` three times in sequence on an isolated `RelayAbuseGuardrails` instance. 
- Because `forget_peer` was removed, the test simulates a "full disconnect" entirely via a code comment, without exercising the actual `SwarmEvent::ConnectionClosed` event loop logic where the defect originally resided.
- **To catch a regression:** A future refactor could silently reintroduce a map-clearing call into `start_swarm_with_config`, and this unit test would still pass. Pinning this behavior requires an integration test that drives a simulated `Swarm` (or directly injects `SwarmEvent::ConnectionEstablished` and `ConnectionClosed` events) to assert that a full disconnect followed by a partial disconnect does not emit a redundant `LedgerExchangeRequest`.

## V6. Anything new.
**No newly introduced defects.** The diff cleanly implements the removal of the cooldown reset mechanism.

---

## Conclusion
Verdict: **APPROVE_WITH_FINDINGS**
W1 CLOSED: YES
REGRESSION RISK: NONE
