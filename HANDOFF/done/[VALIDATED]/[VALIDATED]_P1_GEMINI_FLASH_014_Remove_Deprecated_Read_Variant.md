<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Triage Decision -- 2026-06-11

**Status:** ready
**Bucket:** pending-dispatch
**Cross-ref:** `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` 4 (Identity)
**Decided by:** Hermes Agent (overseer) post-session audit
**Routing model:** `gemini-3.5-flash:cloud` (1-line Rust annotation)
**Rationale:** Per `PLAN_VERIFICATION_2026-06-11.md` 4: `DeliveryStatus::Read` is `#[deprecated(note = "Zero-Status Architecture: Read receipts are no longer emitted or displayed")]` but the variant is still on the wire. Remove the `Read` variant enum entry entirely, mark the field as removed. ~3 LoC of Rust + 1 line in api.udl. Trivial. Flash handles Rust enum changes well.

---

# MODEL: gemini-3.5-flash:cloud
# BUDGET: 300
# token_budget: 5000

# P1_GEMINI_FLASH_014  Remove Deprecated `Read` DeliveryStatus Variant

**Status:** VERIFIED REMAINING WORK
**Agent:** gemini-coder (Gemini 3.5 Flash)
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 P1  Zero-Status cleanup
**Source:** `HANDOFF/STATE/PLAN_VERIFICATION_2026-06-11.md` 4 (claim vs reality delta)
**Depends on:** none

---

## Verified Gap

`core/src/message/types.rs:22-31` defines `DeliveryStatus::Read` as deprecated. Per the verification doc: "Read receipts are no longer emitted or displayed" per the deprecation note. But the variant still exists in:
- The enum (`types.rs`)
- The UDL surface (`api.udl:14`)
- All `match` statements that handle `DeliveryStatus` (cascade risk)

Removing the variant requires a careful sweep of every `match DeliveryStatus` in the codebase.

## Scope (~10 LoC across ~5 files)

### Part A: Remove the variant (LOC: ~3)

In `core/src/message/types.rs`:
- Delete the `Read` arm of the `DeliveryStatus` enum
- Delete the `#[allow(deprecated)]` annotations where they reference `Read`

### Part B: Update UDL (LOC: ~1)

In `core/src/api.udl`:
- Remove `Receipt` from the UDL `MessageType` enum (per verification: still in wire)

### Part C: Sweep `match` blocks (LOC: ~6, distributed)

`grep -rn "DeliveryStatus::Read" core/` returns the call sites. Remove or replace each `Read` arm with `_ => {}` or unreachable!().

## File Targets

- `core/src/message/types.rs` [EDIT  delete variant, ~3 LoC]
- `core/src/api.udl` [EDIT  remove Receipt, ~1 LoC]
- `core/src/**/*.rs` [EDIT  sweep match arms, ~6 LoC total across N files]

## Build Verification

```bash
cd ~/Documents/Github/SCMessenger
cargo check --workspace 2>&1 | tee /tmp/check-$(date +%s).log
grep "DeliveryStatus::Read" core/src -r  # should be empty
cargo test -p scmessenger-core --lib message 2>&1 | tail -20
```

## Acceptance Gates

1. `cargo check --workspace` 0 errors
2. `grep "DeliveryStatus::Read" core/ -r` returns 0 hits (variant fully removed)
3. `cargo test -p scmessenger-core --lib` all pass
4. `cargo test --workspace --no-run` 0 errors (cascade-clean)

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST] [REQUIRES: GEMINI_FLASH] [SERIAL_NEEDED: false] [PRIORITY: 14]
