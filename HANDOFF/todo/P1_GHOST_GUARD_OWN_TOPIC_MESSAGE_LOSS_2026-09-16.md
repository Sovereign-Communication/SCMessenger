# P1 - Nodes dropped every inbound message: ghost guard classified their OWN topic as a ghost

Status: FIXED and verified on the live Windows node; awaiting adversarial review
(rule 8 - core/src/transport) and cloud-node redeploy for parity.
Priority: P1 (silent total loss of inbound messaging on the LAN path)
Filed: 2026-09-16 by the Freebuff lane, from the operator report below
Fix commit: see `core/src/transport/swarm.rs` in the same commit as this ticket
Found because of: operator report - "sent a message to windows ... they aren't
showing as delivered, despite me being ON wifi."

## Symptom

Messages sent from the Pixel to the Windows node sat at `state=stored` /
`state=forwarding` forever with `acked_without_receipt` climbing, on Wi-Fi,
with a live LAN connection and a successful-looking transport result.

## Evidence chain

Phone side (`adb logcat`), repeated every ~60-120s while the message stayed stuck:

```
delivery_attempt msg=9ea92977-... medium=tcp_mdns phase=smart_router outcome=success
                   detail=ctx=outbox_retry route=12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw
delivery_attempt msg=9ea92977-... medium=tcp_mdns phase=aggregate outcome=accepted
                   detail=ctx=outbox_retry source=smart_router transport_ack=true
delivery_state   msg=9ea92977-... state=stored detail=awaiting_receipt_delay_sec=120 acked_without_receipt=23
```

Node side: **zero** `inbox_receive` for the entire window in which the phone
believed it had delivered. Nothing arrived, so no delivery ACK was ever sent,
so no receipt could exist.

Node side, the decisive line (startup of the previous build):

```
19:44:10.135062Z  Peer 12D3KooWD776... subscribed to topic: /scmessenger/peer/69805e17.../v1
19:44:10.135092Z  Auto-subscribing to discovered topic: /scmessenger/peer/69805e17.../v1
19:44:10.135136Z  Peer 12D3KooWD776... subscribed to topic: /scmessenger/peer/30d0fa67.../v1
19:44:10.135163Z  GHOST-IDENTITY-001 skip auto-subscribe ghost peer topic: /scmessenger/peer/30d0fa67.../v1
```

Identity attribution settled by decoding the peer ids (base58 -> identity
multihash -> embedded ed25519 key), not by inference:

| Node | libp2p peer id | embedded public key hex |
|---|---|---|
| Windows node | 12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw | `30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e` |
| Pixel | 12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn | `30dce2bb779b4f1419f6d7d9e91b3ae201aed9e3b181aef674a9496f340a0645` |
| AWS node | 12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31 | `69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c` |

So the skipped topic `30d0fa67...` **is the Windows node's own identity**, and the
phone is subscribing to it precisely because it addresses the node there.

## Root cause

Senders publish to `/scmessenger/peer/<recipient-identity-hex>/v1`.
`is_ghost_peer_topic()` (GHOST-IDENTITY-001, added 2026-09-11) classifies a
64-hex peer topic as a ghost unless `ledger_manager.get_preferred_relays(64)`
contains a matching entry with `success_count > 0`. **A node never dials itself,
so its own key can never satisfy that test** - the guard therefore classified
every node's own topic as a ghost, on every node, deterministically. The node
refused to subscribe to its own topic, and:

- gossipsub `publish` to a topic with no subscriber returns **Ok**, so the
  sender's core call reported success (`sendMessageStatus == null`);
- Android recorded that as `transport_ack=true` and moved to
  `state=stored`/`awaiting_receipt`, correctly waiting for a receipt;
- no message ever reached the recipient, so no receipt could ever arrive.

Notably the earlier design did not self-subscribe at startup either (only
`sc-lobby`, `sc-mesh`, `sc-receipt-convergence`), so a node's reachability
depended on a peer's `Subscribed` event surviving the ghost guard - which it
never did for the node's own topic.

## Fix (core/src/transport/swarm.rs, one file)

1. `is_ghost_peer_topic()` takes `own_peer_key_hex: Option<&str>` and returns
   `false` (never a ghost) when the topic's key equals our own identity.
2. The node subscribes to its own peer topic at startup, derived from
   `swarm.local_peer_id()` via the existing
   `extract_ed25519_public_key_from_peer_id()`, and records it in
   `subscribed_topics`. This removes the dependency on a peer subscribing first.

Both halves are required: (1) stops the wrong classification, (2) guarantees
reachability even if a peer never subscribes.

## Verification (live, Windows node, build containing the fix)

Binary provenance checked before the run: the release exe contains the new
string `Subscribed to own peer topic` (grep count 1) and was rebuilt in 11m00s.

