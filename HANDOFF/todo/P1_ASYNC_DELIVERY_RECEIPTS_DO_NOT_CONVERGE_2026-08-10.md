# P1 -- async delivery receipts never converge; sender stays `pending` forever

Status: FIXED -- dispositioned 2026-08-24 against main ceabdbd4
Disposition: the receipt branch in IronCore::receive_message now calls
`mark_message_sent(receipt.message_id)` (iron_core.rs:3533) gated on
Delivered | Read (:3529-3532, NOT Sent -- the ticket's dequeuing caution is
honored); `mark_message_sent` removes outbox + drift-store entries. Regression
coverage: core/tests/integration_ironcore_roundtrip.rs:329-445
`test_receipt_roundtrip_flips_state` asserts outbox cleared after a Delivered
receipt and fails without the fix. Move to HANDOFF/done/ at next sweep.
Filed: 2026-08-10 ~01:40Z (Windows lane)
Ties into: the existing async receipt-convergence effort (`sc-receipt-convergence`)

## This is NOT "async is slow". Async would converge. This does not.

The design is asynchronous by intent -- `/api/send` accepts a message into the
outbox and the retry machinery delivers later. That is correct and is not the
defect. The defect is that the sender's view **never catches up with reality**,
even after the receiver has the message and an ACK has been exchanged.

## Evidence, both directions

**Windows -> macOS.** Message `21831e84-cd6c-463f-86c9-85ea677aaa88` sent
01:08:41Z. The macOS lane confirmed it in Mac history with `delivered=true`,
content matching exactly. More than ten minutes later the Windows sender still
reports:

```
{"message_id":"21831e84-...","status":"pending","delivered":false}
```

Same for the follow-up `ef9f0318-3ef5-4d4a-91d4-522628ac2728`.

**macOS -> Windows, symmetric.** Their probes `778f9437-...` and `2d7867de-...`
were received by Windows and ACKed:

```
01:08:10.288Z  inbox_receive  message_id=778f9437-...  -> Sending delivery ACK to 12D3KooWP1hv...
01:08:44.557Z  inbox_receive  message_id=2d7867de-...  -> Sending delivery ACK to 12D3KooWP1hv...
```

Their sender-side status stayed "accepted but still pending" for both.

So **both lanes independently show the same failure**: delivery succeeds, the
receipt does not make it back into the sender's status.

## Why it matters beyond cosmetics

1. **It makes the five-node run unscoreable from the sender side.** Scoring has
   to fall back to receiver-side `inbox_receive` plus the ACK, which means every
   delivery claim requires access to the *receiving* node. That is workable for
   two lanes and impractical for five.
2. **It is indistinguishable from real failure.** An operator watching
   `/api/send/:id` sees a message that never delivers. There is no way to tell a
   stuck message from a delivered one.
3. **It probably drives redundant retries.** The outbox logged
   `outbox_retry_attempt (attempt #1/12)` for a message the peer already had.
   Retrying delivered messages wastes dial budget and feeds the concurrent-
   connection storms behind the P0.

## The existing effort this attaches to

The node already subscribes to a delivery-convergence gossip topic:

```
16:05:45  Subscribed to delivery convergence topic: sc-receipt-convergence
16:05:49  Peer 12D3KooWP1hv... subscribed to topic: sc-receipt-convergence
```

So the async receipt-convergence mechanism EXISTS and both peers are subscribed
to it. This ticket is not a request to design one -- it is that the existing one
is not closing the loop. Start there rather than building a parallel path.

Prior related work worth reading first:
- `HANDOFF/done/CRITICAL_ANDROID_FALSE_DELIVERY_FAILURE_NO_RECEIPT_ACK.md`
- `HANDOFF/done/P1_CORE_004_Mobile_Receipt_Wiring.md`

## Strong lead: the identifiers do not match

One exchange used three different identifier forms:

| Where | Value |
|---|---|
| addressed by the caller | `12D3KooWP1hvZbqCCPMMfrZbW16EHy7wXp41pDPWtHzdn3MbwG5e` |
| outbox retry target | `c40fa8137108c523541739f1384a63df93f1f038c7208f3db7d14449a3d71239` |
| inbound `sender_id` | `7dad8fdf5dfce395a15ef88ac88870554fa580a38a57fb5cdf49ff109851ce17` |

Neither 64-hex value is the peer's PeerId or its published public key
(`a185af9484e8f42ef5eeea4f431371ec89895ef24adb0991a17625663b941d0c`), and
neither is a plain SHA-256 of the PeerId string, the public-key hex string, or
the public-key bytes -- all three were checked and ruled out.

**Hypothesis:** an ACK arrives keyed by one identifier form while the outbox
entry is keyed by another, so the receipt never matches an outstanding message
and the status is never updated. That would explain the symmetry across lanes
exactly.

