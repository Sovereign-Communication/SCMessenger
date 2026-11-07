# CircleCI wave-2 port - what was written, what it asserts, what it does not

Date: 2026-11-07
Lane: `lane/burndown` (base `main` `ba7fa778`)
Predecessor: `docs/audit/CIRCLECI_WAVE1_PORT_2026-11-06.md` (wave 1 green,
5 of 5 `ci/circleci:*` contexts on `b3453a188`)

## Why wave 2

Wave 1 put the five branch-protection contexts on an admissible rail. The
burndown needs more than those five: the cluster cars (#643, #609, #656, #507)
are each blocked on a CI-owner decision about a NON-required gate, and those
gates currently live only on Actions, which this account cannot run. Porting
them is what makes those decisions answerable from a rail that returns verdicts.

## What was written

Six jobs added to `.circleci/config.yml` (11 jobs total), commands copied from
the Actions originals:

| Display name | Source | Executor |
|---|---|---|
| `Orchestration control plane` | `ci.yml` job `orchestration` | `python-linux` |
| `Docs` | `ci.yml` job `docs` | `rust-linux` |
| `FFI Surface Contract` | `ci.yml` job `ffi-surface` | `rust-linux` |
| `Forgery/Negative Security Regression Tests` | `security-regression-tests.yml` job `forgery-regression` | `rust-linux` |
| `Kotlin Linting` | `lint.yml` job `kotlin` | `android-linux` (`cimg/android:2026.10.1`) |
| `JavaScript Linting` | `lint.yml` job `javascript` | `node-linux` (`cimg/node:20.20.2`) |

No secrets, no new required contexts, no protection change.

## TWO ENFORCEMENT CAVEATS - both printed by the job on every run

These are the reason this document exists. A ported gate that cannot fail must
never be counted as a passing gate, so each job states its own limit in its own
log rather than leaving the badge to be read as more than it is.

### 1. `Kotlin Linting` runs ktlint but cannot fail on a violation

`android/app/build.gradle` sets, at its `ktlint` block:

    ktlint {
        version = "1.0.1"
        android = true
        ignoreFailures = true
        ...
    }

`ignoreFailures = true` means `ktlintCheck` completes successfully whether or not
it found violations. The Actions job has the same property, so the port is
faithful - and that is exactly the problem: **a green `Kotlin Linting` context
proves ktlint RAN on that commit; it does not prove ktlint passed.** The job
prints `[NON-ENFORCING]` plus any report files it produced.

Flipping `ignoreFailures` to `false` is a behaviour change to the Android build
(C2 cluster territory), not a port, and is deliberately NOT done here: it would
land a red Android gate for pre-existing violations in the middle of the
burndown. It is filed as its own decision-ready item instead - measure the
violation count first, then decide.

### 2. `JavaScript Linting` has no target in this repository

`wasm/package.json` does not exist (read 2026-11-07: `find . -name package.json`
outside vendored/archive trees returns only `.mimocode/`, `.opencode/`,
`log-visualizer/`, and a stale copy under `tmp/`). The Actions original handles
this by printing "No package.json found in wasm directory - skipping JavaScript
linting"; the port keeps that behaviour and adds an explicit
`[NON-ENFORCING]` line, because a skip that reads as a green lint context is the
same failure shape the C2/C5 re-review rejected for #611.

The job enforces automatically if `wasm/package.json` ever appears: target
detection writes `JS_LINT_TARGET` to `$BASH_ENV` and the install and lint steps
branch on it.

**Neither of these two contexts may be added to branch protection.**

## Parity notes (each one a place CircleCI or this port differs, handled on purpose)

1. **No path filtering.** The Actions `docs` and `ffi-surface` jobs carry
   `if: needs.changes.outputs.rust_relevant == 'true'`, and `kotlin` carries
   `kotlin_relevant`. Those are not ported: a path filter is a mechanism that
   makes a context silently skip, and the hygiene job in this same config has a
   step (`check_required_contexts.py`) whose entire purpose is to refuse
   path-filtered contexts. Running always costs minutes and can never report a
   gate that did not run. If run time becomes the binding constraint, the fix
   is a dynamic-config continuation with the filter made visible in the
   pipeline, not a silent skip.
2. **Forgery non-vacuity count under pipefail.** The copied script sums
   `passed; failed` across every harness summary line and fails when the total
   is 0. `grep` exits 1 when nothing matches, so under `set -euo pipefail` a bare
   `MATCHED=$(grep ...)` would abort before the zero-match diagnostic printed.
   The substitution therefore carries `|| true` and `MATCHED=${MATCHED:-0}`:
   the gate still fails closed on zero matches, it just says why. Same class of
   interaction as the wave-1 trailing-whitespace note.
3. **Orchestration needs no dependency install, and proves it.** The contract
   and its evals are stdlib-only (the manifest is JSON-compatible YAML, parsed
   without PyYAML), so instead of installing anything the job first imports
   `orchestration_contract`, `orchestrate_strict` and `jev_canonical_check` from
   `scripts/` and prints `[OK]` - the same "prove the dependency is present"
   shape as wave 1's PyYAML import-back, without the install.
4. **Image pins are read, not guessed.**
   - `cimg/android:2026.10.1` - Docker Hub tag list and the `2026.10`
     Dockerfile were read this session: JDK 21 is the default `JAVA_HOME`,
     `platforms;android-35` and `build-tools;35.0.0` are installed (the project
     is `compileSdk 35`, AGP 8.13.2), and the Gradle 8.13 wrapper the project
     pins runs on JDK 21.
   - `cimg/node:20.20.2` - Docker Hub tag list read this session; the Actions
     job pins major 20, so the newest 20.x point release is pinned here.
5. **`Kotlin Linting` asserts the wrapper version before using it.** A moved
   wrapper is a configuration failure, not a lint failure to triage (same
   reasoning as the wave-1 `rustc 1.99.0` preflight).

## Local verification (this session, real exit codes)

- `.circleci/config.yml` parses under the repository's own duplicate-key loader
  (`scripts/check_workflow_yaml.DuplicateKeyLoader`): `[OK]`. 11 jobs defined, 11
  workflow entries, every entry's job id resolves. Executors: `rust-linux`,
  `python-linux`, `node-linux`, `android-linux`.
- 0 trailing-whitespace lines, LF endings (the hygiene gate's two file-level
  checks, applied to the file that will carry the diff).
- `cd scripts && python3 -c "import orchestration_contract, orchestrate_strict,
  jev_canonical_check"` -> `[OK] orchestration modules import`, exit 0.
- `python scripts/orchestration_contract.py` -> `[OK] orchestration manifest
  2.0.0 is valid`, exit 0.
- `python scripts/test_orchestration_v2.py` -> `Ran 25 tests in 16.403s`, `OK`,
  exit 0. It creates and removes its own worktrees; `git worktree list` after
  the run shows only the three pre-existing trees and `git status` shows no new
  files.

## NOT verified locally, and why

- `cargo doc`, the FFI generation steps and the forgery regression all require a
  full Rust build of the workspace or of `scmessenger-core`. AGENTS.md rule 17
  makes CI the default builder and forbids carrying a local `target/`; the rail
  is the verifier for these three.
- `./gradlew ktlintCheck` needs an Android SDK and downloads Gradle 8.13 plus
  AGP/KSP artifacts. Not run here; the rail is the verifier. This is the
  highest-risk job in the wave - first run downloads the toolchain into the
  container, and its failure mode (if any) will be a configuration error, which
  its own `--stacktrace` output will name.
- `circleci` CLI is still not installed on this host, so no `circleci config
  validate`. The authoritative validation is the first pipeline on the pushed
  head.

## Cost note

The workflow now runs 11 jobs per push. The three heaviest are unchanged from
wave 1 (`Lint`, `Test (ubuntu-latest)`, `Rust Linting`); wave 2 adds `Docs` and
`FFI Surface Contract` (both full Rust builds, cache-backed), `Forgery/...`
(a single-package test build with a 60-minute no-output allowance) and
`Kotlin Linting` (cold Gradle). Caches use the existing `cargo-registry-`
keys; the Android job has no Gradle cache yet, which is deliberate for the first
run (a cache key that has never been proven is just a second variable).

## Still not ported (wave 3 candidates)

`naming-policy` (a real gate, python-only, cheap - next candidate),
`windows-cli-artifact` and every macOS/Windows leg (no such CircleCI executors
without a machine executor), `mobile` (the Android APK jobs), `docker-test-suite`,
`cross-platform-test`, `dist`, `release`, and the scheduled sweeps.

## Next step

Push and read the first pipeline on the new head. Every context is reported as
it reads; the two `[NON-ENFORCING]` jobs are reported as non-enforcing even when
green, and a `Kotlin Linting` success is never cited as a Kotlin lint pass.

## Addendum (2026-11-07, after the first wave-2 battery) - the FFI red, and the one mechanism I could prove

The first wave-2 battery on `1b1308545` read **10 success + 1 failure**. The failure was
`ci/circleci: FFI Surface Contract` (job 45, 06:39:55Z). At the **same commit**, the Actions
`FFI Surface Contract` job - running the same four commands - completed **success** (06:41:20Z),
so the checked-in snapshots agree with the generated surface on that commit and the red is
environmental to the container, not a content disagreement.

**The failing step's own output is not readable from this host.** CircleCI API v2 returns 401
without a token, and the job page redirects to a CircleCI login in the shared browser (verified by
opening `circleci.com/gh/Treystu/SCMessenger/45` and being sent to `/login`). The failing step, the
error text, and the cause are therefore **not asserted anywhere in this record**.

What *is* proven, locally and reproducibly, is that this gate is **locale-dependent**, which is one
of the ways a clean container can disagree with a warm runner:

    LC_ALL=C sort scripts/ffi-snapshots/kotlin-symbols.txt   -> byte-identical to the snapshot
    LC_ALL=en_US.UTF-8 sort scripts/ffi-snapshots/kotlin-symbols.txt -> differs in 68 lines

The reordering is exactly the class of difference that turns into
"ERROR: Kotlin FFI surface changed without snapshot update": a line beginning
``fun `blockPeer` `` sorts before ``fun clear`` under C and after it under a locale that folds
backticks into collation, and **286 of this file's 462 lines** begin with ``fun ` ``
(read: `grep -c '^fun `' scripts/ffi-snapshots/kotlin-symbols.txt` -> 286,
`wc -l` -> 462), so a large fraction of the snapshot is exposed to the difference.

So the snapshot is only reproducible under C/POSIX collation. Any runner whose locale does not
resolve `LC_COLLATE` to C - and which does not pin it - can fail this gate with no change to the
code, and can equally pass it while the surface genuinely drifted, if the drift happens to sort
away. That is a defect in the gate itself, and it exists on the Actions rail too; this port only
made it visible.

Cure applied here: `ffi_surface_contract` now sets `LC_ALL=C` and `LANG=C` at job level, so every
step - the extractors' `sort` included - is collation-stable regardless of the image default. The
snapshot was produced under C collation, so C is the setting that makes the comparison meaningful.

Stated honestly: pinning the locale is **not** claimed to be the cause of job 45. It removes one
proven class of nondeterminism. If the next battery still reads red, the locale hypothesis is
falsified and the remaining candidates (a missing system library for UniFFI library-mode metadata
extraction, or container disk pressure on a cold build) need the job log to separate.

### The experiment came back, and it falsifies my caution rather than my mechanism

`2dffad301` is the same job with exactly one functional change - `LC_ALL` and `LANG` set to `C`
at job level; the only other edit in that commit is this document. The read after it:

| head | `ci/circleci: FFI Surface Contract` |
|---|---|
| `1b1308545` (before the pin) | **failure**, 06:39:55Z, job 45 |
| `2dffad301` (after the pin) | **success**, 07:13:20Z |

(read: `commits/1b1308545.../status` and `commits/2dffad301.../status`, 2026-11-07T07:13Z)

Single variable, opposite verdict, plus a mechanism that reproduces locally at 68 reordered lines.
That is enough to record the cause as **locale-dependent collation in the symbol extractors**, and
to record the corollary the record above would otherwise have been too cautious to state: the
CircleCI container's default collation is NOT C, and an unpinned locale was silently deciding
whether this gate passed.

The Actions job still does not pin the locale. It passes today on its runner's default, which means
the same gate on that rail is one image change away from failing without a code change - and,
worse in principle, could pass while the surface genuinely drifted. That is a finding for the CI
owner, not a lane repair, because `.github/workflows/ci.yml` is shared with the required contexts.
