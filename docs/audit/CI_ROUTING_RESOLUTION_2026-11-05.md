# CI routing resolution decision - `ci.yml` conflict on #609 and #643

Status: decision recorded, artifact verified, nothing landed
Date: 2026-11-05
Lane: `lane/burndown` (SCMessenger)

This document is the explicit resolution decision for the one conflicted path
shared by the two `dirty` cars, `.github/workflows/ci.yml` (register row A24),
plus the C1 verdict for #660. It records a **decision**, not a merge: the lane
may not merge, and both PRs are CI-owner items.

The artifact `ci_resolution_2026-11-05/643-guarded.patch` applies cleanly to
main's current files and reproduces the verified build exactly
(`git apply --check` then `git apply`, then `diff` against the build tree,
14:33Z).

## 1. C1 (#660) - nothing to fix, and nothing a re-run can cure

#660 head `210b7d932f4e`, base `main`, `MERGEABLE`, `behind_by=0`, **5/5
required contexts success**, **0 reviews**. It carries 14 non-success jobs, and
every one is non-code:

| class | jobs |
|---|---|
| billing refusal (`steps=0`, empty runner) | 13 |
| never acquired by a hosted runner | 1 (`Test (macos-latest)`, `126648237559`) |

They span five workflows, none of them the required set: `CI` (Test
windows-latest, Windows CLI Artifact), `Lint` (Swift, Kotlin, JavaScript), `iOS
Build & Test`, `Mobile` (Android JVM, Android Debug APK, Android Wiring Gate,
Detect changed areas), `Cross` (WASM), `Auto Label` (read: check-run
annotations and `actions/runs/<id>/jobs` per job id, 14:27-14:29Z).

So C1 has **zero product failures**. A re-run changes nothing: the same hosted
labels are refused by the same account lock. #660's actual gate is clause 10
(two non-author Jev PASSes) with zero reviews on file, and A16 records that the
only collaborator is the owner's own account. Decision needed: a named
non-author reviewer (SR-6); the 13 refusals clear with SR-1.

## 2. #643 - resolvable, but only as a split

#643 (`0340c094468f`) touches three files: `.github/workflows/ci.yml` +19/-14,
`.github/workflows/security-regression-tests.yml` +4/-1, `scripts/ci_pool_route.py`
+151/-0. It conflicts on `ci.yml` alone.

### 2a. Guard-compatible subset: ADOPTED, verified

Exactly two hunks preserve every reviewed mechanism while adding #643's
dependabot carve-out:

1. `ci.yml` required leg: keeps `vars.USE_OCI_RUNNER != 'false'` and
   `vars.CI_RUNNER == 'oci'`, adds
   `&& github.event.pull_request.user.login != 'dependabot[bot]'`.
2. `security-regression-tests.yml` `forgery-regression`: the same carve-out on
   a line that already carries the guard.

Verification, all from the artifact applied to main's files in an isolated tree
(no workflow in this repository was modified to produce it):

- `scripts/check_workflow_yaml.py` -> exit 0, `[OK] 8 workflow/action files
  contain no duplicate mapping keys`.
- `scripts/ci_route_check.py` -> exit 0, **8/8 invariants OK**, including "a
  Rust-relevant diff still takes the pool" and "the guard never fires on any
  contract fixture".
- Rollback switch preserved: `USE_OCI_RUNNER` occurrences `ci.yml` 8 -> 8,
  `security-regression-tests.yml` 1 -> 1. This is A12's exact metric, and the
  subset does not move it.

### 2b. Rejected as-authored: the pool-allocation re-scope

The rest of #643 is not a carve-out. Its own contract script rejects main's
routing for **seven** CI jobs - `ci_pool_route.py` output against the resolved
tree, exit 1:

```
- ci.yml docs: selects a self-hosted pool directly; must go through vars.CI_SMALL_RUNNER
- ci.yml ffi-surface: selects a self-hosted pool directly; must go through vars.CI_SMALL_RUNNER
- ci.yml changes: unapproved consumer of the OCI pool
- ci.yml handoff-scope: unapproved consumer of the OCI pool
- ci.yml naming-policy: unapproved consumer of the OCI pool
- ci.yml lint: unapproved consumer of the OCI pool
- ci.yml orchestration: unapproved consumer of the OCI pool
```

That is broader than A12 recorded (which caught the Test-leg guard deletion).
Two reasons it is deferred rather than resolved:

1. **`CI_SMALL_RUNNER` is unset** (read: `actions/variables` 14:26Z). The
   expression `vars.CI_SMALL_RUNNER || 'ubuntu-latest'` therefore resolves to
   hosted for Docs and FFI Surface, and hosted is exactly what the billing lock
   refuses. Adopting it now converts two currently-green jobs into refusals.
