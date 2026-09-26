# V040-T8 -- Restore the WS11 test deleted under a false premise

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: MERGED -- this work landed as PR #271 (merged 2026-09-03, merge commit
36fc1faa). Both acceptance-4 artifacts are on main: the restored test and the
risk-register link. Status corrected 2026-09-19 by the queue-reconcile pass;
the ticket body below is unchanged.

## Why this exists

`V040-T5` (PR #260) repaired the docs-sync gate and, in doing so, found that the
deletion which broke it was itself wrong.

`149d3725` ("fix: resolve all CI failures for PR #139", 2026-08-14) removed
`android/app/src/test/java/com/scmessenger/android/test/DiagnosticsBundleFormatterTest.kt`
on the stated rationale "remove orphaned test (class deleted by iterations)".

**That rationale was false.** Verified 2026-08-31:

```bash
find android -name "DiagnosticsBundleFormatter.kt"
# android/app/src/main/java/com/scmessenger/android/ui/diagnostics/DiagnosticsBundleFormatter.kt

grep -rln "DiagnosticsBundleFormatter" android/app/src/main
# .../ui/diagnostics/DiagnosticsBundleFormatter.kt
# .../ui/screens/DiagnosticsScreen.kt

grep -rln "DiagnosticsBundleFormatter" android/app/src/test android/app/src/androidTest
# (no output -- nothing covers it)
```

The class was never deleted. It exists and is consumed by `DiagnosticsScreen.kt`.
The test was not orphaned, so the coverage is **genuinely lost, not superseded**,
and `DiagnosticsBundleFormatter.format()` has had no test since 2026-08-14.

T5 marked the risk register entry `UNVERIFIED` rather than deleting the link.
This ticket is what clears that `UNVERIFIED`.

## The work

1. Recover the test:
   ```bash
   MSYS_NO_PATHCONV=1 git show '149d3725^:android/app/src/test/java/com/scmessenger/android/test/DiagnosticsBundleFormatterTest.kt'
   ```
   It is 58 lines (verified). Note `MSYS_NO_PATHCONV=1` -- without it, Git Bash
   on Windows mangles the `rev:path` colon and the command fails as a plausible
   emptiness rather than an error.
2. Restore it at its original path, or the current test-source convention if
   that directory has moved -- check where sibling tests live now rather than
   assuming.
3. **Confirm it still compiles against the current class.** The API was
   reported unchanged, but verify rather than assume: if `format()`'s signature
   or output shape has drifted since 2026-08-14, update the assertions to the
   current behaviour and say in the PR exactly what you changed and why. Do not
   weaken an assertion to make it pass -- if the test now fails against real
   current behaviour, that is a finding, and it goes to `inbox/` before you
   change anything.
4. Update `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`: replace the `UNVERIFIED`
   evidence note added by PR #260 with a working link to the restored test.

## Acceptance

- The test file exists and the Android unit-test task passes.
- `grep -rln "DiagnosticsBundleFormatter" android/app/src/test` returns it.
- `bash scripts/docs_sync_check.sh` still exits 0, now with a live link rather
  than the `UNVERIFIED` note.
- The PR states whether the test compiled unmodified, and if not, precisely what
  changed. Never read `$?` after a pipe.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- No hardcoded UI strings in Android code.
- This runs entirely in CI -- no handset required, so it is valid never-idle
  work under `docs/rules/CONTINUOUS_EXECUTION.md` whenever the device is away.
- Shared checkout: touch only what this task requires.

### Status reconciliation (2026-09-19)

The status line said "PR FILED -- #271 open ... awaiting review" while the PR
had been merged for over two weeks, which is how a lane picks already-finished
work off this queue (it nearly happened again on 2026-09-19).

```
gh pr view 271 --json state,mergedAt,mergeCommit,files
#  MERGED 2026-09-03T11:26:07Z  merge commit 36fc1faa
#  files: android/app/src/test/java/com/scmessenger/android/test/DiagnosticsBundleFormatterTest.kt
#         docs/V0.2.0_RESIDUAL_RISK_REGISTER.md
git ls-tree --name-only origin/main android/app/src/test/java/com/scmessenger/android/test/
#  ... DiagnosticsBundleFormatterTest.kt   <- acceptance 2 satisfied
git show origin/main:docs/V0.2.0_RESIDUAL_RISK_REGISTER.md | grep -c UNVERIFIED
#  0 (no UNVERIFIED note remains)          <- acceptance 4 satisfied
```
