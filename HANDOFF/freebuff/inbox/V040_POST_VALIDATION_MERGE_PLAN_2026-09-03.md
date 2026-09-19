# V040 Post-validation merge-execution plan (paste-ready, 2026-09-03)

Status: READY FOR EXECUTION after (a) the qwen FINAL APPROVE verdicts and (b)
the three-node validation at 177bd840. Every branch tip below was re-verified
LIVE on origin in this pass (2026-09-03, commands cited). Grounding:
V040_CEO_CTO_SYNC_2026-09-03.md section 3 (sequence), V040_3NODE_ROLLOUT_PLAN_2026-09-03.md
(go-gate), HANDOFF/review/ verdict files (Rule-8 evidence). On any conflict,
this plan's live-SHA table wins over the sync note's older SHAs.

## 0. Verified current state (re-run before each merge)

- All 8 merge-sequence branches are LIVE on origin and NOT in origin/main
  (git merge-base --is-ancestor <tip> origin/main => NOT-IN-MAIN for every
  tip below; verified this pass).
- Every branch tip equals its PR head (git ls-remote refs/heads + refs/pull).
- Rule-8 FINAL APPROVE verdict files exist for #267 and #272 (see table).
- The 2026-09-01 commit `a759e0c7` in the sync note is SUPERSEDED: the
  candidate is now `177bd840` (test-only CI fix; tree == 3891d11c; CI green).
- Re-verify at go-time, in this order:

    git ls-remote origin refs/heads/freebuff/v040-t12-ci-pacing \
      refs/heads/freebuff/v040-t8-restore-test \
      refs/heads/freebuff/v040-t13-f7-hint-widen \
      refs/heads/freebuff/v040-t14-preexisting-fixes \
      refs/heads/freebuff/v040-t14-ephemeral-port \
      refs/heads/freebuff/v040-t13-fdht-gate \
      refs/heads/cto/v040-candidate-2026-09-02
    git -C /c/Users/SCM/Documents/GitHub/scm-v040-candidate status -sb   # clean, at 177bd840

## 1. PR -> branch -> live SHA -> Rule-8 status

| # | Branch | Live tip (2026-09-03) | Scope | Rule-8 required | Non-author APPROVE evidence on file |
|---|--------|----------------------|-------|-----------------|--------------------------------------|
| 264 | freebuff/v040-t12-ci-pacing | ec177fd9 | CI workflows only (9 workflow files) | NO | n/a |
| 271 | freebuff/v040-t8-restore-test | 8fc58817 | Android test + doc only | NO | n/a |
| 268 | freebuff/v040-t13-f7-hint-widen | 7bafe83d | core/src/routing + iron_core (routing) | YES | REVIEW FILE ONLY -- see FLAG-1 |
| 269 | freebuff/v040-t14-preexisting-fixes | b2a7b345 | core/src/transport/swarm.rs (transport) | YES | APPROVE verdict in V040_T13_F7_REVIEW_QWEN_2026-09-01.md (combined 268/269 file) |
| 270 | freebuff/v040-t14-ephemeral-port | 6fd0230b | transport/observation.rs + swarm.rs | YES | REVIEW FILE ONLY -- see FLAG-2 |
| 267 | freebuff/v040-t13-fdht-gate | 80197ef5 | cli/src/ledger.rs, store/ledger_entry.rs, transport/swarm.rs | YES | **FINAL APPROVE: HANDOFF/review/V040_T13_FDHT_FINAL_APPROVE_QWEN_2026-09-03.md** |
| 272 | cto/v040-candidate-2026-09-02 | 177bd840 | transport/routing + cli (arch pass) | YES | **FINAL APPROVE: HANDOFF/review/V040_CANDIDATE_272_FINAL_APPROVE_QWEN_2026-09-03.md** (delta addendum covers 3891d11c -> 177bd840) |

Reviewer identity (all qwen free lane, non-author, ledger-recorded):
#268/#269 qwen-max; #270 qwen3-30b-a3b-thinking-2507; #267/#272
qwen3.8-2.4t-a95b (continuous across R1 -> R2 -> FINAL).

## 2. Merge sequence (dependency-ordered; merge into main, in this order)

Merges happen from LIVE origin branches, never worktrees. Each merge is
individually CEO-approved. After each merge, main advances -- re-run the
per-merge gates against the ADVANCED main for the next branch.

