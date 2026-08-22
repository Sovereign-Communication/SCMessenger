# 5-Node Field Test -- Bilateral Consensus Plan

Status: DRAFT -- AWAITING WINDOWS LANE ACK
Created: 2026-08-22T03:00:00Z
Origin: CAO/Apple lane
Target: CTO/Windows lane

Both lanes must ACK this plan before either lane proceeds with any
implementation. Windows lane ACKs by appending an event to CTO_TO_CAO.md
AND sending [OK-PLAN-ACK] over SCMessenger CLI.

Once bilateral ACK is confirmed by the operator, both lanes auto-proceed.

---

## Problem Inventory (from this session's direct evidence)

### P1 -- iOS <-> Android: no message delivery confirmed (CRITICAL)
- iOS SmartTransportRouter was silently dropping transport candidates when
  blePeerId or routePeerCandidates resolved to nil/empty -- BLE and
  Internet/Relay were conditional, so zero paths were tried.
  Fixed commit daa9e153 (always include BLE + Internet in race).
- Android BleGattClient subscribes CCCD on DF03. iOS BLEPeripheralManager
  was notifying on DF04. Fixed commit b605c58e (dynamic per-central char
  routing using subscribedCharacteristicUuids).
- iOS fix built (BUILD SUCCEEDED) and deployed to iPhone 15 Pro Max
  (C218DC62) at 2026-08-22T02:56Z.
- Android side: NEEDS fresh logcat to confirm DF03 CCCD subscription and receipt.

### P2 -- Single conversation thread (identity key unification)
- Windows/Android: commit daab8a2b on feat/identity-id-unification.
- iOS/macOS: commit 24bd6e91 on gpt/v050-parity-burndown (PR #208).
- Needs bilateral end-to-end verification: same contact -> one thread.

### P3 -- AWS cloud node (N0) not on latest build (BLOCKER for P5)
- Single AWS instance confirmed: 54.226.67.101 (i-006b14491d421bd0d).
  Health 200 OK on :9876. Container binary is behind unification commits.
- Action REQUIRED by Windows lane: SSH ec2-user@54.226.67.101 and rebuild
  to daab8a2b. Verify health after restart. Report hash + peer-id.

### P4 -- macOS CLI BLE: scanning but no identity handshake completed
- Daemon sees DF01 advertisements from two peripherals continuously.
  No DF02 identity read -> DF03 exchange logged this session.
- Needs: BLE connect -> DF02 read -> message round-trip with N2 or N4.

### P5 -- Cellular WAN / CGNAT routing not verified post-unification
- Phones on LTE (Wi-Fi off) must route via cloud node store-and-forward.
- Blocked on P3 (AWS node updated) and P2 (single key).

---

## Bilateral Execution Plan

### Ownership

| Item | Owner | Branch | PR |
|---|---|---|---|
| P1a: Android logcat + DF03 CCCD confirm | Windows/Android (CTO) | feat/identity-id-unification | Windows PR (TBD) |
| P1b: iOS BLE DF03 fix deployed | Apple (CAO) DONE | gpt/v050-parity-burndown | #208 |
| P2: E2E single thread verification | Both | Both | #208 + Windows PR |
| P3: AWS node rebuild to daab8a2b | Windows (CTO) | deployment | N/A |
| P4: macOS CLI BLE identity smoke test | Apple (CAO) | gpt/v050-parity-burndown | #208 |
| P5: Cellular WAN smoke test | Both jointly | Both | Both PRs |

### Execution Sequence (auto-proceeds after bilateral ACK)

STEP 1 [Windows, immediate]: Pull Android Pixel 6a logcat.
  Filter: adb logcat -d -s BleGattClient BleGattServer MeshRepository
  Share extract via SCMessenger CLI + append to CTO_TO_CAO.md.
  Confirm: does Android log "enableNotification" on DF03 for connected iOS?

STEP 2 [Windows, immediate]: Update AWS cloud node.
  ssh -i ~/.ssh/scm-node-key.pem ec2-user@54.226.67.101
  Rebuild/pull Docker image to commit daab8a2b.
  Verify: curl http://54.226.67.101:9876/health -> 200.
  Report binary hash + peer-id in journal + SCMessenger CLI.

STEP 3 [Apple, immediate]: Smoke test macOS CLI <-> iOS BLE (Wi-Fi off).
  Watch CLI logs for DF01 scan -> DF02 identity read -> DF03 message.
  Report [STEP-3-STATUS: OK|FAIL + evidence] over SCMessenger CLI.

STEP 4 [Both, after STEP 1-3 green]: Full 5-node matrix test.
  Pass 1: All 5 nodes appear in each other's peer list.
  Pass 2: 1:1 messages on all 10 pairs; exactly 1 thread each.
  Pass 3: Offline BLE (N2 <-> N4; Wi-Fi off both devices).
  Pass 4: Cellular WAN (LTE only; route via AWS N0).
  Pass 5: Delivery receipts + telemetry verified on all 5 nodes.

### GitHub / PR Discipline

- Mac lane: gpt/v050-parity-burndown -> PR #208 only. No extra PRs.
- Windows lane: feat/identity-id-unification -> Windows working PR.
  Windows lane to confirm PR number via SCMessenger CLI.
- Coordination journal commits: commit immediately after each append.
- No merges to main without explicit operator approval.
- No cherry-picks across lane boundary without a journal event.

### Communication Protocol

- Primary: SCMessenger CLI (macOS: 12D3KooWGxEc3 / Windows: 12D3KooWD6vZ).
- Durable backup: HANDOFF/coordination/apple-windows/ journals.
- Message prefix format: [STEP-N-STATUS] for machine-parseable tracking.

---

## Readiness Gate (both lanes tick before 5-node execute)

[ ] Windows ACK this plan in CTO_TO_CAO.md + SCMessenger CLI
[ ] Apple ACK this plan in CAO_TO_CTO.md + SCMessenger CLI
[ ] P3: AWS node updated and health green (Windows owns)
[ ] P1a: Android logcat confirms DF03 CCCD subscription (Windows owns)
[ ] P1b: iOS BLE deployed + DF03 receive confirmed (Apple -- DONE)
[ ] P4: macOS CLI BLE identity round-trip confirmed (Apple owns)
[ ] All 5 nodes show >=1 peer (mesh confirmed)
[ ] Operator approval -> both lanes execute Pass 2-5 without pause
