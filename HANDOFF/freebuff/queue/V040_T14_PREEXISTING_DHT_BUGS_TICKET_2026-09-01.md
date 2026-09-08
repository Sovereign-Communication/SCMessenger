# V040-T14 -- Two pre-existing DHT bugs (found by Rule-8 review of #267)

Status: PR FILED -- #269 open (NOT in PR #267 -- CEO ruling: "do not fold pre-existing bugs into this PR"); awaiting adversarial review
Source: RULE8_PR267_VERDICT.md [FAIL] F-3, confirmed in RULING_2026-09-01_PR267_REJECTED_my_ruling_was_wrong.md
Filed: 2026-09-01

Two `kademlia.add_address` feeds on `main` are ungated and pre-date the F-DHT
work. Both are separate from #267's gate rework and need their own PR.

## Bug 1 -- mDNS `Discovered` feed (swarm.rs ~5009)

`(peer_id, addr)` come straight from an **unauthenticated LAN multicast**. A
hostile device on the same Wi-Fi can announce any peer id at any **public**
address (or RFC1918 -- `is_discoverable_multiaddr` deliberately permits both)
and have it inserted into our Kademlia table and re-published globally.

This is exactly the hearsay the F-DHT gate was written to keep out; the mDNS
feed predates the gate and was never given one. mDNS is a LAN-local trust
domain, so the fix is a judgment call: either gate it like the other feeds
(require a proven pair) or explicitly document mDNS's LAN trust boundary as an
accepted exception. The rule-8 reviewer flagged it; the F-DHT PR leaves it
byte-identical per the CEO's "ticket separately" direction.

## Bug 2 -- DCUtR hole-punch inserts OUR OWN addresses under the REMOTE peer (swarm.rs ~4885)

The comment says "Add this peer's direct addresses", but the code collects
`swarm.external_addresses()` -- the **local** node's external addresses -- and
inserts them under `remote_peer_id`. This publishes our own addresses into the
DHT as the remote peer's. Separate from the gate question, this is an outright
correctness bug (identity misattribution + leaks our addresses bound to a
stranger's identity).

Fix shape: the hole-punched DIRECT address for `remote_peer_id` is the
remote's observed address from the DCUtR event / connection, not our external
addresses. Needs the actual remote endpoint captured at the successful
hole-punch.

## Acceptance for the follow-up PR

1. mDNS: either gated behind the same proven-pair predicate or an explicit
   documented exception (decision needed from the CEO seat / reviewer).
2. DCUtR: insert the remote's direct address under `remote_peer_id`, never
   `swarm.external_addresses()`.
3. Both: full gate set (fmt, workspace clippy, workspace no-run, core + cli
   tests, wasm32 check), Rule-8 adversarial review (touches
   `core/src/transport/`).
