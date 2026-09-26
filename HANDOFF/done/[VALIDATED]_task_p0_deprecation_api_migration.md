# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

# BUDGET: 1200

## P0: Remove Remaining @Suppress("DEPRECATION") Annotations (API Migration)

**Priority:** P0
**Model:** qwen3-coder-next:cloud
**Budget:** 1200
**Assigned to:** implementer
**Created:** 2026-05-13
**Source:** 2026-05-13 MASTER AUDIT  Deprecated API usage with @Suppress("DEPRECATION") at targetSdk=35

### Current State
Two remaining @Suppress("DEPRECATION") sites require API migration:

1. **MdnsServiceDiscovery.kt:459**  `nsdManager?.resolveService(serviceInfo, resolveListener)` is deprecated in API 33.
   - Replacement: `resolveService(NsdServiceInfo, Executor, NsdManager.ResolveListener)`
   - Use `ContextCompat.getMainExecutor(context)` for the executor on API 33+, fallback to legacy API on older versions.

2. **BleGattServer.kt:30**  `BluetoothManager.openGattServer(context, callback)` is deprecated in API 31.
   - Replacement: `openGattServer(Context, Executor, BluetoothGattServerCallback)`
   - Use `ContextCompat.getMainExecutor(context)` for the executor on API 31+, fallback to legacy API on older versions.
   - Remove class-level @Suppress and use method-level suppression only in the fallback branch.

### Required Work
1. Update `MdnsServiceDiscovery.kt` `resolveService()` to use the Executor overload with SDK version gate.
2. Update `BleGattServer.kt` `openGattServer()` to use the Executor overload with SDK version gate.
3. Remove class-level @Suppress("DEPRECATION") from BleGattServer.kt.
4. Run `cd android && ./gradlew assembleDebug -x lint --quiet` to verify build.

### Verification
- `./gradlew assembleDebug -x lint --quiet` passes
- `grep -r '@Suppress.*DEPRECATION' android/app/src/main/java/` returns zero results (or only the targeted Theme.kt suppression which is functionally required)
- No lint errors introduced by the API migration
