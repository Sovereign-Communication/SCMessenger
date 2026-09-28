# DISPATCH WAVE: Crypto + CODER Implementation (Qwen CODER Tier)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: READY FOR DISPATCH
Tier: CODER (Qwen CODER + Groq FLASH for validation)
Wave: A (4 parallel CODER tasks)
Authority: SCMessenger orchestrator (2026-07-17 autonomous drive)

## CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.

---

## Task Set: PQC-07 Ratchet Wiring + Platform Unification

### 1. PQC-07: WIRE_RATCHET_STEP (Crypto, Mandatory Adversarial Review)

**File:** `HANDOFF/todo/PQC_07_WIRE_RATCHET_STEP.md`

**Scope:** Wire PQ ratchet-step cadence into live encrypt/decrypt path:
- **Sender (encrypt_message_ratcheted):** Call `session.perform_pq_ratchet_step()` on DH ratchet boundary. Populate `pq_kem_ciphertext`/`pq_encaps_key` in envelope with fresh values.
- **Receiver (decrypt_message_ratcheted_v2):** Read incoming `envelope.pq_kem_ciphertext`/`pq_encaps_key`, call `handle_incoming_pq_fields()` with decapsulation.
- **DH ratchet boundary:** Check `RatchetSession::encrypt()` for existing DH-step trigger (Signal-style = receiving-side only); hook PQ step to same trigger OR implement cadence every N messages (grep MAX_RATCHET_STEPS/rekey constants first).
- **Pre-confirmation bootstrap:** Keep existing behavior (first message carries bootstrap ciphertext), add cadence alongside (do not replace).

**Success Criteria:**
- integration_pq_session.rs: all 6 tests pass (cadence assertions green)
- integration_e2e.rs + integration_drift_mule.rs: no custody field regressions
- cargo test --workspace --no-run: clean
- `--mode diff --apply --verify "cargo check --workspace"`: exit 0

**Review:** [AUDIT-GATE] Fusion adversarial panel UNANIMOUS PASS required (crypto-touching). Use rule from `docs/ORCHESTRATION.md` Section 10.

**Estimate:** 250 LOC, 45min.

---

### 2. A-04: U5 Android Receipt Unification (CODER)

**File:** `HANDOFF/todo/` (extract from queue or create from spec below)

**Scope:** Port core's unified `encode_receipt()`/`decode_receipt()` to Android via UniFFI:
- Call core's unified functions from Kotlin MeshRepository
- Remove duplicate/legacy receipt listeners (if any exist)
- Farm gate: FD-10 (delivery-truth audit)

**Success Criteria:** Kotlin compiles, receipt round-trip test passes, no regressions in Kotlin-side state machine.

**Estimate:** 200 LOC, 30min.

---

### 3. A-05: U6 iOS Receipt Unification (CODER)

**File:** `HANDOFF/todo/` (mirror of A-04 for Swift)

**Scope:** Same as A-04 but for Swift/CoreDelegateImpl.swift.

**Estimate:** 200 LOC, 30min.

---

### 4. D-05: Unwrap/Panic Hardening (Parallel, CODER x2)

**File:** `HANDOFF/todo/D-05_unwrap_panic_hardening_v1.md`

**Scope:** Replace ~60 `unwrap()`/`panic!` sites with Result/logged-defaults:
- Priority zones: FFI boundary (mobile_bridge.rs), startup path, crypto, storage
- Use `Result` types + logging for non-critical paths
- Critical panics (crypto invariant failures) = document with safety comment, not removed

**Success Criteria:**
- Diff applies cleanly via `--mode diff --apply --verify "cargo check --workspace"`
- No functional regressions (all tests pass)
- Documented critical panics remain

**Estimate:** 600 LOC, 90min. Dispatch as 2x parallel (file-sharded): one agent handles FFI/startup, second does crypto/storage.

---

## Dispatch Rules

- **Mode:** `--mode diff --apply --verify "cargo check --workspace"`
- **Fusion gate (PQC-07 only):** Unanimous PASS required before commit
- **Build verification:** `cargo test --workspace --no-run` must pass
- **File movement:** Move your task file(s) to `HANDOFF/done/` ON COMPLETION, not during work
- **Git discipline:** Do not commit; orchestrator will batch-commit all completed tasks

## Next Wave (Queued, Blocked on This One)

After these 4 tasks land:
- **B-01:** PQC-04 suite negotiation verify (frozen until E-01c + PQC-07 done)
- **B-02..B-07:** PQC wave 2 (frozen on E-01c)
- **D-01:** FARM_TESTRUNNER_REST_API_GAP (already done, verify + close)
- **D-02/D-03/D-04:** Platform test wiring (Android/iOS/Emulator)
- **C-05/C-06:** Farm-sim validation tasks

## Execution

Dispatch to `qwen3-coder:480b:cloud` with `--add-dir`, `--max-cost 0.05`, `--mode diff --apply --verify`.

PQC-07 requires Fusion panel (3-model unanimous). A-04/A-05/D-05 route to standard Qwen CODER.
