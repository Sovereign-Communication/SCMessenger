# PR #274 review record -- dial dispatch self/connected guards

- PR: https://github.com/Sovereign-Communication/SCMessenger/pull/274
- Branch: freebuff/v040-nimble-peer (base cto/v040-candidate-2026-09-02)
- Reviewer: qwen3.8-2.4t-a95b (ledger-confirmed 100%/1M, free lane)
- Author of the change: freebuff lane (does not satisfy Rule-8 alone)
- Gate: Rule-8 (transport dispatch) -- requires non-author APPROVE

## Rounds

| Round | Head | Verdict |
|---|---|---|
| R1 (2026-09-03) | 1075af8c | REQUEST_CHANGES (A1-A10) |
| R2 (2026-09-03) | d00a2296 | REQUEST_CHANGES (C1-C8) |
| R3 (2026-09-03) | 2f9c9c07 | REQUEST_CHANGES (E1-E9) |
| R4 (2026-09-03) | 6764e2b0 | **APPROVE** (F1-F3 INFO/LOW, no required fix) |

Raw verdict files: tmp/V040_NIMBLE_RECYCLE_REVIEW_{R1..R4}_2026-09-03_response.md
Task briefs: tmp/V040_NIMBLE_RECYCLE_REVIEW_{,_R2,_R3,_R4}_TASK_2026-09-03.md
Diffs per round: tmp/pr274.diff, tmp/pr274-round{2,3,4}.diff

## Disposition summary

Code fixes across rounds (all on PR #274, 5 files: core swarm.rs + Cargo.toml
+ Cargo.lock, cli ledger.rs + main.rs):
- single dispatch owner `dial_skip_reason` in both SwarmCommand::Dial arms:
  self peer id, connected peer, own-socket address skips;
- own-socket predicate `OwnSockets` (normalized IPs, proto-aware TCP/UDP
  bound ports from listeners only, interface IPs via if-addrs native-only,
  /p2p-circuit forms excluded, trusted dials exempt);
- seed-path connected + self gates with one snapshot per command;
- skipped dials reply Err("skipped: ...") and the CLI ledger releases
  claims neutrally (complete_dial_skipped) -- no phantom connections, no
  stale-address reaps, no backoff burn on deliberately-skipped dials.

Tests added: addr_targets_self (loopback, interface-IP, concrete-own, QUIC,
mapped-IPv6, circuit-hop, proto-separation, negative classes);
test_complete_dial_skipped_releases_claims_neutrally.

## Verdict

APPROVE at head 6764e2b0. Rule-8 gate CLOSED.
