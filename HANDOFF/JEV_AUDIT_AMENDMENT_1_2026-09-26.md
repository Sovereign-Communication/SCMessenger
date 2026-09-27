# SCMessenger owner handoff — amendment 1: the Android fix scope is three PRs, the Android gate is not a merge blocker, and a silent Dependabot cargo outage

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This is an assist-only handoff. The owning repository retains all decisions, edits, merges, and publication authority.

It **amends**, and does not replace, `HANDOFF/JEV_AUDIT_ANDROID_GATE_2026-09-25.md`. Three things in that document are now known to be wrong, incomplete, or materially understated:

1. Its Finding 1 names a single carrier (#361) for the two broken `it.action ==` assertions. There are **three**, and all three are based on `main`.
2. It treats a failing `Android JVM Unit Tests` as an obstacle to merging. **That check is not required on `main`, so a red Android gate does not block anything.** The real risk is larger than a red gate, and it is stated in Amendment C.
3. It carries no finding at all for a cargo dependency-update outage that has been silently failing.

Everything else in the 2026-09-25 document still holds at the current head, and its root-cause analysis of the failing assertions is now confirmed by a green run rather than by inference — see Amendment A.

## Pinned state

- Measured `main` at `bceacb94` ("feat(handoff): add an auditable, per-document waiver register (#387)"), 2026-09-26. The two commits since `cd511fa0` are handoff/governance only and touch **no product code**, so every line number cited below is current.
- 49 pull requests were open at measurement time. All were enumerated; none were modified.
- All evidence below is read-only. No product worktree was touched, no branch of this repository was used for measurement, nothing was deployed.

---

## Amendment C — nothing stops a red Android gate from reaching `main`

This is the most important section in this document, and it changes how Amendment A should be read.

### The required-check list on `main`

`GET /repos/Sovereign-Communication/SCMessenger/branches/main/protection` returns exactly these required status checks:

```
["Repository Hygiene Checks", "Lint", "Rust Linting",
 "Test (ubuntu-latest)", "Handoff ownership scope"]
```

**`Android JVM Unit Tests` is not on that list.** There are no repository rulesets that might govern this instead.

### What that means for each carrier

- **#361** — 32 of 33 checks pass. The single failure is `Android JVM Unit Tests`. Four of the five required checks pass; the fifth, `Handoff ownership scope`, has never reported on this branch. State is `MERGEABLE/BEHIND`, i.e. one rebase from current. **On the checks this repository actually enforces, #361 is green.** The red Android result is real, visible, and not a blocker.
- **#364** — the same shape: every required check either passes or has never reported, the only failure is the non-required Android job, and the state is `CONFLICTING/DIRTY`.
- **#359** — none of the five required checks has ever reported. Its only workflow run in its entire history is the CodeQL run `35836517373`. The Android job has never run on it.

### The honest ordering, which is the opposite of the intuitive one

A red Android gate is the **best** of the three outcomes, because at least a human or a reviewer can see it. It is not a safety net — nothing enforces it — but it is a visible signal.

#359 is the **worst** outcome, and this is the point: its Android result does not exist. Nobody has ever looked at it, so the defect on that branch is invisible to CI, invisible to a reviewer scanning checks, and invisible to anyone reading this document without also reading the identity argument in Amendment A. A branch in that state, based on `main`, is the one most likely to reach `main` with the defect and with no red anywhere to explain what happened.

One honest qualification, so this is not overclaimed: a new push that touches `android/**` would most likely cause the job to run and go red, because the branch carries an `android/**` change. But that red would still not block the merge, and the run is a side effect of pushing rather than something the configuration demands. "Nobody has looked" is the accurate description; "it can never go red" would not be.

### This is a protection gap, not a fix

"Not required" describes a missing guard, not a remedy. The two ways this ends:

1. **The lane makes `Android JVM Unit Tests` a required status check**, and the defect can no longer reach `main` through any of the three carriers.
2. **The defect lands on `main`**, and from then on every branch cut from `main` carries it, including branches with nothing to do with #359, #361 or #364.

### Recommendation: require the check

**Recommendation: add `Android JVM Unit Tests` to `required_status_checks.contexts` on `main`.** Reasons, in order of weight:

- **It costs no build time the project does not already spend.** The job already exists in `.github/workflows/mobile.yml` as `android-unit-tests`, already runs on every pull request touching the paths it cares about, and already took `22m 55s` on #372. Requiring it spends minutes the project is already spending.
- **The repository's own rules already require it.** `docs/rules/ANDROID.md:59-64` lists under "Pre-Merge Checklist": `./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"` passes. The rule exists in the repository's own documentation; only the enforcement is missing.
- **The workflow was explicitly designed as a pre-merge gate.** The comment above the job at `.github/workflows/mobile.yml:60-68` says the standalone JVM job exists precisely to give test feedback on every pull request and to serve as the pre-merge gate that `docker-test-suite.yml` cannot. It was built to be a gate. It was never wired up as one.
- **It converts this handoff from advisory to enforced.** Required, it would have blocked #361 and #364 automatically and forced #359 to be gated.

**One prerequisite, and it matters.** `.github/workflows/mobile.yml` gates the job behind `paths:` filters (`android/**`, `iOS/**`, `core/**`, `Cargo.toml`, `Cargo.lock`, `scripts/build_xcframework.sh`, `scripts/check_wiring.py`, `scripts/test_check_wiring.py`, `core/src/api/udl`, `mobile/**`). GitHub treats a required check that never reports as *pending*, which blocks the merge. Adding the job to the required list **without** also making it always report — by giving the job an unconditional trigger, or adding a lightweight always-running companion job that reports the same result — would wedge every pull request that does not touch those paths, including the 46 open ones that are not Android work.

Sequence it deliberately: fix the three carriers first, or accept that #361 and #364 become correctly blocked the moment the check is required. That blockage is the intended outcome, not a regression.

---

## Amendment A — the broken assertions are carried by three pull requests, not one

### What the previous handoff said, and why it is incomplete

`HANDOFF/JEV_AUDIT_ANDROID_GATE_2026-09-25.md` reports the root cause of the red `Android JVM Unit Tests` gate and recommends cherry-picking `e7466639b59f6c07f19090ecd9867b39ac36b58e` onto **#361**. That advice is correct for #361 and insufficient overall. Because the other two carriers are also based on `main`, fixing only #361 leaves a pull request able to land the defect on `main`.

### Method, so the scope can be checked rather than trusted

All 49 open pull requests were enumerated. For each one, the single test file at issue was fetched at that pull request's exact head commit and hashed:

```
android/app/src/test/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModelTest.kt
```

via `GET /repos/Sovereign-Communication/SCMessenger/contents/<path>?ref=<head sha>`. Result: **3 carriers, 46 clean, 0 absent.** The search was over every open pull request, not over the two previously known, so this scope is complete as of measurement.

### The three carriers

The test file on all three heads is **byte-identical** — `sha256:0c4aff33456757e1…` on each — with 9 `@Test` methods and `it.action` at lines 102 and 118.

| PR | Head branch | Head commit | Base | State | `Android JVM Unit Tests` |
|---|---|---|---|---|---|
| #359 | `glm/canonical-outlier-audit` | `3f41005d1a6a2b9b608dd3960640a642c92f792e` | `main` | `CONFLICTING/DIRTY` | **never run** — see below |
| #361 | `d9-libp2p-degrade` | `63f4a7d73ef05ace437b54d11c14a256c71eb923` | `main` | `MERGEABLE/BEHIND` | **fail**, 20m 41s, run `35787291582` |
| #364 | `freebuff/and-stopflood-outbox-sweep-040` | `45b0f8b8a8019f94b9c39105e45a84315ef314b7` | `main` | `CONFLICTING/DIRTY` | **fail**, 19m 38s, run `35894514034` |

For contrast, the same file on `main` is `sha256:7943c4f2e115de32…`, 7 `@Test`, no `it.action` — so `main` is clean and the defect lives only on these branches.

### #372 is not a fourth carrier, and merging it does not fix the other three

This needs saying plainly, because #372 is the one branch in this document that already carries the corrected file, and the two facts are easy to run together in a reader's head.

- **#372 is not a carrier of the defect.** Its test file is `sha256:e7769b6b2b39…`, 7,469 bytes, 9 `@Test`, and **zero** occurrences of `it.action`. It is the corrected file, not a fourth instance of the broken one, and it does not widen the fix scope.
- **Merging #372 does not fix the three carriers.** #372 is a *different branch* from all of them. Its corrected file reaches `main` on #372's own terms or not at all; it does not travel to #359, #361 or #364 by #372 being merged. Each of the three needs the file corrected **on its own branch**, which is what action 3 below says.
- **#372 is not a clean route to `main` for this fix either.** It is `base=main` and it merges cleanly, but it is a four-part transport pull request — leak, WASM parity, admission cap, and the JVM tests. Obtaining the test fix by merging it means accepting the other three changes in the same merge, each of which needs its own decision here. The *file* can be lifted from it, as action 3 says; the pull request cannot be merged for that purpose alone.
- **And it is not blocked.** It merges cleanly against current `main`, as the table below shows. `BEHIND` means "needs an update", not "conflicting", and the two GitHub labels are not interchangeable.

### All three carriers are no longer mergeable into `main`, and that blocks the fix

`main` has moved since these branches were cut. Measured by a three-way merge against current `main` (`05df2960`), computed in memory — **no branch, ref or working tree of this repository was modified to produce it**:

| PR | Head | Merge-base with `main` | `main` commits behind | GitHub label | Actual conflicting files |
|---|---|---|---|---|---|
| #359 | `3f41005d` | `629a3eef` | **150** | `CONFLICTING` | **48 — of which 8 are product code** |
| #361 | `63f4a7d7` | `56d66f71` | 15 | `BEHIND` | **0 — merges cleanly** |
| #364 | `45b0f8b8` | `56d66f71` | 15 | `CONFLICTING` | **2 — both documentation** |

The GitHub labels overstate the obstacle for #361 and understate it for #359:

- **#361 is not conflicting at all.** It is behind and merges cleanly with no conflicting hunks. It needs an update to clear the `BEHIND` state, and nothing has to be resolved.
- **#364's two conflicts are `add/add` on documentation** — `HANDOFF/todo/AND_STOP_START_FLOOD_AND_CANCEL_2026-09-23.md` and `HANDOFF/todo/OUTBOX_NO_PERIODIC_RETRY_SWEEP_2026-09-23.md`. Two Markdown files, each created independently on both sides. Resolvable by keeping one side; no product judgement is involved.
- **#359 is the real one, and it is the branch nobody is looking at.** 48 conflicting files, 150 commits behind, and **8 of them product code with genuine content conflicts**: `ChatScreen.kt`, `ChatViewModel.kt`, `ContactsViewModel.kt`, `ConversationsViewModel.kt`, `DashboardViewModel.kt`, `PeerIdValidator.kt`, `OrderingAndNotificationRoutingTest.kt` (add/add) and `core/src/transport/swarm.rs`. The other 40 are Markdown. This is the same branch Amendment A already identifies as the least visible of the three, and it is now also by a wide margin the most expensive to land.

**One thing is *not* blocked, and it is the file this finding is about.** The defective test file `android/app/src/test/java/com/scmessenger/android/ui/viewmodels/MeshServiceViewModelTest.kt` is **not in any of the three conflict sets**. All three branches can take the corrected file without resolving a single conflict. The fix to Finding 1 and the merge debt on these branches are two separate problems; running them together would stall the cheap one behind the expensive one.

### Measured on two, unmeasured on one

- **#361** and **#364** have each had the Android gate run, and each failed with the identical signature: `353`/`357` tests completed, `2 failed, 3 skipped`, the same two test names (`toggle during STARTING|STOPPING`), and the same `Only one matching call to Context(#926)/Context(#965)` null-argument signature mismatch. That is measured, from CI logs.
- **#359 has never had the Android gate run.** Its head commit has exactly one workflow run in its history — the CodeQL "PR #359" run `35836517373`, success, 6 check-runs, no Android job. So #359 is a carrier **by file identity, not by a measured failure**. Because its test file is byte-identical to the two that do fail, it is expected to fail the same way; that expectation is inference and is labelled as such.

Read this together with Amendment C: because `Android JVM Unit Tests` is not a required check, the two measured reds do not block those merges, and the one unmeasured carrier is the one nobody is looking at.

### The two assertions, verbatim

From the shared carrier file (identical on all three heads):

- line 101-103 — `mockContext.startService(match {` / `it.action == MeshForegroundService.ACTION_STOP` / `})`
- line 117-119 — `mockContext.startForegroundService(match {` / `it.action == MeshForegroundService.ACTION_START` / `})`

The assertions sit in the two tests named `toggle during STARTING …` and `toggle during STOPPING …`.

### The corrected assertions are verified green, and what they now assert

The earlier caveat on #372 is discharged. `Android JVM Unit Tests`, job `108481836751`, run `36269937310`, head `2cb046cf`: **conclusion `success`**, 22m 55s, `BUILD SUCCESSFUL in 22m 33s`, 40 actionable tasks executed. Tallied from the job log:

| | #361 (red) | #372 (green) |
|---|---|---|
| STARTED | 353 | **356** |
| PASSED | 348 | **353** |
| FAILED | 2 | **0** |
| SKIPPED | 3 | **3** |

Both formerly-failing tests are present and explicitly `PASSED`. Two things in that log are noise, not failures: the only `FAILED` substrings are `TIMBER … Receipt encode FAILED … simulated transient encode error` from a passing retry-guard test, and the `NetworkRequest$Builder` `NullPointerException` lines are `STANDARD_OUT` from tests exercising android.jar stubs. The 356-vs-353 difference is #372's other work — both copies of this test file have 9 `@Test`, so the file swap does not change the count.

**What the replacement asserts, and the one thing it stops asserting.** The full diff is four hunks: one import added, one dropped, and the two assertion bodies. No test was deleted (`@Test` 9 → 9) and no assertion was removed; `verify` calls go from 2 to 4.

```kotlin
// #361, unpassable
verify(exactly = 1) { mockContext.startService(match { it.action == MeshForegroundService.ACTION_STOP }) }

// #372
verify(exactly = 1) { mockContext.startService(any<Intent>()) }
verify(exactly = 0) { mockContext.startForegroundService(any<Intent>()) }   // new
```

It keeps the discriminating assertion — `Context.startService` versus `Context.startForegroundService` is the observable contract that separates the stop path from the start path in production (`MeshServiceViewModel.kt:87-91` and `:106-109`) — and it **adds** an `exactly = 0` negative assertion the original did not have. On that axis the corrected version is more constraining than the one it replaces.

What it genuinely gives up is the `Intent.action` **constant**. A regression that called `startService` with the wrong action string would now pass. That reduction is unavoidable in this environment rather than a convenience: `returnDefaultValues = true` (`android/app/build.gradle:232-234`) makes android.jar's `Intent.action` a stub returning `null`, and Robolectric was removed (`:315-319`), so the original could not pass by construction — which is exactly why it was red on three carriers. The in-code comment documents this and cites `android/app/build.gradle:315`. Restoring action-level coverage needs an instrumented test or Robolectric reinstated for this class, which is separate work and is not part of this fix.

So: the lane inherits a genuine, stated narrowing alongside a verified-green gate. Both facts belong in the same breath.

### The corrected assertions already exist in this repository

PR **#372** (`fix/361-review-blockers`, head `2cb046cf8cab34c15327a4fbfbb90fb91ab212f4`) carries `e7466639` as an ancestor. Its test file is `sha256:e7769b6b2b39dd4a…` — 9 `@Test` methods and **zero** `it.action`. So the fix is already written and already reviewed in this repository; it is simply not on the two branches that need it, and not on `main`.

Two caveats stated plainly: #372 is `MERGEABLE/BLOCKED` and is not a route to `main` on its own, and the file it carries is the one whose green run is recorded above — the content hash and the passing job now agree.

### Root cause, restated against the current code (line numbers verified at `bceacb94`)

Unchanged from the 2026-09-25 handoff, and re-verified at this head:

- `android/app/build.gradle:232-234` — `unitTests { returnDefaultValues = true }`.
- `android/app/build.gradle:307-308` — the comment documenting that `android.jar` stubs no-op under `returnDefaultValues`, which is what makes a stubbed `Context.startService(intent)` drop the `intent`.
- `android/app/build.gradle:315-319` — records that Robolectric was removed on 2026-07-27; Robolectric is nowhere a dependency, so nothing restores real `Intent` objects.
- The **production** code is correct and is not the defect: `MeshServiceViewModel.kt:87-91` uses `startForegroundService` with `ACTION_START`, and `:106-109` uses `startService` with `ACTION_STOP`.

`returnDefaultValues = true` turns the mocked `Context` into a no-op, so `it` in the matcher is `null` and the assertion fails. Robolectric, or a test that asserts on the `Intent` that production actually builds, are the two ways out; the corrected version on #372 is the second.

### Requested owner actions

1. **Clear the merge state before landing the fix, and before re-running the gate.** Two carriers conflict with `main` and the third is behind; the evidence is in *All three carriers are no longer mergeable into `main`*. It is cheap on #361 (an update, nothing to resolve) and on #364 (two `add/add` documentation files), and it is the substantial one on #359 (48 files, 8 of them product code). This is a decision for this repository and is **deliberately left unresolved here** — these are your branches, and how to reconcile them is your call, not an operations finding.
2. Treat the fix scope as **#359, #361 and #364** — not #361 alone. All three are `base=main`.
3. Apply the corrected test file (available on #372, or as commit `e7466639b59f6c07f19090ecd9867b39ac36b58e`, which applies cleanly onto the #361 head) to **each** of the three branches. This is **independent of action 1**: the test file is in none of the three conflict sets, so the fix can be applied and reviewed while the merge debt is being worked through.
4. Re-run `Android JVM Unit Tests` on each of the three after the change. For #359 this is the first run that branch will ever have had. A rebase is the point at which a previously-passing or previously-failing result should be re-read rather than carried over.
5. Do not merge any of the three with the file unchanged. Per Amendment C this is a human decision, not an enforced one: on #361 and #364 every check this repository actually requires is green or has never reported, so nothing will stop the merge for you. Merging any of the three with the file unchanged puts the defect onto `main`, and from `main` it propagates to every future branch.

---

## Amendment B — cargo dependency updates have been silently failing

This is new, and it is not a footnote: it is an outage of the automated-update path, and the outage is currently invisible to CI and to releases.

### Symptom

Workflow **Dependabot Updates**, run `36264033807`, `main` at `9e668682`, 2026-09-26 18:51Z, conclusion `failure`. This is the first failure since 2026-09-21.

Every cargo dependency fails with the same error, e.g. for `clap`:

```
Handled error whilst updating clap: dependency_file_not_resolvable {message:
"error: failed to load manifest for workspace member `dependabot_tmp_dir/wasm`
referenced by workspace at `dependabot_tmp_dir/Cargo.toml`

Caused by:
  failed to read `dependabot_tmp_dir/wasm/Cargo.toml`

Caused by:
  No such file or directory (os error 2)"}
```

The same error is recorded 82 times in that run — for `mockall`, `curve25519-dalek`, `camino`, `clap`, `windows`, `tokio-tungstenite`, `btleplug`, `tower`, `crc32fast`, `uniffi`, `chacha20poly1305`, `async-trait`, `argon2`, `axum`, `blake3`, `lz4_flex` and the rest — i.e. the entire cargo dependency set. Across the whole run there is exactly one error shape (`dependency_file_not_resolvable`, 82 occurrences) and no reference to the `github-actions` or `gradle` entries, so those two entries are not implicated.

`.github/dependabot.yml` configures cargo with `directory: "/"` and no `exclude-paths`; the job definition confirms `"package-manager":"cargo"`, `"directory":"/."`, `"exclude-paths":null`.

### Root cause — reproduced, not inferred

Root `Cargo.toml` lists `wasm` **twice**:

- line 2 — `members = ["core", "cli", "wasm", "mobile", "desktop_bridge"]`
- line 5 — `exclude = ["wasm"]`

On the cargo version the Dependabot log reports (1.98.1), `exclude` does **not** remove a path that is also named in `members`. Proof: with the tree exactly as on `main`, `cargo metadata --no-deps` reports `scmessenger-wasm` among the workspace packages. Cargo therefore still requires `wasm/Cargo.toml` to exist, and Dependabot's working directory did not contain it.

Local reproduction, byte-for-byte the same failure:

| Tree | `cargo metadata --no-deps` |
|---|---|
| `main` as-is, all five member directories present | exit 0 |
| `main` with `wasm/` removed (mimics the Dependabot directory) | **exit 101** — `failed to load manifest for workspace member …/wasm` / `failed to read …/wasm/Cargo.toml` |

### The fix is two changes, and the first one alone breaks CI

**This section was previously headed "The fix: one line", and that heading was ours and it was wrong.** Both changes were stated in the body of this document from the first version, but a heading saying "one line" is what a reader acts on — and #391 acted on it correctly. It took the one line, applied the wrong half of it, and shipped. The misleading heading is the defect here, not the reader of it.

Both lists were edited in a scratch copy and the result measured:

- **Removing `"wasm"` from `members` (line 2)** → `cargo metadata --no-deps` at the workspace root **resolves, exit 0** — and then the `wasm32` CI jobs fail, because `wasm/Cargo.toml` is written to inherit from the workspace. This is change 1 of 2, not a fix.
- **Deleting the `exclude` line (line 5)** instead → **still exit 101**, with the identical `failed to read …/wasm/Cargo.toml` error. Change 1 and nothing else.

`exclude` is inert for this failure and removing it fixes nothing. The fix is two changes, landed together:

1. remove `"wasm"` from `members` (line 2), **and**
2. make `wasm/Cargo.toml` standalone in the same change.

They are not a change and a follow-on. Landed alone, change 1 trades a silent Dependabot outage for three red `wasm32` jobs: louder, and worse.

### Required follow-on: one removal is necessary but not sufficient

Removing `wasm` from `members` makes it a non-member, and `wasm/Cargo.toml` is written to inherit from the workspace: `version.workspace`, `edition.workspace`, `license.workspace`, and 13 dependencies declared as `{ workspace = true }`. It then fails to parse on its own:

```
error: failed to parse manifest at …/wasm/Cargo.toml
Caused by:
  error inheriting `edition` from workspace root manifest's `workspace.package.edition`
```

Adding an empty `[workspace]` table to `wasm/Cargo.toml` and spelling out the three package fields advances it to the next failure, `error inheriting 'anyhow' from workspace root manifest's 'workspace.dependencies.anyhow'`, and 13 `workspace = true` dependencies remain to be spelled out.

That matters because CI builds this package: `cargo build --target wasm32-unknown-unknown -p scmessenger-wasm --release` at `.github/workflows/cross.yml:143`, plus the `wasm32-unknown-unknown` steps at `.github/workflows/cross-platform-test.yml:179` and `.github/workflows/release.yml:357`. A change to `members` that is not accompanied by making `wasm/Cargo.toml` standalone will fix Dependabot and break those.

### #391 is right about the comment, and its own text rules out its fix direction

PR #391, *"fix(cargo): wasm in members AND exclude is what silences the cargo Dependabot updater"* (opened 2026-09-27), keeps `wasm` in `members` and deletes `exclude`. **That is the edit measured above as still exit 101.**

Two things are credited before anything else, because both are correct and both improve the manifest:

- **The stale comment was wrong.** `Cargo.toml:3-4` described `wasm` as "excluded from default members". `exclude` has no effect on default members — that is `default-members` — so it described a mechanism that was never active, and it is part of what made the misconfiguration look deliberate. #391's replacement comment is accurate and the line should go however this is resolved.
- **`members` is not `default-members`,** and #391 is right to say so.

The fix direction is nevertheless the one that does not work, and #391 establishes that itself. Its diff comment reads:

> a members-only entry whose directory is absent makes every cargo command fail to load the workspace, which is what stopped Dependabot's cargo updater

That sentence is correct, and it is the entire problem. Dependabot materialises the workspace **without** `wasm/`, so a `members`-only entry naming `wasm` has an absent directory there and fails by precisely the quoted mechanism. #391's own failing measurement is that same case — manifest untouched, `wasm/` renamed away, RC 101 — and in that measurement **`exclude` was still present**. `exclude` was therefore not the variable in the failing run, so its absence cannot be what changes the outcome.

This has been raised on #391 with the measurement, rather than left for a reader to infer from this document.

### What is not claimed

*Why* Dependabot omits `wasm/Cargo.toml` from its working directory is internal to Dependabot and cannot be verified from this repository. The double-listing is the strongest candidate trigger — a fetcher honouring `exclude` when choosing which member manifests to materialise — but that mechanism is inference and is not asserted here. The reproduction above does not depend on it: it establishes that any consumer which materialises the workspace without `wasm/Cargo.toml` fails, and that removing `wasm` from `members` is the change that makes the root workspace resolve without it.

### Requested owner actions

1. Remove `"wasm"` from `members` in root `Cargo.toml` line 2. Confirm locally with `cargo metadata --no-deps` (expect exit 0).
2. Make `wasm/Cargo.toml` standalone in the same change: an empty `[workspace]` table, literal `version`/`edition`/`license`, and the 13 `{ workspace = true }` dependencies spelled out.
3. Confirm `Cross` and `release` still build `scmessenger-wasm` for `wasm32-unknown-unknown`.
4. Re-run the Dependabot job and confirm the cargo entry succeeds.
5. Separately and cheaply: `Cargo.toml:3-4` comments that `wasm` is "excluded from default members". `exclude` has no effect on default members at all — that is `default-members` — so the comment is wrong on its own terms and is part of what made this misconfiguration look intentional.

---

## Addendum — re-verified 2026-09-27, no finding changed

**Why this addendum exists, stated rather than assumed.** A handoff that was accurate when written and has since become quietly incomplete is worse than one that is merely wider, because a reader has no way to know which parts to discount. A wrong number announces itself; a stale one reads exactly like a correct one. So the facts that change what a reader should *do* are recorded here rather than left to rot in the body. Every item below alters the action this repository takes, not merely the description of it.

**This is a dated snapshot, not a standing claim, and it is labelled as one.** Every finding in this document is a **property of a revision** — `main` at a named commit, the carriers at their named heads — and it holds only for that revision. A later reader should re-check rather than trust, and should treat the date above as the moment the check was last true, not as a claim that it still is. The cheapest re-check is to re-run the commands named in *Pinned state* against current `main`; nothing in this document should be read as a standing property of the code.

Everything above was re-checked against live state. **No finding, measurement or recommendation changed.** What changed is the context around them:

- `main` has advanced from `bceacb94` to `05df2960` (merges #389, #390 on top of the two already noted). All five commits since `cd511fa0` touch handoff and governance files only — `ci.yml`, `scripts/validate_handoff_scope.py`, `handoff_scope_waivers.json` and four `HANDOFF/*.md` — and **none touches `android/`, `Cargo.toml` or any product code**, so every line number cited above is still current.
- The required-check list on `main` is still exactly `Repository Hygiene Checks`, `Lint`, `Rust Linting`, `Test (ubuntu-latest)`, `Handoff ownership scope`, `strict: true`, and the repository still has **no rulesets**. `Android JVM Unit Tests` is still not required.
- The three carriers are still the three carriers. The defective test file is byte-identical at all three heads (`sha256:0c4aff33…`, 6,655 bytes, 9 `@Test`, `it.action` at lines 102 and 118), and a re-enumeration of all **51** open pull requests found no fourth copy.
- **Two of the three carriers now conflict with `main`.** #359 and #364 are `CONFLICTING`; #361 is `BEHIND`. Each needs a rebase before the corrected assertions can land, and a rebase is the point at which the Android result should be re-read rather than carried over.
- **The corrected assertions already exist in a mergeable pull request.** #372 (`fix/361-review-blockers`, head `2cb046cf`, `base=main`) carries the fixed test file — `sha256:e7769b6b…`, 7,469 bytes — and is fully green, including `Android JVM Unit Tests` passing in 22m 55s on run `36269937310`. This is the run already cited above as proof the corrected assertions pass. The fix is therefore available to lift from #372 rather than to be rewritten. **It is not a fourth carrier, and merging it does not fix the other three** — see the note beside the carrier table, which also corrects an earlier draft of this addendum that described #372 as blocked. It is merge-clean; it is `BEHIND`, which is a different thing.
- **All three carriers are now unmergeable into `main` without work**, which sits between this repository and closing Finding 1. Two conflict and one is behind, with the real cost on #359. The evidence, the per-branch breakdown, and the fact that the defective test file is in none of the conflict sets are in *All three carriers are no longer mergeable into `main`*; it is now requested action 1.

## Severity and limits of this handoff

- **Severity of Amendment A**: high, and higher than a red gate alone would suggest. `Android JVM Unit Tests` is not a required check, so a failing Android suite does not block a merge. Two `base=main` carriers are red while fully green on every enforced check; a third has never been gated at all and is the least visible of the three.
- **Severity of Amendment C**: the highest of the three, because it is the one that recurs. Fixing the three carriers by hand closes three branches; requiring the check closes the class.
- **Severity of Amendment B**: medium-high. It does not block CI or releases; it means Rust dependency updates — including security updates — have not run for five days, and the failure is silent by construction.
- **Limit**: the Android gate cannot be executed locally; all Android findings rest on CI logs and on file content read at the exact head commits named above.
- **Limit**: no statement here should be read as a merge, an approval, or a sign-off. The owning repository decides what lands.
