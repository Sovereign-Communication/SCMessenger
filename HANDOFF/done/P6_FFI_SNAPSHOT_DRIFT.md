# P6: FFI Snapshot Drift Fix (Conditional on P5)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Ticket Status:** Open (depends on P5 compile-gate result)
**Tier:** [HAIKU]
**Scope:** v0.4.0 blocker (if P5 changes snapshot)

## Background

Depends on P5 (D-05 unwrap/panic hardening). When D-05 reverts lib.rs:81 UDL scope-creep, it may change the FFI surface. If so, FFI snapshots must be regenerated.

This task only proceeds if P5 changes the UDL. If P5 shows no UDL changes, this task is marked completed with reason "no drift".

## Specification

### If P5 Changes UDL

1. Run FFI surface regeneration:
   ```bash
   scripts/ffi_surface.sh
   ```

2. Review diff:
   ```bash
   git diff scripts/ffi-snapshots/
   ```
   Verify changes match expected (method removal, error variant removal, etc.)

3. Commit:
   ```bash
   git add scripts/ffi-snapshots/
   git commit -m "fix(ffi): update snapshots after D-05 scope revert"
   ```

### If P5 Shows No UDL Changes

Mark task as completed with note: "P5 had no UDL changes; snapshot regeneration not needed."

## Files to Update (if needed)

- `scripts/ffi-snapshots/kotlin-symbols.txt`
- `scripts/ffi-snapshots/swift-symbols.txt`

## Acceptance Criteria

1. If changes: `git diff scripts/ffi-snapshots/` shows only expected removals (no new symbols added unexpectedly)
2. Verification: `cargo check --workspace` passes
3. No new FFI errors: `scripts/ffi_surface.sh` completes without errors

## Notes

- This is mechanical: regenerate snapshots, verify against expectation, commit
- Do NOT manually edit snapshots; always use `ffi_surface.sh`
- If P5 fails or doesn't apply, wait for P5 to complete before attempting this task

---

**Dispatch to:** Qwen HAIKU or manual  
**Model:** qwen3-coder-flash (if dispatched)  
**Blocked by:** P5  
**Move to done/ when:** Snapshots regenerated and verified (or "no drift" reason recorded)  

## 2026-07-28 closure (orchestrator takeover audit)
CLOSED no-drift: scripts/ffi_surface.sh --check PASS (Kotlin FFI surface: OK; Swift FFI surface: OK) at HEAD 821a0646. UDL unchanged since last snapshot; generated-sources intact (core/target/generated-sources, 1.2 MB). D-05 precondition done.
