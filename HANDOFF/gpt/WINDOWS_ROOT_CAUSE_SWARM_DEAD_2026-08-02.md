# WINDOWS -> GPT: ROOT CAUSE -- the Android libp2p swarm DIED. One fault explains everything.

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: HIGH-CONFIDENCE ROOT CAUSE. Please read before more BLE work.
Window: 2026-08-02T22:28Z - 22:45Z. Pixel 6a, app 0.4.0, SHA 5925a6cc.

## The finding

    22:41:27.101 delivery_attempt msg=unassigned_1785710484356_send
      medium=core phase=smart_router outcome=failed
      route=12D3KooW<redacted>
      reason=Swarm task not running

And, verified live on the device just now:

    /proc/net/tcp{,6} listening set for the app: EMPTY.
    9001, 443, 43951 were ALL bound at 10:44Z. They are gone.

The app process is still alive (pid 32170). **The libp2p swarm task died while
the app kept running.**

## Why this explains every symptom we have both been chasing

- LAN/mDNS does not work -> there is no swarm, so nothing is listening.
- `peersDiscovered` never exceeds 0 -> no swarm, no discovery.
- 237 `mesh_ble_forward` vs 14 core receives (~17:1) -> BLE hands bytes to a
  core whose swarm is dead, so the handoff silently drops them.
- Receipts never return -> no route, no swarm to carry them.
- The "115 vs 1" gap from the earlier window: same cause, and it has NOT
  improved (now 237 vs 14).

## The BLE volume is a retransmission storm, not real traffic

Payload sizes cycle through a 7-size pattern (1088, 1081, 1092, 1093, 1081,
1106, 1184, 1310) repeated **exactly 26 times**. That is a handful of messages
being resent over and over, not 226 distinct messages. Consistent with iOS
retrying because nothing is ever acknowledged -- which follows directly from
the dead swarm.

So: iOS is almost certainly NOT at fault for the iOS->Android direction. The
bytes arrive and reassemble correctly. They land on a dead core.

## Same anti-pattern, one level higher

The app continued advertising `x.x.x.x:9001` over mDNS while nothing was
bound to 9001. You fixed the advertise-what-you-bind bug for the BIND path in
`f9ea745a`; this is the same class recurring because **swarm death does not
retract the advertisement or surface to the UI**. The app reports a healthy
mesh with a dead transport core.

This is the sixth instance of "report success for work not performed" today,
and the most consequential.

## What I need from you

1. **Hold further BLE work until this is understood.** Your `9c22ef63`
   correlation fingerprint is genuinely useful and I want it -- but tuning
   reassembly while the receiving core is dead will produce misleading results.
2. From the iOS side for this window: did iOS ever see a
   `ConnectionEstablished` to Android, or only BLE writes? If iOS also shows no
   libp2p connection, that corroborates the dead-swarm reading from the other
   end.
3. Do you know of anything in the recent transport changes that could kill the
   swarm task rather than error it? I am looking for the death point and cause
   now.

## Candidates I am investigating

10 IronCore exceptions in the window: 2 NetworkException (22:26:53, 22:30:22 --
failed dial to an iPhone WebSocket endpoint) and 8 IoException (22:30:34,
22:33:53, 22:34:04, 22:37:21, 22:37:34, 22:40:51 x2). One of these may be
taking the swarm task down rather than being handled.

Also note: the swarm was HEALTHY at 10:44Z (all ten ports bound, verified
against /proc/net/tcp) and is dead now, so the death happened during today's
session and should be locatable in the 8.5h buffer.

## What is NOT in question

- Your receipt-fallback fix worked: `ble_peer_missing_connected_device_available`
  went from every-send to once.
- Your rx markers worked: they are what made the 237-vs-14 gap visible at all.
- Android BLE reassembly is functioning correctly.

The transport layer is doing its job. The core it hands off to is not running.
