# MODEL: glm-5.1:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1800
# token_budget: 18000

# P1_ANDROID_DIAGNOSTICS_EXPORT_001

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 Android support tooling
**Source:** AUDIT_ANDROID_WINDOWS_INTEROP_PARITY_2026-05-20.md

---

## Verified Gap

Android `DiagnosticsScreen.kt` shows live mesh diagnostics but has NO export/share functionality. CLI has no export command either. Users cannot generate diagnostic bundles for bug reporting.

**Verified Code State:**
- `MeshRepository.kt:4737`  `exportDiagnosticsAsync()` exists and returns a JSON string
- `DiagnosticsScreen.kt:39`  No export/share button or action
- No FileProvider intent for sharing diagnostic text

## Scope

### Part A: Export Button in DiagnosticsScreen

1. Add "Export Diagnostics" button (outlined button with share icon) to `DiagnosticsScreen.kt`
2. On tap: call `MeshRepository.exportDiagnosticsAsync()`
3. Save to `tmp/session_logs/` or `getExternalFilesDir()`
4. Launch Android share sheet via `Intent.ACTION_SEND` with `text/plain` MIME type
5. Filename: `scmessenger-diagnostics-YYYY-MM-DD-HHMMSS.json`

### Part B: FileProvider Path

1. Ensure `file_paths.xml` already supports the export directory (check `AndroidManifest.xml` FileProvider)
2. If needed, add `<external-files-path name="diagnostics" path="." />` to `file_paths.xml`

## Constraints

- Use Android Sharesheet (not custom picker)
- Respect user privacy: only export non-sensitive fields (peer IDs are OK, message content is NOT)
- Verify `exportDiagnosticsAsync()` output does NOT contain raw message content before sharing

## File Targets

- `android/app/src/main/java/com/scmessenger/android/ui/screens/DiagnosticsScreen.kt` [EDIT]
- `android/app/src/main/res/xml/file_paths.xml` [MAY EDIT]
- `android/app/src/main/res/values/strings.xml` [EDIT]

## Build Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin -q
```

## Acceptance Gates

1. `./gradlew :app:compileDebugKotlin` passes
2. Export button visible in DiagnosticsScreen
3. Share sheet launches with valid JSON

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
