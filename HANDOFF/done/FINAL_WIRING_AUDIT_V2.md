# FINAL WIRING AUDIT V2  Google Play Readiness & Edge-Case Exception Sweep

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Date:** 2026-04-30
**Auditor:** Claude Code Lead Auditor (read-only sweep)
**Scope:** Repository-wide  Android `app/`, Rust `core/`, WASM `wasm/`, CLI `cli/`
**Methodology:** Parallel agent grep; no files modified.

---

## Executive Summary

The codebase is notably clean. There are **zero** `TODO("Not yet implemented")` stubs, **zero** `unimplemented!()` macros, **zero** `todo!()` macros, **zero** `NotImplementedError` / `UnsupportedOperationException` throw sites, **zero** hardcoded API keys or secrets. All `PendingIntent` usages are mutability-flagged correctly.

The findings below are real but mostly low-severity. The only item approaching Play Store scrutiny is the deprecated API suppressions in the transport layer combined with targetSdk=35.

---

## P0  Play Store Blockers / High Risk

### P0-1: Deprecated API usage with `@Suppress("DEPRECATION")` (targetSdk=35)
**Risk:** Google Play scans for deprecated API calls, especially when targetSdk is high. With targetSdk=35, these suppressions may trigger manual review flags.

| File | Line | Deprecated API |
|------|------|---------------|
| `android/.../transport/MdnsServiceDiscovery.kt` | 199 | `resolvedInfo.host`, `resolvedInfo.port`, `resolvedInfo.attributes`  all deprecated on `NsdServiceInfo` |
| `android/.../transport/MdnsServiceDiscovery.kt` | 231 | `resolvedInfo.host?.hostAddress`  use `hostAddresses` (API 34+) |
| `android/.../transport/MdnsServiceDiscovery.kt` | 250 | `nsdManager?.resolveService(serviceInfo, resolveListener)`  overload deprecated |
| `android/.../transport/ble/BleGattClient.kt` | 475 | `BluetoothGattCallback()`  no-arg constructor deprecated |
| `android/.../transport/ble/BleGattServer.kt` | 260 | `BluetoothGattServerCallback()`  no-arg constructor deprecated |
| `android/.../transport/WifiDirectTransport.kt` | 286-289 | `NetworkInfo` and `NetworkInfo.isConnected`  deprecated |

**Recommendation:** Each instance already has API-level branching. Audit each to confirm the fallback path for API 34+ actually works. Migrate mDNS resolution to the `NsdManager.ResolveListener` overload for API 34+ if not already done. Replace `BluetoothGattCallback()` with the `BluetoothDevice`-parameter constructor.

### P0-2: Single foreground service type (`connectedDevice`)  no `dataSync`
**Risk:** If the foreground service performs any data synchronization (message relay, outbox flush) while the app is in the background, Google requires `foregroundServiceType="dataSync"` in addition to (or instead of) `connectedDevice`.

**File:** `android/app/src/main/AndroidManifest.xml`, line 106
```xml
android:foregroundServiceType="connectedDevice"
```

**Recommendation:** Review the actual operations performed in `MeshForegroundService` during background execution. If outbox delivery or relay custody syncing happens, add `dataSync`:
```xml
android:foregroundServiceType="connectedDevice|dataSync"
```

---

## P1  Feature Incomplete / Missing Wiring

### P1-1: Multi-device blocking not implemented
**Files:** `core/src/store/blocked.rs`, lines 4, 17, 59

Three TODO comments all referencing the same gap: device-ID pairing with identity for multi-device blocking. The `BlockedStore` currently operates on identity-level blocking but lacks per-device granularity.

```rust
// Line 4:  // TODO: Add device ID to identity pairing for multi-device blocking.
// Line 17: /// TODO: Implement device ID pairing with identity
// Line 59: /// TODO: Requires device ID infrastructure
```

**Impact:** If a user blocks an identity, the block applies to all of that identity's devices. Conversely, there is no way to block a specific compromised device while keeping the identity trusted. This is a privacy edge case that may surface in multi-device scenarios.

