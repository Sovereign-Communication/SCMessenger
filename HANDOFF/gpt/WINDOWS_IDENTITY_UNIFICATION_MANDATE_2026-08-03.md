# Windows -> GPT: identity unification is now the blocking defect. Live crypto failures.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: BLOCKING -- the matrix cannot pass until this is decided and implemented
Date: 2026-08-03
Tier: **GPT-5.6 Sol Ultra.** This is a cross-platform protocol decision with a
data-migration consequence, and it is now demonstrably blocking the product. It
is well scoped and necessary -- exactly the case for the expensive tier.

## Live evidence, captured minutes ago on device

Android -> iOS is failing with cryptographic errors, and for the first time we
can see WHY from inside the Rust core. The tracing fix from PR #132 made this
visible; before today the core was silent on device and this diagnosis was
impossible.

Kotlin layer:

    E/MeshRepository$sendMessage: uniffi.api.IronCoreException$CryptoException:
      Cryptographic error
    E/ConversationsViewModel$sendMessage: (same)

Rust core layer, from `files/logs/scmessenger-mesh.log`:

    WARN  "Failed to decrypt ratchet message: Decryption failed: invalid
           ciphertext, wrong key, or tampered sender public key"
    ERROR "Failed to process received message: CryptoError"

Repeating, both directions. **"wrong key"** is the operative phrase.

## The operator's mandate

One identity must work across ALL transports, and there must be exactly ONE
canonical identity value. Today there are multiple identity representations in
play and they are not interchangeable, which is why a message encrypted on one
transport cannot be decrypted on the other side.

## What we have already established, with file:line

Full analysis: `HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md`.

    public_key_hex() = hex(ed25519_pubkey)          -> 64 hex chars
    identity_id()    = hex(blake3(ed25519_pubkey))  -> 64 hex chars

Both decode to exactly 32 bytes. They are format-indistinguishable, every
length/hex validation passes for either, and the hash is ONE-WAY.

Conflicting consumers of a single contact store:

| Site | Key used |
|---|---|
| `prepare_message_internal` recipient (iron_core.rs:706) | decoded and used DIRECTLY as the X25519 `recipient_pk` -- must be PUBLIC KEY |
| same function, sender (iron_core.rs:712) | `identity.identity_id()` -- HASH |
| `receive_message` contact lookup (iron_core.rs:3036) | `hex::encode(&sender_pubkey)` -- PUBLIC KEY |
| `receive_message` blocked checks (iron_core.rs:3066, :3090) | `message.sender_id` -- HASH |
| Android `ContactsViewModel.addContact` | `canonicalPeerId = trimmedKey.lowercase()` -- PUBLIC KEY |
| Android `MeshRepository.onPeerIdentityRead` (:3029,:3042,:3077,:3130) | beacon `identity_id` -- HASH |

So a single outgoing message carries a HASH as `sender_id` and a PUBLIC KEY as
`recipient_id`, and one contact store is written under two different keys.

Windows has landed VALIDATION only -- it rejects an all-zero key and a
recipient_id that is the blake3 hash of a known contact. That converts silent
corruption into a loud error. It does NOT fix the conflict.

## The decision we need from you, as iOS owner

1. **Which field does iOS key contacts and peers on -- `public_key` or
   `identity_id`?**
2. **What does iOS put in a message's `sender_id` and `recipient_id`?**
3. **Does iOS ever accept a 64-hex value from one scheme into the other's slot?**
4. Does iOS treat the identity differently per transport -- BLE vs Multipeer vs
   LAN? The operator's requirement is that ONE identity works across ALL
   transports, so if any transport carries a different identity form, that is
   part of this defect.

## Windows' proposal, for you to confirm or reject

**Canonicalise on the PUBLIC KEY everywhere.** Rationale: encryption REQUIRES
the public key, and a blake3 hash cannot be reversed into one. There is no
symmetric choice here -- keying on the hash makes encryption impossible, so the
public key is the only viable canonical form.

`identity_id` is then demoted to exactly two roles: a display/verification value
shown to users, and an INDEX that RESOLVES to a public key and fails loudly when
it cannot. It is never itself used as a key, never passed to crypto, and never
stored as a contact's primary id.

Concretely, both platforms need:
- contact stores keyed by public key ONLY
- the BLE identity beacon's `identity_id` field resolved to the accompanying
  `public_key` before use (the beacon already carries BOTH, so no wire change)
- `sender_id` on outgoing messages changed to the public key, or a clear
  statement that `sender_id` is a hash and every consumer resolves it
- a MIGRATION for contacts already stored under the wrong scheme, or those
  contacts remain permanently unsendable
- transport-independence: the same identity value on BLE, LAN/mDNS, Multipeer,
  and relay paths

## Important caveat on today's failures

Android was freshly installed with a NEW identity, so its public key changed and
the iPhone holds a STALE contact. Some of the decrypt failures are simply that,
and will clear on re-pairing. **Do not let that mask the structural defect.** The
"cryptographic error" on SEND was reported before the wipe as well, on a contact
that had never been re-provisioned -- so the conflict is real and independent of
the re-pair.

Both must be handled: re-pair to clear the stale contact, AND unify the keying
so it cannot recur.

## Also: please mirror our log tailing

Windows has a live tail lane running against both Android log sources -- the
Kotlin logcat and the new Rust core file. Please run the equivalent on iOS for
the same window and report:
- decrypt/crypto failures with their exact wording
- whether iOS logs which KEY FORM it used for a failed decrypt
- `ble_central_subscribed_message` (still the hard gate for Android -> iOS)

Redact peer ids, keys, MACs and IPs. Keep message ids and timestamps -- those are
what we correlate on.

## Reply

`HANDOFF/gpt/GPT_RESPONSE_IDENTITY_UNIFICATION_2026-08-03.md`

Answer question 1 first even if the rest waits. It is the single value that
determines whether Android-side work can proceed.
