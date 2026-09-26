# F2 drift custody persistence audit - findings + MeshStore fix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: VERIFY-FIRST COMPLETE. Custody question answered; a separate real gap
found. Free-lane investigation (agy/Gemini, read-only, no edits).

## F2 core question: does custody survive process death on mobile? YES.
`RelayCustodyStore` (relay custody = messages a node holds to forward for
others, the farm sneakernet-relay path) IS persistent: initialized via
`RelayCustodyStore::persistent(backend)` at `iron_core.rs:307,400,491`, backed
by `SledStorage` opened at an on-disk path (`store/backend.rs:109-115`), with
write-through gets/puts (`store/relay_custody.rs:1094,1106`). It survives an
app kill. The `iron_core.rs:264` in-memory-construction worry did NOT apply to
custody. Farm-critical relay continuity is safe on this axis.

## Separate finding: MeshStore does NOT persist (dead wiring)
`MeshStore` (drift protocol store) is constructed in-memory via
`MeshStore::new()` at `iron_core.rs:313,406,497` - an in-memory HashMap
(`drift/store.rs:90`). Its load/save methods EXIST but are never called in
production - the same dead-on-arrival wiring pattern seen elsewhere this farm
cycle (unused `record_attempt`, unused `on_receipt_received`). So whatever
drift state MeshStore holds is lost on every app kill.

### Determination needed before fixing
Whether this is a real bug depends on what MeshStore holds: if it is drift-sync
state that self-heals by re-syncing from peers on restart, in-memory is
tolerable; if it holds unique local data, losing it on kill is a real bug. The
presence of unused load/save methods strongly implies persistence was INTENDED
(dead wiring), i.e. a bug.

### Minimal fix (from the audit, NOT yet applied)
Pass `backend.clone()` to `MeshStore` (as `RelayCustodyStore::persistent`
already does), load existing keys under the `drift:` prefix on creation, and
write-through on insert/remove. `core/src/drift/store.rs` +
`core/src/iron_core.rs` (3 construction sites). NOT audit-gated (drift/ is not
crypto/transport/routing/privacy) but confirm load-on-startup doesn't corrupt
drift-protocol invariants. Standard compile + drift tests gate.

Not confirmed beta-blocking until the "what does MeshStore hold" determination
is made.
