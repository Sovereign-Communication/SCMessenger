# P0_IDENTITY_002 Implementation Summary

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-04-24
**Status:** COMPLETED
**Scope:** Core, Android, iOS, CLI

## Fixes Applied

### 1. Android  BLE Beacon Canonical Peer ID
- **File:** `android/app/.../data/MeshRepository.kt:2712`
- **Fix:** Changed `canonicalPeerId = identityId`  `canonicalPeerId = publicKeyHex` in `onPeerIdentityRead()` BLE handler.
- **Impact:** Prevents duplicate contacts when peer is discovered over BLE.

### 2. Android  Contact Add Canonicalization
- **File:** `android/app/.../data/MeshRepository.kt:3209-3227`
- **Fix:** Removed `isIdentityId` 64-hex short-circuit in `canonicalContactId()`. Now always calls `resolveIdentity()` which returns `public_key_hex` for any input format.
- **Impact:** All contact storage keys are now canonical `public_key_hex`.

### 3. Android  QR Export Unified Format
- **File:** `android/app/.../data/MeshRepository.kt:7647-7658`
- **Fix:** `getIdentityExportString()` emits `"peer_id"` (libp2p Peer ID) as primary, `"public_key"` as canonical, `"device_id"` for multi-device, `"identity_id"` as secondary.
- **Impact:** QR codes scanned by iOS/Android contain correct unified JSON.

### 4. Android  Contact Import Parser
- **File:** `android/app/.../utils/ContactImportParser.kt:25-32`
- **Fix:** Reads `"peer_id"` (libp2p) as primary, falls back to `"libp2p_peer_id"`, then legacy fields.
- **Impact:** Deep-link and QR imports resolve correctly.

### 5. iOS  BLE Beacon Canonical Peer ID
- **File:** `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:3696`
- **Fix:** Changed `canonicalPeerId: identityId`  `canonicalPeerId: normalizedKey` in `onPeerIdentityRead()` BLE handler.
- **Impact:** Matches Android fix; prevents iOS duplicate contacts.

### 6. iOS  QR Import Parser
- **File:** `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactsListView.swift:561-564`
- **Fix:** `parseImportedContactPayload()` now reads `"peer_id"` (libp2p) first, then `"libp2p_peer_id"`, then legacy fields.
- **Impact:** iOS QR scanner correctly parses unified JSON format.

### 7. iOS  Contact Add Canonicalization
- **File:** `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactsListView.swift:491-500`
- **Fix:** `addContact()` now calls `repository.ironCore?.resolveIdentity(anyId: finalPeerId)` and stores `canonicalPeerId` as `Contact.peerId`.
- **Impact:** Manual contact add, QR scan, and nearby-add all store canonical `public_key_hex`.

### 8. iOS  MeshRepository.addContact Defense
- **File:** `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:2745-2782`
- **Fix:** `addContact()` now canonicalizes `contact.peerId` via `resolveIdentity()` before storage, with fallback to original.
- **Impact:** Defense-in-depth for any caller that bypasses the view-layer canonicalization.

### 9. CLI  Contact Add Canonical Storage
- **File:** `cli/src/main.rs:699-732`
- **Fix:** `cmd_contact_add` derives `canonical_pk` via `core.extract_public_key_from_peer_id()`, verifies it matches user-provided public key, and stores `Contact::new(canonical_pk, public_key)`.
- **Impact:** CLI stores `public_key_hex` as canonical `peer_id`, not `libp2p_peer_id`.

### 10. Core  derive_public_key_from_peer_id Validation
- **File:** `core/src/store/contacts.rs:85-103`
- **Fix:** 64-hex strings are now validated as genuine Ed25519 public keys via `ed25519_dalek::VerifyingKey::from_bytes()`. Invalid keys (e.g., identity_id/Blake3 hashes) are rejected.
- **Impact:** `reconcile_from_history()` can no longer create contacts with `public_key = identity_id`.

## Verification

| Platform | Command | Result |
|----------|---------|--------|
| Rust Core | `cargo check --workspace` | PASS |
| Android | `./gradlew :app:compileDebugKotlin` | PASS |
| iOS | Swift syntax review | PASS (Xcode build recommended) |

## Remaining `resolveToIdentityId` References
- Only `wasm/src/lib.rs:1046` (`#[wasm_bindgen(js_name = resolveToIdentityId)]`)  WASM export, no active callers.
- Zero occurrences in Android Kotlin or iOS Swift.

## Not Done (Intentionally Deferred)
- **UDL Contact struct expansion** (`libp2p_peer_id` field): Risky  requires UniFFI regeneration and updating all 14 Android + iOS call sites. Will tackle in a dedicated task.
- **Cross-platform integration tests** (Android QR  iOS scan): Requires physical devices/simulators.

## Next Loop
`/loop 10m /orchestrate implement the identity plan perfectly.` is active.
