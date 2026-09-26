# P0 — SCMessenger WiFi delivery + canonical identity/transport (umbrella)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** OPEN — CTO dispatch; **implementation authority** is
`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md`
(code-verified matrix + WP file anchors + harness/JEV completion gates)
**Priority:** P0 (messages not delivered even on WiFi; identity/transport mismatch)
**Filed:** 2026-09-21
**Authority:** `HANDOFF/V040_CTO_HANDOFF_SCMESSENGER_IDENTITY_TRANSPORT_WIFI_2026-09-21.md`
+ implementation plan above
**Repo:** SCMessenger only

## Dispatch files (paste after they are on origin/main)

| WP | Freebuff ticket |
|---|---|
| WP1 | `HANDOFF/freebuff/queue/V050_WP1_IDENTITY_UNIFICATION_2026-09-21.md` |
| WP2 | `HANDOFF/freebuff/queue/V050_WP2_ROUTING_FEED_ALL_TRANSPORTS_2026-09-21.md` |
| WP3 | `HANDOFF/freebuff/queue/V050_WP3_INBOUND_COMPLETENESS_2026-09-21.md` |
| WP4 | `HANDOFF/freebuff/queue/V050_WP4_DELIVERY_TRUTH_2026-09-21.md` |
| WP5 | Evidence on **this** ticket (not a freebuff impl paste) |

**Completion rule:** mechanical CI/greps **and** harness JEV canonical pack
(`HANDOFF/V040_IMPLEMENTATION_PLAN_WIFI_IDENTITY_2026-09-21.md` §3) before any
WP is marked DONE. **Do not claim WiFi fixed without WP5 3-node logs.**

## Code-truth snapshot (2026-09-21, origin/main — see implementation plan §1)

| Item | Truth |
|---|---|
| CRYPTO-01 / WASM own-topic | Largely on main — verify/tests, do not re-implement blindly |
| CLI hex send | Resolver on main — regression tests + WP5 |
| Contact recovery | Partial — WP1 residual fail-closed placeholder |
| Swarm routing feed | On main |
| Non-swarm routing feed | **OPEN** — WP2 |
| Outbox dual-key | Closed #339 |
| Receipts / honest delivered / wedge | **OPEN** — WP4 |

## Problem

Across the 3-node deployment (Windows + Android + AWS/Ubuntu), messages fail to deliver even on WiFi. Identity and transport are not canonical end-to-end: different id flavors (libp2p peer-id vs public-key hex), send paths that parse the wrong format, transports that connect without feeding routing, and own-topic / spoof issues that drop or mislabel inbound messages.

## Canonical model (non-negotiable)

| Concept | ONE definition |
|---|---|
| Contact identity | **Public-key hex** (canonical store format) |
| Network peer id | Derived via documented `peer_id_from_public_key_hex` / inverse — **never** a second addressing flavor |
| Routing feed | `IronCore::routing_peer_seen(peer_id_hex, transport)` on **every** data-link establishment |
| Send addressing | Contact → canonical hex → peer id if needed; accept hex **and** legacy base58 |
| Inbound identity | Delegate `sender_public_key_hex` = **authenticated** material, not payload `sender_id` |

## Work packages

### WP1 — Identity unification

- [ ] One write format for `contact.peer_id` / `public_key_hex` / ledger peer fields
- [ ] CLI, UI send, core receive, Android bridge, WASM aligned
- [ ] Tests: hex send, name send, recovery restore, restart stability

**Refs:** `todo/P1_CLI_SEND_CANONICAL_IDENTIFIER_PARITY_2026-09-16.md`, `todo/P1_CONTACT_RECOVERY_WRITES_PEERID_AS_PUBLIC_KEY_2026-08-10.md`, `todo/P1_CORE_IDENTITY_SPOOF_AND_WASM_TOPIC_PARITY_2026-09-16.md` (acceptance 1)

### WP2 — Transport → routing feed

- [ ] Same `routing_peer_seen` entry from BLE / WiFiAware / WiFiDirect / WiFi bridges on data-link establish (not discovery only)
- [ ] Fail-closed: unknown/blocked peer → no feed
- [ ] WiFi LAN must not be “connected but unroutable”

**Refs:** `todo/P2_NON_SWARM_TRANSPORT_ROUTING_FEED.md`, `todo/P1_ROUTING_ENGINE_NEVER_LEARNS_PEERS_2026-08-10.md`

### WP3 — Inbound completeness

- [ ] WASM `start_swarm_wasm` subscribes to own `/scmessenger/peer/<own_hex>/v1`
- [ ] Ghost-guard own-topic loss closed with tests
- [ ] Authenticated sender key passed to delegate

**Refs:** `todo/P1_CORE_IDENTITY_SPOOF_AND_WASM_TOPIC_PARITY_2026-09-16.md` (acceptance 2–3), `todo/P1_GHOST_GUARD_OWN_TOPIC_MESSAGE_LOSS_2026-09-16.md`

### WP4 — Delivery evidence

- [ ] Outbox/receipts converge on all 3 nodes
- [ ] Honest crypto-send vs delivered status
- [ ] Windows silent wedge covered in live runs

**Refs:** `todo/P1_ASYNC_DELIVERY_RECEIPTS_DO_NOT_CONVERGE_LIVE_RCA_2026-08-25.md`, `todo/P0_SEND_CRYPTO_FAILS_VS_DELIVERED_2026-08-30.md`, `todo/P1_WINDOWS_NODE_SILENT_WEDGE_2026-09-15.md`, `todo/P1_CLI_OUTBOX_CANONICAL_DRAIN_AND_SLED_UNIFICATION_2026-09-16.md`

### WP5 — 3-node log + live WiFi proof

- [ ] Logs from **all three** nodes for a failed WiFi send window
- [ ] Trace: enqueue → identity resolve → transport → route → send → ack → UI
- [ ] Evidence paths attached to **this** ticket

```text
# Windows
scm ledger tail; contact list; CLI/app logs for the window

# Android
adb logcat -d | grep -E 'scmessenger|packet_lifecycle|outbox|deliver|peer|topic'

# AWS/Ubuntu
service/journal logs + scm ledger tail + contact list
```

## Acceptance

1. One canonical identity across nodes after restart
2. WiFi A→B and B→A delivered on all three nodes
3. CLI hex + name send both succeed
4. WASM/native own-topic subscribed
5. Routing learns peers from every live transport
6. Inbound identity authenticated
7. Receipts converge; operator sees honest status
8. Evidence listed in this file

## Dispatch

- **Lane:** SCMessenger core + CLI + Android + node ops
- **Rule:** isolated worktrees when the repo uses them; prefer one WP per PR
- **Do not merge red; do not claim WiFi fixed without WP5 evidence**
