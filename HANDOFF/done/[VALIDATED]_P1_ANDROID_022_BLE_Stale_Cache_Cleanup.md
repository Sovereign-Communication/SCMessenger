<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision  2026-06-08

**Status:** SHIPPED
**Bucket:** done (on integration/v0.2.2-pre-android-push-2026-06-05)
**Commit SHA:** 0fa8dea8 (same commit as P0_024; both fixes in one commit, merged at 23174061)
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Fix shipped in commit `0fa8dea8` (file:
`android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`,
11-line `clearPeerCache()` call in `stopScanning()`). Per
`HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md` the build is
green and tests pass. This ticket should be moved to `HANDOFF/done/`.

The companion test file `android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerTest.kt`
was created by gemini on 2026-06-05 20:17 PT.

---

# MODEL: qwen3-coder-next:cloud
# BUDGET: 600
# token_budget: 6000

# P1_ANDROID_022_BLE_Stale_Cache_Cleanup

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 600s (MICRO tier)
**Phase:** v0.2.1 P1 Android stability
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md 1.2 (gratuitous nearby entries persistence) + planfromclaudeforhermes 2 Phase D.4
**Depends on:** P0_BUILD_001

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` 1.2: "Fix gratuitous nearby entries persistence (stale peer cache after discovery stop)".

Per `HANDOFF/WIRING_TASK_INDEX.md` row 50: `task_wire_clearPeerCache` is one of 350 wiring tasks. `BleScanner.clearPeerCache()` exists in code per wiring manifest, but unit test coverage is missing.

## Scope (~40 LoC across 1-2 files)

### Part A: Verify clearPeerCache wiring (LOC: ~10)

In `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`:

Find `clearPeerCache()` (per WIRING_PATCH_MANIFEST row 50). Verify it's called from:
- `onDiscoveryStop()` (or `stopScanning()`)
- `onTransportPause()` (when user backgrounds the app)
- `clearAllRequestNotifications` flow

If not already called, add the call(s).

### Part B: Add unit test (LOC: ~30)

In `android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerTest.kt` (NEW if doesn't exist):

```kotlin
@Test
fun clearPeerCache_removesAllDiscoveredPeers() {
    val scanner = BleScanner(mockContext)
    scanner.onPeerDiscovered(peer1)
    scanner.onPeerDiscovered(peer2)
    assertEquals(2, scanner.discoveredPeers.size)
    
    scanner.clearPeerCache()
    
    assertEquals(0, scanner.discoveredPeers.size)
}

@Test
fun onDiscoveryStop_callsClearPeerCache() {
    val scanner = BleScanner(mockContext)
    scanner.onPeerDiscovered(peer1)
    scanner.onDiscoveryStop()
    assertEquals(0, scanner.discoveredPeers.size)
}
```

## File Targets

- `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt` [EDIT  verify clearPeerCache is called]
- `android/app/src/test/java/com/scmessenger/android/transport/ble/BleScannerTest.kt` [NEW]

## Build Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin -q
./gradlew :app:testDebugUnitTest --tests "*BleScannerTest"
./gradlew :app:assembleDebug -x lint --quiet
```

## Acceptance Gates

1. `./gradlew :app:assembleDebug -x lint` succeeds
2. `BleScannerTest` passes (both test methods)
3. `clearPeerCache()` is reachable from at least 2 call sites (onDiscoveryStop, transport pause)
4. Manual: start BLE discovery  5 peers discovered  stop discovery  0 peers in cache
5. Commit: `android: v0.2.1 BLE scanner stale cache cleanup + unit test`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001] [MICRO_TASK]
