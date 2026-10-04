# Unified Infallible Identity Strategy v1.0

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-04-23
**Status:** PLAN  awaiting your approval before implementation
**Scope:** Android, iOS, CLI, Core (all variants)
**Confidence:** High  based on exhaustive codebase audit of 47 source files

---

## 1. The Four Canonical ID Formats

Every identity in SCMessenger is derived from a single Ed25519 keypair. From that root, four formats emerge. The strategy demands **each format is used only where semantically correct**.

| Format | Derivation | Length | Primary Use | Example |
|--------|-----------|--------|-------------|---------|
| **public_key_hex** | `hex(ed25519.pubkey)` | 64 hex chars | **Canonical identity.** Used for contact storage, message encryption, history, QR sharing. The ONE TRUE ID for persistence. | `c47ffd723275c6e1fd05f0071f92bb72a3ec857b3290aaf09a04a8a6611b36b1` |
| **identity_id** | `hex(blake3(ed25519.pubkey))` | 64 hex chars | **Human-readable fingerprint.** Displayed as "Identity Hash" or "Device ID". Used ONLY for visual identification, NEVER for routing or contact keys. | `acef56b777c766ea85e564740ca2172655f7d4e112317f9e6f54fb9850dfc148` |
| **libp2p_peer_id** | `base58(multihash(ed25519.pubkey))` | ~52 chars | **Network routing.** Used for libp2p dial, mDNS, Kademlia, relay, transport. The ONE TRUE ID for network operations. | `12D3KooWP3RGmGgRNtqGsfBCZgu8Wzao6qSsqYzLeLRmkqBdf5Ag` |
| **device_id** | `UUIDv4()` | 36 chars | **Installation-local instance.** Used for WS13 tight-pair routing, multi-device discrimination. | `a3f7b2e1-9c4d-4e8f-b123-456789abcdef` |

**The Golden Rule:**
- **Store contacts by `public_key_hex`** (canonical, stable, works across reinstalls)
- **Dial/transport by `libp2p_peer_id`** (network-routable)
- **Display `identity_id` as secondary** (human fingerprint)
- **Include `device_id` in routing** (multi-device support)

---

## 2. The Six Critical Inconsistencies (All Must Be Fixed)

### CRITICAL-1: QR Export Creates Contacts with `identity_id`, Transport Resolves to `libp2p_peer_id`  Duplicate Contacts/Threads

**Root cause:** `ContactImportParser.kt:25-31` reads `"identity_id"` from QR JSON and stores it as `contact.peerId`. But `resolveTransportIdentity()` in `MeshRepository.kt:6398-6405` was changed (March 2026 duplicate-thread fix) to return `canonicalPeerId = libp2pPeerId`. When the same peer is later discovered over LAN, their libp2p Peer ID doesn't match the stored identity_id. Result: **two contacts for one person, two message threads**.

**Evidence:**
```kotlin
// ContactImportParser.kt:25-31  WRONG: stores identity_id as peerId
val peerId = json.getString("identity_id")  // caccf865...
val publicKey = json.getString("public_key")  // c47ffd72...
// Creates Contact(peerId="caccf865...", publicKey="c47ffd72...")

// MeshRepository.kt:6398-6405  transport resolves to libp2p_peer_id
val canonicalPeerId = libp2pPeerId  // 12D3KooW...
// upsertFederatedContact checks existingById["12D3KooW..."]  stored["caccf865..."]
//  Creates SECOND contact
```

**Fix:**
1. Change QR export JSON format to use `"peer_id": "<libp2p_peer_id>"` (not `"identity_id"`)
2. Change `ContactImportParser` to read `"peer_id"` as the contact key
3. Add `resolve_identity()` call in `upsertFederatedContact` to canonicalize ANY incoming ID to `public_key_hex`

---

### CRITICAL-2: CLI `looks_like_blake3_id()` Rejects Valid `public_key_hex`

**Root cause:** `cli/src/main.rs:2813-2815` rejects any 64-char lowercase hex string as "looks like Blake3 identity ID". But `public_key_hex` is ALSO 64 lowercase hex chars. When users paste a public key into `scm contact add <peer_id> <public_key>`, the CLI rejects it.

**Evidence:**
```rust
fn looks_like_blake3_id(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit() && c.is_ascii_lowercase())
    // public_key_hex = 64 lowercase hex  ALSO MATCHES
}
```

