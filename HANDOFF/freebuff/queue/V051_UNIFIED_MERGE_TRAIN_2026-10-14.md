# V051 unified merge train: land all remaining work on main, tag v0.5.1

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Repository:** SCMessenger
**Status:** ACTIVE
**Priority:** P0
**Lane:** Freebuff (orchestration lane, push and merge rights per D3)
**Created:** 2026-10-14
**Last-updated:** 2026-10-14
**Owner:** operator
**Supersedes for execution:** `V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md` (kept as history; v0.4.0, v0.4.1 and v0.5.0 are tagged)
**Trackers:** issue #452 (residual open PRs), issue #567 (desktop-only items)

## 1. Operator decisions (2026-10-14, from the operator directly)

- **D1.** `v0.5.0` is already tagged on origin (lightweight tag on `df3bd3af4`, committed 2026-10-05 -10:00, which is 2026-10-06 UTC; `integrate: consolidate 55 train PRs (0.5.0)`) and is never rewritten. The unified release target is **v0.5.1**: land ALL remaining work (residual open PRs tracked by #452, the open queue cars, and the useful parts of the preserve snapshot) on main, then tag v0.5.1. At queue creation main still reads 0.4.1 (`Cargo.toml:12`, `android/build.gradle:24-25`); the bump to 0.5.1 is car F-1 below, superseding PR #471.
- **D2.** Merge policy: a PR merges when green: CI green on the exact head SHA, no conflicts, and required reviews / Rule-8 adversarial review satisfied for `core/src/{crypto,transport,routing,privacy}/`. Land everything, one car at a time, in dependency order.
- **D3.** The Freebuff lane is the orchestration lane for this train, with full permission to push branches and merge PRs to main once they are green per D2. Protected-path cars still need `crypto-security-auditor` / Rule-8 sign-off before merge.
- **D4.** Preserve snapshot `e4050f8` (branches `preserve/shared-checkout-20261003` and `-ekmihd`) is a NEVER MERGE backup. Its useful parts land only as separate cars cut from current main: MT-05 (D9/D1 vendored `libp2p-swarm-0.48.0` fix, Rule-8 gated); MT-07 (OUTBOX-SWEEP-001) already landed as #413. Desktop-only branches `wip/session-close-20260929`, `archive/recovery/*` and the ~90-path staged candidate must be verified and pushed by the operator from the desktop (issue #567); this lane does not touch them.
- **D5.** Operator decision D5 (2026-10-14): PR #521 (dynamic admission, evict-don't-deny, derived per-peer path budget) is the chosen connection-admission design. Rationale: repo philosophy is no hard/static connection caps, only dynamic allowance; caps must never break messaging. This overrides the earlier CTO constraint quoted in #489 (keep `MAX_ESTABLISHED_PER_PEER=16` and its floor assert). Consequences: #521 is the MT-05 vehicle (carries the D9 vendored `libp2p-swarm-0.48.0` patch); #489, #372 and #364 become superseded-close AFTER #521 merges, once their uncarried remnants are checked (e.g. MeshServiceViewModelTest fix from #372). Gates before #521 merges: update to current main; CI green on exact head (NAT Simulation Tests failed and Full Test Suite was cancelled on the 2026-10-13 run, unresolved); Rule-8 adversarial review by a reviewer independent of the author account, per `docs/rules/SECURITY_PROTOCOL.md` (prior sign-offs were posted from the author's account).

## 2. Gates for every car

1. CI green on the exact head SHA (re-check after every rebase or push).
2. `cargo test --workspace --no-run` for any car touching code (CLAUDE.md compile gate); `build-verify` for the full scope. One build tool at a time (shared checkout).
3. `scripts/pr_scope.sh` run against the PR; scope must match the car.
4. `bash scripts/docs_sync_check.sh` for any car touching docs.
5. Rule-8: adversarial review by an uninvolved reviewer for any change under `core/src/{crypto,transport,routing,privacy}/` (see `docs/rules/SECURITY_PROTOCOL.md`). Marked R8 below.
6. No force-push to main; no rewrite of existing tags; merge one car at a time, then re-verify main is green before the next.
7. Cars never merge the preserve snapshot or any branch containing it.

## 3. Carry-over cars (status of the V040_V050 file's cars)

Verified against origin/main (2677ea38) and PR state on 2026-10-14:

| Car | Scope | PRs | Gate | Status |
|---|---|---|---|---|
| MT-03 | P0 umbrella WP1-WP4 | #383, #352, #349, #355, #356 | n/a | DONE: all merged (#383 11b30c85f 2026-10-03; #352, #349, #355, #356 merged 2026-10-06 in the 0.5.0 integration) |
| MT-06 | Android lifecycle (AND-SS-001, STOP-TEARDOWN-TIMEOUT-001) | #351, #367 | android-qa | DONE: #351 and #367 were CLOSED UNMERGED; the work landed instead as #432 (30caacc36, AND-SS-001 serialize start/stop), #515 (bounded stop path) and #519 (re-ensure FGS); `StopStartFloodTest.kt` is on main. Residual #364 lifecycle diff not checked here |
| MT-07 | Periodic outbox sweep (OUTBOX-SWEEP-001) | #413 | R8 (swarm.rs) | DONE: #413 merged 2026-10-09 (f7b8fc22f); `sweep_outbox_over_swarm` is in `core/src/transport/swarm.rs` on main |
| MT-04 | D6 routing feed on ConnectionEstablished | unknown | R8 | UNVERIFIED: likely covered by MT-03 WP2 (commit 3c899062 `v050-wp2-routing-feed-all-transports`); #514 is observability only. First step: grep main for the feed and mark done or open a car |
| MT-05 | D9 + D1 transport, vendored `libp2p-swarm-0.48.0` fix | #521 (land); #489, #372, #364 (superseded-close after #521 merges) | R8 (independent reviewer) | DECIDED (D5): #521 is the vehicle; carries the vendored D9 patch; not yet on main |
| MT-08..MT-12 | #359 remainder; defect tickets; tooling; housekeeping; drift guard | #359 (closed unmerged) | per item | UNVERIFIED: not checked against main in this pass |
| P0-8 BK-01 | Back up everything to GitHub | operator tool | operator | operator-gated |

MT-05 decision: resolved by operator decision D5 (section 1). #521 is the surviving set. #489, #372 and #364 are not merged; each closes as superseded only after #521 merges, once its remnants are confirmed carried (for #364, its remaining hunks such as Dockerfile and `per_peer_cap.rs`; for #372, the MeshServiceViewModelTest fix). Until #521 merges, #489, #372 and #364 stay open and unmerged. #364 predates #413 and #432, which already landed its sweep and lifecycle halves.

## 4. Residual cars (open PRs against main on 2026-10-14)

Two further open PRs, #220 and #216, target base `feat/identity-id-unification` (not main) and are out of this table; they follow #209. Dispositions: land (merge when green), rebase (needs rebase first), superseded-close, decide (operator or overlap ruling). CI/mergeable state was NOT checked when this table was written; the lane re-checks per D2.

| Car | PR | Title (short) | Gate | Disposition |
|---|---|---|---|---|
| R-01 | #503 | gitignore .claude/worktrees/ | none | land |
| R-02 | #504 | hardened ffi_surface.sh | none | land |
| R-03 | #505 | PR disposition ledger tooling | none | land |
| R-04 | #506 | API CORS loopback + jna.library.path | review, android-qa (android/app/build.gradle) | land |
| R-05 | #507 | Android BLE write-type | android-qa | land |
| R-06 | #508 | conn-fanout diagnostic tool | android-qa (MeshServiceViewModel.kt) | land |
| R-07 | #509 | seeding security hardening | R8 (core/src/transport/swarm.rs) | land |
| R-08 | #510 | iOS durable delivery (re-port) | review (iOS only, no protected path) | land |
| R-09 | #511 | agent drift-guard tooling | none | land |
| R-10 | #500 | docker: retry NDK download | none | land |
| R-11 | #562 | ios CI: drop duplicate macOS test | none | land |
| R-12 | #550 | StoreAndCarry outbox retry, receipts on dup | review, android-qa (core/src/store/outbox.rs, iron_core.rs not protected paths) | land |
| R-13 | #556 | wire network-change signal into core (#523) | review, android-qa (core/src/mobile_bridge.rs not a protected path) | land |
| R-14 | #554 | canonical recipient hint, identify hint binding | R8 (routing/local.rs, routing/optimized_engine.rs, transport/swarm.rs) | land |
| R-15 | #521 | dynamic admission, evict-don't-deny | R8 (independent reviewer) | land (MT-05, D5) |
| R-16 | #209 | unify multi-flavor identity ID | review, android-qa (21 android files) | rebase |
| R-17 | #208 | docs(apple): 4-node parity status (fork) | none | decide |
| R-18 | #178 | API limit OSX/iOS (fork, RULE-8) | R8 (core/src/transport/ble/gatt.rs) | decide |
| R-19 | #471 | chore: version 0.5.0 | none | superseded-close (by F-1) |
| R-20 | dependabot group | #528-#542 (15 PRs) | android-qa for #531, #534, #540, #541, #542; none for the rest | land |

R-20 dependabot group (#528 rust-toolchain, #529 setup-java, #530 cache/restore, #531 splashscreen, #532 bincode 3.0.0, #533 login-action, #534 zxing, #535 tower-http, #536 rustc-hash, #537 metadata-action, #538 x25519-dalek 3.0.0, #539 libc, #540 kotlin 2.4.20, #541 security-crypto, #542 navigation-compose). Land as one car, one PR at a time, lowest risk first (actions, patch bumps, then majors). Major bumps (#532, #535, #536, #538, #540) need a green full build. #538 (x25519-dalek 3.0.0) changes only `Cargo.toml` and `Cargo.lock`, so no protected path is touched and R8 is not mechanically required; because it is the X25519 primitive, still request `crypto-security-auditor` review before merge. A bump that cannot go green is closed with a reason on #452 rather than left open.

## 5. Final cars

- **F-1 Version bump to 0.5.1:** `Cargo.toml` workspace version, `android/build.gradle` versionName/versionCode (and any other site `docs_sync_check.sh` lists). Cut from main after all R/MT cars land; supersedes #471. Gate: `cargo test --workspace --no-run`, build-verify.
- **F-2 Docs sync:** update SHIP_PLAN.md status, CHANGELOG/release notes, `docs/DOCUMENT_STATUS_INDEX.md`; `docs-sync` skill must pass.
- **F-3 Tag v0.5.1:** performed by the Freebuff lane (D3) only after every car above is landed or explicitly closed, main is green on the exact head, and F-1/F-2 are merged. Annotated tag on the main head; no tag rewrite; record the tag SHA on #452 and close it. Items remaining under #567 are operator-owned and are listed in the release notes rather than blocking the tag unless the operator says otherwise.

## 6. Kernel queue relation

`scripts/orchestrate_strict.py` defaults `--queue` to repo-root `scm_v1_farm_queue.jsonl` (line 738), and `orchestration/manifest.yaml` names no queue or release target. That jsonl is the older farm-wave ticket queue (Z/A/... tickets; statuses done/open/blocked/frozen/retired) and does NOT hold these cars. Gap: until a follow-up adds these cars to a jsonl (or the manifest names this file), run the kernel with an explicit `--queue` pointing at a purpose-built file, and treat this document as the execution authority. No script or jsonl change is made by the commit that introduced this file.

## 7. Authority

Execution pointer: `SHIP_PLAN.md` (2026-10-14 entry). `HANDOFF/todo/_QUEUE.md` is backlog reference only. Update this file's table (status column, Last-updated) as cars land; live per-PR state is on #452.
