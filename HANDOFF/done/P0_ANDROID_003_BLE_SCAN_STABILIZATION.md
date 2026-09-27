# P0_ANDROID_003: BLE Scan Stabilization

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0 CRITICAL
**Platform:** Android
**Status:** DONE
**Completion Date:** 2026-04-20

## Implementation Summary

### 1. BleQuotaManager (NEW)
- Extracted inline quota logic from BleScanner into `BleQuotaManager.kt`
- Enforces Android 12+ quota: max 5 scan starts per 30s window
- Returns cooldown delay when quota exhausted (with 500ms safety margin)
- Thread-safe via `@Synchronized`

### 2. TransportHealthMonitor (NEW)
- Created `TransportHealthMonitor.kt` for per-transport health tracking
- Tracks success/failure counts, latency, consecutive failures
- `shouldUseTransport()` returns false when success rate drops below 30%
- `isDegraded()` returns true after 3+ consecutive failures
- Used by MeshRepository for graceful BLE degradation

### 3. BackoffStrategy Integration
- Replaced hardcoded retry delays in BleScanner with existing `BackoffStrategy`
- `backoffStrategy.reset()` called on successful scan start
- `backoffStrategy.nextDelay()` used for all retry scenarios (error codes 1, 2/3, unknown)

### 4. Scan Session Reuse (already existed)
- BleScanner already returns early on duplicate `isScanning` check
- No changes needed  session reuse was already implemented

### 5. Graceful Degradation
- `handleBleTransportDegradation()` in MeshRepository  switches BLE to background mode on degradation
- `recordTransportEvent()`  records transport health events, auto-checks for degradation
- Periodic health check added to `startStorageMaintenance()` (every 15 min)
- `getTransportHealthSummary()` exposed for diagnostics UI

### Build Verification
- `./gradlew :app:compileDebugKotlin -x :app:buildRustAndroid`  **BUILD SUCCESSFUL**

## Files Created
1. `android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt`
2. `android/app/src/main/java/com/scmessenger/android/transport/TransportHealthMonitor.kt`

## Files Modified
1. `android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt`
2. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`