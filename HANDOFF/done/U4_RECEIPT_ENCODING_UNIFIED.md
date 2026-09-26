# TASK: U4 — Receipt encoding unified (encode_receipt / decode_receipt)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Tier:** [SONNET] — design + implementation  
**Delegation:** `/scmqwen` → CODER model  
**Priority:** F0 gate (blocks A2 receipt-round-trip fix, CRITICAL delivery bug)  
**Related:** A2, CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md, UNIFICATION_AUDIT_FINDINGS.md  
**Blocker for:** A2 receipt round-trip fix

---

## Problem

Receipt format inconsistency across codebase (confirmed root cause of CRITICAL delivery bug):

1. `core/src/message/types.rs:181` — `Message::receipt()` bincode-serializes a `Receipt` struct
2. `cli/src/main.rs` — deserializes receipts expecting bincode
3. `core/src/transport/swarm.rs::prepare_receipt()` — outputs JSON
4. Android receives JSON, tries to decode as bincode, fails → delivery marked as failed, message deleted after 12 retries

**Result:** Farm-mates see "sent message failed" even though it was delivered. Trust poison for adoption.

Fix landing in A2; this task unifies the encoding format once so A2 can be clean.

---

## Solution

Create two canonical functions in `core/src/message/types.rs`:
- `encode_receipt(receipt: &Receipt) -> Result<Vec<u8>>` — serialize to JSON bytes
- `decode_receipt(buf: &[u8]) -> Result<Receipt>` — deserialize from JSON bytes

All platforms use these. Zero format variation.

### Implementation spec

**File: `core/src/message/types.rs`**

Locate the `Receipt` struct definition (likely early in the file). After the impl block for Receipt, add:

```rust
use serde_json;

/// Serialize a Receipt to JSON bytes (the canonical wire format).
/// 
/// This is the ONLY way receipts should be serialized anywhere in the codebase.
/// All platforms (CLI, Android, iOS, WASM) use this function.
pub fn encode_receipt(receipt: &Receipt) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(serde_json::to_vec(receipt)?)
}

/// Deserialize a Receipt from JSON bytes (the canonical wire format).
/// 
/// This is the ONLY way receipts should be deserialized anywhere in the codebase.
/// All platforms (CLI, Android, iOS, WASM) use this function.
pub fn decode_receipt(buf: &[u8]) -> Result<Receipt, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(buf)?)
}
```

Export from `core/src/lib.rs`:
```rust
pub use crate::message::types::{encode_receipt, decode_receipt};
```

**File: `core/src/transport/swarm.rs` — replace prepare_receipt**

Locate `prepare_receipt()` function. It currently outputs JSON manually. Replace internals to use the unified function:

- **Before:** Manual serde_json::to_vec(...) or hand-built JSON
- **After:** `encode_receipt(&receipt)?`

Update call sites: anywhere that calls `prepare_receipt()` will now get bytes guaranteed to be valid JSON.

**File: `core/src/message/types.rs:181` — replace Message::receipt()**

Locate the `Message::receipt()` method that currently bincode-serializes. Replace:

- **Before:** `bincode::serialize(&self_receipt)?`
- **After:** `encode_receipt(&self_receipt)?`

**File: `cli/src/main.rs` — replace receipt deserialization**

Search for any manual bincode deserialization of receipts. Replace with `decode_receipt()`:

- **Before:** `bincode::deserialize::<Receipt>(&buf)?`
- **After:** `decode_receipt(&buf)?`

(Grep for `Receipt` + `bincode` to find all sites.)

**File: `cli/src/server.rs` (if it exists) — WebSocket receipt handler**

Search for any receipt handling on the WS path. Ensure it uses `encode_receipt()` when sending, `decode_receipt()` when receiving.

---

## Acceptance criteria

- [ ] `encode_receipt()` and `decode_receipt()` defined in `core/src/message/types.rs`, exported from `lib.rs`
- [ ] All `Message::receipt()` calls use `encode_receipt()` (zero bincode serialization)
- [ ] All receipt deserialization uses `decode_receipt()` (zero manual bincode deserialization)
- [ ] Grep finds 0 remaining `bincode` references to `Receipt` outside tests/historical code
- [ ] `prepare_receipt()` in swarm.rs uses the unified function
- [ ] `cargo test --workspace --no-run` passes (compile gate)
- [ ] Integration test (A2): create receipt in core, encode, decode on CLI, verify round-trip succeeds

---

## Notes

- **Format choice:** JSON is canonical. It's human-readable (debugging), already used in one path, and serde_json is stable across platforms.
- **Error handling:** Return `Result<Vec<u8>, Box<dyn std::error::Error>>` for compatibility; A2 will decide whether to propagate or unwrap given context.
- **A2 follows this:** Once these functions exist, A2 lands the fix that actually calls them in the receipt round-trip path. This task just unifies the format.
- **Android/iOS:** After C-lane iOS lands, same functions will be called from UniFFI-generated Swift/Kotlin wrappers. Zero platform variation.

