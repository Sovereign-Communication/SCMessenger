# TASK: Fable 5 sprint review follow-ups (F4, F7, F8)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P2 (review-sanctioned follow-ups; the blocking findings F1/F2/F3/F5/F6
are already fixed and committed). Source: tmp/audit_reports/fable5_sprint_adversarial_review.md.
**Lane:** Qwen-coder generates -> orchestrator applies + gates -> Qwen-thinking
audit -> Fable final. F4 touches AUDIT-GATE transport (swarm.rs).

## F4 (MEDIUM) - startup outcome decided by first listener event of ANY listener
core/src/mobile_bridge.rs ~978-1004: the ListeningOn/ListenerFailed arms signal
start success/failure on the FIRST listener event regardless of address. But the
native swarm also binds always-on QUIC + WebSocket 0.0.0.0:9002 (swarm.rs ~1983/
1992) + relay-circuit listeners. So a QUIC ListenerError can false-fail a healthy
tcp/9001 bind, and a WS/QUIC ListeningOn can false-succeed when tcp/9001 is
EADDRINUSE (the documented CLI-daemon-on-same-host LAN setup).
FIX: thread the REQUESTED listen multiaddr (listen_multiaddr, computed ~656) into
the swarm event loop, and only resolve the startup signal on a listener event
whose address matches the requested one; ignore incidental QUIC/WS/relay listener
events during the 15s startup window. Requires tracing how the spawned swarm
closure captures state (read swarm construction ~660-960 first).

## F7 (LOW) - `tracked` map never evicts (unbounded growth under MAC rotation)
cli/src/ble_mesh.rs ~223-247: entries created per unique peripheral ID, never
removed. BLE privacy addrs rotate ~15min; a 24/7 CLI daemon accretes indefinitely.
FIX: cap size or sweep entries that are active==false, failures==0, or have an
expired cooldown older than N minutes.

## F8 (LOW) - ListenerFailed emitted for benign listener closures
core/src/transport/swarm.rs ~4029/5407 map every ListenerClosed to
SwarmEvent2::ListenerFailed, including deliberate remove_listener of relay-circuit
reservations (~3861). Produces false "[ERROR] listener failed" during normal relay
churn and can fail a healthy start (feeds F4).
FIX: distinguish reason: Ok(()) / self-initiated closes, or tag events with the
listener's addresses.

## GATE (orchestrator runs)
cargo check -p scmessenger-core -p scmessenger-cli; cargo test --workspace
--no-run; gradlew assembleDebug (for any Kotlin touch). AUDIT-GATE (swarm.rs) ->
Fable final re-audit before merge-ready.

---
**IMPLEMENTED 2026-07-07 (Qwen-coder, orchestrator-applied):** before inserting a new key, if `tracked` exceeds 2048 entries, retain only entries that are neither idle-safe (inactive+zero-failures) nor past their cooldown expiry - bounds memory against unbounded growth under BLE MAC rotation. Gate: `cargo check -p scmessenger-cli` PASS.
