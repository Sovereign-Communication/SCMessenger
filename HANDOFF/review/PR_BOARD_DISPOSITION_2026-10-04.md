# Open-PR board disposition and merge plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Date: 2026-10-04
Board size at enumeration: **69 open PRs** (11 drafts).
Base: `origin/main` = `051dbb5b9cd900669f11ecf5c40c63a38c2e86a6`
Train: `integrate/train-20261004` = `8ac7bc6a8`, **211 ahead of main, 0 behind**
-- a clean fast-forward, no conflict resolution required to land it.

---

## 1. How the board was classified

Three independent tests, because a PR can be redundant in one sense and novel
in another:

1. **Ancestry** -- `git merge-base --is-ancestor <head> <train>`, on the exit
   code. 47 of 69 PR heads are already commits inside the train.
2. **Blob identity** -- compare each changed file's blob SHA between the PR head
   and the train. Used with hash comparison, NOT text, because reading binary
   files as text silently returns empty on both sides and manufactures false
   "identical" verdicts.
3. **Per-commit patch-id** -- `git show <c> | git patch-id --stable`, each commit
   of each PR, tested for membership in the train's patch-id set. This is the
   test that answers "is this work already done".

> A caution recorded because it cost a wrong result: computing patch-id over a
> whole-branch `git diff merge-base..head` yields ONE id for the entire squash,
> which will never match any single commit. That test reports every multi-commit
> PR as "novel" and is worthless. It must be done per commit.

## 2. Disposition

| Class | Count | PRs | Action |
|---|---|---|---|
| Already in the train (by commit) | 47 | incl. #451 | nothing; #451 lands them |
| Fully covered by the train (by patch-id) | 9 | #316 #357 #367 #376 #386 #388 #412 #415 #425 | **close as superseded** once #451 merges |
| Partially covered | 2 | #411 (19/24 in), #413 (19/25 in) | rebase, merge only the novel commits |
| Zero covered -- all novel | 11 | #178 #208 #209 #211 #216 #220 #351 #359 #361 #364 #372 | rebase onto main, then review and merge |

### Why the 9 must be closed, not merged

#316 and #357 fail the required `Handoff ownership scope` check. The job log
names three files with `found 0 begin markers`:
`V040_OUTBOX_RETRY_DIAGNOSIS_2026-09-19.md`, `TRAIN_STATUS_2026-09-21.md`,
`AND06_KOTLIN_CUTOVER_PREREQUISITE_2026-09-21.md`. Those three files **do not
exist on `origin/main` at all**, and the train already carries metadata-fixed
versions of all three. Merging #316/#357 first would land the metadata-less
copies that the train merge would then have to undo.

---

## 3. The gated transport cluster -- a CRITICAL five-way conflict

An independent, non-author Opus 5.5 review (read-only, isolated session, scoped
disclosure recorded in
`HANDOFF/review/RULE8_OPUS_TRANSPORT_CLUSTER_2026-10-04.md`) returned
**VERDICT: BLOCK** with one CRITICAL. I verified it against source rather than
accepting it, and **corrected part of it**.

### C1 CONFIRMED -- the caps disagree

All five PRs add `core/src/transport/per_peer_cap.rs` and wire it into
`behaviour.rs`. Verified values:

| Source | Admission cap | How it reaches libp2p |
|---|---|---|
| **train** | `MAX_ESTABLISHED_PER_PEER = 16` (`behaviour.rs:50`), floor 16, two compile-time asserts (`:61-68`) | `behaviour.rs:572` `Some(MAX_ESTABLISHED_PER_PEER)` |
| #372 `2cb046cf8` | `ADMISSION_MAX_ESTABLISHED_PER_PEER = 8` | `behaviour.rs:545-546` reads the module constant |
| #359 `3f41005d1` | `= 16` | same |
| #361 `63f4a7d73` | `= 16` | same |
| #364 `45b0f8b8a` | `= 16` | same |
| #413 `5c80035f4` | **no `per_peer_cap.rs` at all** | `behaviour.rs:532` `with_max_established_per_peer(Some(4))` -- a **literal** |

So: **8 vs 16 vs 4**, against a train that enforces a floor of 16. **#372's
value of 8 cannot satisfy the train's floor assertion** and cannot land as
written.

The train has **no** `per_peer_cap.rs`, so all four PRs are add/add conflicts
against each other *and* a new-file conflict against the train's inline
constant.

### CORRECTION to the review's stated failure mode

The review claimed "no test ties the module constant to the builder call" and
that a resolution keeping 16 at the call site with #372's module would pass
#372's tests. **That is wrong for four of the five PRs.** In #372, #359, #361
and #364 the call site reads the constant directly:

