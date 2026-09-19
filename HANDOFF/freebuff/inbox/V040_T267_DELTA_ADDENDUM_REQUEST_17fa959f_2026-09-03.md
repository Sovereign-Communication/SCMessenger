# T267 DELTA ADDENDUM REQUEST -- PR #267 head moved 80197ef5 -> 17fa959f

Date: 2026-09-03 (Z)
Origin: merge-execution seat, per stop-and-report rule (verdict file pins a
head SHA)

## Why

The governing Rule-8 artifact for PR #267 is
HANDOFF/review/V040_T13_FDHT_FINAL_APPROVE_QWEN_2026-09-03.md, whose target
line reads: "Target: PR #267 freebuff/v040-t13-fdht-gate @ 80197ef5" and
whose closing note states the verdict is tied to 80197ef5 content.

Strict branch protection (required_status_checks.strict=true on main)
forced an update-branch after #264/#271/#269/#268 landed: the branch head
moved 80197ef5 -> 17fa959f (2026-09-03 ~20:55Z).

## What changed in the move (verified, evidence below)

- Reviewed head 80197ef5 is an ancestor of 17fa959f (git merge-base
  --is-ancestor -> YES).
- The update added ONLY main content (the already-landed, individually
  Rule-8-gated PRs #264/#271/#269/#268). No PR-own commit was amended,
  reordered, or dropped; the PR diff (main...branch, 16 files, swarm.rs
  165 hunks, ledger gating) is the same content the lane approved at
  80197ef5.
- merge-tree (main, 17fa959f) will be re-checked at merge time; no
  conflicts expected (same-file churn ordering was preserved by plan order).

## Ask

A one-page delta addendum from the reviewing lane (qwen or harness),
ASCII-clean, stating in substance:

1. The FINAL APPROVE at 80197ef5 remains valid for the reviewed content as
   carried at 17fa959f (update-branch only, content byte-intact), or
   any specific re-review the lane requires;
2. Confirmation the pair-gate / migration-gating / observed-id-hygiene /
   Kademlia-bypass findings dispositions still hold at 17fa959f;
3. A plain APPROVE line for PR #267 @ 17fa959f.

File to HANDOFF/review/V040_T13_FDHT_DELTA_ADDENDUM_17fa959f_2026-09-03.md
beside the original verdict.

## Gate state at request time

- #267 CI: re-running at 17fa959f (required 4 checks) -- pending.
- Merge will NOT proceed without the addendum (stop-and-report rule).