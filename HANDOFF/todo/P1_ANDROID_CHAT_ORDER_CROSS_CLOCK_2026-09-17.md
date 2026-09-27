# P1 - Android chat renders out of order: the sort key is a REMOTE clock

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Open - diagnosed from the device's own store, fix not yet implemented
Priority: P1 (operator-visible correctness defect: a reply appears BEFORE the
message that caused it, so the conversation reads as causally impossible)
Filed: 2026-09-17 by the Freebuff lane, CTO seat
Affects: Android UI ordering only. No Rust change implied by the diagnosis.
Author: Freebuff lane (this seat)

## Symptom (operator report, 2026-09-17)

"somehow the auto-reply shows as if it came in BEFORE i sent my message, which
makes no logical sense" - a message the operator sent appears BELOW the reply
it triggered.

## Root cause

The chat list is ordered by `senderTimestamp`. That field is NOT one clock:

- **outbound** records get the PHONE's clock at send time
  (`MeshRepository.kt:5569` `val now = (System.currentTimeMillis() / 1000)`;
  written at `:5795` and `:5901`; `ChatViewModel.kt:195` does the same),
- **inbound** records get the SENDER's clock, straight off the wire
  (`MeshRepository.kt:2499` `senderTimestamp = senderTimestamp`, fed by the core
  at `iron_core.rs:3848` `delegate.on_message_received(..., message.timestamp, ...)`,
  which is the decrypted envelope's own stamp).

So the UI compares the Pixel's clock against the node's clock and sorts the
union by that comparison. Both sides are truncated to whole seconds.

Sort sites (all read this session):
`ChatScreen.kt:82`, `ChatViewModel.kt:129/204/340`,
`ConversationsViewModel.kt:40-42` - every one of them
`sortedBy { it.senderTimestamp }` / `sortedByDescending { it.senderTimestamp }`.

## Measured proof (device ground truth, not inference)

The Pixel's own history store (`files/history.db/db`, a sled log, read with the
operator-authorised `run-as` read) holds both rows:

    Received  "[auto-reply] Node is on but unattended right now - I will read
              your message when I am back."
              timestamp = 1789679984  sender_timestamp = 1789679984   -> 21:19:44Z
    Sent      "baseline test"
              timestamp = 1789679985  sender_timestamp = 1789679985   -> 21:19:45Z

and the node that produced the reply logged it in the same window:

    Windows node log, 2026-09-17T21:19:44.907175Z:
      auto_reply_ack_queued in_reply_to=fdd6b22a-3c06-46f3-acc6-8466a8a49032
                            to=12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn

The reply is stamped **one second EARLIER** than the phone's own record of the
message that triggered it. That is only possible if the Pixel's clock runs about
1 second ahead of the Windows node's clock, and second-granularity stamps turn
that skew into a visible inversion. Ascending sort then renders the reply first.

Same-second collisions are common, not rare: **10 distinct seconds** in this
store contain both a Sent and a Received row, e.g. `1789614910` (09-17 03:15:10Z)
holds Sent "test" and a Received auto-reply in the same second, and `1789619914`
(09-17 04:38:34Z) likewise. In those cases the order is decided by insertion
order, not by time, because Kotlin's `sortedBy` is stable - so the rendering is
arbitrary rather than merely wrong.

## Latent second defect on the same line

For inbound records the code applies a local-clock fallback to `timestamp` but
**not** to the field the UI actually sorts by:

    MeshRepository.kt:2492   val canonicalTimestamp = if (senderTimestamp > 0uL) senderTimestamp else fallbackNow
    MeshRepository.kt:2497   timestamp = canonicalTimestamp,
    MeshRepository.kt:2499   senderTimestamp = senderTimestamp,      <- raw, no fallback

A peer that stamps 0 sorts to the very top of the conversation. The core models
the fallback correctly for its own store (`store/history.rs:35-36` and
`mobile_bridge.rs:3113-3114`: `if sender_timestamp == 0 { sender_timestamp = timestamp }`),
so this is the Android bridge not mirroring the core's normalisation.

## Fix direction (not yet implemented)

Order by something the PHONE assigns, never by a peer's clock:

1. Give every inbound record a locally-assigned ordering key at write time
   (local receive time, or a monotonic insertion sequence), and sort the merged
   conversation by that key.
2. Keep `senderTimestamp` as provenance/display data ("sent at" on the bubble,
   `MessageBubble.kt:70`), which is what it is good for.
3. Add a monotonic tiebreaker so a same-second Sent/Received pair has a
   deterministic order rather than an insertion-dependent one.
4. Mirror the core's zero-fallback for `senderTimestamp` at
   `MeshRepository.kt:2499` regardless, so a zero-stamped peer cannot jump the
   queue.

Open question the fix must answer before it lands: whether `timestamp` itself is
safe to repoint at the local receive time, because it also feeds the
`history_sync_data` payload (`MeshRepository.kt:3199` `obj.put("ts", ...)`) and
core-side history ordering. A separate ordering field avoids touching sync
semantics; reusing `timestamp` is smaller but wider-reaching.

## Evidence paths

- `tmp/pixel_3node_20260917/db/sled.db` (the phone's history store, pulled via
  `adb exec-out run-as com.scmessenger.android cat files/history.db/db`)
- `tmp/pixel_3node_20260917/pixel-mesh-tail.log` (300,000-byte app-log tail)
- Windows node: `%LOCALAPPDATA%/scmessenger/logs/scm.log.2026-09-17-21`

## Not claimed here

The same raw read shows every outbound "test" row appearing under two distinct
message ids. That is **not** reported as a defect: a sled log retains superseded
values until compaction, so a normal `delete` of the provisional record can
still be visible to a raw grep. Confirming real duplicates needs a read through
the store API (for example a node `history`-style dump), not this file scan.
