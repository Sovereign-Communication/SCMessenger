# Qwen Task: Identity Canonicalization on Public Key (CRITICAL BLOCKER)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date**: 2026-08-04
**Status**: EXECUTE IMMEDIATELY
**Priority**: CRITICAL - blocks 5-node run 2 entirely
**Owner**: Qwen free tier / Windows execution lane

---

## Problem (from IDENTITY_HASH_VS_PUBKEY_CONFLICT.md)

Two indistinguishable 64-hex values for same peer:
- `public_key_hex` = Ed25519 public key (REQUIRED for X25519 encryption)
- `identity_id` = blake3(public_key) (one-way hash, CANNOT encrypt)

Message envelope carries `identity_id` (hash), but `prepare_message_internal` uses `recipient_id` directly as public key for encryption. Result: "wrong key" failures.

---

## Files to Fix

### 1. `core/src/identity/keys.rs`
Add validation to distinguish the two forms, make them visually distinct in logs

### 2. `core/src/iron_core.rs`
- Line ~706: `prepare_message_internal` recipient handling - validate recipient_id is public key, not hash
- Line ~712: sender uses `identity.identity_id()` - change to use public key or add both
- Line ~3036: contact lookup - must use public key
- Line ~3066, 3090: blocked checks - must handle both forms

### 3. `android/app/src/main/java/com/scmessenger/android/MeshRepository.kt`
- `onPeerIdentityRead`: BLE beacon carries BOTH `public_key` and `identity_id` - MUST key contacts by `public_key`, not `identity_id`

### 4. `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift` (GPT lane)
- Same fix: key contacts by `public_key` from beacon

---

## Required Changes

### A. Canonicalize on Public Key Everywhere
```rust
// In iron_core.rs prepare_message_internal:
fn prepare_message_internal(&self, recipient_id: &str, ...) {
    // REJECT if recipient_id matches a known identity_id (hash)
    if self.contact_manager.is_known_identity_hash(recipient_id) {
        return Err(IdentityError::HashUsedAsPublicKey);
    }
    // Validate it's a valid Ed25519 public key
    let recipient_pk = validate_and_decode_public_key(recipient_id)?;
    // ... rest of encryption
}
```

### B. Add identity_id → public_key Index
```rust
// In ContactManager:
fn resolve_identity_hash(&self, hash: &str) -> Option<PublicKey> {
    // Look up hash in index, return associated public key
}
```

### C. Migration for Existing Contacts
```rust
// One-time migration:
fn migrate_contacts(&self) {
    for contact in self.all_contacts() {
        if contact.peer_id != contact.public_key {
            // peer_id is hash, fix it
            self.update_contact_peer_id(contact.public_key);
        }
    }
}
```

### D. Visual Distinction in Logs
- Prefix `identity_id` with `id:` in logs
- Prefix `public_key` with `pk:` in logs
- Or use different lengths/formats

---

## Acceptance Tests

1. **Unit**: `prepare_message_internal` rejects known identity_hash as recipient
2. **Unit**: Contact lookup works with both identity_id (resolves to pk) and public_key
3. **Integration**: Android → Windows CLI message decrypts (currently FAILS)
4. **Integration**: Windows CLI → Android message decrypts (currently WORKS)
5. **Both directions** must work for same node pair
6. **Migration**: Existing contacts with hash-as-peer-id become sendable

---

## Deliverable

- Branch: `fix/identity-canonicalization-public-key`
- PR against `main`
- All Windows gates pass (fmt, clippy, build, test, Android)
- FFI snapshot check (P6)
- Adversarial review for crypto/transport change

---

## Notes

- This is the SINGLE BLOCKER for run 2 - per FIVE_NODE_RUN_1_ANALYSIS.md: "nothing else here matters until it is settled"
- iOS MUST agree on same convention (GPT will handle iOS side)
- BLE beacon already carries both fields - wire format supports this
- Cross-platform coordination required: Windows (core + Android), Mac (iOS)