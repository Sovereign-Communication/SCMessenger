# P1 -- promiscuous dial sweep spends ~60% of its budget on this node itself and on unreachable carrier addresses

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: Active
Severity: P1 (blocks LAN convergence; the desktop nodes are not finding each other)
Discovered: 2026-08-09, visible only after raising `RUST_LOG` to debug
Observed on: `33c16712` (Windows node, restart at 12:24:09Z)
Related: `P0_REQUEST_RESPONSE_PANIC_KILLS_DESKTOP_ON_MESH_GROWTH_2026-08-09.md`

## Summary

On startup the node runs a promiscuous dial sweep over its ledger. In the first
~4 minutes after a restart it issued **700 dial attempts** and connected to
**nothing**. The distribution is the problem:

| Target | Dials | Share |
|---|---|---|
| **This node's own address** (`192.168.0.121`, ports 443/9004/9101/...) | **88** | 12.6% |
| **Carrier IPv6** (`2600:381:...`, not reachable from this LAN) | **331** | 47.3% |
| The macOS node (`192.168.0.136`) -- an actually reachable LAN peer | 17 | 2.4% |
| everything else | 264 | 37.7% |

**About 60% of the dial budget goes to addresses that cannot possibly succeed**,
while the one reachable LAN peer we are actively trying to reach gets 2.4%.

Zero successful connections in that window. `GET /api/peers` returned `{"peers":[]}`
roughly four minutes after restart, on a LAN with three other live nodes.

## Evidence

```
695.  Dialing 192.168.0.121:443 (promiscuous)...
700.  Dialing 192.168.0.121:9101 (promiscuous)...
696.  Dialing 2600:381:9b57:6b48:e125:d55a:4e95:7896:8080 (promiscuous)...
699.  Dialing 2600:381:9b58:2de7:8d7a:6baa:b39d:40ee:9090 (promiscuous)...
DEBUG dial_policy: [DIAL-POLICY] Peer is not eligible for dial attempt (backed off or dead) addr_key=/ip4/192.168.0.121/tcp/9004/ws
```

`192.168.0.121` is this node's own address, confirmed independently: the macOS
node's identify reported `Identify observed address via 12D3KooWNC5r...:
192.168.0.121:9001`.

The `2600:381:...` addresses are the Pixel's carrier IPv6 interfaces, harvested
into the ledger from identify. They are globally scoped and therefore look
routable to the address filter, but they are not reachable from this LAN.

## Two distinct defects

**A -- self-exclusion is not applied on the promiscuous path.** Review F14 added
self-exclusion for seed dialing: `ConnectToSeedPeers` builds `my_addrs` from
`swarm.listeners()` and `swarm.external_addresses()` and passes it to
`build_seed_dial_candidates` (`core/src/transport/swarm.rs:5922`). The
promiscuous sweep does not appear to apply the same filter -- 88 self-dials in
one sweep. Whatever list feeds the sweep needs the same exclusion.

**B -- unreachable carrier addresses are dialled at full weight.** A peer's
cellular IPv6 is a legitimate ledger entry (the peer really is reachable there,
from the internet) but it is near-useless from a LAN peer behind NAT, and it
dominates the sweep 331-to-17 over the LAN address of the same fleet. Dial
ordering should prefer same-subnet/private candidates before global ones, or
carrier-scope entries should be de-prioritised when a same-subnet candidate for
the same peer exists.

## Why this matters now

This is the most likely explanation for symptoms currently blocking the
five-node run:

- **Windows and macOS are not connecting** despite both running on the same LAN.
  The macOS node gets 2.4% of the dial budget.
- **Convergence is slow when it does happen.** An earlier run took 5m39s to
  converge with the Android peer, arriving seconds before an unrelated crash.
- Android reports `reason=no_route_candidates` on every outbox retry, consistent
  with a mesh that is not converging rather than with a messaging defect.

## Note on visibility

**None of this is visible at the default `info` level.** It only appeared after
starting the node with:

```
RUST_LOG="info,scmessenger_core::transport=debug,scmessenger_core::store::outbox=debug,scmessenger_core::store::inbox=debug,scmessenger_core::relay=debug,scmessenger_cli=debug"
```

That is a strong argument for running the five-node test at this level on every
desktop node, as the operator directed.

## Acceptance criteria

1. A node never dials its own listen or external addresses. Regression test with
   a ledger seeded with the node's own address.
2. Same-subnet/private candidates are attempted before global candidates for the
   same peer, or carrier-scope entries are de-prioritised when a same-subnet
   candidate exists.
3. On a LAN with three live peers, a restarted node connects to all three within
   a defined budget (suggest 60 seconds) rather than exhausting hundreds of dials
   on unreachable targets.
4. Report the dial-target distribution in the fleet run evidence, so a regression
   here is visible rather than showing up as "slow convergence".
