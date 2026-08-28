# V0.4.0 Completion Plan — Final Push

Status: Active
Created: 2026-08-25
Authority: Executes SHIP_PLAN.md D1-D7; supersedes nothing. Subsidiary to GAP_AUDIT_REMEDIATION_PLAN.md for post-tag sprints.
Current position: `v0.4.0-rc.1` tagged @ 134e06d2, release pipeline exercised, Mac lane dispatched.

---

## 0. Scope truth

v0.4.0 requires ZERO feature LoC beyond what exists at rc.1. Remaining work is
configuration, evidence, and documentation. Budget: **~150-300 LoC net** (signing
config wiring, possible demo-failure fixes), plus operator time on hardware.

Skipping v0.4.0 is rejected as strategy: MILESTONE_RELEASE_PLAN states every
v0.5.0 item assumes v0.4.0 is solid. Farm-sim debugging on an undemonstrated
base converts known evidence tasks into unknown distributed-systems bugs.

## 1. Gate-by-gate status and remaining tasks

### D1 — main is green (CP1)
| Task | LoC | Lane | Done when |
|---|---|---|---|
| D1-a Confirm latest origin/main CI run state; record run URL | 0 | subagent | Run URL in this file's ledger |
| D1-b If red: triage failing lanes per SHIP_PLAN S1-1..S1-4 (mobile signing config from keystore.properties.template + secrets; hygiene check enforceable pre-push; clippy error extraction; docker suite blocking-status ruling) | 50-120 | subagent drafts, operator merges | Green run recorded |
| D1-c If the four S1 lanes were already fixed by tag-day work: capture that run as CP1 evidence | 0 | subagent | Ledger row filled |

### D2 — Signed APK downloadable (CP2)
| Task | LoC | Lane | Done when |
|---|---|---|---|
| D2-a Inspect existing releases (v0.4.0-rc.1 and any alpha) for APK asset + signing provenance | 0 | subagent | Asset inventory recorded |
| D2-b If missing: wire release signing (S1-1 prerequisite) and build signed APK from tagged SHA with SCM_GIT_HASH embedded | 30-80 | agy + operator secrets | APK installs on Pixel 6a |
| D2-c Publish/refresh GitHub release with APK + notes from CHANGELOG.md | 0 | operator | Public download works |

### D3 — README explains product + install (CP2 same checkpoint)
| Task | LoC | Lane | Done when |
|---|---|---|---|
| D3-a README exists (4,033 bytes) but predates rc.1: verify version numbers, install steps, build-from-source commands are accurate against actual release artifacts | ~20 edits | subagent drafts diff, operator approves | Stranger-successful download path |

### D4/D6/D7 — Two-device receipt, transport racing, offline proximity (CP3/5/6)
| Task | LoC | Lane | Done when |
|---|---|---|---|
| E-a Rebuild all nodes at tagged SHA (per PR139_FIVE_NODE_GATE_STATUS, nodes were stale — gate has never run clean) | 0 | agy + operator | Every node reports tag hash |
| E-b Two-device test on RELEASED APK, cross-network (cellular + WiFi) | 0 | operator + devices | Receiver decrypt + durable history + receipt |
| E-c Transport racing: first-choice transport unavailable -> fallback delivers | 0 | operator + devices | Same receiver-side evidence standard |
| E-d Offline proximity: two devices, no internet | 0 | operator + devices | Same receiver-side evidence standard |
| E-e Contingency: any demo failure becomes the single ticket; fix and re-run | 50-200 (contingency) | free lanes | Re-run passes |

### D5 — No long-lived integration branch (CP4)
| Task | LoC | Lane | Done when |
|---|---|---|---|
| D5-a Verify disposition of PR #139 / integration branch #230/#231 chain; confirm no long-lived branch remains outside 48h rule | 0 | subagent | Branch audit recorded |

## 2. Execution order

