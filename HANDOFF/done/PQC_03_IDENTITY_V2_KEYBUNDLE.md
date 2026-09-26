# TASK: PQC-03 — Identity v2: dedicated encryption keys + ML-KEM key bundle

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-01. Wave 0. Min tier: Sonnet.

## Why

Today the Ed25519 identity key IS the encryption key via `ed25519_to_x25519_secret` (`core/src/crypto/encrypt.rs`). Audit finding F3: one secret for signing + confidentiality, and no place to put an ML-KEM key. This task introduces dedicated encryption keys, cross-signed by the identity key. `public_key_hex` (Ed25519) REMAINS the canonical cross-platform identifier — nothing about contact IDs changes.

## Target structures

In `core/src/identity/keys.rs` (extend `IdentityKeys`, currently only `signing_key: SigningKey`):

```rust
pub struct IdentityKeys {
    pub signing_key: SigningKey,                       // unchanged, canonical identity
    pub x25519_encryption_secret: StaticSecret,        // NEW: freshly generated, NOT derived from Ed25519
    pub mlkem_keypair: crate::crypto::pq::MlKem768KeyPair, // NEW (PQC-01 wrapper)
}
```

Public bundle (new, in `core/src/identity/` — this is what peers learn about us):

```rust
pub struct PublicKeyBundle {
    pub ed25519_public: [u8; 32],
    pub x25519_public: [u8; 32],
    pub mlkem_encaps_key: Vec<u8>,       // 1184 B
    pub created_at: u64,
    pub signature: Vec<u8>,              // Ed25519 over domain-separated bytes below
}
```

Signature input: `b"iron-core keybundle v1" || ed25519_public || x25519_public || mlkem_encaps_key || created_at.to_le_bytes()`. Provide `sign_bundle(&IdentityKeys) -> PublicKeyBundle` and `verify_bundle(&PublicKeyBundle) -> Result<()>` (verify signature under the bundle's own ed25519_public; callers separately check that key equals the expected contact identity). Bundle serialization: bincode behind a format tag byte 0x01, same discipline as PQC-02.

## Steps

1. Extend `IdentityKeys` + `KeyPair` generation paths. New keys generated with `OsRng`. Zeroize on drop.
2. Storage migration in `core/src/identity/store.rs` (and inspect `core/src/store/` for identity persistence — run `rg -n "identity" core/src/store --type rust -l`): on load of a v1 identity (no encryption keys), generate the missing keys ONCE, persist as v2, and log an INFO line. v2 persisted format must be tagged (bincode discipline). Loading a v2 identity is idempotent. A v1 blob must still load forever.
3. Identity backup/restore (`core/src/crypto/backup.rs` encrypts a serialized identity payload): confirm the new fields ride inside the encrypted payload. Restoring an OLD backup must produce a working identity via the same migrate-on-load path (test this).
4. Contact model: find where a contact's public key material is stored (`rg -n "public_key_hex" core/src/store core/src --type rust -l`). Add optional `key_bundle: Option<PublicKeyBundle>` to the contact record WITHOUT breaking existing sled data — if contact records are bincode in sled, apply the tag-or-new-tree pattern (a new sled tree `contact_bundles` keyed by `public_key_hex` is the safest choice; state what you chose and why).
5. Seniority/continuity: bundle carries no independent trust — trust flows from the Ed25519 signature. Assert in a test that a bundle whose signature does not verify under its own ed25519_public is rejected.
6. Tests: keygen non-derivation (new x25519 public != `ed25519_public_to_x25519(ed25519_public)`), bundle sign/verify/tamper (each field), v1-identity migration, v1-backup restore + migration, contact bundle store/load roundtrip.

## Definition of Done

- [x] Standard gates PASS.
- [x] `cargo test -p scmessenger-core --test test_persistence_restart` PASSES (proves storage migration does not corrupt state).
- [x] Explicit statement in this file of every sled tree/key touched and the chosen encoding.
  - **Identity Keys Key**: `identity_keys` (in the `IdentityStore` database/sled backend). Encoded using a tag byte `0x02` followed by the bincode serialization of `IdentityKeysV2Raw`.
  - **Contact Bundles Prefix**: `contact_bundle:<public_key_hex>` (in the `StorageBackend` database/sled backend). Encoded as JSON (`serde_json::to_vec`) to maintain cross-platform storage compatibility.
- [x] File moved to HANDOFF/done/ + committed.

## Do NOT

- Remove or change `ed25519_to_x25519_secret` / `ed25519_public_to_x25519` (legacy decrypt of old messages depends on them, forever).
- Change `public_key_hex` semantics or any UniFFI surface.
- Wire the bundle into session establishment (PQC-06) or invites (PQC-10/11).
