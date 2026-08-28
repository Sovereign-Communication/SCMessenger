# Enforcement Rubric

Use this rubric to enforce the philosophy canon consistently.

## Gate Model

Evaluate each proposed change against applicable canon rules.

Statuses:
- `PASS`: clearly compliant with evidence
- `CONDITIONAL`: partially compliant; requires explicit remediation
- `FAIL`: conflicts with rule intent or text

## Severity

- `blocker`: any `FAIL` on non-negotiable rule
- `major`: `FAIL` on negotiable rule
- `minor`: `CONDITIONAL` with low residual risk

## Evaluation Steps

1. Identify applicable canon rules.
2. Collect direct evidence from code/docs/plans.
3. Assign status per rule.
4. Propose precise remediations for non-pass statuses.
5. Determine final verdict.

## Final Verdict Logic

- `APPROVE`:
  - No `FAIL`
  - No unremediated `CONDITIONAL`
- `APPROVE WITH CONDITIONS`:
  - No non-negotiable `FAIL`
  - Remaining issues are remediable and explicitly tracked
- `REJECT`:
  - Any non-negotiable `FAIL`
  - Or unresolved contradictions against top-priority principles

## Enforcement Output Template

| Rule ID | Rule summary | Status | Evidence | Remediation |
|---|---|---|---|---|
| PHIL-001 | ... | PASS | ... | N/A |

After table:
- Residual risk summary
- Required actions with owners
- Final verdict

## Exception Handling

Allow exception only when all are true:
- Clear rationale tied to higher-priority canon principle
- Time-bound window for exception
- Named owner and rollback plan
- Explicit user approval captured
