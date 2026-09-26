<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: glm-5.1:cloud
# BUDGET: 3600
# token_budget: 36000

# P1_CORE_001_Drift_Protocol_Production_Wire

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 3600s (LARGE tier)
**Phase:** v0.2.1 P1 wire dormant modules
**Source:** HANDOFF/AGENT_HANDOFF_GUIDANCE.md (Drift Protocol COMPLETELY DORMANT) + planfromclaudeforhermes 2 Phase C.1
**Depends on:** P0_BUILD_001 (test gate must be green)
**Blocks:** All other Phase C tasks (this is the transport-core wire; others build on top)

---

## Verified Gap

Per `HANDOFF/AGENT_HANDOFF_GUIDANCE.md`: "CRITICAL: Drift Protocol COMPLETELY DORMANT - 8 implemented files, zero production integration. Using legacy bincode format instead of optimized DriftEnvelope/DriftFrame. No compression - LZ4 compression available but not used. SyncSession never triggered - PeerDiscovered events don't activate sync."

Drift module: 10 files in `core/src/drift/` (compress, envelope, frame, mod, policy, rate_limit, relay, sketch, store, sync). All unit-tested, none called from `swarm.rs` or `iron_core.rs` production paths.

`SwarmHandle` currently uses legacy bincode envelope. The migration to DriftEnvelope/DriftFrame touches the hot path of message dispatch.

## Scope (~400 LoC across 4 files)

### Part A: Replace bincode with DriftEnvelope in SwarmHandle send (LOC: ~150)

In `core/src/transport/swarm.rs` (5144 lines, primary wiring target):

Find current message dispatch (search for `bincode::serialize` or `encode_envelope`):
- Replace with `drift::envelope::DriftEnvelope::from_message(msg, &policy)`
- Apply LZ4 compression via `drift::compress::lz4_encode()` before envelope wrap (this is C5 but inline here since C1 is the natural locus)
- Set envelope fields: timestamp, sender, recipient, ttl, priority, fragment_hint
- Pass through `PolicyEngine` check before sending

### Part B: Wire DriftFrame for fragmented messages (LOC: ~80)

In `core/src/transport/swarm.rs`:
- When message > 16KB, split via `drift::frame::DriftFrame::split(message, mtu)`
- Reassemble on receive via `drift::frame::DriftFrame::reassemble(frames)`
- Track in-flight frame sets in swarm state

### Part C: Trigger SyncSession on PeerDiscovered (LOC: ~100)

In `core/src/transport/swarm.rs` `on_peer_discovered()`:
- New peer discovered  spawn `drift::sync::SyncSession::start(peer_id)`
- SyncSession uses IBLT sketch to compute set reconciliation with peer's known messages
- Bidirectional: pull missing messages from peer, push our missing from them
- Audit-log the sync initiation (reuse audit_log_entry! from P0_SECURITY_008)

In `core/src/drift/sync.rs`:
- Verify `SyncSession` exists with `start()`, `apply_delta()`, `complete()` methods (per ledger, present at line 560 with `test_sync_large_symmetric_difference` ignored due to IBLT capacity)
- If capacity is still limited, increase IBLT size (current is 2x; per AGENT_HANDOFF_GUIDANCE line 12: capacity 2x4x+8 was already done)

### Part D: Wire PolicyEngine into Drift (LOC: ~70)

In `core/src/drift/mod.rs`:
- On `apply_policy_config(policy)`, update rate limit, retention, routing hints
- Called from `IronCore::start()` after policy load

In `core/src/iron_core.rs`:
- Add `apply_drift_policy()` call to `start()` after `*running = true`
- Add `apply_drift_policy()` call to `routing_tick()` if policy changes mid-flight

## File Targets

- `core/src/transport/swarm.rs` [EDIT  primary wire site, ~250 LoC across multiple functions]
- `core/src/drift/envelope.rs` [EDIT  verify DriftEnvelope API surface; may need new fields]
- `core/src/drift/frame.rs` [EDIT  verify frame API; add if missing]
- `core/src/drift/sync.rs` [EDIT  ensure start/apply_delta/complete exist]
- `core/src/drift/mod.rs` [EDIT  apply_policy_config integration]
- `core/src/drift/policy.rs` [EDIT  verify policy types]
- `core/src/iron_core.rs` [EDIT  start() and routing_tick() hook]
- `core/src/transport/manager.rs` [EDIT  pass through drift events to swarm]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib drift
cargo test -p scmessenger-core --lib transport
cargo test --workspace --no-run

# Run integration: drift-aware send
cargo test -p scmessenger-core --test integration_all_phases -- --nocapture drift

# CLI smoke: send a message, expect DriftEnvelope in core log
cargo run -p scmessenger-cli -- daemon &
sleep 2
cargo run -p scmessenger-cli -- send "test message to myself"
# Should see "DriftEnvelope dispatched" in daemon log
grep "DriftEnvelope" /e/.hermes/logs/daemon-*.log
```

## Acceptance Gates

1. `cargo test --workspace` passes
2. New tests cover: DriftEnvelope roundtrip, frame split+reassemble, SyncSession start+complete, PolicyEngine hot-swap
3. `grep "DriftEnvelope" core/src/transport/swarm.rs` returns  3 hits (not just unit tests)
4. `grep "SyncSession::start" core/src/transport/swarm.rs` returns  1 hit
5. `grep "bincode::serialize" core/src/transport/swarm.rs` returns 0 hits (legacy fully replaced)
6. Manual: Android  CLI message delivery triggers "DriftFrame dispatched" log entry
7. Commit: `feat(wire): v0.2.1 Drift Protocol  production envelope, frame split, sync on PeerDiscovered`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

This is a **serial, transport-core task**. Do not parallelize with other transport edits. After this lands, C2-C5 can run in parallel.

## Routing Tags
[REQUIRES: RUST_CORE] [REQUIRES: GLM-5.1] [DEPENDS_ON: P0_BUILD_001] [SERIAL] [CROSS_PLATFORM_IMPACT]
