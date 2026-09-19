Task: V040_CEO_CLEANUP_MERGE_DIRECTIVE_2026-09-03.md
Type: DONE

## PR disposition (verified live 2026-09-02, `gh pr view`, origin/main still 67d19d3c)

### Corrections to the directive's expected table (verified, do-not-assume)

- PR #262 (T2 unify peer ledgers): MERGED 2026-09-01. Close/delete branch; no pending decision.
- PR #263 (T4 routing feed): MERGED 2026-09-01.
- PR #266 (T1 half2), #260 (docs sync), #261 (T2 disk ruling), #258 (T1 boot seed dial): MERGED. All confirmed content-on-main via GitHub state; note 3-dot/2-dot content-diff is NON-empty for these squash merges because the branches were cut from older main - ancestry/diff cannot prove these, GitHub state is the proof.

### Open v0.4.0-lane PRs (content verified NOT on main, 2-dot diff non-empty)

| PR | Branch | Head | Rule-8 evidence on file | Recommendation |
|---|---|---|---|---|
| #267 T13 FDHT gate | freebuff/v040-t13-fdht-gate | 80197ef5ed | HANDOFF/review/V040_T13_FDHT_REVIEW_QWEN_2026-09-01.md + RULE8_PR267_VERDICT.md (rejected once, rework + triage commit 79b4958c, F1/F3 fixed, F2/F4/F5 documented) | Merge after APPROVE |
| #268 T13 F7 hint widen | freebuff/v040-t13-f7-hint-widen | 7bafe83dda | HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md + self-audit in PR body | Merge after APPROVE |
| #269 T14 preexisting fixes | freebuff/v040-t14-preexisting-fixes | b2a7b345d8 | HANDOFF/review/V040_T14_PREEXISTING_REVIEW_QWEN_2026-09-01.md + self-audit | Merge after APPROVE |
| #270 T14 ephemeral port | freebuff/v040-t14-ephemeral-port | 6fd0230b31 | HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md + self-audit (finding rejected, guard kept) | Merge after APPROVE |
| #264 T12 CI pacing | freebuff/v040-t12-ci-pacing | ec177fd98b | workflow-only (no core/src/{crypto,transport,routing,privacy}) - Rule-8 not triggered | Merge when CI evidence on file |
| #271 T8 restore test | freebuff/v040-t8-restore-test | 8fc5881730 | Android-only test restore; CI 30/30 was reported green | Merge when CI confirms |
| #272 V040 architecture candidate | cto/v040-candidate-2026-09-02 | a759e0c7 | NONE YET - requires non-author adversarial APPROVE (see candidate note) | Blocked on review |

Qwen verdicts (HANDOFF/review/, 2026-09-01) are filed but NOT posted to the PRs - the offer to post with attribution remains open with the CEO.

### Proposed merge order (dependencies first)

1. #267 FDHT gate (doctrine PR; others build on its pair-gate semantics).
2. #268 F7 hint widen (4->8-byte), #269 T14 preexisting, #270 T14 ephemeral - mutually independent content; order among them only matters for conflict surface (all three touch swarm.rs regions; suggest 269, 270, 268 if conflicts appear).
3. #271 T8, #264 T12 (CI-only, any time).
4. #272 architecture candidate LAST among transport/routing PRs (it reconciles to post-merge main; after 267/268/269/270 land, re-run gates and re-base if conflicts).

Each merge requires individual CEO approval. No merge, tag, or release performed by this lane.

## Next decision needed

1. Rule-8 APPROVE for #272 (non-author).
2. Approve merge order above, per-PR.
3. Group C worktree code disposition (see cleanup record).
4. Post qwen verdicts to PRs with attribution: yes/no.
