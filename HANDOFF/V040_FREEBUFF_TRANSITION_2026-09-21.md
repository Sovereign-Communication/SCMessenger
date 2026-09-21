# SCMessenger Freebuff transition — canonical handoff (2026-09-21)

Status: **ACTIVE — sole execution handoff for Freebuff after orchestrator session close**
Owner: CTO/orchestrator session close
Canonical: this file wins for post-session Freebuff execution until the
operator replaces it. D1–D7 remain the **future release** definition, not the
current work bar.

Entry points after this file lands on `origin/main`:
- `/CTO` → `HANDOFF/CTO_STATE.md` (resume block at top) + this file
- `/CEO` → `HANDOFF/CEO_STATE.md` (resume block at top) + this file
- Freebuff paste index → `HANDOFF/freebuff/README.md`
- Freebuff rules → `docs/rules/FREEBUFF.md`

---

## 1. Operator direction (standing)

| Rule | Source |
|---|---|
| **Working-first:** reliable day-to-day mesh before tag/secret pressure | Interview 2026-09-20 + inbox rulings |
| **Then** 0.4.0 tag readiness via master plan checklist | `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md` |
| WiFi/identity implement from **verified matrix**, no new root-cause plans | `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` |
| **DONE = mechanical gates + keyed JEV `is_passing`** | Freebuff README + implementation plan §3 |
| Local harness **inside SCMessenger**; do not edit external Harness product trees | 2026-09-21 |
| TypeSafe ISE → OpenRouter `~typesafe/jev-latest` via `/api/alpha/decisions` | 2026-09-21 |
| CI hygiene: cancel superseded Actions after merges | `docs/rules/BUILD_AND_CI.md` |
| Fresh Android install allowed; Pixel UI is **operator-driven** | 2026-09-20/21 |
| No mid-wave scoring; after wave: harness + logs + operator phone session | Interview 2026-09-20 |

---

## 2. Authority stack (read in this order)

1. `AGENTS.md` — hard rules, capability classes
2. `docs/rules/FREEBUFF.md` — lane contract
3. **This file** — session close + Freebuff next steps
4. `HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` — WP matrix + JEV DONE
5. `HANDOFF/V040_CTO_MASTER_PLAN_2026-09-20.md` — 0.4.0 checklist + merge train
6. `HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md` — working-first rulings
7. `HANDOFF/V040_JEV_HARNESS_INTEGRATION_2026-09-21.md` — JEV consumer contract
8. `HANDOFF/freebuff/README.md` — DISPATCHABLE paste set
9. P0 umbrella: `HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md`

Historical CTO_STATE/CEO_STATE bodies below the new resume blocks are archive.

---

## 3. Live state snapshot (orchestrator close, command-backed)

