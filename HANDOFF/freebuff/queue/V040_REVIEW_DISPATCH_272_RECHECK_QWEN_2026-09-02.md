# V040 REVIEW DISPATCH -- #272 re-review at triage head fc0f5ae0 (qwen free lane)

Status: DISPATCHED 2026-09-02 (re-review after triage commit fc0f5ae0)
Target: PR #272 cto/v040-candidate-2026-09-02 @ fc0f5ae0 (original 7-file pass + fix commit: 3 files +140/-10)
Model: qwen3.8-2.4t-a95b (ledger-confirmed 100%, 1M context; first review of this PR used the same model and returned REQUEST_CHANGES with 6 findings, 2 HIGH)
Context: tmp/rev272b.diff (full PR diff incl. the fix commit) + docs/ARCHITECTURE_SCOPE_V040.md
Reviewer constraint: MUST NOT be the author. Draft the verdict BEFORE reading the PR body's triage table or HANDOFF/review/V040_CANDIDATE_272_REVIEW_QWEN_2026-09-02.md. Read-only.

## What the fix commit changed (verify, do not trust)
1. ConnectionEstablished routing feed RESTORED in both native and wasm arms (calls IronCore::routing_peer_seen with endpoint_transport_string(remote_addr)); endpoint_transport_string restored (relay/quic/ws/tcp from the remote multiaddr).
2. parse_transport_type restored: ws/wss -> TCP; circuit/p2p_circuit/relay -> TransportType::Circuit; TransportType::Circuit variant restored in routing/local.rs; the four route-scoring sites give Circuit its 0.07 bonus.
3. Reliability comparator switched from partial_cmp().expect to f64::total_cmp (two sites).
Regression tests restored: routing_peer_seen_distinguishes_circuit_from_direct_tcp, parse_transport_type_distinguishes_direct_from_circuit.

## Attack checklist (re-review)
1. The restored feed: correct placement (after ledger-exchange block, before reported_peer_discoveries.insert), correct guard (peer_is_blocked), correct string contract (endpoint_transport_string output must round-trip through parse_transport_type: relay->Circuit, ws/wss->TCP, quic->QUIC, tcp->TCP). Any path where the wasm arm's remote_addr hoist changed behavior?
2. The restored Circuit tier: exhaustive matches covered (the 4 scoring sites)? Any remaining non-exhaustive match or dead arm? Serialization compatibility of the enum (serde derive - old stored values)?
3. total_cmp: correct ordering direction preserved at both sites (descending reliability, peer_id tie-break)?
4. RE-ATTACK the original invariants (they still hold after the fix?): observation allowlist fail-closed; sync_external_address lockstep; ExpiredListenAddr/ListenerClosed retraction; promotion sites record-then-sync.
5. Anything in the fix commit that weakens the original 5 clean areas.

## Output contract
1. Verdict to HANDOFF/review/V040_CANDIDATE_272_RECHECK_QWEN_2026-09-02.md (attack -> verdict -> evidence; then plain APPROVE / REQUEST_CHANGES, max 60 lines).
2. Inbox reply 3 lines in HANDOFF/freebuff/inbox/.
Do NOT pad. Do NOT post to the PR.
