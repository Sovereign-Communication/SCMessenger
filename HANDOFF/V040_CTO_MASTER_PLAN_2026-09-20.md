# CTO master plan — v0.4.0 completion (2026-09-20)

Status: Active
Owner: CTO/orchestrator
**Implementation authority for identity/transport/WiFi (2026-09-21):**
`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md`
(code-verified WP matrix + harness/JEV completion gates). P0 umbrella:
`HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md`.
Harness: `sovereign-harness` — use **harness `origin/main`** / worktree
`C:\Users\SCM\Documents\GitHub\Harness-jev-use` for JEV (`harness/jev.py`);
local clone may lag. Clarification: harness verify + JEV typed questions when
confidence < 99%. Canonical DONE requires JEV `is_passing` + mechanical
evidence (implementation plan §3).
**Repo insight packs / batched sentiment:** `HANDOFF/V040_JEV_HARNESS_INTEGRATION_2026-09-21.md`
+ `scripts/jev_repo_insights.py` + `scripts/jev_packs.py`.

## Direction (operator, cumulative)

1. **Working-first** (interview 2026-09-20): reliable day-to-day mesh before tag pressure.
2. **Complete work to 0.4.0 tag readiness** with 3-node log analysis; merge train + rollout + iterate.
3. **2026-09-21:** integrate WiFi/identity audit WP1–5 into the plan; implement from verified matrix only — no new root-cause plans.

Both stand: implement the working bar **and** close 0.4.0-gate items; tag only when §4 checklist is green **and** WP5 WiFi proof is on the P0 ticket.

## Live fleet (2026-09-20)

| Node | SHA / build | State |
|---|---|---|
| AWS cloud | `34b56d54` via `testbotz/scmessenger:sha-34b56d5` | HEALTHY; identity `12D3KooWGvCWJNo...` preserved |
| Windows CLI | `34b56d54` CI artifact | HEALTHY; identity `12D3KooWD6vZQrUq...`; peers includes cloud; `connection_path_state: DirectPreferred` |
| Pixel 6a | App on device still pre-candidate | **BLOCKED** on `adb install -r`: CI APK not signed with pinned debug identity (`SCMESSENGER_DEBUG_KEYSTORE_BASE64` unset) |

