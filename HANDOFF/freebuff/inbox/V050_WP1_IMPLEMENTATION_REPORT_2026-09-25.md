# V050-WP1 identity unification — implementation report

**Written:** 2026-09-25
**Verification section updated:** 2026-09-26, after PR #383 CI concluded green
**Task:** `HANDOFF/freebuff/queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md`
**Authority:** `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` section 2, WP1
**Status: IMPLEMENTED AND CI-VERIFIED — NOT DONE.**
**Blocker:** the keyed JEV completion gate could not run. Per the Freebuff
lane contract, `UNVERIFIED-JEV` is not DONE. This is a stop condition, not a
ranking. **The green CI run does not soften it** — see Blocker 1.
**Delivery state:** branch `freebuff/wp1-identity-unification`, commit
`c0a12149`, pushed, PR #383 OPEN and green. Not merged.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Headline

The ticket's stated premise was wrong about which code path is live, and wrong
about the shape of the defect. Two distinct bugs were found and fixed, and the
contact-key rule was then consolidated onto a single owner shared by the CLI and
the core. All suites touching the changed code are green on CI.

## Delivery

| Item | Value |
|---|---|
| Branch | `freebuff/wp1-identity-unification` |
| Commit | `c0a12149` — "fix(core): close the contact-key fabrication and read/write asymmetry" |
| PR | #383, OPEN, `mergeable=MERGEABLE` |
| Files | 6 changed, +1212 / -96 |
| Merged | No. The Freebuff lane does not self-merge. |

## Premise corrections

The ticket says contact recovery uses
`contacts_bridge::placeholder_or_derived_contact`. That function is correct and
was not modified. It is not the live path either.

The path `IronCore::emergency_recover` actually takes is
`core/src/store/contacts.rs::ContactManager::reconcile_from_history`
(reached from `core/src/iron_core.rs:4562`). Verified by reading the call
chain, not inferred from the ticket.

## Defect 1 — a public key was fabricated from a hash (WP1.1)

`derive_public_key_from_peer_id` in `core/src/store/contacts.rs` ended with a
fallback that returned the **last 32 bytes of any base58 blob** as an Ed25519
public key:

```rust
// Fallback: take last 32 bytes for non-standard PeerIds
if bytes.len() >= 32 {
    return Ok(hex::encode(&bytes[bytes.len() - 32..]));
}
```

For a non-identity peer id (for example a SHA-256 multihash) those bytes are
hash output, not a key. The stored `public_key` then looked populated, so no
later layer re-checked it, and every send to that contact encrypted to a value
the peer can never decrypt with. This is the same defect as storing the peer id
as the key, reached by a different route.

Fixed by deleting the fallback and delegating to the one self-certifying helper
the `contacts_bridge` recovery path already uses, so both paths agree by
construction.

`reconcile_from_history` now records a placeholder (empty key plus a notes
marker) for a peer it cannot derive, instead of silently skipping the peer.

**Behaviour change for review:** the recovered count returned by
`emergency_recover` can now be higher than before, because a placeholder row
counts as recovered where it was previously not created at all.

## Defect 2 — read and write disagreed on the contact key

Found while triaging a pre-existing red test, not by reading code.

`add()` canonicalizes a libp2p peer id to the contact's public-key hex and
files the row under the hex. `get()` only knew how to resolve an
`identity_id`. The manager therefore could not read back a row it had just
written.

Executed evidence, from a temporary probe run against the real store and since
deleted:

```text
add(Contact::new(peer_id, peer_id))     <- legacy poison shape
get(peer_id)      -> None
get(key_hex)      -> Some(peer_id=0f9c9dc2... public_key=0f9c9dc2...)
list() len = 1
```

This is not cosmetic. CLI envelope learning looks contacts up by the peer id
seen on the wire. A miss there skips the nickname-preference branch and
rebuilds the record, dropping a user-set `local_nickname`.

Fixed by making `get()` resolve a libp2p peer id through the same derivation
`add()` uses, with a guard that bounds the recursion (an already-canonical
value derives to itself, so unguarded re-entry would loop forever). Both call
sites now go through one owner, `ContactManager::canonical_contact_key`.

## Consolidation — one owner for the contact-key rule

`is_valid_public_key` in `core/src/identity/keys.rs` is the single owner of
the "is this a public key rather than an identity_id" predicate. Two
competing copies were deleted rather than left to drift:

| Deleted | Replaced by |
|---|---|
| `is_ed25519_public_key_hex` (core) | `is_valid_public_key` |
| `looks_like_ed25519_pk` (cli) | `is_valid_public_key` |
| 3 of 4 `PLACEHOLDER_KEY_NOTE` definitions | 1, in `store/contacts.rs` |

