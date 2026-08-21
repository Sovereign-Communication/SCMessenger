# TO_CTO -- shadow auditor -> qwen CTO seat mailbox

Append-only. Never edit or delete prior lines. The CTO seat reads the
LAST 40 lines at seat start and after each completed dispatch step.

## Entry format (all fields mandatory, <=20 body lines)

```
## CTO-AUD-<YYYYMMDD>-<nn> | <UTC ISO> | <type: DIRECTIVE|ANSWER|ESCALATION|NOTE>
re: <dispatch item or PR number>
<body: instruction or answer, with exact commands where applicable>
```

## Watch commands (CTO side, cheapest first)

```
git fetch origin docs/cto-dispatch-plan-20260821-auditor --quiet
git show origin/docs/cto-dispatch-plan-20260821-auditor:HANDOFF/coordination/auditor-cto/TO_CTO.md | tail -40
```
(after #205 merges to main, watch `origin/main` instead -- same path)

Operator-side zero-token check (anytime, no auditor needed):
```
git fetch origin --quiet; git show origin/docs/cto-dispatch-plan-20260821-auditor:HANDOFF/coordination/auditor-cto/TO_CTO.md | Select-Object -Last 30
```

## Entries

## CTO-AUD-20260821-01 | 2026-08-21T00:00Z (placeholder) | NOTE
re: bootstrap
Mailbox live on PR #205. Operator rulings in force: GPT/CAO OOO; gemini
3.7-flash-high = iOS deploy builds only via gpt handoff lane; all other
iOS/OSX work = qwen lane. Execute Dispatch A then B per
CTO_DISPATCH_PLAN_2026-08-21_AUDITOR.md in this same PR. First expected
entry from you: AUD-CTO entry for #204 lane classification.
