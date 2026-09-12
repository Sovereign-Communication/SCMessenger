# V040 Rule-8 adversarial review — PR #276 outbox drop-hop fix (rounds R9-R14, gate CLOSES)

Status: **ROUND 14 — APPROVE (plain). Rule-8 gate for PR #276 CLOSED on the
review side** (filed 2026-09-06 00:40Z; reviewer runs R9-R14 on 2026-09-05).
PR: #276 (freebuff/v040-outbox-transport-fix @ **6359f661**, base
cto/v040-candidate-2026-09-02 e97c3f82). Delta: iron_core.rs, outbox.rs,
manager.rs, swarm.rs — +995/-84 over base (verified `git diff --stat`).

Reviewer identity (RECORDED CHANGE per the quota-ledger rule):

- R1: `qwen3.8-2.4t-a95b` (2026-09-04 21:05Z ledger line; verdict file
  V040_OUTBOX_FIX_REVIEW_QWEN_2026-09-04.md).
- R2 onward: **`qwen3.8-max-0902`** (reserve bucket, expires 2026-11-30,
  825,245 tokens at the 2026-09-04 snapshot). The 2.4t reviewer bucket drained
  below comfortable round size, so the identity moved to the same-tier
  replacement named in docs/QWEN_QUOTA_LEDGER.md. R2 shows two extra probe runs
  in the ledger (2.4t twice, then qwen3-30b-a3b-thinking-2507 once) before the
  accepted max-0902 response at 22:36Z; the accepted R2 artifact matches the
  max-0902 ledger line by mtime. R3-R14 are one continuous max-0902 identity —
  14 ledger lines, all `result: ok`.

Raw artifacts (all verified present this session):

- R14 response: tmp/rev276_r14_response.md (no brief file was saved for R14;
  response + diff are the artifacts). Diff: tmp/rev276_r14.diff.
- R9-R13: tmp/rev276_r{9..13}_response.md + _brief.md + .diff.
- Ledger: tmp/lakes/ledger.jsonl. NOTE: the R14 run is recorded there under
  task label `rev276_r13_brief` (ts 2026-09-06T00:22:51Z, in 15024 / out 853) —
  a labeling slip, not a missing run; in/out tokens and model match the R14
  artifact.

Provenance (rule 13, verified this session): all four blob-hash pairs in
tmp/rev276_r14.diff (`74e89c6a..4626e24d` iron_core.rs, `189ae29e..5d8cb6ae`
outbox.rs, `cdc6db06..085a009e` manager.rs, `6f2f5825..1d0312e5` swarm.rs) are
byte-identical to `git diff e97c3f82 6359f661` on the pushed branch. **R14
reviewed exactly the current PR head 6359f661.**

## Verdict trail (verbatim lines)

- R9 (2026-09-05 19:30Z): `VERDICT: REQUEST_CHANGES`
- R10 (20:04): `VERDICT: REQUEST_CHANGES`
- R11 (21:31): `VERDICT: REQUEST_CHANGES`
- R12 (22:05): `VERDICT: REQUEST_CHANGES`
- R13 (22:23): `VERDICT: REQUEST_CHANGES`
- R14 (2026-09-06 00:22Z): **`VERDICT: APPROVE`**

The iteration contract (REQUEST_CHANGES -> CTO disposition -> re-dispatch) was
followed each round; disposition commits f437a6e8 (R8), e015d371 (R9),
fda3a059 (R10), 95fe0368 (R11), 1fc8c60d (R12), 6359f661 (R13) are the pushed
responses. Worktree: C:/Users/SCM/Documents/GitHub/scm-outbox-fix; shared
checkout untouched for code.

## R14 findings (advisories — the verdict line is unconditional)

- F1 Medium: outbox.rs:453-458 `restore_drained` has no cap check — repeated
  restore under enqueue failure grows the queue unbounded past
  MAX_QUEUE_PER_PEER. Suggested: hard ceiling or warning log.
- F2 Medium: outbox.rs:504-509 `retry_now` returns false on serialize/put/
  flush failure with no event; caller falls back to the 120s grace timer.
  Suggested: Result or tracing event.
- F3 Low-Medium: swarm.rs:6222-6237 `reconnect_request_to_message` entries can
  leak if a request dies without OutboundFailure. Suggested: drain on
  ConnectionClosed.
- F4 Low: WASM identify-vs-ConnectionEstablished ordering race is safe via the
  HashSet gate; reviewer asks the R12 comment say so ("no code change needed").
- F5 Low: OUTBOX_EGRESS_GRACE_SECS hardcoded 120s; suggest config or
  RTT-derived.

None of F1-F5 is written as a merge condition and the verdict line carries no
condition. Disposition is NOT required by the contract (only REQUEST_CHANGES
obliges dispositions). Recommendation: fold F1/F2/F3 into a fast-follow PR or
the 3-node validation hardening list — F1 is the only one with a real
unbounded-growth shape, and it needs memory pressure + repeated enqueue
failure to manifest.

## Gate effect and what remains before merge

- Rule-8 gate (non-author APPROVE with evidence): **CLOSED** at 6359f661.
- CI on 6359f661 was still RUNNING at filing time: pass = Android Wiring Gate
  (8s), iOS Build (15m24s), label (7s); pending = Android JVM Unit Tests,
  iOS Build & Simulator Test, macOS Native Tests, Android Debug APK. macOS has
  no prior green on this PR — treat its result as the open risk.
- No fix commit landed after R14, so no CI re-run is owed.
- Remaining sequence (resume file "Next steps" 4-8): merge #276 into
  cto/v040-candidate-2026-09-02 (per-merge gates + execution log), re-pin the
  #272 verdicts at the new head, rebuild wincli + APK via CI artifacts,
  full 3-node validation on the fixed tree, report to CEO for the tag decision.
  Merge/tag/release remain operator/CEO calls (standing directive).

Filed by the Freebuff seat completing the CTO checkpoint left at tmp/cto/
V040_PR276_R13_STATE.md; every claim above carries the command or file it came
from.
