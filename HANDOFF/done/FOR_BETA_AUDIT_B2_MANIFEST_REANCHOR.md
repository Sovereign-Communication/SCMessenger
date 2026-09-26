# [FOR BETA - AUDIT] B2 Manifest Re-Anchor & Cleanup

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P1
**Parent Task:** FOR_ALPHA_WIRE_B2_MANIFEST_REANCHOR.md (now in HANDOFF/review/)
**Status:** Open

## Mission
Audit Alpha's changes from the B2 manifest re-anchor task. Verify build, diff correctness, and manifest accuracy.

## Actions Required
1. Run `cargo check --workspace` and confirm **zero errors, zero warnings**.
2. Run `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` and confirm pass.
3. Review `git diff` for the following files and verify:
   - `HANDOFF/WIRING_PATCH_MANIFEST.md` B2 line numbers are accurate (spot-check at least 10 symbols against actual source).
   - `core/src/store/relay_custody.rs`  `force_state_for_test` is fully removed with no orphan references.
   - `core/Cargo.toml`  `default-features` warning is eliminated.
   - `Cargo.toml` (root)  no unintended changes.
   - `HANDOFF/todo/task_security_tooling.md` deletion is appropriate (confirm it's a stale/outdated task file).
4. Check that `HANDOFF/todo/FOR_BETA_SWEEP_B2_CORE_TRANSPORT_ROUTING.md` was moved to `HANDOFF/done/` (closeout hygiene).
5. Write audit verdict to `HANDOFF/review/FOR_ALPHA_WIRE_B2_MANIFEST_REANCHOR.md` under an `## Beta Audit Verdict` section with one of:
   - `APPROVED`  all checks pass, ready for Orchestrator commit.
   - `REJECTED`  list specific defects with severity and fix instructions.

## Constraints
- Do NOT modify source code. Read-only audit.
- Do NOT run `cargo test --workspace --no-run` (Windows OOM known blocker).
- Spot-check manifest anchors with `grep -n "fn <symbol>"` against target files.
