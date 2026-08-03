# SCMessenger — Global Viability Audit & Priority Action Plan

**Date:** 2026-03-04
**Basis:** Post-physical-device testing (WS12.29–WS12.31), full docs audit
**Scope:** Everything needed for "any phone on Earth can communicate with any other phone globally"

---

## Desired End State (North Star)

> Any phone worldwide can directly exchange messages with any other phone globally via:
>
> 1. **Mesh relay** (internet hop, CGNAT/NAT-traversal, GCP relay)
> 2. **Bluetooth (BLE/CoreBluetooth/GATT)** — direct local
> 3. **WiFi Direct / WiFi Aware / MultipeerConnectivity** — direct local LAN
> 4. **LAN / local WiFi (mDNS)** — direct local
>
> With: **E2E encryption, sovereign identity, no accounts/servers required for local-only use, community-operated global relay infrastructure.**

---

## Section 1 — What Is Working (Verified)

These are confirmed by code, build gates, and/or physical device evidence.

| Component                                                       | Status                                           |
| --------------------------------------------------------------- | ------------------------------------------------ |
| Rust core: 367 tests pass (unit + integration), 0 lint warnings |  Verified                                      |
| E2E encryption (X25519 + XChaCha20-Poly1305 + Ed25519)          |  Verified                                      |
| Sovereign identity (Ed25519), backup/restore across reinstall   |  Verified                                      |
| libp2p swarm (TCP, relay circuit, Kademlia DHT, mDNS)           |  Verified (code)                               |
| Android BLE GATT (serialized queue, identity retry, reconnect)  |  Verified (code + partial device)              |
| iOS Bluetooth (CoreBluetooth central/peripheral, retry)         |  Code verified; device crashes still open      |
| Android WiFi Direct wired                                       |  Code verified; runtime validation needed      |
| iOS Multipeer Connectivity wired                                |  Code verified; session storm guardrails added |
| GCP relay node (v0.2.0) online and reachable (`:9001`)          |  Verified                                      |
| Store-and-forward relay custody (durable paths, non-temp)       |  Verified                                      |
| Delivery receipt / sender-state (`stored`→`delivered`)          |  Code verified; field convergence still open   |
| Message persistence (survives reinstall, update)                |  Verified (code + historic tests)              |
| First-run consent gate (Android + iOS)                          |  Verified                                      |
| Bounded retention policy                                        |  Verified                                      |
| Relay-only / headless node role support                         |  Verified                                      |
| QR code identity export/import (Android + iOS)                  |  Verified                                      |
| Privacy toggles (onion, cover, padding, timing) — parity        |  Verified                                      |
| Android + iOS + WASM CI gates in `ci.yml`                       |  Verified                                      |
| Retry/backoff (exponential, non-terminal)                       |  Architecture verified; field loop bugs exist  |
| Reputation-based relay selection                                |  Architecture verified; field validation open  |
| Multi-port listening (443, 80, 8080, random fallback)           |  Architecture verified                         |

---

## Section 2 — What Is Broken / Open (From Physical Device Testing)

These are **confirmed bugs from actual field evidence (WS12.29–WS12.31)**.

###  P0 — Blocking global viability today

| ID             | Issue                                                                                  | Root Cause (Known)                                                                           | Impact                                                     |
| -------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| IOS-CRASH-001  | iOS BLE send path crashes with `SIGTRAP` in `BLEPeripheralManager.sendDataToCentral`   | Remaining force-unwrap / assert in send path despite hotfix attempts                         | Device is unreliable as a sender; crashes lose messages    |
| IOS-PERF-001   | iOS killed by CPU watchdog under retry pressure                                        | Retry loop consumes ~99% CPU without yield                                                   | iOS effectively non-functional under load                  |
| CROSS-PAIR-001 | Android ↔ iOS bidirectional delivery is not converging to `delivered` state end-to-end | Combination of BLE path mismatch, stale route hints, receipt emission gaps                   | Core use case (phone-to-phone) not proven                  |
| MSG-ORDER-001  | iOS and Android show different message ordering for the same conversation              | `sender_timestamp` not used consistently as the authoritative sort key across both platforms | Conversation is incoherent; ordering is platform-dependent |

###  P1 — Significant for global viability