```
20:31:39.047995Z  Subscribed to own peer topic: /scmessenger/peer/30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e/v1

20:31:40.137827Z  inbox_receive message_id=9ea92977-44cd-4aa4-9888-7099cfd4b43e sender_id=f83ab163...   <- operator message
20:31:40.138763Z  Sending delivery ACK for 9ea92977-44cd-4aa4-9888-7099cfd4b43e to 12D3KooWD776...
20:31:40.199692Z  inbox_receive message_id=7eaba29b-2e95-41ae-8fa3-ecad47eef5b9 sender_id=f83ab163...   <- operator message
20:31:40.274665Z  inbox_receive message_id=b7b1fcce-459a-41d2-9b73-275ff41d9938 sender_id=f83ab163...   <- operator message
20:31:40.666817Z  inbox_receive message_id=034f6b1f-5f46-424e-8bbf-01f5bdc1ff58 sender_id=f83ab163...   <- machine envelope
20:32:40.038506Z  inbox_receive message_id=e3b20df9-7523-431a-8411-cc307b372b87 sender_id=f83ab163...   <- machine envelope
```

Every message that had been stuck for hours arrived within ~2s of the node
coming up with the fix, and each one earned a delivery ACK back to the phone.
Node-side return path also resumed in the same window (receipts for the node's
own outbound messages to the phone were processed: `81c846c1`, `17ad45c4`,
`efd33378`, `Delivered: <id>`).

Auto-reply 1:1 behaviour re-confirmed live on the same run: 6 inbound, **3
auto-replies, exactly one for each genuine chat message** (`9ea92977`,
`7eaba29b`, `b7b1fcce`) and **zero** for the three machine envelopes.

Phone side captured on the same device, same window (Pixel 6a
`26261JEGR01896`, APK with identity `f83ab163` / `12D3KooWD776`):

```
10:31:40.621  [RECEIPT-RX] Received from core: msg=9ea92977-... status=Delivered
10:31:40.651  [RECEIPT-RX] Emitted MessageEvent.Delivered: msg=9ea92977-...
10:31:40.699  [RECEIPT-RX] Received from core: msg=7eaba29b-... status=Delivered
10:31:41.315  delivery_state msg=9ea92977-... state=delivered
10:31:41.327  delivery_state msg=7eaba29b-... state=delivered
10:31:41.286  delivery_state msg=b7b1fcce-... state=delivered
```

`files/pending_outbox.json` on the device now reads `[]` - the stuck
messages drained. The return path is proven in the same second: the phone
received the node's own queued messages as real inbound traffic
(`delivery_attempt msg=81c846c1-... medium=core phase=rx outcome=received
detail=sender=30d0fa678c2...`, then a notification posted for that peer).

## Still open (tracked, not silently dropped)

1. **Cloud-node parity.** The AWS node runs image `testbotz/scmessenger:sha-31776b4`
   (up 44h), built before this fix: `docker exec ... strings
   /usr/local/bin/scmessenger-cli | grep -c "Subscribed to own peer topic"`
   returns **0**, so it still carries the unfixed guard: a message addressed to
   the cloud node's own topic is dropped the same way, and the cloud node's
   subscriptions to peers' topics are gated by the same impossible ledger test.
   Operator approved the deploy path; `docker-publish.yml` was dispatched on
   this branch (run 35147959061, head `6acaa2317`) to publish
   `testbotz/scmessenger:sha-6acaa23`. The container swap preserving `/data`
   is the remaining step; until it lands, cloud-addressed messaging is NOT
   proven.
2. **The guard still skips live-but-unproven peers.** On this same run the
   Windows node skipped `/scmessenger/peer/69805e17.../v1` - the AWS node's
   topic - because AWS has no `success_count > 0` entry inside
   `get_preferred_relays(64)`. That is the same false-positive class at a
   different address, and it is what would block a cellular-only phone whose
   only live path is through the cloud node. The correct test for "ghost" is
   not "never succeeded in the ledger" but "no longer live": a peer we are
   currently connected to should never be classified as a ghost. That is a
   follow-up design change, not part of this narrow fix.
3. **Adversarial review (rule 8).** This change is in `core/src/transport/`.
   It cannot be considered done until a recorded adversarial APPROVE from a
   reviewer that did not author it is on file. The change narrows behaviour
   (adds one exemption for our own identity and one self-subscription); it does
   not add a new code path that accepts external input.
4. **Silent-failure ergonomics (diagnostic only).** A publish to a topic with
   no subscriber is indistinguishable from success to the sender, and the
   Android layer labels that a transport ACK. The UI already waits for a real
   receipt, which is the correct contract, so no behaviour change is filed
   here - but an operator-visible "no subscriber for recipient topic" warning
   on the sending side would have surfaced this in minutes instead of hours.
5. ~~**Phone-side confirmation.**~~ LANDED: `[RECEIPT-RX]` and
   `state=delivered` were captured on the device for all three operator
   messages, and the pending outbox drained to `[]`. See the Verification
   section above.
6. **A transient false alarm worth recording.** `adb devices` returned empty
   mid-session because this lane ran `adb kill-server`, which drops the
   wireless-debugging (TLS) connection; the device re-registered over mDNS
   afterwards. Do not kill the adb server while the device is connected only
   over wireless debugging - it costs a reconnect and looks like a missing
   device.

## Scope note

The previous pass carried an instruction not to touch the ghost-topic policy.
The RCA shows that policy is the direct cause of the operator's symptom, so
that constraint was deliberately revised for this fix and the revision is
recorded here rather than applied silently.
