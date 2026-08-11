# Contact lookup misses the public-key flavor -- every send to a known contact takes the unknown-peer path

## RESOLVED 2026-08-09 by `ed57d818` (macOS lane) -- with two corrections

Fix landed as `ed57d818 fix(core): resolve contacts by public key`, touching
exactly the two files this ticket named. Reviewed by the Windows lane:

- Adds `ContactManager::get_by_public_key()` (`core/src/store/contacts.rs`),
  case-insensitive match on `contact.public_key`.
- Wires it as a fallback in the send path (`core/src/iron_core.rs:779-785`).
- Adds `lookup_by_public_key_resolves_peer_keyed_contact` unit test.

Functionally correct. It resolves the reported symptom.

### CORRECTION 1 -- this ticket overstated the severity. It was not P1.

Filed as "P1, delivery-truth, silently downgrades contact validation on every
send". That was wrong and is corrected here rather than left to mislead.

`known_by_pubkey` is referenced in exactly two places
(`iron_core.rs:779` and `:787`, grep-confirmed) and gates ONLY the
hash-confusion error check and the warning. Nothing downstream branches on
it: encryption uses `recipient_id` directly, and unknown-peer sends were
already permitted by design. So the real impact was a spurious WARN plus
altered reachability of one diagnostic guard -- not a delivery, routing, or
encryption defect. Accurate severity: **P3, diagnostic**.

The finding was still worth filing -- the log line was actively misleading two
lanes debugging delivery -- but it did not belong at the same tier as the
receipt-marker defect, which does drop real deliveries.

### CORRECTION 2 -- the fix trades an O(1) miss for an O(n) scan on EVERY send

`get_by_public_key()` calls `list()`, and `list()` does `scan_prefix` plus a
`serde_json::from_slice` per contact (`contacts.rs:309-321`). Because
contacts are keyed by PeerId, the primary `get(public_key)` lookup ALWAYS
misses, so the fallback scan now runs on **every send to every contact**, not
just on a genuine miss.

The comment still sitting directly above the call site says the opposite:
"Only when that misses do we pay for a scan, so the common send path stays
O(1) against the contact store". That comment is now inaccurate.

This ticket's original fix sketch called for a `public_key -> peer_id` index
maintained at write time, which keeps the send path O(1). The scan is a
smaller and perfectly reversible change, and at current contact counts (7 on
the Windows node) the cost is negligible -- but on a farm node with a large
contact list this is a full deserialize of the contact store per message.

Recommended follow-up (NOT urgent, not blocking the tag): either add the
write-time index, or update that comment so the next reader is not misled
about the cost. Filed as a note here rather than a new ticket to avoid
backlog churn over a non-blocking performance point.

### Soak impact: none

`git diff --name-only 49bc3f56 ed57d818` returns only `core/src/iron_core.rs`
and `core/src/store/contacts.rs`. Transport, CLI, and the Cargo manifests are
byte-identical, so panic/connection-behaviour evidence gathered at
`49bc3f56` transfers to `ed57d818` unchanged. A running soak does not need to
be restarted for this fix.


Status: Active
Severity: P1 (delivery-truth; silently downgrades contact validation on every send)
Discovered: 2026-08-09, Windows lane, live node run at anchor `49bc3f56`
Discovered by: running the five-node link, not by code reading
Gate: touches `core/src/` delivery path -- adversarial review required before merge

## Summary

`IronCore`'s send path looks a contact up by **public key**. The contact store
is keyed by **libp2p PeerId**, with a fallback that resolves an **identity_id**.
There is no public-key index. So a send to a contact that demonstrably exists
logs `[WARN] sending to a recipient with no contact record; proceeding
(unknown-peer sends remain allowed)` and proceeds down the unknown-peer branch.

This is the same identifier-flavor class that `T1` fixed for BLOCKS
(`core/src/store/blocked.rs` dual-flavor write). Contacts never got the
equivalent treatment.

## Evidence (live, this run -- not inferred)

Contact exists. `GET /api/contacts` returns:

```json
{"peer_id":"12D3KooWNC5rEKFhuxDNDNsJ6Q58Ca75LnxfjUqspGzGRdYRUWyt",
 "public_key":"b7dc9198306d41952c49410f63cfd19f231536f37e886767753cbccc78616e0f",
 "name":"MacLane-GPT"}
```

That endpoint reads the SAME store as the send path --
`cli/src/api.rs:849-852` calls `core.contacts_store_manager()`, and
`core/src/iron_core.rs:3530-3532` returns `self.contact_manager.read().clone()`.
So this is not a two-store split; it is one store answering `list()` and
missing on `get()`.

Send to that exact public key, same process, seconds later:

```
2026-08-10T06:35:50.192968Z WARN scmessenger_core::iron_core:
  [WARN] sending to a recipient with no contact record; proceeding
```

## Mechanism (file:line)

- Write path keys by PeerId: `core/src/store/contacts.rs:221`
  `let key = contact_key(&contact.peer_id);`
- Read path: `core/src/store/contacts.rs:241` `get(peer_id: String)` builds
  `contact_key(&peer_id)`, and on miss (`:252-253`) tries
  `resolve_identity_id(&peer_id)`.
- Indexed flavors: **peer_id** and **identity_id**. NOT public_key.
- Caller passes a public key: `core/src/iron_core.rs:779-785`
  `self.contact_manager.read().get(recipient_id.to_string())`, where
  `recipient_id` is the recipient public key (the `/api/send` `recipient`
  field is a 64-char public key hex).

`b7dc9198...` is a public key, so the primary key misses and the identity_id
fallback also misses -- identity_id is `blake3(public_key_bytes)`, a different
value. Result: `None`.

## Why this matters beyond a noisy log line

The miss silently routes every send to a KNOWN contact through the
unknown-peer branch, so contact-record-based checks do not run on the path
that carries real traffic. It also defeats the loud, deliberate guard
immediately below it (`iron_core.rs:797-806`), which is designed to catch the
hash/pubkey confusion by scanning contacts -- that scan compares
`blake3(contact.public_key)` against the recipient, so it only fires for an
identity_id-valued recipient, never for this case.

Note this is NOT the same defect the Mac lane hit earlier (a contact whose
`public_key` field literally contained a PeerId string). That one is repaired;
this one is a missing index and is still live.

## Fix sketch (do not implement without the adversarial gate)

Mirror the T1 dual-flavor approach in `core/src/store/contacts.rs`:

1. On write (`:221`), in addition to the PeerId-keyed entry, maintain a
   `public_key -> peer_id` index alongside the existing
   `identity_id -> public_key` index (`:227-230`).
2. In `get()` (`:241`), after the PeerId key and the identity_id fallback both
   miss, resolve via the public-key index.
3. Keep `list()` deduped so one contact does not surface three times.

Alternative considered: normalise the caller to look up by PeerId. Rejected as
the primary fix -- the send path legitimately holds only a public key at that
point, and the other two flavors are already indexed, so the store is the
right place to close the gap.

## Acceptance

- Unit test: store a contact keyed by PeerId, then `get(public_key_hex)`
  returns it.
- Unit test: `list()` still returns exactly one entry for that contact.
- Live: repeat the send that produced the warning above and confirm the
  `no contact record` WARN no longer fires.

## Verification not yet done

The observed behaviour is confirmed; the FIX is not written and not verified.
No code was changed by the session that filed this.
