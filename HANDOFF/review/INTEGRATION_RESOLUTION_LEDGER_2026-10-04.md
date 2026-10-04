<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# Integration resolution ledger — `integrate/train-20261004`

Every resolution below was made while consolidating SCMessenger's merge train.
Repository: `Sovereign-Communication/SCMessenger`.

Branch: `integrate/train-20261004` @ `242dc534`
Base: `main` @ `051dbb5b`
PR: #451 — 55 PRs, 188 commits, 129 files, +13,908/-648

Every conflict resolved while consolidating the merge train, with the evidence that
decided it. Written because a resolution with no recorded reason is indistinguishable
from a resolution made by accident, and this branch made eleven of them.

## Superseded PRs — closed or annotated, never merged twice

Verified with `git cherry`, which prints commits in `$2` not already in `$1`. Empty
output means every commit of the left ref is already an ancestor of the right ref.

| Contained | Superseded by | Evidence |
|---|---|---|
| #408 | #416 | `git cherry -v refs/tmp/pr416 refs/tmp/pr408` → **empty** |
| #412 | #408 + 1 file | `git diff --stat refs/tmp/pr408 refs/tmp/pr412` → 1 file, +209, new |
| #409 | #413 | single `-` line (patch-equivalent) |
| #209 → #216 → #220 | #220 | `git cherry` empty at each hop |
| #351, #367 → #411 → #413 | #413 | `git cherry` empty at each hop |
| #361 | #364 | `git cherry` empty |

#408 was closed with this proof in the closing comment. #209, #216, #351, #361, #367
and #411 were annotated with the same proof but deliberately **left open**: their
supersets are themselves held back for want of a Rule-8 review, so closing now would
drop the content rather than deduplicate it.

## Resolutions, and what decided each one

### 1. #108 — `android/app/build.gradle` — newest of each coordinate

| coordinate | HEAD | #108 | taken |
|---|---|---|---|
| `androidx.core:core-ktx` | 1.12.0 | **1.19.0** | 1.19.0 |
| `androidx.lifecycle:*` | **2.11.0** | 2.6.2 | 2.11.0 |

Taking #108's hunk wholesale would have regressed four lifecycle coordinates that an
earlier integrated PR had already moved to 2.11.0. Taking HEAD's would have dropped the
core-ktx bump that is the PR's entire content.

### 2. #210 — `android/app/build.gradle` — newest of each coordinate

| coordinate | HEAD | #210 | taken |
|---|---|---|---|
| `kotlinx-coroutines-test` | 1.7.3 | **1.11.0** | 1.11.0 |
| `io.mockk:*` | **1.14.11** | 1.13.10 | 1.14.11 |

Same shape as #108: neither side is acceptable whole.

### 3. #398 — `cli/src/api.rs` — option B, decided by the compiler, not by preference

