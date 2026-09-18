# Rule-8 adversarial review — PR #305 (TRN-04, TRN-07, AND-06)

Status: APPROVE-WITH-NOTES — fixes verified on anchored windows; two named follow-ups; NOT MERGED (merge is the operator's)
Date: 2026-09-18
Scope: `core/src/store/relay_custody.rs`, `core/src/identity/keys.rs`, `core/src/transport/swarm.rs`, `cli/src/server.rs` (delegated consumer)
Branch: `freebuff/v040-blocking-fixes-20260917`
Base: `origin/main` (`c2ce2f6406b1`)
Review head: `880765c9` (fixes) + `8cfd9cc9` (verdict doc; code-identical for review purposes)
CI at verdict time: Lint success, Repository Hygiene success; CI/Mobile/iOS runs in progress on `8cfd9cc9`, zero failures.

## CORRECTION of the earlier version of this document

The first re-gate after the fixup (`verify_pr305_{identity,custody}_fixed_20260918.json`
plus two custody retries) is RETRACTED as fix-verification. Its source windows were
generated from hard-coded line ranges captured BEFORE the fixup commit, and the fixup
shifted every line below its first insertion. None of the fix symbols
(`retention_decision`, the exact-serve filter, `purge_expired_custody_transitions`,
`PeerRelayUse`, both `C8-CHARGE-POINT`s, the retention tick) were inside those windows;
the seats voted on drifted text. The earlier claim of "identity 3/3, c3+c8 flipped" was
therefore not evidence about the fixes and is withdrawn. The runs remain on disk,
marked invalid.

The re-gate below is valid: windows are anchored to unique code symbols and the
builder fails closed unless every symbol a claim depends on is inside an emitted
window (`tmp/rule8-pr305/build_review_fixed.py`; per-window anchor lists are recorded
in `source_<part>_map.json` and were verified — e.g. the identity window contains both
`C8-CHARGE-POINT` sites, the custody windows contain `retention_decision`,
`KeepRecentlyDispatched`, `purged_transitions`).

## Method

Sanctioned gate: `harness.cli verify` structured-claims mode — candidate-defect claims
manifest + numbered verbatim source window, per-claim panel votes, deterministic
tally, judge synthesis; author (Freebuff lane) excluded by construction. Free tier
was attempted first on the initial pass and saturated (reasoning-only/truncated
outputs), as in the PR #296 precedent. Paid escalation per operator authorization;
each run capped by the harness's own per-run ceiling ($0.10 — hard-enforced; a $0.20
request is rejected). The review was split into two domain-scoped parts so each run's
worst case fits that ceiling at a vote budget (`--max-tokens 8192`) large enough for
seats to emit parseable votes. Panels: deepseek-v4-flash, deepseek-v4.1-flash,
ling-3.0-flash, gpt-4o-mini (judge z-ai/glm-5.3-flash); gemini-3.8-flash is
BYOK-skipped on this host.

## Authoritative re-gate result (anchored windows, 3/3 seats voted on BOTH parts)

### Identity/transport part — c1, c3, c4, c8 — 3/3 voted — actual $0.0088

Tally: c1=not_real (0R/3NR); c3=not_real (1R/2NR); c4=real (2R/1NR); c8=not_real
(1R/2NR). Judge: "c1, c3, and c8 are not real defects (strict canonical key rejection
is intentional hardening; the per-peer relay map is bounded via
prune_peer_relay_use/RELAY_PEER_BUCKET_MAX_TRACKED; budget accounting only occurs in
the Ok(custody) branch after accept_custody commits). c4 is a real, medium-severity
issue ... arguably a distinct Sybil threat outside the one-peer fairness control's
intended scope."

Seat disposition (verified against the diff, not taken on the vote):
- c1 not_real — accepted. The strict check IS the fix (dalek was laxer than Kotlin).
- c3 not_real — fix verified: bounded map + prune, both loops, two unit tests.
- c8 not_real — fix verified: charge moved to the commit branches; structural guard.
- c4 real — NOT CLOSED. See follow-ups.

### Custody part — c2, c5, c6, c7 — 3/3 voted — actual $0.0125

Tally: c2=not_real (1R/2NR); c5=real (2R/1NR); c6=not_real (1R/2NR); c7=not_real
(1R/2NR). Judge: "c2 NOT REAL: retention_decision(snapshot, current) re-reads the live
record before purging, guarding the TOCTOU case (KeepChanged path). c5 REAL (medium
severity): separator ban is enforced only at ingestion; pre-existing records with
separators are never migrated or quarantined. c6 NOT REAL: purge_expired_custody
explicitly calls purge_expired_custody_transitions(audit_window), bounding the audit
trail with the same window. c7 NOT REAL" (own retention tick; failure logged loudly).

Seat disposition:
- c2 not_real — fix verified (pure decision + re-read + abandon-on-change + in-window
  Dispatching guard; three tests).
- c6 not_real — fix verified (whole-trail cull, live-trail invariant, floored window;
  three tests).
- c7 not_real — fix verified (own 5-minute tick; failure at error level).
- c5 real — PARTIALLY CLOSED. The serve path no longer hands a pre-ban row to the
  wrong destination (exact-destination match + regression test). What remains real is
  the residual the judge names: no migration/quarantine of pre-existing aliased rows.
  See follow-ups.

## Disposition of the original eight claims

