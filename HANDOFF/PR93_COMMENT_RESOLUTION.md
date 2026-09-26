# PR #93 Comment Resolution Log

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

This document maps every review comment on PR #93 to the concrete update made in this branch.

## 1) cubic-dev-ai — Batch range inconsistency (P2)
- **Comment:** `12–15` batches conflicted with `B1..B8` naming.
- **Resolution:** Normalized the plan to exactly **8 batches (B1..B8)** and updated ordering text accordingly.
- **Files:** `HANDOFF/WIRING_MASTER_EXECUTION_PLAN.md`

## 2) cubic-dev-ai — Null anchor coordinates in manifest (P2)
- **Comment:** Three entries had null `definition_line` / `anchor_line`.
- **Resolution:** Generator now fails fast on unresolved anchors and includes alias resolution for known task-name mismatches.
- **Files:** `scripts/generate_wiring_patch_manifest.py`, regenerated `HANDOFF/WIRING_PATCH_MANIFEST.json`, `HANDOFF/WIRING_PATCH_MANIFEST.md`

## 3) cubic-dev-ai — `find_def_line` directory crash risk (P2)
- **Comment:** `exists()` check could pass for directories; `read_text()` would crash.
- **Resolution:** Replaced guard with `target.is_file()` and defensive handling.
- **Files:** `scripts/generate_wiring_patch_manifest.py`

## 4) cubic-dev-ai — incorrect hotspot path `wasm/lib.rs` (P3)
- **Comment:** Path should be `wasm/src/lib.rs`.
- **Resolution:** Corrected hotspot path in risk register section.
- **Files:** `HANDOFF/WIRING_MASTER_EXECUTION_PLAN.md`

## 5) codex-connector — exclude planning docs from reference-hit counts (P2)
- **Comment:** `external_reference_hits` inflated by `HANDOFF/**` and generated manifests.
- **Resolution:** Restricted reference scan to source-code extensions and excluded documentation/planning directories including `HANDOFF`, `docs`, `reference`.
- **Files:** `scripts/generate_wiring_patch_manifest.py`

## 6) codex-connector — fail generation when definition unresolved (P2)
- **Comment:** `None` anchors should not be emitted in an "exact coordinates" manifest.
- **Resolution:** Generator now aborts with explicit unresolved task listing and non-zero exit code.
- **Files:** `scripts/generate_wiring_patch_manifest.py`

## 7) Copilot — reference counts include todo/manifests
- **Comment:** Same inflation concern as #5.
- **Resolution:** Same fix as #5; counts now come from source files only.
- **Files:** `scripts/generate_wiring_patch_manifest.py`

## 8) Copilot — `find_def_line` should use `is_file()`
- **Comment:** Same directory crash concern as #3.
- **Resolution:** Same fix as #3.
- **Files:** `scripts/generate_wiring_patch_manifest.py`

## 9) Copilot — unresolved anchors should be explicitly flagged
- **Comment:** Null anchors need explicit handling.
- **Resolution:** Hard fail on unresolved anchors + explicit stderr listing; additionally added `resolved_symbol` field for alias transparency.
- **Files:** `scripts/generate_wiring_patch_manifest.py`, regenerated manifests

## 10) Copilot — formatting/style consistency in Python script
- **Comment:** Script style was inconsistent and hard to maintain.
- **Resolution:** Rewrote script with 4-space indentation, clearer naming, docstring, structured helpers, and PEP8-ish spacing.
- **Files:** `scripts/generate_wiring_patch_manifest.py`

## Regeneration verification
- Ran `python scripts/generate_wiring_patch_manifest.py`.
- Confirmed output shows `generated 350`.
- Confirmed no null anchors remain.
