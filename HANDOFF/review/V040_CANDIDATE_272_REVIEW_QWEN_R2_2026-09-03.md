# PR #272 re-review (round 2) — qwen verdict + triage record

- Model: qwen3.8-2.4t-a95b (ledger-confirmed 100% status, 1M context)
- Reviewed head: fc0f5ae0; fixed head: 3891d11c (pushed, PR body carries the table)
- Verdict: REQUEST_CHANGES — 6 findings; 4 VERIFIED REAL + 1 SPLIT + 1 NOT-APPLICABLE

## Disposition summary
- F1 relay-first classification: FIXED (endpoint_transport_string whole-address P2pCircuit scan; test extended with ws/p2p-circuit -> relay)
- F2 parser drops Circuit: FIXED (iron_core arms restored + confidence test); wasm-feed half was a reviewer hunk-position error, feed present, now block-guarded
- F3 UDP/QUIC port allowlisting: FIXED (listen_port_from_bound_addr rejects UDP/Quic/QuicV1)
- F4 wasm asymmetry: NOT-APPLICABLE (no wasm promotion path; wasm mirror never populated)
- F5 reduced regression coverage: FIXED (both deleted tests restored, +ws-circuit case)
- F6 eviction nondeterminism: FIXED (sort_by_reliability ordering reused for eviction)

## Gates on 3891d11c (all PASS)
- cargo check -p scmessenger-core --all-targets: 0 errors
- cargo test -p scmessenger-core --lib: 1400 passed / 0 failed / 5 ignored
- cargo test -p scmessenger-cli --lib: 82 passed / 0 failed
- cargo check -p scmessenger-wasm --target wasm32-unknown-unknown: clean
- rustfmt + git diff --check: clean

## Lane verification addendum
Same standard as prior rounds: each finding verified against the actual tree before any edit
(raw verdict: tmp/rev272b_response.md; diff context: tmp/rev272b.diff). No qwen-attributed
comments posted to the PR per the standing no-posting decision.
