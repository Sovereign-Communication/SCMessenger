# Task C-05

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Description
P1-14 hostile-network test: lossy/NAT-ed LAN scenario in farm harness (+200 T)

## Implementation Instructions
Implement the changes described above.

**CRITICAL FORMATTING REQUIREMENT**:
You MUST format your responses exactly like this:
The exact filename must be the FIRST LINE inside the code block:
  // path/to/file.ext
followed immediately by the full file content.

## Target Files
- core/src/transport/nat.rs
- core/src/transport/diagnostics.rs
- android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt
- android/app/src/main/java/com/scmessenger/android/transport/TransportHealthMonitor.kt
- iOS/SCMessenger/SCMessenger/Services/MeshBackgroundService.swift
- iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift

## SCOPING NOTE (2026-07-19)

This ticket is a thin, cross-platform stub (Rust + Kotlin + Swift, 6 target
files, no concrete design) -- not safe to blind-dispatch to Qwen as-is (would
likely produce plausible-looking but disconnected code across 3 languages
with no shared test harness tying it together).

Found existing PARTIAL infrastructure: `docker/docker-compose.network-test.yml`
(170 lines) already defines a relay + Alice-behind-NAT topology with real
`tc qdisc`/`netem` bandwidth (10mbit) and latency (50ms) injection via
`cap_add: NET_ADMIN` -- this is real, usable hostile-network simulation. BUT
it is not referenced anywhere else in `docker/` or `HANDOFF/` (grep confirms
zero call sites) -- it's orphaned, not wired into `run-integration-tests.sh`
or any CI job, and has no actual pass/fail assertion tied to it.

Real next step before dispatch: (1) decide what "test passes" means
concretely (e.g. message delivery succeeds within N seconds despite the
netem-injected loss/latency/bandwidth cap -- reuse the same delivery-receipt
mechanism other farm tests check), (2) wire the existing compose file into
`run-integration-tests.sh` (or a sibling script) with that assertion, (3)
only THEN decide whether `core/src/transport/nat.rs`/`diagnostics.rs` or the
Android/iOS files actually need code changes, or whether existing NAT/
diagnostics code already handles this and only the TEST HARNESS is missing
(similar to how D-01 turned out to need zero code changes). Not attempted
this session -- deferred pending this design step, not because it's
unimportant but because dispatching without it risks the same
plausible-but-wrong output pattern seen elsewhere with loosely-scoped Qwen
tasks.