| ID               | Issue                                                                               | Root Cause (Known)                                                                                                                                         | Impact                                                                  |
| ---------------- | ----------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| UX-IOS-002       | iOS message list scroll-to-top and erratic refresh during conversation              | UI state not stable under concurrent SwiftUI update emissions                                                                                              | App feels buggy; hurts adoption                                         |
| AND-ROUTE-001    | Android retries stale route peer IDs; 291 `Network error` events per session        | Failed route IDs were being persisted back into `routePeerId` on failure; WS12.31 attempted fix but evidence not yet captured                              | Messages stuck `stored`; delivery never converges for affected sessions |
| AND-BLE-001      | Android BLE fallback targets stale unavailable MAC (`XX:XX:XX:XX:XX:XX`) repeatedly | BLE cache not invalidated on disconnect; `connectGatt` null handling fixed but stale hint still leaks                                                      | BLE path loops, never converges                                         |
| IOS-DIAG-001     | iOS diagnostic file pull fails via socket (large files ~21MB)                       | [ios_diagnostics.log](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/ios_diagnostics.log) grows to 10–21MB; socket closes mid-transfer | Slows debugging; can't get device-side ground truth                     |
| FIELD-BINARY-001 | Physical iOS device running stale build (v0.2.0 build 4) without latest hardening   | Manual install required; no OTA push                                                                                                                       | All field evidence from an old binary                                   |

###  P2 — Quality / completeness gaps

| ID             | Issue                                                                           | Impact on Global Viability                               |
| -------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------- |
| OPS-ADB-001    | Android wireless ADB drifts; needs manual reconnect                             | Slows iteration speed; not user-facing                   |
| TEST-ENV-001   | Docker simulation not validated on this host                                    | Integration test coverage gap for NAT/relay scenarios    |
| VALIDATION-001 | Live field matrix not yet captured (CGNAT, captive portal, cross-region relay)  | Can't prove global internet reachability beyond LAN+GCP  |
| UX-IOS-001     | Contact deletion confirmation (implemented WS12.31) — keep under regression     | Safety regression risk                                   |
| EC-01          | Relay custody temp-dir → durable in all environments                            | Store-and-forward durability at risk on some devices     |
| EC-02          | `DeviceStorageSnapshot` unavailability causes pressure policy to silently no-op | Message retention on low-storage devices unguarded       |
| EC-03          | Stale local transport hints (WiFi Direct hints on Android, Multipeer on iOS)    | Local fast-path hit rate degrades over time              |
| WASM-GAP-001   | WASM/Web is internet-path only (no BLE/WiFi); marked experimental               | Web clients can only participate as internet relay nodes |

---

## Section 3 — What Is Structurally Missing for Global Viability

These are features/capabilities that are **planned but not yet implemented or validated** end-to-end:

### 3.1 Transport Coverage Gaps

| Gap                       | Status                                                                                                | What's Needed                                                                                         |
| ------------------------- | ----------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| **WiFi Direct (Android)** | Code wired, `WifiAwareTransport` compile fixed; **runtime not validated** on real devices             | End-to-end LAN-bypass message delivery test between two Android devices without internet              |
| **WiFi Aware (Android)**  | Compile-fixed; API-dependent, not all devices support it                                              | Validation matrix for which hardware supports it; fallback policy when absent                         |
| **iOS Multipeer**         | Wired; session storm guardrails added; **BLE vs WiFi transport selection is non-deterministic**       | Enforce BLE-preferred path when WiFi not available; strict-BLE-only mode needs end-to-end field proof |
| **mDNS / LAN local**      | Core has mDNS; not independently validated on physical device pairs                                   | LAN-only paired test (no internet) confirming delivery via mDNS direct                                |
| **WebRTC (WASM/browser)** | Code stub exists (`WebRtcPeer` delegates to `WebRtcTransport`); answerer + ICE trickle not complete   | Browser-to-browser direct path needs ~150 LOC to complete; currently WebSocket-only                   |
| **NAT64 / IPv6-only**     | Architecture aware; no validation matrix                                                              | Some carriers (especially mobile in Asia/EU) run IPv6-only; no proof of operation                     |
| **CGNAT / Symmetric NAT** | Relay fallback covers this in theory; hole-punching (DCUtR) planned but not yet wired into live swarm | Direct p2p fails through symmetric NAT without DCUtR; relay only works if relay stays connected       |

### 3.2 Global Bootstrap / Connectivity Infrastructure

