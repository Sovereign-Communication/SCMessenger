# TASK: Graceful-AF Dial Policy — self-dial prevention + network-aware RFC1918 filtering + backoff

Status: READY FOR DELEGATION (Qwen THINK → CODER)
Source: Direct observation 2026-07-20 (Lucas CLI connected to alpha relay via
ss verified; then proceeded to promiscuously dial its own LAN IP, emulator
10.0.2.x junk from ledger, and all 48 ledger entries with 5s backoff each)

## Problem

After the initial relay-connection ledger exchange, the CLI received 48 peer
addresses via the ledger and began dialing ALL of them with ~5s backoff in a
"promiscuous" loop. This included:
- Its own LAN IP (x.x.x.x) — self-dial, wastes resources
- Emulator-internal 10.0.2.x addresses (RFC1918 private, but the CLI is on a
  public internet network, not on x.x.x.x/8)
- Other private/internal addresses that cannot route from this machine

## Required Changes

### 1. Self-dial prevention
Track the local node's own known addresses (listen addrs + external addrs).
Before dialing any address, check if it resolves to an address this node
already holds. Skip if yes.

Location: `core/src/transport/swarm.rs` (where dial is initiated) or
`cli/src/ledger.rs` (where dial targets are enumerated). Ledger-side is
preferred since the CLI ledger dial loop is the promiscuous source.

### 2. Private-range awareness
The existing `is_dialable_multiaddr` filter (commit 36635cb0) rejects
loopback/link-local/site-local. It should also reject RFC1918 private
addresses (x.x.x.x/8, x.x.x.x/12, x.x.x.x/16) UNLESS the local
node itself is on that private network.

How to determine "my network": compare each private-range address against
the node's own listen/external addresses. If ANY listen address is in the
same private range, allow dialing that range. Otherwise, reject.

Location: `cli/src/ledger.rs` is_dialable_multiaddr or the caller.

### 3. Backoff discipline
The promiscuous dial loop uses 5s backoff per address but chains them
sequentially with no per-peer cap. Add:
- Per-peer backoff state (dedup by peer ID, not address)
- At most 3 concurrent outbound dials to unknown peers
- Exponential backoff per peer: 5s, 30s, 120s, 5min, 30min
- After the initial relay connection, prefer circuit-relay routing over
  direct promiscuous dials to unknown addresses

### 4. Success-based pruning
If the relay circuit is working (circuit-reservation granted, relay forwarding
established), use it for message delivery. Only fall back to direct dials
when a message needs to reach a specific peer that doesn't respond via relay.

## Files to Modify

- `cli/src/ledger.rs` — add self-dial check + private-range-awareness + backoff
- Possibly `cli/src/main.rs` — pass own-address list into the ledger filter

## Verification

1. Start CLI, connect to relay, observe ledger exchange → should NOT dial
   own IP or junk private addresses
2. Should dial the relay only (bootstrap seed) and learn peers via ledger
   without promiscuous looping
3. Relay circuit should be used for routing instead of direct dials

## Security Review

Mandatory crypto-security-auditor (transport/ routing concerns: modified dial
behavior affects connection-layer security properties).

## PROGRESS (2026-07-20)

Items 1 (self-dial prevention) and 2 (private-range network-awareness)
implemented directly in `cli/src/ledger.rs` (`is_self_address`,
`is_dialable_for_this_node`) and wired into all 5 raw-dial call sites in
`cli/src/main.rs`. Verified: `cargo check`/`clippy -D warnings`/`fmt --check`
clean, `cargo test --workspace --no-run` clean, 12 unit tests pass
(10 pre-existing + 2 new, later expanded to cover more edge cases).

Mandatory crypto-security-auditor review (connection-layer/routing concern,
per this ticket's own request) found a real MEDIUM-HIGH bug before merge:
`is_dialable_for_this_node` didn't carry the `/p2p-circuit` unconditional-
allow exemption its sibling `is_dialable_multiaddr` has, so a relay-circuit
address whose RELAY HOP happened to be in a different RFC1918 class than
this node's own address would be silently rejected -- breaking the primary
NAT-traversal path this project relies on. Fixed: circuit addresses now
short-circuit past the RFC1918 check entirely (self-dial check still
applies). Added a regression test using this project's own test-fixture
circuit-address shape (`core/src/transport/swarm.rs`'s
`.../p2p-circuit/p2p/...` pattern). Also caught and fixed a bug in my OWN
added test during verification (a wrong assertion about self-dial detection
on circuit addresses -- the exact-string-match in `is_self_address` doesn't
treat a circuit address as "self" just because the relay hop's IP matches,
since the `/p2p-circuit` suffix makes the stripped strings differ; confirmed
this is correct behavior, not a gap, and corrected the test's expectation).

Auditor also flagged (both explicitly NOT fixed here, deferred to items 3+4
below, but recorded so they aren't mistaken for closed):
- Items 1+2 narrow WHICH addresses can be dialed but do not bound HOW MANY
  concurrent dials one ledger-exchange burst can trigger -- a malicious/
  compromised peer sending many filter-passing addresses in one
  `LedgerReceived` event still causes a burst of concurrent `tokio::spawn`
  dials (a third-party-hammering/resource-exhaustion vector). This is
  exactly item 3 below; shipping 1+2 alone does not close it.
- IPv6 Unique Local Addresses (`fc00::/7`) aren't covered by the private-
  range-awareness logic (IPv4-only) -- a pre-existing gap in
  `is_dialable_multiaddr` too, not a regression, but worth a follow-up.
- `get_bound_addresses()` returns addresses populated only from successful
  `NewListenAddr` events -- a node whose listeners all fail to bind (but is
  still outbound-reachable on its LAN) would have permanently-empty
  `my_addrs`, meaning it would NEVER dial any private-range peer for the
  life of the process, with no diagnostic. Rare edge case, not fixed here.

Items 3 (per-peer backoff + concurrent-dial cap) and 4 (prefer relay-circuit
over promiscuous direct dial) still need larger dial-loop restructuring --
not attempted this session.
