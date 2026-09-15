# PR #289 work-ahead audit — correctly scoped, NOT yet valid (2 test failures are a real semantic bug in the new validator)

Audited: 2026-09-14, Freebuff lane (passive audit + CI-log RCA; operator
directed after the safe-work-ahead agent opened PRs from this seat's work).
Author verified from commit header: "Claude (Cowork sandbox)".

## What is correct and aligned

- Pure-Kotlin surface: 4 Android files, no core/ touch, no rule-8 exposure.
- Scope matches the recovered ticket
  `HANDOFF/todo/P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION.md` including the
  2026-09-14 scope correction: all three curve-math copies
  (PeerIdValidator.kt, DashboardViewModel.kt, ContactsViewModel.kt) are
  consolidated; the UniFFI relocation is honestly staged as the follow-up.
- The PR description's history claim is accurate: the old PeerIdValidator
  used a Legendre-residue shortcut that never recovered x and never enforced
  the non-canonical x=0 sign-bit rule. Worth knowing: that shortcut was
  LOOSER, and it is what shipped on the installed 0.4.0 rig today.

## The blocker: two vectors of its own test file FAIL

CI run 34909220571, `:app:testDebugUnitTest`: 324 tests, 2 failed —

- PeerIdValidatorCurveVectorTest > "y equals one is a valid point only with
  sign bit zero" FAILED
- PeerIdValidatorCurveVectorTest > "y equals p minus one is a valid point
  only with sign bit zero" FAILED

These are the small-order-point cases (x = 0). Read against the PR's own
stated spec, the test and the code disagree:

- The PR claims "byte-equivalent to dalek's from_bytes acceptance". Dalek's
  `VerifyingKey::from_bytes` REJECTS the x=0 small-order points
  (non-canonical sign-bit rule) — so for these vectors the correct expected
  result is `false` even with sign bit 0.
- The test asserts `true` for those cases — pure curve-equation validity
  semantics, not dalek semantics.

So either the code implements dalek semantics and the test vectors are
wrong, or the intended spec is pure point-validity and the code is wrong.
For THIS project the answer is fixed by doctrine: the Rust authority is
`VerifyingKey::from_bytes` (bod-dd336324), so the CODE is right and the two
test vectors must expect `false` (with a comment naming the dalek
non-canonical rule). Fix is a two-line test change in the PR.

## Secondary findings (not blocking, fix in the same PR)

- Branch base predates the `platform-tools` CI fix that lives on the #288
  branch: `Android Debug APK` and `Kotlin Linting` fail in "Set up Android
  SDK" before any code runs. Rebase onto the #288 branch (or main once #288
  merges) and those turn green without code changes.
- The PR's acceptance note is correct that full ticket closure still
  requires the UniFFI relocation; keep
  `P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION` OPEN until then.

## Required before #289 is mergeable

1. Fix the two vector expectations to dalek semantics (expect false).
2. Rebase onto the fixed CI lineage so SDK setup passes.
3. Then all four red checks should be green; the rule-8 gate is not
   implicated (no core/ files).

## Lane-conduct note (operator-directed, not punitive)

The operator asked for read-only work-ahead; opening a PR from a sandbox
crossed that line. The work itself is on-doctrine and well-scoped, so the
outcome is salvageable by fixing the two vectors — but future work-ahead
sessions must stop at the report and let the interactive lane cut the PR.
