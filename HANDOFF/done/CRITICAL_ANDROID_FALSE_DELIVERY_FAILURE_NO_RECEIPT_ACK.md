# TASK [CRITICAL]: Android reports message delivery failure despite successful delivery (missing receipt/ack)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: PARTIAL - steps 1-2 (core receipt classification + CLI serde_json fix)
LANDED 2026-07-13. Receipt round-trip now works core-side: incoming Receipt
messages are classified (after the blocked-peer check, with a fall-through to
the normal pipeline and explicit parse-failure logging - fixed after triangulated
review found the first attempt skipped the blocked check and silently swallowed
parse errors), and the CLI's bincode/serde_json mismatch is fixed to serde_json
consistently at construction (message/types.rs) and both consumption sites.
Step 3 (Android Kotlin-side retry suppression - never escalate transport-success
to failed/corrupted, widen receipt window, regression test) is STILL OPEN - this
is the FARM WS-A3 remainder. Do not close this ticket until step 3 lands.

Found 2026-07-12 verifying the user's live manual test (sent "Hi"
from the Android emulator, contact nickname "Emmy", to the Windows CLI).

## What was proven (live device evidence, not speculation)

The message DID arrive. Confirmed via direct inspection of the Windows CLI's
own persisted store (`%LOCALAPPDATA%\scmessenger\storage\db`):

```json
{"id":"3962eb30-7005-4b39-86cb-3c59775d1fb8","direction":"Received",
 "peer_id":"1e81494d41ba7d23eee9e2513d5700778755ce463938c3375bcd42bccb346cc0",
 "content":"{...\"kind\":\"text\",\"text\":\"Hi\",\"sender\":{...
 \"public_key\":\"f583919e981275d4a9ca669987592d88fb03519630dad7068969f7b1dd651cd4\",
 \"nickname\":\"Emmy\",
 \"libp2p_peer_id\":\"12D3KooWSLkR1yNngFGG7mheNM4wbQYRRo4D9599Rwga1gvcVfY7\"...}"}
```

But the Android app's own tracking says the opposite. From
`mesh_diagnostics.log` (rotated copies `.4`/`.5`), traced by message ID
`3962eb30-7005-4b39-86cb-3c59775d1fb8`:

```
11:04:24.106 delivery_attempt medium=tcp_mdns phase=smart_router outcome=success route=12D3KooWD6vZQ... lan_addrs=28
11:04:24.166 delivery_state state=stored detail=awaiting_receipt_delay_sec=30
11:04:56.299 delivery_attempt medium=tcp_mdns phase=smart_router outcome=success route=12D3KooWD6vZQ...
11:04:56.359 delivery_state state=stored detail=awaiting_receipt_delay_sec=30
  (30s later, no receipt arrived -> treated as still-pending, retried again)
...
11:07:43.433 W/Mesh: Dropping message 3962eb30... after 12 attempts (max=12)
11:07:43.434 W/Mesh: Message 3962eb30... marked as corrupted
11:07:43.435 delivery_state state=failed detail=dropped_pending_outbox reason=max_attempts_exceeded
```

The underlying transport send genuinely succeeded at least twice (real
sub-30ms latencies, both `tcp_mdns` and `core/direct` mediums independently
confirmed `outcome=success`). Each time, the app entered a
`state=stored detail=awaiting_receipt_delay_sec=30` wait for a delivery
receipt/ack that never came back, so it re-queued the "still pending" message
and tried again -- eventually exhausting `MAX_RETRY_ATTEMPTS=12`
(`MeshRepository.kt:605`), permanently dropping a message that had already
been delivered, and flagging its tracking entry `corruptionDetected=true`
(surfaced later as "Corrupted message tracking detected... recovering" on the
next app boot, `MeshRepository.kt:700-723`).

## The bug

Two related gaps, both in the Android/Kotlin-side `SmartTransportRouter` /
`outbox_retry` logic (`MeshRepository.kt`), NOT in the Rust core's routing
engine (that's the separate, already-filed
`CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md`):

1. **No working receipt/ack path.** After a transport-layer `outcome=success`,
   the code waits `awaiting_receipt_delay_sec=30` for confirmation but nothing
   ever satisfies that wait -- either no ack message type exists on the wire,
   or the receiving side (CLI) never sends one, or the sending side never
   correctly correlates one back to this message ID. Needs tracing on both
   ends: does the CLI (or core) send any kind of delivery receipt at all
   today? Grep core/src for a receipt/ack message type; if none exists, this
   needs to be designed and built, not just wired.
