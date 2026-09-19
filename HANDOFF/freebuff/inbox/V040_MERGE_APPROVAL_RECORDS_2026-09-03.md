# V040 MERGE APPROVAL RECORDS -- post-validation terminal phase (2026-09-03)

Grounding: V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md (sequence + gates),
V040_CEO_DIRECTIVE_REVIEWS_ITERATE_TO_APPROVE_MULTI_TRANSPORT_2026-09-03.md
(review discipline + multi-transport deferral), the verdict files in
HANDOFF/review/. All seven branch tips below were re-verified LIVE on origin
at 2026-09-03 write time -- identical to the plan's table. Merges happen from
LIVE origin branches, never worktrees; each merge is individually
CEO-approved; no self-merge, no tag, no release.

Executor fills the gate rows with raw output and attaches the Rule-8 artifact
to the merge request; the CEO signs the approval line per section, in order.
If a tip has moved since this record, STOP and re-draft that section.

## Gate checklist (repeat per merge; $BR and TIP per section)

- [ ] a) tip check:    git ls-remote origin refs/heads/$BR  -> must print TIP
- [ ] b) delta check:  git diff --quiet origin/main...$BR; echo $?
                       (expect 1 = mergeable; 0 = already merged, ABORT)
- [ ] c) commits:      git cherry -v origin/main $BR  (review the list)
- [ ] d) conflict pre-check:
                       git merge-tree --write-tree --messages origin/main $BR \
                         | grep -E 'CONFLICT|swarm.rs|observation.rs|ledger_entry.rs' \
                         || echo "no conflicts"
- [ ] e) Rule-8 artifact cited (verdict file + reviewer + reviewed SHA == TIP)
- [ ] f) post-merge:   git diff --quiet origin/main...$BR; echo $?  (expect 0)

---

## 1. PR #264 -- T12 CI pacing

- Branch: freebuff/v040-t12-ci-pacing
- Tip SHA (verified 2026-09-03): ec177fd98b8f1215c1f4a32ac2bee6faa2f3346a
- Scope: CI workflows only (9 workflow files; verified diff vs origin/main)
- Rule-8: NOT REQUIRED (no core/src/{crypto,transport,routing,privacy} changes)
- Gates a-f: [ ] [ ] [ ] [ ] [ ] [ ]
- Gate output (executor):

CEO APPROVAL -- merge PR #264 @ ec177fd9 into main:
Approved: ____________  Date: ____________

---

## 2. PR #271 -- T8 restore test

- Branch: freebuff/v040-t8-restore-test
- Tip SHA (verified 2026-09-03): 8fc5881730b2b8561f0b6bdb841db56d787001d1
- Scope: Android test + doc only (DiagnosticsBundleFormatterTest.kt + doc)
- Rule-8: NOT REQUIRED (Android test + doc; verified diff vs origin/main)
- Gates a-f: [ ] [ ] [ ] [ ] [ ] [ ]
- Gate output (executor):

CEO APPROVAL -- merge PR #271 @ 8fc58817 into main:
Approved: ____________  Date: ____________

---

## 3. PR #268 -- T13-F7 hint widen

- Branch: freebuff/v040-t13-f7-hint-widen
- Tip SHA (verified 2026-09-03): 7bafe83dda24ac1578c49e29129afc32b9a546f2
- Scope: core/src/routing + iron_core (routing)
- Rule-8: REQUIRED -- status: PENDING
  pending: HANDOFF/review/V040_T13_F7_CONFIRM_APPROVE_QWEN_2026-09-03.md
  (confirm pass dispatched to the qwen lane; iterate-to-APPROVE per the
  CEO directive. Existing evidence: V040_T13_F7_REVIEW_QWEN_2026-09-01.md --
  reviewer REQUEST_CHANGES with the HIGH finding verified FALSE by the lane;
  does NOT satisfy the plain-APPROVE requirement on its own.)
- Gates a-f: [ ] [ ] [ ] [ ] [ ] [ ]
- Gate output (executor):

CEO APPROVAL -- merge PR #268 @ 7bafe83d into main
(ONLY after the pending confirm-pass verdict is a plain APPROVE on file):
Approved: ____________  Date: ____________

---

## 4. PR #269 -- T14 preexisting-fixes

- Branch: freebuff/v040-t14-preexisting-fixes
- Tip SHA (verified 2026-09-03): b2a7b345d860db50f4c32bc4978894f4e923b93e
- Scope: core/src/transport/swarm.rs (transport)
- Rule-8: REQUIRED -- APPROVE ON FILE:
  HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md
  (reviewer qwen-max; "Verdict: APPROVE" for #269, comment-only suggestions;
  combined 268/269 file)
- Gates a-f: [ ] [ ] [ ] [ ] [ ] [ ]
- Gate output (executor):

CEO APPROVAL -- merge PR #269 @ b2a7b345 into main:
Approved: ____________  Date: ____________

---

