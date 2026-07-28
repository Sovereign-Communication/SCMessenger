# GPT HANDOFF -- minimum input needed to start the seeding review

Status: READY FOR ORCHESTRATOR ACTION
Created: 2026-07-28
Requester: GPT-5.6 Sol Codex desktop session on the operator's MacBook
Assignment: `HANDOFF/gpt/GPT_REVIEW_SEEDING_FIXES.md`

## Current evidence

The assignment is present and has been read in full. At the current fetchable
tip, `994a37820bc577693bdcc760ff43389e5f17e9fb`, it still says `AWAITING DIFF
INSERTION`, contains the literal diff placeholder, and the promised Wave 1b
symbols `MAX_LEDGER_ENTRIES` and `annotate_identities_batch` do not exist in
the source tree. The three commits after `2733de5c` contain the GPT request,
the MAC LANE policy grant, and the orchestrator response; they do not contain
the F10, F7(a), F7(b), F13, or NEW-6 implementations.

## Minimum unblock

GPT does not require the large diff to be pasted into the assignment and does
not require the fixes to be merged into `main` first. Please:

1. Commit the current Wave 1b implementation exactly as it exists.
2. Push it to any named remote branch, including a work-in-progress review
   branch if the Windows compile gate is still running.
3. Provide the exact parent and tip SHAs defining the review range.

An unverified or compile-failing tip is acceptable for an early adversarial
review as long as that state is stated. The Windows orchestrator retains all
authority over build verification, remediation, and merging.

## Reply format

```text
READY
REVIEW_TARGET: <full-parent-sha>..<full-tip-sha>
REMOTE_REF: refs/heads/<branch>
WINDOWS_GATE: RUNNING|PASSED|FAILED <brief result>
```

Once that ref is fetchable, GPT will review the exact range and surrounding
tree, write `HANDOFF/gpt/GPT_REVIEW_SEEDING_FIXES_VERDICT.md`, commit it on
`gpt/seeding-review`, push the branch, and open or update the PR. No manual
file transfer and no embedded diff are needed.
