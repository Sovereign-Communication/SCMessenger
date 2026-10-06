# CRYPTO-T13 -- The V1 recovery path has the same rebuild-overwrite defect

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN -- not started
Severity: HIGH (same class as the finding fixed on the V2 path)
Gate: touches `core/src/crypto/` -- Rule-8 adversarial review required before merge.
Raised: 2026-10-04

## What was fixed, and what was not

Commit `71dfac8c1` closed a HIGH on the **V2** path: the session rebuild wrote
the map entry before proving the rebuilt session could decrypt, so a replayed
envelope destroyed a healthy session through a route that never reached the
"no bootstrap material" arm. The fix split
`build_receiver_session_hybrid` from `RatchetSessionManager::insert_session`, so
the V2 path builds a candidate, attempts the decrypt, and commits only on
success.

**The V1 path was deliberately left untouched** to keep that fix confined to the
decision point it was aimed at. Reading it now shows the same defect, unchanged.

## The V1 twin

`RatchetSessionManager::create_receiver_session`
(`core/src/crypto/session_manager.rs:86-100`) writes the map as its
unconditional final act, exactly as the V2 builder did:

```rust
let session =
    RatchetSession::init_as_receiver(our_signing_key, sender_identity_public_x25519)?;
match self.sessions.entry(peer_id.to_string()) {
    Occupied(mut e) => { e.insert(session); Ok(e.into_mut()) }
    Vacant(e) => Ok(e.insert(session)),
}
```

and the V1 divergence-recovery arm in `decrypt_with_ratchet_fallback`
(`core/src/crypto/encrypt.rs`, the `WireEnvelope::V1` ratchet branch) calls it,
retries the decrypt, and on failure removes the session outright:

```rust
manager.create_receiver_session(&peer_id, recipient_signing_key, &sender_x25519)?;
let session = manager.get_session_mut(&peer_id)...;
match decrypt_message_ratcheted(session, envelope) {
    Ok(plaintext) => Ok(plaintext),
    Err(retry_err) => Err(first_error),
}
...
if recovered.is_err() {
    manager.remove_session(&peer_id);
```

So on V1: the pre-existing session is overwritten, and if the replacement cannot
decrypt, the session is deleted outright. Same destruction, reached by a
replayed or malformed envelope on the legacy ratchet path.

## Why it was not fixed in the same commit

The V2 change was scoped to the V2 decision point on instruction, and the V1
path has a different recovery rationale (it rebuilds from static keys rather than
bootstrap material, per #394). Changing it deserves its own review rather than
riding along on a fix aimed elsewhere.

## Proposed fix

The pattern already exists and is proven by the V2 change:

1. Add `build_receiver_session(...) -> Result<RatchetSession>` that derives
   without touching the map; keep `create_receiver_session` as build-then-insert
   for callers with no session to protect.
2. In the V1 recovery arm, build a candidate, attempt
   `decrypt_message_ratcheted` against it, and `insert_session` only on success.
3. On failure, leave the pre-existing session installed, matching the V2 arms.

## Acceptance criteria

- `test_v1_rebuild_that_cannot_decrypt_keeps_existing_session`, mirroring
  `test_rebuild_that_cannot_decrypt_keeps_existing_session` for the V1 arm:
  assert the session survives, that exactly one session exists, and that the
  next genuine message still decrypts.
- The assertion must be provable, not tuned. For the V2 sibling the next
  message is number 1 against a chain at index 1, so no skipped-key window is
  involved; reproduce that shape rather than advancing the chain.
- Existing V1 ratchet tests pass unmodified.
- Confirm the first-contact V1 path, which has `!has_session` guarding it, still
  commits immediately.

## Related

- Fixed on V2: `71dfac8c1`, review trail in
  `HANDOFF/review/RULE8_HAIKU_OPUS_VERDICT_a34c60017_2026-10-04.md`.
- Same file also has unguarded `copy_from_slice` calls into fixed-size buffers on
  the inbound path (see `CRYPTO_T12_MALFORMED_PQ_FIELDS_PANIC.md`); the V1
  recovery arm's `sender_ed.copy_from_slice(&envelope.sender_public_key)` is one
  of them.