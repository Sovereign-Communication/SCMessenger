# V040 Candidate PR #272 Adversarial Review
Scope: independent static review of tmp/rev272.diff (a759e0c7); no author audit consulted.

Attacks tried / result
- Observer admission/replacement/recalculate: pass; non-listen ports rejected and prior observation removed (`observation.rs:45-77`), retention on port-set change (`observation.rs:55-60`), deterministic tie (`observation.rs:141-144`).
- Empty-listener fail-closed: pass; new observer allowlist empty and observations rejected (`observation.rs:45-77`).
- Native listener lifecycle: pass for retraction on Closed/Expired (`swarm.rs:5918-5947`), NewListenAddr updates allowlist (`swarm.rs:5287-5292`).
- Promotion race: pass in shown arms; sync immediately follows record with no visible await (`swarm.rs:4048-4055`, `5150-5154`).
- Local selection determinism: pass for ordering due peer_id tie-break (`routing/local.rs:203-235`).

Findings
1. High — Protocol reconstruction is TCP-only while admission can accept UDP/QUIC/WS ports. `sync_external_address` formats `/tcp/` (`swarm.rs:422-448`) but `extract_socket_addr` treats UDP socket forms as valid (`observation.rs:338-347`), so a UDP/QUIC observation can be advertised as a TCP endpoint. Fix: preserve transport in observations or restrict admission/sync to TCP-only and reconstruct exact multiaddr.
2. High — Transport parser regression: `ws`/`wss` no longer map to TCP and fall through to BLE (`iron_core.rs:119-124`). Any reachable call path passing WebSocket strings will mis-rank WS as BLE. Fix: restore explicit `ws`/`wss` mapping or prove/remove all producers and add regression tests.
3. Medium — `sync_external_address` deletes every non-primary external address (`swarm.rs:441-446`). The "only two callers" claim is not enforced; autonat/relay/future/manual confirmed addresses can be removed, causing divergence from other subsystems. Fix: remove only observer-managed previous primary or tag observer-owned addresses.
4. Medium — Wasm/secondary listener close does not sync advertised set. `sync_external_address` is non-wasm (`swarm.rs:425`) and the wasm `ListenerClosed` arm updates observer but does not retract libp2p external addresses (`swarm.rs:8190-8198`). Stale addresses can remain advertised after listener loss. Fix: platform-safe retraction in all close/expiry paths.
5. Medium — Removed ConnectionEstablished routing feed creates an unverified local-routing gap. The diff removes the visible `routing_peer_seen` feeds in both native and wasm arms (`swarm.rs:5542-5561`, `7975-7990`); without a full call-site grep, LocalCell activation may be broken. Fix: provide grep evidence of remaining callers or restore the feed.
6. Low — Reliability comparator can panic on NaN via `partial_cmp().expect` (`routing/local.rs:227-232`). Deterministic tie-break is otherwise good. Fix: use `f64::total_cmp` or sanitize scores.

REQUEST_CHANGES
---

## Lane verification addendum (2026-09-02, Freebuff lane)

Reviewer model: qwen3.8-2.4t-a95b (ledger-confirmed 100% quota - 1,000,000 remaining, 1M-context November reserve bucket per docs/QWEN_QUOTA_LEDGER.md 2026-08-31). Dispatch: tmp/qwen_review_dispatch.py, usage recorded in tmp/lakes/ledger.jsonl. enable_thinking:true required by this model on this gateway (first attempt with thinking off returned HTTP 400; re-dispatched with --thinking).

Triage (lane author, commit fc0f5ae0): findings 2 (ws/circuit parse regression) and 5 (routing feed removal) VERIFIED REAL and FIXED with restored regression tests; finding 6 (partial_cmp NaN panic) FIXED via total_cmp; findings 1/3/4 verified NOT-APPLICABLE on this tree with evidence (no UDP/QUIC transport in either builder; only two add_external_address callers in-tree and libp2p confirmed set fed only by that API; wasm bound_addresses mirror never populated). Gates on fc0f5ae0: core 1398/0, cli 82/0, all-targets 0 errors, wasm32 clean, fmt/diff clean.

NOTE per ledger never-delegated rule: this is ANALYSIS input; the Rule-8 APPROVE/merge decision stays with the CEO seat.
