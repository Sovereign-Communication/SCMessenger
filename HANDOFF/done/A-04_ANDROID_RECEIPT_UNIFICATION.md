# TASK: A-04 Android Receipt Unification (U5)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: DISPATCH-READY
Model: Qwen CODER tier
Scope: Tight (platform unification, receipt pipeline)

## Objective

Port core's unified `encode_receipt()` / `decode_receipt()` to Android via UniFFI bindings. Remove duplicate/legacy receipt listeners.

## Current State

- Core unified functions exist: `core/src/crypto/receipt.rs` + `core/src/store/receipt_store.rs`
- Android Kotlin has legacy/duplicate receipt handling scattered across MeshRepository.kt + transport layer
- UniFFI bindings already exist for receipt types

## Implementation

### 1. Verify UniFFI Bindings
Read `core/api.udl` - search for receipt-related exports (EncodeReceipt, DecodeReceipt, Receipt types). Confirm signatures match core Rust `encode_receipt()` / `decode_receipt()` functions.

### 2. Kotlin Integration (android/app/...)
Locate receipt handling in:
- `MeshRepository.kt` - update to call unified FFI functions instead of local encode/decode
- `TransportManager.kt` / related - check for duplicate receipt listeners; consolidate to single pipeline

### 3. Test Coverage
Add unit test in `android/app/src/test/.../ReceiptUnificationTest.kt`:
- Create receipt message via core's `encode_receipt()`
- Parse via core's `decode_receipt()`
- Verify round-trip integrity
- No Kotlin-side local encoding/decoding

### 4. Success Criteria
- `cd android && ./gradlew assembleDebug -x lint` PASS
- `./gradlew :app:testDebugUnitTest` PASS (new receipt test + no regression)
- All duplicate listeners removed or consolidated
- Diff applies cleanly via `--mode diff --apply --verify "cargo check --workspace"`

## Files to Modify

- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`
- `android/app/src/test/java/com/scmessenger/android/ReceiptUnificationTest.kt` (NEW; use --allow-new-file)
- `core/api.udl` (verify, no changes needed if bindings exist)

## Estimate
200 LOC, 30min

## Review Gate
None (platform-specific unification, no crypto). Verify compiles + tests pass.

## Execution
Dispatch to Qwen CODER with `--mode diff --apply --verify "cd android && ./gradlew assembleDebug -x lint"`.

## Handoff
Move this file to `HANDOFF/done/` ON COMPLETION via `mv` command.

## 2026-07-28 closure (orchestrator takeover audit)
Moved to done/ on authority of _QUEUE.md status-correction header (COMPLETED 2026-07-21) plus ledger entry A-04-receipt-unification result=ok. compileDebugKotlin re-verified 2026-07-28; local Robolectric test-execution hangs are a known infra issue (CI green on main 2026-07-24).
