# D2 — No durable reconnect: seed sweep never re-dials a missing seed-tier peer

Status: Todo
Priority: HIGH — the always-on node's entire purpose is a constant custody
target; today a bridge between nodes dies until a human restarts a service.
Found by: OpenClaw dogfood session, 2026-09-22 (SCM_NODES_AUDIT.md section 2, D2)
Node model note: applies to every node; "relay" below is the custody/forward
behavior, not a node type (see `docs/rules/NODE_MODEL.md`).

## Defect, with evidence obtained by running commands

- `[DIAL-BACKOFF] Peer marked as dead after 3 failed attempts` — 15x against
  the node's own peer id, 4x against the always-on node, 2x D6vZ, 1x Dx in
  90 minutes.
- `cli/src/seed_dial.rs::sweep_decision()` returns `WatchOnly` whenever
  `peer_count > 0`, so once ANY peers are attached, the seed sweep never
  re-dials a missing seed-tier peer. The 2026-09-22 recovery required a
  manual `systemctl --user restart scm-node` (boot log: `peers=0` ->
  `[SEED-DIAL] sweep 1: 62 candidate(s), peers=0`).
- `scm config list` exposes no reconnect/backoff/bootstrap-interval key —
  no live mitigation exists.

## Mechanism (as diagnosed by OC, verify before fixing)

Backoff state marks a failed peer dead; the sweep's decision only triggers
dials at zero peers. The missing rule: re-dial when a PROVEN seed-tier peer
is missing or marked dead, regardless of `peer_count`.

## Acceptance criteria

1. Unit test: sweep with 1 attached non-seed peer + 1 dead/marked seed peer
   schedules a re-dial of the seed peer (fails on current `WatchOnly` logic).
2. Backoff still respected (no dial storm); re-dial cadence and cap decided
   and documented in the fix.
3. No config-key addition required unless the fix needs one; if it does, it
   lands in `scm config list` output with a test.

## Gates

`cli/` scope (no core gate by file path), but the dial-policy interaction is
transport-adjacent — reviewer's call whether Rule-8 applies; default to
getting the review.

## References

- Live confirmation 2026-09-22T09:05:10Z: the Windows node logged
  `[DIAL-BACKOFF] Peer marked as dead after 3 failed attempts` against a
  peer id (`12D3KooWMFSh...`) absent from the OC audit's counts — the
  mechanism is continuous, not a one-day artifact. Independent JEV sort
  of the live line landed it in `backoff` (keyed, jev-1.13.0); record:
  `HANDOFF/harness/JEV_DOGFOOD_RUN_2026-09-22.md` extension 2.
- OC audit `~/Documents/GitHub/OC/SCM_NODES_AUDIT.md` section 2 (D2).
- D3's self-dial loop interacts with this backoff state; fix ordering with
  D3 in mind.
