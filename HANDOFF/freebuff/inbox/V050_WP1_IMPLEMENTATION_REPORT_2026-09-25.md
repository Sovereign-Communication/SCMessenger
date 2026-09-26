# V050-WP1 identity unification — implementation report

**Written:** 2026-09-25
**Verification section updated:** 2026-09-26, after the audit pass and a full
re-verification of PR #383 CI on the final head
**Task:** `HANDOFF/freebuff/queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md`
**Authority:** `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` section 2, WP1
**Status: IMPLEMENTED AND CI-VERIFIED — NOT DONE.**
**Blocker:** the keyed JEV completion gate could not run. Per the Freebuff
lane contract, `UNVERIFIED-JEV` is not DONE. This is a stop condition, not a
ranking. **The green CI run does not soften it** — see Blocker 1.
**Delivery state:** branch `freebuff/wp1-identity-unification`, code head
`8d9013f4`, pushed, PR #383 OPEN and green. Not merged. This document is the
commit after that head; see Delivery for the full commit list.
**WP1.4 ghost-topic guard (2026-09-26):** the own-topic acceptance is now
asserted behaviourally -- `own_topic_subscribe_is_registered_on_the_running_node`
starts a real node and reads `SwarmHandle::get_topics`, with a control proving
the observable discriminates -- so deleting the startup own-topic subscribe in
`core/src/transport/swarm.rs` fails it. Compiled and executed by CI on PR #383,
NOT locally: `scripts/disk_budget.py` returned BLOCKED (exit 2) at 1.4 GB free
at that time, so the local suite never ran against this test. The Rule-8 gated file was not
edited.

**Audit pass (2026-09-26):** an external four-dimension audit raised four
findings against this branch. All four are now settled, and one of them
corrected the audit rather than the code. See Audit pass below. Net effect:
the branch ships no behavior the ticket did not ask for, and
`core/src/identity/keys.rs` is now byte-identical to `origin/main`.

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
contact-key rule was then consolidated onto a single owner inside the contact
store, shared by its read and write paths. (The CLI resolves through
`peer_id_from_contact_identifier`, which delegates to the pre-existing
`ledger_entry` helpers; the store's rule is a separate question, deliberately —
see Audit pass item 2.) All suites touching the changed code are green on CI,
including the Android and iOS jobs.

An audit then found the branch shipped three behaviors the ticket did not ask
for and one test that could not fail. All four are settled: two behaviors
removed, one reverted, one test replaced, and the audit's own recommendation to
delete `canonical_contact_key` was tried and found to break a pre-existing
pinned contract, so it was not taken.

## Delivery

| Item | Value |
|---|---|
| Branch | `freebuff/wp1-identity-unification` |
| Code head | `8d9013f4` — "refactor(core): settle the unrequested-behavior findings" |
| PR | #383, OPEN, green at that head |
| Files | 8 changed, +1812 / -97 |
| Merged | No. The Freebuff lane does not self-merge. |

Commits, oldest first:

| Commit | Subject |
|---|---|
| `c0a12149` | fix(core): close the contact-key fabrication and read/write asymmetry |
| `618fad3e` | docs(handoff): record PR #383 CI evidence in the WP1 report |
| `d374cd30` | test(core): assert the own-topic subscribe, not just the topic shape |
| `c6330322` | style: rustfmt the WP1.4 test |
| `5a513877` | test(core): make the CRYPTO-01 regression actually spoof |
| `77540ac8` | test(core): pin where contact and ledger canonicalization must differ |
| `8d9013f4` | refactor(core): settle the unrequested-behavior findings |

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

## Changed files (8, all in this repository)

| File | Change |
|---|---|
| `core/src/store/contacts.rs` | removed fabrication fallback; placeholder recovery; `canonical_contact_key` as single owner used by both `add()` and `get()`; sole `PLACEHOLDER_KEY_NOTE`; boundary test against the ledger owner |
| `core/src/contacts_bridge.rs` | imports `PLACEHOLDER_KEY_NOTE` instead of a private copy |
| `core/src/iron_core.rs` | the CRYPTO-01 spoof regression; placeholder and identity-hash tests. `prepare_message` is unchanged from main |
| `core/src/lib.rs` | +3: declares the `#[cfg(test)]`-only `test_support` module |
| `core/src/test_support.rs` | new, 26 lines, `#[cfg(test)]` only: the shared self-certifying keypair builder |
| `cli/src/main.rs` | contact-identity and dial-scheduler test modules; `looks_like_ed25519_pk` deleted (the `cmd_contact` gate is back on the pre-existing lax validator — see Audit pass) |
| `core/tests/integration_wp1_identity_unification.rs` | new, 12 tests, 469 lines |

`core/src/identity/keys.rs` was briefly given a 20-line test helper and no longer
is: `git diff origin/main...HEAD -- core/src/identity/keys.rs` is empty. That
file is byte-identical to main. It is not Rule-8 gated either — `identity/` is not
in `core/src/{crypto,transport,routing,privacy}`.

