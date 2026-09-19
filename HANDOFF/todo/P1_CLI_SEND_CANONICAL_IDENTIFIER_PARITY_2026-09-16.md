# P1 - CLI `send` could not address any contact after the canonical-hex migration

Status: Fixed (pending CI on the push that carries it)
Priority: P1 (the CLI node's primary function - sending a message - reported
failure for every send; one of the three node types could not start a
conversation)
Filed: 2026-09-16 by the Freebuff lane, during a three-node functional proof
Affects: `cli/src/main.rs` only (no core change, no rule-8 surface)
Author: Freebuff lane (this seat)

## Symptom (reproduced three ways, same failure)

    scm send 30dce2bb...0645 "test"     -> outbox_enqueue OK, then
                                           Error: Invalid peer ID in contact: {}
    scm send 12D3KooWD776...            -> Error: Contact not found
    scm send "LucasFix1" "test"         -> outbox_enqueue OK, then
                                           Error: Invalid peer ID in contact: {}

Observed 2026-09-16T18:25-18:26Z against the live Windows node's store. The
first and third forms enqueue the encrypted envelope (payload 1088-1094 bytes,
`packet_lifecycle ... event="outbox_enqueue"`) and THEN fail, so the operator
sees a hard error for a message that was in fact accepted.

## Root cause (code + the live store, not inference)

`contact list` prints the canonical public-key hex as the contact's "Peer ID":

    • LucasFix1
      Peer ID: 30dce2bb779b4f1419f6d7d9e91b3ae201aed9e3b181aef674a9496f340a0645

and the store now canonicalises to that hex on write
(`contacts_canonical_hex_live from=12D3KooWD776... to=30dce2bb...`, logged live
on both the Windows and AWS nodes). But the send path still parsed that field as
a base58 libp2p peer id:

    // cli/src/main.rs (before)
    let recipient_peer_id = contact
        .peer_id
        .parse::<libp2p::PeerId>()
        .context("Invalid peer ID in contact: {}")?;

hex `30dce2bb...` fails base58 at byte 1 (`'0'` is not in the base58 alphabet),
which is the exact error string above. The `{}` with no argument is a second,
cosmetic defect: the error path had no value to interpolate, consistent with it
never having been exercised.

Two further send paths carried the identical three-line pattern and the same
breakage: `server::UiCommand::Send` (`main.rs` ~3074) and
`ClientIntent::SendMessage` (`main.rs` ~3268), whose fallback was
`contact.peer_id.parse().ok()`.

## Fix (this commit)

One resolver, used by all three sites:

    fn peer_id_from_contact_identifier(identifier: &str) -> Option<PeerId> {
        // canonical public-key hex -> PeerId, legacy base58 still accepted
        ...scmessenger_core::store::ledger_entry::peer_id_from_public_key_hex...
    }

The CLI already used that core helper for sender resolution
(`resolve_sender_peer_id`, `main.rs` ~3390), so this reuses the existing
conversion rather than adding a second one. The `{}` context also interpolates
the identifier now.

Verified on the live node store after the fix (release binary, 2026-09-16T18:49Z):

    [OK] Message encrypted: 1091 bytes
    [OK] Sending message to 12D3KooWD776DQdWh6iHV8Qcnpj9jvTXhpJAgSsFPbMCaRtQpFmn...

i.e. the recipient resolves and the send proceeds. That run then hit an
unrelated transient sled lock (`database locked by another process`) while a
`cargo build` held the store directory; a later run against a free store
produced the same two lines with no error.

## Still open (recorded, not fixed here)

1. `send <base58>` returns `Contact not found`: `find_contact` matches
   `peer_id` (hex), nickname, or `public_key`, so a legacy base58 peer id is no
   longer a valid address form. Canonical hex is the intended identifier and is
   now accepted, so this is left as-is pending an operator call on whether
   legacy base58 lookups should resolve too.
2. The three `send`-created messages enqueued before the fix
   (`a853610c...`, `dd3936a2...`, `1ee220c7...`) are queued but were not
   observed delivering; the store reports `Undelivered: 159` (history-stats),
   so outbox draining is a separate backlog question.
3. `send` writes history only on the success path, so a message that is
   enqueued and then fails is invisible to `scm history`.

## Feature shipped alongside: custom auto-reply body

Operator request (2026-09-16): `--auto-reply` should accept a custom message so
an always-on, unwatched node can say something situational, defaulting to the
generic acknowledgement.

    --auto-reply                       -> generic body (unchanged behaviour)
    --auto-reply "text"                -> that text
    SCM_AUTO_REPLY=1 | true            -> generic body
    SCM_AUTO_REPLY="text"              -> that text
    (argv wins over the environment)

`resolve_auto_reply_body()` (unit-tested) trims operator text, keeps the generic
body for an empty/absent value, and always ensures the `[auto-reply] ` machine
marker - that prefix is what `should_send_auto_reply` uses to stop two responder
nodes from answering each other forever, so it is applied to custom text too
(and recognised rather than doubled when already present).

