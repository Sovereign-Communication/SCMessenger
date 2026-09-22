# D5/D6 — Bootstrap peers env-only: persist `bootstrap_nodes` in config, fix relay DHT bootstrap

Status: Todo
Priority: LOW-MEDIUM — not currently harmful (seed candidates arrive via the
ledger), but any code path reading the persisted config sees zero bootstrap
candidates, and the always-on node's Kademlia reports it has no peers.
Found by: OpenClaw dogfood session, 2026-09-22 (SCM_NODES_AUDIT.md section 2, D5/D6)
Node model note: every node can carry bootstrap peers; "relay" here is the
always-on node's custody behavior, not a node type.

## Defect, with evidence obtained by running commands

- OpenClaw node unit carries
  `Environment=SC_BOOTSTRAP_NODES=/ip4/18.234.62.247/tcp/9001/p2p/12D3KooWGvC...`,
  but `~/.config/scmessenger/config.json` on that node has
  `"bootstrap_nodes": []`.
- Always-on node slice: 10x `libp2p_kad::behaviour: Failed to trigger
  bootstrap: No known peers.` plus outbound dial failures back to the peer
  that dials it fine (asymmetric reachability).

## Fix shape (config-first, no rebuild required for part 1)

1. **Ops (config-only):** write the always-on node's multiaddr into
   `bootstrap_nodes` in `config.json` on both cloud nodes.
2. **Code:** persist discovered seed peers into `bootstrap_nodes` (or the
   ledger's durable equivalent) so a config-reading path has candidates;
   reconcile env vs config precedence explicitly.
3. **Code (optional):** Kademlia bootstrap mode/config on nodes with
   persisted candidates.

## Acceptance criteria

1. On a node configured per part 1, `scm config list` shows the bootstrap
   candidates and `kad` bootstrap stops failing with "No known peers" in a
   two-node test.
2. Test: config-persisted candidates survive a restart and are used by the
   seed sweep when the env var is absent.

## Gates

`core/src/transport/bootstrap.rs` if touched — Rule-8 review.

## References

- OC audit `~/Documents/GitHub/OC/SCM_NODES_AUDIT.md` section 2 (D5, D6).
- Interacts with D2 (seed-dial re-dial policy): D2 decides WHEN to dial,
  this decides WITH WHAT candidate list.
