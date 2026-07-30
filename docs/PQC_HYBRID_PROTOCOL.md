# SCMessenger Post-Quantum Hybrid Protocol Specification

Status: Active
Last updated: 2026-07-29

This document specifies the post-quantum hybrid cryptographic protocol as implemented in the `core/src/crypto/` module of SCMessenger.

---

## 1. Cryptographic Suite Registry

| Suite Identifier | Label | Key Exchange / KEM | Identity & Envelope Signatures | Ratchet Construction |
|---|---|---|---|---|
| `0x01` | Classical (v1) | X25519 | Ed25519 (64B) | Signal Double Ratchet (HKDF / Blake3) |
| `0x02` | Hybrid PQ (v2) | Hybrid X25519 + ML-KEM-768 (1184B pubkey, 1088B ct) | Dual Ed25519 (64B) + ML-DSA-65 (3309B sig, 1952B pubkey) | Hybrid Double Ratchet with PQ Secret Injection |

---

## 2. Format Tag Values and Fixed Parameters

| Format / Object | Version Tag Byte | Key/Ciphertext Lengths | KDF / Hash Context String |
|---|---|---|---|
| Envelope V2 Wire Format | `0x02` | Outer signature: Ed25519 (64B) + ML-DSA-65 (3309B) | Domain prefix: `0x02 \|\| bincode(EnvelopeV2)` |
| ML-KEM-768 Public Key | N/A | 1184 bytes | Encapsulated CT: 1088 bytes, SS: 32 bytes |
| ML-KEM-768 Private Key | N/A | 2400 bytes (zeroized on drop) | Seed: 64 bytes |
| ML-DSA-65 Verifying Key | N/A | 1952 bytes | Signature: 3309 bytes |
| ML-DSA-65 Signing Key | N/A | 32 bytes (seed format, zeroized) | Signature: 3309 bytes |
| Hybrid KEM Combiner | N/A | Input IKM: 2368 bytes | `"iron-core hybrid-kem v1 X25519+MLKEM768 2026-07"` |
| Suite Transcript Binding | N/A | Material: concatenated suite arrays | `"iron-core suite-transcript v1"` |
| Classical Root KDF (v1) | N/A | Input: `root_key (32B) \|\| dh_secret (32B)` | Context: `"iron-core ratchet-dh v1 2026-07-06"` |
| Hybrid Root KDF (v2) | N/A | Input: `root_key \|\| dh_secret \|\| pq_ss` | Context: `"iron-core hybrid-ratchet-dh v2 2026-07-10"` |

---

## 3. Hybrid KEM Construction (`core/src/crypto/pq/hybrid.rs`)

The hybrid key encapsulation mechanism combines classical X25519 ECDH and ML-KEM-768 decapsulation:

### Encapsulation:
1. Generate ephemeral X25519 keypair (`ephemeral_secret`, `x25519_ephemeral_public`).
2. Compute `ss_x25519 = X25519(ephemeral_secret, their_x25519_public)`.
3. Perform ML-KEM-768 encapsulation: `(mlkem_ciphertext, ss_mlkem) = MlKem768.encapsulate(their_mlkem_encaps_key)`.
4. Construct input key material (`ikm`):
   ```text
   ikm = ss_x25519 (32B)
         || ss_mlkem (32B)
         || x25519_ephemeral_public (32B)
         || their_x25519_public (32B)
         || mlkem_ciphertext (1088B)
         || their_mlkem_encaps_key (1184B)
   ```
5. Derive shared ratchet key:
   ```text
   shared_key = blake3::derive_key("iron-core hybrid-kem v1 X25519+MLKEM768 2026-07", ikm)
   ```
6. Explicitly zeroize `ss_mlkem` and `ikm`.

### Decapsulation:
1. Perform X25519 DH using receiver's static/ephemeral secret: `ss_x25519 = X25519(our_x25519_secret, ct.x25519_ephemeral_public)`.
2. Perform ML-KEM-768 decapsulation: `ss_mlkem = MlKem768.decapsulate(our_mlkem_keypair, ct.mlkem_ciphertext)`.
3. Construct identical `ikm` buffer and derive `shared_key` using the same Blake3 context string.

---

## 4. Negotiation & Transcript Binding (`core/src/crypto/negotiation.rs`)

Session establishment negotiates highest mutually supported suite between initiator and responder:
- Supported suites array is serialized and hashed with separator bytes (`0xFF`).
- Transcript hash derived via:
  ```text
  transcript_hash = blake3::derive_key("iron-core suite-transcript v1", material)
  ```
- If suite `0x02` is negotiated, both parties store `transcript_hash` in `RatchetState` and `SessionState` for session context validation.

---

## 5. Hybrid Double Ratchet (`core/src/crypto/ratchet.rs`)

When suite `0x02` is active:
- **DH Steps**: Each Diffie-Hellman ratchet step encapsulates/decapsulates a fresh ML-KEM-768 key pair alongside the X25519 key exchange.
- **Root Key KDF**:
  ```text
  combined = blake3::derive_key("iron-core hybrid-ratchet-dh v2 2026-07-10", input)
  new_root = blake3::derive_key("iron-core hybrid-ratchet-dh v2 2026-07-10:root", combined)
  chain_key = blake3::derive_key("iron-core hybrid-ratchet-dh v2 2026-07-10:chain", combined)
  ```
- **Symmetric Chain**: Symmetric message keys continue to derive via `blake3::derive_key` chain stepping.

---

## 6. Envelope V2 Wire Layout & Dual Signatures (`core/src/crypto/encrypt.rs`)

Envelope V2 structures messages with optional post-quantum fields:
- `version`: `0x02`
- `suite`: `0x02` (or `0x01` in compatibility mode)
- `sender_id` / `recipient_id`: Blake3 identity hashes
- `ephemeral_pubkey`: X25519 public key (32B)
- `pq_ciphertext`: Optional `HybridCiphertext` bincode payload
- `signature`: Classical Ed25519 signature (64B)
- `pq_signature`: Optional ML-DSA-65 post-quantum signature (3309B)

Envelope authentication:
- Signed bytes = `0x02 || bincode::serialize(EnvelopeV2)`.
- Ed25519 signature verified against sender's classical public key.
- ML-DSA-65 signature verified against sender's ML-DSA-65 verifying key when suite `0x02` is active.

---

## 7. Operational & Security Boundary Notes

- **Relay Envelope Signatures**: Per-envelope relay transit signatures remain classical Ed25519 for performance efficiency across intermediary hop nodes.
- **Transport Security (Noise Protocol)**: Lower-level libp2p connection security uses standard `libp2p-noise` (X25519+ChaCha20Poly1305); end-to-end payload security is provided at the core payload layer via Envelope V2 hybrid encryption.
- **Require PQ Mode**: Configurable flag `require_pq` enforces suite `0x02` mandatory rejection of fallback suite `0x01` sessions.