**Fix:** Check for `12D3Koo...` prefix (libp2p Peer ID) vs identity_id/public_key_hex by trying to parse as libp2p::PeerId first. Only reject if it's 64 hex AND fails PeerId parse.

---

### HIGH-3: `derive_public_key_from_peer_id()` in Core is a Stub

**Root cause:** `core/src/store/contacts.rs:85-95` simply returns the input string unchanged:
```rust
pub fn derive_public_key_from_peer_id(peer_id: &str) -> Option<String> {
    Some(peer_id.to_string()) // STUB  does nothing
}
```

This is called by `reconcile_from_history()` to recover contacts from message history. If history stored `identity_id` or `libp2p_peer_id` in the `peer_id` field, the recovery creates a contact with `public_key` set to that invalid string. Future encryption to that contact fails.

**Fix:** Implement proper derivation:
```rust
pub fn derive_public_key_from_peer_id(peer_id: &str) -> Option<String> {
    // Try parse as libp2p PeerId  extract public key
    if let Ok(id) = libp2p::PeerId::from_str(peer_id) {
        if let Some(pk) = id.to_bytes().get(2..) {  // multihash prefix
            return Some(hex::encode(pk));
        }
    }
    // If 64 hex chars, assume it's already a public key
    if peer_id.len() == 64 && is_hex(peer_id) {
        return Some(peer_id.to_string());
    }
    None
}
```

---

### HIGH-4: `device_id` Missing from QR Exports

**Root cause:** Neither Android (`MeshRepository.kt:7598-7622`) nor iOS (`MeshRepository.swift:6035-6071`) includes `device_id` in the identity export JSON. WS13 tight-pair routing requires `intended_device_id`. QR-added contacts cannot participate in tight-pair until a subsequent transport handshake reveals the device ID.

**Fix:** Include `"device_id"` in `getIdentityExportString()` output on both platforms.

---

### MEDIUM-5: Android UI Labels `identity_id` as "Peer ID"

**Root cause:** `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt:185` displays `identity_id` with label "Peer ID". But the actual network Peer ID is `libp2p_peer_id`. Users copy the wrong ID for `scm contact add`.

**Fix:** Rename label to "Identity Hash" (matching iOS). Add a new field showing "Peer ID (Network)" with the libp2p Peer ID.

---

### MEDIUM-6: `Contact.peer_id` Semantics Are Ambiguous

**Root cause:** `core/src/api.udl:487` defines `Contact.peer_id` as `string`, but in practice it stores whatever the caller passed: QR import stores `identity_id`, manual add may store `libp2p_peer_id`, nearby discovery stores `libp2p_peer_id`. This ambiguity is why duplicate-contact bugs keep resurfacing.

**Fix:** Enforce that `Contact.peer_id` is ALWAYS the `public_key_hex` (canonical identity). Add `libp2p_peer_id` as a separate optional field on `Contact` for transport hints. Update all callers.

---

## 3. Unified ID Usage Matrix (After Fix)

| Operation | Must Use | Must NOT Use | Location |
|-----------|----------|--------------|----------|
| Contact storage key | `public_key_hex` | `identity_id`, `libp2p_peer_id` | `core/src/store/contacts.rs` |
| QR code primary ID | `libp2p_peer_id` | `identity_id` | `MeshRepository.kt/swift` |
| QR code pubkey | `public_key_hex` |  | `MeshRepository.kt/swift` |
| QR code device ID | `device_id` |  | `MeshRepository.kt/swift` |
| Contact add CLI arg | `libp2p_peer_id` | `identity_id` | `cli/src/main.rs` |
| Contact add pubkey arg | `public_key_hex` | `identity_id` | `cli/src/main.rs` |
| Message encryption | `public_key_hex` |  | `core/src/lib.rs` |
| Network dial | `libp2p_peer_id` | `identity_id`, `public_key_hex` | `core/src/transport/` |
| mDNS advertisement | `libp2p_peer_id` |  | `core/src/transport/discovery.rs` |
| Display "Peer ID" | `libp2p_peer_id` | `identity_id` | All UI screens |
| Display "Identity Hash" | `identity_id` |  | All UI screens |
| Display "Public Key" | `public_key_hex` |  | All UI screens |
| History lookup | `public_key_hex` |  | `core/src/store/history.rs` |
| Contact lookup (any input) | Resolve to `public_key_hex` | Store raw input | `core/src/lib.rs` |
| Transport identity resolution | `libp2p_peer_id` | `identity_id` | `core/src/mobile_bridge.rs` |

