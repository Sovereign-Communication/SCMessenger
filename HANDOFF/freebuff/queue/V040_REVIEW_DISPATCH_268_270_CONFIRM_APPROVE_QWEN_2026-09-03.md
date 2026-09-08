# V040 CONFIRM-APPROVE dispatch -- PR #268 and PR #270 (qwen free lane)

Task: close FLAG-1 and FLAG-2 of V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md
Type: DISPATCH -- final Rule-8 confirmation pass (CEO-authorized 2026-09-03)
Reviewer: qwen free lane, non-author (ledger-recorded)
Targets:
- PR #268 freebuff/v040-t13-f7-hint-widen @ 7bafe83d
  (head unchanged since the 09-01 review, verified this dispatch)
- PR #270 freebuff/v040-t14-ephemeral-port @ 6fd0230b
  (head unchanged since the 09-01 review, verified this dispatch)

## Why this pass exists

Both PRs have adversarial review files on file, but neither carries a PLAIN
non-author APPROVE verdict -- each review ended REQUEST_CHANGES, and the
dispositions to APPROVE were made by the CTO-side lane verifying the findings
were FALSE POSITIVES. The merge plan requires a plain APPROVE verdict from a
non-author for every transport/routing merge. This pass converts the
disposition chains into formal verdicts at the CURRENT heads. Model routing:
same tier as the #267/#272 FINAL APPROVE pattern (qwen3.8-2.4t-a95b if
funded, else the ledger's soonest-expiring funded large-general; console
authoritative per docs/QWEN_QUOTA_LEDGER.md).

## PR #268 -- T13-F7 hint widen (routing; core/src/routing + iron_core)

Review file: HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md
(reviewer qwen-max; verdict REQUEST_CHANGES: 1 HIGH, 1 MEDIUM, 2 LOW)
Prior disposition (verified on the branch, 2026-09-01):
- HIGH "iron_core.rs:820/1026 still use unwrap_or([0u8; 4])" is a FALSE
  POSITIVE: the actual sites (iron_core.rs:825, :1031) read
  unwrap_or([0u8; 8]); zero [0u8; 4] in the file; the family is
  8-byte-consistent.
- MEDIUM "no length check on the parse" is STALE: the diff already carries
  the length-checked try_from parse (iron_core.rs:2729).
- 2 LOW (document sort contract, confirm exclusions): comment-only, optional.

CONFIRM-PASS TASK:
1. Re-run the HIGH check at 7bafe83d: grep the whole iron_core.rs for
   "[0u8; 4]" (expect zero) and confirm the two sites are 8-byte.
2. Re-run the MEDIUM check: confirm the length-checked parse exists and the
   version-skew path fails cleanly.
3. Spot-check the ordering change (core/src/routing/local.rs peers_for_hint /
   active_peers share sort_by_reliability) is as filed.
4. If all hold: issue plain APPROVE for #268 @ 7bafe83d.

## PR #270 -- T14-ephemeral port (transport; observation.rs + swarm.rs)

Review file: HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md
(reviewer qwen3-30b-a3b-thinking-2507; verdict REQUEST_CHANGES: 1 Critical,
2 Low)
Prior disposition (full investigation on the branch, 2026-09-01; supersedes
an erroneous first addendum that had marked the Critical REAL):
- Critical "empty listen_ports on wasm never advertises" premise is FALSE:
  both promotion sites (swarm.rs:4061 consensus, :5185 Identify) sit inside
  the #[cfg(not(target_arch = "wasm32"))] event loop -- the entire native
  swarm construction is wrapped in that cfg block. The wasm32 branch has its
  own event loop with diagnostics-only parity (Identify swarm.rs:7956,
  address-reflection :7584); file-wide add_external_address count = 2, both
  native; wasm has zero promotion calls.
- Accept-any-on-empty was considered and REJECTED (cannot help wasm -- no
  promotion path exists there -- and would weaken the guard on native).
- Resolution: guard kept unchanged; commit 6fd0230b (+16/-1, comments only)
  documents the empty-set semantics at both guards and the wasm
  diagnostics-only parity; gates re-run green; PR #270 body updated.
- 2 Low findings (redundant filters) accepted as-is; timing finding benign.

CONFIRM-PASS TASK:
1. Re-verify the wasm-gating claim at 6fd0230b: confirm the two promotion
   sites are inside the native cfg block and the wasm build has zero
   add_external_address promotion calls. If you can, compile-check
   scmessenger-wasm (cargo check -p scmessenger-wasm --target
   wasm32-unknown-unknown) as the strongest proof.
2. Confirm the guard still fails closed on empty listen_ports on native
   (the ephemeral-source-port class this P0 removes).
3. If all hold: issue plain APPROVE for #270 @ 6fd0230b.

## Verdict contract

- Two files in HANDOFF/review/:
  HANDOFF/review/V040_T13_F7_CONFIRM_APPROVE_QWEN_2026-09-03.md
  HANDOFF/review/V040_T14_EPHEMERAL_CONFIRM_APPROVE_QWEN_2026-09-03.md
- Each: reviewed head SHA, evidence commands + raw output, plain "Verdict:
  APPROVE" or "Verdict: REQUEST_CHANGES" with findings.
- No PR posting required (per the #267/#272 FINAL APPROVE pattern).
- If either head has moved since this dispatch, STOP and report the new SHA
  instead of reviewing a stale tree.

## After this pass

Both verdicts on file -> the merge plan's FLAG-1/FLAG-2 close, and the
sequence (264 -> 271 -> 268 -> 269 -> 270 -> 267 -> 272) is fully
Rule-8-evidenced, ready for per-merge CEO approval after the three-node
validation at 177bd840.