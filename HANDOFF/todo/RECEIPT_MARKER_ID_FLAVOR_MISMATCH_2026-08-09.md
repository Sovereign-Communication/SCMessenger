# Receipt/convergence markers are discarded as marker_not_locally_tracked -- outbox never dequeues

Status: Active
Severity: P0 (delivery truth -- successful deliveries are re-sent up to 12 times)
Discovered: 2026-08-09, Windows lane, live node run at anchor `49bc3f56`
Gate: `core/src/` delivery path -- adversarial review required before merge

## Summary

A receipt/convergence marker for a delivered message DOES arrive, and is then
thrown away with `reason=marker_not_locally_tracked`, because the marker is
keyed by a **synthetic routing id** while the outbox tracks the **real message
id**. The outbox entry therefore never clears and the retry machinery re-sends
a message the peer already holds.

This is the same identifier-flavor class as
`T1_DUAL_FLAVOR_BLOCK_DISPATCH` (blocks, fixed) and
`CONTACT_LOOKUP_PUBKEY_FLAVOR_MISS_2026-08-09` (contacts, open). Three
instances now. Consider fixing the pattern, not the instance.

## Evidence (live run, orchestrator-verified, not delegated-claim)

Two independent messages, two different transports, same outcome:

```
06:35:51.010  [OK] Message relayed successfully via <relay> to <macOS> (254ms)
06:35:51.303  WARN Ignoring convergence marker
              message=12D3KooWNC5rEK...-1786343750194
              reason=marker_not_locally_tracked
06:40:52.737  outbox_retry_attempt attempt #1/12
06:41:24.376  [OK] Message delivered successfully to <macOS> (117ms)   [direct]
06:41:52.739  outbox_retry_attempt attempt #2/12
06:41:52      outbox_retry_attempt attempt #1/12   (the second message)
```

`Ignoring convergence marker` fired **4 times** this run; `reason` was
`marker_not_locally_tracked` on all 4 (`grep ... | uniq -c` = 4).

Both messages still report `status: pending, delivered: false` from
`GET /api/send/:id`. Outbox flushes end `succeeded=0 failed=N`.

## Mechanism

Two id forms for the same message:

- Outbox / API id: `8e114d41-c6cc-4e7f-aa60-99220895aa88` (uuid), recorded by
  `event="outbox_enqueue" message_id=8e114d41-... queued_at=1786343750194`.
- Routing/marker id: `<destination_peer_id>-<queued_at_millis>`, e.g.
  `12D3KooWNC5rEK...-1786343750194`.

The `1786343750194` suffix is exactly the outbox `queued_at`, so the two ids
are mechanically relatable -- but the marker matcher does not relate them, and
reports the marker as not locally tracked.

Start from the `Ignoring convergence marker` emit site in
`core/src/transport/swarm.rs` and the outbox tracking map it consults; compare
against `event="outbox_enqueue"` in `core/src/store/outbox.rs`.

## Why this is P0 rather than cosmetic

- Delivery truth: a delivered message is reported `pending` forever.
- Amplification: every delivered message is re-sent up to 12 times, over the
  relay, for peers we already reached. On a farm topology with constrained
  uplinks that is real traffic.
- It masks the very fix the two lanes are trying to validate: the receipt
  handling change cannot clear an outbox entry when the callback never
  receives an attributable marker.

## Fix sketch (do NOT implement without the adversarial gate)

Either (a) carry the real message id through the routing/marker path so the
marker returns keyed by it, or (b) have the matcher accept both flavors by
deriving `<peer_id>-<queued_at>` from the outbox entry at match time.

(a) is the cleaner long-term shape and matches how blocks were fixed (store
both flavors so either inbound form hits). (b) is smaller and reversible.
Whichever is chosen, the marker path must not silently drop an unmatched
marker -- downgrade to a counter plus a WARN that names both ids, so the next
occurrence is diagnosable without a full re-run.

## Acceptance

- Send a message to a connected peer. Assert `/api/send/:id` transitions to
  delivered, AND `outbox_retry_attempt` does not appear for that id after the
  delivery log line.
- Assert `Ignoring convergence marker ... marker_not_locally_tracked` count
  stays at 0 for the run.
- Regression test at the unit level: a marker built from
  `<peer_id>-<queued_at>` matches an outbox entry created with that
  `queued_at`.

## Not yet established

Whether the recipient also fails to emit a proper application-level receipt
(distinct from this convergence marker). The macOS lane has been asked to
confirm receiver-side. Do not close this ticket on the sender-side fix alone.