```rust
.with_max_established_per_peer(Some(
    super::per_peer_cap::ADMISSION_MAX_ESTABLISHED_PER_PEER,
))
```

The constant *is* consulted on the hot path. Those four are fine.

**The real hazard is #413**, and it is worse than what was reported. #413 hardcodes
`Some(4)`, bypassing the module entirely, and its `behaviour.rs` **predates and
therefore lacks both compile-time floor assertions** -- there is no
`MAX_ESTABLISHED_PER_PEER_FLOOR` in it at all. If #413's `behaviour.rs` wins any
part of the rebase, the resolved file can silently enforce 4 while carrying no
floor guard, and no test in the cluster asserts the value handed to libp2p.

### Resolution

Adopt **16**, from the train, and make the cluster conform to it:

1. #359, #361, #364 agree at 16. Rebase them first, in that order; their
   `per_peer_cap.rs` files are near-identical.
2. #372 must move 8 -> 16 or be dropped. Its distinct value is what makes it
   conflict. Its other content (the `release_path`/`release_peer` map-drain fix
   for finding M1, and the wasm parity) is worth keeping -- so rebase it and
   set 16, rather than discarding the PR.
3. #413 must be converted to reference the module constant instead of the `Some(4)`
   literal, and its `behaviour.rs` must retain the train's floor assertions.

### Also found by the review, unverified here -- do not treat as confirmed

- **M1** (#359/#361/#364): `path_last_activity` leaks one entry per full
  disconnect; `ConnectionId` is monotonic, so a long-lived relay grows
  unbounded. #372 fixes this with `release_path`/`release_peer`.
- **M2** (#372): `add_relay` keeps only the 8 most recent relays and is called
  for every identified peer, so attacker-controlled ids can displace legitimate
  relays from the circuit ladder. Traffic stays E2E-encrypted, but the attacker
  sees who talks to whom and can drop.
- **M3**: `max_pending_incoming` reportedly set nowhere, so pending inbound
  handshakes may be unbounded -- the actual fd-exhaustion path, independent of
  the per-peer cap.

These are recorded as claims from the review, scoped to what it read. They were
**not** re-derived in this pass.

---

## 4. Merge order

1. **#451 -> main.** Fast-forward, no conflicts. Lands 47 PRs' worth of work and
   clears the other 9 for closure. **Gated on the Rule-8 decision** (see below).
2. **Close the 9 superseded PRs**, with the supersession reason in the close
   comment. Do this after step 1 so the reason is verifiable.
3. **Rebase the gated transport cluster** onto main in the order #359, #361,
   #364, then #372 (retuned to 16), then #413 (de-literalised). Each rebase
   needs CI green on its own before the next.
4. **Rebase the non-gated remainder**: #178, #208, #209, #211, #216, #220, #351.
   #209/#216/#220 are near-duplicates of each other (76/76/77 files, 64 divergent
   each) and need an explicit keep/drop decision first, not a blind rebase.
5. **#411, #413**: rebase, then merge only the novel commits (5 and 6
   respectively); the rest is already in the train.

## 5. What still blocks step 1

`scripts/pr_scope.sh 451` returns STOP. The Rule-8 requirement is unchanged: no
merge of a change touching `core/src/{crypto,transport,routing,privacy}` without
a recorded non-author review. There is now a filed non-author APPROVE for
`a34c60017` alone (scope-limited), and this cluster review is BLOCK. The
transport cluster review covers the five conflicting PRs, **not** the train's own
already-merged transport commits.

Rule-8 remains **unmet** for the train as a whole. Options unchanged: fund an
independent review of the train's current head, have a non-author human review
it, or record an explicit operator override. That is an operator decision
(AGENTS.md rule 9).

## 6. Harness `panel_verify` -- refused, and why

`spend_status` reports `remaining_credit_usd: 4.9995` of a $5.00 monthly JEV
credit with 21 calls made. `panel_verify` nevertheless refuses:

```
worst-case estimate $0.226094 exceeds remaining ceiling $0.050000
(outstanding reservations: $0.000000; phase=attempt). Refusing.
```

The context was accepted; the blocker is a **session** budget ceiling of $0.05
that `spend_status` does not surface. The $0.226 estimate is fixed lane
overhead, not prompt size -- a 1400-character prompt still estimated $0.226 in
an earlier attempt -- so no prompt size fits. Reported as a configuration
discrepancy rather than retried further.

This blocks nothing: `docs/rules/SECURITY_PROTOCOL.md:98` states a multi-model
panel is **not** the Rule-8 gate. Panels find candidate findings; a recorded
non-author verdict is the gate.