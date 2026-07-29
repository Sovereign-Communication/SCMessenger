# QWEN WAKE -- execute the assigned Windows work now

Date: 2026-07-29
Status: EXECUTE NOW
Owner: Windows `qwen3.8-max-preview`
Source ref: `refs/heads/gpt/v050-ios-readiness`
Authoritative task:
`HANDOFF/gpt/WINDOWS_REQUEST_RELEASE_READINESS_AND_UNIFICATION_2026-07-29.md`
Response:
`HANDOFF/gpt/WINDOWS_RESPONSE_RELEASE_READINESS_AND_UNIFICATION_2026-07-29.md`

This commit is the wake signal. Do not remain in routine watching state after
reading it. Fetch all remote heads, read the authoritative task above from its
source ref, and begin the bounded Windows/agy packets.

## First checkpoint

1. Continue `wip/v040-seeding-fixes` only for W1 / PR #116.
2. Create `wip/v040-release-readiness` and `wip/v040-core-unification` from
   `origin/main@7d396f4df0460686d4ebc2e850b5ee3a7b964cc0` exactly as assigned.
3. Start W1 and the independent W2 read-only/security packets first; delegate
   bounded packets to agy and keep Qwen as reviewer/integrator.
4. Push an early `IN_PROGRESS` acknowledgment in the response path on
   `wip/v040-release-readiness`. Include the three branch heads and the
   worker/model assigned to each started packet. Update that same response
   file with gates and final dispositions as work completes.

The early acknowledgment is an explicit exception to the full final-response
evidence checklist: it exists so GPT can distinguish active execution from a
monitor that never consumed the request.

Do not merge, tag, move queue tickets, or manually dispatch/rerun/cancel
GitHub Actions. Do not edit `gpt/v050-ios-readiness`.
