# Rule 8 Adversarial Security Review: Multi-Transport Store-and-Forward Cooperative Mesh

**Date:** 2026-09-14
**Review Type:** Rule 8 Adversarial Review (Pre-merge Perimeter Gate)
**Target Branch:** `feat/v040-multi-transport-store-forward` -> `main`
**Gated Files Touched:**
- `core/src/store/relay_custody.rs`
- `core/src/transport/behaviour.rs`
- `core/src/transport/swarm.rs`
**Non-gated Complementary Files:**
- `cli/src/main.rs`
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `HANDOFF/BOD_STATE.md`, `HANDOFF/CTO_STATE.md`, `HANDOFF/CEO_STATE.md`
**Governance Authority of Record:** Board of Directors Resolution `bod-9ee86618` (5/5 Unanimous APPROVE + Judge Concurrence; deepseek-v4-flash 1.00, gpt-5.6-luna 0.93, gpt-5-mini 0.95, ling-3.0-flash-fin 0.95, ling-3.0-flash 1.00; cost $0.003999).

---

## Verdict: APPROVE

The changes under `core/src/store/relay_custody.rs`, `core/src/transport/behaviour.rs`, and `core/src/transport/swarm.rs` are assessed as safe, doctrinally aligned, and cryptographically sound for merge into `main`.

---

## Technical Security Assessment

### 1. Cooperative Mesh Custody (`core/src/store/relay_custody.rs`, `core/src/transport/swarm.rs`)
- **Finding Addressed**: Previously, when an intermediary node received a store-and-forward packet for a recipient that had not established a direct registration on that intermediate node, `enforce_custody` failed with `CustodyError::NoRegistration`, causing relay custody rejection.
- **Doctrinal Alignment**: In accordance with the project doctrine ("Nodes, not relays; custody store-and-forward is a universal behavior all nodes perform"), nodes accept custody for transit without requiring prior local identity registration.
- **Cryptographic Bounds & Invariants**:
  - `recipient_identity_id` is strictly validated to be a 64-character lowercase/uppercase ASCII hexadecimal string representing a Blake3 identity hash (`identity_id.len() == 64 && identity_id.chars().all(|c| c.is_ascii_hexdigit())`). Malformed, truncated, or spoofed identifiers are rejected before custody acceptance.
  - `intended_device_id` is bounded to `1..=128` characters.
  - Envelope payload size is bounded to `1..=65536` bytes. Empty payloads and oversized buffers are rejected immediately.
  - `relay_message_id` is bounded to `1..=128` characters.
- **Defensive Unit Testing**: Four negative test cases were added in `core/src/store/relay_custody.rs`:
  - Format rejection on short identity ID.
  - Format rejection on non-hex characters in identity ID.
  - Format rejection on empty envelope payload (0 bytes).
  - Format rejection on oversized envelope payload (> 65536 bytes).

### 2. Transport Header Headroom & Dead Socket Cleanup (`core/src/transport/behaviour.rs`, `core/src/transport/swarm.rs`)
- **Connection Limits**: `max_established_per_peer` was increased from 2 to 4. During mobile network handovers (e.g. Wi-Fi dropping to Cellular WAN), transient sockets co-exist while the new interface negotiates. Setting limit to 4 accommodates simultaneous direct, relay, and transitioning sockets without triggering connection exhaustion panic.
- **Aggressive Dead Socket Cleanup**: In `SwarmEvent::Behaviour(IronCoreBehaviourEvent::Ping(event))`, ping failures immediately call `swarm.close_connection(event.connection)`. This prevents stale mobile carrier NAT allocations and closed cellular sockets from lingering and blocking subsequent connection attempts.

### 3. Originating Sender Resolution & Parity (`cli/src/main.rs`, `MeshRepository.kt`)
- **ACK Misdirection Prevention**: `resolve_sender_peer_id` validates the authenticated envelope's 64-hex public key, preventing intermediary relay nodes from receiving and dropping delivery ACKs intended for the originator.
- **Dumb Byte Pipe Parity**: Android Kotlin code strips manual relay route parsing in favor of Rust-core managed relay ladders, maintaining platform parity across Android, iOS, and CLI.

---

## Adversarial Threat Analysis

1. **Unauthenticated Flooding via Unregistered Custody**:
   - *Threat*: An attacker floods a node with packets for random fake identities to exhaust disk/memory custody capacity.
   - *Mitigation*: The node's global and per-peer rate limiting (`RELAY_PEER_BUCKET_MAX_TRACKED`, `RELAY_MAX_TRACKED_DUPLICATES`, `RELAY_DUPLICATE_WINDOW_MS`) and total custody disk quotas in `RelayCustodyStore` continue to govern custody intake. The strict 65KB payload bound prevents jumbo buffer attacks.
2. **Identity Spoofing in Originating Sender Resolution**:
   - *Threat*: A rogue peer claims to be another peer by forging metadata.
   - *Mitigation*: The envelope's inner payload is signed with the originator's Ed25519 identity key and bound with authenticated additional data (AAD). Intermediaries cannot alter the envelope without invalidating the end-to-end cryptographic verification.

---

## Disposition

[OK] Rule 8 Gate PASSED. Safe to commit, push to PR branch, and merge following CI verification.
