# V040 REVIEW DISPATCH -- FINAL Rule-8 APPROVE pass: #267 @ 80197ef5 and #272 @ 3891d11c (qwen free lane)

Status: **COMPLETE 2026-09-03 -- BOTH PLAIN APPROVE** (first launch returned a vacuous
REQUEST_CHANGES -- payload lacked the final trees; re-dispatched with
tmp/rev272c_final.diff + tmp/rev267c_final.diff attached; both APPROVE verdicts on file).
Verdicts: HANDOFF/review/V040_CANDIDATE_272_FINAL_APPROVE_QWEN_2026-09-03.md (272 @ 3891d11c),
HANDOFF/review/V040_T13_FDHT_FINAL_APPROVE_QWEN_2026-09-03.md (267 @ 80197ef5).
Model: `qwen3.8-2.4t-a95b` (ledger-confirmed 100% / 1M context, reserve bucket 11-11; SAME model as R1 and R2 so the reviewer identity on the Rule-8 artifact is one continuous non-author)
Context files: `tmp/rev272b_response.md` + `tmp/rev272b.diff` (R2 raw verdict/diff, if still present); prior verdict records `HANDOFF/review/V040_CANDIDATE_272_REVIEW_QWEN_R2_2026-09-03.md`, `HANDOFF/review/V040_CANDIDATE_272_REVIEW_QWEN_2026-09-02.md`, `HANDOFF/review/V040_T13_FDHT_RECHECK_QWEN_2026-09-03.md`
Reviewer constraint: MUST NOT be the author of either branch. Read-only. Draft the verdict BEFORE reading the PR bodies' triage tables. Do NOT post to the PR (standing no-posting decision).

This is the pass that closes the Rule-8 gate recorded as "pending" in the triage
notes. The two verdict files below are the approval artifacts; without them
neither PR merges. For BOTH PRs: verify against the committed tree at the
stated SHA, do not trust the triage tables, and return a plain verdict.

## PR #272 -- architecture candidate @ 3891d11c (review first, it is the harder one)

Branch: `cto/v040-candidate-2026-09-02`. Commit chain vs main:
a759e0c7 (arch pass) -> fc0f5ae0 (R1 fixes) -> **3891d11c (R2 fixes)**.
Full surface: `git diff origin/main...3891d11c --stat` -- expect core/src/
transport/{swarm,observation}.rs, core/src/routing/{local,optimized_engine}.rs,
core/src/iron_core.rs, cli/Cargo.toml, plus tests.

R1 had 6 findings (2 REAL + 1 partial_cmp + 3 N/A); R2 had 6 (4 REAL + 1 SPLIT +
1 N/A). Your job is NOT to re-derive those tables -- it is to verify the FINAL
head, per finding, then sweep the whole diff for anything both rounds missed.

Verify at 3891d11c:
1. F1 relay-first classification: `endpoint_transport_string` scans the WHOLE
   address for P2pCircuit (circuit-over-WS classifies as relay, not ws); test
   extended with ws/p2p-circuit -> relay.
2. F2 parser arms: circuit/relay parse arms restored in iron_core; the two
   confidence tests present at iron_core.rs:5043 (`routing_peer_seen_raises_confidence_after_connection_established`) and :5092 (`..._distinguishes_circuit_from_direct_tcp`). The wasm-feed half was ruled a reviewer hunk-position error -- confirm the wasm ConnectionEstablished arm at swarm.rs:8057 feeds `routing_peer_seen` (block-guarded), same as native at swarm.rs:5608.
3. F3 `listen_port_from_bound_addr` rejects UDP/Quic/QuicV1 (TCP-only allowlist).
4. F5 both deleted D6 tests restored (+ws-circuit case).
5. F6 deterministic eviction reuses `sort_by_reliability` ordering.
6. NEW SWEEP -- the two rounds may still have missed: (a) `routing_peer_seen`
   and `update_reliability` callers are live and reachable (native + wasm); is
   the wasm empty-engine path a real no-op or a silent black hole? (b)
   `TransportType::Circuit` producers after the diff -- is every variant
   constructible or dead? (c) the `sync_external_address` consensus-primary
   deletion: does anything still reference a deleted confirmed address? (d) the
   CLI default-run change in cli/Cargo.toml -- does the binary still behave
   identically with no args?

Verdict file: `HANDOFF/review/V040_CANDIDATE_272_FINAL_APPROVE_QWEN_2026-09-03.md`
(attack -> verdict -> evidence, then plain APPROVE or REQUEST_CHANGES, max 60 lines).

## PR #267 -- T13-FDHT gate @ 80197ef5 (confirm the recheck, then APPROVE)

Branch: `freebuff/v040-t13-fdht-gate`. The 2026-09-03 R2 recheck dispositioned
all 4 qwen findings NOT-APPLICABLE/CONFIRMED with zero code changes. Verify the
disposition holds at the committed tree (do not re-derive from scratch, spot-
verify the cited lines): (1) hex_casefold arm fires only at len==64 hex; (2)
migration observed_peer_ids gated on `!e.locally_verified` (ledger_entry.rs
2253-2261); (3) record_observed_peer_id_locked dedupe+cap at 528-542; (4)
pair-gated Kademlia inserts (swarm.rs 4549-4552/4698-4705/5188-5191) with
ledger_verified_pair failing closed (2707-2712) and wasm Identify inserting
nothing (7910-7920). If the disposition holds, issue the formal plain APPROVE.

Verdict file: `HANDOFF/review/V040_T13_FDHT_FINAL_APPROVE_QWEN_2026-09-03.md`
(attack -> verdict -> evidence, then plain APPROVE or REQUEST_CHANGES, max 40 lines).

## Output contract

1. Two verdict files (paths above), each: attack -> verdict -> evidence, then a
   PLAIN one-line verdict. No "with findings" ambiguity: plain APPROVE means the
   SHA is merge-ready on your review.
2. Inbox reply, 3 lines, in `HANDOFF/freebuff/inbox/`:
   `Task: V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md`,
   `Type: DONE | BLOCKED`, one line per verdict.
3. Do NOT pad. Do NOT post to the PR. Read-only.