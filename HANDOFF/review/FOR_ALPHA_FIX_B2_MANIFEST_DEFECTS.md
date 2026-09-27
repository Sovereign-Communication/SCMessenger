# [FOR ALPHA - FIX] B2 Manifest Re-Anchor Defects (Beta Rejection)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Priority:** P1
**Parent:** FOR_ALPHA_WIRE_B2_MANIFEST_REANCHOR.md (Beta Audit: REJECTED)
**Status:** Open

## Beta Rejection Summary
Beta found 3 defects in the manifest re-anchor work. All must be fixed before commit.

---

## Defect 1 (P1) — `force_state_for_test` manifest row NOT deleted

**What happened:** Alpha removed `force_state_for_test` from `core/src/store/relay_custody.rs` but re-anchored its manifest row from 1409 → 1421 instead of deleting it. Line 1421 no longer contains this function.

**Fix:** Delete the entire `force_state_for_test` row from `HANDOFF/WIRING_PATCH_MANIFEST.md` §B2.

---

## Defect 2 (P1) — `relay_custody.rs` anchors off by -11 lines

**What happened:** Alpha deleted `force_state_for_test` (~11 lines) from `relay_custody.rs` but did not re-measure the remaining entries in that file. All entries below the deletion site are now stale by exactly -11 lines.

**Fix:** Update these 10 entries in `HANDOFF/WIRING_PATCH_MANIFEST.md` §B2 to their actual current line numbers:

| Symbol | Current Manifest | Actual Line | New Manifest |
|--------|-----------------|-------------|--------------|
| `custody_transitions_are_recorded` | 1704 | 1693 | 1693 |
| `custody_deduplicates_same_destination_and_message_id` | 1747 | 1736 | 1736 |
| `converge_delivered_for_message_removes_matching_pending_records` | 1778 | 1767 | 1767 |
| `storage_pressure_quota_bands_follow_locked_policy` | 1820 | 1809 | 1809 |
| `storage_pressure_purge_prioritizes_non_identity_then_identity` | 1944 | 1933 | 1933 |
| `storage_pressure_purge_records_audit_transition_before_delete` | 2000 | 1989 | 1989 |
| `storage_pressure_emergency_mode_rejects_non_critical_and_recovers` | 2030 | 2019 | 2019 |
| `custody_audit_persists_across_restart` | 2094 | 2083 | 2083 |
| `storage_pressure_state_uses_synthetic_snapshot_when_probe_unavailable` | 2133 | 2122 | 2122 |
| `for_local_peer_prefers_explicit_custody_dir_override` | 2151 | 2140 | 2140 |

**Verification:** Use `grep -n "fn <symbol>" core/src/store/relay_custody.rs` for each.

---

## Defect 3 (P2) — `get_unhealthy_connections` stale anchor in `health.rs`

**What happened:** Pre-existing stale anchor Alpha missed during spot-check.

**Fix:** Update `get_unhealthy_connections` in `HANDOFF/WIRING_PATCH_MANIFEST.md` §B2 from line 391 to **403**.

**Verification:** `grep -n "fn get_unhealthy_connections" core/src/transport/health.rs` must return 403.

---

## Build Verification
- `cargo check --workspace` must pass with **zero errors and zero warnings**.
- `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` must pass.
- Do NOT run `cargo test --workspace --no-run`.

## Closeout
1. Update evidence section below.
2. Move this task file to `HANDOFF/review/`.
3. The Orchestrator will re-trigger Beta audit.

## Evidence
- Files touched: `HANDOFF/WIRING_PATCH_MANIFEST.md`
- Build/test evidence: `cargo check --workspace` clean
- Parity/doc updates: Manifest accuracy restored