The two sides asserted incompatible meanings of `success`: *the transport confirmed
delivery* (HEAD) versus *the server durably accepted responsibility* (#398).

Three independent facts made this decidable rather than a judgement call:

1. HEAD's `SendMessageResponse` has **no `warning` field** — verified by reading
   `git show HEAD:cli/src/api.rs`. HEAD's hunk assigns `error`, so taking it means
   discarding the field #398 adds, and the `warning:` initialisers already present in
   the surrounding merged code would not compile.
2. The non-conflicting lines immediately above and below the hunk were merged from
   #398's own edits and are already option B ("success means the server durably
   accepted responsibility"). Taking HEAD here contradicts its own neighbours.
3. Issue #392, "Rework POST /api/send to option-B contract semantics", is the open task
   this implements; #379 is the defect it fixes.

### 4. #103 — `.github/workflows/lint.yml` — bump taken, guards kept

#103's real content is `actions/cache@v3` → `@v6`. Its branch is 996 commits behind
`main` and predates the `is_docs_only` gating, so its hunks *omit* the guard lines
rather than deleting them. Resolution: take both `uses:` bumps, retain all 15
`is_docs_only != 'true'` guards. Verified after resolving: 15 guards present, 2 bumps
applied.

### 5. #141 — `ci.yml`, `cross.yml`, `mobile.yml` — same shape as #103

Pure action-version bump (`actions/upload-artifact@v4` → `@v7`) on branches that
predate the platform-relevance gating. Same resolution: take the bumps, keep every
`if: steps.platform.outputs.*_relevant == 'true'` guard. Verified after resolving: 22
guards in `ci.yml`, 26 in `cross.yml`, 14 in `mobile.yml`.

### 6. #360 — `scripts/harness_gate.py` — combined, neither side replaced the other

| | provides |
|---|---|
| HEAD | `HARNESS_REPO` override, `vendor/sovereign-harness`, in-repo `Harness/` fallback |
| #360 | `SCM_HARNESS_ROOT` override, operator Windows checkout, the 0.3.3 floor check |

Resolution keeps both override spellings and the whole portable chain, with the Windows
path demoted to a last resort that is only accepted if it actually carries
`harness/jev.py`. Both sides were improvements over a hardcoded default and neither
replaced the other.

### 7. #416 — `docs/rules/BUILD_AND_CI.md` — both sections kept whole

`main` added `## Rust toolchain pin` (commit `87932c08`, #430); #416 adds
`## CI-Primary Build Doctrine`. Both are additive `##` sections inserted at the same
anchor and neither supersedes the other. Both retained in full.

### 8. #416 — `HANDOFF/freebuff/queue/V040_V050_MERGE_TRAIN_UNIFY_2026-09-27.md` — #416's side

All three hunks strictly newer: status `TODO` → `IN PROGRESS`; pre-merge JEV gate →
post-merge closing gate; and #416 adds the `14.6` operator rulings record that HEAD
lacks entirely.

### 9. #369 — four documentation files — #369's side, after verifying its claims

#369 asserts the harness admission flow "is implemented". Verified before accepting:
`scripts/harness_admission.py`, `scripts/harness_source.py`,
`scripts/update_local_harness.py`, `scripts/local_harness.py`,
`scripts/harness_admission.json` and `scripts/test_harness_admission.py` all exist in
the integrated tree. Its newer wording is therefore the accurate one.

### 10. #369 — `scripts/harness_gate.py` — HEAD kept, refactor deferred

#369 replaces ad-hoc harness resolution with a `HarnessSource` abstraction and
**deletes** #360's `_harness_version` / `_version_tuple` / `_check_version` 0.3.3 floor
check. Taking it would silently remove a fail-closed gate that #360 introduced on
purpose. That is an architecture fork, not a textual merge, so the documentation landed
and the refactor did not. **Open question for the operator:** does `HarnessSource`
replace the floor check, or must the floor check be reimplemented on top of it?

### 11. #406 — `HANDOFF/CTO_STATE.md` — #406's side

HEAD's side of the hunk was empty; #406 adds one line.

## A latent gate violation this surfaced

`.github/workflows/security-regression-tests.yml:83` carried
`uses: dtolnay/rust-toolchain@stable`, introduced by `730ae59ec` (#228, a draft).
`scripts/check_toolchain_pin.py` fails closed on it — 1 of 28 declarations disagreed
with `rust-toolchain.toml`. Pinned to `1.99.0`.

This is precisely the drift `docs/rules/BUILD_AND_CI.md` documents as unacceptable: an
unpinned `stable` moves under the repository's feet and produces a red build on a diff
that never touched it. Left alone it would have done exactly that, later, to someone
else's PR.

## Audit of this branch before merge

| check | result |
|---|---|
| `<<<<<<<` or `>>>>>>>` anywhere in the tree | **0** (`git grep -nE '^(<{7}\|>{7})'`) |
| `=======` lines that are markers rather than markdown | **0** — no file contains both `<<<<<<<` and `=======` |
| build artifacts / `target/` / binaries introduced | **0** (`git diff --name-only origin/main..HEAD` against artifact patterns) |
| tracked files under `target/` or with binary extensions | **0** |
| duplicate coordinates in `android/app/build.gradle` | 1 hit, `junit:junit:4.13.2` — **pre-existing on main**, and legitimate: `testImplementation` and `androidTestImplementation` are different scopes |
| duplicate Cargo dependency keys | **0** — all six manifests parse under `tomllib`, which rejects a true same-table duplicate. Earlier apparent duplicates were an artifact of a naive line scan crossing `[dependencies]` / `[dev-dependencies]` / `[package]` sections |
| resolutions with recorded evidence | **11 of 11** (this file) |

## Regression found and fixed during the pre-merge audit

**Four workflows were dead and nothing said so.**

Resolutions #103 and #141 kept HEAD's step key *and* appended the incoming PR's
same key, producing a step with two `uses:` (and, in `lint.yml`, two `if:`):

```yaml
      - name: Upload CLI artifact
        if: steps.platform.outputs.rust_relevant == 'true'
        uses: actions/upload-artifact@v4     # kept from HEAD
        uses: actions/upload-artifact@v7     # appended from #141
```

That is invalid GitHub Actions YAML. The workflow does not dispatch at all, and the
run reports the least legible failure Actions offers:

```
conclusion=failure   total_count=0
```

No jobs, no steps, no log. The four affected runs were `37179717815` (`lint.yml`),
`37179718434` (`ci.yml`), `37179718955` (`mobile.yml`) and `37179719485` (`cross.yml`) --
exactly the four files those two resolutions touched, and no others.

**Why every review missed it.** `yaml.safe_load` accepts a duplicate mapping key and
silently keeps the last one. So every parse check passed: the tree-wide YAML sweep,
`check_toolchain_pin.py`, `rules_check.py`, and my own duplicate-coordinate audit --
which scanned Gradle `implementation '...'` lines and never looked at YAML keys at
all. The pre-merge audit in this ledger initially reported "0 problems" on a tree
containing nine duplicate keys.

**Fixed** by keeping the incoming value, which is what the PRs were bumping to: `@v7`
supersedes `@v4`, and the `if:` carrying the `docs_only` clause supersedes the bare one.
The guards survive intact -- 15 `is_docs_only` guards in `lint.yml`, and the
platform-relevance guards in `ci.yml`, `cross.yml` and `mobile.yml`. Nine duplicate
pairs across four files; zero remain.

**Guard added so it cannot recur silently:** `scripts/check_workflow_yaml.py`, wired into
the required `Repository Hygiene Checks` context with a `--self-test` step proving the
loader still raises, mirroring `check_toolchain_pin.py --self-test`. A gate that has
quietly stopped failing looks exactly like a gate with nothing to check.

## Resolution 12: `is_route_pending_fresh` hint width (the blocker that failed 17 checks)

`core/src/routing/engine.rs:203` passed `[u8; 8]` to a method declared at
`core/src/routing/global.rs:239` as taking `&[u8; 4]`. That single mismatch cascaded into
17 failing checks, because every job that compiles Rust was reporting the same two errors:

```
error[E0308]: mismatched types          engine.rs:203
error[E0277]: [u8; 8]: Borrow<[u8; 4]> is not satisfied   global.rs:240
```

**This was not a design fork.** Five independent facts agree, with no counter-evidence:

1. The map it queries is keyed 8-wide: `pending_requests: HashMap<[u8; 8], RouteRequest>`
   (`global.rs:76`).
2. Its own body does `self.pending_requests.get(hint)` (`global.rs:240`), which cannot
   typecheck against a 4-byte hint -- so the declared width contradicts the function it is in.
3. The sibling one line above takes the same value 8-wide:
   `is_route_pending(&self, hint: &[u8; 8])` (`global.rs:233`).
4. The test helper that feeds it returns `[u8; 8]`: `fn make_hint(id: u8) -> [u8; 8]`
   (`global.rs:384`).
5. `&[u8; 4]` was the **only** occurrence of `u8; 4]` anywhere in `core/src/routing/`.

`git log -S` shows the definition and the call site arrived in the *same* commit,
`c8e1ce7f6` ("resolve all 10 untriaged perimeter underscore-param violations"), so this was
never two PRs disagreeing -- it is a typo in one commit that only surfaces once the two
halves are in the same tree.

Resolution: `&[u8; 4]` -> `&[u8; 8]` in the declaration. One line, no behaviour change; the
function now matches the map it reads, its sibling, its tests, and its caller.

**Rule-8 note.** `core/src/routing/` is merge-blocked, so this is flagged for a non-author
reviewer rather than waved through. It is recorded here as a mechanical signature correction
decided by the type of the data structure being queried, not as a routing-policy decision:
the freshness window, the map, and every call site were already fixed, and nothing about
route semantics changed.

## Deliberately not in this branch

| PR | reason |
|---|---|
| #364 (76 files, +16,596) | touches `core/src/transport/` — Rule-8 gated, zero formal reviews on the board; also vendors libp2p-swarm 0.48.0 and changes workspace members |
| #413, #411, #351, #367 | `MeshForegroundService.kt` cluster, held with #364 |
| #220, #216, #209 | 23 conflicting files each, stacked on `feat/identity-id-unification`, Rule-8 exposure via `core/src/store` and `core/src/lib.rs` |
| #359 | 49 conflicting files, mostly `HANDOFF/**`, plus `core/src/transport/swarm.rs` |
| #178 | 12 files across `cli/src/ble_*.rs`, `core/src/iron_core.rs`, `core/src/transport/ble/gatt.rs`; Rule-8 gated, on a `gpt/*` base |
| #208 | 2 iOS files; needs `xcodebuild`, which is authoritative only on the operator's Mac |

## Self-reported process failures during this consolidation

Recorded because both happened and both are the kind of thing a later reader would
otherwise have to rediscover:

1. `scripts/harness_gate.py` was committed once **with conflict markers still in it**,
   because a `\U` escape inside the Windows path broke the patch script and the commit
   ran before the parse check. Fixed in the same turn and amended while local and
   unpushed. The tree-wide marker audit above is what now proves none remain.
2. On two occasions a PR was merged on top of an unresolved conflict before the state
   was unwound. Both were caught by re-reading `git status` before committing. The
   practice adopted afterwards: verify parse plus tree-wide markers after **every**
   resolution rather than only at the end.