| claim | initial panel | after fix (anchored) | state |
|---|---|---|---|
| c1 strict-check rejects legit keys | real (1R/0NR, mirrored text) | not_real 0R/3NR | closed (was the fix itself) |
| c2 sweep TOCTOU vs dispatch | real 2R/0NR | not_real 1R/2NR | closed by fix, panel-verified |
| c3 unbounded per-peer map | real 2R/0NR | not_real 1R/2NR | closed by fix, panel-verified |
| c4 multi-identity budget bypass | real 2R/0NR | real 2R/1NR | OPEN — escalated (rule 9) |
| c5 legacy rows alias on serve | real 2R/0NR | real 2R/1NR (residual) | PARTIAL — serve path closed; migration open |
| c6 unbounded transition trail | real 2R/0NR | not_real 1R/2NR | closed by fix (disclosed trade), panel-verified |
| c7 sweep rides dial-prune, fail-open | real 2R/0NR | not_real 1R/2NR | closed by fix, panel-verified |
| c8 charge before parse | real 1R/0NR→2R/0NR | not_real 1R/2NR | closed by fix, panel-verified |

## Follow-ups (named, non-blocking for this PR's scope, decided by the operator)

1. c4 — multi-identity relay starvation: the per-peer share caps ONE PeerId; 4+
   identities aggregate the node budget. Closing it needs an identity-aware admission
   input (ledger-attested peer identity, or a distinct-peers-aware share). Security
   trade-off → rule 9 escalation. The global budget still bounds total relay work;
   measured natural inbound peak is 15/hour against a 200/hr budget.
2. c5 — bulk migration/quarantine of pre-ban custody rows: serve path is safe now
   (exact-destination match); a one-shot migration that rewrites aliased keys under a
   separator-free layout is the remaining half and needs a storage-migration decision.
3. c6 — long-horizon audit beyond the retention window is deliberately sacrificed
   (operator-authorized trade); off-device archiving is the alternative if wanted.
4. c7 — observability: sweep failure is now error-level on its own schedule; a
   metrics/diagnostics counter ("last sweep at, consecutive failures") is optional
   polish.

## Regression tests (each fails without its fix)

- c2: `retention_decision_defers_to_a_live_dispatch_and_to_a_changed_record`,
  `sweep_does_not_delete_a_record_that_is_mid_dispatch` (store).
- c3: `per_peer_budget_map_is_bounded_like_the_token_bucket_map`,
  `per_peer_budget_use_accumulates_and_reports_unknown_peers_as_unused` (swarm).
- c5: `a_legacy_record_with_a_separator_is_not_served_to_another_destination` (store).
- c6: `retention_bounds_the_transition_trail_too`,
  `trail_of_custody_that_still_exists_is_never_truncated`,
  `a_delivered_trail_survives_a_zero_tolerance_message_sweep` (store).
- c8: `core/tests/test_rule8_pr305_charge_points.rs` — pins exactly two
  `C8-CHARGE-POINT` markers (native + wasm) and asserts the old pre-parse increment
  (`or_insert(0) += 1`) is gone.
- Local gates on the fix head: `cargo fmt --all -- --check` clean;
  `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`
  exit 0; `cargo test -p scmessenger-core --lib` 1,467 passed / 0 failed;
  targeted suites green. Full CI sweep (Windows/macOS/Ubuntu, WASM, Android JVM, FFI
  surface, CodeQL) runs on `8cfd9cc9`.

## What was not exercised

- c2's fix is deterministic-tested; concurrent API callers (operator sweep overlapping
  a live dispatch) are protected by the re-read-and-compare protocol, not a sled
  transaction, and were not raced under real threads on a live node.
- The custody liveness claim (storage stays bounded across N retention windows under
  sustained load) is argued from code + tests, not demonstrated against the live fleet
  (fleet untouched this pass by instruction).
- Wasm parity is compile-checked (`wasm32-unknown-unknown` lane in CI), not run in a
  browser harness.
- Kotlin/Swift FFI migration onto `is_valid_public_key` remains future work (AND-06
  half); the UniFFI export is snapshot-pinned by the FFI Surface Contract lane.

## Artifacts (Harness checkout, outside the product tree)

Valid (anchored windows):
- `verify_pr305_identity_regate_20260918.json` — 3/3 voted, actual $0.0088
- `verify_pr305_custody_regate_20260918.json` — 3/3 voted, actual $0.0125
- `lint_pr305_{identity,custody}_regate_20260918.json` — ok:true, issues:[]

Invalid as fix-verification (stale windows; retained for the record, marked):
- `verify_pr305_{identity,custody}_fixed_20260918.json`,
  `verify_pr305_custody_fixed{2,3}_20260918.json`

Initial pass (pre-fix diff; the evidence that produced the eight claims):
- `verify_pr305_claims_20260918.json` (8-claim, 1/3 — deferred),
  `verify_pr305_identity_20260918.json` (2/3), `verify_pr305_custody_20260918.json` (1/3)

Inputs: `SCMessenger/tmp/rule8-pr305/claims_{identity,custody}.json`,
`source_{identity,custody}.txt`, `source_{identity,custody}_map.json`,
`prompt_{identity,custody}.md`, builder `build_review_fixed.py`.

Total paid spend, all runs including invalid and initial ones: ≈ $0.0797 across 9
runs, every run inside its own $0.10 worst-case ceiling (actuals 6–13% of ceiling).
