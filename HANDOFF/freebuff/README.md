# Freebuff lane -- live queue

Status: Active
Last updated: 2026-09-21
Rules: `docs/rules/FREEBUFF.md`
**Implementation authority (identity/transport/WiFi):**
`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md`
**0.4.0 master plan:** `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md`
Operator rulings: `HANDOFF/freebuff/inbox/V040_OPERATOR_DECISIONS_TAGPATH_2026-09-20.md`.

## Direction

1. **Working-first:** reliable day-to-day mesh (Windows + AWS + Pixel).
2. **WiFi/identity WP1–5** from the implementation plan matrix (no guesswork).
3. **Then 0.4.0 tag** checklist + WP5 3-node proof + operator phone session.

**DONE = mechanical gates + harness JEV canonical pack `is_passing`** (plan §3).
Never claim WiFi fixed without WP5 evidence on the P0 umbrella ticket.

**JEV / harness integration (future work):** see
`HANDOFF/V040_JEV_HARNESS_INTEGRATION_2026-09-21.md`.
- Before paste waves / after merge trains:
  `python scripts/jev_repo_insights.py --mode full`
- Before marking any WP DONE:
  `python scripts/jev_canonical_check.py --wp WPn --state-file <state.json>`
- Clarification when <99% confident: harness `verify` + JEV typed questions
  (`HARNESS_REPO=C:\Users\SCM\Documents\GitHub\Harness-jev-use`).
- Unkeyed JEV = `is_fallback` — not canonical DONE.
- Do not edit concurrent Harness WIP branches (P2/P3/jev-phase).

## Paste protocol

- Operator is the transport into Freebuff desktop.
- Paste only tickets that exist on **`origin/main`** and are DISPATCHABLE.
- One ticket per paste. Premise-check first; wrong premise → inbox note.
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

## DISPATCHABLE — P0 WiFi/identity first (2026-09-21)

| Order | Task file | WP | Gate |
|---|---|---|---|
| 1 | `queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md` | WP1 | tests + JEV |
| 2 | `queue/V050_WP2_ROUTING_FEED_ALL_TRANSPORTS_2026-09-21.md` | WP2 | **Rule-8** + JEV |
| 3 | `queue/V050_WP3_INBOUND_COMPLETENESS_2026-09-21.md` | WP3 | Rule-8 if gated + JEV |
| 4 | `queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` | WP4 | tests + JEV |

WP5 live 3-node proof is **not** a freebuff paste — evidence lands on
`HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md`.

## DISPATCHABLE — parallel Wave-1 / mesh reliability

| Order | Task file | Gate |
|---|---|---|
| A | `queue/V040_T_CONN_LIMITS_MULTIPORT.md` | **Rule-8** |
| B | `queue/V040_T_ANDROIDTEST_COMPILE_FIX.md` | CI Mobile (PR #341 may already cover) |
| C | `queue/V040_T_AND06_KOTLIN_COLLAPSE.md` then UNIFI cutover | Rule-8 if FFI/core |
| D | `queue/V040_T_WATCHDOG_POSITIVE_TEST.md` | test-only |
| E | `queue/V040_T_LEDGER_IP_CHURN_AUTONOMOUS.md` | **Rule-8** likely |
| F | `queue/V040_BEACH_JOIN_PHASE0_1_2026-09-20.md` | Phase 2-3 later |

CO-B-001 dual-drain is **already merged** (#339) — do not paste that ticket as impl.

---

## DO NOT PASTE

| Ticket | Why |
|---|---|
| T1/T2/T4 impl, T5–T12/T14 | CODE ON MAIN / MERGED |
| Completed `V040_REVIEW_DISPATCH_*` | Reviews filed |
| Keystore / release / tag | Operator / post-working-bar |
| C4 / beach-join Phase 2-3 | Post-wave |
| Canonical outlier audit task file | Reference only |

---

## Never idle — tiers

| Tier | Nodes | Now |
|---|---|---|
| **A** | AWS + Windows | Wave + WP code; fleet on `51edac4b` (redeploy after merges) |
| **B** | Pixel | Operator drives UI; agents install + passive logs |
| **C** | iOS/macOS | 0.5.0 |

## Adding a task

1. Premise-verify against `origin/main`.
2. File in `queue/` + index here + implementation plan if it is WP work.
3. Merge dispatch packets to `main` before paste.


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
| **A** | AWS + Windows CLI | Drive Wave 1 reliability; install anytime from CI artifacts |
| **B** | Pixel 6a | Operator drives UI; agents install + passive logs only |
| **C** | iOS/macOS | Out of Wave 1 |

## Reference -- canonical outlier audit (landed inventory)

| Item | Path |
|---|---|
| Master index (FINAL) | `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md` |
| Iterations 0-6 | `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_iter0.md` .. `iter6.md` |
| Queue task file | `HANDOFF/freebuff/queue/AUDIT_CANONICAL_OUTLIERS_V040_V050_V100_2026-09-19.md` |
| Tracking note | `HANDOFF/freebuff/inbox/AUDIT_CANONICAL_OUTLIER_FINAL_TRACKING_2026-09-19.md` |
| Docs landing branch | `docs/canonical-outlier-audit-2026-09-19` (from `origin/main`) |
| Superseded mixed PR | #335 (`glm/canonical-outlier-audit`) -- do not force-push; lineage split after docs land |

## Adding a task

1. Premise-verify end to end.
2. File in `queue/` per FREEBUFF.md section 3.
3. Index here + ensure on `origin/main` before paste.
4. Keep `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md` consistent.
