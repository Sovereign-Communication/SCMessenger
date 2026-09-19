# V040 candidate merge conflict brief -- #272 into final tree

Date: 2026-09-04 (~07:30Z)
Status: DECISION REQUESTED (not resolved, not merged)
Author: CEO-seat execution (freebuff lane)

## Situation

The landing sequence has merged everything except the architecture candidate.
Main is at 45ab59f9 (contains #264, #269, #271, #268, #270, #275, #267).
The candidate branch (PR #272) is at 48672b18 (contains #273, #274).
`gh pr update-branch 272` FAILS: "Cannot update PR branch due to conflicts."
The true final tree (and hence the exact final-tree artifacts + the 3-node test
the CEO chose to wait for) is BLOCKED on resolving this merge.

Isolated diagnosis worktree: `tmp/cand-merge/` (off 48672b18, `git merge origin/main`
in progress -- merge state preserved there for the resolver). Shared refs untouched.

## Conflict map: 5 files, 13 hunks

### Mechanical / equivalent-expression (safe to fold, no decision needed)

- `core/src/transport/swarm.rs` h3 (~5683): candidate
  `bound_addresses.iter().filter_map(listen_port_from_bound_addr)` vs main
  `listen_ports_from_multiaddrs(&bound_addresses)` -- same port list, two helpers.
- `core/src/transport/swarm.rs` h4 (~8447): main-only comment insert (wasm has no
  ledger / F-DHT note) -- keep.
- `core/src/transport/observation.rs` h3 (~191): candidate
  `sort_by_key((Reverse(*count), *address))` vs main inline
  `sort_by(b.1.cmp(a.1).then_with(...))` -- same ordering (count desc, address asc
  deterministic tie-break). Candidate form is cleaner; either is correct.
- `core/src/transport/observation.rs` h1 (~6): imports -- union
  (candidate `std::cmp::Reverse`, main `libp2p::multiaddr::Protocol`; both needed).
- `core/src/routing/local.rs` h2 (~232): candidate `Self::sort_by_reliability(&mut peers)`
  vs main inline `sort_by(partial_cmp)` -- same F7-C fix, candidate factored into the
  shared helper. Take candidate (the architecture's single-owner ordering).

### Structural refactor collision (main's guard content must land, but re-homed)

- `core/src/transport/swarm.rs` h1 (~4333) + h2 (~5502): main's #270 defense-in-depth
  publication guard (never advertise a port we do not listen on; refuse on empty) is
  INLINE at two call sites on main; the candidate replaced both with
  `sync_external_address(&mut swarm, &address_observer)` (single call). VERIFIED: the
  candidate's helper does NOT contain the listen-port guard -- it is a naive
  consensus-primary sync. The #270 guard content must be folded INTO the candidate
  helper (or kept at the call sites) so the final tree keeps #270's P0 publication
  protection.

### DESIGN DECISIONS (approved-reviewed behavior collides; NOT a merge preference)

#### D1 -- empty listen-set admission policy (observation.rs h2 ~64, h4 ~375, h5 ~410)

- Candidate (qwen-APPROVED architecture): empty set FAILS CLOSED -- every observation
  rejected; deliberately tested (`empty_listen_port_set_fails_closed`,
  `non_listen_port_observations_are_rejected`, `removing_a_listen_port_removes_its_observations`);
  `set_listen_ports` also retain-prunes stored observations.
- Main #270 (T14 P0, reviewed): empty set ACCEPTS ALL -- "browser/wasm transport has
  no listeners"; deliberately tested (`test_ephemeral_source_port_observation_is_dropped`,
  `test_consensus_excludes_ephemeral_port_even_when_more_common`,
  `test_set_listen_ports_re_filters_stored_observations`); the filter is applied at
  record time only when the set is non-empty.
- Both policies cannot survive. The deciding fact: does the candidate's WASM path
  record observations while its listen set is empty? Candidate calls
  `set_listen_ports` at 5+ sites incl. the wasm cfg region (swarm.rs 6283/6299/8634)
  and `record_observation` in wasm paths -- needs the architecture owner to state
  intent, then the loser's tests are removed/adjusted deliberately.
- Recommendation: resolve toward #270's wasm carve-out IF the candidate's wasm build
  observes addresses with no listener; otherwise candidate's fail-closed wins. Must be
  decided with evidence, then the surviving test set kept.

#### D2 -- routing hint width (local.rs h1 ~208, optimized_engine.rs h1 ~38, iron_core.rs h1 ~5043)

- Candidate architecture: `local_hint: [u8; 4]`, `OptimizedRoutingEngine::new(local_id,
  local_hint: [u8; 4])`, NO dead parity copies on the engine (single-owner doctrine),
  `peers_for_hint(&[u8; 4])`, tests call `new([0u8; 32], [0u8; 4])`.
- Main #268 (T13/B, reviewed): hint widened to `[u8; 8]`; engine carries
  `#[allow(dead_code)]` parity fields `local_id`/`local_hint: [u8; 8]` ("retained for
  parity with base_engine's copy"), tests call `new([0u8; 32], [0u8; 8])`.
- Candidate predates #268. If the final tree keeps the candidate's [u8;4], #268's
  widen is effectively reverted in-tree -- the candidate's unified
  `sort_by_reliability` already delivers the F7-C security property #268 also fixed,
  so the widen may be redundant IN THE CANDIDATE ARCHITECTURE; but if the hint width
  is wire-visible (messages/ledger), reverting is a protocol decision that #268's
  reviewers approved forward. If the final tree carries [u8;8], the candidate's
  surface (peers_for_hint, engine ctor, tests) is mechanically widened and main's
  dead parity fields can be dropped (D3).
- Recommendation: carry #268's [u8;8] forward and drop the dead parity fields --
  unless the architecture owner confirms the widen is wire-format-breaking AND
  superseded by the candidate's ordering design, in which case reverting must be
  declared explicitly with review, never silent.

#### D3 -- engine parity fields (optimized_engine.rs h1 ~38)

- Main added `#[allow(dead_code)] local_id/local_hint` copies on OptimizedRoutingEngine
  "for parity, not read directly today". Candidate's architecture removed them
  (single owner of state -- the AGENTS.md doctrine). Low risk.
- Recommendation: candidate wins; main's parity fields are deleted in the final tree.
  Only reason to keep: some main-side code reads them directly (none found -- both
  marked dead_code on main).

## Resolver options

1. CEO-seat execution (this tab) authors the resolution in `tmp/cand-merge/`,
   per-hunk decisions above, after the D1 wasm-evidence check and CEO go on D1/D2.
2. CTO lane authors (owns the candidate architecture; first-hand D1/D2 intent).
   Slower cadence.

## Gates after the resolution commit (either resolver)

- NEW CONTENT in core/src/transport + core/src/routing (Rule-8 zone): an adversarial
  review of the resolution delta must be on file before merge -- qwen lane (non-author)
  reviewing the merge-resolution commit vs 48672b18 (or the reviewer of record).
- CI at the new head must be fully green (the pinned cargo-deny Lint fix #275 is in
  main, so the tinyvec infra failure is dead).
- Exact final-tree artifacts at the new head (wincli/APK via PR CI on the updated
  branch -- branch head == merge ref; docker image via CTO `workflow_dispatch`).
- Focused 3-node test on the true final tree (CEO decision: wait for true final tree).
- Then #272 to main under the standing CEO go.

## Not decided here

- D1 and D2 policy choices (above) -- CEO/architecture-owner decision requested.
- Executor (this tab vs CTO).
- The Rule-8 delta review is mandatory regardless; no waiver is being sought.
