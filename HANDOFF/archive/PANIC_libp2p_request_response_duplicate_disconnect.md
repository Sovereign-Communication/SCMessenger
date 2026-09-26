# PANIC: libp2p-request-response assertion on duplicate disconnect

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

Status: OPEN -- node-fatal, reproduced on the Windows CLI node
Found: 2026-08-03 during 5-node prep, by the Haiku node-driver lane
Severity: HIGH -- kills the process, loses all listeners, blocks unattended running

## The crash

    thread 'tokio-rt-worker' panicked at
      libp2p-request-response-0.29.0/src/lib.rs:678:9
      assertion `left == right` failed
        left: false
       right: true

Node-fatal. All 27 listeners lost; the node became LAN-unreachable until a
manual restart.

## Trigger, from the log immediately preceding

The SAME peer is disconnect-processed several times inside about one
millisecond, while a listener closes and dial-backoff marks it dead:

    22:05:51.228  [ERROR] Disconnected from <peer>
    22:05:51.229  Lost relay peer <peer>
    22:05:51.229  Listener ListenerId(31) closed for addresses [...]
    22:05:51.229  [WARNING] Connection failed to <addr>
    22:05:51.229  [ERROR] Disconnected from <peer>      <-- same peer again
    22:05:51.230  [ERROR] Disconnected from <peer>      <-- and again
    22:05:51.230  [DIAL-BACKOFF] Peer marked as dead after 3 failed attempts
    -> panic

Hypothesis: the connection-closed path fires more than once for a single peer
(multiple connection ids, or a listener-close and a peer-disconnect racing), and
libp2p-request-response asserts on bookkeeping it expects to be consistent.

## ROOT CAUSE CONFIRMED -- it is OURS, not upstream

    core/src/transport/swarm.rs:4803  (and :6837, the wasm arm)
    SwarmEvent::ConnectionClosed { peer_id, .. } => {

We destructure ONLY `peer_id` and discard the rest with `..`. The discarded
fields include `num_established` -- how many connections to that peer REMAIN
after this one closed.

`grep -c num_established core/src/transport/swarm.rs` returns **0**. It is never
checked anywhere.

libp2p emits ConnectionClosed PER CONNECTION. We handle it PER PEER, and on the
FIRST connection close we unconditionally tear down all peer-level state:

    connection_tracker.remove_connection(&peer_id);
    ledger_exchanged_peers.remove(&peer_id);
    reported_peer_discoveries.remove(&peer_id);
    reported_peer_info.remove(&peer_id);
    swarm.remove_listener(listener_id);        // relay reservation
    ... plus cancelling pending custody dispatches

...while OTHER connections to the same peer are still live. libp2p-request-
response then asserts on bookkeeping we removed out from under it.

This explains the observed signature exactly: three "Disconnected from <peer>"
lines for the SAME peer inside one millisecond are three CONNECTIONS closing,
not three peers.

## The fix

Guard peer-level teardown on `num_established == 0`:

    SwarmEvent::ConnectionClosed { peer_id, num_established, .. } => {
        // per-connection cleanup may run every time
        if num_established == 0 {
            // peer-level teardown ONLY when the last connection is gone
        }
    }

Both arms (:4803 and the wasm arm at :6837) need it. Anything that removes a
listener, cancels dispatches, or clears per-peer maps belongs inside the guard.

Likely also explains relay churn and reconnect thrash previously attributed to
network conditions.

## Second defect exposed by the recovery attempt

After the panic, the first restart attempt was REFUSED:

    SCMessenger is already running!
    Run scm stop to stop the existing node first.

A stale lock or PID file survived the crash, so the node could not self-recover.
A crashed process that then blocks its own restart cannot be left running
unattended, which is exactly what a multi-node test requires.

## Why it matters now

The 5-node matrix needs nodes that stay up across peer churn. This crash fires
precisely during relay peer reconnection churn, which is the normal condition
during a multi-node test. Hitting it mid-matrix would look like a messaging
failure rather than a node death.

## Suggested work

1. Determine whether the duplicate disconnect originates in our swarm event
   handling or in libp2p. Check whether we handle ConnectionClosed per
   CONNECTION or per PEER -- treating a per-connection event as per-peer is the
   classic source of this.
2. If ours: dedupe disconnect handling by connection id.
3. If upstream: check libp2p-request-response for a fixed release, or gate the
   assertion path.
4. Independently: make the lock/PID guard detect a dead PID and clear it, so a
   crashed node can restart itself.
5. Add a supervisor for test runs so a node death is visible as a node death,
   not misread as a delivery failure.

## Evidence

Full driver log: `HANDOFF/audit/windows_node_driver_log.md`.
Cycles 1-3 healthy at 27 listeners, cycle 4 degraded to 2, panic, restart to a
new PID with full listener restoration, cycles 5-8 stable.
