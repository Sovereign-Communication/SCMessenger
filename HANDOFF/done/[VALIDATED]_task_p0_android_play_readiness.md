# Task: Verify and Finalize P0 Android Play Readiness Changes

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P0
**Model:** gemma4:31b:cloud
**Budget:** 2000
**Assigned to:** worker
**Created:** 2026-05-13
**Status:** IMPLEMENTATION DONE  verification only

## Summary

The P0 Android Google Play Readiness code changes have been implemented by workers. This task verifies the changes and finalizes them.

### Already Implemented (verify these exist):

1. **AndroidManifest.xml** line 107: `android:foregroundServiceType="connectedDevice|dataSync"`  CONFIRMED
2. **MdnsServiceDiscovery.kt**: 3 `@Suppress("DEPRECATION")` sites resolved (host API, resolveService)
3. **WifiDirectTransport.kt**: `IntentCompat.getParcelableExtra` migrated to `intent.getParcelableExtra<>()`
4. **BleGattClient.kt**: Class-level `@Suppress("DEPRECATION")` removed, TRANSPORT_LE replaced
5. **Theme.kt**: `window.statusBarColor` replaced with WindowCompat approach

### Known Regression (separate task handles this):
- Theme.kt lost status bar color tinting  tracked in `[VALIDATED]_task_p0_theme_regression_fix.md`

## What To Do

1. Run `cd android && ./gradlew assembleDebug -x lint --quiet` to verify build
2. Run `./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"`
3. Run `git diff --stat` to confirm all expected files modified
4. If build passes: move this file to `HANDOFF/done/` and commit with message "swarm: completed P0 Android Play Readiness"
5. If build fails: log the error in this file and leave in todo/

## Verification

- Build must pass
- Tests must pass
- No `@Suppress("DEPRECATION")` remaining in these files (check with grep)
