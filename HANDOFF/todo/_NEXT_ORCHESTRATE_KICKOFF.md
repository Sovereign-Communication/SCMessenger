# NEXT ORCHESTRATOR KICKOFF -- Nemotron Ultra sub-orchestrator takeover

Written: 2026-08-05 by the outgoing Qwen Code orchestrator (API window
exhausted; operator resumes orchestration on Nemotron ultra). Read THIS file
first, then AGENTS.md, then docs/ORCHESTRATION.md Section 0 (five absolute
rules). You are a SUB-orchestrator with a bounded mission. Stay in it.

## Your mission (in order)

1. Get PR #136 fully green and MERGE it.
2. Prepare the 5-node test. DO NOT initiate it (pause gate below).

Current state at handoff: PR #136 branch
fix/identity-canonicalization-steps2-5 at e3af54a3. ONE remaining CI fail at
handoff, confirmed: `Test (macos-latest)` in run 30971518114
(job 92196680972) -- all other jobs pass or pending. Diagnose with
`gh run view 30971518114 --log-failed`. Note: the same four suites pass on
Windows locally, so suspect a platform difference (timing, ports, case
sensitivity), not logic. Local state of the same tree: all four identity/block-gate
suites green on Windows (integration_contact_block, integration_e2e,
integration_ironcore_roundtrip, cli integration_message_requests).

## Hard guardrails (violation = stop and report)

1. SCOPE LOCK: only mission items above. Do NOT start the dependabot
   integration batch, do NOT apply or merge the staged follow-up diffs
   (below), do NOT touch releases/tags, do NOT merge any other branch.
2. CI discipline (operator directive): never commit a PR-scope fix until the
   FULL CI cycle completes; batch every surfaced failure into ONE commit +
   ONE push. Untouched scopes are presumed green -- re-test only what your
   fix touches (cargo -j6, CARGO_INCREMENTAL=0, one build tool at a time).
3. Push authority: you hold AGENTS.md rule 5(b) ONLY for this mission's
   branch and main. No force-push, ever. No pushing other branches.
4. Claude Code LOCKOUT: never launch `claude` / `claude -p` in any form.
   See HANDOFF/todo/CLAUDE_CODE_SONNET_LOCKOUT_2026-08-04.md.
5. Delegated work goes via scripts/delegate_task.py --provider qwenpaid
   (model qwen3.8-max-preview) with ledger recording
   (scripts/lake_route.py --record). Never any anthropic/claude model slug.
6. Security-gated paths (AGENTS.md rule 8): if the remaining CI fail
   requires touching core/src/{crypto,transport,routing,privacy}/, STOP and
   hand back to the operator. Block-gate code elsewhere needs an
   adversarial review on file before merge -- escalate, do not self-review.
7. Ambiguity = halt: write what you know into this file and stop. Do not
   improvise architecture, do not rewrite rules docs.

## Merge procedure for PR #136 (only when ALL checks pass)

1. Confirm review evidence exists: HANDOFF/review/
   PHASE0B_MSGREQ_GATE_REVIEW_QWENPAID_2026-08-04.md and the block-gate
   review docs referenced in HANDOFF/plans/PR_MERGE_UNIFY_PLAN_2026-08-04.md.
2. `git checkout main && git pull --ff-only origin main && git merge --no-ff
   fix/identity-canonicalization-steps2-5 -m "merge: PR #136 identity
   canonicalization + block gate (steps 2-5)" && git push origin main`
   (expect conflicts: none anticipated; if any, stop and report).
3. Watch main CI to green (`gh run watch` or poll `gh pr checks`-style with
   `gh run list --branch main`).

## Staged follow-up work (NOT yours to land -- context only)

Diffs held in tmp/ (qwenpaid-generated, footers parsed, ledgered):
- tmp/t2_keys_validation_response.md (T2, keys.rs curve validation)
- tmp/t3_cli_listing_failclosed_response.md (T3, CLI listing fail-closed)
- tmp/t1a_store_dual_flavor_response.md (T1a, dual-flavor block storage;
  worker noted: sequential fail-closed writes, no true transactions;
  store-error test omitted -- review must cover this)
Design: HANDOFF/plans/T1_BLOCK_FLAVOR_FIX_DESIGN_2026-08-05.md. These are
pre-v0.4.0-tag items owned by the RETURNING Qwen orchestrator (API reset
~21:00 local, cron scheduled). Mention them in your final report; do not
apply them.

## 5-node test PREP (prepare only -- PAUSE GATE before initiation)

Prepare: download the post-merge Android Debug APK artifact from main CI
(`gh run download`), and verify `adb devices` shows the operator's Pixel 6a
(it did NOT at last check -- remind the operator).
PAUSE GATE -- do not initiate the 5-node test unless ALL hold:
(a) main CI fully green post-merge; (b) APK artifact downloaded and present;
(c) operator device connected via adb; (d) operator explicitly confirms go.
Known-open risk you must state to the operator before any go decision: P1
mixed-fleet block bypass is still OPEN (handoff/IDENTIFIER_GATE_FOLLOWUPS_
2026-08-04.md); run-2 fresh installs mostly close it, stragglers do not.

## Final report format

RESULT: DONE|BLOCKED, PR #136 state, merge SHA, main CI state, APK path,
5-node readiness checklist (a)-(d), anything handed back. No emoji. Keep it
under 15 lines. Then stop.
