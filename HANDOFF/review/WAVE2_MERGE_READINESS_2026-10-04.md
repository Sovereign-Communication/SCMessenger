# Wave-2 merge readiness: full board triage (2026-10-04)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Owned by `Sovereign-Communication/SCMessenger` (this repository).

Base for every prediction below: `origin/main` = `051dbb5b9cd900669f11ecf5c40c63a38c2e86a6`.
Method: enumerated from git and the GitHub API, never from the PR board by eye.
The board is 69 open PRs; a PR's merge state on GitHub lags and is not evidence.

## Board totals (enumerated, not eyeballed)

| bucket | count |
|---|---|
| open PRs | 69 |
| drafts | 11 |
| from forks | 3 |
| Rule-8 CLEAR (0 files under the gated paths, non-draft, non-fork) | 45 |
| Rule-8 BLOCKED (>=1 gated file) | 13 |
| of the 45 clear: conflict-free vs main | 33 |
| of the 45 clear: conflicting vs main | 12 |

`mergeStateStatus` on the board: 47 BEHIND, 17 DIRTY, 3 UNSTABLE, 1 BLOCKED, 1 CLEAN.
`mergeable`: 52 MERGEABLE, 17 CONFLICTING.

Tripwire: PR #451 reports exactly 100 files, which is the API pagination cap, so its
18 gated files are a FLOOR not a total. The same cap applies to #209 (76 files, under
the cap, so exact) and none other.

## Read this first: 10 PRs are known-poison

These are dependabot bumps of coordinates **already proven to break this build**,
because they were reverted out of the #451 train after they turned CI red. Merging
any of them will break `main` again. Do not queue them without a toolchain upgrade.

| PR | coordinate | proven failure |
|---|---|---|
| #439 | `x25519-dalek` 2.0 -> 3.0 | every Rust job, E0599 `Utf8Bytes::into_bytes` |
| #441 | `uniffi` 0.31 -> 0.32 | every Rust job, E0599 |
| #443 | `tokio-tungstenite` 0.24 -> 0.29 | every Rust job, E0599 |
| #106 | `androidx.lifecycle:*` 2.6.2 -> 2.11.0 | needs compileSdk 37 + AGP 9.1; project is 35 / 8.13.2 |
| #108 | `androidx.core:core-ktx` 1.12.0 -> 1.19.0 | same |
| #213 | `hilt-navigation-compose` 1.1.0 -> 1.4.0 | same |
| #107 | `io.mockk:mockk-android` 1.13.10 -> 1.14.11 | Kotlin 2.2 metadata vs pinned `kotlin_version = 1.9.20` |
| #210 | `kotlinx-coroutines-test` 1.7.3 -> 1.11.0 | same; KSP rejects 6 `.kotlin_module` descriptors |

`#440` (hyper-util 0.1.21) and `#442` (blake3 1.8.7) touch only `Cargo.lock` at
patch/minor level and were NOT part of the proven-bad set. They are unproven, not
proven-good: treat as a normal PR and let CI decide.

A first pass at this scan produced 14 false positives (#349, #352, #357, #376, #388,
#410 matched because `blake3::hash(...)` appears in source and crate names appear in
prose). The corrected scan only counts version changes inside real dependency
manifests. If you re-run it, keep that restriction or you will get the same noise.

## Wave 2 candidates: Rule-8 clear AND conflict-free (23)

Ordered by risk ascending. None touches `core/src/{crypto,transport,routing,privacy}`.

### Tier A - docs and plans only, no code (7)
Zero build risk. Merge first to drain the queue.

- #316 docs(inbox): outbox retry delay diagnosed (1 file)
- #376 docs: HTTP-compliant transport plan (1 file)
- #386 docs(handoff): JEV audit + #361 Android JVM gate root cause (1 file)
- #388 docs(handoff): amend the Jev audit, Android fix scope (1 file)
- #415 docs(bod): record three Board runs (1 file)
- #425 docs: unify 0.4.0/0.5.0 handoff, record Pixel evidence (2 files)
- #357 docs(freebuff): command-backed train state WP1-WP4 (5 files)

### Tier B - small isolated code fixes (9)
Each is 1 file and does not collide with the Android cluster.

- #380 fix(cli): grant consent in cmd_test (1 file)
- #381 fix(cli): reuse live IronCore in send queue fallback (1 file)
- #382 fix(api): report `success:false` when /api/send only queues (1 file)
- #401 fix(outbox): drain all peer-identity spellings in one pass (1 file)
- #400 fix(crypto): verify contact bundle signature before persist (1 file)
  NOTE: title says crypto but the diff has 0 files under the gated paths; confirm
  the actual file before merging.
