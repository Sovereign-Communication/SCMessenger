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
# token_budget: 12000

# P0_ANDROID_019_Auto_Backup_Stale_Data_Fix

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P0 Android stability
**Source:** docs/historical/MASTER_BUG_TRACKER.md P0 Android auto-backup issue + planfromclaudeforhermes 2 Phase D.1
**Depends on:** P0_BUILD_001

---

## Verified Gap

Android auto-backup restores stale `contacts.db`, `identity.backup`, and `sled/` data after app reinstall, causing contacts to vanish and identity to roll back. Per `ANDROID_PIXEL_6A_AUDIT_2026-04-17`: "Contacts migration already completed, skipping. Contact data verification - Found 0 contacts."

Auto-backup is enabled by default for `targetSdk >= 31` unless explicitly disabled.

## Scope (~80 LoC across 3 files)

### Part A: Disable auto-backup for sensitive data (LOC: ~30)

In `android/app/src/main/AndroidManifest.xml` `<application>` element:

Add:
```xml
<application
    android:allowBackup="false"
    android:fullBackupContent="@xml/backup_rules"
    android:dataExtractionRules="@xml/data_extraction_rules"
    ...>
```

### Part B: Backup rules XML (LOC: ~25)

Create `android/app/src/main/res/xml/backup_rules.xml`:
```xml
<?xml version="1.0" encoding="utf-8"?>
<full-backup-content>
    <exclude domain="database" path="contacts.db" />
    <exclude domain="database" path="contacts.db-journal" />
    <exclude domain="database" path="identity.backup" />
    <exclude domain="file" path="sled/" />
    <exclude domain="sharedpref" path="identity_prefs.xml" />
</full-backup-content>
```

### Part C: Data extraction rules XML (LOC: ~25)

Create `android/app/src/main/res/xml/data_extraction_rules.xml`:
```xml
<?xml version="1.0" encoding="utf-8"?>
<data-extraction-rules>
    <cloud-backup>
        <exclude domain="database" path="contacts.db" />
        <exclude domain="database" path="identity.backup" />
        <exclude domain="file" path="sled/" />
    </cloud-backup>
    <device-transfer>
        <exclude domain="database" path="contacts.db" />
        <exclude domain="database" path="identity.backup" />
        <exclude domain="file" path="sled/" />
    </device-transfer>
</data-extraction-rules>
```

## File Targets

- `android/app/src/main/AndroidManifest.xml` [EDIT  add allowBackup=false, backup_rules, dataExtractionRules]
- `android/app/src/main/res/xml/backup_rules.xml` [NEW]
- `android/app/src/main/res/xml/data_extraction_rules.xml` [NEW]

## Build Verification Commands

```bash
cd android
./gradlew :app:assembleDebug -x lint --quiet
# Inspect manifest in built APK
aapt dump xmltree app/build/outputs/apk/debug/app-debug.apk AndroidManifest.xml | grep -E "allowBackup|backup|dataExtraction"
```

## Acceptance Gates

1. `./gradlew :app:assembleDebug -x lint` succeeds
2. Built APK manifest has `android:allowBackup="false"`
3. `backup_rules.xml` excludes all 5 sensitive paths
4. `data_extraction_rules.xml` excludes contacts/identity/sled
5. Manual: uninstall app, reinstall  no stale data restored
6. Commit: `android: v0.2.1 disable auto-backup for sensitive data`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: KOTLIN] [REQUIRES: QWEN_CODER_NEXT] [DEPENDS_ON: P0_BUILD_001]
