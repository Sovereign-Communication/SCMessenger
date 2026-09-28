# TASK [P0 farm-critical]: Outbox enqueue-on-disconnect + flush-on-connect with retry

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: TODO. Re-scoped 2026-07-13 after a 296-line implementation attempt reached
95% (compiled clean, 1131/1132 lib tests pass) but one unit test stayed red across
4 free-lane fix attempts. Torn down per the verification protocol
([[feedback_micro_container_delegation]]) to keep main clean; the attempt is
preserved at `HANDOFF/review/OUTBOX_FLUSH_ATTEMPT_296LINES.patch` (reuse ~90% of it).

## Goal (closes CRITICAL_OUTBOX_NEVER_FLUSHES Site 1 + Site 3)
Messages to an offline recipient must be QUEUED and later FLUSHED when that peer
reconnects, with bounded retry/backoff - not dropped. (Site 2, the swarm.rs handler,
is already fixed on main.)

- **Site 1 - `core/src/iron_core.rs::prepare_message_internal`:** when the recipient
  is not currently connected, enqueue the outgoing message to the outbox store
  instead of failing/dropping it. Route peer-discovered/disconnected events to the
  outbox.
- **Site 3 - `core/src/store/outbox.rs`:** add `flush_peer_messages(peer_id)` that
  drains that peer's due queue entries and sends them via a callback, with an
  exponential-backoff scheduler (`next_retry_at`/`last_attempt_at` fields on
  QueuedMessage, `MAX_DELIVERY_ATTEMPTS`) and a returned sent-count. Also update the
  `QueuedMessage` instantiation in `cli/src/main.rs` for the new fields.

## The exact bug that blocked the attempt (fix this precisely)
Unit test `store::outbox::tests::test_flush_peer_messages` enqueues 2 msgs for
"peer_a" + 1 for "peer_b", marks peer_a connected, calls
`flush_peer_messages("peer_a")`, expects return 2 and `total_count()==1` after.
It returned 0. Prime suspects (confirm against the reference patch):
1. `enqueue` initializes `next_retry_at` to a FUTURE time, so the flush filter
   `next_retry_at.is_none() || next_retry_at.unwrap() <= now` excludes fresh msgs
   -> pending list empty -> early `return Ok(0)`. Fix: fresh enqueue should set
   `next_retry_at = None` (due immediately).
2. OR the send loop does not increment the returned count / does not `remove` sent
   entries so `total_count()` is wrong.
Do NOT weaken the test.

## Verification (protocol)
Compile gate + `cargo test -p scmessenger-core --lib store::outbox` green, then
Fusion Lite triangulation (once the OpenRouter key has a spend cap) or 3 distinct
DashScope Qwen verifiers must find no issues before commit. Not crypto/transport/
routing/privacy, so no mandatory adversarial gate - but it IS delivery logic; review
the retry/drop semantics carefully (a bug here silently loses farm messages).
