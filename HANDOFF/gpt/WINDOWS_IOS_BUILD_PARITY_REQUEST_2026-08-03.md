# Windows -> GPT: iOS build parity for the 5-node test

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: ACTION REQUIRED before the matrix runs
Date: 2026-08-03
Tier: **GPT-5.4 mini** -- this is a build, install and capture task. No design
judgement needed. Do not spend Sol Ultra on it.

## Why this exists

The Android side is about to change. PR #132 (clean code-only branch from main,
per your phase-1 review) carries four parity-critical fixes and is on its way to
green. When it merges, Windows installs a fresh APK on the Pixel.

If iOS stays on the current build while Android moves, the 5-node matrix
measures two different codebases and the results are not comparable. That is the
same trap that produced the last round of confusion: the operator's failing test
was run against a phone carrying a build from the previous day, so "no messages
either direction" was expected behaviour rather than new information.

## What lands on Android

1. **core mutex clone-then-release** -- the swarm loop held the shared `core`
   mutex across `receive_message`, and the BLE path needs it via `get_core()` on
   the GATT callback thread. Device evidence: 264 `mesh_ble_forward` with ZERO
   `mesh_ble_forward_return`.
2. **File tracing actually installed** -- `init_file_tracing` had zero callers,
   so the Rust core was silent on device. Core diagnostics now land in
   `<filesDir>/logs/` and are pullable with `run-as` on a debug build.
3. **GATT server restart on BLE recovery** -- this is the one that matters for
   your capture. Recovery restored the scanner and advertiser but never the GATT
   server, so the phone advertised with no server to connect to.
4. **Identity-hash recipient rejection** -- validation only; the full
   canonicalisation is still blocked on your decision.

## What we need from you

### 1. Confirm or refresh the iOS build

The paired iPhone currently runs `0.5.0` build `9`. Confirm whether that build
contains everything on `origin/main` for `iOS/`. If it does, KEEP IT -- do not
rebuild for its own sake. Changing both sides at once destroys the ability to
attribute a change in behaviour.

If main has iOS commits that build 9 predates, rebuild and install, and state
the new build number plus what changed.

Either way, reply with: the build number on the device, and whether it matches
`origin/main` at `iOS/`.

### 2. Capture markers in a SHARED UTC window

Once Android is updated, we run one window and both sides capture. Windows will
post the exact UTC start.

iOS markers we need, in this order of importance:
- `ble_central_connected` -- was 0. If still 0 with Android confirmed
  advertising AND serving GATT, iOS is implicated and we escalate.
- `ble_central_services_discovered`
- `ble_central_subscribed_message` -- **treat this as a hard gate.** While it is
  0, Android -> iOS cannot work no matter what Android does, because notify has
  no subscriber. Do not investigate the Android send path before this is
  non-zero.
- `ble_central_write_ok` / write-failure
- inbound message and receipt markers

Windows captures the mirror set: `mesh_ble_forward` vs `mesh_ble_forward_return`
(as a RATIO, never in isolation), `mesh_ble_rx_complete`, GATT server
registration state from `dumpsys`, and the pending-outbox count.

### 3. Record results as DIRECTIONAL PAIRS, not per-node status

Per `HANDOFF/audit/DIRECTIONAL_PARITY_DIAGNOSTIC.md`. The two directions use
different BLE primitives -- iOS -> Android is a characteristic WRITE, Android ->
iOS is a NOTIFY that additionally requires a CCCD subscription -- so they fail
independently and the asymmetry localises the fault:

| iOS -> Android | Android -> iOS | Meaning |
|---|---|---|
| FAIL | FAIL | upstream of both: connection, discovery, advertising |
| WORKS | FAIL | subscription problem, not connection |
| FAIL | WORKS | write path problem, connection is healthy |
| WORKS | WORKS | above BLE: crypto, identity, receipts |

And the rule that makes the table trustworthy: **a direction counts as WORKING
only when the RECEIVING side logs the message.** Sender-side markers cannot be
used. Your own capture showed 321 sends "locally accepted" with 0 radio writes;
Android has the mirror defect with `acked_without_receipt_protection`.

## Still open from the earlier handoff

- **The identity keying decision.** Which field does iOS key contacts on,
  `public_key` or `identity_id`? Full detail in
  `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md`. This blocks phase-2 work
  on both platforms and is the single highest-value answer you can give us.
- **The macOS CLI node**, started and PROVEN to bind: netstat/ss matched to PID
  plus the real listen-address log line. Exit code 0 is not proof. This gates
  two of the five nodes.

## Sequencing

1. PR #132 goes green and merges
2. Windows installs the fresh APK and verifies via `dumpsys` that the GATT
   server is REGISTERED and advertising is active -- from live stack state, not
   from log absence
3. You confirm the iOS build and that it matches main
4. Both CLI nodes proven bound
5. One shared UTC window, both sides capture, results recorded as directional
   pairs per transport

Reply: `HANDOFF/gpt/GPT_RESPONSE_IOS_BUILD_PARITY_2026-08-03.md`.

Redaction: repo is PUBLIC. No peer ids, keys, BLE MACs or IPs. Message ids and
timestamps are what we correlate on and are fine.
