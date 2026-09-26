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
# BUDGET: 1200
# token_budget: 12000

# P0_CLI_027_Drift_Protocol_Still_Dormant_At_0_2_1

**Status:** VERIFIED REMAINING WORK (driven by Claude Code 2026-06-04, /api/drift-status)
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P0  closes existing `[VALIDATED]_P1_CORE_001` task
**Source:** Live API test of running binary
**Depends on:** P0_BUILD_001
**Note:** Supersedes part of `[VALIDATED]_P1_CORE_001_Drift_Protocol_Production_Wire.md` (the
production-wire task exists but Drift is still dormant at HEAD `14ea6d61`).

---

## Verified Gap (with reproduction)

```
$ curl -s http://127.0.0.1:9876/api/drift-status
{"state":"Dormant","store_size":0}

$ curl -s http://127.0.0.1:9876/api/diagnostics | jq .drift
{
  "state": "Dormant",
  "store_size": 0
}
```

`/api/drift-status` and `/api/diagnostics` both confirm Drift is dormant with an empty
store. The `[VALIDATED]_P1_CORE_001_Drift_Protocol_Production_Wire` task was queued
months ago but never executed (HEAD `14ea6d61`).

The fields exist (`drift_active: bool`, `drift_engine: Option<RelayEngine>`, `drift_store: MeshStore` in
`core/src/iron_core.rs:131-133`) but are initialized to `false` / `None` and never toggled
to active in any swarm event handler.

## Scope (~80 LoC across 2 files)

This is the **activation** half of the original P1_CORE_001 task  the production wire
is mostly in place; the boot path just needs to enable it.

### Part A: Activate Drift on swarm start (LOC: ~50)

In `core/src/transport/swarm.rs` (or wherever `SwarmHandle::start_swarm` initializes):

```rust
// After the libp2p swarm is built and the event loop is running:
*core.drift_active.write() = true;

// Lazily create the engine on first use
if core.drift_engine.read().is_none() {
    let store = core.drift_store.read().clone();
    let engine = RelayEngine::new(store, core.drift_policy());
    *core.drift_engine.write() = Some(engine);
}
```

Add the corresponding `*core.drift_active.write() = false;` in the swarm-stop path.

### Part B: Expose activation state via the API (LOC: ~30)

In `core/src/transport/swarm.rs` (and the API handler in `cli/src/api.rs`):

```rust
pub fn drift_status(&self) -> DriftStatusResponse {
    DriftStatusResponse {
        state: if *self.core.drift_active.read() {
            "Active".into()
        } else {
            "Dormant".into()
        },
        store_size: self.core.drift_store.read().len() as u64,
    }
}
```

## File Targets

- `core/src/transport/swarm.rs` [EDIT  flip drift_active on swarm start, build RelayEngine]
- `cli/src/api.rs` [VERIFY  `handle_get_drift_status` already exists; just confirm it reads the new fields]

## Build Verification Commands

```bash
cargo check -p scmessenger-core
cargo check -p scmessenger-cli
cargo test -p scmessenger-core --lib drift::
```

## Acceptance Gates

1. After `scmessenger-cli start` then `GET /api/drift-status`, the response is `{"state":"Active",}` (not Dormant)
2. `GET /api/diagnostics` shows `drift.store_size > 0` after one message round-trip
3. `cargo test -p scmessenger-core --lib drift::` passes (the existing unit tests in the
   drift module are all green per the project state memory; the activation wires them up)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: GLM_5.1] [DEPENDS_ON: P0_BUILD_001] [SUPERSEDES: P1_CORE_001]
