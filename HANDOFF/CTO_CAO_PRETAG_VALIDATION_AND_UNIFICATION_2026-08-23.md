# Pre-tag Validation Ledger and Unification Pass -- 2026-08-23

Status: Active
Created: 2026-08-23
Owner: Interim CTO/CAO (Claude seat)
Purpose: claim-by-claim validation of the state the four-node gate relies on,
plus the repo unification pass the operator requested. Every VALIDATED entry
carries the command/output observed this session. Companion to
`HANDOFF/plans/FOUR_NODE_GATE_EXECUTION_PLAN_2026-08-23.md`.

Method note: this was a read-only audit plus two authored handoff files. No
code was changed. Where a claim could not be verified it is recorded UNKNOWN
with its owner -- never treated as safe.

---

## Part 1 -- Validation ledger

### 1.1 VALIDATED (evidence obtained 2026-08-23)

| # | Claim | Evidence |
|---|---|---|
| V1 | `main` HEAD = `e5ff72cf`; latest tag is v0.3.5; NO v0.4.x tag exists | `git ls-remote --tags origin`; `git log origin/main --oneline -3` |
| V2 | Main is green: CI, Lint, Cross, Docker Publish, Docker Integration Suite, Repository Hygiene, CodeQL all `completed/success` on e5ff72cf | `gh run list --branch main` |
| V3 | Mobile + iOS lanes green on `b538f3ba` (parent of e5ff72cf); PR #226 merged ONLY `HANDOFF/CTO_STATE.md` + `SHIP_PLAN.md`, so e5ff72cf is code-identical to b538f3ba | `gh pr view 226 --json files`; workflow run list. Precision: "nine lanes on e5ff72cf" is seven-on-e5ff72cf + two-on-parent-docs-only. Risk: none identified |
| V4 | Gate A PR states: #221 OPEN/DRAFT BEHIND/MERGEABLE; #222 OPEN/DRAFT BEHIND/MERGEABLE; #227 OPEN/DRAFT UNSTABLE; #220 OPEN wiring-gate-red (expected findings); #219 OPEN/DRAFT BEHIND | `gh pr view 219,220,221,222,227 --json ...` |
| V5 | #219 RED confirmed: `Lint` fail + `Rust Linting` fail (runs 32617007302 / 32617007321) | `gh pr checks 219` |
| V6 | Release machinery present and tag-triggered (`on: push: tags: v*`); builds linux/macos-amd64/macos-arm64/windows CLIs incl. asset name `scm-windows-amd64.exe`; Android keystore secrets wired (`SCMESSENGER_KEYSTORE_BASE64` etc.) with apksigner verify step | Read `.github/workflows/release.yml` |
| V7 | Version tag-gate passes NOW: `[OK] Cargo/Android/Desktop/WASM agree at 0.4.0; iOS marketing version is 0.4.0; build numbers exceed baselines` exit 0 | `bash scripts/verify_versions.sh` via Git bash, EXIT=0 |
| V8 | README.md exists on origin/main (~80 lines, non-empty) -- SHIP_PLAN S2-1 "currently 0 bytes" is INVALIDATED/superseded | `git show origin/main:README.md \| Measure-Object -Line` -> 80 lines |
| V9 | A7 work intact and uncommitted: worktree `C:/Users/SCM/Documents/GitHub/_scm_wt/cihard`, branch `cto/ci-hardening-2026-08-23` based on e5ff72cf, 8 modified + 2 untracked files matching the stand-down inventory PLUS `.github/workflows/security-regression-tests.yml` | `git -C <wt> status --short`; `git worktree list` (NOTE: worktree lives OUTSIDE the repo dir, not `SCMessenger/_scm_wt`) |
| V10 | A7 perimeter-underscore scanner policy sound; all three transport-file allowances legitimate: multiport `_port` read under cfg(unix); wifi_aware/wifi_direct are test mocks + one documented deliberate no-op referencing P1-15/P1-17 docs. None is a second sender-auth-class hole | Read full diffs of the three transport files |
| V11 | A7 negative-test job handles both stand-down traps: `forg` substring selector (survives test-name drift; already matches 2 pre-existing forg tests so cannot be silently green-on-zero today) AND hard-fails when selected+ran count = 0 | Read `.github/workflows/security-regression-tests.yml` |
| V12 | A6 BLOCK-1 real at source: libp2p-request-response-0.29.0/src/lib.rs line 670 `.expect("Expected some established connection to peer before closing.")`, line 676 `.expect("Expected connection to be established before closing.")`, line 678 `debug_assert_eq!` (release-stripped) | Read registry source at `$env:USERPROFILE\.cargo\registry\src\...\libp2p-request-response-0.29.0\src\lib.rs` |
| V13 | A6 BLOCK-2 targeted check is feasible: ladder log line exists at `core/src/transport/swarm.rs:6095` (`tracing::debug!("Dialing candidate ladder for {}: {:?}"...)`); dial_policy implements max-3 concurrent dials per peer keyed map | Select-String on swarm.rs, dial_policy.rs |
| V14 | N4 AWS relay LIVE + healthy: `HTTP 200 {"status":"healthy"}` at http://54.226.67.101:9876/health | Invoke-WebRequest 2026-08-23 |
| V15 | N4 image STALE for the gate: address doc records image built at 6b2573fa (pre-#139); gate requires tag-SHA image; Docker Publish lane succeeds on main so fresh images exist in registry | `HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` + V2 |
| V16 | N3 Windows relay RUNNING (PID 16156, since 08-22 15:34), listening multiport 9001/9002/9090/8080/80/443 + 127.0.0.1:9876, and ESTABLISHED to 54.226.67.101:9001 (meshed with N4 right now) | Get-Process + Get-NetTCPConnection |
| V17 | Unify wave complete: all four `feat/unify-*` branches are 0 commits ahead of origin/main (merged or empty) | `git rev-list --count origin/main..<branch>` x4 |

### 1.2 INVALIDATED / STALE claims (do not act on them)

| Claim | Where it lives | Truth 2026-08-23 |
|---|---|---|
| "#227 verified green" | CTO_STATE stand-down table A3 | **[DRIFT]** `Android JVM Unit Tests` FAILING on run 32670592900 (2026-08-23T22:47Z): `MeshRepositoryTest > isStorageDegraded initial state is false`, ClassCastException `java.lang.Object` -> `android.net.ConnectivityManager` in JVM (non-Robolectric) context, plus NPE in NetworkRequest.Builder chain. New failure, post-dates the state file |
| "README currently 0 bytes" | SHIP_PLAN S2-1 | README exists on main (~4 KB). D3 partially satisfied; content review still owed |
| "tracked md files: 1,695" | SHIP_PLAN governance section | Now 1,733 tracked (`git ls-files -- "*.md"`). Sprawl continues slowly |
| "five-node gate" as the target | older plans (V040_V050_FIVE_NODE_GATE_PLAN_2026-08-05, FIVE_NODE_UNIFIED_TEST_PLAN_2026-08-09, FIVE_NODE_RUN_2_PLAN, UNIFIED_MASTER_MERGED_PLAN_2026-08-10) | Superseded by CTO_STATE 0-2026-08-23b four-node gate; those files remain historical harness/evidence sources only |
| "AWS always-on node teardown+rebuild gated on Docker Publish" | _QUEUE 2026-08-05 header | Docker Publish now green (V2); the gate has MOVED: what remains unproven is the redeploy mechanics themselves (SSH-key absence recorded 2026-08-05) |

### 1.3 UNKNOWN -- verify-or-disposition BEFORE the tag (each needs an owner)

| # | Item | Why it matters to the gate | Suggested action |
|---|---|---|---|
| U1 | `P0_ANDROID_FINITE_RETRY_ABANDONMENT_2026-08-10.md` still Status: Active, filed against anchor 68fcc3f1 | PF-1 freeze blocker: an accepted undelivered message abandoned at attempt threshold would corrupt D4/D6 scoring | Re-verify outbox/retry behavior on current main; disposition FIXED / STILL-OPEN / OBSOLETE with evidence |
| U2 | `P0_ANDROID_SELF_RATCHET_RESET_2026-08-10.md` Status: Active (mDNS-loss resets own ratchet) | Crypto-state corruption class; would surface as inbound decrypt failures during D7/BLE churn | Same re-verify pass |
| U3 | `ANDROID_INBOUND_CRYPTOERROR_2026-08-09.md` P1 Active (840 drops/31h at old anchor) | D4 receiver-side decrypt evidence could be polluted | Same re-verify pass |
| U4 | `P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_2026-08-10.md` Open, "queued behind five-node anchor" | Sender-convergence is half of delivery truth scoring | Confirm fixed by receipt round-trip work (iron_core classify path) or schedule post-tag |
| U5 | GitHub signing secrets presence | First tag attempt proves it; a red release lane on tag day costs hours | Optional pre-check by admin; otherwise accept first-tag risk |
| U6 | N2 second Android handset availability | D4 cross-network leg needs two handsets | Operator logistics |
| U7 | Does #222 alone stop N3 identity churn? (decides whether #219 joins Gate A or closes) | A5 fork in the execution plan | Local repro: start relay, run CLI identity query twice against locked storage, compare PeerIds on #222 branch |
| U8 | AWS N4 redeploy mechanics without SSH key | Deploy step 7 of the plan | Prove teardown+rebuild (EC2 API/user-data pull of tag image) BEFORE tag day |
| U9 | A7 lint proof-of-fire not yet demonstrated | A lint not proven to fire is not a lint | Deliberate violation -> paste CI/local failure -> revert -> paste pass; then commit+push A7 |
| U10 | Apple lane real capability (what iOS app can do TODAY) | Determines whether N5 join is realistic inside the gate window | One artifact: `scm-macos-arm64 --version` at tag hash + one send attempt (AW-BILAT-0003 item 6.1) |

---

## Part 2 -- Unification pass (operator-requested)

Scope: where/how/if the repo needs unifying, for 0.4.0 / 0.5.0 / 1.0.0 awareness.
This is an assessment, not a dispatch queue -- nothing below blocks the tag
unless marked otherwise.

### 2.1 State of prior unification efforts

| Effort | State | Evidence |
|---|---|---|
| July unify wave (`feat/unify-cli-dedup`, `-batch-b`, `-ironcore-retire`, `-swarm-topics` in worktrees scm-unify-a/b/c/c2) | LANDED -- 0 ahead of main; worktree registrations are residue | rev-list counts (V17). Reclaim candidate AFTER `scripts/reclaim_safe.py` verification; deletion is gated, not mine |
| `HANDOFF/todo/CODEBASE_UNIFICATION_PLAN.md` (ranked duplication audit) | STALE anchor: audited at 6761ac4 (2026-07-26), predates #139 merge + CRLF renormalization + identity work. Ranks 1a/1b/1c etc. need re-verification against e5ff72cf before any dispatch | Header of that file |
| `LEDGER_CHOKE_POINT_REFACTOR.md` (cross-referenced) | Not re-checked this session | -- |
| `docs/CURRENT_STATE.md` | Superseded until tag (by design), last VERIFIED 2026-07-21 -- three weeks of drift | Header |

### 2.2 Current duplication/drift findings (this session)

1. **Authority-chain drift is the top unifier.** Execution authority is now:
   SHIP_PLAN.md <- CTO_STATE.md (four-node section) <- AW-BILAT-0003 <- the two
   files authored today. At least five older plan docs still describe five-node
   or two-node premises and none carries a superseded-banner pointing at the
   current chain (only _QUEUE.md and CURRENT_STATE.md have banners). Cheapest
   durable fix: one-line banner atop each superseded plan naming its replacement.
   This is exactly the "disposition made against a premise, premise changed"
   failure class that produced A6.
2. **HANDOFF/todo holds ~26 files of mixed validity** (Aug-10 P0/P1 tickets
   undispositioned vs main; INBOX_* files from 08-11; seeding/gossip ticket).
   Section-1.3 items U1-U4 are the subset that can bite the gate. Post-tag,
   re-run an amnesty-style sweep WITH disposition lines (POST_TAG_QUEUE section 5 rules).
3. **Contested-file pattern**: `HANDOFF/gpt/CTO_TO_CAO.md` remains contested
   (uncommitted working-tree edit by another session). Protocol already says
   use dated files; keep honoring it.
4. **Worktree sprawl**: 40+ registered worktrees across three roots
   (`_scm_wt/`, repo-root ad-hoc dirs `scm-*`, `tmp/orchestration/worktrees/`,
   `.claude/worktrees/`, `.qwen/worktrees/`). Several hold merged branches
   (reclaim candidates) and at least one ghost (`e01c-pq-mixing`, known).
   Unification action: post-tag reclaim sweep using reclaim_safe.py verdicts;
   do NOT bulk-delete before that.
5. **Doc volume**: 1,733 tracked markdown vs ~120k lines Rust (SHIP_PLAN ratio
   roughly held). Governance rule "new handoff docs require a reason beyond
   context transfer" stands; today's two files carry reasons (validation ledger
   + execution prep requested by operator).