`PLACEHOLDER_KEY_NOTE` is now defined only in `store/contacts.rs`, which has no
cfg gate, so the marker still reaches the wasm build. `contacts_bridge.rs`
imports it instead of keeping a private copy.

Remaining call sites of the single owner: `core/src/identity/keys.rs`,
the `identity/mod.rs` re-export, and four sites in `cli/src/server.rs` plus one
in `cli/src/main.rs`.

**Not fully deduplicated, flagged not forced:** two test-local
`self_certifying_peer` helpers remain, in `core/tests/integration_wp1_identity_unification.rs:25`
and `cli/src/main.rs:990`. They sit in two different crates, so they cannot
share the `#[cfg(test)] pub(crate)` helper added at
`core/src/identity/keys.rs:94`. The pre-existing `self_certifying_pair` at
`core/src/store/ledger_entry.rs:2805` is untouched by this work and was already
on `origin/main`.

## Correction to my own first implementation

An earlier draft added a curve-decompression check to `add()` and documented it
as the thing that distinguishes a public key from a hash. That claim was false
and was measured rather than left standing:

```text
random 32-byte values accepted as Ed25519 points: 966/2000 = 48.3%
```

A second, wider sample gave 2022/4000 = 50.55%. Because roughly half the curve
is valid points, a blake3 `identity_id` passes decompression about half the
time. Curve decompression is documented in the code as a cheap pre-filter,
never a trust decision. The real boundary is self-certification, and where that
is genuinely decidable it is enforced at the point of use.

A second draft gated `add()` on self-certification and broke a pre-existing
test. Reading that test showed why: "a valid 64-hex key IS the canonical
identity" is the established unification contract, and once a contact is keyed
by its own public key there is no separate peer id left to check against. That
gate was reverted rather than forced through.

## Rebase, and the predicate this work consolidated onto

Two facts a reviewer must have before reading the diff.

**1. The branch was rebased onto `origin/main` by a 3-way patch, not a plain
fast-forward.** The work was developed against an older base, then landed on
`origin/main` (at `03b7d294`) with `git apply --3way`. One conflict occurred, in
`core/src/identity/keys.rs`, and was resolved by keeping `main`'s side. The
resulting diff to that file versus `origin/main` is **+20 lines, purely
additive** — the test helper only. Main's existing body was preserved intact.

**2. The predicate the work consolidated onto is stricter than the one the
local tests were written against.** `origin/main`'s `is_valid_public_key`
already carried an RFC 8032 canonical-encoding check
(`is_canonical_ed25519_encoding`, `core/src/identity/keys.rs:46`): it rejects
`y >= p` and rejects a set sign bit on an `x = 0` encoding. The local tests
were originally written against a decompression-only (dalek) check, which
accepts roughly half of all 32-byte values. So the surviving predicate is
**strictly stricter** than what the local test evidence exercised, and the
parity test was rewritten accordingly as
`consolidated_key_shape_check_is_strictly_stricter_than_the_old_cli_check`.

Consequence for the reader: the local numbers below were produced on the
**pre-rebase base** and do not describe the shipped tree. The shipped tree is
what CI ran.

## Changed files (6, all in this repository)

| File | Change |
|---|---|
| `core/src/store/contacts.rs` | removed fabrication fallback; placeholder recovery; `canonical_contact_key` as single owner used by both `add()` and `get()`; sole `PLACEHOLDER_KEY_NOTE` |
| `core/src/identity/keys.rs` | +20 lines only: `#[cfg(test)] pub(crate) self_certifying_keypair`. Main's strict `is_valid_public_key` body untouched |
| `core/src/contacts_bridge.rs` | imports `PLACEHOLDER_KEY_NOTE` instead of a private copy |
| `core/src/iron_core.rs` | `prepare_message` refuses an empty recipient key by policy; attribution + identity-hash tests |
| `cli/src/main.rs` | contact-identity and dial-scheduler test modules; `looks_like_ed25519_pk` deleted in favour of the single owner |
| `core/tests/integration_wp1_identity_unification.rs` | new, 10 tests, 316 lines |

No file under `core/src/{crypto,transport,routing,privacy}` was modified, so
Rule-8 adversarial review is not claimed by this change. The own-topic and
ghost-guard paths in `core/src/transport/swarm.rs` were deliberately NOT edited;
the tests assert the public seam (`extract_ed25519_public_key_from_peer_id` and
the topic string shape) rather than the private `is_ghost_peer_topic`, because a
private unit test inside that directory would require the review this change
does not claim.

