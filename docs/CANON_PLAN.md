# Canonical Planning Tracker

**This is the single planning source of truth.** No parallel plans — every
workstream tracks here. Historical plan docs in `docs/` remain as archives;
new planning goes here, not in new files.

## Active workstreams

| Workstream | Status | Tracker |
|---|---|---|
| v0.5.1 merge train | In progress | Issue #595 |
| v0.5.1 release | Blocked on train | Issues #581, #582, #603 |
| Connection admission (#521 train) | In progress | Issues #577, #578; PRs #521, #599 |
| Ubicloud runner migration | In progress | Issue #607; PR #608 |
| CI cost reduction | In progress | This batch (#610–#615) |

## Archived plans

The following docs are historical records, not active plans. Do not extend them;
open or update an issue instead, and link it here.

- ANDROID_COMPLETION_PLAN.md, AUTO_UPDATE_PLAN_ALPHA.md, CROSS_OS_BUILD_PLAN.md
- DSP_SCM_PLAN.md, GLOBAL_ROLLOUT_PLAN.md, HTTP_TRANSPORT_PLAN.md
- MILESTONE_PLAN_V0.2.0_ALPHA.md, NAT_TRAVERSAL_PLAN.md, P2P_CONNECTION_PLAN.md
- PEER_SERVED_SELF_UPDATE_PLAN.md, SCMessenger_V1_Readiness_and_Takeover_Plan_2026-09-13.md
- SCRIPT_IMPLEMENTATION_PLAN.md, SPRINT_PLAN_OPTIMIZATION_BUNDLE.md
- TEST_PLAN_KMP_DESKTOP.md, UNIFFI_0.31_MIGRATION_PLANNING.md
- UNIFIED_GLOBAL_APP_PLAN.md, V0.2.0_API_35_REGRESSION_TEST_PLAN.md
- V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md, WS12_GATE_CLOSURE_AND_STORE_ROLLOUT_PLAN.md

## Rules

1. One active plan per workstream, tracked here and in its issue.
2. Plans live in issues (discussable, closeable); this doc indexes them.
3. When a workstream completes, move its row to "Done" with the merge/tag SHA.

## Roles state substrate

Installed from `docs/roles/state-seed.md` in response to the Landing request of
2026-10-22T10:00:45Z (#602). Each section below has exactly one writer role.
`docs/CANON_PLAN.md` already existed on main; the sections are appended, and no
existing line was replaced.

## QUEUE

Writer: Landing only.

States: OPEN, READY, QUEUED, IN-TRAIN, REVIEW-PENDING, REVIEW-BLOCKED,
BOUNCED, STUCK, HELD, LANDED. Release: TAGGED, GATE-RUNNING,
GATE-PASSED, GATE-FAILED, PUBLISHED. MAIN-RED is a global flag, recorded
in the OPERATION JOURNAL, not here.

Installed live queue -- derived 2026-10-23T05:05Z from the open-PR list and
each PR's per-SHA check-runs at main `946400c62`. Green = every completed check
`success`/`skipped`. Failing = at least one `failure`. `cancelled` and pending
are unverified, never a pass. Train order per issue #595: CI and harness first,
then green cars, then the Rule-8 train, then dependabot, then held.

| PR | State | Batch | Review | Notes |
|----|-------|-------|--------|-------|
| #636 | QUEUED | A3 | none | Docker suite harness repairs (audit tool + cdylib build); 23 green, 4 queued on starved self-hosted,oci. Head cadb44dd4, behind 0. |
| #601 | OPEN | fix-forward | none | Route required checks to the OCI runner; 32 green, 1 failure, 1 queued. Head 7f93eed6, behind 13. |
| #609 | STUCK | ci-batch | none | Five core jobs to Ubicloud; 23 green, 4 cancelled (unverified). Head 4ed8168b, behind 0. |
| #611 | READY | ci-batch | none | cargo nextest swap; 26 green, 0 failing. Head c85eb7fe, behind 7. |
| #508 | READY | train-1 | none | conn-fanout diagnostic tool; 29 green. Head c6bb8b53, behind 3. |
| #531 | READY | train-1 | none | dependabot androidx core-splashscreen 1.2.0; 27 green. Head 2c5adef1, behind 0. |
| #536 | READY | train-1 | none | dependabot rustc-hash 2.1.3; 33 green. Head f95dd9e9, behind 0. |
| #629 | READY | dependabot | none | rust-dependencies group, 20 updates; 26 green. Head 157af161, behind 0. |
| #632 | READY | dependabot | none | argon2 0.6.0; 26 green. Head dbed5380, behind 0. |
| #634 | READY | dependabot | none | dirs 7.0.0; 27 green. Head 220297013, behind 0. |
| #635 | READY | dependabot | none | thiserror 2.0.19; 26 green. Head 1d655404, behind 0. |
| #506 | STUCK | train-1 | none | API CORS loopback + jna path; 22 green, 7 cancelled (unverified). Head 81061b5b, behind 5. |
| #507 | STUCK | train-1 | none | Android BLE write-type; 19 green, 7 cancelled (unverified). Head 2c0f9924, behind 5. |
| #509 | STUCK | train-1 | none | Seeding security hardening; 31 green, 1 cancelled (unverified). Head 32e64212, behind 5. |
| #510 | STUCK | train-1 | none | iOS durable delivery re-port; 19 green, 10 cancelled (unverified). Head 9c30fc4e, behind 5. |
| #550 | STUCK | train-1 | none | StoreAndCarry outbox retry + receipts; 32 green, 2 cancelled (unverified). Head 6441203a, behind 0. |
| #554 | STUCK | train-1 | none | Canonical recipient hint binding; 29 green, 4 cancelled (unverified). Head 5b3efc47, behind 0. |
| #529 | OPEN | train-1 | none | dependabot setup-java 6; 2 failures at `Install protoc (Windows)` (runner-side), 2 queued. Head 7648aba3, behind 0. |
| #530 | OPEN | train-1 | none | dependabot cache/restore 6.1.0; 1 failure at `Install protoc (Windows)` (runner-side). Head b15cd71e, behind 0. |
| #533 | OPEN | train-1 | none | dependabot docker/login-action 4; 1 failure at `Install protoc (Windows)`, 2 queued. Head 3060289d, behind 0. |
| #534 | BOUNCED | train-1 | none | dependabot zxing 3.5.4; Android JVM Unit Tests fails `ContactsViewModelTest` (553 tests, 1 failed), plus `libscmessenger_core.so` load failure. Head 9c049fcf, behind 0. |
| #537 | OPEN | train-1 | none | dependabot docker/metadata-action 6; 2 failures at `Install protoc (Windows)`, 2 queued. Head 1348c60f, behind 0. |
| #556 | OPEN | train-1 | none | Network-change signal into core (#523); 1 failure at `Install protoc (Windows)`, 5 cancelled. Head 9282583a, behind 0. |
| #521 | REVIEW-BLOCKED | train-2 | NEEDS_WORK | Dynamic admission; 40 green, 2 failing. Blockers #573-#578. Head 230b6169 unchanged, behind 17. |
| #599 | HELD | train-2 | approval withdrawn | Closes #521 review blockers; stacked on #521, 8 green, 0 failing. Head 3362c1f3, behind 17. |
| #489 | HELD | -- | Rule-8 | Conn-cap v2; 37 green, superseded by #521, close after it lands. Head fc3540e4, behind 43. |
| #216 | HELD | -- | none | Draft; Android feature wiring + passphrase. 3 failing, blocked on identity-id-unification. Head b2020a20, behind 761. |
| #528 | BOUNCED | dependabot | none | dtolnay/rust-toolchain 1.120.0; 7 failing, 3 queued. Head 80603842, behind 0. |
| #535 | STUCK | dependabot | none | tower-http 0.7.1; 30 green, 3 cancelled (unverified). Head 32bb0fae, behind 0. |
| #538 | BOUNCED | dependabot | none | x25519-dalek 3.0.0; 14 failing -- needs rand_core 0.10 / curve25519-dalek 5.0. Head 990d7ad2, behind 3. |
| #540 | BOUNCED | dependabot | none | Kotlin 2.4.20; 3 failing -- KSP has no 2.4.x release. Head 77bbec27, behind 3. |
| #541 | QUEUED | dependabot | none | androidx security-crypto 1.1.0; 26 green, 1 queued on self-hosted,oci. Head 91e3d5a7, behind 0. |
| #542 | BOUNCED | dependabot | none | navigation-compose 2.10.2; 2 failing -- needs AGP 9.1.0 + compileSdk 37. Head b3e1fe82, behind 3. |
| #630 | BOUNCED | dependabot | none | zbus 5.19.0; 2 failing. Head 5811dfc1, behind 0. |
| #631 | BOUNCED | dependabot | none | uniffi 0.32.2; 6 failing, 1 queued. Head 6d144c12, behind 0. |
| #633 | BOUNCED | dependabot | none | rand 0.10.2; 14 failing. Head 4a8d7614, behind 0. |
| #178 | BOUNCED | -- | Rule-8 | API limit OSX/iOS; 3 failing. Head c1b457bb, behind 1091. |
| #208 | BOUNCED | -- | none | Apple 4-node parity status; 6 failing. Head 11f3e5e9, behind 761. |
| #220 | BOUNCED | -- | none | Android reachability fixes; 2 failing. Head 92b8ec12, behind 761. |

## LEASES

Writer: the leaseholder for its own row; the Overseer for revocations.

| Role | Holder (session) | Epoch | Since (UTC) | Status |
|------|------------------|-------|-------------|--------|
| Implementer | -- | 0 | -- | vacant |
| Landing | -- | 0 | -- | vacant |
| Shepherd | -- | 0 | -- | vacant (on demand) |
| Reviewer | -- | 0 | -- | vacant (per request; a review takes a short lease naming the PR + head SHA) |
| Overseer | -- | -- | -- | function, not a session -- no lease row is ever held |

Revocation log:
- (none yet)

## HEARTBEATS

Writer: each role appends only its own lines.

Format: `HEARTBEAT <role> epoch <n> <ISO-8601 UTC> queue <queued/in-train/held counts> last-action <one line>`

- (none yet -- a role's first heartbeat confirms its lease)

## OPERATION JOURNAL

Writer: Landing (Shepherd updates entries for its assigned PRs only).

Landing's live journal: active batch membership, each IN-TRAIN PR's head
SHA, pending CI waits, MAIN-RED declarations and clears, GATE-REQUESTs.
A replacement leaseholder resumes from this journal, not from scratch.

- 2026-10-23T05:05Z -- state seed installed into this doc at main `946400c62`.
  No seeded section existed before, so nothing live was reconciled or replaced;
  the QUEUE table was re-derived from the open-PR list and per-SHA check-runs
  rather than pasted from the seed appendix. Installing PR: docs(canon): install
  roles state seed.

## OWNER QUEUE

Writer: Overseer only. Roles never write here and never ask the owner
technical questions directly; owner-bound items reach the Overseer, which
frames them as end-state or ramification questions (OWNER-ITEM) and
records the owner's answer verbatim (OWNER-RESOLVED).

| # | Opened (UTC) | Item (end-state / ramification) | Blocked | Proceeds meanwhile | Status |
|---|--------------|----------------------------------|---------|--------------------|--------|
| -- | -- | -- | -- | -- | -- |

## AUTHORSHIP LOG

Writer: Implementer only (sole writer).

Every session + PR + branch where code was authored, including WIP
branches and shepherded rebases by other sessions on its PRs (recorded
on the acting session's request). Landing and Reviewer check authorship
exclusions against this log; GitHub history is the fallback if disputed.

| Session | PR / branch | What | Date (UTC) |
|---------|-------------|------|------------|
| Freebuff Implementer | #636 / fix/docker-suite-harness | docker harness repairs: cargo-audit in the rust-test image, cdylib build before gen_kotlin (7 lines, 2 files) | 2026-10-22 |

## EXPERIENCE

Lessons live in `docs/EXPERIENCE.md` (append-only; any role appends a
LESSON entry: date, role, one-line lesson, evidence link). This section
is only a pointer -- do not duplicate lessons here.

[INFO] Governing train plan: `HANDOFF/plans/V051_MERGE_TRAIN_PLAN_V7_2026-10-22.md` (v7, with the 2026-10-23 Overseer ERRATA section appended to it).
