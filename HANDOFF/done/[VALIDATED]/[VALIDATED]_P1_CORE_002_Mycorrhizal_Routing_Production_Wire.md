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

# P1_CORE_002_Mycorrhizal_Routing_Production_Wire

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 3600s (LARGE tier)
**Phase:** v0.2.1 P1 wire dormant modules
**Source:** HANDOFF/backlog/P1_CORE_002_Mycorrhizal_Routing_Activation.md (dormant) + planfromclaudeforhermes 2 Phase C.2
**Depends on:** P1_CORE_001 (must run after Drift wire  uses DriftEnvelope for routing decisions)

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` Module Status Matrix: "Routing:  Dormant,  Not wired".

Routing module: 12 files in `core/src/routing/` totaling ~5,170 LoC (adaptive_ttl, engine, global, local, multipath, negative_cache, neighborhood, optimized_engine, reputation, resume_prefetch, smart_retry, timeout_budget). All unit-tested.

Per `HANDOFF/ACTIVE_LEDGER.md` "Wired" column, `optimized_engine` is partially wired (it delegates to multipath + reputation which have callers in `iron_core.rs`). But the swarm's actual message send path bypasses it  uses direct or relay instead of `OptimizedRoutingEngine::route()`.

## Scope (~350 LoC across 5 files)

### Part A: Route through OptimizedRoutingEngine (LOC: ~150)

In `core/src/transport/swarm.rs` `send_message()`:

Currently: `swarm.behaviour_mut().send_d direct(peer)` or via relay
Replace with:
```rust
fn send_message(&mut self, dest: PeerId, envelope: DriftEnvelope) -> Result<MessageId> {
    let route = self.routing_engine.route(
        &dest,
        &self.peer_table,
        &self.reputation,
        &self.negative_cache,
    )?;
    match route.transport {
        Transport::Direct(peer) => self.behaviour_mut().send_direct(peer, envelope),
        Transport::Relay(relay_id) => self.behaviour_mut().send_via_relay(relay_id, envelope),
        Transport::Multipath(paths) => {
            // send via multiple paths in parallel, await first success
            for path in paths {
                self.behaviour_mut().send_via(path, envelope.clone());
            }
        }
    }
    // ... track with message_id, register in routing_engine for feedback
}
```

### Part B: Wire negative cache into dispatch (LOC: ~70)

In `core/src/transport/swarm.rs`:
- Before attempting any send, check `routing_engine.negative_cache().is_unreachable(peer, since=now-5min)`
- If unreachable, fail fast and trigger smart_retry
- On successful delivery, clear from negative cache

### Part C: Wire adaptive TTL (LOC: ~50)

In `core/src/transport/swarm.rs` and `core/src/routing/adaptive_ttl.rs`:
- Use `calculate_dynamic_ttl(message_priority, peer_distance)` instead of static TTL
- `calculate_dynamic_ttl` already at `adaptive_ttl.rs:150` per ACTIVE_LEDGER; verify it's actually called from `send_message` (per ledger: 3 call sites in iron_core.rs  add one in swarm.rs)

### Part D: Wire reputation into routing decisions (LOC: ~50)

In `core/src/routing/optimized_engine.rs` `route()`:
- Weight transport selection by `peer_reputation(peer)` from `reputation.rs:64`
- Exclude peers with reputation < threshold from multipath selection
- Auto-mark paths failed via `mark_path_failed()` (already at multipath.rs:119 per ledger)

### Part E: Wire resume_prefetch for offlineonline (LOC: ~30)

In `core/src/iron_core.rs` `start()`:
- Call `routing_engine.start_refresh()` (resume_prefetch.rs:78 per ledger)
- On `PeerDiscovered`, check `is_prefetch_complete()` and resume if needed

## File Targets

- `core/src/transport/swarm.rs` [EDIT  primary wire, ~200 LoC]
- `core/src/routing/optimized_engine.rs` [EDIT  reputation weighting in route()]
- `core/src/routing/adaptive_ttl.rs` [EDIT  already mostly wired, add swarm.rs call]
- `core/src/routing/negative_cache.rs` [EDIT  swarm.rs dispatch check]
- `core/src/routing/multipath.rs` [EDIT  already wired via optimized_engine, verify path]
- `core/src/iron_core.rs` [EDIT  start() hook for resume_prefetch]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib routing
cargo test -p scmessenger-core --lib transport
cargo test --workspace --no-run

# Integration: routing decisions observable
cargo test -p scmessenger-core --test integration_mycorrhizal_routing -- --nocapture

# CLI smoke: send through multi-transport scenario
cargo run -p scmessenger-cli -- daemon &
sleep 2
RUST_LOG=core::routing=info cargo run -p scmessenger-cli -- send "test multi-route"
# Should see "RouteDecision: direct+relay+multipath" in log
grep "RouteDecision" /e/.hermes/logs/daemon-*.log
```

## Acceptance Gates

1. `cargo test --workspace` passes
2. New tests cover: route() returns expected transport for given peer+reputation+cache, negative cache blocks unreachable peers, adaptive TTL scales with priority, multipath selection excludes low-rep peers
3. `grep "OptimizedRoutingEngine::route\|optimized_engine.route" core/src/transport/swarm.rs` returns  1 hit
4. `grep "negative_cache.is_unreachable" core/src/transport/swarm.rs` returns  1 hit
5. `grep "calculate_dynamic_ttl" core/src/transport/swarm.rs` returns  1 hit (in addition to existing iron_core.rs hits)
6. Manual: CLI daemon log shows routing decisions per message
7. Commit: `feat(wire): v0.2.1 Mycorrhizal Routing  route through OptimizedRoutingEngine, adaptive TTL, negative cache, reputation-weighted multipath`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST_CORE] [REQUIRES: GLM-5.1] [DEPENDS_ON: P1_CORE_001] [PARALLEL_WITH: P1_CORE_003, P1_CORE_004, P1_PLATFORM_001]
