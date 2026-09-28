# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1200
# token_budget: 12000

# P0_ANDROID_020_Permission_Request_Loop_Fix

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P0 Android stability
**Source:** ANDROID_PIXEL_6A_AUDIT_2026-04-17 4 (BLE scan failures) + planfromclaudeforhermes 2 Phase D.2
**Depends on:** P0_BUILD_001

---

## Verified Gap

`MainActivity.kt` and `PermissionHelper.kt` re-request permissions on every recomposition. Users see repeated system dialogs. Per `ANDROID_PIXEL_6A_AUDIT_2026-04-17`: "Permission request spam" listed as critical.

Pattern: `LaunchedEffect(Unit) { requestPermissions(...) }` re-fires on every recomposition because `Unit` key doesn't change.

## Scope (~120 LoC across 2 files)

### Part A: Permission state machine (LOC: ~80)

In `android/app/src/main/java/com/scmessenger/android/utils/PermissionHelper.kt` (NEW if doesn't exist; otherwise rewrite):

```kotlin
sealed class PermissionState {
    object Idle : PermissionState()
    object Requesting : PermissionState()
    data class Granted(val permissions: Set<String>) : PermissionState()
    data class Denied(val permissions: Set<String>, val canShowRationale: Boolean) : PermissionState()
    data class PermanentlyDenied(val permissions: Set<String>) : PermissionState()
}

class PermissionHelper(private val activity: ComponentActivity) {
    private var state: PermissionState by mutableStateOf(PermissionState.Idle)
    private var lastRequestTime: Long = 0L
    private val cooldownMs = 5000L  // 5 second cooldown between requests
    
    fun requestIfNeeded(permissions: List<String>) {
        // Skip if recently requested
        if (System.currentTimeMillis() - lastRequestTime < cooldownMs) return
        
        // Skip if permanently denied
        if (state is PermissionState.PermanentlyDenied) {
            // Show "open settings" dialog instead
            return
        }
        
        // Skip if all granted
        val missing = permissions.filter { 
            ContextCompat.checkSelfPermission(activity, it) != PackageManager.PERMISSION_GRANTED 
        }
        if (missing.isEmpty()) {
            state = PermissionState.Granted(permissions.toSet())
            return
        }
        
        lastRequestTime = System.currentTimeMillis()
        state = PermissionState.Requesting
        activity.requestPermissions(missing.toTypedArray(), REQUEST_CODE)
    }
    
    fun onRequestResult(granted: List<String>, denied: List<String>) {
        state = when {
            denied.isEmpty() -> PermissionState.Granted(granted.toSet())
            denied.any { !activity.shouldShowRequestPermissionRationale(it) } -> 
                PermissionState.PermanentlyDenied(denied.toSet())
            else -> PermissionState.Denied(denied.toSet(), canShowRationale = true)
        }
    }
}
```

### Part B: Wire state machine into MainActivity (LOC: ~40)

In `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt`:

Replace existing `LaunchedEffect(Unit) { requestPermissions(...) }` blocks with:
```kotlin
val permissionHelper = remember { PermissionHelper(this) }

LaunchedEffect(Unit) {
    permissionHelper.requestIfNeeded(listOf(
        Manifest.permission.BLUETOOTH_SCAN,
        Manifest.permission.BLUETOOTH_ADVERTISE,
        Manifest.permission.BLUETOOTH_CONNECT,
        Manifest.permission.ACCESS_FINE_LOCATION,
        Manifest.permission.NEARBY_WIFI_DEVICES,  // API 33+
    ))
}

DisposableEffect(Unit) {
    val listener = ActivityResultListener { _, results ->
        val granted = results.filter { it.value == PackageManager.PERMISSION_GRANTED }.keys.toList()
        val denied = results.filter { it.value != PackageManager.PERMISSION_GRANTED }.keys.toList()
        permissionHelper.onRequestResult(granted, denied)
    }
    activityResultRegistry.register("permissions", listener)
    onDispose { }
}
```

## File Targets

- `android/app/src/main/java/com/scmessenger/android/utils/PermissionHelper.kt` [NEW]
- `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt` [EDIT  replace LaunchedEffect patterns]

## Build Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin -q
./gradlew :app:assembleDebug -x lint --quiet
```

## Acceptance Gates

1. `./gradlew :app:compileDebugKotlin` passes
2. `./gradlew :app:assembleDebug -x lint` produces APK
3. No `LaunchedEffect(Unit) { requestPermissions` patterns remain in MainActivity
4. New unit test: `PermissionHelperTest` covers Idle  Requesting  Granted, Idle  Denied  Retry, Denied  PermanentlyDenied
5. Manual: install on device, deny permission  see system dialog ONCE; deny again with "don't ask again"  see "open settings" CTA, no more system dialogs
6. Manual: rapid recomposition does NOT trigger additional permission dialogs
7. Commit: `android: v0.2.1 permission state machine  no more request spam`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001]
