# P0 -- operator sends "fail crypto" to a (new) node identity while old ids read delivered

Status: OPEN (filed 2026-08-30)
Operator report: "I'm failing to send message to the new node ID (something about
crypto failing), but the old Id still says delivered, but I'm getting no response."

## Windows-side evidence (this checkout's node)

- Inbound from the Pixel works and DECRYPTS: `fa23e01d` "you there?" and
  `1a66306c` "working?" both arrived and logged `received_and_decrypted`.
- REAL wire crypto failure:
  `Drift envelope signature verification failed: IoError("Signature verification
  failed: signature error: Verification equation was not satisfied")`
  at 2026-08-30T04:28:15Z, immediately after the Pixel connected (04:28:11) and
  sent identity envelopes (04:28:12) that Windows learned as contact
  serial number `c047e72d...` (identity `b46dcf21...` / peer `12D3KooWNkx3...`);
  the peer then disconnected (04:28:18).
- Sender-side `delivered=true` is set on transport ACK / a prior receipt, NOT on
  recipient app crypto/verification, so a message can read delivered while the
  payload never verifies -- the exact "delivered but no response" disconnect.

## Working hypothesis (needs on-device confirmation)

The same user material has produced MULTIPLE key generations (identity_ids
`b6486de2`, `d01c3751`, `b46dcf21`: three cryptographically unrelated public
keys; two peer ids `12D3KooWJoW9r` and `12D3KooWNkx3`). After an identity
rotation/reinstall a device can end up SIGNING with one key while the envelope
still names the old identity/public-key flavor of the same peer; the recipient
verifies against a stale cached key and reports "equation not satisfied".
Everything (contacts, identity_sync, outbox flush) then treats delivery as
accepted while the payload is rejected -- no response ever returns.

## NOT addressed by the pending merge lane

PR #244 (D4 coalescing), #245 (never-drop retry), #246 (desktop version) touch
history keying, outbox retention, and release metadata -- none touch the Drift
signature path. This is a separate root-cause ticket.

## Required next evidence (operator / on-device)

1. The exact RECIPIENT identity the operator is sending to when it fails
   (a peer/public-key/contact name).
2. Pixel-side logcat around a failing send (adb) -- the encryption/sign
   path and which key it used.
3. Confirm the identity_id vs public_key the Pixel currently advertises
   vs the one Windows has cached in contacts.

## Fix direction (once root-caused)

- On Drift signature failure for a peer whose identity envelope was recently
  learned, re-resolve the peer's CURRENT public key (from the identity
  envelope) and re-verify before rejecting, rather than failing against a
  stale cached key.
- Ensure a device's signing key and its advertised public-key/identity_id can
  never diverge after reinstall/rotation (identity bookkeeping invariant).