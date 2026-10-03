# P0_ANDROID_IDENTITY_003: Identity Persistence Hardening

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** COMPLETED
**Priority:** P1 (HIGH)
**Estimated LoC Impact:** ~30 lines changed across 2 files
**Completion Date:** 2026-04-24

## Issues to Fix

### Issue #7: SharedPreferences backup cleared  identity detection fails
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:3898-3930`
**Problem:** `isIdentityInitialized()` fast path checks `identityBackupPrefs.contains(IDENTITY_BACKUP_KEY)`. If this key is lost (app data clear, upgrade, corruption), falls through to disk check. No backup redundancy.
**Fix:** Consider:
- Adding a secondary sentinel file on disk (separate from SharedPreferences) to detect prior identity creation
- Or verifying `identity.db` exists alongside the SharedPreferences check
- Or adding an in-memory flag that survives ViewModel lifecycle

### Issue #12: Backup SharedPreferences file not protected from storage pruning
**File:** `android/app/src/main/java/com/scmessenger/android/utils/StorageManager.kt`
**Problem:** StorageManager explicitly excludes `identity.db` from pruning, but doesn't protect `identity_backup_prefs.xml` in SharedPreferences.
**Fix:** Add the SharedPreferences backup file to the exclusion list in StorageManager.

## Files to Modify
1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`  harden isIdentityInitialized()
2. `android/app/src/main/java/com/scmessenger/android/utils/StorageManager.kt`  protect backup prefs
3. `android/app/src/main/java/com/scmessenger/android/data/PreferencesRepository.kt`  consider adding backup flag

## Implementation Summary

### MeshRepository.kt (lines 3916-3958)
The `isIdentityInitialized()` function already implements a multi-tier approach:
1. **Fast path**: Check SharedPreferences backup (`identityBackupPrefs.contains(IDENTITY_BACKUP_KEY)`)
2. **Fallback**: Check disk-based sentinel file (`identity_backup_prefs.xml.sentinel`)
3. **Authoritative**: Check Rust core identity database (`identity.db`)

Additionally, if the backup exists but Rust core reports uninitialized, the function attempts automatic restore:
```kotlin
val restored = restoreIdentityFromBackup(ironCore!!)
if (restored) {
    ironCore?.grantConsent()
    Timber.i("Identity restored from backup during isIdentityInitialized check")
}
```

### StorageManager.kt (lines 114-141)
The `pruneOldFiles()` function already protects critical files:
```kotlin
private const val IDENTITY_BACKUP_PREFS = "identity_backup_prefs.xml"
val protectedFiles = setOf(IDENTITY_DB, IDENTITY_BACKUP_PREFS)
```

The `IDENTITY_BACKUP_PREFS` (identity_backup_prefs.xml) is included in the `protectedFiles` set and is skipped during the 30-day file pruning process.

## Verification
- [x] Code review confirms both protection mechanisms are in place
- [x] Build verification: `./gradlew assembleDebug` passes
- [x] No compilation errors

## Notes
The implementation was already present in the codebase at the time of task review. The task is marked completed as the hardening mechanisms are functioning as designed.
