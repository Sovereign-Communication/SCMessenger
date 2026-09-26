# A-09: Relay-discovery dial amplification + unauthenticated peer injection

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: OPEN -- HIGH (DoS) -- pre-existing, partially mitigated 2026-07-17.
Additional partial mitigation 2026-07-19 (commit 36635cb0, mislabeled "(A-09) [OK]"
in its commit message -- it is NOT closed): added `is_dialable_multiaddr` filter to
`cli/src/ledger.rs` rejecting loopback/link-local/site-local addresses in the CLI's
own ledger dial loop. This fixed a real, separate bug (the relay was dialing
non-routable addresses peers advertised via ledger-sharing, storming its own
request_response handler and blocking legitimate peers) but does NOT touch
`core/src/transport/behaviour.rs` connection_limits, relay-discovery
authentication, dial dedup, or the `RelayMessage::from_bytes` input-size guard
listed below. Ticket remains OPEN pending the actual "Remaining work" items and
the mandatory crypto-security-auditor pass.
Tier: THINK (design) -> CODER (impl)
Review: crypto-security-auditor MANDATORY (touches core/src/transport/, routing)
Source: adversarial review of the Hermes transport port (2026-07-17), findings 1, 2, 4, 7

## Problem

The `/sc/message` request handler acts on unauthenticated `RelayMessage`
peer-discovery payloads (`PeerJoined`/`PeerListResponse`) by dialing every
attacker-supplied address. `is_discoverable_multiaddr` accepts any public IP.
DriftFrame's CRC32 is not authentication -- any peer can forge a frame whose
payload deserializes as a relay-discovery message. Two DoS vectors:

1. **Dial amplification / address poisoning** (HIGH): one request induces our
   node to dial arbitrary third-party hosts -- reflected connection flood plus
   exhaustion of our own pending-connection / FD budget. No
   `libp2p::connection_limits::Behaviour` is installed anywhere in
   `IronCoreBehaviour`.
2. **Synchronous event-loop stall** (HIGH): the parse+dial loop runs inline in
   the single swarm `tokio::select!` loop that serves all peers/protocols; a
   large list head-of-line-blocks everything.

## Mitigation already applied (2026-07-17, commit porting Hermes fixes)

- Relay probe only runs on payloads <= 64 KiB (`RELAY_CONTROL_MAX_BYTES`).
- Total dials per message capped at 32 (`MAX_DISCOVERY_DIALS`), counted across
  the whole peer list, not per-peer.
- Blocked-sender check moved to the top of the Request arm (a blocked peer can
  no longer drive discovery dialing).

These bound the blast radius but do not close the class.

## Remaining work (this ticket)

1. Install `libp2p::connection_limits::Behaviour` with a max-pending-outbound
   ceiling in `IronCoreBehaviour` (`core/src/transport/behaviour.rs`).
2. Authenticate relay-discovery: only act on `PeerJoined`/`PeerListResponse`
   from peers that completed the relay handshake / carry a verifiable signature,
   rather than any peer on `/sc/message`.
3. Dedup dial targets against already-known/already-dialed peers before dialing.
4. Add an explicit input-size guard inside `RelayMessage::from_bytes`
   (`core/src/relay/protocol.rs`) independent of the transport ceiling.
5. Consider moving discovery dialing off the hot event-loop into a bounded
   queue drained across iterations.

## Acceptance

- Hostile `PeerListResponse` with 10k addresses results in <= cap dials, no
  event-loop stall (add an integration/property test in
  `core/tests/`), and unauthenticated relay-discovery is rejected.
- crypto-security-auditor PASS.
