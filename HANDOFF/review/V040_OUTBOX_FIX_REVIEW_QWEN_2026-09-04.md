# V040 Rule-8 adversarial review — PR #276 outbox drop-hop fix

Status: **ROUND 1 — REQUEST_CHANGES** (qwen free lane, 2026-09-04 ~21:00Z)
PR: #276 (freebuff/v040-outbox-transport-fix @ 22e23c60, base cto/v040-candidate-2026-09-02 e97c3f82)
Reviewer: qwen3.8-2.4t-a95b (ledger: reserve bucket 11-11, full 1M; same non-author as #273 R1-R4)
Raw response: tmp/rev276_brief_response.md · Brief: tmp/rev276_brief.md · Diff: tmp/rev276.diff
Ledger: tmp/lakes/ledger.jsonl 2026-09-04T21:05:00Z (est in 2189 / out 735)

## Verdict (verbatim from the lane)

`Verdict: REQUEST_CHANGES`

1. **High** swarm.rs:5632 — Identify site calls `handle_peer_connection_event(pk_hex, true)` BEFORE `set_swarm_peer_connection(pk_hex, true)`; flush can run before the registry knows the peer (late identify / reconnect race) and outbox entries stay stranded. Fix: register first, then flush.
2. **Critical** iron_core.rs:3267 + 1073 — marking peers connected makes `prepare_message_internal` skip the outbox toward `send_to_peer`, but the transport-manager outgoing queue has no production consumer; `prepare_message` may grow `pending_sends` without ever delivering (and without retaining an outbox entry). Fix: real egress consumer or keep outbox fallback until delivery proven; caller audit for CLI/API/mobile/relay.
3. **High** swarm.rs:5632/6073 — registration/teardown asymmetric: final close de-registers only when `public_key_hex_from_libp2p_peer_id` is Some; identify handler lacks an active-connection guard; stale `is_peer_connected=true` risk. Fix: track PeerId→registered-pk, deregister from that map, guard identify by live connections.
4. **Medium** iron_core.rs:3278 — `register_transport(Internet, ...)` per connect; test only proves single-peer behavior. Fix: register once at init or add churn/idempotency tests.
5. **Medium** swarm.rs:5809 — None fallback keeps the known base58/hex no-op instead of failing closed. Fix: remove fallback or loud diagnostic + deny direct send.
6. **Medium** iron_core.rs:5065 — test drives the method directly, not real ConnectionEstablished/ConnectionClosed, multi-connection flapping, identify-after-close, hashed peer ids, or delivery. Fix: event-loop/integration tests.

## Disposition (CTO, next pass — verify each against the tree before re-dispatch)

- A2/Critical: partially pre-disclosed in the PR body boundary 1 (outgoing queue has no production consumer). Confirmed for the reconnect-flush arm (`send_to_peer` Ok now parks in the dead queue); the api_axum/api live-send callers do swarm-send after prepare, so the live path delivers. Disposition pending caller audit; likely fix = deliver the flush over the swarm link rather than the transport-manager queue.
- A1: order fix in the identify site (register before flush) — trivially correct, apply.
- A3/A5: canonical-derivation asymmetry — all first-party peers are inline-Ed25519 (verified byte-level for 12D3KooWBPdNE); fallback + hashed-peer edge documented; decide fail-closed vs documented boundary.
- A4: register Internet transport once (idempotent per-peer registration is a HashMap overwrite — verify state-preserving, add multi-peer test if kept).
- A6: add an event-loop-level regression if the surface permits (native swarm loop test harness).

Iteration contract: CTO attaches disposition evidence and re-dispatches; only a plain `Verdict: APPROVE` closes the Rule-8 gate.