---

## 4. Exact Remediation Plan (File by File)

### Phase A: Core (Foundation)  Must Land First

#### A1. `core/src/api.udl`  Add `libp2p_peer_id` to `Contact`
```udl
dictionary Contact {
    string peer_id;           // ALWAYS public_key_hex (canonical)
    string public_key;        // Same as peer_id, kept for compatibility
    string? libp2p_peer_id;   // NEW: network Peer ID for transport hints
    string? nickname;
    string? last_seen;
    string? device_id;        // NEW: for WS13 tight-pair
    // ... rest unchanged
};
```

#### A2. `core/src/store/contacts.rs`  Fix `derive_public_key_from_peer_id`
Implement proper derivation as shown in HIGH-3 above. Also update `reconcile_from_history()` to call `resolve_identity()` before creating contacts.

#### A3. `core/src/lib.rs`  Enforce `resolve_identity()` at all ingress points
- `add_contact()`: call `resolve_identity(peer_id)` to canonicalize to `public_key_hex`
- `prepare_message()`: resolve recipient to `public_key_hex`
- `send_message()`: resolve recipient to `public_key_hex`, then derive `libp2p_peer_id` for transport

#### A4. `core/src/mobile_bridge.rs`  `SwarmBridge.send_message()`
Accept `recipient_identity_id` (any format), resolve to `public_key_hex` for encryption, derive `libp2p_peer_id` for dialing. Do NOT require caller to pass `libp2p_peer_id`.

### Phase B: Android

#### B1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `getIdentityExportString()`: Change `"identity_id"`  `"peer_id"` with value `libp2p_peer_id`. Add `"device_id"`. Keep `"public_key"`.
- `resolveTransportIdentity()`: Ensure `canonicalPeerId` is resolved through `core.resolve_identity()` to guarantee `public_key_hex`.

#### B2. `android/app/src/main/java/com/scmessenger/android/utils/ContactImportParser.kt`
- Read `"peer_id"` (not `"identity_id"`) from QR JSON
- Validate it starts with `12D3Koo...` (libp2p Peer ID format)
- Call `core.resolve_identity(peerId)` to get canonical `public_key_hex`
- Store contact with `peer_id = public_key_hex`, `libp2p_peer_id = qr_peer_id`

#### B3. `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`
- Rename "Peer ID" label  "Identity Hash" (shows `identity_id`)
- Add "Peer ID (Network)" field (shows `libp2p_peer_id`)
- Add copy button for `libp2p_peer_id`

### Phase C: iOS

#### C1. `ios/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- `getIdentityExportString()`: Change `"identity_id"`  `"peer_id"` with value `libp2p_peer_id`. Add `"device_id"`. Keep `"public_key"`.
- Same canonicalization logic as Android.

#### C2. iOS QR import parser (find equivalent to `ContactImportParser.kt`)
- Same fix: read `"peer_id"`, validate `12D3Koo...`, resolve to `public_key_hex`

#### C3. `ios/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`
- Already labels correctly ("Identity Hash" vs "Peer ID (Network)")  verify it shows `libp2p_peer_id`
- Ensure copy buttons copy the correct IDs

### Phase D: CLI

#### D1. `cli/src/main.rs`  Fix `looks_like_blake3_id()`
```rust
fn looks_like_blake3_id(s: &str) -> bool {
    if s.len() != 64 { return false; }
    if !s.chars().all(|c| c.is_ascii_hexdigit() && c.is_ascii_lowercase()) {
        return false;
    }
    // If it parses as a libp2p PeerId, it's NOT an identity_id
    if libp2p::PeerId::from_str(s).is_ok() {
        return false;
    }
    true
}
```

#### D2. `cli/src/main.rs`  `cmd_contact_add()`
- Accept `peer_id` (libp2p Peer ID) and `public_key` (hex)
- Call `core.resolve_identity(peer_id)` to verify it resolves to the same public key
- Store contact with `peer_id = public_key` (canonical)

#### D3. `cli/src/main.rs`  `cmd_send_offline()`
- Resolve recipient to `public_key_hex` via `core.resolve_identity()`
- Derive `libp2p_peer_id` from public key for transport

### Phase E: Cross-Platform Validation