Verified end to end on hardware, 2026-09-16:

| Step | Evidence |
|---|---|
| argv -> body plumbing | banner: `body: [auto-reply] Node is on but unattended right now - I will read your message when I am back.` (19:06Z) |
| incoming phone message triggers it | node `auto_reply_ack_queued in_reply_to=b729bf67-... to=12D3KooWD776...` (19:07:38Z) |
| the ack actually reaches the phone | phone logcat `Message from 30d0fa67...: eaa2698d-5eef-4a91-9630-2e5a520cc0bb` then `onMessageReceived ... kind=text transport=INTERNET` and `ChatViewModel$loadMessages: Loaded 17 messages` (19:08:40Z) |
| the phone received and receipted it | node: `[OK][OK] Delivered: eaa2698d`, `Processed application delivery receipt ... message_id=eaa2698d-...` (19:08:41Z) |
| unit tests | `cargo test -p scmessenger-cli --bin scmessenger-cli auto_reply` -> 4 passed (2 new), 0 failed |

### Fixed: gratuitous auto-replies to machine envelopes (operator report)

Operator report 2026-09-16: "i got 6 auto replies despite me sending 0
messages". Confirmed and fixed in `8f885780`.

Evidence, all pulled this session:

    # every ack today, mapped to the message that triggered it (13 distinct)
    for f in "$LOCALAPPDATA/scmessenger/logs"/scm.log.2026-09-16-*; do
      grep -ah "auto_reply_ack_queued" "$f"; done | sort -u

    # the body of those triggering messages, straight from the node's store
    scm stop && scm history | grep -A2 "\[2026-09-16 08:28:42\]"
      <- f83ab163... [2026-09-16 08:28:42]
         {"schema":"scm.message.identity.v1","kind":"history_sync",
          "text":"","sender":{...}}

    # received entries carrying any non-empty text today: 0
    grep -a 'received' history | grep -c '"text":"[^"]'   -> 0

Root cause: the phone broadcasts an identity envelope about once a minute and
its `text` field is EMPTY. `should_send_auto_reply` only rejected bodies that
already began with the `[auto-reply] ` marker, so a machine envelope with an
empty body passed the guard and earned a courtesy reply - one per envelope,
indefinitely, for an operator who had sent nothing.

Fix: `is_answerable_text(incoming, envelope_kind)` gates the reply on the
envelope kind being chat (absent or `"text"`), the trimmed body being
non-empty, the marker still absent, and identity-envelope JSON rejected even
when it arrives undecoded. Five unit cases added; `cargo test -p
scmessenger-cli --bin scmessenger-cli auto_reply` -> 5 passed.

Live differential (same phone, same command line, 6 minutes each):

| build | inbound machine messages | acks |
|---|---|---|
| before the fix | 3 | **3** |
| after the fix | 6 | **0** |

with the banner still showing the armed custom body, so the suppression is the
new guard and not a disarmed feature. Two process notes worth keeping: the
first differential run was run against a stale `target/debug` binary
(`cargo test --bin` does not refresh it) and produced 3 acks from 3 messages -
the binary must be checked for the change (`grep -c
"auto_reply_skipped_machine_message" <bin>`) before any run is credited.

## Feature: auto-reply is capped at one acknowledgement per minute (operator directive)

Operator, 2026-09-16: "auto reply 1:1 but with a max of 1x auto reply per minute,
so it doesn't ever spam more than once per minute."

Implemented as `AUTO_REPLY_MIN_INTERVAL_SECS = 60` plus a pure predicate
`auto_reply_rate_limited(last_sent_at, now)`, evaluated in the responder's gate
order BEFORE the per-message dedup check:

```
answerable text -> rate limit -> per-message dedup -> queue reply
```

Rate-limit first is deliberate. Because the message id is recorded only when a
reply is actually queued, 1:1-per-message still holds exactly: a message can earn
at most one acknowledgement ever, and the node as a whole emits at most one per
minute. A suppressed message is NOT deferred, and only a reply that was really
queued consumes the window (`auto_reply_last_sent_at` is set in the successful
queue branch, not before the attempt).

Suppressions are observable in the log as `auto_reply_suppressed_rate_limit`
(separate from `auto_reply_suppressed_duplicate`), so "no reply" can always be
told apart from "reply throttled".

Unit proof: `cargo test --bin scmessenger-cli auto_reply` -> 6 passed, including
the new `auto_reply_is_capped_at_one_per_minute` (five-message burst yields
exactly one reply; window shut at 59s, open at 60s; a later distinct message is
answerable; a redelivery of an answered message is still a duplicate).

### Why the operator's Android test still saw uncapped replies (provenance)

That test hit a node running a binary built BEFORE this change:

- node PID 18024 started 10:31:36, binary `target/release/scmessenger-cli.exe`
  built 10:31:25 - `grep -c auto_reply_suppressed_rate_limit` on that binary = 0.
- Its log shows the uncapped behaviour the operator reported:
  `21:07:53.606, 21:07:56.455, 21:07:57.990, 21:07:59.698, 21:08:01.699,
  21:08:03.311, 21:08:04.738` - **seven `auto_reply_ack_queued` in 11 seconds**.
- Note the first attempt to build this change failed with `Access is denied.
  (os error 5)`: on Windows the running node holds the exe, so the link step
  cannot replace it. Stop the node, rebuild, then start - a build that "ran"
  while the node was up can silently leave the old binary in place.
- The node now runs PID 18124 on the binary built 11:19:39, which contains both
  the own-topic fix and the rate-limit fix (`grep -c` = 1 for each string).

### Live proof (Windows node PID 18124, binary built 11:19:39, armed)

Inbound chat was generated WITHOUT touching the Pixel, by sending through the
cloud node's HTTP API (`POST http://18.234.62.247:9876/api/send`), which delivers
to the Windows node as a real inbound chat message from peer `12D3KooWGvCWJN`.

```
21:28:03  auto_reply_ack_queued                 <- window consumed (one reply)
21:28:04  auto_reply_suppressed_rate_limit
21:28:08  auto_reply_suppressed_rate_limit
21:28:10  auto_reply_suppressed_rate_limit
21:28:12  auto_reply_suppressed_rate_limit
21:28:25  auto_reply_suppressed_rate_limit      (cloud probe e6675d8d)
21:28:33  auto_reply_suppressed_rate_limit      (burst probe 1)
21:28:37  auto_reply_suppressed_rate_limit      (burst probe 2)
21:28:41  auto_reply_suppressed_rate_limit      (burst probe 3)
21:29:19  auto_reply_ack_queued                 <- window reopened (>60s)
```

Nine inbound human messages inside one minute produced exactly ONE reply; the
window then reopened on its own at +76s, so the cap throttles without ever
silencing the responder. Counts for the hour: 8 acks, 8 rate-limited, 0
duplicates, 0 machine-skip lines attributed to this window (all 8 acks belong to
the pre-restart uncapped run at 21:07-21:08, whose sibling `skipped` lines are in
the previous hour's file).

## Device handling rule recorded (operator directive, 2026-09-16)

"NEVER EVER drive the pixel - that's a mandatory rule - only app deploy and
passive log pull for SCMessenger logs." Recorded in `docs/rules/FREEBUFF.md`
section 4 as a hard prohibition: no UI automation (`input text`/`input tap`), no
`am start`/`am force-stop`, no reading state by provoking it. Permitted device
interactions are exactly two: `adb install -r` of an APK, and passive log pulls
(`adb logcat`, `run-as` reads of the app's own files).

## Canonical-identifier exposure in the HTTP API send path (corrected)

An earlier version of this note claimed `POST /api/send` returns `400 Invalid
peer ID` for every real contact. **That claim was wrong and is withdrawn** - it
was inferred from reading `cli/src/api_axum.rs:260-269` (find the contact, then
`contact.peer_id.parse::<libp2p::PeerId>()`) plus the fact that some stores now
hold the canonical 64-hex form. A live request settles it:

```
POST http://18.234.62.247:9876/api/send
  {"recipient":"12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw","message":"..."}
  -> {"success":true,"message_id":"e6675d8d-...","status":"accepted"}  HTTP 200
```

So the endpoint works when the contact's stored `peer_id` is the base58 form
(this cloud node's contact for the Windows node is), and would fail for a
contact stored as 64-hex (which `contact list` shows for at least the phone on
the Windows node). The honest statement is therefore: **the API send path is
form-dependent, not universally broken.** It is the same class as the CLI defect
above and deserves the same resolver, but it has NOT been shown to fail, so it is
not filed as a break. One test - send to a contact whose `peer_id` is hex - would
settle it.

### Auto-reply findings from the same window (report only)

- Auto-reply messages are NOT written to local history: every delivery logs
  `Message <id> not found in history, could not mark as delivered` (seen for
  `eaa2698d` and `77dd0ed7`). The node cannot show the operator what its own
  responder said.
- `Processed application delivery receipt event="receipt_outbox_cleared"
  ... removed=false` on every receipt: the outbox entry is not removed by the
  receipt path. Consistent with the 159-message undelivered backlog; worth a
  separate look, not changed here.
- Unauthorized use on the local node on 2026-09-16 (13 acks to the operator's
  phone across 18:23Z-19:08Z) was raised by the operator; the flag was removed
  from the running node and the node was restarted without it. The AWS node was
  never affected (`docker inspect`: `CMD=[scm start]`, no auto-reply env). The
  flag now carries the operator's explicit authorization for testing and for
  always-on nodes, and the node was left running unarmed.