2. **A successful send is retried anyway**, because "success" at the
   transport layer is not treated as terminal -- the retry loop only seems to
   key off the receipt, which never arrives, so an already-delivered message
   keeps getting re-attempted until it hits the retry ceiling and is
   permanently dropped and marked corrupted. At minimum, a confirmed
   transport-layer `outcome=success` should suppress further retries for that
   message (optimistically treat as delivered, or hold in a separate
   "delivered-unconfirmed" state) instead of continuing the full retry/backoff
   cycle toward eventual deletion.

## Why this matters more than it looks

From the user's perspective this is worse than an honest failure: the
message actually arrived, but the sender's own app tells them it didn't (and
after ~3.5 minutes, silently deletes the retry record entirely). A user who
trusts the Android UI's failure indication would reasonably re-send, assume
the recipient never got it, or lose confidence in the app -- when the
recipient already has it. This was only caught because the operator happened
to check the CLI's raw storage directly.

## Live reproduction, same session (2026-07-12, after Site 2 fix rebuild)

While re-verifying the separate Site 2 routing fix, restarting the CLI
daemon caused a backlog "Hi 2" message from the same Android contact to be
delivered instantly and repeatedly -- the exact same message content arrived
4+ times within seconds (`ROUTE_DECISION ... [OK] Message delivered
successfully ... (5-11ms)` fired again every ~2.5s), live-confirming the
retry-without-suppression behavior described above: Android keeps re-sending
an already-delivered message because it never sees a receipt.

**Ruled out one candidate mechanism:** `core/src/transport/swarm.rs`'s
`DELIVERY_CONVERGENCE_TOPIC` ("sc-receipt-convergence" gossipsub topic,
`swarm.rs:167`, handler at `swarm.rs:3360`/`5399`) is NOT the missing
application-level receipt -- traced its handler and it only deduplicates
*relay-hop* custody/pending-request bookkeeping between core nodes
(`pending_messages`, `pending_relay_requests`, `pending_custody_dispatches`,
`relay_custody_store`). It has no path back up to the Android FFI layer or
into `MeshRepository.kt`'s `awaiting_receipt_delay_sec=30` wait. Whatever
receipt Android's retry loop is actually waiting for is either a different,
not-yet-located mechanism, or genuinely does not exist end-to-end yet.

## ROOT CAUSE FOUND (2026-07-12, definitive, traced end to end)

The receipt mechanism is not missing conceptually -- Android's
`sendDeliveryReceiptAsync`/`onReceiptReceived` (`MeshRepository.kt:2118,
2255`) and the core's `prepare_receipt`/`on_receipt_received` FFI trait
(`iron_core.rs:1567`, `api.udl:108`) all genuinely exist and are real,
working infrastructure on the Android side. The break is that **nothing in
the shared Rust core ever recognizes an incoming payload as a receipt and
fires that callback** -- confirmed by grepping all of `core/src/` for
`Receipt`: it appears in exactly two places, `iron_core.rs:1572`
(construction/JSON-serialize in `prepare_receipt`) and the `lib.rs`
re-export. There is no classification logic anywhere in
`core/src/transport/swarm.rs` or `core/src/mobile_bridge.rs` that inspects
an incoming message and routes it to `on_receipt_received` -- that trait
method is defined and forwarded (`mobile_bridge.rs:1907-1910`) but never
actually invoked by anything. It is dead-on-arrival infrastructure, the same
shape of bug as the outbox's unused `record_attempt` found earlier this
session.

The ONLY code anywhere that attempts to recognize an incoming receipt lives
in the CLI's own client code, not the shared core:
`cli/src/main.rs:1913-1925`, a `MessageType::Receipt` match arm that calls
`bincode::deserialize::<scmessenger_core::Receipt>(&msg.payload)`. But
`prepare_receipt` (`iron_core.rs:1581`) builds the payload with
`serde_json::to_vec(&receipt)` -- **JSON, not bincode.** Deserializing a JSON
byte string as bincode will fail (`Err`, silently swallowed by the `if let
Ok(...)` with no `else`), so even the CLI's own hand-rolled receipt
recognition is broken by a serialization-format mismatch with the very
function that produces the payload it's trying to read.

Net effect: a receipt genuinely cannot complete its round trip today, on
either end, for two independent reasons stacked on top of each other:
1. Shared core (affects Android specifically, since Android has no other
   receipt-recognition code path): no incoming-message classification
   exists to ever call `on_receipt_received` -- so `onReceiptReceived` in
   Kotlin can never fire no matter what the CLI sends.
