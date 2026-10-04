# 0.4.0 train -- status after the WP1-WP4 pass (2026-09-21)

Status: Active dispatch status for the train in
`HANDOFF/freebuff/README.md` "THE SINGLE TRAIN (0.4.0)".
Lane: Freebuff (status author). Merges, HANDOFF ticket moves, WP5 and the
final tag are NOT this lane's authority.
Purpose: replace the README's WP cells with states that were read from the
commands below in this session, so an orchestrator does not have to re-derive
them and no stale cell drives a re-paste.

Sequencing authority is unchanged: implementation plan S4 order. This file
records state only.

## 1. Train steps, with the command that produced each state

| # | Package | State | Evidence (this session) |
|---|---|---|---|
| 1 | WP1 identity unification | **PR #352**: 2 commits, 0 failing checks, MERGEABLE; keyed canonical JEV **pass 0.81** | `gh pr view 352 --json statusCheckRollup,mergeable`; `python scripts/jev_canonical_check.py --wp WP1 --state-file HANDOFF/freebuff/jev/WP1_state_2026-09-21.json` exit 0 |
| 2 | WP2 routing feed, all transports | **PR #349**: 0 failing checks, MERGEABLE; keyed canonical JEV **pass 0.92** | `gh pr view 349 --json statusCheckRollup,mergeable`; `jev_canonical_check.py --wp WP2` exit 0 |
| 3 | WP3 inbound completeness | **PR #355** open; CI running at time of writing | `gh pr create` -> https://github.com/Sovereign-Communication/SCMessenger/pull/355 |
| 4 | WP4 delivery truth | **PR #356** open; CI running at time of writing | `gh pr create` -> https://github.com/Sovereign-Communication/SCMessenger/pull/356 |
| 5 | WP5 live 3-node proof | **NOT STARTED** -- operator phone session + log pack, lands on the P0 umbrella ticket | none (no claim) |
| 6 | Wave-1 leftovers | see section 2 | per-row below |
| 7 | Merge train + fleet redeploy + 0.4.0 checklist | **NOT STARTED** by this lane (no self-merge) | none (no claim) |

The previous README cells for WP1 and WP2 recorded "keyed JEV **fail**". Both
fails came from the harness `diff_question_pack()`, whose single axis asks
whether the changed *code* implements the instruction -- a question a
tests-and-greps residual cannot answer well. The DONE contract names
`scripts/jev_canonical_check.py`; under that pack both packages pass on a live
key. Ruling requested:
`HANDOFF/freebuff/inbox/JEV_CANONICAL_VS_DIFF_PACK_RULING_2026-09-21.md`.

## 2. Wave-1 leftovers (train step 6)

| Row | Package | State after this session |
|---|---|---|
| A | conn-limits multi-port (`V040_T_CONN_LIMITS_MULTIPORT.md`) | **OPEN, not started.** Rule-8 mandatory, and the ticket's own design constraints require choosing between "distinguish productive connections from port probes" and "raise the per-peer bound with a retained-socket test". Loosening a DoS bound is a security trade-off, so per AGENTS rule 9 it is the operator's call, not this lane's: no code was written rather than guess. |
| B | androidTest compile fix | landed as PR #341 (`9de879b9`, on `origin/main`) -- verify-only row |
| C | AND-06 Kotlin collapse (+ UniFFI cutover) | **collapse already done** on `origin/main`: `grep -rn "modPow\|BigInteger" android/app/src/main/java/com/scmessenger/android/` leaves only `utils/PeerIdValidator.kt` (curve math) and `utils/PeerKeyUtils.kt` (base58, exempt), so the paste as written would find nothing to collapse. The UniFFI cutover is **blocked on an unrun prerequisite, not on code**: no JVM unit test calls a UniFFI function today (18 test files use UniFFI *types* only; the only real calls are in main sources), so pointing the validator at `is_valid_public_key` would put the host native library on the Android JVM Unit Tests lane's critical path on a pre-tag identity path. Evidence + a two-move recommended design: `HANDOFF/freebuff/inbox/AND06_KOTLIN_CUTOVER_PREREQUISITE_2026-09-21.md`. No code written. |
| D | watchdog positive test (`V040_T_WATCHDOG_POSITIVE_TEST.md`) | **DONE inside PR #356** -- the exit decision is now `cli/src/watchdog.rs` with the positive case asserted (a node emitting its liveness line is never killed across 600 readings) |
| E | ledger IP-churn autonomous rediscovery | **OPEN, not started** (Rule-8; disclosure-sensitive store/transport paths, needs a design decision on address supersession) |
| F | beach-join Phase 0-1 | **OPEN, not started** (Android) |

## 3. PRs opened or corrected this session

| PR | What |
|---|---|
| #354 | `HANDOFF/freebuff/jev/` evidence (the exact state files behind the WP1/WP2 JEV verdicts) + the instrument ruling request |
| #355 | WP3: one auto-subscribe decision shared by the native and wasm gossip loops, with the own-topic exemption now test-pinned |
| #356 | WP4: watchdog decision extracted to a tested predicate; a refused envelope proven to leave no delivered state |
| #352 (body) | corrected: the send-path hash-confusion guard is **fixed** (commit `6b5b08f3`), superseding the body's "filed, not fixed"; keyed canonical JEV pass recorded |
| #349 (comment) | keyed canonical JEV pass recorded, superseding the README's 0.13 cell |

## 4. What remains outside this lane

- **WP5** (live 3-node, operator phone session). No WiFi claim is made anywhere
  in this session's output; every JEV state file says so explicitly.
- **Merges.** No PR was merged, no HANDOFF ticket was moved between
  `todo`/`in_progress`/`done`, no tag was created, no secret was touched.
- **Rule-8 sign-off** on PR #355 (`core/src/transport/swarm.rs`). Flagged in the
  PR body, not waived.
- **0.4.0 tag checklist rows** (master plan S4): unchanged by this session
  except as noted -- Wave-1 code is *closer* (WP3/WP4 PRs exist, watchdog row
  met) but no row flips to done until its PR merges and WP5 evidence lands.

## 5. Fleet state

Not re-verified this session. The README's last recorded observation stands
(fleet on `51edac4b`, AWS cloud node + Windows node alive, Pixel installed) and
is quoted here as the README's claim, not as a fresh measurement. A redeploy is
step 7 of the train and is the orchestrator's action.

## 6. Reading order for an orchestrator picking this up

1. This file (state), then `HANDOFF/freebuff/README.md` (order + paste set).
2. `HANDOFF/freebuff/jev/README.md` (how to re-run the two JEV verdicts).
3. `HANDOFF/freebuff/inbox/JEV_CANONICAL_VS_DIFF_PACK_RULING_2026-09-21.md`
   (the one open ruling that gates how WP DONE is recorded).
4. `HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md`
   (where WP5 evidence must land).
