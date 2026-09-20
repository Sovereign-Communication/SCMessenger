# V0.4.0 tag path — unified (2026-09-20)

Status: **SUPERSEDED FOR EXECUTION PRIORITY** by
`HANDOFF/V040_WORKING_FIRST_PATH_2026-09-20.md` (operator interview same day:
"no tag, no secrets, just get it working first"). This file remains useful as
the D1-D7 definition archive and audit-disposition history. Where sequencing
conflicts, the working-first path wins.

Status note: D1-D7 definitions below stay the future release exit criteria.

Author: orchestrator session (Windows host, FULL class). Operator directive
this session (initial): take ownership, run gates, score items, unify
contradictions, plan the freebuff lane. **Subsequent operator interview
re-prioritized to working-first — see the successor document.**

Evidence base: `origin/main` @ `efd240d7` (PR #308 merge), commands run
2026-09-20 on this host. Worktree used for verification:
`tmp/wt-tag-path-20260920` (branch `freebuff/v040-tag-path-20260920`).

---

## 0. Definition of done for the v0.4.0 tag

Unchanged from `SHIP_PLAN.md` §0 (operator-settled). A tag is cut only when
every row below is CLOSED with command evidence or a dated operator waiver.

| # | Exit criterion | Evidence required | Scorer |
|---|---|---|---|
| D1 | `main` green | Required push lanes green on the tag SHA; run URL recorded | Orchestrator |
| D2 | Signed APK downloadable | `gh release view v0.4.0` lists a signed APK/AAB asset | Operator + CI |
| D3 | README installable | Non-empty README; links resolve | Orchestrator |
| D4 | Message + receipt Android handset <-> Windows CLI | Receiver decrypt + durable history + receipt on the **released** APK | Operator + Tier A |
| D5 | No long-lived integration branch | Old carrier branches closed; `main` is trunk | Orchestrator |
| D6 | Transport racing proven | Delivery when first-choice transport is unavailable; non-zero routing confidence | Operator + Tier A |
| D7 | Offline proximity proven | Handset <-> Windows with no internet; same evidence standard | Operator + Tier A |
| Gate | Cloud-node parity (operator 2026-08-29) | Custody store-and-forward + connection assistance on live nodes | Orchestrator + logs |
| Gate | SEC-03 disposition | Operator accepts dated sled waivers **or** funds post-tag migration | Operator |
| Gate | External crypto audit **commissioned** | Firm/scope/price/dates on file (completion not required) | Operator |
| Gate | Audit blockers closed or waived | See §2 | Orchestrator + Rule-8 |

Anything that does not move a row above is **post-tag**. Exceptions need a
dated operator ruling in this file or a successor.

---

## 1. Gate results already obtained (this session)

Commands and results on this host, 2026-09-20. Never read `$?` after a pipe
without capture-first; results below are from exit codes captured before read.

| Gate | Command | Result |
|---|---|---|
| Wiring | `python scripts/check_wiring.py` in `tmp/wt-tag-path-20260920` | `[OK] All components, composables, routes, and utilities are correctly wired.` rc=0 |
| Repo rules | `python scripts/rules_check.py` | rc=0 |
| Docs sync | `bash scripts/docs_sync_check.sh` | `docs-sync-check: PASS` rc=0 |
| Versions | `bash scripts/verify_versions.sh` | `[OK] Cargo/Android/Desktop/WASM agree at 0.4.0; iOS marketing version is 0.4.0` / Android 15>14, iOS 10>9 rc=0 |
| Queue status | `python scripts/check_queue_status.py` | rc=1 — `[FAIL] V040_T7_ANDROID_PARITY_STAGING.md: claims PR #312 is open... but it is MERGED (2026-09-19T23:05:21Z)` |
| Disk budget | `python scripts/disk_budget.py` | verdict `OK` (28.17 GB free at check) |
| Windows node | `GET http://127.0.0.1:9876/health` + `/version` | `healthy`; `0.4.0 (0fd69fb:main:...)` — **one merge behind** tip `efd240d7` |
| Branch protection | `gh api .../branches/main/protection` | required contexts: `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`; `strict: true` |
| Push lanes on tip | `gh run list --branch main --limit 15` | On `efd240d7`: CI, Lint, Repository Hygiene, Docker Publish, Docker Integration Suite, Push on main = **success**. `Security Scan` (schedule) = **failure** — **not** a required protection context |
| Tag objects | `git tag` + `gh release list` | Newest tag `v0.4.0-rc.1` (git). **No GitHub Release object** for rc.1. Public releases still stop at v0.1.9 (2026-03-19) |
| Secrets present | `gh secret list` | All four signing secrets exist (`SCMESSENGER_KEYSTORE_BASE64`, `_KEYSTORE_PASSWORD`, `_KEY_ALIAS`, `_KEY_PASSWORD`) — **presence is not correctness**; rehearsal still failed on alias/value mismatch historically |
| Keystore files | `Get-ChildItem C:\Users\SCM\kiee` | `scmessenger-release.jks` (2266 bytes) + `.b64` present. Verification script requires interactive password — **operator-only** |
| Release pipeline | `gh run list --workflow "Multi-Platform Release Pipeline"` | Last five runs **all failure**, including `34996353889` (2026-09-15) known alias mismatch |

### D1–D7 scoreboard (unified, evidence-backed)

| # | Score | Evidence / blocker |
|---|---|---|
| D1 | **[OK]** for required push contexts on `efd240d7` | Lint + Hygiene + Rust Linting + Test (ubuntu) green via CI run + branch protection list. Security Scan schedule red is out of scope for D1 unless operator promotes it |
| D2 | **[BLOCKED — operator]** | No signed GitHub Release; release workflow last failure was keystore alias value; secrets exist but unverified against `C:\Users\SCM\kiee\scmessenger-release.jks` |
| D3 | **[OK]** | `README.md` non-empty historically verified (4,309 bytes); not re-opened this session |
| D4 | **[PARTIAL]** | Live mesh + store-and-forward proven on dev fleet (reval 2026-09-07, live 2026-09-18). **Not** scored on a stranger-downloadable released APK. Gated by D2 |
| D5 | **[OK]** | Carrier #288 **MERGED** 2026-09-17. Residual open PRs are not a second `main`; see §4 disposition |
| D6 | **[IMPLEMENTED, UNSCORED]** | `routing_peer_seen` production call at `core/src/transport/swarm.rs` on main (#263 merged 2026-09-01; #284 disposition 2026-09-13). Field proof of non-zero confidence on a failover path **not** on file for the tag candidate |
| D7 | **[NOT SCORED]** | Offline proximity gate still requires operator-supervised no-internet run with receiver-side evidence. Freebuff **must not** drive the Pixel |
| Cloud parity | **[PARTIAL]** | Seed dial + ledger unification + custody retention live. AWS IP-churn autonomous rediscovery **FAIL** (design gap V050-B1/B2 — recommend **known limitation** + manual bootstrap runbook, not a local code blocker, unless operator rules otherwise) |
| SEC-03 | **[OPEN — operator]** | sled waivers still in `deny.toml` with no expiry. Rule-9 tech-stack decision |
| External audit commission | **[OPEN — operator]** | Standing board ruling; commissioning is the gate |
| Audit blocking set | **[MIXED]** | See §2 |

---

## 2. Audit blocking set — unified disposition

Source of truth for *code* status is `origin/main`, not ticket Status lines.

| ID | Claimed as 0.4.0 blocker | Unified status on `origin/main` | Tag disposition |
|---|---|---|---|
| TRN-04 custody auth + retention | Multi-dim audit 2026-09-17 | **CODE LANDED** via PR #305 (`purge_expired_custody`, admission bounds). Rule-8 file: `HANDOFF/review/RULE8_PR305_VERDICT_2026-09-18.md` (APPROVE-WITH-NOTES) | **CLOSED for tag** unless a new finding re-opens it |
| TRN-07 per-peer relay budget | Same | **CODE LANDED** via #305 (`relay_per_peer_budget`); live verify 2026-09-18 | **CLOSED for tag** |
| TRN-08 / connection_limits cap 4 | Live 09-19 baseline + shadow audit family | **OPEN defect**, operator-visible (mass `limit 4 reached` WARNs; multi-port dials refused) | **OPEN** — see ticket T-CONN-04 in §3. Not optional if D4/D6 field scoring needs multi-port LAN paths |
| AND-06 Kotlin curve math | Multi-dim + bod-dd336324 | **HALF OPEN**. UniFFI `is_valid_public_key` exported (#305). Kotlin `BigInteger`/`modPow` still present (`PeerIdValidator.kt` et al.) | **Operator choice**: (A) finish Kotlin half pre-tag via freebuff + Rule-8 on core exposure already filed; or (B) dated accept for tag, migrate in 0.5.0. Ticket still says "must land before tag" — that is **not** reconciled until operator picks |
| SEC-03 sled / deny.toml | Shadow + multi-dim | **OPEN decision** | **Operator**: (a) dated waiver + owner + 0.5.0 budget, or (b) migrate on a branch (not required to finish before tag if (a)) |
| Multi-dim UNVERIFIED rows | 09-17 audit §9 | Several re-baselined by #305/#308 live work; **audit iteration 2 not written** | Orchestrator writes iteration N+1 before calling audit complete; may be post-code-fix documentation if remaining rows are waived |
| C4 identity-aware relay admission | #305 review c4 | Ruled **0.5.0** (queue row C4) | **OUT of 0.4.0** |
| AWS IP-churn rediscovery | Reval FAIL | Design gap V050-B1/B2 | **Waiver candidate**: record as known limitation + runbook; operator confirm |
| BLE real-time chat / cell-only bootstrap | 09-06 3-node | Historically FAIL/UNVERIFIED; not regressions of current candidate | **UNVERIFIED** — operator-supervised only. Not freebuff device work |
| N-03 log-silence watchdog | Multi-dim | Process can exit when quiet; positive test not on file | Ticket T-WATCH-POS; cheap freebuff/scoring item |
| N-04 chat ordering | Multi-dim | Partial fixes landed (#309, #315); **#325 same-second tie-break OPEN** | Merge #325 after BEHIND update + green |
| N-05 CI vs device debug keystore | Multi-dim | #324/#326 pinned debug signing intended to fix install-in-place | Re-verify on device when operator allows `adb install -r` |

---

## 3. Freebuff queue — verified disposition (do not re-dispatch the old order)

`HANDOFF/freebuff/README.md` still presents a 2026-08-31 order (T5→T2→T1, paste review dispatches first). **That order is obsolete.** Verified against GitHub + main:

| Ticket | README claim | Verified actual | Dispatch now? |
|---|---|---|---|
| T1 seed dial | OPEN, order 3rd | **Code on main** — `cli/src/seed_dial.rs`, `swarm.rs:build_seed_dial_candidates`, `connect_to_seed_peers` at `:3472`, boot call `:8403`. Revalidation PASS 2026-09-07 | **NO** — status rewrite only; residual = live SEED-DIAL cadence check on current tip |
| T2 unify ledgers | OPEN, order 2nd | **Code on main** — core ledger is the process store; legacy `peers.json` migrated + archived (`cli/src/ledger.rs` comments + migration). Disclosure uses `locally_verified: e.is_bootstrap` | **NO** for reimplementation. Residual = T13 follow-ups + live ledger hygiene, not the 2026-08-31 unification diff |
| T4 routing feed | OPEN | Premise **contradicted** on main (`swarm.rs` production `routing_peer_seen`). #263 merged; #284 disposition filed | **NO implementation**. **YES scoring** for D6 field proof |
| T5 docs sync | (moved historically) | `docs_sync_check.sh` **PASS** this session | **NO** |
| T6 Tier A harness | OPEN in README | **#311 MERGED** 2026-09-19 | **NO** reimplementation. **YES** run `scripts/tier_a_conformance.sh` for scoring |
| T7 Android staging | OPEN / PR #312 | **#312 MERGED** 2026-09-19. Ticket Status **STALE** (queue checker FAIL) | **NO** implementation. **YES** status fix + device verification using staged scripts |
| T8 diagnostics test | PR #271 open | **#271 MERGED** 2026-09-03 | **NO** |
| T9 PR burndown | OPEN | Still relevant — ~30 open PRs, many BEHIND/CONFLICTING | **YES** — disposition batch (§4), not bulk-merge |
| T10 FFI vacuous | RESOLVED ON MAIN | Verified | **NO** |
| T11 docs reconcile | OPEN | **#314 MERGED** 2026-09-19 | Status fix; residual doc claims if any |
| T12 CI filters | OPEN | **#319 + #328 MERGED** 2026-09-19 | Status fix; do not re-open path-filter work |
| T13 Rule-8 follow-ups | OPEN | Partially folded into later waves; **re-verify each F-item** against main before any paste | **MAYBE** — only residual F-items with live evidence |
| T14 ephemeral port | PR #270 open | **#270 MERGED** 2026-09-03 | Status fix; confirm external-addr filtering still active on tip |
| T14 pre-existing DHT | PR #269 open | **#269 MERGED** 2026-09-03 | Status fix |
| BJ beach-join | After current mission | Correct — **post-tag or post-mission** per operator | **NO until Wave 4 done** (unless operator moves it) |
| C4 | 0.5.0 | Correct | **NO** for 0.4.0 |
| Review dispatch 267/268/269/270/272/273/276 | "PASTE FIRST" | Reviews **filed**; several PRs merged | **NO** — do not paste completed review dispatches |

### New / remaining freebuff dispatch set to tag (this is the live queue)

Paste **only** these, in order, after the ticket files exist on **`main`**
(FREEBUFF.md: the lane reads `origin/main`, not an unmerged branch).

| Order | Task file (to ensure on main) | What it fixes | Lane | Gate |
|---|---|---|---|---|
| F1 | `V040_QUEUE_STATUS_RECONCILE_TAGPATH.md` (this wave) | Stale Status lines that misdispatch the lane (T7 + others; `check_queue_status.py` green) | Freebuff docs | none |
| F2 | `V040_T_CONN_LIMITS_MULTIPORT.md` | Live `connection_limits: limit 4 reached` storm; multi-port dial denial | Freebuff impl in `core/src/transport/` | **Rule-8 mandatory** |
| F3 | `V040_T_AND06_KOTLIN_HALVES.md` **only if operator picks A** | Move remaining Kotlin curve math behind UniFFI; wire or delete copies | Freebuff android + core export already exists | Rule-8 if core/FFI surface changes |
| F4 | `V040_T_WATCHDOG_POSITIVE_TEST.md` | N-03: healthy-quiet node must not be killed by log-silence watchdog | Freebuff `cli/` tests | none if no prod logic change |
| F5 | `V040_T13_RESIDUAL_VERIFIED.md` | Only residual T13 items still real on main (F1/F7/F-DHT leftovers) | Freebuff | Rule-8 |
| F6 | Open-PR disposition notes / close superseded | #215 (routing, CONFLICTING, superseded), #156 if Docker suite stays green, stale drafts | Orchestrator + freebuff docs | none |
| Score | Not freebuff | D4/D6/D7, churn, keystore, SEC-03, audit commission | Operator + orchestrator | — |

**Freebuff is not the scorer.** Device gates, tag, secrets, and SEC-03 stay
human/operator. Freebuff implements scoped diffs and returns evidence.

---

## 4. Open PR disposition (complete list, 2026-09-20)

`gh pr list --state open` (34 rows returned; no silent cap assumed — this is
the API response this session). Categories:

### 4a. Land before / with the tag candidate (functional)

| PR | State | Action |
|---|---|---|
| #325 chat-order tie-break | BEHIND, MERGEABLE | Update branch from `origin/main`, push, wait required checks, **orchestrator merge** if green. Operator-visible correctness |
| #322 outbox key canonicalization | UNSTABLE, MERGEABLE | Diagnose non-success checks; merge only when required contexts green. Touches store/CLI paths — pr_scope before merge |
| #329 queue status reconcile | BEHIND | Docs; merge **after** F1 tag-path status wave or fold F1 into it to avoid double docs PR |

### 4b. Docs / process that freebuff may finish

| PR | Action |
|---|---|
| #316 outbox retry diagnosis | Docs — merge or close if superseded by #322 |
| #303 merge-plan handoff draft | Close if superseded by this tag-path document |

### 4c. Work-ahead — **not 0.4.0 blockers** unless operator pulls them in

#298–#302 (Android join/seed/apk-host/QR/iOS share), #289 already merged
(re-check: if open base is stale, close or rebase separately). Keep **draft**
until after tag unless they fix a D-gate defect.

### 4d. Legacy / Apple / dependabot — operator batch decision

#207–#208 Apple docs; #209/#216/#220 identity unification family (some
CONFLICTING — do not force-push shared bases); #156 Docker nonblocking
(**premise weak**: Docker Integration Suite success on main tip); #215 routing
wire-up (**CONFLICTING + superseded by landed routing feed** → **CLOSE**);
dependabot wave #103–#141, #210–#214 — one batch ruling: merge if CI green
after rebase, else defer post-tag. **Do not bulk-merge CONFLICTING PRs.**

### 4e. Hard closes recommended now

| PR | Why |
|---|---|
| #215 | Superseded; production `routing_peer_seen` already on main; branch CONFLICTING |
| #156 | If operator agrees Docker Integration Suite is green/non-blocking in practice; otherwise keep with updated body |

---

## 5. Critical path to the tag (ordered)

```text
W0  Docs/queue unification on main (this package)
    + land #325 (and #322 when green)
W1  Operator parallel:
      - keystore verify + re-set secrets + release rehearsal green
      - SEC-03 dated decision
      - AND-06 A or B decision
      - commission external crypto audit
W2  Freebuff F2 (conn limits) + optional F3/F4/F5
    Orchestrator: Rule-8 where gated; merge after APPROVE
W3  Fleet cut to ONE candidate SHA (CI artifacts, not target/)
    Tier A harness + live scoring:
      G3-0 churn (or waiver)
      D4 on released/release-config APK
      D6 failover + routing confidence
      D7 offline proximity
      cloud custody parity
    Audit iteration 2 filed
W4  Freeze candidate; all §0 rows CLOSED or waived in this file
    Operator: git tag v0.4.0 (final, not rc) + release publish
    Orchestrator: fill SHIP_PLAN §5 ledger; retire superseded plans
    Freebuff README: post-tag order (BJ, C4, 0.5.0)
```

**Do not cut another `rc`/`alpha` tag expecting a public download** —
`release.yml` treats those as draft.

---

## 6. Freebuff procedure — efficiency plan (operator paste cost)

The lane has no headless mode (`docs/rules/FREEBUFF.md` §1). Efficiency =
fewer wrong pastes + everything the lane must see is **on `origin/main`**.

### 6.1 Structural fixes (this PR / follow-ups)

1. **Single source of truth:** this file for tag path; freebuff README becomes
   an *index* that links here and lists only DISPATCHABLE tasks.
2. **Status lines must pass `scripts/check_queue_status.py`** before any
   paste wave. Stale "PR #N open" lines are how T7 got picked as work-ahead
   after #312 merged (checker output this session).
3. **Move completed tickets to `HANDOFF/freebuff/done/`** with merge commit
   in the Status line. Keep `queue/` small enough that the operator can see
   the real remaining set without archaeology.
4. **Delete the "PASTE FIRST review dispatches" block** once those reviews
   are on main — it currently sends the operator into finished work.
5. **New tasks land on main before paste** (FREEBUFF.md §2 outbound path).
   Orchestrator merges HANDOFF-only dispatch packets promptly.

### 6.2 Paste protocol (operator + orchestrator)

| Step | Who | Rule |
|---|---|---|
| 1. Premise verify | Orchestrator | Before writing a task file: run the evidence command; if premise fails, do not write the ticket (wrong premise is the expensive failure mode) |
| 2. Scope correction | Orchestrator | Task file must say what **not** to "fix" nearby |
| 3. Index + main | Orchestrator | File in `queue/`, add README row, **merge to main** |
| 4. One task per paste | Operator | No bundling unrelated tickets |
| 5. Model | Operator | Unmetered only: DeepSeek V4 Flash / MiMo / GLM 5.3 Flash |
| 6. Implement + PR | Freebuff | Branch + PR; **no self-merge**; no force-push; no unrelated file commits |
| 7. Evidence | Freebuff | Every gate line = command + output, run URL, or `UNVERIFIED` |
| 8. Rule-8 | Orchestrator | If diff touches `core/src/{crypto,transport,routing,privacy}` — non-author APPROVE on file **before** merge |
| 9. Merge | Orchestrator | `scripts/pr_scope.sh <n>` first (rule 14); required contexts green; then merge |
| 10. Filing | Orchestrator | Ticket → `done/` with PR/merge SHA; README row updated; `check_queue_status.py` green |

### 6.3 Hard lane limits (do not ask freebuff to break these)

- No Pixel UI automation. APK install + passive logs only.
- No secrets, no tags, no releases, no worktree deletion.
- No commits of files the model did not create (shared checkout).
- No architecture/go-no-go/SEC-03/storage-engine choice.
- No claims of D-gate score — that is operator/orchestrator work.
- Host builds serialize — check for live builds before `cargo`/`gradlew`.
- Temp files only under repo `tmp/`.
- No emoji; `[OK]`/`[FAIL]` markers only.
- Never `cargo clean --target <triple>`; never read `$?` after a pipe without capture.

### 6.4 Repo rules that bind the orchestrator on this path

| Rule | Application to tag path |
|---|---|
| AGENTS 5 | Orchestrator may push branch updates and merge **verified** work; Freebuff workers do not inherit push/merge rights |
| AGENTS 8 | Transport/store/routing/crypto changes need Rule-8 before "done" |
| AGENTS 11 | Shared main worktree has dirty unowned files — **all new work in `tmp/wt-*` worktrees** |
| AGENTS 13/15 | Cite commands; no silent truncation of PR/check lists |
| AGENTS 14 | `scripts/pr_scope.sh` before merge; enumerate blockers out loud |
| AGENTS 16 | Wiring gate on any UI/restore change |
| AGENTS 17 | Disk budget before builds; node binaries in `tmp/radio-<sha>/`, not `target/` |
| SHIP_PLAN | D1–D7 only until tag; post-tag work does not start early |
| FREEBUFF §2 | Lane-visible artifacts must be on `main` |

---

## 7. Operator decision checklist (blocking, not agent-routable)

These four items cannot be closed by freebuff or by green CI alone.

1. **Keystore / D2:** run `scripts/verify_release_keystore.sh
   C:\Users\SCM\kiee\scmessenger-release.jks <alias>` interactively; if `[OK]`,
   ensure `SCMESSENGER_KEY_ALIAS` **value** matches exactly (PKCS12 case
   sensitivity); re-dispatch release rehearsal until signed APK job is green.
2. **SEC-03:** (a) dated waiver + owner + 0.5.0 migration plan **or**
   (b) authorize storage migration branch (post-tag OK if (a)).
3. **AND-06:** (A) finish Kotlin half before tag **or** (B) dated accept,
   migrate 0.5.0 — then ticket text and this file updated to match.
4. **External crypto audit commission** — firm/scope/price/dates on file.
5. **Optional waivers** to record here: AWS IP-churn autonomy (use runbook);
   BLE/cell legs as UNVERIFIED for the public alpha if desired.

---

## 8. Residual uncertainties (flattened, not hidden)

| Uncertainty | How it gets resolved | Until then |
|---|---|---|
| Alias value vs keystore | Operator interactive verify | D2 blocked; no public APK |
| Security Scan schedule red | Inspect gitleaks/audit job body; fix or confirm non-required | D1 still OK under current protection contexts |
| #322 UNSTABLE checks | Read failing job logs on the PR | Do not merge #322 |
| T13 residual scope | Re-verify each F-item against main | No blind paste of old T13 text |
| AND-06 Kotlin still in tree | Operator A/B | Audit may remain incomplete on that row |
| Fleet not on tip `efd240d7` | Redeploy from CI artifacts after W2 | Field scores must name SHA |
| connection_limits vs scoring | F2 fix + live WARN-rate drop | D4/D6 LAN scoring fragile |
| Shared-checkout dirty files | Leave untouched (rule 11) | Work only in new worktrees |

---

## 9. What "done" means for this planning package

- [x] Contradictions inventoried against `origin/main` + `check_queue_status.py`
- [x] Local scoring gates run on a clean main-based worktree
- [x] D1–D7 unified scoreboard
- [x] Freebuff live dispatch set replaced with verified remaining work
- [x] Freebuff procedure optimized for operator paste cost and rule compliance
- [ ] Package merged to `main` so the lane can see it
- [ ] Operator decisions §7 recorded
- [ ] W2–W4 executed and evidenced

End of unified path. Supersedes stale queue order; does not delete history.