1. PR #264 T12 CI pacing          ec177fd9   (no Rule-8)
2. PR #271 T8 restore test        8fc58817   (no Rule-8)
3. PR #268 T13-F7 hint widen      7bafe83d   (Rule-8 -- FLAG-1)
4. PR #269 T14 preexisting        b2a7b345   (Rule-8 -- APPROVE on file)
5. PR #270 T14 ephemeral-port     6fd0230b   (Rule-8 -- FLAG-2)
6. PR #267 T13-FDHT gate          80197ef5   (Rule-8 -- FINAL APPROVE; revisit the
                                              2026-09-01 ruling reversal first)
7. PR #272 architecture candidate 177bd840   (Rule-8 -- FINAL APPROVE; merge LAST,
                                              only after same-SHA three-node
                                              validation passes; re-verify SHA)

## 3. Per-merge gate (repeat for each of the 7 branches)

    BR=<branch>; TIP=<recorded sha>
    # a) tip unchanged
    git ls-remote origin refs/heads/$BR          # must print TIP
    # b) branch is NOT already merged (expect exit 1 = delta exists)
    git diff --quiet origin/main...$BR; echo $?  # 1 => mergeable; 0 => already merged, ABORT
    # c) remaining commits on the branch (review the list)
    git cherry -v origin/main $BR
    # d) conflict pre-check on the conflict-prone files
    git merge-tree --write-tree --messages origin/main $BR \
      | grep -E 'CONFLICT|swarm.rs|observation.rs|ledger_entry.rs' || echo "no conflicts"
    # e) Rule-8 artifact: cite the verdict file, reviewer, and the EXACT head
    #    SHA the verdict covers (must equal TIP). Plain APPROVE verdicts for
    #    #269/#267/#272; disposition chain for #268/#270 (see FLAGS).
    # f) merge (squash or merge per CEO direction), then confirm:
    git diff --quiet origin/main...$BR; echo $?  # 0 after merge

