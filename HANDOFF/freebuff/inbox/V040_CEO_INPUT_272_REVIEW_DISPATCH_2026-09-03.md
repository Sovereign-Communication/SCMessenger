Task: V040_PR_DISPOSITION_2026-09-02.md -- "Next decision needed" item 1
Type: CEO INPUT -- decision given

## Decision: dispatch the #272 Rule-8 adversarial review to the QWEN free lane

Yes. PR #272 (architecture candidate, `a759e0c7`) is the only open v0.4.0 PR with
NO Rule-8 evidence on file, and it is the last dependency in the merge order
(the other six have QWEN verdicts filed). Dispatch it now, per the established
pattern (267/268/269): non-author adversarial reviewer, full changed files in
context, verdict to `HANDOFF/review/`, comment on the PR, inbox reply.

The complete dispatch brief -- attack checklist, method, output contract -- is
in `HANDOFF/freebuff/queue/V040_REVIEW_DISPATCH_272_ARCH_QWEN_2026-09-03.md`.
Use it as-is; it is grounded in the actual diff and the candidate tree, not a
template.

Model routing (quota ledger 2026-08-31 + 09-01 precedent): this is the P0
adversarial transport/routing review -- per the ledger's own routing rule this
genuinely needs the strongest funded capability. The 267 precedent used
`qwen3.8-max-0902` (operator-enabled 09-01, fresh 1M); if that bucket is still
funded per the live console, use it. Otherwise take the strongest funded
large-general bucket, preferring the soonest expiring that can do the job
(e.g. `qwq-plus` 906k exp 10-06 or `qwen-max` 748k exp 10-06) before drawing
the November reserve. The console is authoritative -- the ledger snapshot
predates the 09-01 operator enablement. Send the exact dated model code.

## Still open (not decided here -- do not block on them)

1. Merge order per-PR approval -- CTO proposal in the 09-02 disposition is
   accepted as the working order (267 -> 268/269/270 -> 271/264 -> 272 last);
   each individual merge still requires CEO approval at the moment of merge.
2. Group C worktree disposition -- G.9 resolves scm-t13-fdht (discard, do NOT
   cherry-pick); scm-mailbox binary NARC delta still needs CEO review.
3. Posting the filed QWEN verdicts to PRs with attribution -- still open; will
   rule with the CEO.

Return contract: raw command output, PR number on completion, per-PR evidence
bundle (review file + PR comment + inbox reply). No merge, no push beyond the
review comment, no code edits.