# P1 -- doubly-nested relay circuit addresses are still being formed and advertised

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Open
Filed: 2026-08-10 ~03:20Z (Windows lane), anchor `68fcc3f1`
Related: the GPT-MAC lane's `82b52a0a` ("relay candidates now use the identified
relay's direct addresses, never this node's own external addresses; nested and
self-targeted circuit paths are rejected") was intended to stop this. It has not.

## Evidence, observed on the current anchor

Within minutes of wiring the AWS relay into the bootstrap, the Windows node
began listening on a **doubly-nested** circuit address:

```
/ip4/54.226.67.101/tcp/9001/p2p/12D3KooWPJK6KgKsafefLWeGs4kVbj7wBnU67yKe88ni3FHZ3Hr2
  /p2p-circuit/p2p/12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688
  /p2p-circuit/p2p/12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw
```

Read it left to right: through the AWS relay, then through the iPhone, then to
**this node itself**. Two hops of circuit, terminating on the originator.

Same run, the relay peer accumulated:

```
[CIRCUIT-RELAY] Registered relay peer relay_peer_id=12D3KooWPJK6... addr_count=41
```

**41 addresses for one relay peer.**

## Why this matters more than it looks

1. **It is address bloat with a multiplier.** Each nesting level combines every
   known path with every other, so the advertised set grows combinatorially
   rather than linearly with fleet size. 41 addresses for a single peer on a
   4-node fleet is the warning sign.

2. **It is very likely the cause of the Windows mDNS failure.** `libp2p-mdns`
   dies ~200 ms after start on Windows with `WSAEMSGSIZE` (os error 10040) --
   a datagram larger than the receive buffer. Nested circuit addresses are by
   far the longest entries in the advertised set. The other lane independently
   reported `TxtRecordTooLong` exclusions "for long circuit addresses", which is
   the same root cause hitting a different limit on a different platform.
   **Two platform-specific symptoms, one shared cause.**

3. **A self-terminating circuit is never useful.** A path that routes through
   two hops back to the originator cannot deliver anything. Every dial attempt
   against it is wasted budget, and wasted concurrent dials are what feeds the
   `libp2p-request-response` P0 trigger.

## What to determine

1. Where are circuit listener addresses composed? Establish whether nesting is
   deliberate (multi-hop relay support) or accidental (a learned circuit address
   being re-wrapped as a new listener).
2. Why does the existing rejection from `82b52a0a` not catch these? Determine
   whether it checks only the FIRST `/p2p-circuit` segment rather than scanning
   the whole multiaddr, and whether it runs on the advertise path as well as the
   dial path. A filter applied only at dial time still lets bad addresses
   propagate to every peer via ledger exchange.
3. Should ANY multi-hop circuit be advertised? If single-hop is the only
   supported topology, reject depth > 1 outright rather than filtering
   case by case.

## Acceptance criteria

1. No advertised address contains more than one `/p2p-circuit` segment, unless
   multi-hop is an explicit, tested feature.
2. No advertised address terminates on this node's own PeerId, at any nesting
   depth.
3. Per-peer advertised address count stays bounded and is asserted in a test --
   41 for one peer must not be reachable again.
4. Re-check the Windows mDNS failure after the fix. If `WSAEMSGSIZE` stops, that
   confirms the shared cause and closes a second defect for free. If it does not,
   the mDNS buffer issue is independent and needs its own fix.
5. The filter must run on the ADVERTISE path, not only the dial path, so bad
   addresses never enter another node's ledger.

## Scope note

Address composition lives in `core/src/transport/`, which is merge-blocked under
rule 8 -- operator decision plus adversarial review before implementation.