- #349 feat(mobile_bridge): feed routing_peer_seen from all data sources (1 file)
- #352 test(core): WP1 identity-unification regression battery (2 files)
- #405 security: allowlist the two proven-false keystore findings (1 file)
- #156 ci: mark Docker integration suite non-blocking for v0.4.0 (1 file)

### Tier C - umbrellas and workflow (4)
Bigger blast radius; merge after A and B are green.

- #398 Rework POST /api/send to option-B contract semantics (3 files)
- #170 feat(orchestration): operationalize free API lanes (4 files)
- #409 MT-02: four of seven fresh fixes, bundles #401 #400 #381 #380 (9 files)
  OVERLAP WARNING: #409 contains #401, #400, #381, #380. Merging the umbrella and
  the parts separately duplicates work. Pick one: prefer the umbrella, or close the
  parts, not both.
- #410 MT-03: P0 umbrella, three of five work packages (17 files)

### Tier D - CI action bumps (2)
- #212 actions/stale 9 -> 11
- #214 github/gh-aw 0.44.0 -> 0.87.3 (large minor jump; review the changelog)

### Hold, docs-only but conflicting (3)
These are Rule-8 clear but conflict with main on shared doc files. They need a
rebase, not a re-review.

- #412 conflicts on `docs/rules/BUILD_AND_CI.md`
- #416 conflicts on `docs/rules/BUILD_AND_CI.md` (same file as #412 - serialize these)
- #406 conflicts on `HANDOFF/CTO_STATE.md`

## Wave 2 exclusions and why

### Rule-8 blocked, needs a non-author review (13)

| PR | gated files | note |
|---|---|---|
| #451 | 18 (floor) | the integration train; see RULE8_TRAIN_451_REVIEW_GAP_2026-10-04.md |
| #228 | 14 | draft; the perimeter underscore-param enforcement |
| #372 | 5 | CONN-CAP review blockers |
| #359 | 4 | two-tier per-peer connection policy |
| #361 | 4 | D9 vendored libp2p-swarm |
| #364 | 4 | mesh stop/start serialization |
| #421 | 2 | per-peer cap 4 -> 16 (same change as #451's `behaviour.rs`) |
| #178 | 1 | FORK, needs xcodebuild on the operator's Mac |
| #218 | 1 | draft; self-referential circuits |
| #355 | 1 | the auto-subscribe decision (WP3) |
| #356 | 1 | watchdog decision test pins |
| #399 | 1 | ratchet session drop - this is the F4 finding under review |
| #413 | 1 | MT-07 outbox re-flush; Android cluster |

Note #355, #399, #421 and #413 are all already INSIDE the #451 train, so merging
them separately would duplicate work. They are listed because they are open PRs,
not because they should be merged on their own.

### Fork-hosted (3)
#178 (gated), #207, #208. #208's branch `gpt/v050-parity-burndown` does not exist in
this clone (`git fetch` -> `couldn't find remote ref`) and needs `xcodebuild`, which
only exists on the operator's Mac. Not actionable from here.

### Android cluster: 4 PRs collide on ONE file
#351, #367, #411 (and #364) all conflict on
`android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt`.
Standing constraint: do not split that file. These must be resolved as one unit by
a human, not merged one at a time.

## Recommended order of operations

1. Merge Tier A (7 docs PRs). Lowest risk, drains the board.
2. Merge Tier B (9 small fixes), CI-verified one at a time or as a second wave.
3. Decide the #409 umbrella-vs-parts overlap before touching either.
4. Tier C and D after A and B are green on main.
5. Leave the 10 poison PRs open and untouched until the toolchain moves to
   compileSdk 37 / AGP 9.1 and Rust deps are re-validated.
6. Rule-8 PRs wait for a reviewer who authored none of the listed commits.

## Standing constraints that apply to every wave

- Required contexts on `main` are exactly `Repository Hygiene Checks`, `Lint`,
  `Rust Linting`, `Test (ubuntu-latest)`, `Handoff ownership scope` (strict).
  `Mobile` is NOT required and `main` runs no `Mobile` job, so Android regressions
  cannot block a merge. Verify Android by hand on the PR.
- Never push to a branch with in-flight CI; that cancels the runs.
- No local builds. CI is the verifier. `python scripts/disk_budget.py` currently
  reports below the tight threshold.
- Enumerate before acting. `bash scripts/pr_scope.sh <n>` before every merge.