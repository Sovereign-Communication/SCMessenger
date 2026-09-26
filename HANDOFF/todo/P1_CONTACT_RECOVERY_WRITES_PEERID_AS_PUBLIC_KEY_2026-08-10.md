# P1 -- Contact recovery writes the PeerId into the public_key field, blocking outbound sends

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Active
Severity: P1 (blocks all outbound encryption to any affected peer)
Filed: 2026-08-10
Gate mapping: G1 pairwise messaging, G3 delivery truth
Found by: code audit while reconciling the Windows lane's runtime contact repair

## The defect

`core/src/contacts_bridge.rs` creates contacts with the PeerId in BOTH fields,
in two separate production paths:

`:298`
```rust
// We need a public key to create a Contact.
// In a real scenario, we'd use the libp2p peer_id to derive the key.
let contact = Contact::new(msg.peer_id.clone(), msg.peer_id.clone());
```

`:392` (inside `emergency_recover`)
```rust
// For emergency recovery, we use the peer_id as the public key placeholder.
let contact = Contact::new(msg.peer_id.clone(), msg.peer_id.clone());
```

`Contact::new(peer_id, public_key)` -- the second argument is the public key.
A PeerId is not a public key. A contact written this way cannot be used to
encrypt, so **every outbound send to that peer fails while inbound and ACKs
keep working** -- exactly the asymmetry the Windows lane observed in the field.

## Why this is not already fixed

The Windows lane repaired the **data** at runtime: derived the real key with
`scripts/peerid_to_pubkey.py` and re-POSTed `/api/contacts`
(`HANDOFF/ORCHESTRATOR_TAKEOVER_2026-08-10_WINDOWS_LANE.md` Section 3.5).
That fixed one poisoned record. **The code that produces poisoned records is
unchanged**, so the condition recurs on the next recovery pass.

## The correct implementation already exists

`core/src/store/contacts.rs:176` `derive_public_key_from_peer_id()` does this
properly: it validates 64-hex input is a genuine Ed25519 curve point, explicitly
rejects `identity_id` (also 64 hex, but a Blake3 hash, not a key), and otherwise
base58-decodes the libp2p PeerId and extracts the embedded Ed25519 key.

`core/src/store/contacts.rs:167` already uses it correctly for the same
"recover contacts from history" purpose:

```rust
if let Ok(pub_key) = self.derive_public_key_from_peer_id(&msg.peer_id) {
    let contact = Contact::new(msg.peer_id.clone(), pub_key);
    self.add(contact)?;
}
```

So `contacts_bridge.rs` has a second, wrong copy of logic that is already
correct one module over.

## Repo-rule violation

Both sites are placeholder code on a production path. The comments say so
outright ("In a real scenario...", "...as the public key placeholder"). These
are the exact markers `docs/ORCHESTRATION.md` Section 9.1 requires to be
grepped for before accepting any diff, and they violate the repository's
no-placeholder mandate.

## Required fix

Both call sites must derive the public key and **skip the contact when
derivation fails**, rather than writing a record that is guaranteed to break
encryption. A missing contact is recoverable; a poisoned contact silently
blocks delivery and looks like a transport fault.

Prefer reusing `derive_public_key_from_peer_id` rather than adding a third
copy. If it is not reachable from `contacts_bridge.rs`, promote it to a shared
location rather than duplicating it.

## Acceptance criteria

1. Neither site passes `msg.peer_id` as the `public_key` argument.
2. Derivation failure skips the contact and is counted/logged; it must not
   create a record.
3. The recovered-count return value reflects only contacts actually created.
4. A unit test proves: a valid Ed25519-bearing PeerId recovers a contact with a
   real key; an `identity_id`-shaped 64-hex input creates NO contact.
5. `cargo test --workspace --no-run` compiles.

## Field cross-reference

Android device log on anchor `68fcc3f1` shows 144 occurrences of
`[WARN] sending to a recipient with no contact record; proceeding` in the
current-build window, alongside outbox flushes reporting `succeeded:0`. Confirm
after the fix whether that warning count changes; it is consistent with, but
does not by itself prove, this defect.
