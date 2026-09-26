# A-09: Security Design -- Relay-Discovery Auth + Off-Loop Discovery Queue

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Tier: [THINK -> CODER]
Provider: qwen
Scope: v0.4.0 blocker
Language: Rust
Review: crypto-security-auditor MANDATORY after implementation

## Context

See HANDOFF/todo/A-09_RELAY_DISCOVERY_DIAL_AMPLIFICATION.md for full background.
Items 1 (connection_limits) and 3 (dial dedup) are straightforward.
Items 2 (relay-discovery authentication) and 5 (off-loop discovery queue) needed design.
Item 4 (64 KiB size guard in RelayMessage::from_bytes) is DONE.

## Item 1: Install connection_limits::Behaviour

File: core/src/transport/behaviour.rs

Add libp2p::connection_limits::Behaviour to IronCoreBehaviour. Set:
- max_pending_outgoing: 32 (matches MAX_DISCOVERY_DIALS)
- max_established_outgoing: 128
- max_established_incoming: 64
- max_established_per_peer: 4

Check libp2p version in core/Cargo.toml. Use the with_connection_limits() builder API
if libp2p >= 0.52, otherwise compose it into the behaviour macro.

## Item 2: Authenticate relay-discovery messages

In core/src/transport/behaviour.rs, before acting on any PeerJoined/PeerListResponse
discovery payload, check if the sending peer_id is in the known-relay set (peers with
an established relay reservation or handshake). If not, log at DEBUG and discard without
dialing any address. Reuse existing relay connection tracking state -- do NOT invent new fields.
Do NOT require per-message Ed25519 signatures (PQC-11 scope).

## Item 3: Dedup dial targets

Before calling swarm.dial() on discovered addresses, check:
1. swarm.connected_peers().any(|p| p == target_peer_id) -- skip if already connected
2. A HashSet<String> of already-dialed address keys within the current event -- skip duplicates.
Reset the HashSet per discovery event, not globally.

## Item 5: Off-loop discovery queue

Move discovery dials into a tokio::sync::mpsc::Sender<(PeerId, Multiaddr)> bounded channel
(capacity 64). A drain task processes at max 4 dials/second (250ms interval).
In the event handler, use try_send() -- drops if full (intentional backpressure).
Spawn the drain task at swarm startup.

## Files to Edit

- core/src/transport/behaviour.rs (primary)
- core/src/transport/swarm.rs (drain task spawn if needed)
- core/Cargo.toml (only if new libp2p feature flag needed)

## Acceptance Criteria

1. cargo check --workspace passes
2. cargo clippy --workspace -- -D warnings passes
3. Hostile PeerListResponse with 1000 addresses from unauthenticated peer: 0 dials
4. PeerListResponse from known relay with 100 addresses: <= 32 dials, off-loop
5. New unit test verifying auth rejection and dedup in core/src/transport/

## Notes

- Do NOT add per-message Ed25519 signatures (PQC-11 scope)
- crypto-security-auditor review required after -- note in commit message
- Output format: full file contents, first line of each code block must be the filename
  e.g.: // core/src/transport/behaviour.rs
