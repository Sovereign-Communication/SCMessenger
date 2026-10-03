# P0_IDENTITY_002d: Remaining Identity Plan Items

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** TODO
**Scope:** Core, Android, iOS, CLI

## Context
The P0_IDENTITY_002 unified identity plan is mostly implemented. Critical fixes landed:
- Android/iOS BLE beacon canonicalPeerId = public_key_hex
- CLI contact add stores canonical public_key_hex
- Android QR export/import uses peer_id (libp2p) as primary
- Core derive_public_key_from_peer_id implemented

## Remaining Items

### 1. iOS QR / Contact Import Verification
Verify iOS contact import path (if any) uses peer_id from QR JSON and resolves to public_key_hex via core.resolveIdentity(). Check:
- `iOS/SCMessenger/SCMessenger/Views/Contacts/` for add-contact QR scanner
- Any deep-link or share-sheet import flow
- Ensure Contact.peerId = public_key_hex (canonical)

### 2. UDL Contact Struct Expansion (OPTIONAL / RISKY)
Add `string? libp2p_peer_id;` to `core/src/api.udl` dictionary Contact.
This requires:
- a. Update UDL
- b. Update `core/src/store/contacts.rs` struct Contact
- c. Update `core/src/contacts_bridge.rs` struct Contact  
- d. Regenerate UniFFI bindings (`cargo uniffi generate` or equivalent)
- e. Update all 14 Android `uniffi.api.Contact(` call sites
- f. Update all iOS Contact construction call sites
- g. Verify `cargo check` and `./gradlew compileDebugKotlin`

**WARNING:** This was attempted and reverted in a prior session due to compilation errors. Only attempt if confident.

### 3. Cross-Platform Validation Tests
Create/run tests verifying:
- Android QR export JSON has peer_id=libp2pPeerId, public_key, device_id
- iOS QR export JSON matches Android format
- CLI `contact add <libp2p_peer_id> <public_key>` stores public_key as peer_id
- No duplicate contacts after LAN discovery + QR add of same peer

### 4. Update IN_PROGRESS Plan File
Move `../HANDOFF/IN_PROGRESS/P0_IDENTITY_002_Unified_Infallible_ID_Strategy_AWAITING_APPROVAL.md` to `../HANDOFF/done/` after verifying above items.

## Acceptance Criteria
- [ ] `cargo check --workspace` passes
- [ ] `./gradlew :app:compileDebugKotlin` passes (Android)
- [ ] iOS builds without Swift compilation errors
- [ ] No `resolveToIdentityId` or `resolve_identity` misuse in any platform
- [ ] All Contact.peerId assignments use public_key_hex (canonical)
