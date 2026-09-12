# V040 REVIEW DISPATCH -- #267 F-DHT gate rework (adversarial, qwen free coder)

Status: **DISPATCHED 2026-09-01 -- REVIEW FILED + TRIAGED.** Model: qwen3.8-max-0902 (fresh 1M). Verdict: `HANDOFF/review/V040_T13_FDHT_REVIEW_QWEN_2026-09-01.md` (APPROVE with 5 findings). **Triage 2026-09-01 (commit `79b4958c`, pushed):** F1 fixed (case-insensitive pair arm + test), F3 fixed (wire-merge observed_peer_ids + test), F2/F4 documented as intentional (doctrine / dial-scheduler), F5 accepted. PR #267 body carries the full triage table. Gates green (core 1402/0, cli 82/0, clippy 0, wasm32 proof, fmt).
Priority: P0 -- rework already REJECTED once by the CEO for an exploitable gate; this is the second review
Lane: **Qwen free -- qwen3.8-max-0902** (fresh 1M quota, 1M context -- operator-enabled 2026-09-01; the hardest assignment gets the strongest model). Full changed files are supplied in context, not just excerpts.
Target: PR **#267** `freebuff/v040-t13-fdht-gate` (3 files: `core/src/store/ledger_entry.rs`, `core/src/transport/swarm.rs`, `cli/src/ledger.rs`)
Reviewer constraint: MUST NOT be the author of the PR. Independent pass only -- do not read the author's own audit note until AFTER your verdict is drafted.

## Why this review exists
The PR gates all DHT/ledger hearsay feeds on OUR store proving the (address, identity) pair
(`locally_verified`). The CEO rejected the first version (exploitable gate), the rework added a
per-(address, peer-id) predicate plus migration guards, and the author's self-audit (PR body) found
and fixed one more bypass (record_identified_peer slot-fill, commit `7e119375`). Your job: attack the
rework's INVARIANT -- "no path writes a wire or legacy peer id onto a locally_verified entry, and no
entry is dialable without our store proving the pair."

## Attack checklist (attack each; reject hearsay claims)
1. `is_locally_verified_pair` (ledger_entry.rs): address-keyed via `entry_for_multiaddr`? never keyed
   on a wire-supplied peer id? health-tier comparison export-equivalent? current-peer binding match only?
   no `observed_peer_ids` consultation?
2. All add_address sites (native ~4545/4697/5181 + the six-site inventory in the PR body): each enforces
   its claimed behavior; the wasm arm inserts NOTHING -- confirm no other wasm path re-adds hearsay.
3. Bypass B: wire merge/migration with the `!locally_verified` guard -- confirm every writer of
   verified state carries the guard, including any NEW path you can construct (look for other
   setter/update paths on the entry struct, not just add_address).
4. mobile_bridge + CLI fail CLOSED on None (never fail open).
5. Ledger-exchange request/response: exact address/pid forms (stripping, case, /p2p/ handling) match
   what the predicate compares.

## Method
- `cd /c/Users/SCM/Documents/GitHub/SCMessenger && gh pr diff 267 > tmp/rev267.diff` (worktree
  `scm-t13-fdht-main` also on disk if you want the full tree: `git -C ../scm-t13-fdht-main diff origin/main`).
- Run `cargo check -p scmessenger-core --all-targets` if you touch nothing; do NOT edit code --
  this is review-only. Fixes, if any, go back to the lane author as findings.
- Rule-8 applies: this touches `core/src/{routing,transport}`-adjacent paths. Your APPROVE must be
  explicitly recorded, non-author, and state the attacks you actually tried.

## Output contract
1. Verdict to `HANDOFF/review/V040_T13_FDHT_REVIEW_QWEN_2026-09-01.md` (use the PR's own attack-log
   format: attack tried -> verdict -> evidence command+line).
2. One comment on PR #267 with the same verdict (APROVE / REQUEST_CHANGES + findings).
3. Reply in `HANDOFF/freebuff/inbox/` with a 3-line note (verdict, file path, anything the author must fix).

Do NOT pad. If nothing holds, list the attacks tried and say so plainly.