# TASK: A-05 iOS Receipt Unification (U6)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: DISPATCH-READY
Model: Qwen CODER tier
Scope: Tight (platform unification, receipt pipeline)
Prerequisite: A-04 pattern (Android mirror)

## Objective

Port core's unified `encode_receipt()` / `decode_receipt()` to iOS via UniFFI bindings (Swift side). Mirror A-04 for iOS.

## Implementation

### 1. Verify UniFFI Swift Bindings
- Confirm Swift type definitions for Receipt, EncodeReceipt, DecodeReceipt exist (auto-generated from api.udl)
- Match function signatures to core Rust

### 2. Swift Integration
Locate receipt handling in iOS bridge:
- `ios/*/Delegates/CoreDelegateImpl.swift` (or equivalent)
- `ios/*/Transport/SmartTransportRouter.swift` (or equivalent)
- Update to call unified core functions via generated FFI
- Consolidate any duplicate receipt encoding/decoding

### 3. Test Coverage
Add unit test in `ios/*/Tests/.../ReceiptUnificationTest.swift`:
- Create receipt via core's `encode_receipt()`
- Parse via core's `decode_receipt()`
- Verify round-trip integrity

### 4. Success Criteria
- Xcode project compiles cleanly (or verify on macOS CI if available)
- Swift test passes
- No local Swift-side encoding/decoding
- Diff applies cleanly via `--mode diff --apply --verify`

## Files to Modify

- `ios/*/Delegates/CoreDelegateImpl.swift` (or equivalent delegate)
- `ios/*/Transport/SmartTransportRouter.swift` (or equivalent router)
- `ios/*/Tests/.../ReceiptUnificationTest.swift` (NEW)

## Estimate
200 LOC, 30min (mirrors A-04, no complex platform-specific logic)

## Review Gate
None (platform-specific unification). Verify compiles + tests pass.

## Execution
Dispatch to Qwen CODER with `--mode diff --apply --verify`.

## Handoff
Move this file to `HANDOFF/done/` ON COMPLETION via `mv` command.

## Note
A-04 (Android) should be completed first for reference pattern; A-05 can proceed in parallel.
