# Windows -> GPT: Android state for the 5-node test (2026-08-03)

Status: Active
Requested tier: **GPT-5.4 mini** for the log pull and the iOS install; escalate to
Sol Ultra ONLY if the iOS-side analysis turns into a design question.

## Headline: the operator's failing test was against a STALE build

Do not spend effort explaining the failure before reading this.

- Installed Android build: **lastUpdateTime 2026-08-02 11:55**
- App pid **unchanged for ~16 hours** -- the process has never been restarted
- PR 129 merged to main at 2026-08-03 17:58 UTC, so NONE of yesterday's or
  today's fixes are on that device
- The BLE inbound wedge fix is not even on main yet (PR 131, open)

So "no messages either direction" is the expected behaviour of that build. It is
not new information, and iOS-side logs from the same window will show the
mirror image of a known-broken Android peer.

## Android state, from logcat this morning (08:13-08:18 HST)

| Signal | Value | Note |
|---|---|---|
| `mesh_ble_forward` (entry) | 0 | no BLE inbound at all now |
| `mesh_ble_forward_return` | 0 | n/a, nothing entered |
| `Message from` (Kotlin delegate) | 0 | delegate never fired |
| Core peer count | 1 | a peer IS connected |
| Pending outbox | **12 items, growing** | was 9, then 10, now 12 |

Every one of the 12 is
`state=held detail=acked_without_receipt_protection acked_count=1`, and each
flush logs `Skipping retry ... transport-acked message cannot be downgraded`.
The two newest are the operator's just-sent messages -- they wedge on arrival.

Note the CHANGE from yesterday: BLE inbound has gone from 264 forwards to zero,
while the core peer count went 0 -> 1. So the transport situation is different
today, not merely "still broken". Worth keeping in mind before assuming
continuity.

## Root cause already found and fixed (PR 131, not yet merged)

Every access to the shared `core` mutex in the swarm event loop held the guard
ACROSS the call into IronCore:

    mobile_bridge.rs:829   let core_guard = core.lock();
                           core_ref.receive_message(...)   // held throughout

`get_core()` (:1514) is `self.core.lock().clone()`, and the BLE path
(`MeshService::on_data_received`, :1385) calls it on the GATT callback thread as
a SYNCHRONOUS UniFFI call. While the swarm loop was inside `receive_message`,
every inbound BLE message blocked in `get_core()`. Hence 264 forwards / 0
returns. It also explains the ANR and the non-draining outbox: receipts arrive
over that same blocked path, so nothing is ever confirmed delivered and the
retry guard holds messages forever.

All eight sites are converted to clone-then-release. Verified to compile and
format locally; NOT yet verified on device.

## What we need from the Mac lane

### 1. iOS logs for the SAME window (08:13-08:18 HST 2026-08-03)

Specifically:
- Does iOS RECEIVE the Android messages? The Android side shows them
  transport-ACKed, so something acknowledged them. If iOS has them, the gap is
  purely the application-level receipt, not delivery.
- Does iOS EMIT a delivery receipt, and over which transport?
- Does iOS show an inbound message from Android in its UI?
- iOS core peer count.

That first question is the important one. Transport-ACK without receipt is
consistent with EITHER "iOS got it and never acknowledged" OR "the ACK came
from a lower layer and iOS never saw it". The iOS log distinguishes them and we
cannot from this side.

### 2. Confirm the iOS build vintage

Same trap as Android. Give the build timestamp so we do not compare a fresh iOS
build against a day-old Android one and draw the wrong conclusion.

### 3. macOS CLI node -- still outstanding

Start it, and prove it BOUND a port: `netstat`/`ss` matched to the PID plus the
real log line showing the listen address. Do not accept exit code 0. `cli/src/
main.rs` was gutted to a 170-line stub and restored, and to our knowledge has
never run end-to-end since.

## Plan for the 5-node test

The matrix is iOS / Android / macOS CLI / Windows CLI / Cloud. It is not worth
running until both phones carry current builds -- otherwise we are testing
yesterday's bugs.

Sequence:
1. PR 131 merges (BLE wedge fix)
2. Android logging to logcat lands -- REQUIRED. The Rust core is currently
   SILENT on device: `eprintln!` goes to a stderr Android discards
   (`log.redirect-stdio` unset, verified) and no tracing subscriber is bridged
   to logcat. Without it we cannot prove the wedge fix worked; we would be
   guessing again.
3. Fresh APK from CI onto the Pixel, fresh iOS build onto Christy's phone
4. Then the 5-node matrix

Proof the fix worked, on device: `mesh_ble_forward_return` appearing at parity
with `mesh_ble_forward`, and the 12 wedged messages draining.

## Redaction

Repo is PUBLIC. No peer ids, public keys, BLE MACs or IP addresses in anything
committed. Message ids and timestamps are fine and are what we are correlating
on.

## Reply

`HANDOFF/gpt/GPT_RESPONSE_5NODE_IOS_STATE_2026-08-03.md`

## ADDENDUM -- MAJOR FINDING (operator hypothesis, verified in source)

**Identity hash and public key are two incompatible keying schemes, and they are
indistinguishable.**

    public_key_hex()  = hex(ed25519_pubkey)          -> 64 hex chars
    identity_id()     = hex(blake3(ed25519_pubkey))  -> 64 hex chars

Both decode to exactly 32 bytes. Every length/hex validation in the codebase
passes for either. The hash is one-way, so an identity_id can never be turned
back into a key.

Conflicting users of one contact store:
- `prepare_message_internal` decodes `recipient_id` and uses it DIRECTLY as the
  X25519 `recipient_pk` -> must be a PUBLIC KEY
- the same function sets `sender_id = identity_id()` -> a HASH
- `receive_message` looks contacts up by `hex::encode(&sender_pubkey)` (PUBLIC
  KEY) but runs blocked checks on `message.sender_id` (HASH)
- Android `addContact` keys contacts by PUBLIC KEY;
  `MeshRepository.onPeerIdentityRead` keys them by the beacon's `identity_id`
  (HASH)

Consequence: if a contact's id holds a hash, the send path encrypts to 32 bytes
that are not anybody's key. Nothing rejects it. That is the operator's "failed
to send - cryptographic error on one contact, the other works" -- the difference
is which scheme created each contact.

Full writeup, with file:line for every claim:
`HANDOFF/audit/IDENTITY_HASH_VS_PUBKEY_CONFLICT.md`

### What we need from iOS on this -- tier 5.4 mini is fine for the read

The BLE identity beacon carries BOTH `public_key` and `identity_id`, so the wire
format is fine; the bug is which field each consumer keys on. Please report:

1. Which field does iOS key contacts/peers on -- `public_key` or `identity_id`?
2. What does iOS put in a message's `sender_id` and `recipient_id`?
3. Does iOS ever use a 64-char hex value from one scheme in the other's slot?

If iOS keys on `identity_id` anywhere, the two platforms disagree about peer
identity and fixing Android alone will NOT make messaging reliable. Please check
before we run the 5-node matrix -- this determines whether the matrix is even
meaningful.

Windows recommendation: canonicalise on the PUBLIC KEY everywhere, because
encryption requires it and the hash cannot be reversed; keep identity_id as a
display/verification value or an index that resolves to a public key.
