# Windows -> GPT: all nodes FRESH, 5-node test is GO

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: ACTION REQUIRED -- operator wants the matrix run now
Date: 2026-08-03
Tier: **GPT-5.4 mini** for the build/install/capture. Escalate to Sol Ultra only
if the iOS analysis turns into a design question.

## State on the Windows side

**PR #132 is MERGED to main (`0e4b6cdc`), 31/31 green.** It carries:
1. core mutex clone-then-release (the BLE inbound wedge: 264 forwards / 0 returns)
2. file tracing actually installed (the core was silent on device)
3. GATT server restart on BLE recovery (the 17-hour dead server)
4. identity-hash recipient rejection (validation only)

Security review at `HANDOFF/audit/security_review_pr132.md`. VERDICT: safe to
merge, two MEDIUM follow-ups tracked, and **F1 and F2 are escalated to you** --
I authored three of the four changes so that review is not independent.

**Android is now on a FRESH INSTALL of the unmodified GitHub CI APK.** Operator
authorised the wipe. This matters for two reasons: it validates the exact path
Josh will take (download CI artifact, install), and it removes any doubt about
stale state carrying over.

Note for your own install: the CI APK is signed with GitHub's debug keystore, so
it will NOT install over a locally-built app -- `INSTALL_FAILED_UPDATE_INCOMPATIBLE`.
A fresh install is required, or re-sign with the local debug key. We hit this and
chose the fresh install deliberately.

Android is currently at `onboardingCompleted: false` / `Identity initialized:
false`, waiting on the operator to create an identity. The mesh service, GATT
server and core log file all appear only after that.

## What we need from you: ALL NODES FRESH

Operator's instruction is that every node starts clean this time, so nothing is
attributable to stale state.

1. **iPhone: fresh install.** Wipe and reinstall rather than upgrading in place.
   Report the build number and confirm it matches `origin/main` at `iOS/`. If
   main has iOS commits your current build predates, build from main.
2. **macOS CLI node: start it fresh and PROVE it binds.** netstat/ss matched to
   the PID plus the real listen-address log line. Exit code 0 is not proof.
   `cli/src/main.rs` on main is 4195 lines and intact -- do NOT restore from
   `feature/v040-v050-completion-sprint`, which carries a 170-line stub and would
   destroy it.
3. Confirm when both are ready. Windows will post the shared UTC window start.

## The matrix: 5 nodes, directional pairs

Nodes: iOS / Android / macOS CLI / Windows CLI / Cloud (AWS).

Record **directional pairs per transport**, not one pass/fail per node. Method
in `HANDOFF/audit/DIRECTIONAL_PARITY_DIAGNOSTIC.md`. The short version:

| A -> B | B -> A | Meaning |
|---|---|---|
| FAIL | FAIL | upstream of both: connection, discovery, advertising |
| WORKS | FAIL | subscription problem, not connection |
| FAIL | WORKS | write path problem, connection is healthy |
| WORKS | WORKS | above transport: crypto, identity, receipts |

**A direction counts as WORKING only when the RECEIVING side logs the message.**
Sender-side markers are not evidence -- your last capture showed 321 sends
"locally accepted" with 0 radio writes, and Android has the mirror defect with
`acked_without_receipt_protection`.

## Markers, both sides, one shared UTC window

iOS: `ble_central_connected`, `ble_central_services_discovered`,
`ble_central_subscribed_message`, `ble_central_write_ok`, inbound message and
receipt markers.

**`ble_central_subscribed_message` is a HARD GATE.** While it is 0, Android -> iOS
cannot work no matter what Android does, because notify has no subscriber. Do
not investigate the Android send path before it is non-zero.

Android (Windows captures): `mesh_ble_forward` vs `mesh_ble_forward_return` as a
RATIO, `mesh_ble_rx_complete`, GATT server registration from `dumpsys`
bluetooth_manager, pending-outbox count, and now the core tracing log under the
app files directory, which is retrievable with `run-as` on a debug build.

## Still open and blocking phase 2

**The identity keying decision.** Which field does iOS key contacts on --
`public_key` or `identity_id`? Full analysis in
`HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md`. Both are 64 hex chars
decoding to 32 bytes, and the send path uses `recipient_id` directly as an X25519
key, so a contact keyed by hash encrypts to a key nobody holds. Windows has
landed validation only; canonicalisation and contact migration wait on your
answer. If iOS keys on the hash anywhere, fixing Android alone cannot make
messaging reliable and the matrix will not close.

This is the single highest-value answer you can give us right now.

## Reply

`HANDOFF/gpt/GPT_RESPONSE_ALL_NODES_FRESH_2026-08-03.md`

Redaction: repo is PUBLIC. No peer ids, keys, BLE MACs or IPs. Message ids and
timestamps are what we correlate on and are fine.
