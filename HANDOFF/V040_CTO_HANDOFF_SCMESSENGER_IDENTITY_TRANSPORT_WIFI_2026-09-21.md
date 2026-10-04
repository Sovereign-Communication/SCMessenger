# CTO Handoff — SCMessenger identity / transport / WiFi delivery (2026-09-21)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**To:** CTO / SCMessenger implementation lanes
**Date:** 2026-09-21
**Repo:** `C:\Users\SCM\Documents\GitHub\SCMessenger`
**Scope:** SCMessenger only — identity, transport, 3-node WiFi delivery

---

## Findings (SCMessenger product)

### Operator symptom

Messages are **not delivered even on WiFi** across the 3-node set (Windows + Android + AWS/Ubuntu). **Identity/transport mismatch** remains end-to-end.

### Root-cause shape (from OPEN tickets in this repo — implement, do not re-theorize)

| Finding | Ticket |
|---|---|
| Inbound `sender_id` is unauthenticated payload text; delegate UI/notification can be spoofed | `todo/P1_CORE_IDENTITY_SPOOF_AND_WASM_TOPIC_PARITY_2026-09-16.md` (CRYPTO-01) |
| WASM swarm never subscribes to own peer topic → **100% inbound loss** on web/WASM | same ticket (TRN-03) |
| CLI `send` parses contact identity as base58 peer id after store canonicalised to **public-key hex** → enqueue then hard error | `todo/P1_CLI_SEND_CANONICAL_IDENTIFIER_PARITY_2026-09-16.md` |
| Contact recovery can write peer id vs public key in the wrong field | `todo/P1_CONTACT_RECOVERY_WRITES_PEERID_AS_PUBLIC_KEY_2026-08-10.md` |
| Routing engine never learns peers → no next hop | `todo/P1_ROUTING_ENGINE_NEVER_LEARNS_PEERS_2026-08-10.md` |
| Non-swarm transports (BLE/WiFiAware/WiFiDirect) connect without feeding `routing_peer_seen` | `todo/P2_NON_SWARM_TRANSPORT_ROUTING_FEED.md` |
| Delivery receipts do not converge live | `todo/P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_LIVE_RCA_2026-08-25.md` |
| Crypto send fail vs claimed delivered | `todo/P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md` |
| Own-topic / ghost-guard message loss | `todo/P1_GHOST_GUARD_OWN_TOPIC_MESSAGE_LOSS_2026-09-16.md` |
| Windows node can wedge silently mid-run | `todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md` |

**Net:** WiFi can “connect” while messages still fail because (1) identity format is not one flavor, (2) some transports never enter routing, (3) inbound identity/own-topic paths drop or mislabel messages.

---

## Dispatch

| File | Priority |
|---|---|
| `HANDOFF/todo/P0_SCMESSENGER_WIFI_DELIVERY_IDENTITY_TRANSPORT_CANONICAL_2026-09-21.md` | **P0 umbrella — start here** |

Implement the listed OPEN tickets in the umbrella work packages; do not open parallel plans.

---

## Acceptance (SCMessenger “WiFi fixed”)

1. **One** identity flavor (public-key hex) on all nodes after restart
2. CLI send by hex **and** by contact name both succeed (no post-enqueue parse error)
3. Own-topic subscribed on native **and** WASM
4. Inbound delegate identity is **authenticated**, not payload `sender_id`
5. **Every** data-link transport feeds `routing_peer_seen` (same entry as swarm)
6. WiFi A↔B↔C delivered; receipts converge on all three nodes
7. 3-node logs for a failed send window attached on the P0 umbrella ticket

---

## Evidence still required

Collect Windows + Android (logcat / app export) + AWS/Ubuntu node logs for a **failed WiFi send window**. Attach paths on the P0 umbrella. If a node has no collector, mark **blocked** with the exact missing command — do not invent logs.

---

*End of SCMessenger CTO handoff.*
