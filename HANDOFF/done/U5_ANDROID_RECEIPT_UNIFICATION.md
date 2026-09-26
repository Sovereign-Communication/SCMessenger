# TASK: U5 — Android receipt handling unified via UniFFI

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET] — platform-specific implementation  
**Delegation:** `/scmqwen` → CODER model (Kotlin)  
**Priority:** F1 gate (after C-lane iOS building, after U4 receipt encoding lands)  
**Related:** A2, U4, C-lane iOS (blocked until this lands)  
**Dependency:** U4 landed (unified receipt encoding), A2 landed (core fix)

---

## Problem

Android's receipt handling is platform-specific Kotlin code. Once U4 (unified receipt encoding) and A2 (core receipt round-trip fix) land, Android must use the same encode/decode functions as CLI and iOS, via UniFFI bindings.

Currently, there is no FFI exposure for `encode_receipt`/`decode_receipt`. This task wires them into the UniFFI surface so Android and iOS can call them.

---

## Solution

1. Export `encode_receipt` and `decode_receipt` from `core/src/api.udl` (UniFFI interface definition)
2. Generate Kotlin bindings via `gen_kotlin` (behind `gen-bindings` feature)
3. Update `MeshRepository` and receipt-handling code to use the unified functions
4. Verify round-trip: sent message → receipt → Android displays correct status

### Implementation spec

**File: `core/src/api.udl`**

Add to the interface definitions (after other exported functions):

```idl
namespace scmessenger_core {
    // Receipt encoding/decoding (unified format across all platforms)
    
    /// Serialize a Receipt to JSON bytes (canonical wire format).
    /// Throws an error if serialization fails.
    [Throws=CoreError]
    bytes encode_receipt(Receipt receipt);
    
    /// Deserialize a Receipt from JSON bytes.
    /// Throws an error if the data is not valid JSON or the structure is wrong.
    [Throws=CoreError]
    Receipt decode_receipt(sequence<u8> data);
};
```

Ensure `Receipt` struct is already exposed (it should be if it's part of the message API). If not, add:

```idl
dictionary Receipt {
    string message_id;
    string sender_id;
    u64 timestamp;
    // ... other fields
};
```

**File: `core/src/lib.rs` — UniFFI exposure**

If `encode_receipt` and `decode_receipt` are in `core/src/message/types.rs`, ensure they're re-exported at the top level:

```rust
pub use crate::message::types::{encode_receipt, decode_receipt};
```

**Build step:**

Regenerate Kotlin bindings (after code changes):

```bash
cd core
cargo build -p scmessenger-core --features gen-bindings --target aarch64-linux-android
# or via the gen_kotlin binary if it exists
```

**File: `android/app/src/main/kotlin/.../MeshRepository.kt` (or equivalent)**

Update receipt-handling code:

- **Before:** Manual receipt parsing / platform-specific JSON handling
- **After:** `scmessenger_core.encodeReceipt(receipt)` and `scmessenger_core.decodeReceipt(data)`

Exact location depends on where Android currently processes receipts. Common sites:
- Message-received callback that processes `on_receipt_received`
- WebSocket message handler that deserializes receipts
- Storage layer that serializes receipts to disk

(Grep for `receipt` in Android code to locate all sites.)

---

## Acceptance criteria

- [ ] `encode_receipt` and `decode_receipt` exported from `api.udl`
- [ ] Kotlin bindings regenerated and compile
- [ ] Android code updated to use `scmessenger_core.encodeReceipt()` / `scmessenger_core.decodeReceipt()` (zero platform-specific JSON parsing)
- [ ] Integration test: send message from Android, receive receipt from CLI, verify Android displays `DELIVERED(receipt-verified)` status
- [ ] `./gradlew assembleDebug` succeeds (no lint/build errors)
- [ ] No behavior change (same receipt data, just unified encoding path)

---

## Notes

- **Blocked by:** C-lane iOS must be building first (to regenerate bindings once, not twice)
- **Unblocks:** iOS (same bindings, same functions, just Swift FFI wrapper instead of Kotlin)
- **Farm gate:** FD-10 (delivery-truth audit) depends on this: "zero false 'failed' for delivered messages" requires Android to use the same receipt format as CLI
- **Emulator ready:** Test this on `scm_pixel_34` AVD after landing

