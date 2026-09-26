<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `STATE/PLAN_VERIFICATION_2026-06-11.md` 4 (Identity  backup security)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (XML config + manifest, no logic)
**Rationale:** D1 from the Android stability plan. Auto-backup was restoring stale `sled/` and `identity.backup` files across reinstalls, causing identity corruption on Pixel 6a. Fix is mechanical: two new XML files + manifest attributes. ~80 LoC of pure config. Flash-friendly.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 8000

# P1_GEMINI_FLASH_004  Android Auto-Backup Exclusion Rules

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Android stability (D1)
**Source:** `ANDROID_PIXEL_6A_AUDIT_2026-04-17.md` (critical issue: stale sled restore)
**Depends on:** none
**Blocks:** P0_SECURITY_007 (Identity backup encryption  once exclusion is in place, encrypted backup is the next step)

---

## Verified Gap

`AndroidManifest.xml` does NOT set `android:fullBackupContent` or `android:dataExtractionRules`. Default behavior: Google Auto Backup includes everything in `getFilesDir()` and `getDatabasePath()`. On Pixel 6a this caused `sled/` store, `contacts.db`, and `identity.backup` to be backed up to Google Drive, then restored to a fresh install  restoring stale state and corrupting the new identity.

## Scope (~80 LoC across 3 files)

### Part A: Create backup rules XML (LOC: ~40)

New file `android/app/src/main/res/xml/backup_rules.xml`:
```xml
<?xml version="1.0" encoding="utf-8"?>
<full-backup-content>
    <exclude domain="file" path="sled/" />
    <exclude domain="file" path="identity.backup" />
    <exclude domain="database" path="contacts.db" />
    <exclude domain="sharedpref" path="identity_prefs.xml" />
</full-backup-content>
```

### Part B: Create data extraction rules (LOC: ~30)

New file `android/app/src/main/res/xml/data_extraction_rules.xml`:
```xml
<?xml version="1.0" encoding="utf-8"?>
<data-extraction-rules>
    <cloud-backup>
        <exclude domain="file" path="sled/" />
        <exclude domain="file" path="identity.backup" />
        <exclude domain="database" path="contacts.db" />
    </cloud-backup>
    <device-transfer>
        <exclude domain="file" path="sled/" />
        <exclude domain="file" path="identity.backup" />
    </device-transfer>
</data-extraction-rules>
```

### Part C: Wire in `AndroidManifest.xml` (LOC: ~5)

Add to `<application>`:
```xml
android:allowBackup="false"
android:fullBackupContent="@xml/backup_rules"
android:dataExtractionRules="@xml/data_extraction_rules"
```

## File Targets

- `android/app/src/main/res/xml/backup_rules.xml` [NEW  ~40 LoC]
- `android/app/src/main/res/xml/data_extraction_rules.xml` [NEW  ~30 LoC]
- `android/app/src/main/AndroidManifest.xml` [EDIT  3 attributes, ~5 LoC]

## Build Verification

```bash
cd android
./gradlew :app:assembleDebug -x lint --quiet
# Verify XML parses:
xmllint --noout app/src/main/res/xml/backup_rules.xml
xmllint --noout app/src/main/res/xml/data_extraction_rules.xml
# Verify manifest:
aapt2 dump xmltree app/build/outputs/apk/debug/app-debug.apk --file AndroidManifest.xml | grep -E "allowBackup|fullBackupContent|dataExtractionRules"
```

## Acceptance Gates

1. APK builds
2. `xmllint` validates both XML files
3. `aapt2 dump` confirms the three attributes are present in the built manifest
4. Manual: install APK, enable backup, `adb shell bmgr backupnow com.scmessenger.android`, then `adb shell bmgr restore`, verify `sled/` and `identity.backup` are NOT restored

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: XML] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 4]
