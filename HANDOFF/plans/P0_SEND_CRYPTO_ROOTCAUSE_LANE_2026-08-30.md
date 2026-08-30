# P0 send-crypto root-cause lane (design)

Pair: `HANDOFF/todo/P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md` (evidence/hypothesis).
This file is the execution design for the next step of that ticket.

## Goal

Turn the operator report — *"send fails crypto to a newly-generated node id, old id
reads delivered, no response"* — into a deterministic reproduction and a root-caused
fix, without touching the merge lane, the v0.4.0 tag, or redeploying anything.

## STATIC-TRACE RESULT (2026-08-30, read-only, no build) — P0 root cause narrowed

This section supersedes the earlier working hypothesis in the paired ticket. Key
correct: the Drift receive-side verify uses a SELF-carried key (`sender_public_key`
inside the envelope), never a recipient-side cache. Combined with
`Verification equation was not satisfied` being ed25519_dalek's `SignatureError`
Display text, the failure proves the sign key != the `sender_public_key` bytes in
that same envelope.

Traced the exact wire string `Signature verification failed: ... Verification
equation was not satisfied` to its sources and the receive verify layer:

- `core/src/drift/envelope.rs:730-741` `DriftEnvelope::verify()`:
  `VerifyingKey::from_bytes(&self.sender_public_key)` then
  `verifying_key.verify(&hash, &signature)`.
- `core/src/crypto/encrypt.rs:889-923` `verify_envelope()` (v1) and
  `:945+` `verify_envelope_v2()` — both extract `sender_public_key` FROM THE
  ENVELOPE ITSELF and verify the signature against it. bincode canonical bytes.
- Confirmed: NEITHER verifies against a recipient-side contact/known-key cache.
  The next `sender_public_key` consumers (`iron_core.rs` receive path,
  ~3465-3531) use the envelope's key to derive identity_id/peer id AFTER
  signature verification passes — i.e. a sig failure is rejected before any
  cross-check.

**Conclusion: the failure is sender-side self-inconsistency, refuting the
stale-recipient-cache branch (H1a).** `equation not satisfied` means the signing
private key != the public key the sender embedded in that same envelope's
`sender_public_key`. This is either:
- **(H1b) post-rotation bookkeeping bug in the SENDER's sign/attach step** —
  device signs with K_new but attaches a stale K_old as `sender_public_key`;
  or
- a genuine spoof/foreign-signed envelope (system working as intended).

Repro now focuses on the SENDER: an on-device (Pixel logcat) trace of which
`SigningKey` signs vs which `verifying key` is attached to outbound envelopes.
This is the sharpened prerequisite the P0 ticket needs from the operator.

## Design below remains valid; updated reproduction target

The reproduction seam in section "Reproduction" must assert the narrowed case:
sign with a FRESH key while attaching a STALE public key to the envelope → the
current code returns the `equation not satisfied` branch (documents the sender-side
bookkeeping bug before the fix).

## Confirmed facts (Windows-side)
- Inbound Pixel->Windows DECRYPTS fine (`received_and_decrypted` observed).
- A real wire failure fired right after the Pixel connected directly:
  `Drift envelope signature verification failed: Verification equation was not satisfied`
  (2026-08-30T04:28:15Z), then peer disconnect.
- `delivered=true` is set on transport ACK / receipt, not on recipient app crypto, so
  the two signals diverge.

## Falsifiable hypotheses (H1 primary)

**H1 — post-rotation key divergence.** The same user material produced multiple key
generations (identity_ids `b6486de2`, `d01c3751`, `b46dcf21`; 2 peer ids). After a
reinstall/identity rotation a device can SIGN with key K_new while an envelope still
names the stale identity/public-key flavor of the same peer; the recipient verifies
against a stale cached pubkey and reports "equation not satisfied".

- H1a: recipient has cached the OLD pubkey in `contacts`; the fresh identity envelope
  (which Windows DID ingest at 04:28:12) is not applied because the peer is matched
  to the old contact row.
- H1b: the sender signs with K_new but does not attach/refresh the envelope's
  advertised pubkey, so the recipient has no K_new to verify against.
- **H2 (must-exclude):** a genuine cross-device key mismatch — the message genuinely
  originates from a device that never owned the named identity (spoof or a stray
  cached contact id).

## Reproduction (pure-local first; no Pixel round-trip)

1. **Static trace:** grep the one failing envelope path in `core/src/transport/drift.rs`
   (or wherever `Signature verification failed` is emitted) and map: verifyKey input,
   the pubkey the envelope names, and the contact pubkey the recipient resolved.
2. **Unit seam:** add a `#[cfg(test)]` harness to the drift payload verify that:
   a. accepts a valid sign by the SAME key it names,
   b. reproduces H1 exactly — a doc/known-good sign with K_new delivered in an envelope
      whose header still names pubkey(K_old) → assert the current code returns the
      "equation not satisfied" branch (documents the bug before the fix).
3. **Fix direction (only after repro):** on signature failure for a peer whose identity
   envelope was recently learned, re-resolve that peer's CURRENT pubkey from the
   identity envelope and re-verify before rejecting; enforce a device bookkeeping
   invariant that its signing key and advertised pubkey/identity_id can never diverge
   after rotation/reinstall.
4. Each step carries its own adversarial review via the repo's free-lane rule-8 gate.

## Required operator evidence (the unblocking inputs)

- The exact RECIPIENT identity the operator sends to when it fails.
- Pixel-side logcat around a failing send (adb): which key it SIGNED with and which
  pubkey the envelope names.
- The identity_id/public_key the Pixel currently advertises vs the one Windows has
  cached in contacts.

## Explicit non-goals

No redeploy, no Pixel round-trip until repro exists, no changes to #244/#245/#246/#247,
no tag movement. This lane is read-verified until the repro lands.

--- END DESIGN ---