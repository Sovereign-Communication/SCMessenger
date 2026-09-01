# V040-T14 -- The node advertises an ephemeral NAT source port as its external address

Status: OPEN (filed 2026-09-01, observed live)
Priority: **P0 -- this breaks inbound reachability for every peer**
Lane: Freebuff / DeepSeek V4 Flash
Scope: the external-address confirmation path in `core/src/transport/`.
**Rule-8 applies.**

## Observed live, not inferred

The Windows node, `GET /api/diagnostics`:

```json
"external_addrs": ["147.81.41.188:7196", "192.168.0.121:9001"]
```

`147.81.41.188` is its public IP. **`7196` is an ephemeral source port**, not a
listener. Its actual listen port is `9001` (visible on the LAN address).

The consequence, from the AWS node's log the moment T1's seed dial went live:

```
[SEED-DIAL] sweep 2: 49 candidate(s), peers=0 -- dial outcome:
  Failed to negotiate transport protocol(s):
  [(/ip4/147.81.41.188/tcp/7196: Connection refused (os error 111))]
```

The AWS node learned that address, dialled it, and was refused -- because nothing
is listening there and the NAT mapping for that outbound flow is gone.

## Why this is P0

The node is telling the entire mesh to reach it at a port where it does not
listen. Every peer that learns this address wastes dial budget on it, and any
peer relying on it for inbound cannot connect. This plausibly underlies much of
the mesh fragility attributed to other causes: **the ledger, the seed dial and
the routing engine can all be perfectly correct and still fail, because the
address being distributed is wrong at the source.**

It also poisons everything downstream. `record_connection` marks an address
`locally_verified` when we dial it successfully -- but an address we *advertise*
enters other nodes' ledgers as hearsay and, under `record_identified_peer`,
historically as verified. So one node's bad self-report propagates.

## Root cause to confirm

libp2p confirms external addresses from `identify`'s `observed_addr` and AutoNAT.
The observed address of an **outbound** connection carries our NAT-mapped
*source* port, which is not dialable unless the NAT is full-cone and the mapping
is still open. Accepting it as a confirmed external address is the defect.

Determine precisely where this node promotes an observed address to
`external_addresses()`, and whether it distinguishes:

- an observed address whose port matches one of **our own listen ports**
  (legitimate -- this is the NAT reflection case we want), from
- an observed address carrying an arbitrary ephemeral source port
  (never dialable; must be discarded).

## Required change

Only confirm an external address when its **port matches a port we actually
listen on**. An observed address with any other port is evidence about our NAT,
not about our reachability, and must never enter `external_addresses()` or be
advertised.

If libp2p's own AutoNAT/identify machinery is doing the promotion, gate at the
point where we read `external_addresses()` for advertisement rather than
fighting the behaviour internally -- but say which you chose and why.

## Acceptance

1. A test proving an observed address whose port is not a local listen port is
   rejected, and one whose port matches is accepted.
2. Live: after deploy, `GET /api/diagnostics` on the Windows node shows **no**
   ephemeral-port entry in `external_addrs`.
3. Live: the AWS node's `[SEED-DIAL]` sweeps stop attempting the ephemeral port.
4. `cargo test --workspace --no-run`, `cargo fmt --check`, clippy `-D warnings`.
   Never read `$?` after a pipe.

## Related, do not conflate

- **T13/F4** notes the ephemeral-port *filter* misses ranges. That is about what
  we accept from others. **This ticket is about what we say about ourselves**,
  which is the upstream half of the same problem. Both are needed.
- Port `7196` sits below the Linux ephemeral range (32768-60999), so a filter
  written against that range would not catch it. Whatever range logic lands must
  be justified against observed data, not against a documented default.

## Rules

- No emojis. `[OK]`, `[FAIL]`, `[WARNING]`, `[INFO]`.
- Rule-8 required; no self-certification.
- Shared checkout: touch only what this task requires.
