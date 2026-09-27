# Harness admission audit handoff

Date: 2026-09-23  
Worktree: `C:\Users\SCM\Documents\GitHub\SCMessenger-v040-harness-plan`  
Branch: `freebuff/v040-v050-harness-plan`  
Base HEAD: `bac5d753` (`docs: pin Harness admission and staged rollout`)  
Implementation handoff: Audit thread `738f1a27-57df-4870-a52c-ab77e8509a06`

## Audit boundary

This was an audit-only pass after the operator changed the lane to
read-only, with documentation as the only permitted write. No source, test,
configuration, staging, vendor, or external-Harness file was changed after
that instruction. No commit, push, pull request, merge, rollback, reset,
restore, stash, or cleanup was performed.

The worktree already contained a large, uncommitted admission implementation
when audit mode began. The findings below describe that current working tree;
they do not approve it for implementation completion or release.

## Release blockers

### A1. Task files violate the repository LF contract

`git check-attr` and `.gitattributes` require LF for Python, JSON, and
Markdown. Every new admission file is currently CRLF-only:

- `scripts/harness_admission.py`: 667 CRLF, 0 bare LF
- `scripts/harness_source.py`: 218 CRLF, 0 bare LF
- `scripts/harness_gate.py`: 219 CRLF, 0 bare LF
- `scripts/test_harness_admission.py`: 555 CRLF, 0 bare LF
- `scripts/harness_admission.json`: 9 CRLF, 0 bare LF

`git diff --check` exits 2 and reports every line of the tracked
`harness_gate.py` rewrite as trailing whitespace because of the CR bytes.
This is a mechanical gate failure, not a style preference.

Implementation action: normalize only the task-owned files to LF, then run
`git diff --check` and the scoped rules check before any further validation.

### A2. The report gate can delete production state through `--out`

`harness_gate.py` accepts any absolute output path. The current verify path
resolves it, creates its parent, and executes `out.unlink(missing_ok=True)`
before invoking Harness. The only rejected collision is the prompt file.

Consequently, this valid-looking command removes the production admission
manifest before Harness runs:

```text
python scripts/harness_gate.py --kind verify --prompt-file <file> \
  --out scripts/harness_admission.json
```

The lint-claims branch also forwards an arbitrary output path without a
repository-output-root policy.

Implementation action: require all generated reports to remain under a
repo-local evidence root such as `tmp/harness-runs/seat-gates/`, using one
shared preparation helper for verify and lint-claims. Refuse absolute paths
outside that root before unlinking or creating anything. Add tests proving
that an output aimed at `scripts/harness_admission.json`, the active Harness
checkout, or another repository source file returns 4 and preserves the
file.

### A3. Rollback trusts the embedded previous path and shape

`rollback()` validates only that `previous` is a dict and that its `ref`
equals `refs/tags/<tag>`. It then directly indexes `root`, `sha`, `remote`,
`package_version`, and `tag`.

Consequences substantiated from the code:

- missing fields escape as `KeyError`, not the `AdmissionError` caught by
  `update_local_harness.py`;
- non-string or non-path values can escape as `TypeError` or an uncaught
  `OSError` from path resolution;
- `repo_root / previous["root"]` accepts an absolute path, so a malformed
  manifest can make rollback consume and move a matching checkout outside
  this repository;
- a relative path is required only to be currently valid, not to remain in
  the admission staging area.

Implementation action: validate the complete previous record before use:
non-empty remote/tag/version, exact immutable tag ref, 40-hex SHA, relative
staging root, resolved path inside `tmp/harness-admission/`, and a path
distinct from active production. Convert every malformed-shape failure to
`AdmissionError`. Add missing-key, wrong-type, absolute-path, traversal, and
valid-record tests.

## Current implementation state: valid but incomplete evidence

A focused run completed before the final source/test edits with:

```text
PYTHONDONTWRITEBYTECODE=1 PYTHONIOENCODING=utf-8 \
  python scripts/test_harness_admission.py
Ran 14 tests in 33.166s
OK
```

That green run covered the first recovery completion fix, interrupted
promotion/rollback recovery, canary cleanup on validation failure, exact-tag
branch substitution, and malformed zero-exit report rejection.

After that run, and before audit-only mode began, the working tree gained:

- non-empty/non-string production-tag validation in `harness_source.py`;
- finite `--max-cost` validation in `harness_gate.py`;
- stale verify-output removal and prompt/output collision rejection;
- five focused tests covering those changes plus transient post-commit
  pending cleanup recovery.

The file now contains 19 test methods. AST parsing succeeds, but the 19-test
suite has **not** been executed. Do not cite the earlier 14-test run as
validation of the current tree.

## Gate report contract remains presence-only

The verify gate rejects malformed JSON, a non-object root, and missing
`verdict`, `consensus`, or `actual_cost` fields. It does not validate their
types or value ranges. A zero-exit report with a null verdict, non-dict
consensus, or nonnumeric cost currently reaches the success return.

The admitted `v0.4.1` Harness CLI emits the verify result and normally exits
zero; it does not derive a process exit code from the report verdict. The
wrapper must therefore state and enforce whether it is a report producer or
an acceptance gate. If it is a gate, validate the exact admitted report
schema and make an invalid or non-passing result nonzero. If it is only a
producer, rename the success language so a zero exit cannot be mistaken for
a verification pass. This needs an explicit contract decision, not invented
policy.

## Boundaries checked

- Real consumer resolution currently lands on the manifest-owned production
  root, exact remote, exact SHA, package version, and exact `refs/tags/...`
  object.
- The active checkout read during audit was SHA
  `ad4a30052955e1f574c90f426edfc7a274e16ebf`; its Git status output was
  empty.
- Consumer-side `HARNESS_REPO` source override is gone from the current
  `local_harness.py` diff.
- Canary validation is inside `try/finally`, so a failed probe attempts
  candidate cleanup.
- No async entry point exists in this admission surface. No async defect is
  claimed, and no lock or cross-process concurrency mechanism is justified
  by the observed synchronous single-operator lifecycle.
- Power-loss durability, forced process termination between two local
  filesystem operations, and multi-process operators were not substantiated
  as current requirements. Do not add fsync engines, locks, or speculative
  recovery complexity without a reproduced failure.

## Required implementation sequence

1. Fix A1, A2, and A3; keep the output policy and previous-record validation
   narrow and shared rather than adding a general transaction framework.
2. Decide and encode the report acceptance contract described above.
3. Run the 19-test focused suite and scoped Ruff/format checks.
4. Exercise real `update_local_harness.py --mode admit-tag` and
   `--mode canary-main`; snapshot manifest, active HEAD/tag/remote/clean
   state, staging, and production before and after.
5. Exercise rollback against a hermetic two-release fixture, including a
   forced handled failure and next-entry recovery.
6. Run the production ledger gate and no-key JEV fallback check.
7. Run `git diff --check`, docs sync, queue status, scoped rules checks, and
   the repository's CI-primary verifier. No local Cargo build is required
   for this Python-only change.

## Unverified release evidence

- Current 19-test suite after the last edits.
- Current real no-op admission and real `main` canary after the last edits.
- Keyed JEV evaluation.
- Upstream Harness CI for the exact admitted SHA.
- SCMessenger CI.
- Windows/Android/cloud-node release gates; this audit is Python-only and
  does not replace them.

## Required final report format

The implementation thread should report:

```text
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: <exact commands and environments, or NONE>
FILES: <explicit paths>
NOTES: <remaining risks and release evidence not run>
```

Do not mark the admission work complete while A1-A3 remain open or while the
19-test/current real-entry-point matrix is unverified.
