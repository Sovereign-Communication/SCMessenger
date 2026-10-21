<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
# Work-ahead and dependabot disposition (SCMessenger, 2026-09-25)

Owner of this document: SCMessenger. Read-only triage by the coordination seat.
Source of every check count below: `tmp/pr_disposition_ledger.md`, generated
2026-09-25T09:52:06Z by `python scripts/pr_disposition_ledger.py` (45 open
PRs, every check state parsed; a skipped check is counted as `skip`, never as
`pass`). The ledger, not this table, is authoritative; when they disagree,
re-run the ledger. PRs are identified by head SHA because several branch
names contain the word the handoff-scope gate treats as a foreign alias.

## Groups

### A. Merge-train PRs (single writer, sequenced)

| PR | Head | Checks (pass/skip/fail) | Gated | Disposition |
|---|---|---|---|---|
| #372 | `a76d7d66` | 33/0/0 | 5 | LAND FIRST after independent Rule-8 |
| #364 | `45b0f8b8` | 32/0/1 (Android JVM Unit Tests fail) | 4 | SPLIT: Kotlin half superseded by #372; core outbox sweep + Docker vendor copy as its own PR (Rule-8) |
| #367 | `63f761ab` | 31/0/0 | 0 | CLOSE or fold into the #364 split |
| #361 | `63f4a7d7` | 32/0/1 (Android JVM Unit Tests fail) | 4 | CLOSE after #372 |
| #351 | `db475b23` | 33/0/0 | 0 | REBASE after the #364 split (touches reserved `MeshForegroundService`) |
| #349 | `6a133520` | 33/0/0 | 0 | REBASE after #372 (`mobile_bridge.rs` overlap) + Rule-8 + keyed JEV |
| #352 | `6b5b08f3` | 33/0/0 | 0 | REBASE + keyed JEV (JEV row fails per its title) |
| #354 | `cb617149` | 19/0/0 | 0 | LAND before any WP DONE claim (holds the WP1/WP2 state files) |
| #355 | `dd146c5d` | 33/0/0 | 1 (`swarm.rs`) | REBASE + Rule-8 + keyed JEV |
| #356 | `4f87d52f` | 33/0/0 | 1 (`swarm.rs`) | REBASE + Rule-8 + keyed JEV |
| #357 | `089a7fc3` | 19/0/0 | 0 | REBASE docs; reconcile with #368 before closing |
| #359 | `3f41005d` | 6/0/0 (CONFLICTING) | 4 | REBASE docs residue only; drop the transport commit (#372 supersedes); add scope blocks to all 59 HANDOFF docs |
| #368 | `bac5d753` | 19/0/0 | 0 | MERGE after scope blocks; add WS queue rows in the same edit |
| #369 | `d56a6222` | 26/0/1 (Repository Hygiene fails) | 0 | CLOSE (snapshot of #368) |
| #360 | `e0dea501` | 6/0/0 (CONFLICTING) | 0 | SUPERSEDED by WS0 (0.3.3 floor -> pin file) |
| #316 | `7b6f15bf` | 19/0/0 | 0 | DOCS LAND (diagnosis only; the fix is #364's core half) |
| #329 | `e1b9263d` | 19/0/0 (CONFLICTING) | 0 | REBASE docs (queue status reconciliation) |
| #303 | `da3438ea` | 19/0/0 | 0 | HOLD (operator release hold) |

### B. Held work-ahead (operator release pending)

#298 (`8af8c607`) 33/0/0, #300 (`73730e2f`) 33/0/0, #301 (`64e66b73`)
33/0/0, #302 (`b107084b`) 33/0/0, #299 (`1291e918`) 32/0/1. All drafts, 0
gated files. They stay held until the operator releases them; re-base each on
`main` after the train, not before.

### C. Dependabot