#### E1. Unified QR Format Specification
```json
{
    "peer_id": "12D3KooWP3RGmGgRNtqGsfBCZgu8Wzao6qSsqYzLeLRmkqBdf5Ag",
    "public_key": "c47ffd723275c6e1fd05f0071f92bb72a3ec857b3290aaf09a04a8a6611b36b1",
    "device_id": "a3f7b2e1-9c4d-4e8f-b123-456789abcdef",
    "nickname": "Alice",
    "version": "1.0"
}
```

#### E2. Unified mDNS TXT Records
```
peer_id=12D3KooWP3RGmGgRNtqGsfBCZgu8Wzao6qSsqYzLeLRmkqBdf5Ag
pubkey=c47ffd723275c6e1fd05f0071f92bb72a3ec857b3290aaf09a04a8a6611b36b1
device_id=a3f7b2e1-9c4d-4e8f-b123-456789abcdef
version=1.0
transport=tcp
```

---

## 5. Verification Checklist (100% Must Pass)

### Unit Tests (Core)
- [ ] `resolve_identity("12D3KooW...")`  returns `public_key_hex`
- [ ] `resolve_identity("caccf865...")`  returns `public_key_hex`
- [ ] `resolve_identity("c47ffd72...")`  returns `public_key_hex` (idempotent)
- [ ] `derive_public_key_from_peer_id("12D3KooW...")`  returns `public_key_hex`
- [ ] `derive_public_key_from_peer_id("c47ffd72...")`  returns `c47ffd72...`
- [ ] `derive_public_key_from_peer_id("invalid")`  returns `None`
- [ ] Adding contact with `libp2p_peer_id` stores `public_key_hex` as key
- [ ] Adding contact with `identity_id` stores `public_key_hex` as key
- [ ] Adding contact with `public_key_hex` stores `public_key_hex` as key
- [ ] Duplicate add (different ID format, same person)  updates existing, no duplicate

### Integration Tests (Cross-Platform)
- [ ] Android QR  iOS scan  contact stores correct `public_key_hex`
- [ ] iOS QR  Android scan  contact stores correct `public_key_hex`
- [ ] CLI `contact add <peer_id> <pubkey>`  stores `public_key_hex`
- [ ] Android displays "Peer ID (Network)" = `libp2p_peer_id`
- [ ] iOS displays "Peer ID (Network)" = `libp2p_peer_id`
- [ ] CLI `identity` shows `libp2p_peer_id` and `identity_id` separately
- [ ] Message send AndroidiOS works after QR contact add
- [ ] Message send iOSAndroid works after QR contact add
- [ ] Message send CLIAndroid works after `contact add`
- [ ] Message send CLIiOS works after `contact add`
- [ ] No duplicate contacts after LAN discovery + QR add of same peer
- [ ] No duplicate message threads after transport discovery of QR-added contact

### Regression Tests
- [ ] Existing contacts (stored with old format) still load correctly
- [ ] Existing message history still accessible
- [ ] Identity backup/restore works with new format
- [ ] mDNS discovery still finds peers on LAN

---

## 6. Migration & Rollback

### Contact Store Migration
On first app start after update:
1. Iterate all contacts
2. If `contact.peer_id` is `identity_id` (64 hex, not `12D3Koo...`), call `core.resolve_identity(peer_id)` to get `public_key_hex`
3. Update `contact.peer_id` to `public_key_hex`
4. Set `contact.libp2p_peer_id` by deriving from `public_key_hex`
5. Save migrated contact

### Rollback
If issues detected:
1. Revert code changes
2. Run migration in reverse (if needed)
3. Clear app data as nuclear option

---

## 7. Confidence Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Core UDL change breaks Android/iOS bindings | Low | Critical | Add `libp2p_peer_id` as optional; old bindings ignore it |
| Contact migration corrupts existing data | Low | Critical | Backup before migration; idempotent migration |
| QR format change breaks old app versions | Medium | Medium | Old apps can't scan new QR  acceptable (require update) |
| `derive_public_key_from_peer_id` implementation wrong | Low | High | Unit test with known keypairs; verify round-trip |
| iOS Swift optional unwrapping on new UDL fields | Medium | Medium | Mark new fields as optional in UDL; Swift handles nil |

**Overall confidence: HIGH**  all changes are additive or renaming; no data format changes at the storage layer. The root cause is simply enforcing canonical IDs at ingress points.

---

*Compiled from exhaustive codebase audit + planning agent review*
*Awaiting your approval before implementation begins*
