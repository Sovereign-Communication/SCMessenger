# TASK: P1-DOCS-T12c  Correct stale T12c finding: WiFi Aware `send()` `false` is a deliberate documented no-op, not a missing write path

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [HAIKU] (docs-only, verbatim-scoped).
**Gates:** none (documentation only; does not touch `core/src/crypto|transport|routing|privacy`).

## Source

P1-15 transport-matrix ground-truth audit (`HANDOFF/plans/P1-15_transport_matrix_audit.md`, sections (a) and (b)), 2026-07-04 [V-READ].

## Problem (exact, verified)

`docs/release-readiness-2026-07-02.md:426-429` records finding T12c:

> (c) `PeerConnection.send()` is now a hard-coded `false` no-op while `sendData`-capable routes still call it  either restore a real write path or unregister WiFi Aware from those routes so delivery fails over loudly, not silently.

This finding is **stale**. The code it refers to  `AwareConnection.send()` at `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt:512-519`  does return `false`, but it is a **deliberate, documented architectural dead-end**, and the "restore a real write path" remedy already exists via a different mechanism:

- `WifiAwareTransport.kt:401-425` `startLoopbackProxy` bridges the real cross-device WiFi Aware socket to a local loopback `ServerSocket` and reports `127.0.0.1:<port>` to `onDataPathConfirmed` (`:420`).
- `AwareConnection.acceptAndPump` (`:446-481`) pumps bytes bidirectionally between the loopback socket and the peer socket.
- The `send()` doc (`:501-511`) and class doc (`:427-437`) state that once the proxy is active, "the raw peer-socket bytes ARE the libp2p connection" and delivery happens via **libp2p dialing the loopback address**, not via `send()`. The `false` is logged at ERROR to keep the structural dead-end visible.
- The confirmation reaches Rust and drives a real libp2p dial: `onDataPathConfirmed` -> `TransportManager.kt:442-443` -> `MeshRepository.kt:895-896` `meshService?.onWifiAwareDataPathConfirmed(...)` -> `core/src/mobile_bridge.rs:1340` -> resolves the `create_data_path` oneshot -> `mobile_bridge.rs:1333` dials `/ip4/127.0.0.1/tcp/<port>`.

So the "delivery fails silently through a false-returning send()" concern no longer applies: `sendData` is *intentionally* out of the WiFi Aware data path, and delivery runs through the loopback proxy + libp2p dial. No "restore write path" code work is required.

## Root Cause

The T12c finding (dated 2026-07-02) predates the loopback-proxy rework of `WifiAwareTransport.kt`. The ledger was not updated when the send-path semantics changed.

## Blast Radius

Documentation only. One ledger entry in `docs/release-readiness-2026-07-02.md`. No source code. No transport/crypto surface -> no audit gate. Must run docs-sync after editing.

## Files to Touch

- `docs/release-readiness-2026-07-02.md` (T12c entry, `:426-429`)  reclassify from "open code fix needed" to "resolved-by-design; residual is a live-device confirmation only". Cross-reference `HANDOFF/plans/P1-15_transport_matrix_audit.md`.
- Any other ledger/status doc that cites T12c as an open WiFi Aware code gap (grep `T12c` repo-wide before editing).

## Verification Commands

```bash
grep -rn "T12c" docs/ HANDOFF/
./scripts/docs_sync_check.sh   # or scripts/docs_sync_check.ps1 on PowerShell
```

## Do NOT

- Do NOT delete the T12c entry  reclassify it (audit trail must show it was investigated and resolved-by-design, per the release-readiness doc's verified-vs-claimed style).
- Do NOT assert the WiFi Aware path is device-verified  it is code-wired and reachable, but live carriage across two Aware radios is still [DEVICE][BLOCKED-HW] (one phone). Say exactly that.
- Do NOT touch `WifiAwareTransport.kt`  the `send()` no-op is correct as documented; the fix is a ledger correction, not a code change.