No file under `core/src/{crypto,transport,routing,privacy}` was modified, so
Rule-8 adversarial review is not claimed by this change. The own-topic and
ghost-guard paths in `core/src/transport/swarm.rs` were deliberately NOT edited;
the tests assert the public seam (`extract_ed25519_public_key_from_peer_id` and
the topic string shape) rather than the private `is_ghost_peer_topic`, because a
private unit test inside that directory would require the review this change
does not claim.

## Test evidence — CI (authoritative for the shipped tree)

PR #383, code head `8d9013f4`. All seven workflow runs report
`conclusion=success`; the last concluded **2026-09-26T23:12:11Z**.

| Run | Conclusion | Completed (UTC) |
|---|---|---|
| `CI` (run 36275896975) | success | 2026-09-26T22:44:41Z |
| `Lint` | success | 2026-09-26T22:24:45Z |
| `Cross` | success | 2026-09-26T22:52:20Z |
| `Mobile` (Android) | success | 2026-09-26T23:11:41Z |
| `iOS Build & Test` | success | 2026-09-26T23:12:11Z |
| `Repository Hygiene` | success | 2026-09-26T22:19:48Z |
| `Auto Label` | success | 2026-09-26T22:19:56Z |
| **Total PR checks** | **34 pass / 0 fail / 0 pending / 0 skipped** | |

`Cross`, `Mobile` and `iOS Build & Test` had never been read to completion at any
earlier head on this branch; all three are now observed green on the final head.

Within the `CI` run: `Test (ubuntu-latest)`, `Test (windows-latest)` and
`Test (macos-latest)`, `Windows CLI Artifact`, `FFI Surface Contract`, `Docs`,
`Handoff ownership scope` and `Lint` (rustfmt + clippy + cargo-deny) all
succeeded.

Log evidence, `Test (ubuntu-latest)` job 108498465510, every line `ok`:

```
iron_core::tests::placeholder_contact_is_not_an_encrypt_target ... ok
iron_core::tests::verified_recipient_still_encrypts ... ok
cli_contact_add_gate_stays_lax_and_matches_the_public_key_validator ... ok
iron_core::tests::crypto01_spoofed_payload_sender_id_cannot_decide_attribution ... ok
iron_core::tests::identity_hash_not_usable_as_recipient ... ok
store::contacts::tests::contact_and_ledger_canonicalization_agree_except_where_the_contract_differs ... ok
```

A case-sensitive scan of that log for `test result: FAILED`, `panicked at`,
`^error[` and `could not compile` returns 0.

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

## Audit pass (2026-09-26)

An external four-dimension audit (SPEC / DESIGN / CORRECTNESS / QUALITY) graded
this branch and raised four findings. All four are settled. One of them
corrected the audit, not the code.