### P1-2: MeshVpnService disabled by default
**File:** `android/app/src/main/AndroidManifest.xml`, line 111
```xml
android:enabled="false"
```

The VPN transport path exists in code but is disabled in the manifest. This is either intentionally deferred or incomplete wiring. If VPN transport is meant to ship in v0.2.1, it needs enabling and testing.

### P1-3: `@deprecated` methods in MeshRepository without migration path
**Files:** `android/.../data/MeshRepository.kt`

| Line | Description |
|------|-------------|
| 3230 | `@deprecated Use canonicalContactId() for new code.` |
| 7258 | `@deprecated Use the suspend version with circuit breaker support` |

Neither method has a removal timeline. Callers may still be using the deprecated variants.

---

## P2  Technical Debt / Code Quality

### P2-1: `IllegalStateException` proliferation in MeshRepository (14 throw sites)
**File:** `android/.../data/MeshRepository.kt`

All 14 production-code `throw IllegalStateException(...)` calls are guard clauses for uninitialized state (`ironCore == null`, mesh service not running, etc.). This pattern converts recoverable state errors into crashes. Consider a sealed `Result` type or a `MeshStateException` hierarchy to allow callers (ViewModels) to handle gracefully rather than crashing.

Key throw sites:
- Line 117, 2023, 2047  service startup failures
- Line 2931, 3528, 3542, 3738, 3742  core not initialized
- Line 3616, 3717, 3730  invalid key/identity format
- Line 3837, 3985  mesh service initialization

### P2-2: Duplicate notification channel creation
Both `NotificationHelper.kt` (line 93, called from `MeshApplication.onCreate()`) and `MeshForegroundService.kt` (line 520) create the foreground service notification channel. The service-level creation at line 520-531 is a redundant fallback  it runs every time the service starts, even though the channel already exists from app launch. This is harmless but unnecessary.

### P2-3: `emptyList()` guard clauses are legitimate  no stubs found
All 9 instances of `return emptyList()` in `MeshRepository.kt` are guard-clause early returns (missing env var, missing file, null array, blank input, invalid peer ID). None are stubbed "not yet implemented" placeholders. No action needed.

### P2-4: Test mockk comments are contained to test files
All 4 `// mock` comments found are in `ConversationsViewModelTest.kt` (a test file using mockk). No production code stubs.

---

## Items Verified Clean (No Findings)

| Audit Target | Result |
|-------------|--------|
| `TODO("Not yet implemented")` | 0 matches |
| `NotImplementedError` / `UnsupportedOperationException` | 0 matches |
| `unimplemented!()` in Rust core | 0 matches |
| `todo!()` in Rust core | 0 matches |
| `// FIXME` in Rust core | 0 matches |
| `// HACK` in Rust core | 0 matches |
| Hardcoded API keys / passwords | 0 matches |
| `PendingIntent` mutability flags | All compliant (7 IMMUTABLE, 1 MUTABLE for RemoteInput) |
| NotificationChannel setup | 5 channels created at app launch, Android 14+ compliant |
| `SCHEDULE_EXACT_ALARM` | Not needed  no AlarmManager usage |
| `ForegroundServiceStartNotAllowedException` | Explicitly handled (string-name check for API 31+) |
| `targetSdk` | 35 (current, correct) |
| `minSdk` | 26 (correct for WiFi Aware) |
| `compileSdk` | 35 (current, correct) |
| `core/src/privacy/` | 0 TODOs, 0 stubs |
| `core/src/routing/` | 0 TODOs, 0 stubs |
| `core/src/relay/` | 0 TODOs, 0 stubs |
| `core/src/transport/` | 0 TODOs, 0 stubs |

---

## Severity Summary

| Severity | Count | Top Item |
|----------|-------|----------|
| P0 (Play Store Blocker) | 2 | Deprecated API suppressions + missing dataSync foreground service type |
| P1 (Feature Incomplete) | 3 | Multi-device blocking, VPN disabled, deprecated methods |
| P2 (Technical Debt) | 4 | IllegalStateException proliferation, duplicate channel, redundant fallbacks |

---

STATUS: AUDIT_COMPLETE