## 5. PR #270 -- T14 ephemeral-port

- Branch: freebuff/v040-t14-ephemeral-port
- Tip SHA (verified 2026-09-03): 6fd0230b31bce92059f619557db14cab542495d9
- Scope: transport/observation.rs + swarm.rs
- Rule-8: REQUIRED -- status: PENDING
  pending: HANDOFF/review/V040_T14_EPHEMERAL_CONFIRM_APPROVE_QWEN_2026-09-03.md
  (confirm pass dispatched; iterate-to-APPROVE per the CEO directive. Per the
  directive, the verdict must also cover the transport-agnostic guard check:
  UDP/QUIC source ports are ephemeral too. Existing evidence:
  V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md -- reviewer REQUEST_CHANGES,
  Critical premise verified FALSE by the lane; not a plain APPROVE on its own.)
- Gates a-f: [ ] [ ] [ ] [ ] [ ] [ ]
- Gate output (executor):

CEO APPROVAL -- merge PR #270 @ 6fd0230b into main
(ONLY after the pending confirm-pass verdict is a plain APPROVE on file):
Approved: ____________  Date: ____________

---

## 6. PR #267 -- T13-FDHT gate

- Branch: freebuff/v040-t13-fdht-gate
- Tip SHA (verified 2026-09-03): 80197ef5ed4c54f179bc470000f5c4f0755d2527
- Scope: cli/src/ledger.rs, store/ledger_entry.rs, transport/swarm.rs
- Rule-8: REQUIRED -- APPROVE ON FILE:
  HANDOFF/review/V040_T13_FDHT_FINAL_APPROVE_QWEN_2026-09-03.md
  (reviewer qwen3.8-2.4t-a95b, plain APPROVE @ 80197ef5; recheck dispositions
  in HANDOFF/review/V040_T13_FDHT_RECHECK_QWEN_2026-09-03.md)
- FLAG-3 RESOLVED (2026-09-03): the 09-01 ruling reversal is the reason the
  rework exists and is covered by the recheck + FINAL APPROVE; citation
  HANDOFF/freebuff/inbox/RULING_2026-09-01_PR267_REJECTED_my_ruling_was_wrong.md
- Gates a-f: [ ] [ ] [ ] [ ] [ ] [ ]
- Gate output (executor):

CEO APPROVAL -- merge PR #267 @ 80197ef5 into main:
Approved: ____________  Date: ____________

---

## 7. PR #272 -- architecture candidate (merges LAST)

- Branch: cto/v040-candidate-2026-09-02
- Tip SHA (verified 2026-09-03): 177bd8406e347ef9e119a9305b77305157bd7dca
  (candidate worktree clean at this SHA; CI green 33/34)
- Scope: transport/routing + cli (architecture pass)
- Rule-8: REQUIRED -- APPROVE ON FILE (with FLAG-5 gate):
  HANDOFF/review/V040_CANDIDATE_272_FINAL_APPROVE_QWEN_2026-09-03.md
  (reviewer qwen3.8-2.4t-a95b; delta addendum covers 3891d11c -> 177bd840)
- FLAG-5 GATE (CEO directive 2026-09-03, supersedes the F3 disposition):
  - pending: multi-transport re-review verdict -- deferral-framed plain
    APPROVE at 177bd840 per
    V040_CEO_DIRECTIVE_REVIEWS_ITERATE_TO_APPROVE_MULTI_TRANSPORT_2026-09-03.md
    (scoping: V040_SCOPING_UDP_QUIC_ADMISSION_2026-09-03.md), OR an APPROVE
    for widening at a NEW head after CI. CTO-only dispositions are not an
    acceptable outcome.
- VALIDATION GATE: merge only after the three-node validation at 177bd840
  passes (Windows + Android evidence on file in
  .codebuff_deploy/EVIDENCE-3NODE-177bd840-2026-09-03.md; AWS leg pending
  the CTO's redeploy).
- Gates a-f: [ ] [ ] [ ] [ ] [ ] [ ]
- Gate output (executor):

CEO APPROVAL -- merge PR #272 @ 177bd840 into main
(ONLY after: three-node validation passes; the multi-transport re-review
verdict is on file; FLAG-5 resolves):
Approved: ____________  Date: ____________

---

## Governance guard (applies to every section above)

- No self-merge: the CTO/agents do not merge their own PRs; every merge is
  individually CEO-approved with the PR number in the approval record.
- No tag, no release, no version bump in this sequence.
- Merges from LIVE origin branches only (never a worktree).
- Conflict-prone files re-checked per pair against the ADVANCED main:
  core/src/transport/swarm.rs, core/src/transport/observation.rs,
  core/src/store/ledger_entry.rs (gate d).
- Each gate row carries raw output; the Rule-8 artifact (verdict file +
  reviewer + reviewed SHA) is attached to the merge request.
- If any gate fails or any tip moves, STOP and report; do not proceed to the
  next section without a fresh CEO sign-off on the amended section.