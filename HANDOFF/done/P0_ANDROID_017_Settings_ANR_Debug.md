# Android Settings ANR Optimization & Debug

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-04-23
**Agent:** implementer (Android specialist)
**Model:** qwen3-coder-next:cloud

## Problem Statement

Android app experiences severe ANRs when loading Settings tab. Fresh reinstall did not resolve issue.

## Symptoms
- Settings tab loads extremely slowly
- Export diagnostic logs crashes the app
- ANR: Input dispatching timed out (5001ms for MotionEvent)
- CPU usage: 500% total, 4 DefaultDispatch threads at 95-98% each
- Root cause: Background network operations blocking UI thread

## Logcat Evidence
- File: `android/android_logcat_4-23-26.md`
- ANR PID: 7368 in MainActivity
- DefaultDispatch threads consuming all CPU during bootstrap dial failures
- Circuit breakers opening for all bootstrap nodes
- MeshRepository doing heavy I/O on main thread

## Task

1. **Analyze** `android/android_logcat_4-23-26.md` for ANR root cause
2. **Review** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` for blocking operations
3. **Review** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt` for slow operations
4. **Optimize**:
   - Move heavy operations off UI thread
   - Add debouncing for settings changes
   - Cache settings data instead of recalculating
   - Fix export diagnostic logs crash
5. **Document** findings in `HANDOFF/IN_PROGRESS/ANDROID_ANR_RCA_2026-04-23.md`

## Expected Deliverables
- Fixed Settings loading time < 500ms
- Export logs works without crash
- No ANRs in Settings tab
- Report with specific code changes made

## Native Routing Directives
- [NATIVE_SUB_AGENT: RESEARCH] - Map all blocking calls in Settings flow
- [NATIVE_SUB_AGENT: LINT_FORMAT] - Format all modified Kotlin files


---
**Gatekeeper Approval:** 2026-04-23 23:35
- Verified: cargo check --workspace (warnings only)
- Verified: ./gradlew :app:compileDebugKotlin (BUILD SUCCESSFUL)
- Status: APPROVED by Lead Orchestrator