This is also the identifier-unification concern raised by the operator: the
forms may all be legitimate and necessary, but they must map to each other
losslessly and be resolvable in both directions.

## Acceptance criteria

1. Enumerate every identifier form in the messaging path with its derivation,
   cited by `file:line`: PeerId, public key, `sender_id`, outbox `peer_id`, and
   any contact/device id. Produce the mapping table. Unification is optional;
   **a documented, tested, bidirectional mapping is not**.
2. Identify where the ACK is matched to an outbox entry and prove which
   identifier each side uses.
3. After a confirmed delivery, `/api/send/:id` transitions to delivered within a
   bounded, documented time.
4. The outbox stops retrying a message once its receipt is recorded. Assert the
   retry count stops climbing.
5. Regression test: enqueue, deliver, ACK, assert sender status converges. It
   must fail without the fix.
6. Verify across a real two-node pair, not only in-process -- this failed
   identically on two different platforms, so an in-process test may pass while
   the real path stays broken.

## Sequencing

**Queued behind the five-node anchor rollout** per operator direction. Do not
start this while nodes are being re-anchored to `68fcc3f1`; a moving target
during a receipt investigation would waste the run.

---

## ROOT CAUSE FOUND AND VERIFIED 2026-08-10 ~03:25Z

**The receipt loop is open. Receipts arrive, are decoded, notify the UI delegate,
and are then dropped without ever touching the outbox.**

My identifier-mismatch hypothesis above was **WRONG**. Recorded rather than
deleted, because the wrong theory shaped two comments to the other lane.

### The code, verified by reading it (not inferred)

`core/src/iron_core.rs:3423-3444`:

```rust
if message.message_type == crate::MessageType::Receipt {
    if let Ok(receipt) = crate::message::types::decode_receipt(&message.payload) {
        if let Some(delegate) = self.delegate.read().as_ref() {
            let status_str = match receipt.status { ... };
            delegate.on_receipt_received(receipt.message_id, status_str);
        }
    } else if let Err(e) = ... { /* parse error logging */ }
    // Fall through to generic pipeline steps
}
```

The receipt is decoded and handed to the delegate. **`mark_message_sent` is
never called.** The function exists and does exactly the right thing --
`core/src/iron_core.rs:1008`:

```rust
/// Mark a message as sent (remove from outbox after transport confirms delivery).
pub fn mark_message_sent(&self, message_id: String) -> bool {
    let outbox_removed = self.outbox.write().remove(&message_id);
    ...
    let drift_removed = ...;
    outbox_removed || drift_removed
}
```

Its callers are the local send path (`iron_core.rs:1048`), a test
(`iron_core.rs:4903`), and the CLI (`cli/src/main.rs:3779`). **No caller on the
receipt-handling path.**

So an inbound delivery receipt updates the UI and nothing else. The outbox entry
survives, `/api/send/:id` keeps reporting `pending`, and the retry machinery
keeps re-sending a message the peer already has -- observed at
`outbox_retry_attempt (attempt #1/12)` for a delivered message.

### Why it presents symmetrically

Both lanes run the same code. Nothing lane-specific is involved, which is why
Windows and macOS showed the identical failure in opposite directions.

### On the identifiers -- the question is answered, and it is not the bug

`identity_id` (Blake3) and the libp2p `PeerId` are **both one-way hashes of the
same `public_key`**, by different algorithms. Neither can be derived from the
other; both can be derived from the public key. That is why neither 64-hex value
matched a SHA-256 of anything I tried.

So they are not redundant and they are not mismatched -- they are two valid
projections of one root identity. **`public_key` is the unifying root.** Any
mapping table should be built around it rather than trying to convert between
the leaves.

This still deserves documenting per the operator's unification request, but it
is **not** the cause of the receipt failure and should not block the fix.

### The fix

Call `mark_message_sent(receipt.message_id)` on the receipt path, after the
delegate callback, when `receipt.status` indicates delivery.

Cautions for whoever implements it:
- Only dequeue on a status that genuinely means delivered. `DeliveryStatus::Sent`
  and `Delivered` are currently collapsed into the same string for the delegate;
  do not collapse them for dequeuing.
- The receipt path deliberately falls through to dedup/metrics/persistence. Do
  not early-return and skip those.
- `mark_message_sent` takes the outbox write lock. Check for a lock-ordering
  hazard against the locks already held in this handler.
- Verify the sender-side status endpoint then reports delivered, and that
  `outbox_retry_attempt` stops climbing for that message id.

### Regression test

Enqueue, deliver, ACK, assert sender status converges to delivered AND the retry
count stops. It must fail before the change. An in-process test alone is not
sufficient -- this failed identically across two platforms, so verify on a real
two-node pair.