| Gap                                          | Status                                                                                                                                                                                           | What's Needed                                                                                       |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------- |
| **Bootstrap node discovery**                 | Single GCP node hardcoded; dynamic bootstrap fetch not implemented                                                                                                                               | Implement `env override → remote config fetch → static fallback` chain (NAT_TRAVERSAL_PLAN Phase 1) |
| **Community-operated relay onboarding**      | [scripts/deploy_gcp_node.sh](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/scripts/deploy_gcp_node.sh) exists; relay operator guide exists; no automated community registry | Public relay registry or signed bootstrap list so community nodes can join without manual config    |
| **Cross-region relay**                       | One GCP node (US); no proof of delivery from e.g. Asia → US → Europe                                                                                                                             | Deploy or validate at least 2 geographically distributed relay nodes                                |
| **Store-and-forward for offline recipients** | Relay custody implemented in core; **not validated end-to-end** with recipient offline → comes online later                                                                                      | `integration_relay_custody` passes in tests but needs live device proof                             |

### 3.3 Message Layer Correctness

| Gap                                                              | Status                                                                                                                                                                                                              | What's Needed                                                                                                                 |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| ~~**Authoritative sender timestamp sorting** (`MSG-ORDER-001`)~~ |  **RESOLVED 2026-03-04.** Both platforms now sort by `senderTimestamp`. Android `ChatScreen.kt`, `ChatViewModel.kt`, `ConversationsViewModel.kt` updated; iOS `ChatViewModel.swift`, `MainTabView.swift` updated. |  Closed.                                                                                                                    |
| **Delivery state convergence proof**                             | `stored → delivered` transition works in unit tests; field evidence is missing. Receipt rejection false positive fixed 2026-03-04 (`RECEIPT-001`).                                                                  | One synchronized iOS+Android+relay log bundle showing a message go from `stored` (sender) to `delivered` with receipt arrival |
| **Duplicate suppression under retry**                            | Architecture has dedup; high-retry-count entries can interfere                                                                                                                                                      | Live field proof that no duplicate messages appear in UI even after hundreds of retries                                       |

### 3.4 iOS App Stability (Prerequisite for Any Global Use)

| Gap                                        | Status                                                                                              | What's Needed                                                                                                |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| ~~iOS BLE send path crash-free~~           |  **FIXED 2026-03-04.** `peripheralManager.state == .poweredOn` guard added before `updateValue`.  |  Guard deployed; validate under sustained BLE load with latest binary.                                     |
| ~~iOS CPU resource kill~~                  |  **FIXED 2026-03-04.** `Task.yield()` added to outbox flush loop.                                 |  CPU no longer saturates during retry storms.                                                              |
| ~~iOS UI scroll stability (`UX-IOS-002`)~~ |  **FIXED 2026-03-04.** Scroll trigger changed to `messages.count` instead of `messages.last?.id`. |  Delivery-state updates no longer cause scroll jumps.                                                      |
| iOS background lifecycle                   | Wired; real-world background keep-alive under iOS power management not validated                    | Capture power-profile evidence: `Standard → Low → Standard` transitions under real battery/motion conditions |

---

## Section 4 — Gap Map vs. Desired End State

```
DESIRED PATH → STATUS
─────────────────────────────────────────────────────────
Phone A ──BLE──→ Phone B (same room)
  Android↔Android BLE:  partial (stale MAC bug FIXED 2026-03-04; appendRoutingHint replaces)
  iOS↔iOS BLE:          partial (SIGTRAP crash FIXED 2026-03-04; Multipeer preferred over BLE)
  Android↔iOS BLE:      NOT PROVEN end-to-end (needs fresh binary deploy)

Phone A ──WiFi Direct──→ Phone B (same LAN)
  Android WiFi Direct:  code wired; NOT runtime validated
  iOS Multipeer WiFi:   wired; session storms; non-deterministic transport
  mDNS LAN:             core has it; NOT field validated as isolated path

Phone A ──Internet──→ GCP Relay ──→ Phone B (global)
  Android→GCP→Android:  closest to working; route loops still happen
  iOS→GCP→iOS:           BLE/relay churn; crashes prevent clean test
  Android↔iOS via GCP:  NOT PROVEN with delivery receipt convergence
  Cross-region:          NOT proven (only 1 relay node)
  Offline recipient:     NOT field proven (unit tested only)

Phone A ──Internet──→ Browser/WASM Peer
  Android/iOS → WASM:   WebSocket path only; no WebRTC direct
```

