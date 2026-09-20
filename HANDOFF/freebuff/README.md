# Freebuff lane -- live queue

Status: Active
Last updated: 2026-09-20 (working-first rewrite)
Rules: `docs/rules/FREEBUFF.md` -- read it before adding a task file here.
**Order authority:** `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md`
(overrides tag-pressure sequencing in older path docs).
Operator rulings: `HANDOFF/freebuff/inbox/V040_OPERATOR_DECISIONS_TAGPATH_2026-09-20.md`.

## Direction (operator 2026-09-20)

**No tag, no secrets pressure. Get the reliable day-to-day mesh working first.**
Do not paste keystore/release/tag tasks. Do not re-dispatch tickets marked
CODE ON MAIN / MERGED / DO NOT PASTE.

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

---

## DISPATCHABLE -- Wave 1 paste order (2026-09-20)

Paste only after the ticket exists on **`origin/main`**. One ticket per paste.

| Order | Task file | What it fixes | Review gate |
|---|---|---|---|
| 1 | `queue/V040_T_CONN_LIMITS_MULTIPORT.md` | Live multi-port dials denied (`connection_limits` cap 4) | **Rule-8** |
| 2 | `queue/V040_T_AND06_KOTLIN_COLLAPSE.md` | A-lite: collapse redundant Kotlin Ed25519 copies to one path | none if android-only |
| 3 | `queue/V040_T_AND06_UNIFI_CUTOVER.md` | A full: wire validation to core UniFFI; delete remaining curve math | Rule-8 if core/FFI surface |
| 4 | `queue/V040_T_WATCHDOG_POSITIVE_TEST.md` | N-03: healthy-quiet node must not be watchdog-killed | test-only preferred |
| 5 | `queue/V040_T_LEDGER_IP_CHURN_AUTONOMOUS.md` | Cloud node IP change must remesh without manual bootstrap edits | **Rule-8** likely |
| 6 | `queue/V040_BEACH_JOIN_PHASE0_1_2026-09-20.md` | Operator pulled Phase 0-1 forward (spec + hotspot share) | Phase 2-3 still Rule-8 later |

Orchestrator (not freebuff paste): merge **#325** and **#322** when green.

Scoring after the wave: harness + logs + **operator** phone session.
**No scoring mid-wave** per operator ruling.

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

---

## Never idle -- tiers

| Tier | Nodes | Now |
|---|---|---|
| **A** | AWS + Windows CLI | Drive Wave 1 reliability; install anytime from CI artifacts |
| **B** | Pixel 6a | Operator drives UI; agents install + passive logs only |
| **C** | iOS/macOS | Out of Wave 1 |

## Adding a task

1. Premise-verify end to end.
2. File in `queue/` per FREEBUFF.md section 3.
3. Index here + ensure on `origin/main` before paste.
4. Keep `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md` consistent.
