# P4: Android Receipt Unification Re-dispatch

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Ticket Status:** Open (dispatch to Qwen CODER, re-dispatch after silent failure)
**Tier:** [SONNET]
**Scope:** v0.4.0 blocker
**Language:** Kotlin

## Background

Previous dispatch (2026-07-21) produced silent 0-byte log failure (no output, no error, task disappeared).

Objective: Use core's unified `encode_receipt()` / `decode_receipt()` via UniFFI bindings instead of custom Kotlin serialization.

**Problem:** Receipt encoding is fragmented:
- Kotlin has its own Receipt struct
- Core (Rust) has its own Receipt encoding in `core/src/crypto/encrypt.rs` or `core/src/iron_core.rs`
- When CLI sends a receipt to Android, deserialization mismatches occur
- When Android sends a receipt to CLI, CLI's decoder fails
- Silent failures cascade (logs are empty, message appears to be sent, but receipt never arrives)

**Solution:** Let core be the single source of truth. Android uses UniFFI bindings to encode/decode.

## Specification

### Step 1: Locate Core Bindings
- Find `encode_receipt(msg: Message) -> Vec<u8>` in core (likely in `core/src/iron_core.rs` or `core/src/uniffi.rs`)
- Find `decode_receipt(bytes: Vec<u8>) -> Receipt` in core (same location)
- Verify both are exported via `#[uniffi::export]` in `core/uniffi.toml` or proc macro

### Step 2: Replace Kotlin-Side Receipt Logic
- In `MeshRepository.kt`, find any custom receipt parsing/encoding code
- Replace with calls to: `IronCore.encodeReceipt(msg)` and `IronCore.decodeReceipt(bytes)`
- Delete any custom Receipt struct if it exists (use core's Receipt type, bridged via UniFFI)

### Step 3: Wire into Message Listener
- Inbound message handler: if message is marked as a receipt, call `IronCore.decodeReceipt(payload)`
- Outbound receipt send: call `IronCore.encodeReceipt(originalMsg)`, then send via transport
- Error handling: if decode fails, log at ERROR level and move to failed-message queue (do NOT silently ignore)

### Step 4: Test Round-Trip
- Write Kotlin test `ReceiptUnificationTest.kt`:
  - Send message via transport (MockTransport or real relay)
  - Receive receipt from core (via `IronCore.decodeReceipt`)
  - Verify all fields match original message (timestamp, sender, message ID, etc.)

## Files to Edit

- `android/app/src/main/kotlin/com/scmessenger/android/data/MeshRepository.kt` (receipt send/receive)
- `android/app/src/main/kotlin/com/scmessenger/android/transport/SmartTransportRouter.kt` (if it has receipt handling)
- `android/app/build.gradle` (if FFI bindings need linking)
- New test: `android/app/src/test/kotlin/com/scmessenger/android/data/ReceiptUnificationTest.kt`

## Acceptance Criteria

1. Kotlin tests compile and pass: `./gradlew :app:testDebugUnitTest --tests "*Receipt*" --quiet`
2. APK builds: `./gradlew assembleDebug -x lint --quiet`
3. No custom Kotlin Receipt struct remains (or if it remains, it's only for local UX, not encoding/decoding)
4. All encode/decode calls go through IronCore UniFFI bindings
5. fusionLite review: confirm no version-drift risk between Kotlin and Rust receipt formats

## Notes

- This is a re-dispatch after previous silent failure: request strong verbose output from model (ask for logging at every step)
- If core's encode_receipt or decode_receipt don't exist, escalate to Qwen THINK for design (may need to add them first)
- Silent failure is the enemy: add ERROR-level logging to every decode path that could fail

---

**Dispatch to:** Qwen CODER (re-dispatch)  
**Model:** qwen3-coder-plus  
**fusionLite verification:** Yes (version-drift risk, design review)  
**Move to done/ when:** Tests pass, APK builds, fusionLite approves  