Phase A (machine-verifiable, today): D1-a/c, D2-a, D3-a draft, D5-a — all evidence gathering + local drafting. No pushes, no tags.
Phase B (operator + hardware): signing secrets if needed, device rebuilds, three demos. Sequential; each produces ledger evidence before next.
Phase C: tag decision. If rc.1 assets are complete and demos pass on rc.1 build -> promote rc.1 to final or cut v0.4.0 final. Else one fix cycle max, then re-demo.
Phase D: publish, fill SHIP_PLAN §5 checkpoint ledger, hand off to GAP_AUDIT_REMEDIATION sprints + v0.5.0.

## 3. Total remaining estimate

- Net new LoC: 150-300 (config + contingency only)
- Demo contingencies: up to ~200 LoC if a gate fails
- Operator sessions: 2-3 (one signing/secrets, one multi-device demo day, one publish)
- Native verdict checkpoints: 2-3 (green-main verdict, stranger-readme verdict, demo-scoring verdict)

## 4. Checkpoint ledger

Fill as evidence lands. Empty = honest unknown.

| Checkpoint | Criterion | Date | Evidence (URL/output) |
|---|---|---|---|
| CP1 | D1 main green | 2026-08-25 | ALL FIVE LANES GREEN on latest main run set (PR #231 merge): CI https://github.com/Sovereign-Communication/SCMessenger/actions/runs/32803311759 (24m27s), Docker Publish 32803311756, Lint 32803311736, Repo Hygiene 32803311753, CodeQL 32803311114. SHIP_PLAN S1-1..S1-4 lane work confirmed done by tag-day effort. |
| CP2 | D2+D3 release published | | BLOCKED: no GitHub release exists for any v0.4.0 tag; v0.1.9 still Latest; no APK asset anywhere. Signing config + tagged-SHA build + publish outstanding. README edits drafted (see Phase A report). |
| CP3 | D4 two-device proof | | Pending CP2 (must demo on released APK) |
| CP4 | D5 #139 resolved | | PARTIAL: #139/#230/#231 all MERGED. Branch hygiene violates 48h rule (dozens of stale cto/*, tracking/*, integration/* branches) — deletion sweep required before honest CP4. |
| CP5 | D6 racing proof | | Pending CP2 |
| CP6 | D7 offline proof | | Pending CP2 |

## 5. Phase A findings (2026-08-25, subagent evidence run)

1. **D1 CLOSED** — see CP1.
2. **Working tree impurity**: `android/.../ble/BleGattClient.kt` (source!), `scripts/rules_check.py`, `.claude/hooks/preflight_guard.py` modified-uncommitted; four untracked HANDOFF docs incl. this plan. Tag purity requires commit-or-discard disposition.
3. **D5 sweep list**: 18+ dated `cto/*` branches from 08-21/22, plus ~40 undated stale refs (`test-merge`, `pr-*-head`, `subagent-*`).
4. **README draft corrections ready**: stale alpha.1 claim -> rc.1 wording; layout table missing `mobile/` + `desktop_bridge/` rows; install-section APK line gated on D2-c landing.
5. **Single largest blocker: D2 signing + publish** (operator secrets + hardware). Everything else is hours of mechanical work.

## 6. Phase B progress (2026-08-25)

- **CP1 remains green** — no regression since the Phase A evidence run.
- **Receipt-convergence defect root-caused and FIXED live.** Android now
  emits signed prepareReceipt envelopes (was bare JSON, rejected by the
  receiver); send-status converges to `delivered:true` on the
  Windows<->Pixel rig. Evidence: message ids
  `4e693533-dbd6-4677-bde8-7ed89dc7e90b` and
  `52a56a77-6f73-45a1-bf35-508aaf78a089` both `delivered:true`.
  Regression guard test landed with the fix.
- **Contact-unknown display bug fixed** — root cause was identity-cache
  poisoning in `MeshRepository.getContact`; conversation names now resolve
  for pubkey-canonical contacts.
- **Three-node version parity achieved** — Windows / Pixel / AWS relay all
  at `0064d49a` (AWS node rebuilt 2026-08-25; see
  HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md).
- **Formal D4/D6/D7 scoring still pending** released-APK demo day, per
  SHIP_PLAN standard. The live receipt convergence above is rig evidence,
  not CP3 credit.
