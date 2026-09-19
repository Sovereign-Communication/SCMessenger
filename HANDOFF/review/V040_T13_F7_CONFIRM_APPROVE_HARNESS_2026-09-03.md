# V040 T13-F7 CONFIRM-APPROVE -- HARNESS free lane -- 2026-09-03

Target: PR #268 freebuff/v040-t13-f7-hint-widen
Reviewed head: 7bafe83d (re-verified live on origin at filing time)
Lane: harness free lane (panel-based, deterministic convergence)
Closes: FLAG-1 of V040_POST_VALIDATION_MERGE_PLAN_2026-09-03.md
Verdict: APPROVE (all confirm claims found FALSE = no real defects at head)

## Panel

- google/gemma-4-31b-it:free
- minimax/minimax-m3:free
- inclusionai/ling-3.0-flash-fin:free
- judge: cohere/north-mini-code:free (convergence specialist step)
- Cost: $0.000000 total (free tier, ceiling enforced)

## Claims manifest (defect propositions) and per-claim tally

Manifest + verbatim source window: tmp/harness/w268-claims.json +
tmp/harness/w268-window.txt (60 lines, sections A-D from iron_core.rs
823-832 / 1029-1035 / 2725-2732 and routing/local.rs 214-244 at 7bafe83d).

| claim | defect proposition | gemma | minimax | ling | tally |
|---|---|---|---|---|---|
| c1 | unwrap_or([0u8; 4]) used for routing hints | false (1.0) | false (0.95) | false (0.99) | 3/3 false |
| c2 | hint parse lacks a length check | false (1.0) | false (0.92) | false (0.95) | 3/3 false |
| c3 | peers_for_hint unsorted by reliability | false (1.0) | false (0.97) | false (0.99) | 3/3 false |

Deterministic convergence: agreement=high, confidence=1.0, defer=false.
Judge synthesis: "The models agree the claims are false; sorting and
length checks are present." Full raw output: tmp/harness/w268-verdict.json.

## Evidence commands (run at filing)

- git grep -c '\[0u8; 4\]' 7bafe83d -- core/src/iron_core.rs -> ZERO
  (the two hint sites, iron_core.rs:825 and :1031, read
  .unwrap_or([0u8; 8]) -- 8-byte-consistent family)
- git grep -n 'try_from' 7bafe83d -- core/src/iron_core.rs ->
  :2729 filter_map(|hint| <[u8; 8]>::try_from(hint.as_slice()).ok())
  -- length-checked parse present
- core/src/routing/local.rs:216-217 comment "Sort by reliability,
  highest first, exactly like `active_peers`"; sort_by on
  reliability_score at :234-237; tests test_peers_for_hint_sorted_by_reliability
  (:588) and test_active_peers_sorted_by_reliability (:634)

## Human double-check (lane operator, independent of panel)

All three flagged findings are FALSE POSITIVES at 7bafe83d, matching the
prior CTO-side disposition: (1) zero [0u8; 4] in iron_core.rs; (2) the
length-checked try_from parse is in the diff; (3) peers_for_hint sorts by
reliability_score exactly like active_peers, pinned by tests.

## Autonomy ledger

- Chain verified: true (harness ledger verify)
- Entries: task_id scmessenger-268-confirm (panel + judge), chain head
  hash 18ef25607932f43b08a986b8a3dda3578e22ec2b4dec8e6dcde2c5cb11459d9b
  (seq 172, ledger verify independently checkable)

Verdict: APPROVE for PR #268 @ 7bafe83d.