2. It is a pool-allocation policy change for the CI owner, not a merge
   resolution. It does not conflict mechanically; it conflicts with the
   reviewed allocation.

Decision: **split #643.** Land the two carve-outs from
`ci_resolution_2026-11-05/643-guarded.patch` as one small PR; send the
pool-allocation re-scope (`ci_pool_route.py` + the Docs/FFI/handoff/naming/lint
routing) to the CI owner as its own proposal, ideally after SR-1 is cured. The
carve-outs alone are worth landing: they keep dependabot re-heads off the two
ARM slots, which is #643's stated purpose.

## 3. #609 - not resolvable as written; its safe form is an inversion

#609 (`4ed8168bbf1f`, 46 behind main) touches `.github/workflows/ci.yml` +7/-5
and `.github/actions/linux-build-deps/action.yml` +31/-10. Its routing hunk
rewrites five jobs to:

```yaml
runs-on: ${{ vars.USE_UBICLOUD == 'false' && 'ubuntu-latest' || 'ubicloud-standard-2' }}
```

`USE_UBICLOUD=true` is set (read 14:26Z), so every one of those jobs resolves to
`ubicloud-standard-2`, and `actions/runners` lists only `oci-a1-phx-1` and
`oci-a1-phx-2` (read 14:26Z) - no runner carries that label. **Two of the five
are required contexts** (`Handoff ownership scope`, `Lint`), so landing it as
written strands two required checks permanently and breaks main's green base by
construction. Resolving the conflict on the guard-preserving side would delete
the PR's routing content entirely; resolving it on the branch's side re-creates
A11.

Safe form, if the owner wants the shape rather than the label: invert the
default so an unprovisioned variable can never strand a required context:

```yaml
runs-on: ${{ vars.USE_UBICLOUD == 'true' && 'ubicloud-standard-2' || 'ubuntu-latest' }}
```

With `USE_UBICLOUD=true` this is *still* unsafe until a runner exists; with the
variable unset or `false` it degrades to hosted. The only landing that is safe
today is to keep `ubuntu-latest` and take the independent
`linux-build-deps` `mode: install` change, which touches no routing.

Decision: **hold #609's routing hunks until #607 provisions the runner class**,
then land the inverted form so the failure mode is hosted rather than stranded.
The `action.yml` `mode` change can go separately if the CI owner wants it.

## 4. What is needed, and from whom

| car | decision | owner |
|---|---|---|
| #660 | name a non-author reviewer; the 13 refusals clear with billing | owner / CI owner |
| #643 | land the verified carve-out patch, split off the re-scope | CI owner |
| #609 | provision the runner class, then land the inverted form; or withdraw | infra owner (#607) |
| #507 | structural, see section 5 - billing, not a re-run | operator (billing) |

## 5. Side finding - why #507's red is structural, not re-runnable

#507 is a **Kotlin-only** diff (`android/.../BleGattClient.kt` +28/-10). Its
Test leg therefore has `rust_relevant != 'true'`, and main's guard sends it to
hosted `ubuntu-latest` - confirmed on run `42003660559`, where `Lint` and
`Handoff ownership scope` ran on `oci-a1-phx-1` while `Test (ubuntu-latest)`
carried the literal `ubuntu-latest` label, an empty runner and `steps=0`.

Both available routes end red, by design:

- **hosted**: refused by the billing lock (the observed failure);
- **self-hosted**: refused by the job's own first step, "Refuse an empty
  required check on the scarce pool", which fails any self-hosted run with
  `rust_relevant != 'true'`.

`scripts/ci_route_check.py` enforces that second behaviour as a contract
invariant ("the emptied leg stays on hosted runners when the diff is inert
(docs-only and android-only must not take the pool)"), so no re-run, refresh or
`update-branch` on #507 can produce a green `Test (ubuntu-latest)` while hosting
is locked. The cure is SR-1, or a deliberate change to the inert-diff contract
that #642's review put in place - an owner/CI-owner decision, recorded as
register row A25.

## Reads

- `pulls/660`, `pulls/643`, `pulls/609`, `pulls/507` + `commits/<head>/check-runs?filter=latest`
  and per-job annotations, 14:23-14:31Z.
- `actions/variables` and `actions/runners`, 14:26Z.
- `actions/runs/42003660559/jobs` (#507), 14:28Z.
- `pulls/643/files`, `pulls/609/files`; `git diff <merge-base>..<head>` for both,
  14:30Z.
- Validators in an isolated tree under `tmp/ciresolve_v2/`, 14:31-14:33Z;
  `git apply --check` + apply + `diff` of the artifact, 14:33Z.
- Main `ba7fa77851f0` re-read 14:29:17Z: 15 non-success check-runs, 15/15 billing
  annotation, 5/5 required contexts success - order (b)'s classification stands
  unchanged and main has not moved.
