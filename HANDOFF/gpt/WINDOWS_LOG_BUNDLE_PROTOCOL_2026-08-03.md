# Windows -> GPT: log bundle protocol, all nodes, for cross-side reconstruction

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: ACTION REQUIRED -- needed to close the remaining failures
Date: 2026-08-03
Tier: **GPT-5.4 mini** -- collection and redaction. Mechanical.

Operator wants logs collected from ALL nodes and reconstructed on BOTH sides so
nothing is diagnosed from half the evidence. Windows has collected its half and
dispatched the analysis; this is the matching ask.

## What Windows collected (bundle exists, analysis in flight)

| Source | Size | Notes |
|---|---|---|
| Android logcat | 26,934 lines | Kotlin layer |
| Android Rust core tracing | 1,605 lines | **JSON, new today.** The core was silent on device before PR #132; this is the first time core-internal failures are visible at all |
| Android mesh diagnostics | 647 lines | Timber app log |
| Windows CLI node log | 2,020 lines | separate machine |
| Android bluetooth dumpsys | 2,988 lines | live GATT/advertiser stack state |
| Windows netstat listeners | 54 lines | bind proof |

Reconstruction dispatched to a reasoning-tier model: unified timeline, per-message
end-to-end traces, failure-signature classification.

## What we need from the Mac lane

Collect the equivalent from BOTH your nodes and commit the redacted analysis
(not raw logs) to the repo.

**iOS (Christy's iPhone):**
- the app-container `mesh_diagnostics.log` and any rotated copies
- whatever the iOS equivalent of our Rust core tracing is, if one exists -- if
  iOS does NOT have core-level logging, say so explicitly, because that is
  itself a parity gap worth fixing
- the BLE central markers: `ble_central_connected`,
  `ble_central_services_discovered`, `ble_central_subscribed_message`,
  `ble_central_write_ok`
- any decrypt/crypto failures WITH THEIR EXACT WORDING

**macOS CLI node:**
- the node log
- the listener set, matched to PID

## The five questions we are both answering

Windows is asking its side exactly these. Please answer the same from yours, so
the two reports can be laid side by side:

1. Do decrypt failures correlate with a SPECIFIC peer, or all peers?
2. **Is there evidence of MORE THAN ONE identity/key form in use for the same
   logical peer?** This is the operator's central concern -- one identity must
   work across all transports. If iOS uses a different identity form on BLE than
   on LAN or Multipeer, that is the defect.
3. Does an identity-registration failure PRECEDE the decrypt failures for that
   peer? Ordering separates cause from symptom. Android shows
   `Failed to register local identity with <peer>` against two distinct peers,
   and we need to know whether iOS shows the mirror.
4. Which transports actually carried traffic?
5. Any evidence of a message succeeding end to end, anywhere?

## Redaction convention -- please match it exactly so the reports merge

- peer ids -> `<peer-A>`, `<peer-B>` ... and keep the SAME label for the SAME
  peer throughout. The mapping is analytically load-bearing; do not randomise it.
- 64-char hex keys/hashes -> `<key-1>`, `<key-2>` likewise
- IPs -> `x.x.x.x`
- BLE MACs -> `XX:XX:XX:XX:XX:XX`
- **KEEP message ids (uuids) and timestamps.** Those are the join keys between
  our two reports. Without them the reconstruction cannot be merged.

Commit the redacted ANALYSIS. Do not commit raw logs.

## Where Windows stands

Confirmed on device after PR #132:
- **BLE inbound FIXED**: `mesh_ble_forward` 29 / `mesh_ble_forward_return` 29.
  Was 264 / 0. Transport delivers.
- GATT service `0000DF01` registered with the `2902` CCCD, 3 advertising sets
  active
- Core tracing log live

Still failing, and now clearly ABOVE transport:
- 23x `Failed to decrypt ratchet message: ... wrong key ...`
- 23x `Failed to process received message: CryptoError`
- 3x `Failed to register local identity with <peer>`
- 4x `[WARN] sending to a recipient with no contact record; proceeding`

Also found and root-caused, unrelated to messaging but node-fatal: we handle
`SwarmEvent::ConnectionClosed` per-PEER when libp2p emits it per-CONNECTION, so
the first connection close tears down all peer state while other connections are
live, and libp2p-request-response then panics. `num_established` is never
checked anywhere in swarm.rs. Filed with the fix; not landing it mid-matrix.

## Reply

`HANDOFF/gpt/GPT_RESPONSE_LOG_BUNDLE_2026-08-03.md`, plus the redacted analysis
alongside it.
