# P0 -- the V2 hybrid handshake provides NO sender authentication

Status: OPEN -- operator decision required
Severity: P0. Message forgery / sender impersonation on the DEFAULT suite.
Found: 2026-08-22, during adversarial review of PR #215's remediation branch
Affects: `core/src/crypto/ratchet.rs`, all platforms (CLI, Android, iOS, WASM)
Discovered by: adversarial review; both legs independently re-verified by the CTO

## Summary

Anyone who can obtain a recipient's published public key bundle can send that
recipient a message that **decrypts successfully and is attributed to an
arbitrary sender of the attacker's choosing**.

This is not a defect in any open PR. It is in shipping code on the default
negotiated suite. It was found because PR #215's remediation asserted
`is_authenticated = true` for V2 envelopes, and the reviewer checked whether
that assertion was true. It is not.

## Leg 1 -- the V2 root key does not bind the sender's identity

`core/src/crypto/ratchet.rs:508-534`:

```rust
pub fn init_as_receiver_hybrid(
    _our_signing_key: &ed25519_dalek::SigningKey,   // UNUSED
    our_x25519_secret: &x25519_dalek::StaticSecret,
    our_mlkem_keypair: &crate::crypto::pq::MlKem768KeyPair,
    _sender_bundle: &crate::identity::PublicKeyBundle,  // UNUSED
    hct: &crate::crypto::pq::hybrid::HybridCiphertext,
    transcript_hash: [u8; 32],
) -> Result<Self> {
    ...
    let ss_hybrid = hybrid_decapsulate(our_x25519_secret, our_mlkem_keypair, hct)?;
    let root_key_0 = blake3::derive_key(
        "iron-core session-root v2 2026-07",
        &[&ss_hybrid.as_bytes()[..], &transcript_hash[..]].concat(),
    );
```

Both `_our_signing_key` and `_sender_bundle` are underscore-prefixed and never
read. The root key is derived from `ss_hybrid || transcript_hash` alone.

- `ss_hybrid` comes from `hybrid_decapsulate`, which consumes only the
  RECIPIENT's secrets and the attacker-supplied ciphertext. `hybrid_encapsulate`
  (`core/src/crypto/pq/hybrid.rs:50-78`) requires only the recipient's PUBLIC
  X25519 key and ML-KEM encapsulation key. It is a sender-anonymous KEM.
- `transcript_hash` (`core/src/crypto/negotiation.rs:28-37`) is derived from both
  suite lists and both Ed25519 PUBLIC keys. Fully attacker-computable.

No term in the derivation requires the sender's private key.

## Leg 2 -- nothing verifies a signature at ingress

- `verify_envelope_v2` exists but has **only test callers**
  (`core/src/message/codec.rs:682`, `:693`). Verified by grep across `core/`
  and `cli/`.
- `core/src/drift/envelope.rs` writes an Ed25519 signature on send (`:554`) but
  contains **no verify function and no VerifyingKey reference at all**.
  `from_bytes` (`:298`) merely parses the 64 bytes into the struct.
- `decrypt_message_ratcheted_v2` uses `sender_public_key` as AAD only
  (`core/src/crypto/encrypt.rs:292`). AAD binds a value to a ciphertext; it does
  not prove possession of the corresponding private key.

## Why it is the default path, not an edge case

`sign_bundle` advertises `supported_suites = vec![0x01, 0x02]`
(`core/src/identity/keys.rs:401`) and `negotiate_suite` takes
`intersection.iter().max()` (`core/src/crypto/negotiation.rs:23`), so healthy
peers always negotiate `0x02` -- the hybrid suite.

Additionally, `DriftEnvelope::to_wire_envelope()` classifies as V2 if ANY of
`suite`, `pq_kem_ciphertext`, `pq_encaps_key`, `transcript_hash` is `Some` --
all four are attacker-controlled wire fields. An attacker can therefore force
the V2 classification at will.

## Attack

Mallory wants Bob to receive a message attributed to Alice. Both bundles are
published key material.

1. `hybrid_encapsulate(bob_x25519_pub, bob_mlkem_pub)` -> `(hct, ss)`.
2. `transcript_hash` from the two public bundles.
3. `root_key_0 = blake3::derive_key("iron-core session-root v2 2026-07", ss || transcript_hash)`
   -- identical to what Bob will compute.
4. Run the sender ratchet from `root_key_0`; set
   `sender_public_key = alice_ed25519_pub`; wrap in a Drift envelope with
   `suite`/`pq_kem_ciphertext` set.
5. Bob decrypts successfully and files the message in Alice's thread.

Precondition: Bob holds Alice's contact bundle and has no receive-side session
for her. The reviewer notes that second condition is the NORMAL state, because
the send path stores sessions under the recipient's pubkey hex
(`iron_core.rs:888`) while the receive path looks them up under
`hex(blake3(pubkey))` (`encrypt.rs:597`) -- so send and receive never share a
session. That mismatch is itself worth a separate ticket.

## NOT YET PROVEN

No forgery test has been executed. The above is derived by reading the
derivation and confirming the two unused parameters and the absent ingress
verification. **Before anyone spends effort on a fix, write
`test_v2_hybrid_envelope_forgeable_without_sender_key` and confirm the
forgery actually succeeds.** If it fails, something authenticates the sender
that this analysis missed, and that path should be documented instead.

## Impact if confirmed

For a product whose stated value is sovereign, authenticated, end-to-end
encrypted messaging, an attacker who knows two published bundles can put words
in a contact's mouth. Confidentiality against third parties is unaffected --
the KEM still protects the payload -- but sender authenticity is absent on the
default suite.

## Blast radius

`core/src/crypto/` is inside the merge-blocked perimeter. Any fix requires
explicit operator sign-off under `docs/rules/SECURITY_PROTOCOL.md`.

Candidate directions (not a recommendation -- needs design review):
- Bind the sender's static key into the handshake, e.g. an X3DH-style
  `DH(IK_sender, SPK_recipient)` term folded into the root-key KDF.
- Mandatorily verify the Drift/V2 envelope signature at ingress before any
  attribution, storage, or routing sighting.

## Consequence for PR #215

`cto/routing-peer-seen-v2-2026-08-22` sets `is_authenticated = true` for V2 and
gates a routing sighting on it. That converts an acknowledged gap into an
asserted guarantee, which is worse than the state it replaced. Until this ticket
is resolved, V2 must be treated as `false` there.
