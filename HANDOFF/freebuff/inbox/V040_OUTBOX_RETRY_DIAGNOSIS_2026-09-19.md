# V040 outbox retry delay -- diagnosis and proposed change (2026-09-19)

Date: 2026-09-19
Lane: Freebuff / DeepSeek V4 Flash
Trigger: the operator's two messages (07:30 and 07:33 local) sat 4.5 and 8 minutes
before leaving the phone.
Status: **diagnosis only, no code change.** The fix's wiring point is
`core/src/transport/swarm.rs`, which **rule 8 gates** -- see "Why nothing was
edited" at the bottom. Everything below is from logs pulled this session.

Evidence corpus: phone `files/logs/scmessenger-mesh.log` (213 MB, JSON-lines, UTC,
retained since 2026-09-16) sliced to 17:25-17:40 UTC, phone
`files/history.db/db` (524,287 bytes) and the Windows node's
`scm.log.2026-09-19-17`. Slice at `tmp/pixel_3node_20260919/`.

## The question put to this pass

Two candidates. Which is the real defect?

- **(a)** No timer-driven retry: nothing flushes a queued message while its peer is
  backed off or dead, so delivery waits for a reconnect event or the revive window.
- **(b)** A flush keyed to the wrong peer: the blackout's reconnect events fired for
  peer `69805e17...` while the queued messages were addressed to `30d0fa67...`.

## Verdict: (a) is the defect; (b) is the same absence seen from the other side

**Both observations are real, and they share one root cause: the outbox has exactly
one drain path, and it is per-peer and event-driven. There is no peer-agnostic,
time-driven retry anywhere in the repository.**

`flush_peer_messages(recipient_id)` (`core/src/store/outbox.rs:525`) drains **only
the queue for the id passed in**, and it does honour `next_retry_at` -- so
due-ness is modelled, it is simply never consulted on a timer. The **only**
production caller is the connect-event path:

```
core/src/iron_core.rs:3225   let messages = self.outbox.write().flush_peer_messages(peer_id);
```

...inside `handle_peer_connection_event_with_egress` (`:3196`), which is reached
only from `handle_peer_connection_event` (`:3090`). A grep for any peer-agnostic
drain (`drain_due`, `flush_due`, `due_messages`, `retry_due`) returns **nothing**.

So a queued message is delivered only if a connect event arrives **carrying the
same id the message is keyed under**. If that never happens -- because the event
carries a different id, a different id *flavour*, or no event fires at all -- the
message waits. That is defect (a); (b) is one of the ways (a) manifests.

## The phone log, exactly

```
17:30:13.992 .. 17:31:37.574   outbox_reconnect_detected  peer=69805e175cdc...   (x7)
17:30:23.181                   outbox_enqueue f2ddf411  queued_at=1789839023180   <- operator's 07:30 send
17:33:43.506                   outbox_enqueue b96826fe  queued_at=1789839223505   <- operator's 07:33 send
                                ... no outbox event at all for 4.5 minutes ...
17:38:09.829                   outbox_reconnect_detected  peer=30d0fa678c21...
17:38:09.829                   outbox_flush_started       pending_count=2
17:38:09.831                   outbox_flush_completed     failed=0 succeeded=2
17:38:09.908                   [OK] Message delivered successfully ... (37ms)
17:38:50.664                   outbox_reconnect_detected  peer=69805e175cdc...   (x1 more)
```

Seven reconnect events arrived for `69805e17...` **while both messages were sitting
in the queue**, and **not one produced an `outbox_flush_started`** -- because
`flush_peer_messages("69805e17...")` found no messages for that id and returned at
the empty path (`iron_core.rs:3226-3235`, a DEBUG `outbox_flush_completed
pending_count=0`). The messages were keyed under `30d0fa67...`. The flush fired the
instant that id finally connected, and delivery then took 37 ms -- i.e. the
transport was never the problem.

`69805e17...` is not a benign alias: the Windows node logs
`GHOST-IDENTITY-001 skip auto-subscribe ghost peer topic:
/scmessenger/peer/69805e175cdc.../v1`, and an open ticket already tracks this class
(`HANDOFF/todo/P1_GHOST_GUARD_OWN_TOPIC_MESSAGE_LOSS_2026-09-16.md`).

## This class is already on record, found independently on the CLI side

`HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md:21`:

> In `cli/src/main.rs:4623`, offline messages are queued under canonical 64-hex
> public keys (`contact.peer_id`). In `main.rs:3746`, `flush_outbox_for_peer` calls
> `ob.drain_for_peer(&peer_id.to_string())` with base58 `12D3KooW...`. The Sled
> prefix `queue:12D3KooW..._` never matches `queue:30dce2bb..._`. Messages are
> stranded forever in the persistent outbox (explaining the 159 undelivered
> backlog).

Different platform, same shape: the drain key and the enqueue key disagree, and
because the drain is only ever triggered by a connect event, the failure is silent
and open-ended. The phone case is the identity-mismatch variant, not the flavour
variant, but both are cured by the same change.

