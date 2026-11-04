# Batch D combined head — Rule-8 review and Amendment 5 comparison

Date: 2026-11-04
Combined head: `cd7c4514c130e65ccc4398fdd3e305164a4e911a`
Base: main `a6a0b0d73d7720da6c0218180497f1c48bd20a8d` (post-#537)
PR: #660

## 1. What the combined head contains

Cars merged in batch order, each as its own merge commit so the batch history
keeps the landing order:

| Car | PR | Merge commit |
|---|---|---|
| 1/4 | #550 StoreAndCarry outbox retry, chat receipts on dup, network log noise | `b00a95808` |
| 2/4 | #556 wire network-change signal into core (#523) | `a1feec389` |
| 3/4 | #554 canonical recipient hint and identify hint binding | `ececa3a1ab` |
| 4/4 | #509 seeding security hardening (DnsPolicy::Reject, NetworkMode::Public) | `cd7c4514c` |

Scope: 23 files, +2318 / -254. No build artifacts, no secrets, no generated
bindings.

Gated paths (AGENTS.md rule 8) touched by the batch: `core/src/routing/local.rs`,
`core/src/routing/optimized_engine.rs`, `core/src/transport/swarm.rs`, all from
car 3/4. Car 1/4 touches `core/src/iron_core.rs` and `core/src/store/outbox.rs`,
car 2/4 touches `core/src/mobile_bridge.rs`, car 4/4 touches
`core/src/mobile_bridge.rs`; none of those are rule-8 directories.

## 2. Conflict resolution (car 2/4)

#550 and #556 both edit `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt`.
Two hand-resolved hunks, both additive:

1. In `updateNetworkType`: kept #550's `val changed = shouldLogNetworkTypeTransition(previousType, newType)`
   (the gate for the duplicated "X -> X" logging this car fixes) and kept
   #556's `lastPhysicalType` bookkeeping plus the `notifyNetworkChange(newType)`
   call. Neither can be dropped: `changed` is read by both logging branches,
   and the core bridge only sees changes through `notifyNetworkChange`.
2. In the top-level declarations: kept `internal fun shouldLogNetworkTypeTransition`
   (#550) and `object NetworkAvailability` (#556). `NetworkAvailability.flagsFor`
   is called by `notifyNetworkChange` in the same file, so omitting it would not
   compile.

`wiring-audit/FINDINGS.md` conflicted only in its re-derived Kotlin file count
(198 on the car-1/4 side, 201 on car-2/4). The combined tree counts 202 files
under `android/app/src`, re-derived with `find android/app/src -name "*.kt" | wc -l`;
that is the number the document now records. `MeshRepository.getMeshService`
is at line 4384 on the combined tree, matching the car-2/4 record; the car-1/4
side's 4373 is pre-#556.

## 3. Rule-8 review of the gated content

The gated files at this head are byte-identical to car 3/4's reviewed head.
Verified by object hash, not by eye:

| File | #554 head | Combined head |
|---|---|---|
| `core/src/routing/local.rs` | `fcd2d7b39` | `fcd2d7b39` SAME |
| `core/src/routing/optimized_engine.rs` | `e1d421750` | `e1d421750` SAME |
| `core/src/transport/swarm.rs` | `b02c58e75` | `b02c58e75` SAME |
| `core/src/identity/keys.rs` | `d636c94d7` | `d636c94d7` SAME |

`git diff --name-only 2677ea38d7..origin/main -- core/src/{crypto,transport,routing,privacy} vendor/`
is empty: main introduced no gated changes between the review base and this
head, so no unreviewed gated code arrives through the base.

Car 3/4 carries independent non-author Rule-8 review on its own PR, including a
full adversarial review signed off on delta `4bc650e795 -> 5be2e8a20d`
(current #554 head `5b3efc47`) and repeated delta SIGN-OFFs at `93d5d2155..ebe6c7e80`
and `4bc650e79`. Because the gated blobs are unchanged at the combined head,
that verdict applies to the same bytes here; it is not being reissued or
diluted by this document.

Substance checked while reading those bytes:

- **Hint derivation is one function.** `routing_hint_from_public_key` is
  `blake3(raw 32-byte key)[0..8]`. The inline expressions it replaces hashed
  the same raw 32 bytes, so this is a single-source refactor, not a wire
  change; the 4-byte envelope hint is the prefix of the same digest. The
  documented failure mode (hashing the 64-char hex string) is called out at the
  definition, and every call site passes `[u8; 32]`.
- **Untrusted announces are bounded before the routing lock.**
  `IronCore::routing_update_peer_hints` validates the peer id, parses each hint
  to `[u8; 8]`, dedupes, and stops at `LocalCell::max_peer_hint_budget()` (128)
  before `routing_engine.write()`, so lock hold time does not scale with the
  caller's vector. The peer's direct hint is injected if missing, so an
  announcement cannot make a connected peer unroutable.
- **The local table has per-peer and global budgets.** Per peer:
  `8 + 120 * reliability` (floor 8, max 128). Global: `max(1, peers) * 64`,
  enforced down to a 90% low-water mark. Eviction ranks lowest reliability
  first, then oldest announce, then peer id, then hint, so the victim set is
  deterministic; `select_nth_unstable_by` is used only to partition, and every
  selected victim is removed, so order inside the victim set is irrelevant.
- **Direct hints are exempt from eviction, TTL expiry and lookup staleness.**
  They are derived from the key, never from an announcement, and
  `index_insert`/`index_remove` do not count them against `announced_total`.
- **Announce rate limit is reliability-derived** (`1 + (1 - r) * 30` seconds)
  and a rate-limited announce is coalesced into `pending_announce`, bounded by
  one entry per known peer, applied on `tick`. Unknown peers are ignored: an
  announcement cannot create a peer.
- **Identify wiring goes through the validated ingress.** The swarm arm now
  calls `IronCore::routing_peer_seen` (transport length bound, peer id parsed
  to 32 bytes, all-zero rejected, fail closed) instead of writing the engine
  directly, and `bind_direct_hint` is reachable only from `peer_seen`. The
  removed `transport_type_to_routing_transport` had no remaining callers.
- **Index consistency is asserted, not assumed.** The tests check that
  `reachable_hints` and the index describe the same set, that `announced_total`
  matches the non-direct count, that a 100k-hint flood stays inside both
  budgets, and that lookup is index-bound for an evicted hint.

Residual notes, recorded rather than fixed (none is a landing blocker):

- `LocalCell::update_peer_hints` preserves the direct hint only because
  `IronCore` injects it; a future direct caller of `LocalCell` could drop it.
  Today the only production caller is `IronCore`, and the swarm path uses
  `bind_direct_hint`.
- The rate-limit window uses unix seconds and a backwards clock resets the
  window by design. An attacker cannot set the local clock.
- The rule-8 verdict carried here belongs to the non-author reviewer named on
  #554. This document is the integration check for this head, not a substitute
  verdict, and it must not be cited as one.

## 4. Amendment 5 comparison

- **Batch inventory.** Amendment 4/5 place #550, #556, #554, #509 in batch D
  (core fixes touching routing and transport). The combined branch contains
  exactly those four, in that order, nothing else.
- **mobile.yml.** ERRATA 1 to Amendment 6 records that `mobile.yml` does fire on
  `pull_request` despite Amendment 6's text; what matters is that the Android
  jobs executed rather than skipped. On this head, run `41770899776` shows
  `Android JVM Unit Tests` and `Android Debug APK` IN_PROGRESS and
  `Android Wiring Gate` SUCCESS, so the jobs are executing on the combined
  head; the skip-vs-execute check is satisfied by execution, and the run's
  final conclusions are recorded separately when they land.
- **security.yml.** Scheduled or manual only, so it is dispatched against this
  branch rather than inherited from the PR checks, per Amendment 5's
  "dispatch security.yml on batch D and E heads".
- **Conflicts inside a batch.** Amendment 5 requires members that conflict in
  the same file to be merged in dependency order with the hand-resolved hunks
  listed; section 2 of this document is that list. Amendment 6's role boundary
  says Landing does not hand-resolve conflicts, and this lane both authored the
  resolution and writes this review, so the resolution hunks still need a
  non-author read. That is recorded as the open item below rather than treated
  as cleared.
- **Stacked PRs.** Batch E stacking (#521 then #599) is not part of this batch.
- **Red cars.** No car in this batch is red: each member's required contexts
  were success on its own head before this branch was built, and this head's
  checks are listed on the PR.
- **Release ordering.** Amendment 5's tag-after-gate ordering is not reached by
  batch D; no tag or release action is taken here.

## 5. Open item before the four land

The conflict resolution in section 2 was authored by this lane, so under
Amendment 6 it needs a non-author check of those two hunks. The gated-file
content does not need a new verdict: it is byte-identical to the head already
signed off by a non-author reviewer. Landing order remains 1/4, 2/4, 3/4, 4/4,
executed by merging this batch PR and then confirming each member reads merged
on its own PR.
