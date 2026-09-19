# V040 #272 VERDICT RE-PIN -- b0f7ac4e (post-#276-merge head)

Date: 2026-09-06 (~01:2xZ session clock)
PR: #272 (architecture candidate, branch cto/v040-candidate-2026-09-02)
Head being re-pinned: b0f7ac4eb2551dd3d6b7d28a308c1765c824007e
Prior pinned head: e97c3f8247b29dd344467e05137b24f0f110a10a
Trigger: CEO approval file V040_CEO_APPROVAL_PUSH_276_TO_FULL_3NODE_GREEN_2026-09-04.md step 5 -- "Re-pin the #272 gate verdicts (FINAL APPROVE delta + FLAG-5 deferral) at the NEW head if it moved past e97c3f82."

## Why the head moved
PR #276 (outbox drop-hop fix) squash-merged onto the candidate line: merge commit b0f7ac4e, squash of branch freebuff/v040-outbox-transport-fix @ 6359f661.

## Ancestor + delta proofs (commands run this session)
- `git merge-base --is-ancestor e97c3f82 b0f7ac4e` -> exit 0 (e97c3f82 IS ancestor; nothing rewritten).
- `git rev-parse 6359f661^{tree}` == `git rev-parse b0f7ac4e^{tree}` == 1f61d3900e2df2b6afde9a3b2455051cdfc007bb (squash preserved the exact reviewed tree -- zero novel content).
- `git diff --stat e97c3f82..b0f7ac4e`: exactly 4 files (iron_core.rs, store/outbox.rs, transport/manager.rs, transport/swarm.rs), 995+/84- -- all of it the PR #276 reviewed delta. No other file touched.

## Coverage argument
1. Every #272 verdict at e97c3f82 remains valid for everything not in the 4-file delta: the reviewed ancestors (3891d11c, 177bd840, 44fee3c4, 48672b18) are still ancestors of b0f7ac4e, and the FINAL-APPROVE delta re-pin + FLAG-5 deferral APPROVE at e97c3f82 cover the pre-merge tree wholesale (ancestor relation just proven).
2. The 4-file delta is exactly the PR #276 change, which carries its own complete non-author Rule-8 chain: qwen R1 through R14, closing in a plain APPROVE (tmp/rev276_r14_response.md, model qwen3.8-max-0902, usage_src=api, ledger line 2026-09-06T00:22:51Z; review record V040_OUTBOX_FIX_REVIEW_QWEN_2026-09-05.md + prior qwen records). No un-reviewed content exists anywhere in the e97c3f82..b0f7ac4e span.
3. FLAG-5 evidence items re-verified PRESENT at b0f7ac4e line-by-line (same file, moved line numbers only):
   - `listen_port_from_bound_addr` still exists and still rejects UDP/QUIC (swarm.rs:466 @ b0f7ac4e; was :464-472 @ e97c3f82; call sites 6530/6546/9004 unchanged in role).
   - `sync_external_address` /tcp/-only promotion helper intact (swarm.rs:477; all four call sites present: 4487, 5622, 6532, 6548).
   - Ephemeral-source-port guards in observation.rs intact (record-time gate ~:74 and consensus re-filter ~:150, both verbatim from the FLAG-5 evidence window).
   - QUIC remains live in outbound classification and the routing ladder (unchanged files).
   The deferral's structural-additivity claims therefore transfer unchanged to b0f7ac4e; option (c) CEO-sanctioned deferral stands.
4. PR #276 CI at 6359f661: all checks green (Android JVM, macOS Native, iOS Build, iOS Simulator, Android APK, Wiring Gate, label; mergeStateStatus CLEAN at merge). mergeStateStatus of #272-family candidate line: CLEAN post-merge (PR #276 merged; candidate tip = b0f7ac4e).

## Disposition
RE-PINNED: the #272 Rule-8 record (qwen FINAL APPROVE @ 3891d11c/177bd840 + resolution delta APPROVE @ e97c3f82 + final-approve delta re-pin @ e97c3f82 + FLAG-5 deferral APPROVE @ e97c3f82) plus PR #276's own R1-R14 chain constitute complete non-author review coverage of b0f7ac4e. No finding was waived; every prior APPROVE transfers by ancestor proof; the only new code is Rule-8 APPROVED on its own record.

#272-lineage merge readiness at b0f7ac4e is now gated ONLY on: (a) final-tree artifacts (wincli + APK + docker image) at b0f7ac4e, and (b) the full three-node validation with the ON-DEVICE Windows->Pixel Text gate + AWS-relayed hop. The 0.4.0 tag decision itself remains the CEO's.
