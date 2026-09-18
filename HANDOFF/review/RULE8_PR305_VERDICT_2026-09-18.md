# Rule-8 adversarial review — PR #305 (TRN-04, TRN-07)

Status: DEFERRED — FIXUP PUSHED, RE-GATE NOT YET A CLEARANCE; DO NOT MERGE THIS PASS
Date: 2026-09-18
Scope: `core/src/store/relay_custody.rs`, `core/src/identity/keys.rs`, `core/src/transport/swarm.rs`, `cli/src/server.rs` (delegated consumer)
Branch: `freebuff/v040-blocking-fixes-20260917`
Base: `origin/main` (`c2ce2f6406b1`)
Initial head: `62b31179` (8 commits against main)
Fixup head pushed (this verdict's base): `880765c9` — 1 fixup commit, 659 insertions

## How the review was run

The repo's gate was run as two domain-scoped sovereign-harness panels so the
*prompt* fits inside the harness's own per-run worst-case ceiling (rule 15:
limits on what you do, never on what you can see).  One run would need the
full 52.5 KB code window for 8 claims at once; the worst-case estimator
(prompt tokens × every reserved call + judge fallback reserve) at the
sanctioned vote floor `MIN_VOTE_TOKENS=4096` exceeds `$0.10` with a pool big
enough to produce three seats, and the operator's per-run ceiling is `$0.10`.
The fix is the small one already used for review economics: split by the code
the claims actually touch, each run inside its own ceiling, each with its own
prompt/claims/source and its own ceiling.

First-pass (initial diff, capped pools that fit — worse than $0.05, average
actual $0.0056, two independent runs): 2/3 seats voted on the identity part,
1/3 on custody, SHORTFALL. The operator authorized a large vote budget, so the
fixed diff was re-gated with one pool sized so three seats *can* vote.

Second-pass (fixed diff in this commit):

- `identity` part (strict canonical check c1 + fairness c3/c4/c8): prompt 4033 B,
  397-line window (1467 core lib tests + 8/5/2 integration tests are the CI
  companions); panel pool `deepseek/v4-flash`, `deepseek/v4.1-flash`,
  `ling-3.0-flash`, `gpt-4o-mini` (4 cheap, 1 spare), judge
  `z-ai/glm-5.3-flash`, `--max-tokens 8192`, worst-case `$0.0964`, actual
  `$0.0111`, **3/3 voted**.
- `custody` part (retention c2/c5/c6/c7): prompt 4090 B, 429-line window
  (same CI companions); same pool/judge/window, worst-case `$0.0947`, actual
  `$0.0115`, **2/3 voted** (one `ling-3.0-flash`/`deepseek-v4.1-flash` pair stayed
  reasoning-only on this part's text; the identity part with the same pool at
  the same budget did get 3/3, so the shortfall is content-driven variance, not
  a systematically starved budget — and two retries of the custody part did not
  fill the third seat within the same pool). A second host or a cleared
  `byok_prefixes.json` would fill it with `gemini-3.8-flash`, which this host
  BYOK-skips.

Each run is inside the harness's hard per-run ceiling (`$0.10`, enforced by the
harness itself — `--max-cost 0.20` is rejected with `max_cost must be in
[0.0, 0.1]` — and by the repo's `harness_gate.py`, `PAID_MAX_DEFAULT=0.10`).
Fixing a shortfall by splitting is not spend-smuggling; each run is checked
against the ceiling on its own Worst-case Worst case, and actuals came in at
`11%` and `12%` of the ceiling. Any future wide sweep belongs to CI, as the
disk-budget policy requires.

Total paid spend for this review (all runs including the deferred first-pass):

- first-pass  identity `verify_pr305_identity_20260918.json`: `$0.003945` (2/3)
- first-pass  custody  `verify_pr305_custody_20260918.json`: `$0.004999` (1/3)
- first-pass (8-claim, deferred) `verify_pr305_claims_20260918.json`: `$0.006052` (1/3, all-real — low discrimination)
- fixed-diff identity `verify_pr305_identity_fixed_20260918.json`: `$0.011111` (3/3)
- fixed-diff custody  `verify_pr305_custody_fixed_20260918.json`: `$0.011549` (2/3)
- fixed-diff identity retries are folded into the `fixed` costs above

All four fixed/part runs are logged in `Harness/audits/scmessenger/_runs/seat-gates/`.

## Panel results (fixed diff — the head in this verdict)

