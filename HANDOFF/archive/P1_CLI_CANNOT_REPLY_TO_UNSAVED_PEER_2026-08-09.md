# P1 -- Three defects in the CLI send path (`handle_send_message`)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Severity: P1 (blocks the reply leg of any five-node gate, and corrupts its scoring)
Discovered: 2026-08-09, live on the Windows node during the candidate soak
Affects: `scmessenger-cli` (Windows, macOS, Linux desktop nodes)
Candidate observed on: `33c16712`

All three live in the same ~80-line handler, `cli/src/api.rs:561-632`.

| # | Defect | Run-1 impact |
|---|---|---|
| A | Cannot reply to a peer that is not in contacts | Reply leg fails; reads as a transport fault |
| B | `/api/send` reports failure for messages that are then delivered by retry | False negatives; under-reports delivery |
| C | Outbound messages are never written to durable history | Sender-side persistence cannot be scored at all |

Defects B and C were found while sending the reply that worked around A.

---

# A -- CLI can receive and decrypt from a peer it cannot reply to

## Summary

The CLI accepts, decrypts, and durably stores a message from a peer that is not
in its contacts store -- but it cannot send anything back to that peer, because
`/api/send` resolves the recipient **only** against the contacts list. The reply
fails with `Contact not found` and a `404`.

Android does not behave this way: it auto-creates a contact on discovery
("Auto-created/updated contact for discovered peer", observed in
`docs/fieldtest/PR139_WINDOWS_LANE_CORRELATION_2026-08-09.md`). The two
platforms disagree about whether an inbound peer becomes addressable.

## Reproduction (actually observed, not theoretical)

1. Start the Windows CLI on a clean-ish contacts store (`Claude-Windows-Driver`,
   `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw`).
2. From the Pixel (`Lucaso`, identity `a43772fe...`, libp2p
   `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG`), send a message.
3. The CLI **receives and decrypts it** -- confirmed in durable history via
   `POST /api/history`, `direction: "received"`, timestamp `1786272628`
   (`2026-08-09T10:50:28Z`), full sender identity block intact.
4. Attempt to reply. All three of these return `404 Contact not found`:
   - `{"recipient":"a43772fe...”}` (sender identity id)
   - `{"recipient":"12D3KooWNnPi9wqUJ7..."}` (libp2p peer id)
   - `{"recipient":"Lucaso"}` (nickname carried in the received payload)

## Root cause

`cli/src/api.rs:568-572`, in `handle_send_message`:

```rust
let list = contacts.list().unwrap_or_default();
let contact = list
    .into_iter()
    .find(|c| c.peer_id == request.recipient || c.nickname.as_ref() == Some(&request.recipient))
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Contact not found".to_string()))?;
```

Recipient resolution is a linear scan of the contacts store on `peer_id` or
`nickname` only. There is no fallback to:

- the identity id of a peer already in durable history,
- the live peer table (`/api/peers` listed the peer as connected at the time),
- the sender identity block that arrived inside the message itself, which
  already carries `identity_id`, `public_key`, `device_id`, `nickname`, and
  `libp2p_peer_id`.

Every field needed to address a reply was already in hand and on disk. The send
path simply does not look anywhere except contacts.

## Why this matters for the five-node gate

The G-gates require **both directions** of an exchange with receipts. As it
stands, a desktop node that is messaged first by a mobile node cannot answer
until a contact is manually created. That would present during a run as "the
Windows node never replied" and be misread as a transport, receipt, or custody
failure, when it is a recipient-resolution bug in the local API.

**Workaround for Run 1 (do this if the fix does not land first):** pre-seed
contacts on every desktop node before the run, via
`POST /api/contacts {"peer_id": "<libp2p peer id>", "public_key": "<hex>",
"name": "<nickname>"}`. Confirmed working -- after adding the Pixel this way,
the reply sent and delivered direct in 81 ms (`10:53:51Z`,
`route=direct`, `policy_reason=DIRECT_FROM_ROUTING_ENGINE`).

## Suggested fix

Extend recipient resolution in `handle_send_message` to fall back, in order:

1. contacts by `peer_id` or `nickname` (current behaviour),
2. contacts by identity id / public key,
3. a peer present in durable history, using the stored sender identity block,
4. a currently connected peer from the live peer table.

