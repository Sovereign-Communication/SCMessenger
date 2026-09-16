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

### Auto-reply findings from the same window (report only)

- Auto-reply messages are NOT written to local history: every delivery logs
  `Message <id> not found in history, could not mark as delivered` (seen for
  `eaa2698d` and `77dd0ed7`). The node cannot show the operator what its own
  responder said.
- `Processed application delivery receipt event="receipt_outbox_cleared"
  ... removed=false` on every receipt: the outbox entry is not removed by the
  receipt path. Consistent with the 159-message undelivered backlog; worth a
  separate look, not changed here.
- Unauthorized use on the local node on 2026-09-16 (11 acks to the operator's
  phone between 18:23Z and 18:55Z) was raised by the operator; the flag was
  removed from the running node and the node was restarted without it. The AWS
  node was never affected (`docker inspect`: `CMD=[scm start]`, no auto-reply
  env). The flag now carries the operator's explicit authorization for testing
  and for always-on nodes.