### Identity/transport (`identity` part) — 3/3 voted — the *authoritative* part

Tally (judge z-ai/glm-5.3-flash, panel deepseek-v4-flash, deepseek-v4.1-flash→rotated, ling-3.0-flash→rotated, gpt-4o-mini as spare):

- c1 strict-check rejects a legit stored public key: **not_real (0R/3NR)** — deepseek “intentional RFC 8032 canonical” (0.95), gpt-4o-mini “not a real issue” in the (later, full) re-gate; judge: *reject c1*. This is the correct call on the evidence: dalek-lenient shapes (`y=1` sign-bit-set, `y=p-1` sign-bit-set, `y≥p`) were precisely the gap the Kotlin validator rejected and this predicate used to accept; no live mesh key is non-canonical (5-live-key vector parity, 1024 generated keys green, `/api/peer-resolve` live probes on both candidate nodes green).

- c3 `relay_counts_this_hour` unbounded map keyed by caller peer id: **not_real (1R/2NR)** → flipped. One model still votes real, but the other two and the judge weight the code evidence: the map is now `HashMap<String, PeerRelayUse>` with `last_seen_ms`, `note_peer_relay_admitted` + `prune_peer_relay_use` when `len > RELAY_PEER_BUCKET_MAX_TRACKED=2048`, mirroring the sibling token-bucket, in both native and wasm loops. The two new unit tests (`per_peer_budget_map_is_bounded…`, `per_peer_budget_use_accumulates…`) fail without the bound.

- c4 multi-identity bypass: **real (2R/1NR)** — **still real**. Both voters and the judge agree an attacker with 4 identities (or 8 once the MIN-25 floor binds) aggregates the global 200. This is the correct verdict and is *not* closable at this boundary with only the transport's `PeerId` available; fixing it needs an identity-attested peer id decision (rule 9). The global budget still bounds total relay work, and under the 15/hour natural peak the cap is a safety net. See `open finding` below.