Decide explicitly whether the CLI should auto-create a contact on first inbound
message, as Android does. Consistency between platforms is the real fix; the
resolution fallback is the minimum.

**Security note:** this is a gated path in spirit even though it is not under
`core/src/{crypto,transport,routing,privacy}/`. Auto-creating contacts from
inbound traffic means an unsolicited peer can insert itself into the local
contacts store. Android already does this, so the behaviour exists in the
product, but the CLI change should be reviewed with that in mind rather than
copied uncritically -- and it must interact correctly with the blocked-peer
gates added in this PR.

## Acceptance criteria (A)

1. A desktop node messaged first by an unknown peer can reply without manual
   contact creation.
2. Blocked peers remain unreplyable -- the fallback must not bypass
   `is_peer_blocked`.
3. Regression test covering reply-to-unsaved-peer and reply-to-blocked-peer.
4. Platform behaviour documented: whether CLI auto-creates contacts, and if it
   diverges from Android, why.

---

# B -- `/api/send` reports failure for messages that are then delivered

## Observed

`POST /api/send` returned `500 Failed to send message via BLE and Swarm` at
`2026-08-09T11:11:20Z`. The node log for the same message id, same second:

```
11:11:20.672  ROUTE_DECISION message_id=...-1786273880672 attempt=1 pass=0
11:11:20.807  WARN Delivery pass failed for message ...-1786273880672; continuing cyclic retries
11:11:20.808  ROUTE_DECISION message_id=...-1786273880672 attempt=2 pass=1
11:11:20.835  [OK] Message delivered successfully to 12D3KooWNnPi9wqUJ7... (27ms)
```

**The message was delivered 28 ms after the API told the caller it had failed.**

## Root cause

`cli/src/api.rs:612-627`:

```rust
let sent = crate::ble_mesh::send_ble_message(&peer_id.to_string(), &prepared.envelope_data)
    .await
    .is_ok()
    || ctx.swarm_handle.send_message(peer_id, prepared.envelope_data, None, None).await.is_ok();

if !sent {
    return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to send message via BLE and Swarm".to_string()));
}
```

`sent` captures only the outcome of the **first** BLE attempt and the **first**
swarm attempt. The swarm's cyclic-retry machinery keeps working after
`send_message` returns, and frequently succeeds on a later pass -- but the HTTP
response has already been decided. The API result and the delivery outcome are
two different facts, and only the pessimistic one reaches the caller.

## Run-1 impact

Any harness that scores sends on the `/api/send` response will **under-report
deliveries**. A message counted as failed here was on the wire and acknowledged.
This is a false negative in the exact direction that makes a passing run look
like a failing one.

## Acceptance criteria (B)

1. The response distinguishes "rejected outright" from "queued, retrying" from
   "delivered". A 202-with-message-id plus a status lookup is the natural shape.
2. Run-1 scoring does not treat a non-200 from `/api/send` as proof of
   non-delivery; correlate on message id in the node log instead.

---

# C -- outbound messages are never written to durable history

## Observed

After two confirmed outbound deliveries from this node (81 ms at
`10:53:51Z`, 27 ms at `11:11:20Z`), `POST /api/history {"limit":10}` returned
**only `direction: "received"` rows**. Neither sent message appears. The local
conversation view is therefore half a conversation.

## Root cause

`handle_send_message` prepares the envelope, attempts delivery, and returns. It
never touches `core.history_store_manager()`. Receives are recorded (the inbound
path emits `event="inbox_receive"` and rows appear in history); sends are not.

## Run-1 impact

This is the more serious of the two. The agreed scoring standard is **receiver
decrypt + durable history + receipt**. On the sender side there is no durable
history to check, so send-side persistence cannot be evidenced at all on a
desktop node -- and a restart loses any record that the node ever sent anything.

## Acceptance criteria (C)

1. A successful send writes a `direction: "sent"` row to durable history with
   message id, recipient, and timestamp.
2. The row survives a node restart.
3. Confirm whether Android and iOS record outbound messages; if they do, this is
   also a platform-parity gap and belongs in `docs/FEATURE_PARITY.md`.
4. Decide whether a queued-but-not-yet-delivered send is recorded, and with what
   status, so B and C stay consistent.
