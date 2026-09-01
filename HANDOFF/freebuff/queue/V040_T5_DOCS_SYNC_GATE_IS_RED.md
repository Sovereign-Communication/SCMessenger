# V040-T5 -- Every agent's finalize gate currently fails

Status: OPEN (filed 2026-08-31, CEO audit)
Priority: P2 -- small, but it is corrupting the process
Lane: Freebuff / DeepSeek V4 Flash (or any free lane)
Scope: `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`. Judgement required -- read below.

## The defect

```bash
bash scripts/docs_sync_check.sh > out.txt; rc=$?; tail -3 out.txt; exit $rc
```

on `main`@`69a8ba57` gives:

```
docs-sync-check: broken markdown link in docs/V0.2.0_RESIDUAL_RISK_REGISTER.md ->
  ../android/app/src/test/java/com/scmessenger/android/test/DiagnosticsBundleFormatterTest.kt
docs-sync-check: FAIL
```

The linked test file was deleted in commit `149d3725` with no surviving
equivalent (`git log --diff-filter=D`).

## Why this is not a link fix

`docs-sync` is a mandatory step of the `finalize-checklist` skill. Because it
fails on unmodified `main`, every agent that runs the finalize checklist sees a
red gate caused by something they did not touch. The predictable result is that
agents learn to ignore or bypass the gate -- which is worse than not having one.

## Why this needs judgement, not a delete

The broken link was **evidence backing a row in a residual risk register**.
Deleting the reference silently weakens a risk claim: the register would then
assert a mitigation with nothing behind it.

Required approach:

1. Read the risk row that cites the link. Identify what the deleted test proved.
2. Determine whether the mitigation still holds:
   - If equivalent coverage exists elsewhere, re-point the link and say where.
   - If the coverage was genuinely lost when the test was deleted, the risk row
     must be **downgraded or reopened**, and the register updated to say so.
   - Do not simply delete the link to make the check pass.
3. If the answer is not determinable from the repo, mark the row UNVERIFIED and
   escalate rather than guessing. `UNVERIFIED` is an acceptable answer here.

## Acceptance

- `scripts/docs_sync_check.sh` exits 0.
- The residual risk register states, in one sentence, what happened to the
  evidence for that row and why the current status is correct.
- No other rows silently changed.

## Rules that apply to this task

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Documentation changes must keep Status / Last-updated headers accurate.
- Shared checkout: touch only what this task requires.
- Never read `$?` after a pipe -- capture output to a file, then test the code.
