# GPT SEEDING REVIEW -- terminal verdict request (040-S2)

Status: READY FOR REVIEW
Date: 2026-07-28
Authority: PR #115 plan gate 040-S2 (independent adversarial verdict with
one evidence-backed disposition per finding + final SHIP/NO-SHIP line).

## Review target

    REVIEW_TARGET: ed13500abaf372836be37bef93f3eaf5a24765a6..b1261fbf (tip of refs/heads/wip/v040-seeding-fixes)
    REMOTE_REF: refs/heads/wip/v040-seeding-fixes
    WINDOWS_GATE: cargo check -j10 PASS on every packet; full
    cargo test -p scmessenger-core --no-run + targeted test run in
    progress (results will be appended to this file before tag decision).

NOTE ON LATER COMMITS: one or two follow-up commits may land after this
request (cargo fmt style pass + any test-compile repair). They will be
style or test-only; the FUNCTIONAL range for this verdict is
ed13500a..b1261fbf. If a substantive functional commit follows, a new
request supersedes this one.

## Commits in range (oldest first)

- d258fd7f 1a: F10 ledger cap + eviction + F7b seed ordering
- 068972f2 1b: F10 save-off-lock + shared annotate helper
- 5b66f896 packet 2: F7a dial-policy register gate, F7b record_failure
  wiring at both failure arms, F13 is_dialer gate on pending-dial
  resolution (design decision: gate KEPT -- evidence integrity over the
  rare collapsed-simultaneous-open edge, which self-heals via the 10s
  sweep while the connection stays up; spurious failure +1 vs threshold
  3), NEW-6 global TokenBucketState (burst tuned to 10, refill 2/s)
- 21095127 v2c-1: save_lock serialization + atomic durable writes
  (unique tmp + sync_all + rename + parent-dir fsync on unix)
- d2497460 1c: batch ledger annotation at both wire handlers
- 02efea70 v2c-2: load sanitization + durable shrink + 5 persistence/
  concurrency tests
- 63051067 v2c-3: corrupt-JSON recovery (quarantine, no startup brick)
  + peer_id parity + rename nonce
- b1261fbf v2b: invite anchors stamp last_seen (no None underclass),
  canonical multiaddr tie-breaks (cross-node determinism), threshold
  completion in dialable_addresses + exchange_response_entries, 5 tests

## Required verdict (per plan 040-S1b: implicit deferral forbidden)

One evidence-backed disposition per finding -- FIXED / NOT-FIXED /
REGRESSION / NEW-ISSUE with file:line -- for EACH of: F2, F3, F6, F7,
F10, F12, F13, F16, NEW-5, NEW-6, plus the 1b lost-update/corruption
regression. Current orchestrator dispositions (challenge them):
- F2: no product accept path exists -- residual is future-wiring only;
  acceptance ticket must require verify_with_policy before
  import_seed_entries. Is "no live path" truly airtight at this SHA?
- F3/F4/F5/F6/NEW-5/F11/F12: claimed FIXED pre-range; verify at tip.
- F7/F10/F13/NEW-6/1b-regression: fixed IN this range (commits above).
- F16: no drift (Kotlin build-time generated; ffi_surface.sh check exit
  0; Swift regenerated in PR #112).
Plus your own NEW-ISSUE hunt on the full diff (races, desync, downgrade,
framing-compat, DoS). Then a final SHIP / NO-SHIP line for
v0.4.0-alpha.1 with justification.

Deliver on gpt/seeding-review as GPT_SEEDING_REVIEW_TERMINAL_VERDICT.md
(branch autonomy granted; do not merge).
