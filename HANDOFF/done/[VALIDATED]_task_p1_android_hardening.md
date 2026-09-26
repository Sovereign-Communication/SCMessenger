# Task: Verify and Finalize P1 Android Hardening (Partial) + Route Remaining Work

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Priority:** P1
**Model:** gemma4:31b:cloud
**Budget:** 2000
**Assigned to:** worker
**Created:** 2026-05-13
**Status:** PARTIALLY IMPLEMENTED  verification + split

## Summary

The P1 Android Hardening task was partially implemented. The "Unknown" hardcoded strings sub-task is done. The IllegalStateException and notification dedup sub-tasks have been split into separate focused task files.

### Already Done:
- Hardcoded "Unknown" strings: only remaining instance is in `IdentityViewModel.kt:77` inside a `Timber.d` log statement (not user-facing UI)  acceptable

### Split Into Separate Tasks:
- `[VALIDATED]_task_p1a_illegalstate_crash_audit.md`  13 IllegalStateException sites to audit
- `[VALIDATED]_task_p1b_notification_channel_dedup.md`  Deduplicate notification channel creation

## What To Do

1. Verify the "Unknown" string situation: `grep -r '"Unknown"' android/app/src/main/java/com/scmessenger/android/ui/`  should return nothing
2. Verify the split tasks exist in HANDOFF/todo/
3. If verification passes: move this file to `HANDOFF/done/` and commit with message "swarm: completed P1 Android Hardening (partial)  split remainder into focused tasks"
4. The new split tasks will be picked up by the heartbeat automatically

## Verification

- No hardcoded `"Unknown"` in UI code (grep on android/app/src/main/java/com/scmessenger/android/ui/)
- Split task files exist and have proper Model/Budget metadata
