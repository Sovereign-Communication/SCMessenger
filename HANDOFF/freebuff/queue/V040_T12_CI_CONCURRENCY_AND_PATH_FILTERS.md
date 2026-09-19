# V040-T12 -- Stop burning the CI queue on superseded and irrelevant runs

Status: IN REVIEW -- PR #319 (platform-relevance half, 2026-09-19). Filed
2026-08-31 on the operator's CI-pacing directive; acceptance 1 landed on main
earlier and is not re-verified here, acceptance 2 landed and is extended by
this pass, acceptance 3-4 are unverified on this branch.
Priority: P2 -- costs no correctness, costs a lot of wall-clock for everyone
Lane: Freebuff / DeepSeek V4 Flash
Scope: `.github/workflows/*.yml`. **Read the trap in section 3 before editing a
required check -- getting this wrong makes every PR unmergeable.**

## The measurement

Taken 2026-08-31 with the queue under load:

```
repo-wide, last 60 runs:  completed=43  in_progress=3  queued=14
```

Of the 17 unfinished, **10+ belonged to a single docs-only branch**
(`cto/t2-disk-ruling-2026-08-31`, PR #261 -- markdown only), including `CI` x3
and `Cross` x3 across three different SHAs. A documentation branch was consuming
more CI than the 2,172-line Rust change in PR #262.

Two independent causes, both structural:

```bash
# concurrency groups declared, sampled across the main workflows:
ci.yml: 0   cross.yml: 0   lint.yml: 0   mobile.yml: 0
# path filters: only mobile.yml has any, across 16 workflow files
```

1. **No concurrency groups.** Five pushes to one PR queue five full matrices and
   nothing cancels the superseded ones. They all run to completion against code
   nobody will merge.
2. **No path filters.** A single-markdown-file change runs `Cross`
   (multi-target cross-compilation), the Android matrix, `iOS Build & Test`,
   Docker, and CodeQL. PR #260 changed one `.md` file and ran 27 checks.

## 1. Concurrency groups -- do this first, it is the safe half

Add to every workflow that triggers on `pull_request`:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

A new push then cancels the superseded run for that workflow on that ref. This
alone would have collapsed the five queued matrices above into one.

**Do NOT set `cancel-in-progress: true` for `push` on `main`.** Main's runs are
the evidence that trunk is green at a given SHA; cancelling one destroys that
record and leaves a commit whose status is permanently unknown. Either omit
concurrency for the main-push trigger, or use a group keyed on the SHA with
`cancel-in-progress: false`.

## 2. Path filters -- the bigger win, the sharper edge

Non-required workflows may skip entirely on docs-only changes. Safe to add
`paths-ignore` for `**.md`, `HANDOFF/**`, `docs/**` to: `Cross`, `iOS Build &
Test`, Docker workflows, and the Android/Mobile matrix.

Judgement required, and state your reasoning in the PR: some `docs/**` paths are
load-bearing. `docs/rules/*.md` changes agent behaviour and
`scripts/docs_sync_check.sh` reads the docs tree, so `Repository Hygiene` and
the docs-sync check must still run on doc changes. Skipping a *build* on a docs
change is safe; skipping a *docs check* on a docs change is absurd.

## 3. THE TRAP -- required checks must never be skipped

Branch protection on `main` requires exactly four checks:

```
Repository Hygiene, Lint, Rust Linting, Test (ubuntu-latest)
```

If a path filter causes a **required** check not to run, GitHub does not treat
it as passed -- the PR sits in `Expected — Waiting for status to be reported`
**forever** and cannot be merged. Adding `paths-ignore` to any of those four
workflows would brick every docs PR in the repo, including the ones fixing the
damage.

So for those four, do **not** use a workflow-level path filter. Instead let the
job start and short-circuit inside it:

- add a first step that determines whether the change is docs-only,
- guard the expensive steps with `if:` on that result,
- let the job still complete successfully in seconds.

The check reports success, protection is satisfied, and no build runs. Verify
this by opening a throwaway docs-only PR and confirming all four required checks
report success without running a build -- do not assume it, GitHub's skip
semantics are exactly the thing people get wrong here.

## Acceptance

1. Every `pull_request`-triggered workflow has a concurrency group with
   `cancel-in-progress: true`; main-push runs are NOT cancel-on-supersede.
2. A docs-only PR runs the four required checks (fast, no builds) and skips
   `Cross`, `iOS`, Docker, and the Android matrix.
3. A Rust change still runs everything it runs today -- verify by pushing a
   trivial whitespace change to a `.rs` file on a scratch branch and confirming
   the full matrix triggers, then close it.
4. Pushing twice in quick succession to one PR leaves exactly one running
   matrix, not two.
5. Report the before/after check count for a docs-only PR in the PR body.
6. Never read `$?` after a pipe.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Do not change what any check actually asserts. This task changes *when* checks
  run, never *whether they can fail*. A check that stops being able to fail is a
  far worse outcome than a slow queue -- see I-21, where a gate that skipped its
  own comparison reported success.
- Shared checkout: touch only what this task requires.


---

## Evidence appendix -- platform-relevance pass (2026-09-19, PR #319)

Added by the 2026-09-19 pass. Everything above this line is the ticket as filed;
premises that turned out to be stale are named here rather than silently
rewritten.

### The second shape of the same waste

Acceptance 2 kept **docs-only** diffs from starting build lanes. The identical
waste survived one axis over: a diff that touches only one platform's code still
started the **other** platform's lanes. Measured from `.github/workflows/` on
this pass (b529011b):

| lane | runner | started by an Android-only diff | does any step read android/ |
|---|---|---|---|
| iOS Build & Simulator Test (ios-build-test.yml) | macos-latest | yes | no |
| macOS Native Tests (ios-build-test.yml) | macos-14, 60min timeout, ~29min of work | yes | no |
| iOS Build (mobile.yml) | macos-latest | yes | no |
| Swift Linting (lint.yml) | macos-latest | yes | no |

The ios-build-test.yml trigger even disagrees with its own header, which says
"pull requests that touch the iOS/FFI surface". `grep -n android
.github/workflows/ios-build-test.yml` returns the two path-filter lines (58, 77)
and nothing else: no step in either job reads android/. The entry was copied
from mobile.yml, where the Android jobs make it correct.

### What changed

1. `.github/workflows/ios-build-test.yml` -- dropped `'android/**'` from the
   pull_request and push path lists (workflow-level: both its jobs are macOS).
2. `.github/actions/detect-platform-change/action.yml` -- NEW composite action
   modelled on `detect-docs-only`, emitting `ios_relevant` / `android_relevant`.
   It fails open: push/dispatch, an uncomputable diff, or any changed path
   outside the known input sets yields true for both.
3. `.github/workflows/mobile.yml` -- the `ios` job still starts, and its four
   expensive steps (rust-toolchain, rust-cache, XCFramework, xcodebuild) are
   gated on `ios_relevant`. The job is kept rather than skipped because
   section 3's trap is about how GitHub reports checks.
4. `.github/workflows/lint.yml` -- the `swift` job's docs-only step is replaced
   by the platform gate, which subsumes it (same inert set, `^docs/|^HANDOFF/|
   \.md$`), so one owner decides whether those steps run.

No workflow was deleted and no required-context workflow gained a path filter:
`ci.yml`, `hygiene.yml` and `lint.yml` still trigger on every pull_request, so
Repository Hygiene Checks, Lint, Rust Linting and Test (ubuntu-latest) always
report. Verified structurally, and observed on the probe PR named in the PR body.

### Local verification performed

```
python tmp/verify_platform_gate.py
```

15 cases run against the action's real `run:` script, with a stub git supplying
the diff: android-only -> android true / ios false; ios-only -> the reverse;
core/**, mobile/**, Cargo.* -> both true; docs-only -> both false; unknown path,
workflow edit, empty diff and main push -> both true (fail open). All 15 matched
expectation. Every changed YAML parses and every gated `if:` names a step id
that exists in its job.

### Observed on GitHub, not only locally

Two throwaway probes carried the same one-line Android-only change
(`android/gradle.properties`, diff confirmed with `git diff --name-only`); both
were closed without merging and the first probe's runs were cancelled after its
trigger set was recorded, to conserve macOS runners.

| lane | before (probe #317, off main) | after (probe #318, head carries this fix) |
|---|---|---|
| iOS Build & Test | triggered: iOS Build & Simulator Test + macOS Native Tests | not triggered at all |
| iOS Build (mobile.yml) | pending, headed for the full XCFramework + xcodebuild | pass in 10s, steps skipped |
| Android Wiring Gate | triggered | triggered, pass 9s |
| Android JVM Unit Tests / Debug APK | triggered | triggered (the real work for this diff) |

Workflows started for the same diff: 7 before, 2 after. That count is confounded
and is not offered as the headline: `ci.yml`, `lint.yml`, `hygiene.yml` and
`cross.yml` all declare `pull_request: branches: [main]`, so the probe's non-main
base suppressed them regardless of any path filter. Only `mobile.yml` and
`ios-build-test.yml` carry no branch restriction, and those are the two rows that
carry the evidence.

[WARNING] Two claims here are therefore verified structurally and locally, not
observed on GitHub for an Android-only PR: the `Swift Linting` skip, and that the
four required contexts still report. Both need a PR into `main` whose diff does
not include `.github/**`, and that cannot exist until this work lands, because a
branch carrying the workflow change has a `.github/**` diff which the action
treats as unrecognised and therefore fails open on. On PR #319 itself all four
required contexts report normally.

### Deliberately not done here

- The reverse direction (an iOS-only diff still starts mobile.yml's three
  Android jobs) was measured and left alone: those are ubuntu runners and the
  saving is much smaller than the macOS side, while gating them would add a
  repeated `if:` to four more jobs.
- `kotlin` and `javascript` in lint.yml keep no relevance gate, for the same
  cost reason.
- `scripts/build_xcframework.sh` is in mobile.yml's iOS list but not in
  ios-build-test.yml's. That is a pre-existing gap, not introduced or worsened
  by this pass, and changing it would widen what runs rather than narrow it.
