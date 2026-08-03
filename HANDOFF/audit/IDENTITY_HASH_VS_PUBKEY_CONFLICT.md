# Identity hash vs public key: two incompatible keying schemes

Status: CONFIRMED, root cause of send failures and unreliable delivery
Found: 2026-08-03 (operator hypothesis, verified against source)

## The two values

```rust
// core/src/identity/keys.rs:86
pub fn public_key_hex(&self) -> String {
    hex::encode(self.signing_key.verifying_key().to_bytes())   // 64 hex chars
}

// core/src/identity/keys.rs:91
pub fn identity_id(&self) -> String {
    let public_key = self.signing_key.verifying_key().to_bytes();
    let hash = blake3::hash(&public_key);
    hex::encode(hash.as_bytes())                               // 64 hex chars
}
```

**Both are 64 hex characters. Both decode to exactly 32 bytes.** They are
format-indistinguishable and completely different values. The hash is one-way:
given an identity_id you CANNOT recover the public key.

This is why nothing catches the confusion. Every length and hex check passes for
either, e.g. `ContactsViewModel.addContact`:

    if (trimmedKey.length != 64) { ... }        // passes for both
    // hex charset check                         // passes for both

## Where each is used -- the conflict

| Site | Key actually used |
|---|---|
| `prepare_message_internal` recipient (iron_core.rs:706) | `hex::decode(recipient_id)` -> used DIRECTLY as `recipient_pk: [u8;32]` -> must be the PUBLIC KEY |
| `prepare_message_internal` sender (iron_core.rs:712) | `identity.identity_id()` -> HASH |
| `receive_message` contact lookup (iron_core.rs:3036) | `get_contact_bundle(&hex::encode(&sender_pubkey))` -> PUBLIC KEY |
| `receive_message` blocked checks (iron_core.rs:3066, 3090) | `message.sender_id` -> HASH |
| `ContactsViewModel.addContact` | `canonicalPeerId = trimmedKey.lowercase()` -> PUBLIC KEY |
| `MeshRepository.onPeerIdentityRead` (:3029, :3042, :3077, :3130) | `identityId` from the BLE beacon's `identity_id` field -> HASH |

So a single outgoing message carries a HASH as `sender_id` and a PUBLIC KEY as
`recipient_id`. And one contact store is written by two different keys.

## Consequences, in order of severity

**1. Encrypting to a hash (CRITICAL).**
If a contact's `peerId` holds an identity hash, `prepare_message_internal` does
`hex::decode(recipient_id)` -> 32 valid bytes -> uses them as the X25519
`recipient_pk`. It encrypts to a key that nobody holds. Nothing rejects this,
because a hash is a perfectly well-formed 32-byte value.

This matches the operator's report exactly: one contact fails with "failed to
send - cryptographic error" while another sends fine. The difference is which
scheme created each contact.

**2. Contact lookups miss.**
`contactManager.get(identityId)` cannot find a contact stored under the public
key. The peer looks unknown, so a duplicate entry gets created under the hash --
after which sending to it hits case 1.

**3. Blocked-list checks silently fail.**
`is_blocked(&message.sender_id)` passes a HASH to a store keyed by public key,
so a blocked peer is not recognised as blocked. Note this is separate from the
fail-open bug already fixed: even failing CLOSED correctly, the lookup asks the
wrong question.

## Why the public key must be canonical

The public key is REQUIRED for encryption. The identity hash is one-way and
cannot be converted back. So:

- `recipient_id` on the send path MUST be a public key
- any store that feeds the send path MUST be keyed by public key
- `identity_id` can only be a display aid, a verification value, or a lookup
  INDEX that resolves to a public key

## Proposed fix

1. **Canonicalise contacts on the public key.** One key scheme, stated in one
   place. `onPeerIdentityRead` must resolve the beacon's `identity_id` to the
   accompanying `public_key` (the beacon carries BOTH) and key everything by the
   public key.
2. **Add a hash -> public key index** for the case where only an identity_id is
   known (e.g. a routing hint). It resolves to a public key or fails loudly; it
   never substitutes.
3. **Validate on the send path.** `prepare_message_internal` must not accept any
   32-byte value as a public key. At minimum, assert the recipient resolves to a
   known contact public key, and reject a value that matches a known
   identity_id -- that is a caller bug and should be a hard error, not a silent
   encryption to nowhere. This is the "validation checks presence, not validity"
   pattern this codebase keeps reproducing.
4. **Make the two visually distinct in logs and payloads.** Same-length hex for
   two different things is the trap. Prefix or tag them wherever they are
   carried or logged.
5. **Migration.** Existing contacts may be keyed either way. A one-time pass
   should detect entries whose `peerId` does not equal their `publicKey` and
   repair them, rather than leaving users with permanently unsendable contacts.

## Cross-platform note

iOS must agree on the same convention or the two platforms will disagree about
peer identity even after Android is fixed. The BLE identity beacon carries both
`public_key` and `identity_id`, so the wire format already supports doing this
correctly -- the bug is purely in which field each consumer keys on.
