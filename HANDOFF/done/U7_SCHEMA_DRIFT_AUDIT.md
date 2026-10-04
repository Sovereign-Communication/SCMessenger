# TASK: U7 — Schema drift audit (persistence formats)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Tier:** [OPUS+] → [SONNET] — investigation + implementation  
**Delegation:** `/scmqwen` → THINK (investigation), then CODER (implementation)  
**Priority:** F2/F3 gate (backlog, non-blocking for F0/F1)  
**Related:** UNIFICATION_AUDIT_FINDINGS.md (lines 74–95), F2 phase  

---

## Problem

Data persistence formats have drifted across the codebase. Identified leads (from 2026-07-12 audit, not yet independently verified):

1. **Ledger format inconsistency:** `serde_json::to_string_pretty()` (disk format) vs wire-exchange format — are they actually different? Do both deserialize correctly?

2. **History record schema versioning:** `adjust_legacy_timestamps()` function exists, suggesting data schema has changed before. Are there OTHER schema versions lurking? Will an old ledger/history DB from v1.0.0 farm deployment still deserialize in v1.1?

3. **Contact-identifier resolution:** Three ways to identify a peer (peer ID vs public key vs identity_id). CLI commands do format detection; Android has a separate instance. Are they consistent?

4. **Platform-specific persistence:** Does Android's `MeshStore` vs iOS's equivalent vs CLI's sled backend all use the same serialization format for shared data structures (Message, Receipt, Ledger entry)?

---

## Solution

Two-phase investigation + consolidation:

**Phase 1 (Investigation):** Read codebase, trace each persistence path, produce a schema versioning map.

**Phase 2 (Implementation):** If drift is found, decide: standardize on one format, or add explicit versioning + migration logic.

### Implementation spec

**Phase 1: Investigation**

Read and grep:

1. **Ledger format:** `core/src/store/ledger.rs`, `core/src/transport/ledger_exchange.rs`
   - How is ledger serialized to disk? (grep for `serde_json::to_string`)
   - How is ledger exchanged on wire? (grep for `peer_exchange`, look for alternative format)
   - Do both deserialize correctly? Test: load old ledger.json, verify it works

2. **History record schema:** `core/src/store/history.rs`, `android/app/src/main/kotlin/.../database/`
   - Does `adjust_legacy_timestamps()` exist? If so, what migration did it perform?
   - Are there OTHER migrations hiding? (grep for schema version, table alteration, migration logic)
   - What happens if you open a database from v0.3.x with v1.0.0 code?

3. **Contact-identifier resolution:** `cli/src/main.rs` (contact add/lookup), `android/app/.../MeshRepository.kt`, `iOS/.../MeshViewModel.swift`
   - How does CLI detect if input is peer ID vs public key? (grep for validation logic)
   - How does Android do the same?
   - Are they the same, or does one accept formats the other rejects?

4. **Platform-specific persistence:** `core/src/store/`, `cli/src/`, `android/app/src/`, `iOS/`
   - Are Message/Receipt/Ledger-entry serialization formats IDENTICAL across all platforms?
   - Or does each platform have custom serialization code?

**Deliverable from Phase 1:** A report file `SCHEMA_VERSIONING_MAP.md` documenting:
- Each persistence format (ledger, history, message, receipt, ledger exchange)
- Whether it has explicit versioning
- Whether disk format == wire format
- Any platforms using different formats
- Any migrations already in place

**Phase 2: Implementation (if drift is found)**

For each drift:
- Option A: Standardize on one format (likely serde_json for human readability)
- Option B: Add explicit versioning + migration logic (if backwards compat is critical)
- Create tests that:
  - Load v0.3.x data on v1.0.0 code (verify migration or explicit error)
  - Verify ledger_exchange uses the same format as disk persistence
  - Verify receipts serialize the same way on CLI and mobile

---

## Acceptance criteria (Investigation phase)

- [ ] `SCHEMA_VERSIONING_MAP.md` created in `HANDOFF/docs/` (or `docs/`), documenting each persistence format
- [ ] For each format: "explicit version tag?" and "disk vs wire match?" columns filled
- [ ] Drift identified (or explicitly confirmed as "no drift, all formats consistent")
- [ ] Recommendations written: which formats to standardize, which need migration

---

## Acceptance criteria (Implementation phase, if needed)

- [ ] Any format inconsistencies resolved (either via standardization or explicit versioning)
- [ ] Migration tests written: load old data on new code, verify success or graceful error
- [ ] Ledger exchange format verified to match disk format (via test)
- [ ] Receipt format unified across platforms (this is U4, but verify ledger format too)
- [ ] `cargo test --workspace --no-run` passes

---

## Notes

- **Not blocking F0/F1:** This is a backlog audit. F0/F1 focus on delivery truth (A1-A4) and infrastructure (B-lane).
- **Farm relevance:** Farm deployment is long-lived. Data format stability is critical for updates and multi-node sync. Better to discover drift now than in the field.
- **Investigation tools:** Use `scripts/delegate_task.py --mode analyze` (or native agent) to trace persistence paths; don't trust manual grepping for this scope.
- **PQC-10 regen:** If any format drift is discovered, re-verify after PQC-10 lands (identity key format changes; may cascade to serialization).