6. **Version/metadata unity is GOOD**: verify_versions.sh proves Cargo /
   android/build.gradle / iOS plist+pbxproj / wasm / desktop all agree at 0.4.0
   (V7). No action.

### 2.3 Unification backlog for 0.5.0 / 1.0.0 (post-tag unless said otherwise)

| Item | Horizon | Note |
|---|---|---|
| Re-verify CODEBASE_UNIFICATION_PLAN ranks against current main; retire CONFIRMED-no-longer rows | early 0.5.0 | Three adversarial BLOCK verdicts traced to this class historically |
| Superseded-banners on the five stale plan docs | pre-tag (cheap, docs-only) | Prevents the next reader from executing a dead premise |
| HANDOFF/todo disposition sweep w/ Disposition lines | post-tag | Per POST_TAG_QUEUE section 5 |
| Worktree reclaim sweep (reclaim_safe.py verdicts first) | post-tag | Disk is the binding constraint on this machine |
| iOS/macOS parity wave (U6 receipts, APP_SHARING parity, cross-install) | 0.5.0 core scope | Already scoped in POST_TAG_QUEUE S4-3 + APP_SHARING ticket |
| Farm drills FD-*, hostile-network rig closure (P1-14/P1-18 debt), meeting mode | 1.0.0 | FARM_FINAL_PLAN + _QUEUE farm sections remain the reference |
| External crypto audit completion + PQC follow-ons shaped by it | gates 0.4.0 final -> S4-5 | Commissioned-status is the publish gate, not completion |

---

## Part 3 -- Confidence statement (honest)

- Plan/readiness validation for the four-node gate: **high (>=99%) that the
  documented state is accurate as of 2026-08-23**; every load-bearing claim was
  checked with a command this session, drift found was corrected above.
- Gate OUTCOME confidence is deliberately NOT claimed: it is a function of
  Gate A landing, the U-list dispositions, and field behavior. That is what the
  gate exists to measure.
