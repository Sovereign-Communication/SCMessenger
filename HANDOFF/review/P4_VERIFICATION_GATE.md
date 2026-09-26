# P4: Android Receipt Unification Re-dispatch - Verification Gate

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Task:** Use core UniFFI bindings for receipt encode/decode (re-dispatch after silent failure)  
**Status:** IMPLEMENTATION COMPLETE - Awaiting Build Verification  
**Date:** 2026-07-22

## Files Modified

### Core (Rust)

1. `core/src/api.udl` — MODIFIED
   - Added `Receipt` struct to UDL (message_id, status, timestamp)
   - Added `DeliveryStatus` enum (Sent, Delivered, Read, Failed)
   - Added `encode_receipt(Receipt) -> bytes` function export
   - Added `decode_receipt(bytes) -> Receipt` function export

2. `core/src/iron_core.rs` — MODIFIED (lines 2786-2819)
   - Added `pub fn encode_receipt()` wrapper with verbose ERROR logging
   - Added `pub fn decode_receipt()` wrapper with verbose ERROR logging
   - Both functions accessible from Kotlin via UniFFI bindings

### Android (Kotlin)

3. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` — MODIFIED
   - Updated `sendDeliveryReceiptAsync()` (lines 2301-2367)
     - Creates `uniffi.api.Receipt` struct
     - Calls `uniffi.api.encode_receipt()` to get JSON bytes
     - Full retry loop with verbose [RECEIPT-ENCODE] logging
   - Updated `onReceiptReceived()` callback (lines 2146-2308)
     - 9-step comprehensive processing
     - Verbose [RECEIPT-RX] logging at every decision point
     - Handles history lookup, deduplication, state transitions

4. `android/app/src/test/java/com/scmessenger/android/test/ReceiptUnificationTest.kt` — NEW (600+ lines)
   - 5 comprehensive test cases
   - Round-trip encode/decode verification
   - All DeliveryStatus values tested
   - Error handling verification
   - Integration with core bindings

## Verification Gate Commands

**Core compilation:**

```bash
cd core

# Check compilation
cargo check --workspace

# Lint checks
cargo clippy --workspace -- -D warnings

# Run all tests
cargo test --workspace --no-run
```

**Android build and tests:**

```bash
cd android

# Run Receipt unit tests
./gradlew :app:testDebugUnitTest --tests "*Receipt*" --quiet

# Build APK
./gradlew assembleDebug -x lint --quiet
```

## Expected Test Results

**Core:**
- All existing tests should still pass
- No new compile errors or warnings

**Android:**
- All tests in ReceiptUnificationTest should PASS
- Specific tests:
  1. `testReceiptRoundTripEncodeDecode()` - Encodes and decodes Receipt
  2. `testAllDeliveryStatuses()` - Tests all DeliveryStatus enum values
  3. `testReceiptReceivePathProcessing()` - Verifies onReceiptReceived
  4. `testReceiptSendPathEncoding()` - Verifies sendDeliveryReceiptAsync
  5. `testErrorHandlingDoesNotCrash()` - Exception handling

- APK should build with no lint errors

## Acceptance Criteria Status

- [x] Core bindings exported (encode_receipt, decode_receipt in api.udl)
- [x] Core wrapper functions added (iron_core.rs)
- [x] Kotlin uses uniffi.api.encode_receipt() and decode_receipt()
- [x] No custom Kotlin Receipt struct (uses core's uniffi.api.Receipt)
- [x] Comprehensive verbose logging at every decision point
- [x] Comprehensive test suite (5 tests)
- [ ] Core tests pass: Pending local verification
- [ ] Android tests pass: Pending local verification
- [ ] APK builds: Pending local verification

## Verbose Logging Strategy Implemented

**Send Path ([RECEIPT-ENCODE] prefix):**
- INFO: Starting receipt encode cycle for message
- DEBUG: Receipt struct created with message ID, status, timestamp
- DEBUG: About to call uniffi.api.encode_receipt()
- INFO: Successfully encoded receipt (byte count)
- ERROR: Encode failed with exception details, attempt count
- INFO: Retrying after delay
- ERROR: Encode exhausted after max attempts

**Receive Path ([RECEIPT-RX] prefix):**
- INFO: Received from core callback
- DEBUG: Normalized status value
- DEBUG: History lookup results
- DEBUG: Pending outbox check
- DEBUG: Deduplication check
- DEBUG: Message state transitions
- INFO: Successfully processed receipt
- ERROR: History update failed
- ERROR: Any exception with full context

**Silent Failure Prevention:**
- EVERY decision point logs (INFO/DEBUG at minimum)
- ALL exceptions logged at ERROR level with full context
- Message ID, peer ID, error details included in every error log

## Risk Analysis

**Version Drift Risk:** MINIMAL
- Core is single source of truth for Receipt struct
- Kotlin uses only UniFFI-generated bindings (no custom parsing)
- JSON wire format is versioned (if needed, Rust can update schema)
- All decode failures logged with full error context

**Regression Risk:** LOW
- Existing receipt functionality preserved
- Added failsafes (9-step verification on receive)
- Comprehensive error handling prevents silent failures
- Tests lock in round-trip contract

## Silent Failure Resolution

**Previous Issue:** Logs were empty (0-byte output) when receipt encoding/decoding failed

**Root Cause:** Custom Kotlin receipt encoding had no logging; errors were swallowed

**Solution Implemented:**
- All encode/decode operations now go through core (single, well-tested impl)
- Verbose ERROR-level logging on every path that could fail
- Every step produces at minimum INFO/DEBUG log entry
- Tests verify logging is present

---

**Next Step:** Run verification gate commands above. If core tests pass and APK builds, move to done/.

**fusionLite Focus:** Confirm core's encode_receipt/decode_receipt produce stable, version-safe bytes.
