# Freebuff lane -- live queue

Status: Active
Last updated: 2026-09-21 (deduplicated; single train section added)
Rules: `docs/rules/FREEBUFF.md`
**SESSION HANDOFF (canonical, read first):**
`HANDOFF/V040_FREEBUFF_TRANSITION_2026-09-21.md`
**Implementation authority (identity/transport/WiFi):**
`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md`
**0.4.0 master plan:** `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md`
Operator rulings: `HANDOFF/freebuff/inbox/V040_OPERATOR_DECISIONS_TAGPATH_2026-09-20.md`.

## Direction

1. **Working-first:** reliable day-to-day mesh (Windows + AWS + Pixel).
2. **WiFi/identity WP1-5** from the implementation plan matrix (no guesswork).
3. **Then 0.4.0 tag** checklist + WP5 3-node proof + operator phone session.

**DONE = mechanical gates + harness JEV canonical pack `is_passing`** (plan S3).
`UNVERIFIED-JEV` / unkeyed fallback is **not** DONE -- Freebuff PRs that claim
WP completion without a keyed `jev_canonical_check.py` exit 0 must be rejected
by the orchestrator. Never claim WiFi fixed without WP5 evidence on the P0
umbrella ticket.

**JEV / harness (local only):** see
`HANDOFF/V040_JEV_HARNESS_INTEGRATION_2026-09-21.md`.
- `python scripts/update_local_harness.py` -> `vendor/sovereign-harness`
- `python scripts/jev_repo_insights.py --mode full` (bucket/triage read)
- `python scripts/jev_canonical_check.py --wp WPn --state-file <state.json>`
- TypeSafe first; OpenRouter `~typesafe/jev-latest` on
  `https://openrouter.ai/api/alpha/decisions` if TypeSafe unhealthy
  (operator: allow OpenRouter provider `typesafe`).
- Do **not** edit external Harness product trees / WIP PRs.

## Paste protocol

- Operator is the transport into Freebuff desktop.
- Paste only tickets that exist on **`origin/main`** and are DISPATCHABLE.
- One ticket per paste. Premise-check first; wrong premise -> inbox note.
- Freebuff opens PRs; no self-merge; Rule-8 on
  `core/src/{crypto,transport,routing,privacy}`.
- CI hygiene: cancel superseded Actions runs (`docs/rules/BUILD_AND_CI.md`).

```
queue/   ready to paste
inbox/   Freebuff replies
done/    completed; Status records the PR number
```

`python scripts/check_queue_status.py` must exit 0 before paste waves.

---

## THE SINGLE TRAIN (0.4.0)

One order, one build, one proof. Sequencing authority is unchanged: plan S4 and
`HANDOFF/todo/_QUEUE.md` own the order; this section only states how the train
is built and verified, so a WP ticket, its PR, its CI build and the fleet SHA
stay in step.

| Step | Action | Gate | State 2026-09-21 |
|---|---|---|---|
| 1 | WP1 `queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md` | tests + JEV | not started |
| 2 | WP2 `queue/V050_WP2_ROUTING_FEED_ALL_TRANSPORTS_2026-09-21.md` | Rule-8 + JEV | code complete, PR #349 open; keyed JEV **fail** (supported 0.13); Rule-8 row open |
| 3 | WP3 `queue/V050_WP3_INBOUND_COMPLETENESS_2026-09-21.md` | Rule-8 if gated + JEV | not started |
| 4 | WP4 `queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` | tests + JEV | not started |
| 5 | WP5 3-node live proof | operator phone + logs | not started; evidence lands on the P0 umbrella ticket, not a PR body |
| 6 | Wave-1 leftovers (A-F below) | per ticket | A/E Rule-8; B covered by #341; C/D open |
| 7 | Merge train + fleet redeploy + 0.4.0 checklist | master plan S4 | fleet still on `51edac4b` (observed on the running node) |

Build path -- a WP ticket is verified against **one** SHA, not three beliefs:

1. Each WP lands on its own `freebuff/*` branch and its own PR (one PR per WP).
2. `freebuff/v050-train` is the integration branch: it takes merged or
   merge-ready WP work plus Wave-1 leftovers, and CI builds it once.
