<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-08

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** see `HANDOFF/STATE/2026-06-08_UNIFIED_BACKLOG.md`
**Decided by:** Claude Code (Overseer) sweep

**Rationale:** Ticket is a real remaining work item with no shipped code on the
integration branch. No blocker identified. Ready for `/orchestrate` dispatch on
the next cloud slot allocation. Per Lucas directive 2026-06-08 "I want it all
fixed," this is part of the ~30-ticket remaining backlog.

---
# MODEL: glm-5.1:cloud
# BUDGET: 1800
# token_budget: 18000

# P1_ANDROID_AUDIT_LOG_VIEWER_001

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1800s (MIXED tier)
**Phase:** v0.2.1 Android security/observability
**Source:** AUDIT_ANDROID_WINDOWS_INTEROP_PARITY_2026-05-20.md

---

## Verified Gap

Android has NO screen to view the cryptographic audit log. The CLI has `audit {export, verify, stats}` commands. `MeshRepository.kt` exposes `exportAuditLog()` and `validateAuditChain()` via UniFFI (verified in `iron_core.rs`), but there is NO UI for it.

**Verified Code State:**
- `iron_core.rs:1447`  `pub fn export_audit_log()` exists
- `iron_core.rs:1453`  `pub fn validate_audit_chain()` exists
- `MeshRepository.kt:3683`  `exportLogs()` exists but no `exportAuditLog()` wrapper
- No `AuditLogScreen.kt` or similar in `ui/screens/`

## Scope

### Part A: Audit Log Screen

1. Create `android/app/src/main/java/com/scmessenger/android/ui/screens/AuditLogScreen.kt`
   - LazyColumn of audit events
   - Each item: event type icon, timestamp, identity_id, peer_id, details
   - Color-coded by event type (green=message sent/received, yellow=relay, red=blocked, blue=backup)
   - Group by date (sticky headers)

2. Create `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/AuditLogViewModel.kt`
   - `events: StateFlow<List<AuditEvent>>`
   - Loads from `MeshRepository.exportAuditLog()` (or add wrapper if missing)
   - Methods: `refresh()`, `exportToFile()` (shares as text file)

### Part B: Navigation

1. Add "Audit Log" menu item to `SettingsScreen.kt` or `DiagnosticsScreen.kt`
2. Add `Screen.AuditLog` route to `MeshApp.kt` NavHost

### Part C: Strings

Add all new strings to `strings.xml`

## Constraints

- Material3 design language
- Follow existing architecture
- All strings externalized

## File Targets

- `android/app/src/main/java/com/scmessenger/android/ui/screens/AuditLogScreen.kt` [NEW]
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/AuditLogViewModel.kt` [NEW]
- `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/ui/MeshApp.kt` [EDIT]
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` [EDIT  add audit log wrapper if missing]
- `android/app/src/main/res/values/strings.xml` [EDIT]

## Build Verification Commands

```bash
cd android
./gradlew :app:compileDebugKotlin -q
```

## Acceptance Gates

1. `./gradlew :app:compileDebugKotlin` passes
2. AuditLogScreen renders in Android Studio preview
3. Navigable from Settings or Diagnostics screen

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
