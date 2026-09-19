# V040 -- qwen free lane reviews DISPATCHED directly (2026-09-01)

All four open PRs now carry a non-author adversarial review from the qwen free
lane, dispatched directly via `tmp/qwen_review_dispatch.py` (no operator paste
cycle). Full verdicts + lane verification addenda:

- **#267** (F-DHT rework) -- qwen3.8-max-0902 (fresh 1M): **APPROVE with 5
  findings** (1 HIGH robustness, 1 MEDIUM bootstrap-None pair, 2 LOW, 1 INFO).
  File: `HANDOFF/review/V040_T13_FDHT_REVIEW_QWEN_2026-09-01.md`.
- **#270** (P0 ephemeral port) -- qwen3-30b-a3b-thinking-2507 (qwq-plus
  non-responsive, same-tier fallback): **REQUEST_CHANGES -- the wasm
  empty-listen_ports finding is VERIFIED REAL** (the Identify promotion site is
  not wasm-gated; residual-2 doc claim wrong). Suggested 1-line disposition.
  File: `HANDOFF/review/V040_T14_EPHEMERAL_REVIEW_QWEN_2026-09-01.md`.
- **#268** (hint widen) -- qwen-max (qwen3-32b non-responsive both modes,
  same-tier fallback): HIGH finding **verified FALSE POSITIVE** (`[0u8;8]` on
  the branch, not `[0u8;4]`); **disposition APPROVE**.
  File: `HANDOFF/review/V040_T13_F7_REVIEW_QWEN_2026-09-01.md`.
- **#269** (pre-existing feeds) -- same dispatch: **APPROVE** 4/4 clean.
  File: `HANDOFF/review/V040_T14_PREEXISTING_REVIEW_QWEN_2026-09-01.md`.

Quota-aware per `docs/QWEN_QUOTA_LEDGER.md`: model per task, same-tier fallback
only, usage recorded in `tmp/lakes/ledger.jsonl` (3 calls: ~79k + 19k + 9k input).
The one earlier mis-dispatched call (delegate_task.py rotation -> qwen-plus) was
discarded, not filed.

Per the ledger's never-delegated rule, these are ANALYSIS inputs; the Rule-8
APPROVE/merge decisions stay with the CEO seat. Next decision needed: #270's
wasm-parity disposition (fix the guard or correct residual-2).