3. The fleet (AWS cloud node, Windows node, Pixel) deploys the artifact from
   that one CI build, so WP5 compares like with like. Record the SHA on the P0
   umbrella ticket; a per-node SHA delta must be written down, not implied.
4. Do not deploy a locally built binary to the fleet: build output is
   reclaimable and a running node's image must not live in `target/`.

Harness read of this queue (2026-09-21, keyed, `--batch-size` batched):
`identity_transport` is the highest-**severity** pain (1.9-2.0 of 2, "high
delivery blocker"); `process_dispatch` is the highest-**frequency** pain
(dominant in 3 batches, 0.88-0.96 confidence) with process health 1.18 of 2,
"stale but recoverable"; the unification pack's `fix_class` favours
`docs_status_fix` (0.56) over `small_code_fix` (0.25). Reproduce with
`python scripts/jev_repo_insights.py --mode full --batch-size 40`.

---

## DISPATCHABLE -- P0 WiFi/identity (train steps 1-4)

| Order | Task file | WP | Gate |
|---|---|---|---|
| 1 | `queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md` | WP1 | tests + JEV |
| 2 | `queue/V050_WP2_ROUTING_FEED_ALL_TRANSPORTS_2026-09-21.md` | WP2 | **Rule-8** + JEV |
| 3 | `queue/V050_WP3_INBOUND_COMPLETENESS_2026-09-21.md` | WP3 | Rule-8 if gated + JEV |
| 4 | `queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` | WP4 | tests + JEV |

WP5 live 3-node proof is **not** a freebuff paste -- evidence lands on
`HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md`.

## DISPATCHABLE -- parallel Wave-1 / mesh reliability (train step 6)

| Order | Task file | Gate |
|---|---|---|
| A | `queue/V040_T_CONN_LIMITS_MULTIPORT.md` | **Rule-8** |
| B | `queue/V040_T_ANDROIDTEST_COMPILE_FIX.md` | landed as PR #341 -- verify only |
| C | `queue/V040_T_AND06_KOTLIN_COLLAPSE.md` then UNIFI cutover | Rule-8 if FFI/core |
| D | `queue/V040_T_WATCHDOG_POSITIVE_TEST.md` | test-only |
| E | `queue/V040_T_LEDGER_IP_CHURN_AUTONOMOUS.md` | **Rule-8** likely |
| F | `queue/V040_BEACH_JOIN_PHASE0_1_2026-09-20.md` | Phase 2-3 later |

CO-B-001 dual-drain is **already merged** (#339) -- do not paste that ticket as impl.

---

## DO NOT PASTE

| Ticket | Why |
|---|---|
| T1 / T2 / T4 implementation | CODE ON MAIN -- scoring/residual only |
| T5 / T6 / T7 / T8 / T10 / T11 / T12 / T14 | DONE or MERGED |
| Completed `V040_REVIEW_DISPATCH_*` | Reviews already filed |
| Keystore / release / tag tasks | Operator deferred -- working first |
| C4 identity-aware relay admission | 0.5.0 |
| Beach-join Phase 2-3 | After Phase 0-1 + working bar |
| `queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md` | Inventory pass FINAL -- reference only; see `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md`. Do not paste as a Wave 1 implementation task |

---

## Never idle -- tiers

| Tier | Nodes | Now |
|---|---|---|
| **A** | AWS + Windows CLI | Drive the train (steps 1-4) + Wave-1 reliability; install anytime from CI artifacts; fleet on `51edac4b` (redeploy after merges) |
| **B** | Pixel 6a | Operator drives UI; agents install + passive logs only |
| **C** | iOS/macOS | Out of Wave 1 / 0.5.0 |

## Adding a task

1. Premise-verify against `origin/main`.
2. File in `queue/` + index here + implementation plan if it is WP work.
3. Merge dispatch packets to `main` before paste.

---

## Reference -- canonical outlier audit (landed inventory)

| Item | Path |
|---|---|
| Master index (FINAL) | `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md` |
| Iterations 0-6 | `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter0.md` .. `iter6.md` |
| Queue task file | `HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md` |
| Tracking note | `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_FINAL_TRACKING_2026-09-19.md` |
| Docs landing branch | `docs/canonical-outlier-audit-2026-09-19` (from `origin/main`) |
