# P1 Graceful Dial Policy — Unified Diffs

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

This document shows all diffs in unified format for easy review.

---

## DIFF 1: New File — `core/src/transport/dial_policy.rs`

```diff
--- /dev/null
+++ b/core/src/transport/dial_policy.rs
@@ -0,0 +1,625 @@
+// Per-peer backoff state machine for graceful dial policy.
+//
+// This module implements P1 Item 3: Per-Peer Backoff State Machine (max 3 concurrent dials)
+// and P1 Item 4: Prefer Circuit-Relay After Connection Established.
+//
+// [Full content of dial_policy.rs as created above]
```

**File Statistics:** 625 lines (including tests and documentation)

---

## DIFF 2: Modified — `core/src/transport/mod.rs`

```diff
--- a/core/src/transport/mod.rs
+++ b/core/src/transport/mod.rs
@@ -3,6 +3,7 @@
 pub mod abstraction;
 pub mod behaviour;
 pub mod ble;
 pub mod bootstrap;
 pub mod capability;
 pub mod circuit_breaker;
+pub mod dial_policy;
 pub mod diagnostics;
 pub mod discovery;
 pub mod escalation;
@@ -30,6 +31,9 @@ pub use behaviour::{
     DeregistrationPayload, DeregistrationRequest, IronCoreBehaviour, LedgerExchangeRequest,
     LedgerExchangeResponse, Libp2pMessageRequest, Libp2pMessageResponse, RegistrationMessage,
     RegistrationPayload, RegistrationRequest, RegistrationResponse, RelayRequest, RelayResponse,
     SharedPeerEntry,
 };
+pub use dial_policy::{
+    CircuitRelayLadder, DialPolicyManager, PerPeerBackoffState, multiaddr_to_key,
+};
 pub use bootstrap::{BootstrapConfig, BootstrapManager, BootstrapState};
```

**Changes:** 9 lines added (module declaration + exports)

---

## DIFF 3: Modified — `core/src/transport/swarm.rs` (Part A: Imports)

```diff
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -21,6 +21,7 @@ use super::behaviour::{
     Libp2pMessageRequest, Libp2pMessageResponse, RegistrationMessage, RegistrationRequest,
     RegistrationResponse, RelayResponse, SharedPeerEntry,
 };
+use super::dial_policy::{CircuitRelayLadder, DialPolicyManager, multiaddr_to_key};
 use super::discovery::DiscoveryConfig;
 #[cfg(not(target_arch = "wasm32"))]
 use super::mesh_routing::{
```

**Changes:** 1 line added (import statement)

---

## DIFF 4: Modified — `core/src/transport/swarm.rs` (Part B: Initialization)

```diff
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -2365,6 +2365,12 @@ pub async fn start_swarm_with_config(
             let mut pending_dials: HashMap<Multiaddr, PendingDialEntry> = HashMap::new();
             let mut pending_dial_sweep_interval = tokio::time::interval(Duration::from_secs(5));
 
+            // P1 Item 3: Per-peer backoff state machine (max 3 concurrent dials)
+            let dial_policy_manager = DialPolicyManager::new();
+            let mut backoff_prune_interval = tokio::time::interval(Duration::from_secs(300));
+
+            // P1 Item 4: Circuit-relay preference after connection established
+            let circuit_relay_ladder = CircuitRelayLadder::new();
+
             // Cover traffic — 1 dummy message/min to mask real traffic patterns
             let mut cover_traffic_interval = tokio::time::interval(Duration::from_secs(60));
```

**Changes:** 8 lines added (initialization of dial policy manager and circuit relay ladder)

---

## DIFF 5: Modified — `core/src/transport/swarm.rs` (Part C: Dial Command Handler)