## Proposed change (for review, not applied)

Three parts; **only the third is gated**, which is why this is a note rather than a
patch.

1. `core/src/store/outbox.rs` (**not gated**) -- add a peer-agnostic collector
   beside `flush_peer_messages`, reusing the same `is_due` closure that method
   already defines:

   ```rust
   /// Every message across all recipients whose `next_retry_at` has elapsed.
   /// Peer-agnostic: the timer path must not depend on a connect event.
   pub fn drain_due_all(&mut self) -> Vec<(String, QueuedMessage)> { .. }
   ```

2. `core/src/iron_core.rs` (**not gated**) -- a public drain that groups by
   recipient and dispatches through the *existing* egress closure, so it reuses
   `handle_peer_connection_event_with_egress`'s success/failure bookkeeping
   (`attempts`, `next_retry_at` backoff) instead of inventing a second sender:

   ```rust
   pub fn flush_due_outbox(&self, egress: &mut dyn FnMut(&str, &[u8]) -> bool) -> usize
   ```

3. **Wiring -- `core/src/transport/swarm.rs`, which rule 8 gates.** All periodic
   ticks in this repository live there (`:4274` pending-dial sweep 5 s, `:4283`
   custody retention 300 s, `:4358` routing optimisation 30 s, ...), and
   `cli/src/main.rs`'s tickers are CLI-only, which would leave the operator's
   affected platform (Android) unfixed. The natural host is the existing 5 s
   pending-dial sweep or a sibling interval calling `flush_due_outbox`.

Do **not** wire it into `IronCore::perform_maintenance`: `iron_core.rs:3144` records
that it "has zero native callers", so a drain placed there would never run -- a
wired-looking change that fixes nothing (rule 16).

### Why parts 1 and 2 were not committed on their own

An API with no caller is dead code, and committing only the non-gated halves would
produce exactly the "present, compiles, does nothing" shape rule 16 exists to
prevent. The three parts must land together, and the third needs adversarial
review.

## Regression guard the fix must carry

On `drain_due_all` (pure `core/src/store/outbox.rs` unit test, no native lib):

- enqueue a message for `recipient_a` with `next_retry_at` in the past and **never
  fire a connect event for `recipient_a`**; assert it is returned -- this is the
  operator's failure, and it fails today;
- enqueue for `recipient_b` with `next_retry_at` in the future; assert it is **not**
  returned (due-ness still respected);
- enqueue two messages for two recipients where only one is due; assert the result
  contains exactly the due one and that grouping preserves each recipient id, so
  the egress closure is never handed a message under the wrong key.

## Separate integrity question: `faa231f6`

The third arrival at the Windows node (`inbox_receive faa231f6-...` at
17:38:09.402, `sender_id` = the phone's identity) is **not** phone-side persistence
loss, and **not** a duplicate artefact. Evidence:

| check | result |
|---|---|
| phone history store, dashed and undashed forms | **0 occurrences**, while the recipient's hex id appears 829 times |
| phone outbox activity for it | **none** -- no `outbox_enqueue`, no `outbox_egress_dispatched`; the flush that second had `pending_count=2` |
| phone log mentions it at all | exactly **1**, a `routing_decision` at 17:38:10.246 (`StoreAndCarry`, `confidence=0.0`, `recipient_hint=7682f17a521ae243`) |
| Windows mentions it | exactly **1** -- the `inbox_receive`; no relay request, no custody accept, no ACK |
| is it one of the traced messages? | no -- distinct id **and** a different recipient hint |

Persistence loss would still leave an enqueue and a dispatch in the phone log, and a
duplicate would carry one of the two known ids. The picture is a single message
carrying the phone's identity as sender, entering the phone's routing pipeline at
the same instant (17:38:10.246 = 0.84 s after the Windows receive, matching the
measured phone-ahead skew) with an unresolved hint, and reaching Windows once.
`7682f17a521ae243` appears 9 times in the window and resolves to nothing.

**Open, and honestly not closeable from these two logs:** whether that message
originated at the cloud/relay node (its log for 17:38:09 would settle it), or is a
system envelope (identity/ledger/receipt) rather than a chat message -- the phone's
own store has been recreated before (`files/fresh_store_discarded_1423Z`,
`files/backup_corrupted_1789548764942`), so an older-generation record is also
possible. Stated as unresolved rather than guessed.

## Why nothing was edited

- The owner of the missing retry is the transport tick loop,
  `core/src/transport/swarm.rs` -- **rule 8 gated**: changes there are not "done"
  without an adversarial security review on file, and this lane cannot satisfy that
  gate itself. Editing it unilaterally would also undo a deliberate DoS bound: the
  dead/backoff state this interacts with lives in
  `core/src/transport/dial_policy.rs`, whose revive window is intentional.
- The non-gated halves alone would be dead code (see above).