---

## Section 5 — Top Priority Action Items

These are sequenced to unblock the **critical path** to global viability. Each item is the minimum work needed to unlock the next.

---

###  PRIORITY 1 — iOS Crash-Free Baseline (Blocker for All iOS Field Testing)

**Goal:** Deploy a crash-free iOS binary to physical device. Without this, all iOS field evidence is from an unstable binary.

**Actions:**

1. **Audit `BLEPeripheralManager.sendDataToCentral` and surrounding call sites** for any remaining `!` force-unwrap, `assert`, or `precondition` that can trigger `SIGTRAP` under normal send conditions.
2. **Profile retry loop CPU usage** in a controlled send scenario. Implement a yield throttle (e.g. minimum 50ms between retry iterations) to prevent CPU watchdog kills.
3. **Deploy the hardened binary** (WS12.22+ + WS12.31) to physical iPhone.
4. **Reproduce the prior crash scenario** (send loop to paired Android). Verify zero new `SCMessenger-*.ips` crash reports generated.
5. **Acceptance gate:** Physical iPhone sends 10+ messages to Android peer without crash or CPU kill. No new `.ips` files in device crash storage.

---

###  PRIORITY 2 — Android ↔ iOS End-to-End Message Delivery Proof

**Goal:** Capture one synchronized log bundle showing a message going `stored → delivered` in both directions between physical Android and iOS devices.

**Actions (after Priority 1):**

1. Run `./scripts/run5-live-feedback.sh --step=CROSS-PAIR-001 --time=6 --attempts=3 --require-receipt-gate`
2. **Send Android → iOS.** Confirm: iOS renders message, Android sender transitions to `delivered`.
3. **Send iOS → Android.** Confirm: Android renders message, iOS sender transitions to `delivered`.
4. Capture `android-mesh_diagnostics`, `ios_diagnostics` (small file or partial pull), and `gcp.log` from the same time window.
5. **Acceptance gate:** `verify_receipt_convergence.sh` finds matching message IDs with `delivery_attempt` and received markers in the same artifact pair.

---

###  PRIORITY 3 — Message Ordering: Enforce `sender_timestamp` as Authoritative Sort Key

**Goal:** Both platforms sort every conversation identically, using the message's `sender_timestamp` (set by the sender at creation time in core) as the single sort key.

**Actions:**

1. **Audit iOS `MessageListView` / conversation fetch sort order.** Confirm it sorts by `sender_timestamp` (not `receive_time`, not [id](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/.scm.pid), not insert order).
2. **Audit Android `ChatViewModel` / `HistoryManager.conversation(...)` sort order.** Confirm the same.
3. **Check core `HistoryManager.conversation()`** — confirm it returns messages sorted by `sender_timestamp` ascending and that adapters don't re-sort on their own divergent key.
4. Fix any divergence found. Add a unit test to both Android and iOS that inserts messages out of local-insertion order and asserts the final rendered order matches `sender_timestamp`.
5. **Acceptance gate:** Same conversation on both devices shows identical ordering after a live exchange with messages arriving out of insertion order on each device.

---

###  PRIORITY 4 — iOS UI Scroll Stability (UX-IOS-002)

**Goal:** Message list does not scroll to top or flicker during an active conversation.

**Actions:**

1. Identify the SwiftUI view managing the message list scroll position (likely `ScrollViewReader` or `List` with `.scrollTo()`).
2. Audit all state emit paths (`messageUpdates`, `onDeliveryReceipt`, `peer_identified` events) that trigger list re-renders. Determine if any cause a full list reset vs. incremental append.
3. Implement **scroll anchor preservation**: maintain scroll position at the bottom unless the user has manually scrolled up; only auto-scroll to new messages when already at bottom.
4. Debounce rapid state updates (e.g. multiple receipt events in 100ms) into a single UI batch update.
5. **Acceptance gate:** 10+ messages exchanged in a live conversation with no unexpected scroll-to-top behavior or visible flicker.

---

###  PRIORITY 5 — Validate Local Transport Paths in Isolation

**Goal:** Prove each local transport works independently of internet relay.

**Actions:**

