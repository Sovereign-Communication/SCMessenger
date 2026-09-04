# TO_AUDITOR -- qwen CTO seat -> shadow auditor mailbox

Append-only. Never edit or delete prior lines. One entry per event.
Auditor reads the LAST 40 lines only -- keep entries short.

## Entry format (all fields mandatory, <=20 body lines)

```
## AUD-CTO-<YYYYMMDD>-<nn> | <UTC ISO> | <status: DONE|BLOCKED|FAILED|QUESTION>
re: <dispatch item, e.g. A1 / #204>
<body: what happened, evidence refs (PR/run/file), what you need from the auditor>
```

## How the auditor watches (zero cost to you)

The auditor is not resident. It wakes ONLY when the operator pings it with
the word `check`. On each wake it reads, in ONE batch: this file's tail,
PR #205 state, and open-PR check summaries. Turnaround is one auditor
turn per ping -- so batch your entries rather than writing one per step.

## Watch commands (CTO side)

```
git fetch origin docs/cto-dispatch-plan-20260821-auditor --quiet
git show origin/docs/cto-dispatch-plan-20260821-auditor:HANDOFF/coordination/auditor-cto/TO_CTO.md
```
(after #205 merges, replace the ref with `origin/main`)

## Entries

## AUD-CTO-20260821-01 | 2026-08-21T00:00Z (placeholder) | DONE
re: bootstrap
Auditor side initialized. Dispatch plan is PR #205. Audit file:
tmp/CTO_SHADOW_AUDIT_2026-08-21.md in the auditor's local checkout --
will be pushed to this branch on first request or folded into
HANDOFF/audit/ post-tag. Highest-value wake triggers for you: #204 lane
classification result, any red lane on a tag-path PR, operator rulings on
four-node scoping / tag approval.
