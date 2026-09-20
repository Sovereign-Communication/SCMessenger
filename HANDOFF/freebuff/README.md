# Freebuff lane -- live queue

Status: Active
Last updated: 2026-09-20 (CTO merge-train update)
Rules: `docs/rules/FREEBUFF.md` -- read it before adding a task file here.
**Order authority:** `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md` (includes
working-first path + concurrent audit dispositions + 0.4.0 tag checklist).
Operator rulings: `HANDOFF/freebuff/inbox/V040_OPERATOR_DECISIONS_TAGPATH_2026-09-20.md`.

## Direction (operator 2026-09-20)

1. **Working-first:** reliable day-to-day mesh on Windows + AWS + Pixel.
2. **Then 0.4.0 tag readiness** via master plan checklist + 3-node log analysis.
Do not paste keystore/release/tag chores as freebuff implementation tasks.
Do not re-dispatch CODE ON MAIN / MERGED tickets. Pinned debug keystore
secret is **operator-owned** (device `adb install -r`).

## Durable CTO continuation

Tracked `/cto` entry: `.claude/commands/CTO.md`.
Package: `HANDOFF/V040_CTO_3NODE_BLE_CONTROLLER_PACKAGE_2026-09-08.md`.
Freebuff has no headless mode -- operator pastes task files. No ignored
`.freebuff/` file is authoritative.

```
queue/   ready to paste
inbox/   Freebuff replies
done/    completed; Status records the PR number
```

**Before any paste wave:** `python scripts/check_queue_status.py` must exit 0.

**CI queue hygiene (operator 2026-09-20):** cancel superseded GitHub Actions
runs after merges/branch updates — see `docs/rules/BUILD_AND_CI.md`. Orchestrator
clears the queue; freebuff reports blocked waits with run ids in `inbox/`.

---

## DISPATCHABLE -- paste order (2026-09-20 CTO update)

Paste only after the ticket exists on **`origin/main`**. One ticket per paste.
Authority: `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md`.
#325/#322 already merged to main — do not wait on them.

| Order | Task file | What it fixes | Review gate |
|---|---|---|---|
| 1 | `queue/V040_T_CONN_LIMITS_MULTIPORT.md` | Live multi-port dials denied (`connection_limits` cap 4) | **Rule-8** |
| 2 | `queue/V040_T_COB001_WASM_OUTBOX_DUAL_DRAIN.md` | Audit CO-B-001 HIGH: IronCore flush single-form key | Orchestrator; harness if not a CLI-pattern port |
| 3 | `queue/V040_T_ANDROIDTEST_COMPILE_FIX.md` | Mobile instrumented-test compile red | none if androidTest-only |
| 4 | `queue/V040_T_AND06_KOTLIN_COLLAPSE.md` | A-lite: collapse redundant Kotlin Ed25519 copies | none if android-only |
| 5 | `queue/V040_T_AND06_UNIFI_CUTOVER.md` | A full: UniFFI cutover; delete curve math | Rule-8 if FFI/core |
| 6 | `queue/V040_T_WATCHDOG_POSITIVE_TEST.md` | N-03 healthy-quiet node watchdog | test-only preferred |
| 7 | `queue/V040_T_LEDGER_IP_CHURN_AUTONOMOUS.md` | Cloud IP change remesh without manual edits | **Rule-8** likely |
| 8 | `queue/V040_BEACH_JOIN_PHASE0_1_2026-09-20.md` | Beach-join Phase 0-1 (operator pull-forward) | Phase 2-3 later |

Orchestrator merge train: **#337** (audit lineage) + CTO docs PR + freebuff code PRs after gates.
Scoring after the wave: harness + 3-node logs + **operator** phone session.

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