2. CLI's own ad-hoc classification (which the shared core lacks): even if it
   were reachable, its bincode deserialization would fail against the
   JSON payload `prepare_receipt` actually produces.

This is NOT the `swarm.rs` routing bug (Site 2, already fixed and verified
this session) -- confirmed live: the receipt payload itself was proven to
physically transmit successfully (`[OK] Message delivered successfully to
12D3KooW... (9-12ms)` immediately following `Sending delivery ACK for
<id> to <peer>` in the CLI's own log, 2026-07-12 21:47:20). The bytes
arrive; nothing on the receiving end (whichever side receives it) is wired
to recognize them as a receipt.

## What's left to do

1. Add message classification in the shared core's incoming-message path
   (`core/src/transport/swarm.rs`'s request-response `Message::Request`
   handler is the natural place, mirroring how `RelayMessage::from_bytes` is
   already tried there for relay discovery) that attempts
   `serde_json::from_slice::<crate::Receipt>(&data)` and, on success, invokes
   the `on_receipt_received` delegate callback instead of treating it as a
   normal text message.
2. Fix `cli/src/main.rs:1915`'s `bincode::deserialize` to `serde_json::from_slice`
   to match what `prepare_receipt` actually produces (or standardize both
   producer and consumer on one format explicitly -- either is fine, they
   just need to agree).
3. Once wired, the retry loop hardening from the original writeup still
   applies as defense-in-depth: a confirmed transport-layer send success
   should not be retried indefinitely just because the (now-fixed) receipt
   hasn't arrived yet within the 30s window -- widen the window or treat
   "sent successfully" as a distinct, less alarming state than "definitely
   failed" while waiting.
4. `DELIVERY_CONVERGENCE_TOPIC` remains confirmed NOT relevant (see above);
   don't re-investigate it.
2. If an ack mechanism exists but isn't wired to this Android retry path,
   find where it should terminate the retry loop and wire it.
3. Regardless of ack fix, make the retry loop itself safer: a confirmed
   transport-layer `outcome=success` should not continue toward
   `max_attempts_exceeded`/drop -- distinguish "we know we sent the bytes"
   from "we have no idea," and don't destroy tracking for the former.
4. Add a regression test for the retry/receipt state machine (Kotlin unit
   test on `MeshRepository`'s message tracking, mocking a transport success
   with no receipt) so this doesn't silently regress.
5. Cross-reference with `CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md`
   -- that ticket's Site 2 fix is about the Windows CLI as SENDER; this ticket
   is about Android as SENDER. Both share the same underlying Rust
   `send_message`/`route_message_optimized` code path in principle, but this
   bug reproduces at the Kotlin retry-orchestration layer sitting above it,
   so fixing one does not fix the other.

## Gate

Not a crypto/transport Rust change by itself (the fix likely lands in
Kotlin + possibly a small protocol addition for receipts) -- if a wire
protocol/envelope change is needed, mandatory adversarial review applies to
that piece per `.claude/rules/security.md`. Verification: repeat the live
Android-to-CLI send, confirm the Android UI's own message status reflects
successful delivery (not silently drops it) within the app's own history/UI,
not just the recipient's raw store.

## Step 3 implementation spec (2026-07-14, ready to dispatch)

Steps 1-2 (core receipt classification + CLI serde_json fix) are DONE. This
is the concrete spec for the remaining Kotlin-side retry-suppression work in
`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`.

Confirmed by direct code read: the queue item type is
`internal data class PendingOutboundEnvelope` (line 470-484: `queueId,
historyRecordId, peerId, routePeerId, listeners, envelopeBase64,
createdAtEpochSec, attemptCount, nextAttemptAtEpochSec, strictBleOnlyMode,
recipientIdentityId, intendedDeviceId, terminalFailureCode` - NOT
"PendingOutboxItem", that name does not exist in this file, do not search
for it). `flushPendingOutbox()` (~line 6526) is the retry loop. On
`delivery.acked == true` (transport genuinely succeeded, ~line 6651), it
currently still increments the SAME `item.attemptCount` used by the
max-attempts check at ~line 6558 (`item.attemptCount >=
pendingOutboxMaxAttempts` where `pendingOutboxMaxAttempts = 12`, ~line 379)
that drops the message and calls `markMessageCorrupted()` (~line 6560). This
is the exact bug: a message that transport-delivers successfully every
single retry, but never sees a receipt (the original root cause, now fixed
at the core level, but this loop has no defense-in-depth against a slow/lost
receipt), still gets dropped and flagged corrupted once `attemptCount`
crosses 12 - regardless of how many of those attempts were confirmed sends.

ALSO NOTE a pre-existing inconsistency, flagged for awareness (not
necessarily in scope to resolve): `pendingOutboxExpiryReason()` (~line 6922,
called at ~line 6543) is documented "PHILOSOPHY: Messages NEVER expire.
Every message retries until successfully delivered. No attempt limit, no age
limit." and its body always returns `null` - directly contradicting the very
real `attemptCount >= pendingOutboxMaxAttempts` hard-drop 20 lines later at
~line 6558. Whoever wrote that comment believed there was no attempt
ceiling; there is one, and it is the bug this ticket is about. Do not "fix"
this by making `pendingOutboxExpiryReason` consistent with the drop logic
(that would remove ALL retry limits including for genuine failures) -
the fix is to stop confirmed-successful sends from counting toward that
ceiling at all, per the fields below.

CONFIRMED (traced both call graphs, not guessing): there is a second,
independent tracking system, `MessageTracking` (lines 555-626, its own
`attemptCount`/`corruptionDetected`/`MAX_RETRY_ATTEMPTS=12`), incremented
only via `incrementAttemptCount()` (line 728) -> `recordFailure(null)`
(line 731). `incrementAttemptCount` is called from `flushPendingOutbox`
ONLY in the genuine-transport-failure branch (~line 6680, after the
`delivery.acked` block already `continue`s away at line 6677) - it is
NEVER called when `delivery.acked == true`, so this second system does
NOT double-penalize confirmed sends and does NOT need the same fix.
`MessageTracking.recordSuccess()` (line 584, resets attemptCount to 0) has
ZERO call sites anywhere in the file - dead code, leave it alone, not in
scope here. The two systems connect at exactly ONE point:
`markMessageCorrupted()` (line 676) is called from `flushPendingOutbox`'s
max-attempts branch (line 6560) and sets `MessageTracking.corruptionDetected
= true` directly via `tracking.markCorrupted()` - this is what produces the
ticket's observed "Corrupted message tracking detected... recovering" log,
sourced entirely from the `PendingOutboxItem` ceiling, not from
`MessageTracking`'s own counter. Fixing the `PendingOutboxItem`/
`pendingOutboxMaxAttempts` bug below (and not calling `markMessageCorrupted`
for an acked-without-receipt drop) is sufficient to fix both observed
symptoms. Do not touch `MessageTracking`/`incrementAttemptCount`/
`shouldRetryMessage` - they are out of scope and already correct.

**Required fix:**
1. Add a field to `PendingOutboundEnvelope` (e.g. `ackedWithoutReceiptCount:
   Int = 0`) that counts retries which occurred AFTER a confirmed transport ack,
   separate from `attemptCount` (which should track only genuine
   transport-level send failures/non-acks).
2. In the `delivery.acked == true` branch (~line 6651-6677): increment
   `ackedWithoutReceiptCount` instead of `attemptCount`; keep using it (not
   `attemptCount`) for the adaptive receipt-wait backoff tiers. Do NOT let
   this counter feed the `pendingOutboxMaxAttempts` check at ~line 6558.
3. Give the acked-without-receipt path its own, much more patient ceiling
   (e.g. a wall-clock-based give-up, not a small attempt count) and, on
   exceeding it, do NOT call `markMessageCorrupted()` - log a distinct
   lower-severity state (e.g. `state=delivered_unconfirmed`) and stop
   retrying without flagging the message-tracking UI's corruption-recovery
   path. Transport-confirmed-success must never look like corruption to the
   user.
4. Leave the existing `attemptCount`/`pendingOutboxMaxAttempts`/
   `markMessageCorrupted()` behavior UNCHANGED for the genuine
   transport-failure branch (~line 6679+) - that escalation is legitimate.
5. Add a Kotlin unit test exercising `flushPendingOutbox` (or the smallest
   testable unit around it) that mocks a transport send returning
   `acked=true` repeatedly with no receipt ever recorded, and asserts the
   message is NOT dropped or marked corrupted even after many such cycles.

Verify with: `cd android && ./gradlew assembleDebug -x lint --quiet`, then
(once the new test's class/method name is known) the specific
`./gradlew :app:testDebugUnitTest --tests "<new test>"` target.