Main tip after docs/CI train: `d7f169c1` (audit package) / tracking PR **#337** pending merge.
Code-bearing tip for mesh: **`34b56d54`** (#322 outbox keys + #325 chat order).

## Concurrent audit — owned disposition

Source: `HANDOFF/audit/CANONICAL_OUTLIER_AUDIT_2026-09-19_INDEX.md` + iter0–6
(landed on main via **#336**). Tracking lineage PR **#337** (AGENTS disk rules,
disk_budget.py, reclaim_safe.py, HANDOFF docs) — orchestrator owns merge.

### Corrections to audit open-rows (re-verified on `origin/main` 2026-09-20)

| Audit row | Audit HEAD claim | Current `origin/main` |
|---|---|---|
| CO-G-003 / TRN-04 | STILL-OPEN (grep retention = 0) | **FALSE STALE** — `purge_expired_custody`, `CUSTODY_DEFAULT_MAX_AGE_MS` present (PR #305) |
| CO-G-003 / TRN-07 | STILL-OPEN (`relay_budget = 200`) | **PARTIAL STALE** — global default still 200 **and** `relay_per_peer_budget` present (PR #305). Residual = policy/tuning, not missing mechanism |
| CO-G-003 / AND-06 | STILL-OPEN (7 BigInteger) | **TRUE** — operator ruled A-lite then UniFFI cutover |
| CO-G-003 / SEC-03 | STILL-OPEN (deny.toml waivers) | **TRUE** — operator ruled (b) migration branch parallel |
| BLOCKER-0.4.0 | empty | **Agree** for D1–D7 integrity; HIGH items still get fixed in train |

### 0.4.0-target findings — remediation lanes

| ID | Sev | Work | Lane / gate |
|---|---|---|---|
| CO-B-001 | HIGH | Dual-drain `IronCore::flush_outbox_for_peer` (base58 + canonical hex) | Freebuff impl ticket; **orchestrator review**; harness verify if design deviates from CLI pattern |
| CO-B-002 | HIGH | Mark `docs/ID_UNIFICATION_IMPLEMENTATION.md` Superseded; point at api.udl | Docs PR (orchestrator/freebuff) |
| CO-G-001 | MED process | Index or archive unindexed freebuff queue files | Orchestrator docs PR |
| CO-G-002 | MED | Disposition stale P1 outbox ticket (verified-fixed vs re-scope for wasm) | Orchestrator ticket update in same PR as CO-B-001 |
| CO-A-002 + doctrine strings | MED | Diagnostics “relay server failure” role-naming | Freebuff docs/string ticket |
| Wave-1 code | P0 | conn-limits, AND-06 A1/A2, watchdog test, IP-churn, beach-join P0–1 | Freebuff DISPATCHABLE (already on main) |
| AndroidTest compile | P0 CI | `MeshRepositoryHistoryTest` MainActivity unresolved; `IdentityCreationFlowTest` junit/compose unresolved — blocks Mobile lane green | Freebuff androidTest fix; orchestrator merge |
| Keystore pin | P0 device | Set `SCMESSENGER_DEBUG_KEYSTORE_BASE64` so CI APKs install over Pixel | **Operator** (secret); then re-dispatch Mobile + `adb install -r` |

## Merge train (orchestrator)

| PR | Content | Action |
|---|---|---|
| #337 | glm audit lineage (process/scripts/docs) | Merge when remaining checks green + AGENTS/scripts review |
| Stage this branch | CTO plan + freebuff tickets + doc corrections | Open PR; merge after required contexts |
| Freebuff Wave-1 code PRs | conn-limits (Rule-8), AND-06, watchdog, etc. | Operator paste; merge after Rule-8 + green |
| CO-B-001 / androidTest | Code fixes | Stage as PRs; merge after gates |
| Dependabot/legacy | Post-train | Operator batch |

**CI hygiene (standing):** after every merge, `gh run cancel` superseded SHA/PR runs.
Policy: `docs/rules/BUILD_AND_CI.md`.

## 3-node confirmation (before tag call)

After fleet is on one code SHA (prefer main tip after Wave-1 merges):

1. `scripts/tier_a_conformance.sh` — full output on file.
2. Log analysis (Windows + AWS + Pixel passive):
   - `[SEED-DIAL]` cadence
   - custody accept/dispatch counts
   - outbox drain / undelivered trend
   - `connection_limits` WARN rate vs 2026-09-19 baseline (~274/hour)
   - identity stable across restart
3. Operator phone session (UI) — working bar §6 of working-first path.
4. Tag checklist §4 below.

## §4 0.4.0 tag checklist

- [ ] Wave-1 code landed or waived in writing
- [ ] Audit 0.4.0 HIGHs closed (CO-B-001/002) + CO-G-001/002 disposed
- [ ] AND-06 A1+A2 or dated operator waiver
- [ ] SEC-03 dated accept **or** migration branch started with owner
- [ ] AndroidTest compile green on Mobile
- [ ] Pinned debug keystore secret set + Pixel install -r succeeds
- [ ] 3-node same-SHA (or documented android-only delta) + log pack
- [ ] D1 required CI green on tag SHA
- [ ] Operator: external audit commission (deferred until mesh reliable — still required for public tag per SHIP_PLAN G4-2 unless operator waives)
- [ ] Final `v0.4.0` tag + release publish (operator)

## Confidence / harness

| Claim | Confidence | Action |
|---|---|---|
| TRN-04/07 code on main | 99%+ | Command-verified; no harness |
| CLI dual-drain pattern for CO-B-001 | 99%+ | Pattern already on main `cli/src/main.rs` |
| Wasm envelope impact of dual-drain | <99% | **Harness** before merge of CO-B-001 if any deviation |
| Instrumented test root causes | ~90% | Freebuff implement; harness verify PR if still red |
| Keystore secret value | unknown | Operator |

## Safe-ahead PRs (staged, not self-merged by freebuff)

1. This CTO plan + tickets package
2. CO-B-002 doc supersede (can ride same PR)
3. CO-G-001 freebuff README index completion
4. CO-B-001 code (after paste/impl)
5. androidTest compile fix
6. #337 lineage merge

## Next freebuff paste order (operator)

1. `V040_T_CONN_LIMITS_MULTIPORT.md`
2. `V040_T_COB001_WASM_OUTBOX_DUAL_DRAIN.md`
3. `V040_T_ANDROIDTEST_COMPILE_FIX.md`
4. `V040_T_AND06_KOTLIN_COLLAPSE.md`
5. Remaining Wave-1 (watchdog, IP-churn, beach-join P0–1)

Do not paste: keystore, tag, review dispatches, CODE ON MAIN tickets.