## Test evidence — CI (authoritative for the shipped tree)

PR #383, commit `c0a12149`. All four workflow runs reported
`conclusion=success`; the last concluded **2026-09-26T05:33:56Z**.

| Check | Conclusion | Completed (UTC) |
|---|---|---|
| `Test (ubuntu-latest)` | success | 2026-09-26T04:59:40Z |
| `Test (windows-latest)` | success | 2026-09-26T05:03:38Z |
| `Test (macos-latest)` | success | 2026-09-26T04:54:54Z |
| `WASM` | success | 2026-09-26T04:36:53Z |
| `Rust Linting` | success | — |
| `Handoff ownership scope` | success | — |
| `Android Wiring Gate` | success | — |
| **Total** | **34 pass / 0 fail / 0 pending / 0 skipped** | |

`gh pr checks 383` returns an empty non-pass list.

This retires the two items that local runs could not cover:

- **Full test sweep: VERIFIED.** The local `cargo test --tests` sweep never
  completed (parallel runs aborted with `E0463` / `E0786` on truncated
  artifacts, and the host was out of disk). CI ran the wide sweep on all three
  platforms and all passed.
- **wasm32 build: VERIFIED.** `cargo check --target wasm32-unknown-unknown`
  could not complete locally on a cold build. The `WASM` job passed in 1m53s.

## Test evidence — local (pre-rebase base, superseded by CI above)

These ran on the pre-rebase base. They are retained as a record of what was
observed during development, not as proof about the shipped tree.

| Command | Result |
|---|---|
| `cargo test -p scmessenger-core --lib` | 1472 passed; 0 failed; 5 ignored |
| `cargo test -p scmessenger-cli --bin scmessenger-cli` | 89 passed; 0 failed |
| `cargo test -p scmessenger-core --test integration_wp1_identity_unification` | 10 passed; 0 failed |
| `cargo test -p scmessenger-core --test integration_contact_block` | 3 passed; 0 failed |
| `python scripts/check_wiring.py` | `[OK] All components, composables, routes, and utilities are correctly wired.` |
| `cargo clippy -p scmessenger-core --lib` | no warnings in changed files |
| `rustfmt` on the changed files | clean |

The CLI suite went from 88 passed / 1 failed to **89 passed / 0 failed**. The
failure was confirmed pre-existing by reverting the changed file to `HEAD` and
re-running before fixing it. The suites were re-run after formatting, not only
before.

## Blockers and open items

1. **JEV completion gate: UNVERIFIED.** `scripts/jev_canonical_check.py` imports
   a JEV evaluation module that is not present on this branch; it is added by
   the still-open PR #347. Observed failure: the script aborts with a Python
   `ModuleNotFoundError` for that missing import. WP1 cannot be marked DONE
   until this runs and returns `is_passing`.
   **This is unchanged by the green CI run and is not softened by it.** CI
   proves the change builds and its tests pass; the JEV gate is a separate
   completion contract that CI does not evaluate. The single status of this
   work remains NOT DONE.
2. **Disk pressure.** `python scripts/disk_budget.py` returned `BLOCKED`, most
   recently at 1.35 GB free (99.4% used), from another session. Broad local
   builds were stopped; this is why the wide sweep was delegated to CI. The
   sanctioned reclaim path currently frees nothing: the three trees
   `reclaim_safe.py` rates SAFE have no `target/` at all, while the trees that
   do hold build output are correctly rated HOLD (PR #383 unmerged; the shared
   checkout holds 64 dirty files belonging to other sessions).
3. **Pre-existing format debt, not touched.** `cargo fmt --check` still reports
   diffs in `cli/src/bin/conn-fanout.rs` and
   `core/src/transport/per_peer_cap.rs`. Neither file was modified by this
   work. Operator instruction was to leave them.
4. **The self-certification opportunity was not taken.** The contact store
   cannot distinguish an identity_id from a public key by width alone; by curve
   decompression it is a coin flip, as measured above. It is decidable at the
   point of use, and that is where the existing check lives. A stronger
   structural fix would require a binding the store does not currently have.
   Flagged, not attempted.
5. **No adversarial review is on file.** The two surviving test-local helpers
   are a known, accepted duplication rather than an oversight.

## Not authorized by this document

This is a status report only. It does not authorize a commit, a push, a PR, a
merge, a device action, a node restart, or a legacy handoff migration. The
commit, push, and PR referenced above were made under separate operator
instruction; this note does not ratify them and does not authorize the merge.
