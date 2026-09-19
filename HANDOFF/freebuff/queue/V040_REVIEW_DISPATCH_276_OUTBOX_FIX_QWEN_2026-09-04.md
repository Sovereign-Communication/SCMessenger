# V040 REVIEW DISPATCH — Rule-8 adversarial review: PR #276 @ 22e23c60 (freebuff/v040-outbox-transport-fix, qwen free lane)

Status: **GATE CLOSED 2026-09-06 — R14 VERDICT: APPROVE (plain) at PR head 6359f661.** Rounds R1-R14: REQUEST_CHANGES x13 (R1-R8, R10-R13 per raw artifacts; R2 onward reviewer = qwen3.8-max-0902 — identity change recorded), then plain APPROVE. No fix commit landed after R14, so no CI re-run is owed; CI on 6359f661 was still running at gate close (macOS = the open risk, no prior green on this PR).
Verdict: **HANDOFF/review/V040_OUTBOX_FIX_REVIEW_QWEN_2026-09-05.md (R9-R14 + provenance)** · R1: HANDOFF/review/V040_OUTBOX_FIX_REVIEW_QWEN_2026-09-04.md
[2026-09-04 header below kept for the R1 record; intermediate rounds live in tmp/rev276_r{2..13}_*]
Raw response: tmp/rev276_brief_response.md · Brief: tmp/rev276_brief.md · Diff: tmp/rev276.diff (200 lines, 2 files)
Model: `qwen3.8-2.4t-a95b` (ledger reserve bucket 11-11, full 1M; SAME model as #273 R1-R4 so the Rule-8 reviewer identity is one continuous non-author)
Ledger: tmp/lakes/ledger.jsonl 2026-09-04T21:05:00Z (est in 2189 / out 735)
Reviewer constraint: non-author, read-only, verdict before PR body, no posting (standing). Iteration contract per V040_NIMBLE_RECYCLE: REQUEST_CHANGES -> CTO disposition evidence -> re-dispatch; only plain APPROVE closes the gate.

## PR #276 — outbox drop-hop fix (register swarm peers in transport manager)

Branch freebuff/v040-outbox-transport-fix, head 22e23c60, base e97c3f82 (cto/v040-candidate-2026-09-02). Delta: core/src/iron_core.rs + core/src/transport/swarm.rs, +154/-2. Gates on the head (Windows host): core check --all-targets clean (2 pre-existing warnings); core lib 1412/0 incl. new regression `swarm_peer_registration_drives_is_connected_and_flush`; clippy --workspace clean (lint allow name corrected to `empty_line_after_doc_comments` — the documented `empty_line_after_docs` is stale for this toolchain); fmt clean; wasm32 check clean.

RCA chain (full): tmp/run-evidence/aws-rebuild-e97c3f82-20260904/rca/RCA-CONFIRMED-DROP-HOP-2026-09-04-2020Z.md — zero production emitters of TransportEvent::PeerDiscovered -> transport manager registry empty -> prepare_message outboxed every live send and the flush returned PeerNotFound forever (0 transport_handoff lines in session). Fix registers swarm peers under TransportType::Internet at the identify/connection-established/connection-closed sites of the shared native swarm loop (covers CLI cmd_start/cmd_relay + mobile_bridge).

## Round-1 findings (summary) and disposition status

- A2 CRITICAL — no egress consumer for the transport-manager outgoing queue (pre-disclosed in PR body boundary 1; confirmed for reconnect-flush arm; live api/CLI send paths swarm-send directly) — disposition in progress.
- A1 HIGH — register-before-flush ordering at identify site — apply.
- A3 HIGH — teardown asymmetry / no active-connection guard on identify — verify; all first-party peers inline-Ed25519 (byte-verified) — likely documented boundary.
- A4 MEDIUM — per-connect register_transport idempotency — verify state-preserving or move to init.
- A5 MEDIUM — None-fallback keeps base58 no-op — decide fail-closed vs boundary.
- A6 MEDIUM — test drives method not event loop — add integration coverage if surface permits.

Next: CTO dispositions against the tree, apply concrete fixes, re-run gates, re-dispatch R2 at the fixed head. No merge/tag until plain APPROVE; APK rebuild + on-device re-proof is the validation step after approval.
