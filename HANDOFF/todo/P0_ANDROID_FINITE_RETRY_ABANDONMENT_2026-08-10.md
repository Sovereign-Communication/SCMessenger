# P0 -- Android abandons accepted undelivered messages (PF-1 / PF-12)

Status: Active -- DISPOSITIONED 2026-08-24: ACCEPTED FOR rc.1, remains v0.5.0-blocking (see disposition below)
Severity: P0 (pre-freeze blocker; violates the durable-delivery philosophy)
Filed: 2026-08-10
Gate mapping: **PF-1** finite-attempt abandonment, **PF-12** accepted-work
capacity semantics, **G3** delivery truth
Authority: `HANDOFF/plans/PR139_FIVE_NODE_FIELD_GATE_REFERENCE.md` Sections 3,
3.3, 4
Anchor observed: `68fcc3f1` (installed APK, versionCode 14, Pixel 6a)

## DISPOSITION 2026-08-24 (interim CTO, verify-against-main pass; operator may veto)

Re-verified against main `ceabdbd4`: Path 1 (expiry discard) is MITIGATED
(`pendingOutboxExpiryReason` returns null, "Messages NEVER expire",
MeshRepository.kt:7504). Paths 2/3/4 remain as filed: acked-without-receipt
7-day age removal (:7083), frozen now+120 reschedule loop (:7091), and
max-attempts -> `markMessageCorrupted` + removal (:7109-7119). Note:
MeshRepositoryTest.kt:248-290 and ReceiptWindowTest.kt currently ASSERT the
ceilings as intended behavior -- the fix must reconcile those tests, so this
is NOT a mechanical pre-tag fix.

ACCEPTED FOR v0.4.0-rc.1 with conditions:
1. Gate-day exposure is bounded: all four gate nodes are FRESH installs
   (empty outboxes) and matrix passes are hours, not days; the 7-day ceiling
   cannot fire. The only reachable path mid-gate is max-attempts corruption
   under D6 failover churn.
2. Scoring rule: a message observed in `corrupted` state during any matrix leg
   INVALIDATES that leg (same force as the G5 liveness fingerprints) -- it is
   scored, never silently absorbed.
3. This ticket stays OPEN and is v0.5.0-BLOCKING. Fix order when picked up:
   delete the `markMessageCorrupted` call in the max-attempts path first
   (acceptance-criterion violation), then reconcile the age-ceiling tests
   against criterion 1 under an operator ruling.

## Why this is a freeze blocker

The field-gate reference, Section 3.3 "What is explicitly wrong", names these
exact behaviours:

- "A static `attempt #12 -> Failed -> next_retry_at=None` rule for
  transient/unreachable delivery."
- "A `3 failed transport attempts -> permanent failure` interpretation when the
  peer may simply be offline."
- "Dropping an accepted message because it has been offline too long."

All three exist in `MeshRepository.flushPendingOutbox()` today.

## Field evidence (current build)

Window 2026-08-10T02:00Z -> 15:13Z on the Pixel 6a:

- ~40 distinct message ids pinned at
  `delivery_state state=held detail=acked_without_receipt_protection
  acked_count=1`, several at `attempt=10`.
- **Every** outbox flush in the window reports
  `Outbox flush complete: 0 queued for transport, N scheduled for retry`
  with `succeeded:0`.
- The retry loop writes `delivery_state` so densely that
  `files/mesh_diagnostics.log` rotates ~100 KB/min; four rotated files covered
  only three minutes of wall clock, destroying all other diagnostics
  (this is also PF-11 / field-gate Section 6.3 "bound verbose transport loops").

## The four terminal paths (all in `flushPendingOutbox`)

File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

| # | Location | Behaviour | Violation |
|---|---|---|---|
| 1 | `pendingOutboxExpiryReason(item, now)` (helper at :7457) | `iterator.remove()`, logs `dropped_pending_outbox` | PF-12 silent discard |
| 2 | `shouldStopAckedWithoutReceiptRetries(...)` (helper at :102), ceiling `pendingOutboxMaxAgeSeconds` = 7 days (:392) | `iterator.remove()`, logs `stopped_pending_outbox reason=max_age_exceeded_acked_without_receipt` | PF-1 + PF-12 |
| 3 | `if (item.ackedWithoutReceiptCount > 0)` | never attempts delivery again; reschedules `now + 120` forever until path 2 removes it | PF-1 (obligation stalls) |
| 4 | `if (item.attemptCount >= pendingOutboxMaxAttempts)` | **`markMessageCorrupted(...)`** + `iterator.remove()` | PF-1, and it misreports an undelivered message as CORRUPTED |

Path 4 is the most damaging: an unreachable recipient produces a *corruption*
claim to the user. Corruption and non-delivery are different truths.

## Required semantics (field-gate reference Section 3.1)

Three lifecycle concepts must stop sharing one overloaded retry counter:

| Concept | Required behaviour |
|---|---|
| History record | Durable after acceptance |
| Delivery obligation | **Indefinite** until confirmed delivered, user/policy cancelled, or genuinely irreversible protocol rejection |
| Network attempt | Finite and adaptive; bounded backoff, jitter; lifetime attempt count NOT capped |

Opportunistic retry triggers to honour (Section 3.2): peer reconnected, new
viable address learned, healthy relay path available, custody state change,
network interface transition, app wake/reconnect reconciliation, and backoff
expiry only when a plausible route exists.

## Acceptance criteria

1. No code path removes or drops an accepted, undelivered, non-cancelled
   message from the pending outbox because of attempt count or age.
2. `markMessageCorrupted` is never reached via a delivery-failure path;
   corruption is reserved for genuinely undecodable payloads.
3. A transport-acked-without-receipt message continues to be retried on a
   bounded, jittered backoff rather than being frozen.
4. Retry attempts remain rate-limited: verify the log volume from one pending
   message over 10 minutes is bounded and does not rotate the diagnostics log.
5. Persistence: obligations survive process restart (unit + persistence tests).
6. Capacity pressure produces explicit backpressure/rejection **before**
   acceptance, never silent post-acceptance loss (PF-12).
7. `cd android && ./gradlew assembleDebug -x lint --quiet` compiles; existing
   `MeshRepository` unit tests updated rather than deleted.

## Ordering constraint

The core-side receipt convergence work (PF-2, `mark_message_sent`,
`core/src/iron_core.rs`) is owned by the Windows lane and is NOT in scope here.
This ticket must not change core receipt handling. Android must behave
correctly whether or not a receipt ever arrives.
