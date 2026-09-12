# V040 Candidate #272 FINAL APPROVE -- qwen free lane (Rule-8 non-author)

Task: V040_REVIEW_DISPATCH_267_272_FINAL_APPROVE_QWEN_2026-09-03.md
Reviewer: qwen3.8-2.4t-a95b (ledger-confirmed 100% / 1M context; same model as R1 and R2 -- one continuous non-author identity)
Target: PR #272 cto/v040-candidate-2026-09-02 @ 3891d11c
Evidence: tmp/rev272c_final.diff (3891d11c vs origin/main), raw response tmp/rev272c_response2.md, usage tmp/lakes/ledger.jsonl

Attack: verified 3891d11c for relay-first classification, parser/test restoration, TCP-only listen-port admission, deterministic eviction, and swept producer/liveness/ownership risks.

Findings:
- None blocking. Info: transport-bonus comments omit Circuit while code still assigns `Circuit => 0.07` (`core/src/transport/swarm.rs:1621-1628`); doc-only cleanup.

Evidence:
- F1 whole-address relay scan: `Protocol::P2pCircuit => return "relay"` (`core/src/transport/swarm.rs:446`); ws-over-circuit test asserts `"relay"` (`swarm.rs:8425-8438`).
- F2 iron_core confidence tests present (`core/src/iron_core.rs:5043`, `:5092`); native/wasm ConnectionEstablished feeds are block-guarded (`swarm.rs:5608`, `:8057`), so wasm empty-engine path is a guarded no-op, not a silent black hole.
- F3 `listen_port_from_bound_addr` rejects UDP/QUIC: `Protocol::Udp(_) | Protocol::Quic | Protocol::QuicV1` (`swarm.rs:469-470`).
- F5 deleted D6 coverage restored and extended (`iron_core.rs:5043`, `:5092`; `swarm.rs:8420-8438`).
- F6 eviction reuses shared ordering via `sort_by_reliability` (`core/src/routing/local.rs:228-233`, `:343-352`).
- Sweep: `sync_external_address` removes only non-primary confirmed addrs and re-adds the primary (`swarm.rs:476-494`); no stale confirmed address remains referenced. `default-run` (`cli/Cargo.toml:3`) does not change no-arg binary behavior.

Verdict: APPROVE

---
## Delta addendum 2026-09-03 (CTO) -- head moved to 177bd840

CI at 3891d11c was red (test_consensus_with_multiple_observations, all 3 OS
matrices): the pre-existing core/tests integration suite was never run by the
local gate (lib-only). The failing test encoded the pre-allowlist contract.
Fixed test-only at 177bd840: set_listen_ports([1234, 5678]) declared before the
observations under test, mirroring swarm NewListenAddr wiring and the arch
pass's own unit tests; all assertions preserved. Code semantics unchanged from
the APPROVE'd 3891d11c. Full workspace suite at 177bd840: 1845 passed, 0
failed (this closes the verification gap). Validation SHA for the three-node
run: 177bd840. Verdict above remains valid for the reviewed code surface.