| PR | Bump | Checks (pass/skip/fail) | Failing checks | Disposition |
|---|---|---|---|---|
| #211 | actions/setup-java 4 -> 5 | 27/0/0 | -- | after train |
| #212 | actions/stale 9 -> 11 | 27/0/0 | -- | after train |
| #214 | github/gh-aw 0.44.0 -> 0.87.3 | 27/0/0 | -- | after train |
| #103 | actions/cache 3 -> 6 | 21/1/2 (CONFLICTING) | FFI Surface Contract, iOS Build & Simulator Test (CodeQL skipped) | reopen against the current cache pin after train |
| #141 | actions/upload-artifact 4 -> 7 | 11/0/16 (CONFLICTING) | all four Android ABI jobs, Android Debug APK, Android JVM Unit Tests, Bindings (Kotlin/Swift), Docs, FFI Surface Contract, and more (stale branch, not rebased since 2026-09-17) | after train, rebase first |
| #106 | lifecycle-service 2.11.0 | 11/0/10 | Android Debug APK, FFI Surface Contract, Lint, Repository Hygiene, Rust Linting, Swift Linting, Test (macos/ubuntu) | 0.5.0 Android toolchain lane, after train, rebase first |
| #107 | mockk-android 1.14.11 | 10/1/13 | Android Debug APK, Bindings (Kotlin), FFI Surface Contract, Lint, Rust Linting, Swift Linting, Test (macos/ubuntu) (CodeQL skipped) | 0.5.0 Android toolchain lane, after train, rebase first |
| #108 | core-ktx 1.19.0 | 11/0/10 | same set as #106 | 0.5.0 Android toolchain lane, after train, rebase first |
| #210 | coroutines-test 1.11.0 | 24/1/1 | Android JVM Unit Tests (CodeQL skipped) | 0.5.0 Android toolchain lane, after train, rebase first |
| #213 | hilt-navigation-compose 1.4.0 | 23/1/2 | Android Debug APK, Android JVM Unit Tests (CodeQL skipped) | 0.5.0 Android toolchain lane, after train, rebase first |

The five Android bumps are NOT green, but for two different reasons, verified
live 2026-09-25 via `gh api repos/.../commits/<head>` (head commit date) and
`gh pr checks` (failing job names):

- #106, #107, #108: stale. Head commits are from 2026-07-12/2026-07-18
  (`3eee8707`, `d51154a8`, `b1e14a0d`); the PR `updatedAt` is 2026-08-13.
  Their heads sit on old main commits, so CI runs the current toolchain
  against July dependency code.
- #210, #213: current branches. Head commits are 2026-09-21 (`51f41d87`,
  `37d29f1a`), parent `9de879b9` (2026-09-21) is an ancestor of `origin/main`,
  and the failures are product-level, not staleness: #210 fails Android JVM
  Unit Tests; #213 fails Android JVM Unit Tests and Android Debug APK. A
  rebase will not fix either; they need the Android 0.5.0 toolchain lane.

The earlier "green" label for all five was wrong (skipped checks folded into
pass); the numbers above are from the generated ledger.

### D. Operator decisions (not this seat's call)

#170 (`d85eb6b9`, orchestration lanes, stale since 2026-08-16), #178
(`c1b457bb`, iOS, base is its own head ref, 1 gated file), #207 (`b02ec7d9`,
Apple continuity docs), #208 (`11f3e5e9`, Apple parity, CONFLICTING), #209
(`594bae18`, identity unification, PIN 066039; base for #216/#220), #216
(`b2020a20`) and #220 (`92b8ec12`) (drafts on #209), #218 (`c4108d99`, draft
self-referential circuit guard, gated), #227 (`2e9a9d96`, draft Android
degraded storage, CONFLICTING, 33/0/0), #228 (`ffa10281`, draft CI hardening,
14 gated files across crypto/privacy/routing/transport, 12/2/20 -- the
deepest-red PR in the set), #363 (`43b0d371`, draft external bridge,
foreign-material handoff), #156 (`a7d7cc49`, Docker suite non-blocking,
rebase after train).

## Notes

- The API file list is capped at 100 per PR; the ledger prints the count with
  that caveat and prints the full gated-file list. The train PRs are below the
  cap and the git-authoritative count equals the API count for each (#372 =
  71, #359 = 93, #364 = 76, #361 = 70; verified with `git diff --name-only
  origin/main...<head>` and `gh pr diff --name-only`). Re-verify the others
  the same way before landing.
- Every PR in group A except #372 is BEHIND main and needs a rebase; the
  rebase order follows the train order above.
- The ledger's contradiction check fails closed only on an OBSERVED
  contradiction (a `LAND FIRST` PR that is conflicting or a draft); when
  GitHub has not yet computed mergeability it prints `[WARNING]` instead, and
  a disposition recorded for a PR that is no longer open is also a warning.
