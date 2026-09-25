# WP1 findings: two gaps this ticket's rows surfaced, deliberately not fixed here

Status: OPEN -- filed 2026-09-21 alongside the WP1 PR
Lane: Freebuff
Related: `HANDOFF/freebuff/queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md`
Authority: none needed to read -- this is the evidence record for the two rows WP1
could not close inside its own scope. Decisions belong to the orchestrator/operator.

Neither item below was "fixed" in the WP1 PR. WP1's scope is
`core/src/contacts_bridge.rs`, `core/src/store/contacts.rs`, and CLI send regression
tests; item 1 is a build-configuration break, item 2 needs the send path in
`core/src/iron_core.rs`, which is crypto-adjacent and outside the stated rows.

## 1. Twelve tests in `cli/src/main.rs` have never compiled or run

Evidence, obtained this session:

```
$ cargo metadata --no-deps --format-version 1        (scmessenger-cli targets)
  ['lib']  name=scmessenger_cli       test=True
  ['bin']  name=heartbeat-probe       test=True
  ['bin']  name=scmessenger-cli       test=False      <- the CLI's main binary
  ['bin']  name=stress-test           test=True
```

```
$ grep -c "#\[test\]" cli/src/main.rs
12
```

`test = false` on that bin means those twelve `#[test]` functions are not compiled,
not run, and cannot fail. They are not "skipped" -- they do not exist as far as
`cargo test` is concerned, which is why nobody noticed.

Consequence for WP1: the ticket's row "CLI send regression tests" cannot be satisfied
in `cli/src/main.rs`. The send-resolution property is therefore asserted in
`core/src/iron_core.rs` against `crate::store::peer_id_from_public_key_hex`, the
canonical helper that the CLI wraps (`peer_id_from_contact_identifier` calls
`parse::<PeerId>()` and then that helper). Same property, a target that runs.

Decision needed, one of:
- set `test = true` for that bin and fix whatever the twelve now surface (they have
  been uncompiled for an unknown period, so some may not build); or
- move the still-valuable ones into `cli/src/lib.rs` or a `cli/tests/` target.

Until one is done, every future test added to `main.rs` silently reports nothing.

## 2. The send path's hash-confusion guard is unreachable for the input it targets

`IronCore::prepare_message_internal` (core/src/iron_core.rs) refuses a recipient that
is a known contact's `identity_id` (blake3 hash) rather than their public key -- a
loud, correct refusal, because encrypting to a hash produces ciphertext nobody can
open. That check lives inside the `if !known_by_pubkey { ... }` branch.

`known_by_pubkey` is decided first by `ContactManager::get(recipient)`, and `get`
falls back to `resolve_identity_id` (`core/src/store/contacts.rs`), an
`identity_id -> public_key` index populated when contacts are stored. So once the
contact exists, its HASH resolves to the contact and the fast path is satisfied by a
hash -- the guard is never reached for exactly the input it was written for.

Measured, in the earlier WP1 test run (`tmp/v050-wp1-core2.log`):

```
[WP1-DIAG] identity_id -> resolved contact: Some(("c2952..e5", "c2952..e5"))
[WP1-DIAG] send-to-identity_id is_ok=false
```

The observed refusal came from key validity in the encryptor, not from the guard: a
blake3 hash is 32 arbitrary bytes, so whether they are accepted as a curve point is
roughly a coin flip per identity. That is why the WP1 test asserts the MECHANISM
(a stored contact resolves by its `identity_id`) instead of the outcome -- an
outcome assertion here passes or fails by luck, and a test that flips is worse than
no test.

Repairing it means deciding the fast path's precedence: either `get` must not resolve
a hash for send-path use, or the hash check must run before the fast path. That
touches `prepare_message_internal` and `ContactManager::get`, i.e. the send path and
the store, and is crypto-adjacent -- hence out of WP1's scope. It needs its own
ticket with its own gates.

## 3. WP1.2 grep result: row satisfied, with one site worth knowing

Every `Contact::new` call site was audited paren-aware and test-aware (a lazy regex
mis-splits these calls -- the first attempt at this grep reported a false zero):

- 35 call sites across `core/src` and `cli/src`
- 12 of them pass the same value for both slots: 11 are `#[cfg(test)]` fixtures
  (including the legacy poison shape that exists to prove repair)
- exactly one is production: `cli/src/server.rs:1510`
  `Contact::new(public_key.clone(), public_key)`

That production site is the canonical-key form, not a PeerId in the key slot: the
code comments the decision, and `ContactManager::add()` canonicalizes
(`contact.peer_id = canonical.clone()` when the public key is 64 hex chars), so the
stored record is consistent with the canonical form either way. No production site
passes a PeerId-shaped or empty value in the public-key slot.

It is recorded here because a future naive grep for "the same value twice" will flag
it, and the answer should not have to be re-derived.
