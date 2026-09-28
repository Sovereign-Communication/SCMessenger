# Harness admission implementation handoff

Date: 2026-09-23  
Target lane: @[Audit : https://github.com/markqvist/reticulum · 738f1a27]  
Target thread: `738f1a27-57df-4870-a52c-ab77e8509a06`  
Source audit: `HANDOFF/review/HARNESS_ADMISSION_AUDIT_HANDOFF_2026-09-23.md`

## Coordination boundary

This document is the implementation handoff. The current tab remains
read-only except for coordination documentation. The Audit lane must perform
all implementation and verification in the existing isolated worktree only:

- worktree:
  `C:\Users\SCM\Documents\GitHub\SCMessenger-v040-harness-plan`
- branch: `freebuff/v040-v050-harness-plan`
- current commit context: `bac5d753`
  (`docs: pin Harness admission and staged rollout`)
- branch relation observed at handoff: ahead of `origin/main` by one commit

Do not touch the original SCMessenger checkout or the external Harness WIP
checkout. Preserve unrelated working-tree changes. Do not commit, push,
open a pull request, or merge from the current tab.

## Exact four release blockers

Implement and verify all four; none is optional.

### 1. Restore LF compliance

The task-owned Python and JSON files are CRLF-only even though
`.gitattributes` requires LF. Normalize only task-owned files. Acceptance:

- `git diff --check` exits 0;
- no CR-byte trailing-whitespace flood remains;
- scoped rules checks pass.

### 2. Constrain report output to a safe, non-destructive contract

`harness_gate.py` currently accepts an arbitrary absolute `--out` and unlinks
it before invoking Harness. It can therefore target production state,
including `scripts/harness_admission.json`. The lint-claims path also lacks a
single output policy.

Implement one narrow shared output contract for generated reports:

- permit output only below the repo-local Harness evidence root, expected at
  `tmp/harness-runs/seat-gates/`;
- resolve paths before validation;
- reject absolute paths outside that root;
- reject traversal, symlink escape, existing directories, and collisions with
  prompts, manifests, source, config, active vendor code, or other repository
  files;
- unlink/create only after validation passes;
- apply the same safe preparation to verify and lint-claims.

Add tests proving an attempted output at
`scripts/harness_admission.json`, under the active Harness checkout, and at
another repository source path returns 4 and preserves every target.

### 3. Strictly validate rollback records inside this repository

`rollback()` currently checks only that `previous` is a dict and has a
matching tag ref. It directly trusts all other fields and accepts an absolute
path.

Validate the complete previous record before use:

- exact required key set and correct JSON scalar/container types;
- non-empty remote, tag, and package version;
- exact `refs/tags/<tag>` immutable ref;
- exact lowercase 40-hex SHA;
- relative root only;
- resolved root inside this worktree and inside
  `tmp/harness-admission/`;
- root distinct from active production and other lifecycle destinations;
- malformed records always become controlled `AdmissionError` values that
  `update_local_harness.py` handles.

Add focused tests for missing fields, wrong types, absolute paths, traversal,
symlink escape, collision with active production, and a valid rollback record.

### 4. Validate report values and pass/fail semantics

The verify gate currently checks only that `verdict`, `consensus`, and
`actual_cost` keys exist. The admitted Harness CLI can emit a result and
exit zero without deriving an acceptance verdict from the report.

Define the wrapper as an acceptance gate and enforce the exact admitted
report contract. At minimum:

- reject null, wrong-type, non-finite, negative, or structurally incomplete
  values;
- validate the consensus object and agreement/defer semantics used by the
  admitted report;
- define the canonical passing verdict explicitly;
- map invalid, failed, deferred, or indeterminate output to nonzero;
- return zero only for the defined passing report;
- share report validation with admission compatibility probes so the probe
  and runtime consumer cannot drift;
- retain fresh-output enforcement so a stale prior report cannot satisfy a
  zero-exit invocation.

Add zero-exit malformed-value, failed-verdict, deferred, stale-output, and
valid-pass tests. If code or compatibility evidence shows the admitted v0.4.1
report uses a different canonical success shape, follow that exact shape and
document the mapping rather than inventing prose heuristics.

## Current unverified test state

The last executed focused command was:

```text
PYTHONDONTWRITEBYTECODE=1 PYTHONIOENCODING=utf-8 \
  python scripts/test_harness_admission.py
Ran 14 tests in 33.166s
OK
```

That run occurred before the latest source and test edits. The current file
contains 19 test methods and has only been AST-parsed, not executed. The Audit
lane must rerun the complete current suite; the earlier 14-test result is not
current evidence.

## Required verification matrix

After implementation:

1. Run all 19 focused admission tests, then add tests required by the four
   blockers and run the expanded suite.
2. Run scoped Ruff check/format validation and syntax compilation for every
   touched Python file.
3. Exercise the real no-op production entry point:
   `python scripts/update_local_harness.py --mode admit-tag`.
4. Snapshot before/after manifest bytes, active HEAD, exact tag, origin URL,
   active clean status, and lifecycle staging.
5. Exercise the real bounded canary:
   `python scripts/update_local_harness.py --mode canary-main`; prove one exact
   main SHA was tested, candidate cleanup ran, and production did not change.
6. Exercise rollback on a hermetic two-release fixture, including handled
   manifest-write failure and next-entry recovery.
7. Run `python scripts/harness_gate.py --kind ledger` against production.
8. Run the sanitized no-key JEV fallback check; no keyed JEV claim unless a
   key is intentionally supplied in the authorized environment.
9. Run `git diff --check`, `bash scripts/docs_sync_check.sh`,
   `python scripts/check_queue_status.py`, and scoped `rules_check.py` checks.
10. Use the repository's CI-primary verifier for the wide sweep. Do not run a
    local Cargo build for this Python-only task.

## Completion report required from Audit lane

Return:

```text
RESULT: DONE|BLOCKED|FAILED
IMPLEMENTATION COMMIT: <sha or UNCOMMITTED>
BRANCH: <branch>
WORKTREE: <path>
VERIFICATION: <exact commands, results, and environment>
FILES: <explicit paths>
ORIGINAL SCMessenger: UNTOUCHED | unexpected pre-existing state only
EXTERNAL Harness: UNTOUCHED | unexpected pre-existing state only
NOTES: <remaining release evidence and concrete blockers>
```

Report the implementation commit if the lane's authority permits a scoped
local commit. Do not merge. If any acceptance item cannot be completed, name
the exact blocker and leave the worktree in a coherent fail-closed state.
