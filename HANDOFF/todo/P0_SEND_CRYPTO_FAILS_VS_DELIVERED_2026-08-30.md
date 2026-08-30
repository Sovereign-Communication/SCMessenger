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

## CONFIRMED ROOT CAUSE (2026-08-30) -- BLE envelope truncation, not key divergence

Live Windows-node capture (route `ble_gatt_ingress`, Pixel 6a as peripheral):

```
08:46:06  Received PeerJoined: 12D3KooWNkx3... (Pixel, via 192.168.0.129)
08:46:09  WARN iron_core: Failed to decode drift envelope: BufferTooShort { need: 1358, got: 1173 }
08:46:11  Peer left: 12D3KooWNkx3...
08:46:38  WARN iron_core: Drift envelope signature verification failed:
          IoError("Signature verification failed: ... Verification equation was not satisfied")
          -- route=ble_gatt_ingress
```

Mechanism: the CLI's central GATT ingress decoded each BLE notify notification as a
complete Drift envelope with NO fragment reassembly. The Android sender fragments
messages >~508B into multiple notifications (4-byte `GattFragmentHeader` + chunk);
Windows received the chunks and either failed `BufferTooShort` or verified a corrupt
signature on the partial tail -- the reported "crypto fails". No key divergence on
this path. Reproduces `P2_WIRE_ENVELOPE_TRUNCATION_2026-08-10`.

## FIX LANDED (2026-08-30) -- ble_gatt_ingress reassembly

`cli/src/ble_mesh.rs` (`subscribe_ingress_for_peripheral`) now buffers
multi-notification fragments and reassembles via the core `GattReassembler` before
decode, mirroring `ble_windows.rs`. Un-fragmented/legacy payloads pass through;
single-fragment (total=1) headers are stripped; buffers are bounded (30s expiry,
4096-fragment cap). Never decodes/verifies partial bytes.

- Branch `fix/ble-gatt-ingress-reassembly`, commit `1831ca4e`, PR **#250**.
- Verified: `cargo test -p scmessenger-cli --bin scmessenger-cli ble_mesh::` -> **6/6 pass**
  (4 new reassembly/edge tests + 2 existing; no regression).
- Windows node redeployed to PR merge build (Core Provenance 0.4.0 `9b3980b:HEAD`),
  identity preserved (pubkey `30d0fa67` / peer `12D3KooWD6vZQrU`), meshed with AWS relay.

### Remaining LGTM step (operator-gated)

Pixel 6a in BLE range + foregrounded, then Pixel->Windows (contact add + message)
over BLE to confirm a large fragmented envelope reassembles and verifies cleanly (no
further `BufferTooShort` / `equation not satisfied` on `ble_gatt_ingress`).

## Prior (key-divergence) fix direction -- only if truncation fix does not clear it

The prior "post-rotation key divergence" (identity_ids `b6486de2`, `d01c3751`,
`b46dcf21`; peer ids `12D3KooWJoW9r`/`12D3KooWNkx3`) is NOT refuted as a possible
cause of some instance, but the live recurring symptom is truncation. If the
truncation fix does not clear `ble_gatt_ingress`, then:
- On Drift signature failure for a peer whose identity envelope was recently learned,
  re-resolve the peer's CURRENT public key and re-verify before rejecting.
- Ensure a device's signing key and its advertised public-key/identity_id can never
  diverge after reinstall/rotation (identity bookkeeping invariant).