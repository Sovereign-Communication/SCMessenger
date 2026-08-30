# P0 (RELABELED) -- inbound Drift envelope truncation on BLE breaks send/verification

Status: OPEN (filed 2026-08-30, root cause CONFIRMED live 2026-08-30)
Operator report: "I'm failing to send message to the new node ID (something about
crypto failing), but the old Id still says delivered, but I'm getting no response."

## CONFIRMED ROOT CAUSE -- BLE envelope truncation / fragmentation (not key divergence)

Captured live during 3-node round-trip testing (Windows node log
`AppData/Local/scmessenger/logs/scm.log.*`, 2026-08-30T08:46Z), the recurring
"crypto failing" is a **truncated inbound Drift envelope over BLE GATT**, not a
signing-key mismatch:

```
08:46:06  Received PeerJoined: 12D3KooWNkx3AjDmXDHpweEsnNm164MS23nuMRVLajgaASyxBrow
          (the Pixel, via 192.168.0.129) with 2 addresses
08:46:09  WARN iron_core: Failed to decode drift envelope: BufferTooShort { need: 1358, got: 1173 }
08:46:11  Peer left: 12D3KooWNkx3AjDmXDHpweEsnNm164MS23nuMRVLajgaASyxBrow
08:46:38  WARN iron_core: Drift envelope signature verification failed:
          IoError("Signature verification failed: signature error: Verification equation was not satisfied")
          -- route=ble_gatt_ingress
```

Mechanism: a 1358-byte Drift envelope arrives as only 1173 bytes on
`ble_gatt_ingress`. The truncated bytes produce (a) a `BufferTooShort` decode
failure and, when it partially parses, (b) a corrupt signature that
`DriftEnvelope::verify()` rejects as "equation not satisfied"
(`core/src/drift/envelope.rs:730-741`; v1/v2 verify in
`core/src/crypto/encrypt.rs:889/945`). So the reported "crypto fails" is the
signature of a truncated envelope -- no key divergence involved on this path.

This reproduces the field ticket `P2_WIRE_ENVELOPE_TRUNCATION_2026-08-10`
(same Pixel 6a, same BLE ingress, earlier surfaced as "unexpected end of file"),
and the unified plan's warning that Android (~512-byte MTU) vs iOS (~185) BLE
chunking can cut a large envelope unless reassembly is correct on both sides.

## On-device (Pixel) evidence -- identity is consistent

- App installed + running (com.scmessenger.android, PID live), MeshSyncWorker OK.
- Advertised identity is internally consistent: libp2p peer
  `12D3KooWNkx3AjDmXDHpweEsnNm164MS23nuMRVLajgaASyxBrow`, identity `b46dcf21...`
- The Pixel meshes with the AWS relay `12D3KooWKMUXfjvW...` over internet and
  joins Windows directly via BLE, then disconnects after the truncation failure.
- No sender-side sign/advertised-key mismatch appears in the Pixel's own logs.

## Supporting live round-trip result

- Windows->Pixel send (`04d91475...`): accepted, but stuck `pending/delivered=false`
  -- the message never reached the Pixel (Pixel logcat stayed quiet), consistent
  with the relay/BLE path dropping the truncated envelope.
- Windows node peers only with the AWS relay; Windows<->Pixel is relay-mediated +
  BLE (fragile).

## Required next step / in-scope code fix (approved direction for next pass)

Implement BLE fragment/MTU **chunking + reassembly hardening** and a
**truncated-envelope retry** so an inbound envelope that is short
(`BufferTooShort`) is either reassembled from the remaining fragments or
retried, never decoded/verified against partial bytes:
  1. `ble_gatt_ingress` reassembly keyed by message id (accumulate fragments
     until the envelope's declared length is met; bounded time).
  2. On `BufferTooShort`, buffer + await the rest (do not verify partial bytes).
  3. Keep `verify()` failing-loud on genuinely corrupt frames (working as
     intended) but never on a flow that should have been reassembled.
Optionally re-verify against the peer's freshly-learned identity envelope only
for the residual non-BLE path; the BLE truncation is the confirmed in-scope fix.

## Reframed from the prior hypothesis

The prior "post-rotation key divergence" (identity_ids `b6486de2`, `d01c3751`,
`b46dcf21`; peer ids `12D3KooWJoW9r`/`12D3KooWNkx3`) is NOT refuted as a
possible cause of some instance, but the live recurring symptom is truncation.
Do not chase a key-mismatch fix until the truncation fix is proven to clear the
`ble_gatt_ingress` failures.

## NOT addressed by the pending merge lane

PR #244/#245/#246/#248 (coalescing, never-drop retry, versions, mobile_bridge
coalescing) do not touch the BLE ingress reassembly or Drift decode path.