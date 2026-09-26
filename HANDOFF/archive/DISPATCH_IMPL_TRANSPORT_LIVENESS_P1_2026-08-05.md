# DISPATCH: IMPLEMENT -- Transport Liveness Phase 1 (No Wire Changes)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Branch: fix/transport-liveness-failover-2026-08-05
Audit basis: HANDOFF/review/TRANSPORT_FAILOVER_AUDIT_QWENPAID_2026-08-05.md
Ticket: HANDOFF/todo/TRANSPORT_BLE_LAN_HICCUP_VERIFICATION_2026-08-05.md

Mode: IMPLEMENTATION (unified diffs). Do not apply anything yourself; the
orchestrator reviews and applies. Output diffs ONLY for the files listed in
SCOPE.

## Problem

Silent transport death (e.g., iOS backgrounds and the OS suspends BLE
without a disconnect callback) leaves zombie entries in
`transports[].connected_peers`; `tick()` prunes `peer_last_seen` but never
reconciles transport state and never emits `PeerDisconnected`, so the
reconnection queue is never populated and message flow halts until an app
restart. Full chain in the audit report section 3.

## SCOPE (exactly these files)

- core/src/transport/manager.rs
- core/src/transport/health.rs
- core/src/transport/escalation.rs
- core/src/transport/mod.rs (only if wiring requires)

Tests: add/extend unit tests in the same files' `#[cfg(test)]` modules (or
core/tests/ if the existing pattern requires an integration test).

## REQUIRED CHANGES (Phase 1 -- bounded, no wire protocol changes)

1. TICK STATE RECONCILIATION: in `TransportManager::tick()` (around
   manager.rs:478-490), when a peer is pruned from `peer_last_seen` for
   staleness, ALSO remove it from every `transports[t].connected_peers` and
   from `peer_transports`. No zombie entries may survive a prune.
2. SYNTHETIC DISCONNECT ON STALENESS: when tick() prunes a stale peer that
   was still listed as connected, emit the EXISTING `PeerDisconnected`
   event variant through the manager's normal event path so the existing
   reconnection queue + downstream consumers fire unchanged. Use the
   existing staleness window; add NO new wire messages and NO new protocol
   behavior.
3. HEALTH-MONITOR HOOK: give `TransportManager` a path to consult
   `TransportHealthMonitor` during tick(): connections reported unhealthy
   (`get_unhealthy_connections`) are treated as stale per (2). Keep it a
   pull-based check in tick(); do not restructure the health monitor.
4. DEESCALATION ON SYNTHETIC DISCONNECT: when the synthetic disconnect
   fires for a peer, call `EscalationEngine::deescalate()` for that peer's
   transport so a fallback transport is preferred while reconnection
   proceeds (escalation.rs:134 currently has zero automatic callers).
5. TESTS: unit tests proving (a) stale prune also clears connected_peers
   across all transports, (b) synthetic PeerDisconnected is emitted on
   stale prune of a connected peer, (c) unhealthy connections from the
   health monitor trigger the same path, (d) deescalation is invoked.

## HARD CONSTRAINTS

- NO wire format changes, NO new protocol messages, NO changes to event
  enum variants (emit existing ones only).
- NO behavioral change for healthy, active connections.
- Preserve existing public API; if a signature must change, keep it
  backward compatible or list it explicitly in the report.
- No emojis anywhere. Match existing code style and comment conventions.
- This code is AUDIT-GATE (AGENTS.md rule 8): an adversarial review will
  run on your diff before merge. Do not take shortcuts that a reviewer
  would flag (no unwrap on peer-controlled data, bounded loops, no
  unbounded channels).

## Report format (mandatory final block)

RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE|CONTAINER(exact commands run)
FILES: <files with diffs>
NOTES: <max 8 lines: what changed, any API impact, what the orchestrator must run>
