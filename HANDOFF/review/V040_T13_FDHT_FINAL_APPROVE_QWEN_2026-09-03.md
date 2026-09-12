# V040 T13-FDHT FINAL APPROVE -- qwen free lane (Rule-8 non-author)

Task: V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md
Reviewer: qwen3.8-2.4t-a95b (ledger-confirmed 100% / 1M context; same model as R1 and R2 -- one continuous non-author identity)
Target: PR #267 freebuff/v040-t13-fdht-gate @ 80197ef5
Evidence: tmp/rev267c_final.diff (80197ef5 vs origin/main), raw response tmp/rev272c_response2.md, usage tmp/lakes/ledger.jsonl

Attack: spot-verified 80197ef5 dispositions for pair-gate strictness, migration gating, observed-id hygiene, and closure of Kademlia bypass paths.

Findings:
- None blocking. Info: `hex_casefold` checks length/case but not charset (`core/src/store/ledger_entry.rs:2404-2406`); safe here because verified bounds are dial-proven and live callers pass canonical `PeerId::to_string()`.

Evidence:
- Hex arm fires only on `trimmed.len() == 64 && trimmed.eq_ignore_ascii_case(bound)`, with exact/canonical arms failing closed (`ledger_entry.rs:2404-2406`).
- Migration observed-peer import is gated by `!e.locally_verified` (`ledger_entry.rs:2253-2261`).
- `record_observed_peer_id_locked` enforces dedupe + cap (`ledger_entry.rs:528-542`).
- Kademlia inserts are pair-gated (`core/src/transport/swarm.rs:4549-4552`, `:4698-4705`, `:5188-5191`); `ledger_verified_pair` fails closed (`swarm.rs:2707-2712`); wasm Identify inserts nothing (`swarm.rs:7910-7920`).

Verdict: APPROVE

---
## Note 2026-09-03 (CTO)

#267 @ 80197ef5 unchanged -- this verdict is unaffected by the #272 test-only
delta (different branch).