Conflict-prone files (from sync note section 3 / Section E.3):
core/src/transport/swarm.rs (#267, #270, #269, #272),
core/src/transport/observation.rs (#270, #272),
core/src/store/ledger_entry.rs (#267 F2/F6 clamp work, ledger-unify heritage),
cli/src/ledger.rs (#267).

## 4. Close list -- all ALREADY HANDLED this pass, no action required

Verified via GitHub API (state/merged/merge_commit_sha) + git ancestry:

- PR #265 (freebuff/v040-t12-docs-only-verify, 6488a28b): state=closed,
  merged=false. CLOSED WITHOUT MERGE already. Its only delta over #264 is the
  4-line docs throwaway (git diff ec177fd9 6488a28b = 2 files, +4/-1).
- PR #266 (T1 half2, cf4995a2): closed, MERGED, mcs=67d19d3c == origin/main
  tip; git cherry origin/main => '-'. Content in main.
- PR #258 (fix/dial-hex-route-candidates, 06066cc2): closed, MERGED into
  69a8ba57; 69a8ba57 IS an ancestor of origin/main (verified). Content in main.
- PR #261 (cto/t2-disk-ruling-2026-08-31, c4c18ff6): closed, MERGED into
  9a45b3e7; 9a45b3e7 IS an ancestor of origin/main (verified). Content in main.

The sync note's "close with evidence (cherry - / 3-dot exit 0)" items are
therefore satisfied; nothing to close.

## 5. Open flags the CEO must decide BEFORE the relevant merge

FLAG-1 -- PR #268 (T13-F7) has NO plain non-author APPROVE on file.
HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md contains the reviewer's
verdict REQUEST_CHANGES (1 HIGH, 1 MEDIUM, 2 LOW), then the lane's
verification addendum proving the HIGH is a FALSE POSITIVE (sites are
`unwrap_or([0u8; 8])`, not `[0u8; 4]`; length-checked try_from present) and
dispositioning APPROVE with the rest comment-only. The file itself states
"ANALYSIS only; the Rule-8 APPROVE decisions stay with the CEO seat."
Decision: (a) accept the verified-false-positive disposition chain as the
Rule-8 evidence (CEO call), or (b) dispatch a confirm APPROVE pass like
#267/#272 before merging.

FLAG-2 -- PR #270 (T14-ephemeral) same shape. The reviewer's verdict in
V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md is REQUEST_CHANGES (Critical +
2 Low). The lane's resolution (supersedes an erroneous first addendum) proves
the Critical premise FALSE: both promotion sites (swarm.rs:4061, :5185) sit
inside the native-only cfg block; wasm has its own loop with diagnostics-only
parity; commit 6fd0230b (+16/-1, comments only) documents the empty-set
semantics; disposition APPROVE (no defect); gates green; PR body updated.
Same (a)/(b) decision as FLAG-1.

FLAG-3 -- RESOLVED: the 2026-09-01 "ruling reversal"
(RULING_2026-09-01_PR267_REJECTED_my_ruling_was_wrong.md -- CEO reversed
its own per-identity gate ruling after the reviewer's REJECT at 44eeb1cd)
is the reason the rework exists, and the rework is what was approved. The
09-03 recheck (HANDOFF/review/V040_T13_FDHT_RECHECK_QWEN_2026-09-03.md)
re-audited the branch against the reversal's corrected requirements -- 4
findings, 0 code changes, all dispositioned with evidence -- and the FINAL
APPROVE covers them by name: ledger_verified_pair fails closed
(swarm.rs:2707-2712), pair-gated inserts (4549-4552/4698-4705/5188-5191),
migration gated on !e.locally_verified (ledger_entry.rs:2253-2261), wasm
Identify inserts nothing (7910-7920). The reversal's mobile_bridge.rs:803
UNVERIFIED item is structurally covered: mobile starts the same shared
start_swarm_with_config, and the gate is fail-closed at the insert sites
independent of the core_weak handle. VERDICT: #267 merges as planned at
80197ef5; the FINAL APPROVE at that head stands. No re-review required.

FLAG-4 -- PR #272 merges LAST and ONLY after the three-node validation at
177bd840 passes (per the rollout plan go-gate). Re-verify the SHA at that
moment (section 0). The FINAL APPROVE verdict's delta addendum covers the
3891d11c -> 177bd840 test-only move; the verdict remains valid for the
reviewed code surface.

FLAG-5 -- ADDED 2026-09-03 (CEO directive, supersedes the F3 disposition):
multi-transport is doctrine -- the app is opportunistic over ANY working
transport; TCP first, never only; UDP/QUIC/ws/relay/BLE/mDNS in scope. The
candidate's TCP-only admission (swarm.rs:464-474 listen_port_from_bound_addr
rejects Udp/Quic/QuicV1; sync_external_address promotes /tcp/ only) is under
re-review per the directive. CEO clarification 2026-09-03: UDP may be
DEFERRED for the v0.4.0 gate -- the re-review may end in plain APPROVE at
177bd840 for the TCP-only admission AS A RECORDED DEFERRAL (verdict must
state: deferred not excluded, nothing structurally forecloses the later
change, QUIC already live in the routing ladder, deferral CEO-sanctioned for
v0.4.0 only), or in APPROVE for widening (a) / documented alternative (b) at
a NEW head after CI. CTO-only dispositions remain not acceptable. #272 does
NOT merge until that re-review APPROVE lands. See
HANDOFF/freebuff/inbox/V040_CEO_DIRECTIVE_REVIEWS_ITERATE_TO_APPROVE_MULTI_TRANSPORT_2026-09-03.md
and the scoping note
HANDOFF/freebuff/inbox/V040_SCOPING_UDP_QUIC_ADMISSION_2026-09-03.md.
The #270 confirm pass must also verify the listen-port guard is
transport-agnostic (UDP/QUIC source ports are ephemeral too).

## 6. Governance guards (non-negotiable)

- No self-merge: the CTO/agents do not merge their own PRs; every merge is
  individually CEO-approved with the PR number in the approval record.
- No tag, no release, no version bump in this sequence (release timing is an
  escalate-to-operator item).
- Every transport/routing/crypto merge carries a Rule-8 artifact naming the
  verdict file, reviewer, and reviewed SHA -- attach it to the merge request.
- Merges from live origin branches only (never a worktree).
- Shared-checkout hygiene: stage explicit paths only; never `git add -A`;
  a clean git status is not a goal.

## 7. Return contract (per merged PR)

For each merge, post in one file under HANDOFF/freebuff/inbox/:
- Task header (which PR, which sequence step), Type: DONE
- Raw gate output: git ls-remote tip, git diff --quiet exit code, git cherry
  -v, merge-tree conflict check
- The Rule-8 artifact citation (verdict file + reviewer + SHA)
- PR number and merge commit SHA
- Any residual notes (conflicts resolved how, what the diff carried)