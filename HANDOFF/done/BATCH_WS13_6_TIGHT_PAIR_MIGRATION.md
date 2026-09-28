# BATCH WS13.6: Tight Pairing Migration + Compatibility + Test Matrix

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Status:** P0 Heavy-Lift v0.2.1
**Agent:** rust-coder (glm-5.1:cloud)
**Budget:** Unlimited (Tier 1)
**Source:** docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md

---

## Context

WS13 (Single Active Device / Tight Pairing) is the flagship v0.2.1 feature. WS13.1 through WS13.5 are COMPLETE on the current tree. WS13.6 is the final phase: migration, compatibility mode, and test matrix closure.

## Scope

1. **Compatibility mode behavior**
   - Legacy no-device traffic must remain operable during migration window.
   - Implement Phase A (compat mode) vs Phase B (enforced mode) gating.
   - File targets: `core/src/store/relay_custody.rs`, `core/src/transport/swarm.rs`

2. **Upgrade / migration tests**
   - Pre-WS13 store -> WS13 store migration without data loss.
   - Test identity hydration backfills missing `device_id` + `seniority_timestamp`.
   - Test legacy contact loads with `None` default for `last_known_device_id`.
   - Test legacy relay requests without `intended_device_id` still accepted in compat mode.
   - File targets: `core/tests/`, `core/src/identity/store.rs`, `core/src/store/contacts.rs`

3. **WS13 residual risk triage**
   - Create or update `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md` with WS13-specific risks.
   - Carry forward any unresolved risks from `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` that affect WS13.
   - Update `REMAINING_WORK_TRACKING.md` to mark WS13 COMPLETE once WS13.6 lands.

4. **Manual runbook and docs**
   - Document compatibility mode decision point and upgrade behavior.
   - Update canonical docs chain (`DOCUMENT_STATUS_INDEX.md`, `CURRENT_STATE.md`).

## Key Constraints

- Do NOT break v0.2.0 clients. Legacy traffic must pass in compat mode.
- `device_id` is installation-local; must NOT leak into identity backup payload.
- All registration/deregistration requests must be signed (already verified in WS13.3).
- Follow PHILOSOPHY_CANON.md: sovereign mesh, eventual delivery, mandatory relay.

## Acceptance Gates

1. `cargo build --workspace` passes.
2. `cargo test --workspace` passes (including new migration tests).
3. `cargo clippy --workspace -- -D warnings` passes.
4. Pre-WS13 identity stores backfill `device_id` + `seniority_timestamp` on hydrate without rotating key material.
5. Legacy relay requests without `intended_device_id` accepted in compat mode.
6. `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md` exists and is current.
7. `REMAINING_WORK_TRACKING.md` updated to reflect WS13 completion.

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo fmt --all -- --check
```

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
