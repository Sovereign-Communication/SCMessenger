# D9 — libp2p 0.48 either-handler task panic on connect/identify

Status: Todo
Priority: HIGH — task-level crash on the connect path of a live mesh; the
process survives, but it fires exactly when a peer attaches.
Found by: OpenClaw dogfood session, 2026-09-22 (SCM_NODES_AUDIT.md section 9)
Filed: 2026-09-22, Buffy (Freebuff lane)
Node model note: affects any node doing connect/identify work — all nodes do
this; no role involved (see `docs/rules/NODE_MODEL.md`).

## Defect, with evidence obtained by running commands

Both cloud nodes log, around peer connect / identify / relay-reservation
activity:

```
thread 'tokio-rt-worker' panicked ... libp2p-swarm-0.48.0/src/handler/either.rs:110
```

- 12 occurrences across the journal, earliest 2026-09-21T21:01:17Z.
- Same panic location in the PREVIOUS build (paths under
  `/usr/local/cargo/...`), so it predates V040-T-CONN-04 — pre-existing, not
  a regression from the per-peer-cap fix.
- Timing correlates with peer attach: fired at 02:56:11Z (the moment the
  phone attached) and 05:53:08Z on the OpenClaw node after its swap.
- Task-level only: `NRestarts=0`, API and mesh stay up. The panic kills one
  tokio worker task, not the process.

## Scope anchor

`libp2p-swarm 0.48.0` is pinned in `Cargo.lock`. The composition in
`core/src/transport/behaviour.rs` (derive `NetworkBehaviour`, line 33; fields
at lines 65+) stacks `relay, dcutr, identify, ping, kad, gossipsub, autonat,
connection_limits, request_response` — the generated `Either` nesting is what
owns `handler/either.rs:110`. The panicking arm is an `unreachable!()` that
an event-ordering corner across this composition can genuinely produce.

## Suspected mechanism (hypothesis to verify first, not a claim)

The arm at `either.rs:110` is reached when one half of a composed handler is
asked for an inbound/outbound transition the other half believes impossible —
classically a connection that is mid-upgrade or mid-closure while a second
behaviour (relay `dcutr` over `identify`-driven reservation refresh is the
prime suspect here) still holds queued events for it. A connection closed in
the same poll cycle that a relay reservation or identify push arrives on is
the ordering the logs' timing fits.

## Acceptance criteria

1. Reproduce locally or in CI: a test that attaches/attaches-then-closes
   peers at high frequency and fails on the panic (the OC session's
   `cli/src/bin/conn-fanout.rs` harness is a starting point for induced
   connect bursts).
2. Root cause identified with file:line into the vendored
   `libp2p-swarm-0.48.0` source (verify what actually sits at either.rs:110
   in the pinned version before trusting this hypothesis).
3. Fix is one of: handler-composition reordering, event de-duplication at
   the swarm loop, or an upstream patch/upgrade path — decided by which the
   repro actually indicts.
4. A regression test pins the ordering.

## Gates

`core/src/transport/` — Rule-8 adversarial review mandatory if the fix
touches behaviour composition or the swarm event loop.

## References

- OC audit (local, outside repo):
  `~/Documents/GitHub/OC/SCM_NODES_AUDIT.md` section 9 (evidence) and
  `NODE.md` (deployment context).
- Related-but-separate: D1 fix V040-T-CONN-04 (committed on this branch),
  D2 seed-dial re-dial, D3 self-addressed poison loop — the other OC
  findings still needing upstream tickets.
