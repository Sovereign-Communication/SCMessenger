# ID Unification Implementation Plan

**Status:** SUPERSEDED / HISTORICAL — do not implement from this document
**Date filed:** 2026-03-10
**Superseded:** 2026-09-20 (canonical outlier audit CO-B-002; CTO disposition)
**Tracking:** was WS13+ Core Work

## Canonical authority (use this, not the body below)

- Canonical contact/identity key for storage and UI routing is the
  **Ed25519 public key hex (64-char)**, not `libp2p_peer_id`.
  See `core/src/api.udl` (IdentityInfo / canonical identity), and
  `core/src/message/types.rs` UNIFICATION notes.
- `libp2p_peer_id` is a **derived transport routing id**, not the primary
  contact key.
- Any sentence below that says contacts MUST use `libp2p_peer_id` as THE
  canonical identifier is **wrong** and must not be implemented.

This file is retained as history only. Indexed nowhere as Active.

## Executive Summary (historical — 2026-03-10)

SCMessenger currently uses multiple identifier types across different layers:
- **Public Key** (canonical identity - Ed25519 64-char hex)
- **Identity ID** (Blake3 hash of public key - derived display ID)
- **PeerId** (libp2p transport routing ID - derived from Ed25519)
- **Device ID** (planned - not yet implemented)

This document standardizes ID usage, eliminates case sensitivity issues, and adds Device ID support.

---

## Current ID Types and Usage

### 1. Public Key (Canonical Identity)
- **Format:** 64-character hex string (Ed25519 public key)
- **Purpose:** Primary identity - persistent across all platforms
- **Usage:** Contacts, encryption, signatures
- **Case:** Lowercase hex (enforced)

### 2. Identity ID (Display/Derived)
- **Format:** 64-character hex string (Blake3 hash of public key)
- **Purpose:** Display identifier, legacy compatibility
- **Usage:** UI display (deprecated in favor of nickname)
- **Case:** Lowercase hex (enforced)
- **Note:** NOT canonical - for display only

### 3. libp2p PeerId
- **Format:** Base58-encoded multihash (starts with "12D3Koo...")
- **Purpose:** Transport-layer routing
- **Usage:** Network discovery, connection management
- **Case:** Case-sensitive (Base58)
- **Note:** NOT canonical - derived from Ed25519 keypair

### 4. Device ID (Planned)
- **Format:** UUID v4
- **Purpose:** Device-specific identifier for multi-device support
- **Usage:** Device pairing, blocking granularity
- **Case:** Lowercase (enforced)
- **Note:** TODO - not yet implemented

---

## Unification Rules

### Primary Identifier: libp2p PeerId (for Contact Storage and UI)
**Contact storage and UI routing MUST use `libp2p_peer_id` as the canonical contact identifier.**

1. **Contacts:** Store and query by `peer_id` (libp2p PeerId)
2. **Messages:** Route by `peer_id`, resolve transport via PeerId
3. **Blocking:** Block by `peer_id` (identity-level)
4. **Encryption:** Use `public_key` for cryptographic operations

### Transport Resolution
**libp2p PeerId IS the canonical identifier for contacts and message routing.**

1. Store `public_key` as a separate field for cryptographic verification
2. Use `peer_id` (libp2p PeerId) as the primary contact identifier
3. Use PeerId for all libp2p dial operations and message routing

### Why libp2p PeerId as Primary?
- **Stable:** Derived from the Ed25519 keypair,不变 across sessions
- **Transport-native:** Used directly by libp2p for routing
- **User-visible:** Shorter format (base58) than full public key (64-char hex)
- **Prevents duplicates:** Single canonical ID prevents multiple threads for same peer

### Historical Note
Prior to this fix (2026-03-18), contacts could be stored with either:
- `peer_id = public_key_hex` (64-char hex)
- `peer_id = libp2p_peer_id` (base58 "12D3KooW...")

This caused duplicate message threads when the same peer was identified using different ID formats. The fix ensures `libp2p_peer_id` is always used as the canonical contact identifier.

### Display Names
**Use nickname for all UI display, fallback to truncated public_key.**

1. Never display raw PeerId to users
2. Never display Identity ID in UI (deprecated)
3. Format: `nickname` or `pub_key[0:8]...pub_key[-8:]` if no nickname

---

## Case Sensitivity Standardization

### Storage
- **Public Keys:** Always lowercase hex in database
- **PeerIds:** Case-preserved (Base58 is case-sensitive)
- **Device IDs:** Always lowercase UUID