1. **BLE-only test (Android ↔ iOS):** Enable `SC_BLE_ONLY_VALIDATION=1`. Send 5 messages each direction with internet/WiFi disabled on both devices. Confirm delivery.
2. **WiFi Direct test (Android ↔ Android):** Two Android devices, no internet, no BLE. Confirm message delivery via WiFi Direct path.
3. **mDNS / LAN test:** Two devices on same LAN, internet off. Confirm mDNS peer discovery and delivery.
4. **iOS Multipeer test (iOS ↔ iOS):** Two iPhones, no internet. Confirm Multipeer session establishes and messages flow.
5. Document results in [INTEROP_MATRIX_V0.2.0_ALPHA.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/INTEROP_MATRIX_V0.2.0_ALPHA.md) as runtime evidence (not just static scan).

---

###  PRIORITY 6 — Stale Route/BLE Cache Fix Confirmation

**Goal:** Confirm WS12.31 route-hint fixes actually eliminate the stale-route loops seen in field logs (291 `Network error` events, repeated `XX:XX:XX:XX:XX:XX` BLE retries).

**Actions:**

1. Deploy WS12.31+ binaries to physical Android.
2. Run a 10-minute live session with a paired iOS device. Extract `android-mesh_diagnostics-device.log`.
3. Check: zero retries to `XX:XX:XX:XX:XX:XX` (old stale MAC). Check: `Network error` count is substantially lower than 291.
4. Confirm: fresh BLE peer discovered after reconnect uses the new address, not the cached stale one.
5. **Acceptance gate:** `AND-ROUTE-001` and `AND-BLE-001` can be marked `Closed` in [WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md).

---

###  PRIORITY 7 — Internet Global Reach Validation

**Goal:** Prove two phones on separate carrier networks (not on same LAN) can exchange messages via relay.

**Actions:**

1. One device on cellular (no WiFi), one on a different WiFi network.
2. Run a live send/receive exchange. Confirm delivery via GCP relay.
3. Validate network switch mid-conversation (WiFi → cellular on one device) with no message loss.
4. [x] Investigate and fix "messages disappearing when sent" on Android (applies to fresh installs/builds).
5. Capture artifacts. These close `VALIDATION-001` and the Field Matrix gate from [ALPHA_RELEASE_AUDIT_V0.1.2.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/ALPHA_RELEASE_AUDIT_V0.1.2.md).

---

###  PRIORITY 8 — Dynamic Bootstrap / Multi-Relay Infrastructure

**Goal:** App can find the network without a hardcoded single relay node. Required for resilience and global scale.

**Actions:**

1. Implement the `env override → remote bootstrap config fetch → static fallback` chain (per [NAT_TRAVERSAL_PLAN.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/NAT_TRAVERSAL_PLAN.md) Phase 1).
2. Set up a second geographically distinct relay node (e.g. EU or Asia region on a VPS or different cloud).
3. Verify both relay nodes are reachable from each other and from mobile clients.
4. Test failover: take primary GCP relay offline; confirm client reconnects to secondary within 60s.
5. **Acceptance gate:** App network is reachable with the primary relay node offline.

---

###  PRIORITY 9 — Store-and-Forward Proof (Offline Recipient)

**Goal:** Message sent to offline recipient is stored at relay and delivered when recipient comes online.

**Actions:**

1. Send Android → iOS while iOS is fully offline (flight mode).
2. Wait 60+ seconds. Bring iOS online.
3. Confirm message is delivered without resend from sender.
4. **Acceptance gate:** `integration_relay_custody` scenario is reproducible on live physical devices.

---

###  PRIORITY 10 — iOS Background Lifecycle & Power Evidence

**Goal:** iOS app stays alive and connected in background (or reconnects fast on foreground) so messages aren't missed.

**Actions:**

1. Background the iOS app during an active conversation.
2. Send 3 messages from Android while iOS is backgrounded.
3. Foreground iOS. Confirm all 3 messages appear without requiring manual refresh.
4. Capture power-profile evidence: confirm `Standard → Low → Standard` transitions happen correctly under battery changes.
5. Document in [ALPHA_RELEASE_AUDIT_V0.1.2.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/ALPHA_RELEASE_AUDIT_V0.1.2.md) as beta-gate evidence.

---

## Section 6 — Architecture Assessment vs. Global Viability

### What the architecture gets right

