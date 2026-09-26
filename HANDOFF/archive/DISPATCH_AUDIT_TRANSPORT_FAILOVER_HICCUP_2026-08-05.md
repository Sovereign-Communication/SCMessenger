# DISPATCH: READ-ONLY AUDIT -- Transport Failover / BLE-LAN Hiccup

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Mode: READ-ONLY AUDIT. Do NOT write code. Do NOT apply diffs. Deliver
analysis only, in the report format at the bottom.

## Problem (operator field observation, 2026-08-05)

During a live bidirectional Android (Pixel 6a) <-> iOS (Christy) session on
the same home network, message flow STOPPED. Restarting the iOS app resumed
flow perfectly. Working theory: only the BLE transport was carrying
traffic; the LAN/WiFi transport never established or failed silently with
no detection, failover, or reconnect -- leaving the session silent until a
process restart rebuilt the connections.

Ticket: HANDOFF/todo/TRANSPORT_BLE_LAN_HICCUP_VERIFICATION_2026-08-05.md

## Attached context (core transport layer)

- manager.rs: transport manager / connection lifecycle
- health.rs: transport health probes
- circuit_breaker.rs: failure circuit breaking
- escalation.rs: transport escalation (presumably BLE -> LAN/WiFi)
- discovery.rs: peer discovery

Platform BLE layers (Android GATT service, iOS CoreBluetooth) live in the
app modules, NOT core. Note where core hands off to them.

## Your task

1. MAP THE TRANSPORT STACK: which transports exist (BLE, WiFi Direct, WiFi
   Aware, LAN TCP/QUIC via internet.rs/multiport.rs, websocket, relay), and
   for a same-LAN Android<->iOS pair, which are expected to be active and
   in what order/priority.
2. KEEPALIVE/DETECTION: for each active transport, is there a heartbeat or
   liveness probe? If a transport dies silently (socket death, BLE bond
   drop, WiFi power-save), what detects it and after how long? Cite
   file:line.
3. FAILOVER PATH: when one transport fails, does the manager redial /
   escalate to another transport automatically? Trace escalation.rs +
   circuit_breaker.rs + dial_policy semantics. Identify any state in which
   ALL transports are considered up by bookkeeping while actually dead
   (the "silent silence" failure mode that an app restart fixes).
4. RESTART-AS-FIX ANALYSIS: what exactly does a process restart rebuild
   that the live node failed to rebuild on its own? That delta is the bug
   surface -- name it.
5. FIX SKETCH: bounded, minimal changes to give (a) bounded-time detection
   of a dead transport, (b) automatic redial/failover without app restart.
   Mark every site touching core/src/transport as AUDIT-GATE (adversarial
   review required per AGENTS.md rule 8 before merge).

## Deliverable

Transport map table; detection/failover gap list with file:line evidence;
the restart-delta bug surface; fix sketch with effort estimates. List any
files you needed but were not provided (e.g. internet.rs, wifi_direct.rs,
app-layer BLE services).

## Report format (mandatory final block)

RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE (read-only audit)
FILES: <files examined>
NOTES: <max 8 lines: primary gap + fix direction + what the orchestrator must run next>
