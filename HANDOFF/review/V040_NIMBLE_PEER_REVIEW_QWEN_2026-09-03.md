# V040 Rule-8 adversarial review -- ROUND 2 FINAL: PR #273 @ d82978ab

Reviewer: non-author adversarial reviewer (qwen free lane, qwen3.8-2.4t-a95b -- ledger-confirmed 100% / 1M context, reserve bucket 11-11; same reviewer identity as the #267/#272 final-approve pass)
Date: 2026-09-03
Base: 177bd840 (candidate head)
Head: d82978ab (freebuff/v040-nimble-peer)
Delta: core/src/transport/swarm.rs, core/src/transport/dial_policy.rs, core/src/routing/resume_prefetch.rs

## Per-surface verdicts

- core/src/transport/dial_policy.rs: APPROVE
- core/src/transport/swarm.rs: APPROVE
- core/src/routing/resume_prefetch.rs: APPROVE

## Round-1 finding recheck (R1 was REQUEST_CHANGES: A1-A4)

### A1 -- identify reset erases address-scoped dead state
Verified `reset_peer_backoff` only affects entries matching
`state.peer_id == Some(peer_id)` and only when `state.is_dead || state.attempt_count > 0`.
The residual behavior (clearing dead marks for stale addresses of a peer that is live
on another path) is now explicitly documented as deliberate. The failure ladder bounds
the cost and re-deads unreachable addresses. No blocking defect.

### A2 -- connected-peer guard not centralized
Verified `record_dial_failure` has exactly two production call sites in the delta,
both in the no-live-path branches of `peer_has_live_path` checks in swarm.rs
(swarm.rs:3467 and swarm.rs:5918). `record_permanent_failure` has no production callers.
The requested centralization is future hardening, not a currently reachable defect.

### A3 -- 60s revive resets strikes, no anti-hammer budget
Verified `is_eligible` and `maybe_revive` share `DEAD_REVIVE_AFTER`. Revived entries
are documented as starting with zero strikes, but immediate failure re-applies the
1s/2s/4s ladder and the third strike re-marks the address dead. The worst-case dial
pressure is bounded and documented. Accepted as a design/risk tradeoff.

### A4 -- weak branch does not prove decay
Verified the weak branch now emits an explicit `[WARN] resume_prefetch decay test`
marker identifying that decay was not exercised. The strict branch remains available
on hosts with sufficient uptime. Residual test-coverage observation, not a behavioral
defect in the reviewed delta.

### A5 -- wasm
No wasm-specific behavior; native-only fix. No new wasm concerns.

## Final verdict

Verdict: APPROVE
