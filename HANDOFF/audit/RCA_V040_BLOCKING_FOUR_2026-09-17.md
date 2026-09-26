# RCA: the four 0.4.0 blocking findings (TRN-04, TRN-07, AND-06, SEC-03)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

- **Date:** 2026-09-17
- **Base:** `main` @ `c2ce2f64` (PR #295 merge)
- **Author:** Buffy (Freebuff lane)
- **Source findings:** `HANDOFF/audit/SHADOW_AUDIT_V040_V050_ADVERSARIAL_REVIEW_2026-09-16.md` section 2
- **Method:** every mechanism below was read in the code at the cited file:line in this
  session, not recalled from the audit summary. Where my RCA disagrees with the
  finding's wording, the disagreement is stated.

Local verification performed for this document:

```
cargo check -p scmessenger-core --lib                                  -> clean, 0 errors
cargo test  -p scmessenger-core --lib                                  -> 1461 passed; 0 failed; 5 ignored
cargo test  -p scmessenger-core --test test_and06_public_key_vector_parity -> 8 passed
cargo test  -p scmessenger-core --test test_trn04_custody_lifecycle    -> 2 passed
```

That is a Windows-host run in the fix worktree. It is not the CI matrix (Linux,
Android, iOS, WASM) and is not a substitute for it.

---

## TRN-04 — Unauthenticated custody ingestion and infinite retention (CRITICAL)

### RCA: the mechanism is narrower and nastier than "no sender auth"

The audit said `accept_custody` has "no sender authentication in its head". Reading
it, there is no *notion* of sender identity to authenticate at that boundary: the
signature takes three caller-supplied `String`s and only ever bounds one of them.
`source_peer_id` and `destination_peer_id` were accepted verbatim, at any length,
and became storage-key material.

`destination_peer_id` is a key component, not just data:

```
destination_prefix(d) = "relay_custody_msg_" + d + "_"
message_key(d, c)     = destination_prefix(d) + c
```

Two concrete consequences, both provable from that arithmetic:

1. **Key aliasing.** `message_key("a", "b_c") == message_key("a_b", "c")`. Two
   different destinations can occupy the same storage key.
2. **Cross-destination scan leakage.** `destination_prefix("a")` is a byte prefix of
   `destination_prefix("a_b")`, so `pending_for_destination("a")` also returns records
   addressed to `"a_b"`.

Plus unbounded key growth: an arbitrarily long destination was stored and indexed as
an arbitrarily long key, and that key was scanned on the *first line* of ingestion
because `find_existing` built the prefix before anything else ran.

Retention: `grep -n "ttl\|expir\|retention\|max_age" relay_custody.rs` returned
nothing. Reclamation was size-only (`enforce_storage_pressure`) or delivery-only.
`MAX_PENDING_PER_DESTINATION = 10_000` is per destination and destinations were
unbounded, so the bound was not a bound.

### Fix landed

- `validate_custody_token(label, value, max)` — non-empty, <= 128 chars, ASCII, no
  control characters. Applied to `source_peer_id` and `relay_message_id`.
- `validate_custody_destination_identifier(value)` — the above **plus** the `_`
  separator ban, applied to `destination_peer_id`, because it is the key component.
  The ban is deliberately on the destination only: banning it on `relay_message_id`
  would have added no security (that field sits after the prefix) while risking real
  relay traffic.
- Self-relay (`source == destination`) rejected: a hop from X to X consumes storage
  under a key the sender also chooses and cannot deliver anything new.
- `CustodyState::Expired` (appended last so existing bincode variants keep their
  encoding) + `purge_expired_custody(max_age_ms) -> CustodyRetentionReport`, which
  writes a `custody_expired` audit transition per dropped record. `max_age_ms == 0`
  disables the sweep rather than expiring everything.
- CORRECTION (found by running the lifecycle, section 0 below): an earlier draft of
  this document said delivered records are "the delivery trail" and are retained.
  That is **wrong**. `mark_delivered` writes the transition and then REMOVES the
  record -- custody ends at handover. The delivery trail is the transition log,
  which the sweep never touches. The sweep's `Delivered` guard is defensive: a
  stored `Delivered` row means a crash between the transition write and the removal,
  or data from an older build, and leaving the one trace of it alone is safer than
  deleting it.
- `CUSTODY_DEFAULT_MAX_AGE_MS = 7 days`.
- **Wired** (rule 16): the swarm's 5-minute `backoff_prune_interval` tick now runs the
  sweep on `relay_custody_store`, and `IronCore::purge_expired_custody` exposes it for
  explicit operator use. The API has a live call site; it is not dead code.
- Ordering note: validation now runs *before* `find_existing` so a malformed
  destination cannot drive a scan prefix.

Tests: 4 new unit tests (`custody_ingestion_rejects_unbounded_and_malformed_identifiers`,
`custody_ingestion_rejects_self_relay`,
`custody_destination_prefix_must_not_alias_across_destinations`,
`custody_retention_expires_only_undelivered_records_past_the_window`). The third
asserts the aliasing arithmetic directly, then asserts ingestion refuses it, so the
rule cannot be "cleaned up" later without the test failing.

Plus 2 integration tests in `core/tests/test_trn04_custody_lifecycle.rs` that drive a
real sled-backed store across three sessions (open, reopen, reopen) and assert the
full lifecycle: accept -> dispatch -> deliver-removes-the-record, the abandoned
record expiring, the expiry transition persisting, the default window leaving fresh
records alone, and the ingestion guards refusing on a real store. The unit tests
cannot cover this: they use `in_memory()` and seed state through the private
`put_message`.

### Regression this work introduced, caught by the existing suite

My first edit to `accept_custody` **deleted the `find_existing` dedup short-circuit**
while inserting validation above it. The code compiled; the behaviour silently changed
from "one custody record per (destination, message_id)" to "a new record per call".
`custody_deduplicates_same_destination_and_message_id` caught it:

```
left:  "relay-msg-dedupe-1789685648929-0"
right: "relay-msg-dedupe-1789685648929-2"
```

Restored, and the full suite is green. Recorded because this is the exact failure
shape rule 16 warns about, and because a targeted test is what found it.

### Not claimed

No sender *cryptographic* authentication was added, and none is possible at this
boundary: the transport has already authenticated the immediate peer (noise), and the
store has no access to the envelope's signature at this point. What is now enforced is
bounded, well-formed, non-aliasing ingestion plus an age bound. Real cryptographic
sender binding belongs one layer up and is NOT part of this change.

---

## TRN-07 — Global relay budget enables network-wide DoS (HIGH)

### RCA: the defect is ordering plus a missing dimension

`swarm.rs` admission ladder, before this change:

1. blocked-peer check
2. cheap heuristics
3. **node-global hourly budget** — `relay_budget: u32 = 200`, hardcoded at `:3901`
   (native) and `:8183` (wasm), gated by `relay_count_this_hour >= relay_budget`
4. inflight dispatch cap
5. **per-peer token bucket** (`consume_peer_token`, 4/s refill, burst 20)

A per-peer dimension already existed — at step 5, *after* the global gate. So a single
peer could drain the entire hourly budget at step 3's expense, and every other peer
was then refused with `relay_budget_exhausted` for the remainder of the hour. One
connection produced a node-wide relay outage. The global counter is also in-memory
only and could not be configured from the CLI at all (only the Android adaptive path
calls `SetRelayBudget`), so on a cloud node the 200 was not merely a default, it was
final.

### Fix landed

- `relay_per_peer_budget(global) = max(global / 4, 25)`.
- Per-peer consumption of the same hourly window is tracked in
  `relay_counts_this_hour` and cleared on the hourly rollover alongside the global
  counter, in **both** the native and wasm loops (parity — a wasm node keeping the old
  rule would reintroduce exactly the defect TRN-03 was about).
- New refusal: `relay_peer_budget_exhausted`, logged with the peer's count and its
  share.
- Deliberately **not** emitted as an `AbuseSignalDetected`: the peer may be carrying
  legitimate mesh traffic, and inflating its spam score would punish delivery.

Tests: the ladder itself was not testable as written -- all four budget/shape gates
were inline `else if` conditions inside a ~10 000-line `select!` arm, so only the
divisor arithmetic could be exercised. The ladder is now a pure function
(`relay_admission`, with an explicit `RelayAdmission` outcome) called by BOTH the
native and wasm loops, which makes the ordering assertable and removes the second
copy of the rule that was the shape of the TRN-03 defect. 8 tests in
`relay_per_peer_budget_tests` now cover it, including the named scenario
`a_greedy_peer_is_throttled_while_others_still_relay` (the greedy peer is refused
at its share while a peer inside its share is admitted, with the node ceiling still
having room), the ordering where several gates would refuse, and that
`relay_budget == 0` still means "no budget enforced" rather than "refuse all
relaying" -- the semantics the original `relay_budget > 0 &&` guard encoded.

NOT COVERED: the stateful half -- that the swarm actually increments
`relay_counts_this_hour` on admission and clears it on the hourly rollover. That
needs two real swarms exchanging relay requests, which no existing test does
(`RelayRequest` appears in no integration test). The decision function is tested and
wired; the counter bookkeeping is verified by inspection only, and the rule-8
reviewer should look at exactly there.

### Trade-off, stated plainly

With the default 200/hr budget, one peer's ceiling drops from 200 to 50. On a 3-node
fleet carrying a handful of messages an hour that is not binding, and the share scales
with the budget, so the operator's knob still works: raising the budget raises every
peer's allowance. If a deployment genuinely needs one peer to carry more than a quarter
of node capacity, the budget is the wrong lever and this policy needs revisiting.

### GATE NOT SATISFIED

`core/src/transport/` is a rule-8 directory. This change is **not done** until an
adversarial APPROVE from a reviewer who did not author it is on file. I cannot
satisfy that gate myself, and I am not claiming it.

---

## AND-06 — Kotlin BigInteger Ed25519 math (HIGH)

### RCA: significantly larger than filed, and a prior attempt that stopped short

Filed as `PeerIdValidator.kt:112` / `DashboardViewModel.kt:41`. Measured now:

```
android/app/src/main/java/.../ui/viewmodels/ContactsViewModel.kt  14 BigInteger refs
android/app/src/main/java/.../ui/viewmodels/DashboardViewModel.kt 16
android/app/src/main/java/.../utils/PeerIdValidator.kt             7
android/app/src/main/java/.../utils/PeerKeyUtils.kt               13   <- not in the finding
```

Four hand-rolled copies of the same Ed25519 decompression + Legendre test, not one.

Branch `p1/curve-uniffi-kotlin-20260914` (`226a4ea4`) already tried this. Its commit
`61209629` "unify three divergent Ed25519 curve checks into PeerIdValidator" touched
**zero `.rs` and zero `.udl` files** — the branch name promised UniFFI and the diff
never reached Rust. So the Kotlin copies were consolidated and the doctrine violation
(Dashboard/Contacts/PeerKeyUtils still carrying their own math) is only partly
reduced, while the core half that would let the Kotlin go away was never built.

The ticket's acceptance path also names `android/.../security/PeerIdValidator.kt`,
which no longer exists; the file moved to `.../utils/`.

### Landed: the missing half

- `is_valid_public_key(hex_str: String) -> bool` in `core/src/lib.rs`, declared in the
  `api.udl` namespace next to `get_build_provenance`, delegating to
  `identity::keys::is_valid_public_key` so the FFI answer and the in-core answer cannot
  drift. Stateless, so it is usable during cold start when there is no `IronCore`.
- No generated file is hand-edited (rule 6): Kotlin bindings are produced at build time
  by the Gradle task at `android/app/build.gradle:218-224` into
  `core/target/generated-sources/uniffi/kotlin`, so the new function reaches Kotlin and
  Swift through normal regeneration.

### The core check was LAXER than the Kotlin it replaces -- found by running it

Exposing the function was not enough, and the difference was not theoretical. Driving
the exact vectors from `PeerIdValidatorCurveVectorTest.kt` (whose own doc comment
requires that "when the UniFFI path is wired on-device the same vectors must pass
against the core implementation") showed the core ACCEPTING three shapes the Kotlin
authority rejects:

| Input | Kotlin | core (before) |
|---|---|---|
| `y = 1`, sign bit set (non-canonical x = 0) | reject | **accept** |
| `y = p-1`, non-canonical sign bit | reject | **accept** |
| `y = p` (outside the field) | reject | **accept** |

`ed25519_dalek::VerifyingKey::from_bytes` decodes leniently, and this predicate is
what separates a 64-hex `public_key` from a 64-hex Blake3 `identity_id`. Roughly half
of all identity_ids decompress to *some* point (a fact this repo already documents in
`test_identity_id_is_not_valid_ed25519_point`), so a laxer test means MORE identity_ids
get misread as public keys -- the documented cause of "callers encrypt to a hash and
produce ciphertext nobody can decrypt". Migrating Android onto this function as it
stood would have LOOSENED validation on the platform.

Fixed in `core/src/identity/keys.rs`: `is_valid_public_key` now requires a canonically
encoded point, via `is_canonical_ed25519_encoding`, which enforces (1) the
sign-bit-masked `y < p`, and (2) no set sign bit when `x = 0` -- i.e. when `y^2 = 1`,
so `y == 1` or `y == p-1`, since there is no negative zero. Both rules are plain
byte comparisons against `p = 2^255 - 19`, so this added no dependency. The two
accepted forms in the table above (`y = 1` and `y = p-1` with sign bit clear) are
asserted as still accepted, so the fix is strictness and not over-rejection.

### Not landed, with the reason

The Kotlin side — deleting the four BigInteger implementations and delegating — is
**not** in this change. The blocker is a real contract question, not effort:
`PeerIdValidator`'s curve check is exercised by JVM unit tests
(`PeerIdValidatorCurveVectorTest`), and the JVM tier does not reliably load the native
library — `ReceiptUnificationTest.kt:53` documents
"load of libscmessenger_core that does not exist on the JVM test tier" as an accepted
condition there. Delegating blind would either break that lane or require a
JVM-safe fallback, which is a behavioural decision: does the adapter keep a Kotlin
fallback when the core is unavailable, and if so, is that fallback subject to the same
doctrine that banned it?

Next step is that decision, then a single migration. Estimated diff is mechanical once
decided (route all four call sites to `PeerIdValidator`, delete the math).

---

## SEC-03 — Unmaintained `sled` and stale `deny.toml` waivers (HIGH)

### RCA

- `sled = "0.34"` (`Cargo.toml:65`), consumed at `core/Cargo.toml:52`.
- Replacement is structurally cheap: `StorageBackend` is a trait with four
  implementations (`MemoryStorage`, `DegradedStorage`, `SledStorage`,
  `IndexedDbStorage`), so an engine swap is bounded behind one interface.
- "Stale waivers" is not one problem. `deny.toml` carries **13** ignores; the sled
  family is 4 (`RUSTSEC-2025-0141`, `-2025-0057`, `-2024-0384`, or the fourth
  per the file's comments), while the rest are libp2p/hickory (`-2026-0118`,
  `-2026-0119`), rustls-webpki (`-2026-0049`, `-0098`, `-0099`, `-0104`),
  quinn/rustls (`-2026-0285`), `if-watch`/netlink (`-2024-0436`) and
  `proc-macro-error2` (`-2026-0173`). Refreshing the sled entries alone would not
  shorten the list, and the non-sled entries have their own upstream status that I
  have not verified.

### Decision: escalated, not taken (rule 9)

Replacing the storage engine is a tech-stack change and a storage-format migration
touching every node's `/data`, including live ones. Rule 9 reserves that to the
operator, and rule 8 would additionally require an adversarial review of the store
change. **I changed nothing about the dependency.**

The operator needs to choose:

- **(a) Accept, dated and owned.** Renew the waivers with an explicit expiry and a
  named owner, and budget the migration for 0.5.0. Cheap now, keeps a known-unmaintained
  engine in a release. Note this is what the audit is objecting to, so doing it again
  needs to be a recorded decision rather than a default.
- **(b) Migrate behind the trait.** Pick a maintained engine (`redb`/`fjall` were
  mentioned as candidates; neither is in the tree today, so this is new dependency
  surface and needs the same rule-9 sign-off), keep `MemoryStorage` as the fallback,
  and treat the on-disk format change as a migration with a `DegradedStorage` path.

I recommend **(b) on a branch, not blocking the tag**, with **(a) as the interim only
if the tag cannot wait** — but the choice is the operator's, and the waivers as written
today have no expiry, which is the part that is unambiguously wrong regardless of
which option is chosen.

---

## New findings from this session

| ID | Severity | Finding |
|---|---|---|
| N-06 | HIGH (fixed here) | Custody storage-key namespace aliasing across destinations (`message_key("a","b_c") == message_key("a_b","c")`), plus a cross-destination `pending_for_destination` scan leak. Proven by test; closed by the TRN-04 destination rule. |
| N-07 | MED (process) | Two large unmanaged disk classes: `~/Documents/GitHub/.scm-shared-target` (22.22 GB, the documented shared warm cache) and emulator state (`~/.android/avd/scm_test_34.avd`, 6.5 GB). No guard reported either. See the disk policy change in the same branch. |
| N-08 | **HIGH (fixed here)** | `identity::keys::is_valid_public_key` accepted non-canonical Ed25519 encodings (`y >= p`, set sign bit on `x = 0`) that the Android validator rejects, so the UniFFI function was not behaviour-equivalent to the thing it must replace. Fixed with strict canonical decoding. This is the defect that would have shipped as "AND-06 done". |
| N-09 | MED (process) | A unit test seeded a `Delivered` custody record through the private `put_message`, asserting a state the real `mark_delivered` path never leaves behind. It passed while the production behaviour went unverified. Now covered by an integration test that drives the public API. |

## 0. Behavioural verification: what was actually run

Unit tests passing is not evidence of behaviour, and two of the four findings had
only been verified at the level of helper functions. So the real surfaces were driven:

```
cargo test -p scmessenger-core --test test_and06_public_key_vector_parity   -> 8 passed
cargo test -p scmessenger-core --test test_trn04_custody_lifecycle          -> 2 passed
cargo test -p scmessenger-core --lib                                        -> 1461 passed; 0 failed
cargo test -p scmessenger-core --lib -- relay_per_peer_budget_tests         -> 8 passed
```

What running them changed:

1. **AND-06 was not fixed, and would have shipped broken.** The first run was 4 of 8
   FAILED, every failure in the direction "core is more permissive". See N-08. Fixed,
   then 8/8.
2. **A claim of mine was false.** The lifecycle test failed on "the delivered record
   must still be present as the delivery trail". `mark_delivered` removes the record;
   the trail is the transition log. Test corrected, document corrected, and the
   misleading unit test re-documented (N-09).
3. **The TRN-07 ladder was untestable**, so it was extracted into a pure function
   used by both loops, and the ordering is now asserted -- including the exact
   scenario the finding names.

Not verified behaviourally, and not claimed: the swarm's own counter bookkeeping
(increment on admission, clear on rollover), because no test in this repo exchanges
`RelayRequest` over a real swarm.

## Status against the 0.4.0 blocking set

| Finding | Status |
|---|---|
| TRN-04 | Fixed, unit + lifecycle tested against a persistent store, wired. Awaits rule-8 review only if the reviewer considers `store/` in scope. |
| TRN-07 | Fixed, ladder extracted and tested, wired in native and wasm. **Rule-8 adversarial APPROVE outstanding — this is a transport-directory change.** Counter bookkeeping is inspection-only; see the TRN-07 section. |
| AND-06 | Core half fixed AND corrected (N-08: the function was laxer than Kotlin and would have loosened Android validation on migration). 8/8 vector parity. The Kotlin migration is still blocked on the cold-start/JVM contract decision — but it is no longer blocked on the core being wrong, which it was. |
| SEC-03 | Escalated with a recommendation. No code or dependency change made. |

The 0.4.0 tag should not be cut on the strength of this document alone: TRN-07's gate is
outstanding, AND-06's Kotlin half is open, and SEC-03 needs an operator decision.
