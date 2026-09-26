# NEXT_ITER_03: Docs Sync + Sprint Residual Debt Cleanup

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P2 (after NEXT_ITER_01/02 pass)
**Recommended worker:** haiku for section A; sonnet for section B
**Source:** Fable 5 session 2026-07-05/06 handoff

## A. Docs sync (haiku-suitable, mechanical)

1. Run `"C:\Program Files\Git\bin\bash.exe" scripts/docs_sync_check.sh`
   (or `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/docs_sync_check.ps1`).
2. Fix mechanical failures (Status/Last-updated headers, broken links) directly.
3. Verify `REMAINING_WORK_TRACKING.md` and `docs/CURRENT_STATE.md` mention the
   Fable 5 sprint completion note dated 2026-07-05/06; if the sprint commit
   landed without them, add a short dated entry (facts are in
   `HANDOFF/done/FABLE_5_COMPREHENSIVE_AUDIT.md`).

## B. Residual debt from the async-FFI migration (sonnet)

The sprint intentionally left three MeshRepository wrappers as bounded
`runBlocking` bridge queries because their callers are deep sync UI paths:

- `getTopics()`  caller: `TopicManager.refreshKnownTopics()` (plain fun)
- `getExternalAddresses()` / `getListeningAddresses()`  callers:
  `updateBleIdentityBeacon()`, `setIdentityBeaconInternal()`,
  `encodeMeshMessagePayload()`, `getIdentityExportString()` (all plain funs)

Task: suspend-ify or cache. Preferred design (pick per call site):
1. Give `TopicManager` a scope; make `refreshKnownTopics()` launch internally
   and update its `_knownTopics` StateFlow asynchronously (no signature change).
2. For the address getters, maintain @Volatile snapshots refreshed on
   `ListeningOn`/peer events (mirror the `dataStoreNicknameSnapshot` pattern
   added to MeshRepository in this sprint), then delete the runBlocking calls.

Bounds: do NOT convert ViewModel/UI functions to suspend for this; the point is
removing runBlocking without a UI-wide cascade. If that proves impossible,
document why and leave the bounded runBlocking (it matches pre-sprint blocking
behavior, so it is not a regression).

Gates afterward: `cd android && ./gradlew :app:assembleDebug :app:testDebugUnitTest -x lint`.

## Outcomes

- **Section B Completed**: Removed `runBlocking` calls from `MeshRepository.kt` getters (`getExternalAddresses` and `getListeningAddresses`) by introducing volatile snapshot variables. Added a background update helper `refreshAddressesSnapshots()` triggered periodically by stats update and on peer connection/identify events.
- **TopicManager Asynchronous Refresh**: Added a local `CoroutineScope` to `TopicManager.kt` and wrapped the `refreshKnownTopics()` logic in `scope.launch` to make topic list updates asynchronous without changing the public function signature.

## Completion

Update this file with outcomes, move to `HANDOFF/done/`, commit
`native: completed NEXT_ITER_03_Docs_Sync_And_Residual_Debt`.