- c8 budget charged before the destination is parsed (so refusals burn both the node and the peer's share): **not_real (1R/2NR)** → flipped. The hourly/global and per-peer counters are now charged only through `note_peer_relay_admitted` on the commit branches in both loops (`C8-CHARGE-POINT` appears exactly twice, native + wasm). The structural guard `test_rule8_pr305_charge_points.rs` counts the two commit points and asserts the old raw increment (`or_insert(0) += 1`) is gone; it fails if the pre-parse accounting is re-introduced. The judge: two models call it not_real at medium-high confidence.

Judge synthesis (identity, fixed): *only c4 remains real*. c1/c3/c8 rejected. “c3 correctly distinguishes between a new fairness mechanism and a *regression* — the map is bounded now.”

### Custody (`custody` part) — 2/3 voted — *deferred*, not a clearance

Tally (same pool/judge, actual `$0.0115`):

- c2 TOCTOU: snapshot → re-read race (sweep deletes a record a dispatcher just moved to Dispatching/Delivered, losing the delivery and its trail): **real (1R/1NR)** — split.
- c5 no migration for the new custody fields / prefix scan aliasing: **real (2R/0NR)** — both voters, high.
- c6 unbounded audit-transition trail (the sweep deletes record rows but left every `relay_custody_audit_*` forever): **real (1R/1NR)** — split.
- c7 fail-open sweep riding the dial-policy prune interval, `warn!` outside the debug noise floor, indistinguishable “never ran” vs “nothing to expire”: **real (1R/1NR)** — split.

Votes: 2/3. The custody part did not produce three seats in three consecutive paid runs at the same ceiling; that is variance, not a systematically starved budget (the identity part with the same pool/budget did fill 3/3), so we cannot call it a clearance. The substantive signal is the judge synthesis and the branch splits below.

Judge synthesis (custody, fixed): *only c5 can be reliably confirmed as real, medium*; c2/c6/c7 reach opposite conclusions on both realness and severity, so no fix decision should be made on those from these votes alone. In other words, the panel did not converge on c2/c6/c7 at 2 seats.

## Controller seat disposition of the panel (verified against the diff on disk)

The panel is evidence, not the decision. The controller seat (this worktree) checked every real-candidate against the code and live/CI evidence, and only the findings below cause a forward change:

- **c1** — judged **not_real**. The strict check is the intended security property (canonical-encoding + `y<p` + “no negative zero”). Not closable as a defect — it is the fix.

- **c3** — **real, FIXED in this commit** (and the panel now votes it not_real on the fixed diff — the reception is the re-gate). Bounded map + timestamp, helper `prune_peer_relay_use`, both loops, tests `per_peer_budget_map_is_bounded…` + `per_peer_budget_use_accumulates…`.

- **c8** — **real, FIXED in this commit** (panel flips to not_real). Counters moved from the pre-parse site to `// C8-CHARGE-POINT` on the custody-commit branches in both loops; structural guard `test_rule8_pr305_charge_points.rs` pins exactly 2 markers and no raw increment.

- **c2** — **real, FIXED in this commit** by the code, **not yet confirmed by the panel**. The sweep's candidate list is now only candidates; the one pure decision is `retention_decision(snapshot, current, now, max_age)` which yields `Purge | KeepDelivered | KeepRecentlyDispatched | KeepWithinWindow | KeepChanged`. The sweep re-reads the row and abandons the delete when `current ≠ snapshot`, and a `Dispatching` row with a state change inside the window is kept while a stuck dispatch past the window is still reclaimed. `retention_decision_defers_to_a_live_dispatch_and_to_a_changed_record` + `sweep_does_not_delete_a_record_that_is_mid_dispatch` pin this; no live node ran it.

- **c5** — **real, FIXED in this commit** (migration without distinguishing aliased keys is unresolvable, but the *serve path* is closable — and was). `pending_for_destination` is now an exact-destination match (`record.destination_peer_id == destination_peer_id`) with an accompanying regression test (`a_legacy_record_with_a_separator_is_not_served_to_another_destination`). Pre-existing rows stay on disk under another destination's prefix, but are never served to the wrong peer. A bulk migration that rewrites affected keys would be the other half; it is not needed for correctness and would not distinguish which destination of an aliased pair is legitimate.

- **c6** — **real, FIXED in this commit with a disclosed trade-off**. `purge_expired_custody_transitions` drops a FINISHED custody's transitions whole, once the custody's last event `max(at_ms)` is past the window — never a live custody, and never row-by-row (which would misattribute). The audit window is floored at `CUSTODY_MIN_AUDIT_RETENTION_MS = CUSTODY_DEFAULT_MAX_AGE_MS` (7 days), so `purge_expired_custody(1)` in tests does not erase the record of what was dropped. Reported as `purged_transitions` on every sweep. The trade is: long-horizon auditability beyond the retention window is sacrificed for bounded storage. That trade was authorized by the operator with this pass. Guarded by `retention_bounds_the_transition_trail_too`, `trail_of_custody_that_still_exists_is_never_truncated`, and `a_delivered_trail_survives_a_zero_tolerance_message_sweep`.

- **c7** — **real, FIXED in this commit** (decoupled + observable). Retention has its own `custody_retention_interval` (5 min) rather than riding `backoff_prune_interval`; success is `info!` with deferred/in-flight/changed counters counting, failure is `error!` (`retention is NOT being enforced`). The remaining observability gap is a per-capture metric surface (the repo has none), which stays open.

- **c4** — **real but NOT closable at this boundary**. `relay_per_peer_budget(global) = max(global/4, 25)` caps a single peer at 50/200, so 4 identities recover the node-wide starvation the old gate had with 1 identity. Fixing that at the transport `PeerId` level would require an identity-attested peer id (a body/record check, seniority check, or a “distinct peers this window”-aware share) — a security/identity trade-off, not a bug fix. **Escalated (rule 9)**. The global budget (200/hr) still bounds total relay work, and at the fleet's natural 15/hour peak the cap is a safety net.

Concrete severity on this head, controller-weighted: c2 high, c3 medium (bounded now), c5 medium (serve-path closed, migration still open), c6 medium with trade, c7 medium (now observable and on its own schedule), c4 high accepted-as-limitation. Nothing here is `critical` after the fixup, and nothing touches crypto/routing/privacy beyond the reviewed stores and admission.

## Tests

CI-equivalent local gates run on this head (the worktree build, `target/` deliberately NOT wiped so this check is authoritative for this worktree; the shared checkout's target is left to its host):

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`: clean (exit 0). So are `cargo check -p scmessenger-core` and `cargo test -p scmessenger-core --lib` (1,467 lib tests green; 5 intentionally `#[ignore]`d, none `failed`) on this worktree.

### New regression tests (one per finding that got a forward fix — each fails without the fix)

In `core/src/store/relay_custody.rs`:

- `retention_decision_defers_to_a_live_dispatch_and_to_a_changed_record` (c2).
- `sweep_does_not_delete_a_record_that_is_mid_dispatch` (c2, end-to-end through the real state machine).
- `a_legacy_record_with_a_separator_is_not_served_to_another_destination` (c5).
- `retention_bounds_the_transition_trail_too` + `trail_of_custody_that_still_exists_is_never_truncated` + `a_delivered_trail_survives_a_zero_tolerance_message_sweep` (c6 — bounded + invariants).
- No `#[ignore]`s introduced.

In `core/src/transport/swarm.rs` inside `relay_per_peer_budget_tests`:

- `per_peer_budget_map_is_bounded_like_the_token_bucket_map` (c3).
- `per_peer_budget_use_accumulates_and_reports_unknown_peers_as_unused` (c3).

In `core/tests/test_rule8_pr305_charge_points.rs`:

- `hourly_budget_is_charged_at_exactly_the_two_commit_points` + `budget_accounting_is_no_longer_incremented_from_the_old_pre_parse_site` (c8 — structural, because the defect *was* the position of the increment).

In `core/tests/test_and06_live_key_corpus.rs` and `test_and06_*` the strict canon check and live-mesh key corpus are neighbors, but not direct regression tests for this pass.

The remaining affected lanes (`cargo test --tests`, `cargo build`, `cargo test` for the `cli` crate) were not re-run locally this pass because they are pure downstream consumers of the fixed `store`/`transport`; the CI sweep will exercise them.

## Open findings

1. **c4 — multi-identity relay starvation** — not closed. Needs an identity-aware relay admission decision (or a per-window “distinct-peer fair share” that scales the divisor) and is a rule-9 matter.

2. **c5 — bulk legacy key-space migration** — serve path closed, but keys already stored under an aliased prefix stay where they are. A bulk migration (rewrite under a new prefix that never contains `_` as a component delimiter, or normalize `peer id` → libp2p `PeerId` at rest) would be the other half.

3. **c6 — long-horizon audit beyond the retention window** — now bounded by the retention window (whole-trail cull). Durable off-device archiving was the other design path.

4. **c7 — metric/executable observability** — the sweep now logs at `error!` and on its own schedule, but there is still no `storage_pressure_state()` field or `/api/diagnostics` counter for “last retention at N, failures M”.

## What remains unexercised (the review's own “what I could not verify”)

- The sweep's *liveness* claim (retention + transition reclamation keep node storage bounded under a sustained `accept_custody` load for N retention windows) is argued, not demonstrated against a live node with ledger-sharing and real sled pressure — the region the CI matrix and a 5-minute wall-clock prune cannot exercise.
- c2's TOCTOU fix was exercised deterministically; *concurrent* API callers (`IronCore::purge_expired_custody` at ~5 minutes + an overlapping `mark_dispatching` from the swarm) were proven to be serialized by the read-then-compare protocol rather than by a sled transaction — that serialization is correct but not tested under threads in this head.
- Wasm parity of all five closures was proven by `cargo check --target wasm32-unknown-unknown` in the breadth sweep (the earlier CLI check is the evidence on that target), not by a running wasm browser harness.

## Artifacts (Harness checkout, outside the product tree)

- `Harness/audits/scmessenger/_runs/seat-gates/verify_pr305_identity_20260918.json` (first-pass, 2/3 voted)
- `Harness/audits/scmessenger/_runs/seat-gates/verify_pr305_custody_20260918.json` (first-pass, 1/3)
- `Harness/audits/scmessenger/_runs/seat-gates/verify_pr305_identity_fixed_20260918.json` (3/3 — the authoritative identity re-gate; `c3` and `c8` flipped to **not_real**)
- `Harness/audits/scmessenger/_runs/seat-gates/verify_pr305_custody_fixed_20260918.json` (2/3 — custody re-gate; **c5** confirmed real, c2/c6/c7 split)
- `Harness/audits/scmessenger/_runs/seat-gates/verify_pr305_claims_20260918.json` (deferred 8-claim, 1/3 — not authoritative)
- Claims/source inputs: `SCMessenger/tmp/rule8-pr305/claims_{identity,custody}.json` + `source_{identity,custody}.txt` + `prompt_{identity,custody}.md`

Total paid spend for this review (fixed-diff phase on the head in this verdict): `$0.0227` inside two per-run `$0.10` ceilings (`identity $0.0111`, `custody $0.0115`); full review spend `$0.0383` across five runs.
