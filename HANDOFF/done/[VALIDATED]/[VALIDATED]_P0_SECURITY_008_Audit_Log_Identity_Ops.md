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
# BUDGET: 1200
# token_budget: 12000

# P0_SECURITY_008_Audit_Log_Identity_Ops

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 1200s (MIXED tier)
**Phase:** v0.2.1 P0 security
**Source:** docs/historical/plans/PRODUCTION_ROADMAP.md P0.6 (No audit logging) + planfromclaudeforhermes 2 Phase B.2
**Depends on:** P0_BUILD_001

---

## Verified Gap

Per `docs/historical/plans/PRODUCTION_ROADMAP.md` P0.6: "No audit logging  Security-relevant events (identity operations, key operations, block/unblock actions) leave no tamper-evident audit trail."

Note: `core/src/iron_core.rs` already has `export_audit_log` (L1447), `get_audit_log` (L1134), `get_audit_events_since` (L1138), `validate_audit_chain` (L1263)  per `HANDOFF/ACTIVE_LEDGER.md` "Wired" column. The gap is that the **identity operation sites don't call them**.

## Scope (~120 LoC across 4 files)

### Part A: Audit log entry macro (LOC: ~40)

Create `core/src/audit.rs`:

```rust
#[macro_export]
macro_rules! audit_log_entry {
    ($core:expr, $event_type:expr, $($field:tt)*) => {{
        let entry = AuditEntry {
            timestamp: SystemTime::now(),
            event_type: $event_type,
            $($field)*
            ..Default::default()
        };
        $core.audit_log_mut().append(entry);
    }};
}
```

### Part B: Wire identity op sites (LOC: ~50)

In `core/src/identity/keys.rs` and `core/src/identity/mod.rs`, add `audit_log_entry!` calls at:

1. **Key generation** (`generate_keypair`): event_type=`IdentityKeygen`, fields=`{ identity_id, public_key_fingerprint }`
2. **Import** (`import_identity`): event_type=`IdentityImport`, fields=`{ identity_id, source }`
3. **Export** (`export_identity`): event_type=`IdentityExport`, fields=`{ identity_id, format_v1_or_v2 }`
4. **Rotate** (`rotate_keys` if present, else create stub): event_type=`IdentityRotate`, fields=`{ old_identity_id, new_identity_id }`
5. **Block/unblock peer** in `core/src/abuse/`: event_type=`PeerBlock` / `PeerUnblock`, fields=`{ identity_id, peer_id, reason }`

### Part C: Hook audit chain validation into maintenance (LOC: ~30)

In `core/src/iron_core.rs`, `perform_maintenance()`:

```rust
pub fn perform_maintenance(&self) -> Result<()> {
    // ... existing work ...
    let validation = self.validate_audit_chain();
    if !validation.valid {
        warn!("Audit chain validation failed: {:?}", validation.errors);
        // log to error log, do not panic
    }
    Ok(())
}
```

## File Targets

- `core/src/audit.rs` [NEW]
- `core/src/lib.rs` [EDIT  `pub mod audit;`]
- `core/src/identity/keys.rs` [EDIT  audit_log_entry! at keygen, rotate]
- `core/src/identity/mod.rs` [EDIT  audit_log_entry! at import, export]
- `core/src/abuse/manager.rs` [EDIT  audit_log_entry! at block/unblock]
- `core/src/iron_core.rs` [EDIT  wire validate_audit_chain into perform_maintenance]

## Build Verification Commands

```bash
cargo check --workspace
cargo test -p scmessenger-core --lib identity
cargo test -p scmessenger-core --lib abuse
# CLI smoke
cargo run -p scmessenger-cli -- identity create test-id
cargo run -p scmessenger-cli -- identity export
cargo run -p scmessenger-cli -- audit stats  # Should show  1 entry
cargo run -p scmessenger-cli -- audit export | head  # Should show keygen + export entries
```

## Acceptance Gates

1. `cargo test --workspace` passes (920+ existing + new audit tests)
2. New tests cover: each identity op produces exactly one audit entry, audit chain validates after 100 entries, perform_maintenance reports chain validation result
3. `scm audit stats` shows entries for keygen, export
4. Audit entry JSON contains: `timestamp`, `event_type`, `identity_id`, context-specific fields
5. `validate_audit_chain()` returns `valid=true` for empty log and for log with 100 entries
6. Commit: `security: v0.2.1 audit log entries at identity operations`

## CRITICAL

You are forbidden from considering this task 'complete' until you execute the `git mv` to move this file from `todo/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Routing Tags
[REQUIRES: RUST_CORE] [REQUIRES: GLM-5.1] [DEPENDS_ON: P0_BUILD_001]