**1. The CRYPTO-01 test could not fail. Replaced.** The previous
`crypto01_attribution_comes_from_authenticated_key_not_payload_sender_id` never
put a third-party value into the message: it sent an honest payload whose
`sender_id` *was* the authenticated key, then compared stored rows, so a
payload-trusting implementation and a correct one produced identical results.
It is replaced by `crypto01_spoofed_payload_sender_id_cannot_decide_attribution`,
which encrypts and signs a forged payload (`sender_id` = a third party's key)
with the **real** sender's keys, so envelope verification succeeds and the
question becomes which identity files it. A control assertion
(`received.sender_id == third_key`) proves the lie survived decryption, so the
stored-row assertion cannot pass vacuously. Green on CI.

**2. `canonical_contact_key` versus `canonical_ledger_peer_id` — the audit was
wrong, and the code was left alone.** The audit called
`canonical_contact_key` a third divergent copy and recommended deleting it in
favour of delegation. That was attempted and it breaks
`lookup_by_public_key_resolves_peer_keyed_contact`, a test that predates this
branch and pins the "V2 canonicalization" contract: a valid 64-hex key supplied
under an arbitrary add-time label **is** the contact's identity. The ledger
owner refuses that input deliberately, because it maps spellings of *the same
key material* and a key that does not re-derive the peer id is not a spelling
of it. The two answer different questions. What genuinely is shared — base58
PeerId to canonical hex — was already single-owner in
`public_key_hex_from_libp2p_peer_id`, which `canonical_contact_key` calls. The
boundary is now pinned by
`contact_and_ledger_canonicalization_agree_except_where_the_contract_differs`
and documented on the function, so it is not re-litigated as drift.

**3. The empty-recipient guard in `prepare_message_internal` was a no-op, and is
gone.** It was 15 lines that restated existing behavior: `""` decodes to 0 bytes
and the 32-byte width check already returned `InvalidInput`; whitespace decodes
as invalid hex. Same error either way. It was removed and
`placeholder_contact_is_not_an_encrypt_target` still passes, which is the proof.
`prepare_message` is now unchanged from main. The behavior stays pinned by that
test, re-attributed to the width check that actually provides it.

**4. `cmd_contact` was rejecting contacts a user can add today, and is
reverted.** The branch had routed that gate through `is_valid_public_key`,
which additionally requires a canonical RFC 8032 encoding. The `--public-key`
argument three lines above is validated by `crypto::validate_ed25519_public_key`,
which is decompress-only, so a 32-byte value that decompresses but is
non-canonical (y >= p, or the sign bit set on x = 0) passed that check and was
accepted as a direct ed25519 key; under the strict predicate it fell through to
the blake3 branch, failed to resolve, and the contact was **silently not
added**, behind a message about identity IDs and exit status 0. Nothing in WP1
asked the CLI to get stricter. The gate is back on the lax validator, whose
acceptance set is provably identical to the pre-branch one (dalek's
`VerifyingKey::from_bytes`, libp2p's `try_from_bytes` and
`validate_ed25519_public_key` are all decompress-and-nothing-else). Pinned by
`cli_contact_add_gate_stays_lax_and_matches_the_public_key_validator`.

**Also settled:** the recovery-count change. It is not a separate decision. The
store's `reconcile_from_history` was the outlier among three copies of the same
function: `contacts_bridge::reconcile_from_history` and
`contacts_bridge::emergency_recover`, both already on main, each record a
placeholder and increment for it. The store now matches its siblings. And
`self_certifying_keypair` moved out of `core/src/identity/keys.rs` into the
`#[cfg(test)]`-only `core/src/test_support.rs`; `iron_core.rs`'s private inline
copy of the same helper now uses it too.

**Not settled here — see Blocker 6:** the Android half of the WP1.3 clause.

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
2. **Disk pressure.** `python scripts/disk_budget.py` returns
   `[FAIL] below the hard floor`; `df -h /c` most recently read 237G total,
   1.3G available, 100% used. Observed from this worktree on 2026-09-26. Broad local
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
5. **No adversarial review is on file.** Three copies of the self-certifying
   keypair builder survive outside `core/src/test_support.rs`:
   `core/tests/integration_wp1_identity_unification.rs`, `cli/src/main.rs`,
   and the pre-existing `core/src/store/ledger_entry.rs:2805`. They cannot be
   de-duplicated without publishing a test API, since `core/tests/` and the CLI
   compile the crate without `cfg(test)`. Known, accepted duplication, not an
   oversight. Not a Rule-8 question: no gated directory is involved.

6. **WP1.3 is met for CLI and WASM; the Android half is not, and cannot be
   closed from here.** Authority section 2 asks that CLI, UI, core, Android and
   WASM send paths all resolve hex to PeerId through one helper. Traced:

   - **WASM: already delegates, closed on evidence.** The JSON-RPC front end
     `core/src/wasm_support/rpc.rs` parses `send_message` into
     `ClientIntent::SendMessage { recipient }` and does no peer-id resolution of
     its own — the recipient string is taken verbatim. Its only in-repo consumer
     is `cli/src/server.rs:481`, which forwards it as `UiCommand::Send`, and the
     CLI main loop resolves it at `cli/src/main.rs:3736` with
     `peer_id_from_contact_identifier` — the same helper the WP1.3 CLI tests
     pin. There is no WASM-side parsing left to unify.

   - **Android: carries its own parsing.** `android/.../utils/PeerKeyUtils.kt`
     re-implements base58 (via `BigInteger`), the 38-byte protobuf identity
     multihash, and its own 64-hex `isValidPublicKey`; `PeerIdValidator.kt`
     adds a second curve-point check and a `canonicalKey`. The send path's
     resolution step is `MeshRepository.toDialableRoutePeerId`, which calls
     `PeerKeyUtils.generateLibp2pPeerIdFromPublicKey`. Neither delegates to
     core.

   Making Android delegate requires exporting `peer_id_from_public_key_hex`
   over UniFFI, and nothing usable is exposed today: the checked-in FFI
   snapshot `scripts/ffi-snapshots/kotlin-symbols.txt` contains exactly one
   peer-id symbol, `extractPublicKeyFromPeerId`, which is the *reverse*
   direction. So the work is (a) an API-contract change, which is an operator
   decision and not a lane decision, and (b) blocked in this environment:
   `scripts/ffi_surface.sh` diffs the regenerated bindings against that snapshot
   and fails on any change, so the snapshot must be regenerated by
   `cargo build -p scmessenger-core --features gen-bindings` plus `gen_kotlin`
   and `gen_swift` — which the 1.3 GB free disk forbids — and the generated
   bindings may not be hand-edited. No partial change was pushed: an Android
   edit that cannot be compile-verified here, on a path that cannot reach the
   core helper without a surface change, would be exactly the kind of unverified
   assertion this note exists to prevent.

## Not authorized by this document

This is a status report only. It does not authorize a commit, a push, a PR, a
merge, a device action, a node restart, or a legacy handoff migration. The
commit, push, and PR referenced above were made under separate operator
instruction; this note does not ratify them and does not authorize the merge.
