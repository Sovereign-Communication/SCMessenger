# OUTBOX-SWEEP-001 — Outbox entries with expired grace timers are never re-flushed while the peer stays connected

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Todo (filed 2026-09-23, Buffy / Freebuff lane, from the 3-node audit)
Owner: SCMessenger -- this is a defect in this repository's own outbox
and custody path; the scope block above is the owner attestation.
Priority: P0 — message delivery stalls until a reconnect event; "messages not
being delivered" on a healthy mesh.
Found by: 2026-09-23 three-node log audit (this session).
Related: WP4 umbrella (`HANDOFF/freebuff/queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md`),
D8 (custody receipt convergence, `HANDOFF/todo/D8_CUSTODY_DELIVERY_RECEIPT_STATUS.md`),
R9-F3 comments in `core/src/iron_core.rs` (~3250–3379).

## Symptom, with evidence obtained by running commands

Windows node Control API (`http://127.0.0.1:9876/api/diagnostics`, pulled
2026-09-23 ~09:12Z):

```json
"outbox_count": 118, "history_stats": {"undelivered_count": 259}
```

AWS always-on node journal (2026-09-23, `docker logs --since 6h scm-node`),
repeated on every identify of BOTH peers:

```
event="outbox_flush_skipped" ... "Flush skipped for this connection; entries remain in the outbox"
```

Both cloud nodes show continuous seed-dial/gossip/custody health (no panics,
no conn-limit denials in 24h), yet undelivered counts stay stuck: the outbox
is waiting for reconnect events that never come because the connections never
drop.

## Root cause (code anchors, current main)

1. The reconnect-flush gate `register_and_flush_swarm_peer`
   (`core/src/transport/swarm.rs:1912-1935`) flushes exactly once per
   connection: `flushed_this_connection.insert(peer_id)`; the set is only
   cleared on `ConnectionClosed` (swarm.rs:6794 native loop; 9469 wasm loop).
2. The post-egress path in `IronCore::handle_peer_connection_event_with_egress`
   (`core/src/iron_core.rs:3257-3277`) re-enqueues dispatched entries as
   `Enqueued` with `next_retry_at = now + OUTBOX_EGRESS_GRACE_SECS (120s)` —
   by design a re-flush safety net for lost receipts.
3. Nothing in the native or wasm swarm loops ever drains the outbox again on
   that still-open connection once the grace timer expires. `retry_now` is
   only called on transport request failures (swarm.rs:4633/4697/8670/8707).
   There is no periodic sweep.

Net effect: a lost or never-sent receipt strands the entry until the
connection happens to bounce. On an always-on mesh with stable cloud links,
that is forever. This is the delivery stall.

## Acceptance criteria

1. A periodic (e.g. 60–120 s) sweep in the native swarm event loop re-invokes
   the flush for connected peers whose outbox entries are Enqueued AND due
   (`next_retry_at <= now`), via the same single-owner egress path — no
   duplicate dispatch storm (receipt-idempotency + due-check keep it bounded).
2. Same sweep exists for the wasm loop (parity; see the existing F5/WASM
   asymmetry note in `HANDOFF/review/D1_D9_HARNESS_ADVERSARIAL_FINDINGS_2026-09-22.md`).
3. Rust test: entry dispatched over a still-open connection with no receipt
   is re-dispatched after the grace window; a receipt clears it exactly once.
4. Live evidence: Windows `outbox_count` and `undelivered_count` decrease
   after deploy on stable connections (WP5 log pack).

## Gates

`core/src/transport/` + `core/src/iron_core.rs` — **Rule-8 adversarial review
mandatory** (flush ordering, duplicate-dispatch exposure, custody boundary).
This ticket's fix must not weaken the D9 patch (vendored libp2p-swarm) — no
changes to `vendor/` needed for the sweep.

## References

- `core/src/transport/swarm.rs` lines 1912–1935 (gate), 6794/9469 (clear on close)
- `core/src/iron_core.rs` lines 3196–3379 (flush + grace re-enqueue)
- `core/src/store/outbox.rs` `flush_peer_messages` (due check exists)
- WP4 / WP5 acceptance on the P0 umbrella ticket