| Item | State |
|---|---|
| `origin/main` | `9d37f9e6` (docs: freebuff WP DONE requires keyed JEV is_passing #345) |
| Windows CLI | `0.4.0 (51edac4:main)` healthy; peer AWS; identity `12D3KooWD6vZQrUq...` |
| AWS cloud | `0.4.0 (51edac4b...)` healthy; identity `12D3KooWGvCWJNo...`; seed-dial + gossip + custody retention live |
| Pixel 6a | APK `51edac4b` CI artifact installed (fresh install authorized); ADB connected; operator launches UI |
| PR #347 | **OPEN** — local harness consumer + OpenRouter fallback (see §5) |
| Tag | No final `v0.4.0`; only `v0.4.0-rc.1` git tag historically |
| Disk | TIGHT (~9–12 GB free depending on CI); shared cargo cache reclaimed 2.53 GB |

**3-node log evidence (Tier A) 2026-09-21:** AWS `[SEED-DIAL] peers=1 -- connected`,
Gossipsub from Windows, custody retention sweep active. Windows `DirectPreferred`
+ cloud peer + circuit listener. Full pack: `HANDOFF/V040_3NODE_LOG_ANALYSIS_2026-09-20.md`,
fleet: `HANDOFF/V040_FLEET_STATUS_51EDAC4B_2026-09-20.md`.

---

## 4. What is already on `origin/main` (do not re-implement)

| Landed | Evidence |
|---|---|
| Working-first path + WP tickets DISPATCHABLE | #331, #343, freebuff README |
| WiFi/identity implementation plan + P0 umbrella | #343 |
| CO-B-001 dual-drain outbox | #339 |
| Chat-order + outbox key fixes | #325, #322 |
| Mobile androidTest compile fix | #341 (APK job SUCCESS on that PR head) |
| Canonical audit package + dispositions | #336, #337, #338 |
| JEV packs / repo insights / canonical_check | #344 |
| Freebuff WP DONE = keyed `is_passing` | #345 |
| Freebuff inbox 3-node baseline evidence | #346 |
| CI: secrets-context action fix, Mobile workflow_dispatch | #333, #334 |
| Seed dial / ledger unification / routing feed (swarm) | earlier train (#263, #266, #267 lineage) |
| TRN-04 retention + TRN-07 per-peer budget | #305 (audit STILL-OPEN rows were stale) |

**CODE ON MAIN / MERGED — do not paste as implementation:** T1, T2, T4 (impl),
T5–T12, T14, CO-B-001, completed review dispatches, keystore/tag chores.

---

## 5. Open PR train (Freebuff/orchestrator merge)

| PR | Title | Action |
|---|---|---|
| **#347** | Local harness consumer + OpenRouter Jev fallback | Merge when required CI green (`pr_scope 347`). Contains `vendor/sovereign-harness` consumer scripts + issue-sort pack |
| #329 / #316 / #303 etc. | Queue docs / outbox notes / merge-plan drafts | Disposition: merge docs-only if clean; else close superseded |
| #298–#302 work-aheads | Android/iOS join/share | **Hold** until Wave-1 code lands unless they fix a working-bar defect |
| Dependabot / Apple / legacy | many | Post-wave batch (operator) |

**Merge protocol:** `python scripts/pr_scope.sh <n>`; required contexts
(Hygiene, Lint, Rust Linting, Test ubuntu) green; Rule-8 on
`core/src/{crypto,transport,routing,privacy}`; cancel superseded Actions;
**Freebuff does not self-merge.**

---

## 6. DISPATCHABLE — Freebuff paste order (Wave 1 / WP)

Paste only files that exist on `origin/main`. One ticket per paste.
Checklist: `python scripts/check_queue_status.py` exit 0.

| Order | Ticket | Gate |
|---|---|---|
| 1 | `queue/V040_T_CONN_LIMITS_MULTIPORT.md` | **Rule-8** + JEV later |
| 2 | `queue/V040_T_AND06_KOTLIN_COLLAPSE.md` | tests + wiring |
| 3 | `queue/V050_WP2_ROUTING_FEED_ALL_TRANSPORTS_2026-09-21.md` | **Rule-8** + JEV |
| 4 | `queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md` | tests + JEV |
| 5 | `queue/V050_WP3_INBOUND_COMPLETENESS_2026-09-21.md` | Rule-8 if gated + JEV |
| 6 | `queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` | tests + JEV |
| 7 | `queue/V040_T_AND06_UNIFI_CUTOVER.md` | Rule-8 if FFI + JEV |
| 8 | `queue/V040_T_WATCHDOG_POSITIVE_TEST.md` | test-only |
| 9 | `queue/V040_T_LEDGER_IP_CHURN_AUTONOMOUS.md` | **Rule-8** + JEV |
| 10 | `queue/V040_BEACH_JOIN_PHASE0_1_2026-09-20.md` | android Phase 0–1 only |

WP5 **live 3-node proof** is **not** a Freebuff paste — evidence lands on the
P0 umbrella after code waves.

---

## 7. Canonical DONE contract (Freebuff + orchestrator)

For any WP / 0.4.0-working claim:

1. Mechanical: `rules_check.py`, targeted `cargo test`, greps from the
   implementation plan, `check_wiring.py` if UI/android, `pr_scope.sh` before merge.
2. JEV:

```text
python scripts/update_local_harness.py
python scripts/jev_canonical_check.py --wp WP2 --state-file tmp/wp2_state.json
```

   - Primary: TypeSafe `harness.jev` via **SCMessenger-local** `vendor/sovereign-harness`.
   - Fallback: OpenRouter **decisions** API `~typesafe/jev-latest`
     (`https://openrouter.ai/api/alpha/decisions`) when TypeSafe is unhealthy.
   - Operator OpenRouter account must allow provider **`typesafe`**.
   - **`is_passing` + non-fallback** required. `UNVERIFIED-JEV` is **not** DONE.
3. Rule-8 APPROVE on file for gated dirs (non-author reviewer).
4. WiFi-fixed claims require **WP5** logs on the P0 ticket.

Insight batches (optional, not a substitute for 3):

```text
python scripts/jev_repo_insights.py --mode full
```

---

## 8. Post-Wave scoring + tag (after Freebuff code lands)

1. Redeploy Tier A to one candidate SHA (CI artifacts → `tmp/radio-<sha>/`).
2. `python scripts/tier_a_conformance.sh` + Windows/AWS log pack.
3. Operator phone session (day-to-day mesh).
4. Master plan §4 checklist (keystore/D2 only when operator reopens tag work;
   SEC-03 dated waiver or migration; AND-06 closed or waived; external audit
   commission when mesh is reliable; WP5 evidence).
5. Operator final `v0.4.0` tag + release publish — **not** another rc/alpha.

---

## 9. Freebuff hard limits

- No self-merge; no secrets; no tags; no force-push; no Pixel UI driving.
- Shared checkout: touch only assigned files; worktrees for new work.
- No emoji; evidence contract on every status line; wrong premise → `inbox/`.
- Do not edit external Harness WIP (PRs #39/#47/#48, Harness product modules).
- Reclaim only via `scripts/reclaim_safe.py` after work is on a remote.

---

## 10. Session close checklist (orchestrator → Freebuff)

- [x] Working-first + WP plan + JEV contract on `origin/main`
- [x] Fleet Tier A on `51edac4b` with live seed-dial/gossip/custody evidence
- [x] Pixel APK installed (fresh identity); operator drives UI
- [x] DISPATCHABLE tickets indexed on main
- [x] JEV local-harness consumer + OpenRouter fallback **staged on #347**
- [x] `/CTO` + `/CEO` resume blocks updated to point at this file
- [ ] **#347 merged** (Freebuff/orchestrator next merge when CI green)
- [ ] Wave-1 implementation PRs from Freebuff pastes
- [ ] WP5 + scoring + operator session → 0.4.0 tag path

**Freebuff starts here:** merge #347 if still open and green, then paste
DISPATCHABLE order §6, one ticket per cycle, DONE only with §7.

---

*End of canonical Freebuff transition handoff.*
