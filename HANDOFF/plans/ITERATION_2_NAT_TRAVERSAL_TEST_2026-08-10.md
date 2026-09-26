# Iteration 2 staging -- external/internal IP mapping and NAT traversal

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

Status: Staged, not started. Five-node run 1/2 is HELD pending the blockers in
Section 4.
Written: 2026-08-10 ~04:00Z
Anchor: `68fcc3f1`

## 1. The scenario (operator, 2026-08-10)

iOS leaves the LAN and goes into town on cellular. Android stays on the home
LAN/WiFi. Windows and the AWS relay stay up.

Expected behaviour, in order:

1. iOS recognises it was previously on the same LAN as Android, and that they
   therefore **share an external IP**.
2. iOS saves that external address and attempts **direct** communication to it.
3. Failing that, it reaches the **AWS relay** to obtain current connection
   information, or has the relay carry traffic so iOS and Android keep talking.

This is the first real test of external-to-internal address mapping in this
fleet, and it is a genuine product scenario rather than a lab construct.

## 2. The mechanism exists -- confirmed in code

| Piece | Status | Evidence |
|---|---|---|
| Circuit relay | working | AWS relay registered, circuit reservations forming |
| AutoNAT | enabled | `autonat` in `core/Cargo.toml` features |
| DCUtR (hole punching) | **wired** | `dcutr::Behaviour` at `core/src/transport/behaviour.rs:40, 342, 529`; used at `swarm.rs:4629` |
| UPnP port mapping | **ABSENT** | removed after the `libp2p-upnp` panic; see the UPnP deferral ticket |

So the intended chain is: **relay rendezvous -> AutoNAT establishes both peers
are NATed -> DCUtR coordinates a simultaneous open through the relay -> direct
connection; relay carries traffic if the punch fails.**

**Because UPnP is gone there is no port forwarding.** DCUtR is therefore the ONLY
route to a direct connection. If DCUtR does not work, the relay is not a
fallback -- it is the only path. That distinction decides how to read the result.

## 3. The trap: do NOT blacklist our own external IP

The home external address is `147.81.41.188`. It appeared **144 times** in one
Windows run as a *self-dial* target -- the node repeatedly dialling its own
external IP, filed under the self-dial defect.

**The obvious fix for that -- blacklisting our own external IP -- would break
exactly the mechanism this test depends on.** That same address is what a remote
iOS node must dial to reach Android behind the same NAT, and what DCUtR needs as
the hole-punch target.

The correct distinction is directional, not address-based:

- dialling our own external IP **from inside the NAT** is useless (hairpinning is
  unreliable and it is usually ourselves)
- dialling it **from outside** is the entire point

Any self-dial fix must preserve the external address for remote peers and for
DCUtR. Whoever implements the self-dial ticket must read this section first.

## 4. Blockers that must clear before this runs

1. **P0 request-response panic.** Reproduced on the anchor WITH the dial-dedup
   fix at 13m23s with six simultaneous connections to one peer. Needs a per-peer
   concurrent-connection cap; address-level dedup is insufficient. A 13-minute
   MTBF cannot support a NAT-traversal test, which needs sustained connectivity
   while a device roams.
2. **Stale address reaping.** Windows spent a whole window retrying Android at
   `.141` while it was live at `.111`. Roaming makes this worse, not better --
   iOS on cellular will renumber repeatedly.
3. **iOS must hold the relay, verified on-device.** Android is verified (4 IP
   refs, 40 PeerId refs in `files/ledger.json`). iOS is INFERRED only. Inference
   was already wrong once for Android.
4. **Mobile-to-CLI messages arrive with empty text.** If mobile message bodies do
   not survive, the test can prove connectivity but not messaging.

## 5. What to capture during the run

- iOS: the external address it records for Android, and whether it attempts a
  direct dial to it before falling back.
- Whether AutoNAT correctly classifies both peers as NATed.
- **Whether DCUtR is attempted at all**, and whether the hole punch succeeds. Log
  the attempt explicitly -- a silent skip looks identical to a failed punch.
- If direct fails: does the relay carry the traffic, and with what latency?
- Time from iOS leaving the LAN to first successful message in each direction.
- Every address iOS tries, in order. The dial ordering is as important as the
  outcome -- today's failures were mostly wrong-order, not wrong-address.

## 6. Scoring rules (carried forward, all learned the hard way)

- Score on receiver-side `inbox_receive` plus ACK. **Sender-side status is
  broken** -- the receipt loop is open (`iron_core.rs:3423-3444` never calls
  `mark_message_sent`).
- A non-200 from `/api/send` is not proof of non-delivery; a 200 is not proof of
  delivery.
- A node that died mid-window invalidates every delivery attributed to it.
- Do not use Android `dumpsys` as a readiness check -- it reports no
  ServiceRecord for a service that is demonstrably running.
- Wake the phone. A dozing device is unreachable inbound and looks identical to
  a broken node.
- Relay transport is `/ws` on 9001; LAN transport is plain TCP. Two opposite
  conventions in one fleet -- get the form right or the dial times out looking
  like a dead host.