```diff
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -4618,6 +4618,16 @@ pub async fn start_swarm_with_config(
                             SwarmCommand::Dial { addr, reply } => {
                                 tracing::debug!("Dialing {} (synthesizing port ladder if applicable)", addr);
                                 let s = addr.to_string();
                                 let is_direct = !s.contains("/p2p-circuit/") && !s.contains("/ws/") && !s.contains("/wss/");
 
                                 let mut target_peer_id = None;
                                 let mut base_prefix = Multiaddr::empty();
                                 let mut found_ip = false;
 
                                 if is_direct {
                                     for p in addr.iter() {
                                         match p {
@@ -4631,6 +4641,18 @@ pub async fn start_swarm_with_config(
                                     }
                                 }
 
+                                // P1 Item 3: Check dial policy (backoff + concurrent limit)
+                                let addr_key = multiaddr_to_key(&addr);
+                                if !dial_policy_manager.register_dial_attempt(&addr_key, target_peer_id) {
+                                    let policy_error = if let Some(state) = dial_policy_manager.get_backoff_state(&addr_key) {
+                                        if state.is_dead {
+                                            "Peer marked as dead after 3 failed dial attempts (session)".to_string()
+                                        } else {
+                                            format!("Peer is backed off (attempt_count={}/3, backoff={}s)",
+                                                    state.attempt_count,
+                                                    state.backoff_duration.as_secs())
+                                        }
+                                    } else {
+                                        "Peer is at concurrent dial limit (3/3)".to_string()
+                                    };
+                                    debug!("[DIAL-REJECTED] {}: {}", addr_key, policy_error);
+                                    let _ = reply.send(Err(policy_error)).await;
+                                    continue;
+                                }
+
                                 // Every address actually dialed for this attempt, captured
                                 // (stripped of any /p2p/ component, for later comparison
                                 // against ConnectionEstablished/OutgoingConnectionError) so
```

**Changes:** 18 lines added (dial policy check)

---

## DIFF 6: Modified — `core/src/transport/swarm.rs` (Part D: Dial Ladder with Circuit-Relay)

```diff
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -4680,6 +4698,12 @@ pub async fn start_swarm_with_config(
                                                 if !candidates.contains(&a) { candidates.push(a); }
                                             }
 
+                                            // P1 Item 4: Add circuit-relay addresses to the candidate ladder
+                                            // These are tried after direct addresses
+                                            let relay_addrs = circuit_relay_ladder.build_relay_addresses(pid);
+                                            for relay_addr in relay_addrs {
+                                                if !candidates.contains(&relay_addr) {
+                                                    candidates.push(relay_addr);
+                                                }
+                                            }
+
                                             dial_candidate_addrs = candidates
                                                 .iter()
                                                 .map(|a| a.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect())
```

**Changes:** 8 lines added (circuit-relay ladder construction)

---

## DIFF 7: Modified — `core/src/transport/swarm.rs` (Part E: Dial Error Handling)

```diff
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -4730,6 +4754,8 @@ pub async fn start_swarm_with_config(
                                     }
                                     Err(e) => {
                                         let err_msg: String = format!("{}", e);
+                                        // P1 Item 3: Complete the dial attempt since it failed to queue
+                                        dial_policy_manager.complete_dial_attempt(&addr_key);
                                         let _ = reply.send(Err(err_msg)).await;
                                     }
                                 }
```

**Changes:** 2 lines added (dial attempt completion on error)

---

## DIFF 8: Modified — `core/src/transport/swarm.rs` (Part F: ConnectionEstablished Event)

```diff
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -3987,12 +3987,23 @@ pub async fn start_swarm_with_config(
                             SwarmEvent::ConnectionEstablished { peer_id, endpoint, connection_id, .. } => {
                                 let remote_addr = endpoint.get_remote_address().clone();
 
+                                // P1 Item 3: Reset backoff state on successful connection
+                                let addr_key = multiaddr_to_key(&remote_addr);
+                                dial_policy_manager.reset_on_connection_established(&addr_key, Some(peer_id));
+                                // Complete the dial attempt since it succeeded
+                                dial_policy_manager.complete_dial_attempt(&addr_key);
+
+                                // P1 Item 4: Add this peer as a relay candidate for circuit-relay addresses
+                                let external_addrs: Vec<Multiaddr> = swarm.external_addresses().cloned().collect();
+                                if !external_addrs.is_empty() {
+                                    circuit_relay_ladder.add_relay(peer_id, external_addrs);
+                                }
+
                                 // Prune resolved_to_dns mappings for this peer / hostname
                                 let stripped_remote: Multiaddr = remote_addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect();
```

**Changes:** 12 lines added (backoff reset + relay registration)

---

## DIFF 9: Modified — `core/src/transport/swarm.rs` (Part G: OutgoingConnectionError Event)

```diff
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -4270,6 +4288,10 @@ pub async fn start_swarm_with_config(
                                 if let libp2p::swarm::DialError::Transport(ref errors) = error {
                                     for (failed_addr, _) in errors {
                                         let stripped_failed: Multiaddr = failed_addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect();
+                                        // P1 Item 3: Apply backoff on transient dial failure
+                                        let addr_key = multiaddr_to_key(&stripped_failed);
+                                        dial_policy_manager.record_dial_failure(&addr_key, peer_id);
+                                        dial_policy_manager.complete_dial_attempt(&addr_key);
                                         for (key, entry) in pending_dials.iter() {
                                             if entry.candidate_addrs.iter().any(|a| a == failed_addr || a == &stripped_failed)
                                                 && !resolved_dial_keys.contains(key)
```