### Comparison
```kotlin
// Public key comparison - case-insensitive
fun publicKeysEqual(a: String, b: String): Boolean {
    return a.lowercase() == b.lowercase()
}

// PeerId comparison - case-sensitive
fun peerIdsEqual(a: String, b: String): Boolean {
    return a == b
}
```

### Normalization
```kotlin
// Apply on storage
fun normalizePublicKey(key: String): String {
    return key.lowercase().trim()
}

fun normalizeDeviceId(id: String): String {
    return id.lowercase().trim()
}

// PeerId - no normalization
fun normalizePeerId(id: String): String {
    return id.trim()
}
```

---

## Device ID Implementation

### Purpose
- Support multi-device per identity
- Enable device-level blocking
- Track device-specific settings

### Schema
```rust
pub struct DeviceInfo {
    pub device_id: String,        // UUID v4
    pub public_key: String,        // Identity public key
    pub device_name: Option<String>,
    pub added_at: u64,
    pub last_active: Option<u64>,
}
```

### TODO Items
1. [ ] Add `device_id` generation on first launch
2. [ ] Store `device_id` in identity store
3. [ ] Include `device_id` in identity beacons
4. [ ] Add device-level blocking table
5. [ ] Sync device list across paired devices

**Estimated:** ~200 LoC (core) + ~150 LoC (mobile) + ~100 LoC (wasm)

---

## Implementation Checklist

### Core (Rust)
- [x] Public key as canonical ID (already implemented)
- [x] Case-insensitive public key comparison (already implemented)
- [ ] Add `device_id` to IdentityInfo
- [ ] Add `device_id` to identity beacon
- [ ] Device ID persistence
- [ ] Device blocking API

### Android
- [ ] Normalize all public_key storage to lowercase
- [ ] Replace identity_id usage with public_key
- [ ] Add device_id field to data models
- [ ] Update contact queries to use public_key
- [ ] Update message routing to resolve PeerId -> public_key
- [ ] Add nickname display fallback logic
- [ ] Implement device blocking UI

### iOS
- [ ] Normalize all publicKey storage to lowercase
- [ ] Replace identityID usage with publicKey
- [ ] Add deviceID field to data models
- [ ] Update contact queries to use publicKey
- [ ] Update message routing to resolve PeerId -> publicKey
- [ ] Add nickname display fallback logic
- [ ] Implement device blocking UI

### WASM
- [ ] Normalize all public_key storage to lowercase
- [ ] Replace identity_id usage with public_key
- [ ] Add device_id field to data models
- [ ] Update contact queries to use public_key
- [ ] Add nickname display fallback logic

---

## Migration Strategy

### Database Migration
1. Add `device_id` column to identity table (nullable initially)
2. Normalize existing `public_key` values to lowercase
3. Remove unused `identity_id` indexes
4. Create `device_id` unique index

### Runtime Migration
1. On app start, check if `device_id` exists
2. If not, generate UUID v4 and store
3. Normalize all in-memory public keys to lowercase
4. Update PeerId -> public_key resolution map

---

## Validation Tests

### ID Resolution
- [ ] Public key lookup is case-insensitive
- [ ] PeerId correctly resolves to public_key
- [ ] Device ID generates on first launch
- [ ] Device ID persists across restarts

### Display
- [ ] Contacts show nickname (not ID)
- [ ] Conversations show nickname (not ID)
- [ ] Fallback to truncated public_key works

### Blocking
- [ ] Identity-level blocking works
- [ ] Device-level blocking works (after implementation)
- [ ] Blocked users still relay messages

---

## Related Documents
- `docs/IDENTITY_BLOCKING_IMPLEMENTATION.md`
- `ANDROID_ID_MISMATCH_RCA.md`
- `CASE_SENSITIVITY_AUDIT_2026-03-09.md`
- `ID_UNIFICATION_PLAN.md`

---

## Lines of Code Estimates

| Component | LoC Estimate |
|-----------|--------------|
| Core (device_id + normalization) | ~200 |
| Android (migration + UI) | ~300 |
| iOS (migration + UI) | ~300 |
| WASM (migration + UI) | ~150 |
| Tests | ~200 |
| **Total** | **~1,150 LoC** |

---

**Next Steps:**
1. Implement device_id generation in core
2. Add normalization enforcement in storage layers
3. Update Android/iOS contact display to use nicknames
4. Add device blocking UI
5. Run full integration tests
