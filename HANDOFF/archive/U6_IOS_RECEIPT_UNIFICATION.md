# TASK: U6 — iOS receipt handling unified via UniFFI

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Tier:** [SONNET] — platform-specific implementation  
**Delegation:** `/scmqwen` → CODER model (Swift)  
**Priority:** F2 gate (after C-lane iOS parity, after U5 Android)  
**Related:** A2, U4, U5, C-lane iOS  
**Dependency:** U4 landed (unified receipt encoding), C-lane iOS building, U5 landed (Android proof)

---

## Problem

iOS's receipt handling will use the same UniFFI bindings as Android (generated in U4/U5). This task ensures iOS implements the same contract: use `encodeReceipt` and `decodeReceipt` from core, not platform-specific JSON.

This is the final platform-parity task: once U6 lands, all three platforms (CLI, Android, iOS) use identical receipt encoding.

---

## Solution

Update iOS `MeshRepository` and receipt-handling code (Swift) to call the unified `encodeReceipt` and `decodeReceipt` functions from UniFFI bindings.

### Implementation spec

**File: `iOS/SCMessenger/Views/ChatView.swift` (or equivalent receipt handler)**

Replace any platform-specific receipt JSON handling:

- **Before:** Manual JSON encoding / Codable structs for Receipt
- **After:** `ScmessengercoreLib.encodeReceipt(receipt)` and `ScmessengercoreLib.decodeReceipt(data)`

Search for:
- `Codable` extensions on `Receipt` (delete if they exist — core provides the canonical format)
- `JSONEncoder().encode(receipt)` → use `encodeReceipt()`
- `JSONDecoder().decode(Receipt.self, from: data)` → use `decodeReceipt()`

Common locations:
- Message model / ViewModel that handles receipts
- Storage layer persisting receipts to disk
- Network layer sending/receiving receipts over WebSocket

(Grep for `receipt` or `Receipt` in iOS Swift code to find all sites.)

**File: `iOS/SCMessenger/Models/Message.swift` (or equivalent)**

If there's a `Message` struct that includes receipt logic:

- **Before:** `func encode() -> Data { JSONEncoder().encode(self.receipt) }`
- **After:** `func encode() -> Data { encodeReceipt(self.receipt) }`

---

## Acceptance criteria

- [ ] All iOS receipt JSON handling replaced with unified `encodeReceipt()` / `decodeReceipt()` calls
- [ ] Grep finds 0 remaining `Codable` or `JSONEncoder`/`JSONDecoder` references to `Receipt` in iOS code (outside tests/comments)
- [ ] iOS app builds with Xcode (no linker errors, no Swift compilation errors)
- [ ] Integration test: send message from iOS, receive receipt from CLI, verify iOS displays `DELIVERED(receipt-verified)` status
- [ ] FD-4 drill (meeting room) includes iOS: 6+ mixed iOS/Android devices, all-pairs delivery, zero lost messages
- [ ] No behavior change (same receipt data, just unified encoding path)

---

## Notes

- **Mirrors U5:** Same fix shape as Android, different language (Swift instead of Kotlin)
- **Unblocks:** FD-4 drill and full farm parity (iOS/Android/CLI all use the same receipt format)
- **Farm gate:** FD-10 (delivery-truth audit) requires this: receipt round-trip must work identically across all platforms
- **Distribution:** After C5 (Apple Developer account decision), TestFlight release can ship with unified receipts

