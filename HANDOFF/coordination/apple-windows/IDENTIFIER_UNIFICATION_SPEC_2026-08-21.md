# SCMessenger Identifier Unification Specification & Parity Audit

Status: Canonical Proposal for Bilateral Lock-in (AW-BILAT-0001)
Date: 2026-08-21
Lanes: CAO (Apple / macOS / iOS) & CTO (Windows / Android / AWS Node)

---

## 1. Identifier Taxonomy & Mathematical Invariants

| Identifier Name | Representation | Generation / Derivation Formula | Role & Scope | Where Stored / Used |
|---|---|---|---|---|
| **`public_key_hex`** | 64 lower-hex chars | `hex::encode(ed25519_verifying_key_bytes)` | **Primary Canonical Identity** (Source of Truth) | `Contact.public_key`, `Contact.peer_id`, `MessageRecord.peer_id`, cryptographic ECDH/envelope signing |
| **`identity_id`** | 64 lower-hex chars | `hex::encode(blake3::hash(ed25519_verifying_key_bytes))` | **Cryptographic Identity Digest** (1-way derivative) | Identity sync payloads, anti-tamper envelope binding, ledger identity index |
| **`libp2p_peer_id`** | Base58 string (`12D3KooW...`) | `libp2p::PeerId::from_public_key(ed25519_pubkey)` | **Swarm Transport Routing** | Libp2p multiaddrs, Gossipsub routing, direct swarm dial, `nearby_ble_peers` |
| **`device_id`** | UUIDv4 string (`xxxxxxxx-xxxx-...`) | `Uuid::new_v4().to_string()` | **Hardware Instance Descriptor** | Per-device blocking, multi-device sync, device telemetry |

---

## 2. Discovered Parity Defects (Root Cause of "Lost / Split" Messages)

### Defect A: Asymmetric `MessageRecord.peer_id` Keying
- **Egress (Sending)**:
  - Android (`MeshRepository.kt:4795`) writes `initialRecord.peerId = publicKey` (`public_key_hex`).
  - iOS (`MeshRepository.swift:1689`) writes `messageRecord.peerId = recipientPublicKey` (`public_key_hex`).
- **Ingress (Receiving)**:
  - Core (`iron_core.rs:3325 & 3477`) writes `MessageRecord.peer_id = canonical_peer_id` (derived as `Blake3(sender_public_key)`, which is `identity_id`).
- **Impact**:
  - In `history.db`, sent messages are stored under `public_key_hex` (`6a05e70d...`), while received messages are stored under `identity_id` (`8493726b...`).
  - `HistoryStore::conversation(peer_id)` performs an exact-string match `record.peer_id == peer_id`.
  - When the UI opens a conversation with `public_key_hex`, **only sent messages are returned**. Inbound messages disappear from the UI even though they were decrypted and saved into storage!

### Defect B: Core Swarm Bridge Parameter Type Mismatch
- In `core/src/mobile_bridge.rs:3115`: `SwarmBridge.send_message_status(peer_id, data)` expects `libp2p::PeerId` (`12D3KooW...`).
- When UI passed `public_key_hex` (`64 hex`), `PeerId::from_str` failed with `invalid_peer_id`, aborting direct unicast.

---

## 3. Mandatory Canonical Invariants (To Be Enforced Across All Platforms)

1. **History Storage Invariant**:
   - `MessageRecord.peer_id` MUST ALWAYS store the 64-hex **`public_key_hex`** of the remote participant for both `SENT` and `RECEIVED` directions.
   - Alternatively, `HistoryStore::recent_internal` in `core/src/store/history.rs` must perform bidirectional alias resolution between `public_key_hex` and `identity_id = Blake3(public_key_hex)` so existing legacy records are never orphaned.

2. **Contact Store Invariant**:
   - `Contact.public_key` is ALWAYS the 64-hex lower-case Ed25519 public key.
   - `Contact.peer_id` is normalized to `public_key_hex`.

3. **Unified Resolution Contract**:
   - Calling `iron_core.resolve_identity(any_id)` MUST accept any valid form:
     - `public_key_hex` (64 hex) -> returns `public_key_hex`
     - `identity_id` (64 hex) -> looks up in contacts/ledger -> returns `public_key_hex`
     - `libp2p_peer_id` (`12D3KooW...`) -> extracts public key -> returns `public_key_hex`
   - Calling `iron_core.resolve_to_identity_id(any_id)` returns `Blake3(public_key_hex)`.

4. **Swarm Routing Contract**:
   - `SwarmBridge` dials and routes using `libp2p_peer_id` (`12D3KooW...`).
   - If `libp2p_peer_id` is not known or direct dial fails, it unconditionally falls back to `swarmBridge.sendToAllPeers(envelope_data)` for store-and-forward mesh broadcast.
