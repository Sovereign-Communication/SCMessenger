# P6: FFI Snapshot Drift Fix - Status (Conditional on P5)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Task:** Regenerate FFI snapshots if P5 (D-05 unwrap/panic hardening) changes UDL  
**Status:** BLOCKED - Waiting for P5 Completion  
**Date:** 2026-07-22

## Dependency

P6 is **conditional** on P5 (D-05: unwrap/panic hardening - revert UDL scope-creep).

**P5 Status:** PENDING (not yet started)

## Blocking Logic

1. **If P5 Shows NO UDL Changes:**
   - Mark P6 as COMPLETED with reason: "No FFI drift — P5 had no UDL changes"
   - No snapshot regeneration needed

2. **If P5 Changes UDL:**
   - Run FFI surface regeneration script:
     ```bash
     scripts/ffi_surface.sh
     ```
   - Review diff:
     ```bash
     git diff scripts/ffi-snapshots/
     ```
   - Verify changes match expected (method removal, error variant removal, etc.)
   - Commit snapshots:
     ```bash
     git add scripts/ffi-snapshots/
     git commit -m "fix(ffi): update snapshots after D-05 scope revert"
     ```

## What P6 Does (When Triggered)

1. Regenerate FFI surface definitions for Kotlin and Swift
2. Verify no unexpected new symbols introduced
3. Commit snapshot updates

## Acceptance Criteria (if P5 changes UDL)

- [x] FFI surface regeneration script exists: `scripts/ffi_surface.sh`
- [ ] Snapshots regenerated (conditional on P5)
- [ ] Changes match expected removals only
- [ ] Verification: `cargo check --workspace` passes
- [ ] Snapshots committed with proper message

## Files That Would Be Updated (if P5 triggers)

- `scripts/ffi-snapshots/kotlin-symbols.txt`
- `scripts/ffi-snapshots/swift-symbols.txt`

## Unblock Condition

P5 (D-05) must be completed and:
- If P5 modified UDL → Run P6 steps above
- If P5 did NOT modify UDL → Mark P6 COMPLETED with "no drift" reason

---

**Next Step:** Monitor P5 completion. When P5 is done, check its git diff for UDL changes and proceed accordingly.