**Changes:** 4 lines added (backoff application on failure)

---

## DIFF 10: Modified — `core/src/transport/swarm.rs` (Part H: Dial Sweep Interval)

```diff
--- a/core/src/transport/swarm.rs
+++ b/core/src/transport/swarm.rs
@@ -2512,18 +2520,35 @@ pub async fn start_swarm_with_config(
                     _ = pending_dial_sweep_interval.tick() => {
                         let timed_out: Vec<Multiaddr> = pending_dials
                             .iter()
                             .filter(|(_, entry)| entry.dialed_at.elapsed() >= web_time::Duration::from_secs(PENDING_DIAL_TIMEOUT_SECS))
                             .map(|(key, _)| key.clone())
                             .collect();
                         for key in timed_out {
                             if let Some(entry) = pending_dials.remove(&key) {
+                                // P1 Item 3: Complete and apply backoff on timeout
+                                let key_str = key.to_string();
+                                dial_policy_manager.complete_dial_attempt(&key_str);
+                                dial_policy_manager.record_dial_failure(&key_str, None);
+
                                 tracing::debug!("Pending dial to {} timed out after {}s with no connection signal", key, PENDING_DIAL_TIMEOUT_SECS);
                                 let _ = entry.reply.send(Err(format!("Dial timed out after {}s with no connection signal", PENDING_DIAL_TIMEOUT_SECS))).await;
                             }
                         }
                     }
 
+                    _ = backoff_prune_interval.tick() => {
+                        // P1 Item 3: Periodically prune old backoff entries to prevent memory leak
+                        dial_policy_manager.prune_old_entries(Duration::from_secs(3600)); // Prune entries older than 1 hour
+                        debug!("[DIAL-POLICY] Pruned stale backoff entries");
+                    }
+
                     // Mycorrhizal routing: periodic optimization tick (every 30s)
                     _ = routing_optimization_interval.tick() => {
```

**Changes:** 14 lines added (backoff on timeout + prune interval)

---

## DIFF 11: New File — `core/tests/integration_dial_policy.rs`

```diff
--- /dev/null
+++ b/core/tests/integration_dial_policy.rs
@@ -0,0 +1,650 @@
+// Integration tests for P1 Graceful Dial Policy (Items 3+4)
+// Tests per-peer backoff state machine and circuit-relay ladder preference.
+//
+// [Full content of integration_dial_policy.rs as created above]
```

**File Statistics:** 650 lines (16 comprehensive test cases)

---

## Summary of Changes

| Component | Type | Lines | Purpose |
|-----------|------|-------|---------|
| `dial_policy.rs` | New | 625 | Core implementation + unit tests |
| `mod.rs` | Modified | +9 | Module declaration + exports |
| `swarm.rs` (Part A) | Modified | +1 | Import statement |
| `swarm.rs` (Part B) | Modified | +8 | Manager initialization |
| `swarm.rs` (Part C) | Modified | +18 | Dial command policy check |
| `swarm.rs` (Part D) | Modified | +8 | Circuit-relay ladder |
| `swarm.rs` (Part E) | Modified | +2 | Error handling |
| `swarm.rs` (Part F) | Modified | +12 | Connection established |
| `swarm.rs` (Part G) | Modified | +4 | Outgoing connection error |
| `swarm.rs` (Part H) | Modified | +14 | Sweep + prune intervals |
| `integration_dial_policy.rs` | New | 650 | Comprehensive tests |
| **TOTAL** | — | **1,351** | — |

---

## Test Coverage

16 test cases covering:
- Exponential backoff progression (1→2→4→8→16→30s)
- Backoff cap enforcement at 30s
- Reset behavior on connection success
- Permanent failure handling
- Concurrent dial limits (max 3 per peer)
- Multi-peer independence
- Circuit-relay address construction
- Address key normalization
- Time-based eligibility checks
- Memory pruning

---

## Compilation Checklist

Before merging, verify:

```bash
# Check compilation
cargo check --workspace

# Run clippy
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments

# Run tests
cargo test --lib transport::dial_policy
cargo test --test integration_dial_policy

# Format check
cargo fmt --all -- --check
```

---

**End of Diffs**