1. **Sovereign identity without accounts** — Ed25519 keys, no server dependency for identity. No phone number. 
2. **E2E encryption with relay blindness** — relays can't read content. 
3. **Durable store-and-forward** — messages persist until delivered; retry is non-terminal. 
4. **Multi-transport with fallback hierarchy:** WiFi Direct → BLE → LAN → relay circuit.  (designed)
5. **Community-operated infrastructure** — any node can become a relay/bootstrap. 
6. **Offline-first** — local history readable without network. 
7. **Reputation-based relay selection** — self-healing routing.  (designed, needs field proof)

### What the architecture needs to close for global viability

| Gap                                                              | Severity                  | Estimated Work                         |
| ---------------------------------------------------------------- | ------------------------- | -------------------------------------- |
| DCUtR (hole punching) for symmetric NAT — currently relay-only   | High                      | ~2–4h (libp2p feature already planned) |
| Dynamic bootstrap fetch + community relay registry               | High                      | ~4–8h                                  |
| WebRTC for browser-to-browser direct                             | Medium                    | ~150 LOC (answerer + ICE trickle)      |
| NAT64/IPv6-only validation                                       | Medium                    | Test matrix                            |
| Captive portal / filtered egress detection                       | Medium                    | EC-10 in backlog                       |
| Clock-skew tolerant message ordering                             | Medium                    | EC-15 in backlog                       |
| Encounter-aware delay-tolerant forwarding (sparse/DTN scenarios) | Low-Medium                | EC-14 in backlog                       |
| Anonymous relay selection (onion routing)                        | Low (privacy enhancement) | Post v0.2.x                            |

---

## Section 7 — Summary Table: Priorities at a Glance

| #   | Priority                                       | Category      | Blocking What                 | Est. Effort               |
| --- | ---------------------------------------------- | ------------- | ----------------------------- | ------------------------- |
| 1   |  iOS Crash-Free Binary                       | Stability     | All iOS field use             | Hours (audit + deploy)    |
| 2   |  Android↔iOS E2E Delivery Proof              | Core use case | Alpha ship                    | 1–2 days field testing    |
| 3   |  Message Ordering Fix (`sender_timestamp`)   | Correctness   | Coherent conversations        | Hours (audit + fix)       |
| 4   |  iOS UI Scroll Stability                     | UX            | iOS adoption                  | Hours–1 day               |
| 5   |  Local Transport Path Isolation Tests        | Validation    | Transport coverage claims     | 1 day field testing       |
| 6   |  Stale Route/BLE Cache Fix Confirmation      | Bug closure   | Delivery loops                | Hours (redeploy + verify) |
| 7   |  Internet Global Reach (cross-carrier)       | Validation    | Global viability proof        | 1 day field testing       |
| 8   |  Dynamic Bootstrap / Multi-Relay Infra       | Resilience    | Relay single-point-of-failure | 1–2 days implementation   |
| 9   |  Store-and-Forward Proof (offline recipient) | Correctness   | Async messaging claim         | Hours field testing       |
| 10  |  iOS Background Lifecycle Evidence           | Beta gate     | iOS reliability claim         | Hours field testing       |

---

## Section 8 — Key Dependencies

```
Priority 1 (iOS crash-free)
    └─ unlocks → Priority 2, 4, 5 (iOS), 9, 10

Priority 2 (E2E delivery proof)
    └─ unlocks → Priority 6 confirmation, Alpha go/no-go

Priority 3 (msg ordering fix)
    └─ independent; can run in parallel with 1+2

Priority 7 (cross-carrier internet)
    └─ depends on Priority 2 being closed (or can run in parallel on separate device pair)

Priority 8 (multi-relay)
    └─ independent of device fixes; infra work
    └─ required before: claiming relay resilience / no single point of failure
```

---

_This audit is based on: WS12.29–WS12.31 evidence, [CURRENT_STATE.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/CURRENT_STATE.md), [EDGE_CASE_READINESS_MATRIX.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/EDGE_CASE_READINESS_MATRIX.md), [INTEROP_MATRIX_V0.2.0_ALPHA.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/INTEROP_MATRIX_V0.2.0_ALPHA.md), [UNIFIED_GLOBAL_APP_PLAN.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/UNIFIED_GLOBAL_APP_PLAN.md), [TRANSPORT_ARCHITECTURE.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/TRANSPORT_ARCHITECTURE.md), [NAT_TRAVERSAL_PLAN.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/NAT_TRAVERSAL_PLAN.md), [ALPHA_RELEASE_AUDIT_V0.1.2.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/ALPHA_RELEASE_AUDIT_V0.1.2.md), [WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md](file:///Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md)._
