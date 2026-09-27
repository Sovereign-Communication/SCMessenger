# TASK: Fix race condition in integration_ledger_convergence.rs (FARM WS-F1)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: TODO
Target File: `core/tests/integration_ledger_convergence.rs`

## Problem Description
The `integration_ledger_convergence` test fails at runtime at:
`Node 2's ledger should contain node 1's pre-existing entry via ledger_exchange`

### Root Cause
1. In the test, Node 1 spawns a background listener task that waits for a `SwarmEvent2::PeerDiscovered` event.
2. When the nodes discover each other via mDNS, this event fires *before* the TCP connection handshake is completed and established.
3. Node 1 immediately calls `share_ledger` to send the request, but because they are not connected yet over TCP, the request fails or is silently dropped.
4. Because the listener task has a `break;` statement, it exits after the first event and never tries again once the connection actually goes live.

## Required Fix
Instead of listening for a racily-timed `PeerDiscovered` event in a spawned background task, trigger the ledger exchange directly in the main test thread **after** the dial succeeds:

1. Remove the spawned event listener task for `PeerDiscovered` in Node 1.
2. After `swarm2.dial(node1_addr.clone()).await.expect("Failed to dial");` succeeds, wait a brief moment for the connection handshake and protocols to negotiate:
   ```rust
   tokio::time::sleep(Duration::from_millis(500)).await;
   ```
3. Trigger the ledger share directly from Node 1 to Node 2 now that they are connected:
   ```rust
   let entries = ledger1.dialable_addresses();
   let shared_entries: Vec<SharedPeerEntry> = entries
       .into_iter()
       .map(|entry| SharedPeerEntry {
           multiaddr: entry.multiaddr,
           last_peer_id: entry.peer_id,
           last_seen: entry.last_seen.unwrap_or(0),
           known_topics: entry.topics,
       })
       .collect();
   swarm1.share_ledger(peer_id2, shared_entries).await.expect("Failed to share ledger");
   ```
4. Let the test wait for 3 seconds (`tokio::time::sleep(Duration::from_secs(3)).await;`) so the ledger is received on Node 2.

## Response Format Requirement
The exact filename must be the FIRST LINE inside the code block:
```rust
// core/tests/integration_ledger_convergence.rs
<full file content here>
```
