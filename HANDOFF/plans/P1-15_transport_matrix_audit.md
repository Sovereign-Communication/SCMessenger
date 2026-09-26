# P1-15 — Transport-Matrix Ground-Truth Audit

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

**Task:** P1-15 [OPUS+] from `HANDOFF/V1_0_0_EXECUTION_PLAN.md` Section 2 (Stage D).
**Type:** READ-ONLY audit. No code changed. No `crypto-security-auditor` gate (read-only, per plan).
**Date:** 2026-07-04
**Method legend:** [V-READ] = verified by reading source/grep this session. [DEVICE] = can only be settled on real hardware. All build/test claims here are [V-READ]; nothing was compiled or run.

This document answers the four P1-15 questions with file:line evidence, then fills in the plan's Section 2.6 exit matrix with actual per-cell status (implemented / wired / testable / blocked), replacing the uniform "open".

---

## Executive summary

- **WiFi Aware** is genuinely Android<->Android-only by physics. Windows has no Aware peer and the Rust WiFi Aware bridge is inert on Windows (its platform bridge is only ever set on Android via UniFFI). The Android<->Windows equivalent is WiFi Direct/LAN, not Aware. **(a) confirmed architecturally, not "probably".**
- The WiFi Aware instantiation at `core/src/mobile_bridge.rs:393` is **NOT dead-end** on the Android side: it is consumed at `mobile_bridge.rs:1312` and drives a real create-data-path -> libp2p-dial chain, and the Kotlin side has a complete loopback-proxy data path. **The T12c "hardcoded `false` no-op" finding is now STALE/superseded**: the `send()` `false` is a *deliberate, documented* dead-end because delivery moved to a byte-stream loopback proxy, not a missing write path. **(b) traffic flows on Android<->Android (device-unverified); it does not dead-end.**
- **BLE** is a **one-directional data path today**: Android(peripheral, GATT notify) -> CLI(central, subscribe+decode) works end to end in code. **CLI -> Android over BLE is not implemented** (the CLI has zero BLE write/TX). The core `GattServer`/`GattClient` traits (`core/src/transport/ble/gatt.rs`) still have **zero implementations** (CORE_SWEEP_03 holds) and are dead architecture that neither live path uses. **(c) not discovery-only, but half-duplex (Android->CLI only).**
- **Windows-side WiFi Direct has no implementation.** `core/src/transport/wifi_direct.rs` compiles on Windows but is inert (its platform bridge only activates on Android). There is no CLI/Windows WiFi Direct code path. **(d) Android-only; Windows is a stub-by-absence.**

---

## (a) WiFi Aware: Android<->Android-only by physics, and the T12c question

### Is Android<->Android the only physically possible pairing? — YES, and it is architecturally true, not "probably".

WiFi Aware (NAN) requires two Aware-capable radios. The evidence that this is architecturally enforced, not just a hardware assumption:

1. **The Rust WiFi Aware transport only activates behind a platform bridge that is never set on Windows.**
   - `WifiAwareTransport::is_available()` (via `PlatformWifiAwareBridge::is_available`, `core/src/mobile_bridge.rs:1898-1900`) returns `true` **only if** a `PlatformBridge` is registered: `Ok(self.with_platform(|_| true).unwrap_or(false))`. No bridge -> `false` -> transport refuses to initialize (`wifi_aware.rs:300-309`).
   - `set_platform_bridge` (`core/src/mobile_bridge.rs:487`) is called **only from tests** (`core/tests/integration_wifi_aware.rs:108`, and `mobile_bridge.rs:3919/3964/3988` mock harnesses). It is never called from CLI/Windows production code.
   - The `PlatformBridge` trait (`core/src/mobile_bridge.rs:1789`) with `wifi_aware_publish` / `wifi_aware_subscribe` / `wifi_aware_create_data_path` (lines 1804-1807) is implemented in production **only** by Android's `AndroidPlatformBridge.kt` (`android/.../service/AndroidPlatformBridge.kt:466-511`, `class AndroidPlatformBridge ... : uniffi.api.PlatformBridge`).

2. **The CLI has no WiFi Aware transport at all** — only a config flag for status reporting.
   - `cli/src/config.rs:29,74` `enable_wifi_aware: bool` (default true), surfaced in `cli/src/api.rs:125,853`, `cli/src/api_axum.rs:264`, `cli/src/main.rs:3007`. **No** `WifiAwareTransport`, **no** `PlatformBridge` impl, **no** `set_platform_bridge` anywhere under `cli/src/`.

Conclusion: On Windows there is structurally no Aware peer — not merely "no hardware plugged in", but no code path that could ever drive one. The honest form of this cell is **Android<->Android [BLOCKED-HW: needs 2nd Aware-capable Android]**, and the **Android<->Windows equivalent is WiFi Direct / LAN**, exactly as the plan states.

### `mobile_bridge.rs:393` — dead-end instantiation, or exercised end to end?

`*self.wifi_aware_transport.lock() = Some(transport);` (`core/src/mobile_bridge.rs:393`) is **exercised**, not dead-end. It is consumed at:

- `core/src/mobile_bridge.rs:1313` — `on_wifi_aware_peer_discovered` reads the transport and calls `transport.add_discovered_peer(...)`.
- `core/src/mobile_bridge.rs:1320-1337` — same fn clones the transport, and on a spawned task calls `transport.create_data_path(peer_id, &pmk)` (`wifi_aware.rs:366`), then dials the returned IP:port via `swarm_bridge.dial_async(multiaddr_str)` (line 1333). So a confirmed Aware data path becomes a libp2p dial.

The data-path confirmation completes the loop: `create_data_path` (`mobile_bridge.rs:1948`) parks on a `tokio::oneshot`, resolved by `handle_data_path_confirmed` (`mobile_bridge.rs:1867`), which is driven by `on_wifi_aware_data_path_confirmed` (`mobile_bridge.rs:1340`).

### T12c ("Kotlin `PeerConnection.send()` a hardcoded `false` no-op") — REFUTED as currently stated (stale finding).

The release-readiness doc (`docs/release-readiness-2026-07-02.md:426-429`) records: "`PeerConnection.send()` is now a hard-coded `false` no-op while `sendData`-capable routes still call it — either restore a real write path or unregister WiFi Aware...".

The code that finding refers to is `AwareConnection.send()` at `android/.../transport/WifiAwareTransport.kt:512-519`, which does return `false`. **But it is a deliberate, documented architectural dead-end, not a bug**, and the "restore a real write path" remedy is already in place via a different mechanism:

- `WifiAwareTransport.kt:401-425` `startLoopbackProxy(...)` opens a loopback `ServerSocket`, bridges the real cross-device Aware socket to it (`AwareConnection.acceptAndPump`, lines 446-481, byte-pumping both directions), and reports the loopback `127.0.0.1:<port>` to `onDataPathConfirmed` (line 420).
- The class doc (lines 427-437) and the `send()` doc (lines 501-511) state explicitly: once the loopback proxy is active, "the raw peer-socket bytes ARE the libp2p connection (Noise handshake, Yamux multiplexing... happen inside that byte stream)". Delivery happens via **libp2p dialing the loopback address**, not via `send()`. The `false` return is logged at ERROR to make the structural dead-end visible.
- The confirmation reaches Rust: `WifiAwareTransport.onDataPathConfirmed` -> `TransportManager.onWifiAwareDataPathConfirmed` (`TransportManager.kt:442-443`) -> `MeshRepository.kt:895-896` `meshService?.onWifiAwareDataPathConfirmed(peerId, ipAddress, port.toUShort())` -> Rust `on_wifi_aware_data_path_confirmed`. Discovery similarly reaches Rust: `onWifiAwarePeerDiscovered` -> `MeshRepository.kt:892-893` -> `meshService?.onWifiAwarePeerDiscovered(...)`.

**Verdict:** T12c should be re-classified in the ledger. The concern it raises ("delivery fails silently through a false-returning send()") no longer applies: `sendData` for WiFi Aware is *intentionally* out of the data path, and delivery runs through the loopback proxy + libp2p dial. The one residual is a **documentation/ledger correction** plus a live-device confirmation that the loopback-proxy path actually carries a libp2p connection (needs 2 Aware Androids — [DEVICE], [BLOCKED-HW]). No "restore write path" code work is required.

---

## (b) WiFi Aware orphan question in full — does traffic flow or dead-end?

Full call chain from the `wifi_aware_transport` assignment to actual send/receive:

```
[Android radio] WiFi Aware discovery (WifiAwareTransport.kt onServiceDiscovered, :208/:236)
   -> onPeerDiscovered callback -> TransportManager -> MeshRepository.kt:892
   -> meshService.onWifiAwarePeerDiscovered  (Rust mobile_bridge.rs:1312)
        -> transport.add_discovered_peer (registers peer)               [wifi_aware.rs:459]
        -> spawn: transport.create_data_path(peer, pmk)                 [mobile_bridge.rs:1327]
             -> PlatformWifiAwareBridge.create_data_path                [mobile_bridge.rs:1948]
                  -> PlatformBridge.wifi_aware_create_data_path (FFI)   [-> AndroidPlatformBridge.kt:496]
                  -> parks on tokio::oneshot rx

[Android radio] data path established -> WifiAwareTransport.kt initiateDataPath / socket
   -> startLoopbackProxy binds 127.0.0.1:<port>, bridges peer socket <-> loopback  [WifiAwareTransport.kt:401]
   -> onDataPathConfirmed(peerId, "127.0.0.1", loopbackPort)                        [:420]
   -> TransportManager.kt:442 -> MeshRepository.kt:895
   -> meshService.onWifiAwareDataPathConfirmed (Rust mobile_bridge.rs:1340)
        -> aware_bridge.handle_data_path_confirmed -> resolves the oneshot          [mobile_bridge.rs:1867]

  create_data_path returns DataPathInfo{ ip=127.0.0.1, port } (mobile_bridge.rs:1327 continuation)
   -> swarm_bridge.dial_async("/ip4/127.0.0.1/tcp/<port>")                          [mobile_bridge.rs:1333]
   -> libp2p Noise+Yamux handshake runs over the loopback socket, which the
      AwareConnection byte-pump bridges to the real cross-device Aware socket       [WifiAwareTransport.kt:462-467]
```

**Traffic flows** (on the Android side, in code): discovery reaches Rust, a data path is created and confirmed, the loopback proxy carries the bytes, and libp2p dials into it. It does **not** dead-end. The `AwareConnection.send()` `false` (WifiAwareTransport.kt:512) is off to the side of this path by design.

**Unverified:** No device evidence exists that this end-to-end path actually carries a live libp2p session across two Aware radios. That is [DEVICE] + [BLOCKED-HW] (one phone). The audit settles *reachability and wiring* (both hold); it cannot settle *live carriage* by reading. Flagged honestly per the P1-15 instruction.

---

## (c) BLE data path — real bidirectional data path, or discovery-only?

**Answer: half-duplex data path. Android -> CLI is a working data path in code; CLI -> Android is not implemented. Not discovery-only. The core GATT traits remain unimplemented (CORE_SWEEP_03 holds).**

### CLI side (`cli/src/ble_mesh.rs`) — RX (ingress) data path, not discovery-only.
- `run_ble_central_ingress` (line 128) scans for SCM service `0xDF01` (`scm_service_uuid()`, line 27), connects, discovers services, finds the notify characteristic `0xDF03` (`scm_notify_uuid()`, line 31), subscribes (line 102), and streams notifications (line 120).
- Each notify payload is decoded through `decode_ble_payload_for_ui` (line 36) -> `DriftFrame::from_bytes` -> `IronCore::receive_message` (decrypt/verify, line 46) -> pushed to the Web UI (line 122). This is a real **data ingress path**, not discovery.
- **The CLI has NO BLE write/TX path.** Grep for `.write(`, `write_without_response`, `write_char`, `send_ble` across `cli/src/` returns nothing. `run_ble_peripheral_advertising` (line 317) is a **deliberate no-op stub** (documented, lines 299-316: btleplug is central-only on desktop). So the CLI can receive over BLE but cannot send over BLE.

### Android side — full GATT peripheral + client, real bidirectional at the app layer.
- `BleGattServer.kt` is a full peripheral: service `0000df01` (`SERVICE_UUID`, line 541), message characteristic `0000df03` (`MESSAGE_CHAR_UUID`, line 545) with `WRITE | WRITE_NO_RESPONSE | NOTIFY` (lines 108-113), MTU-aware fragmentation (`sendFragmented`, line 229), reassembly buffers (line 46), and `sendData(...)` that notifies subscribers (lines 178-223, `notifyCharacteristicChangedSafe` line 523).
- **Direction alignment:** Android's message char `0xDF03` notify == the CLI's subscribe target `0xDF03`. So **Android(notify) -> CLI(subscribe)** is wired on both ends.
- Android outbound BLE at the app layer: `TransportManager.sendData` (lines 289-315) tries L2CAP -> GATT client write (`bleGattClient.sendData`, :292) -> GATT server notify (`gattServer.sendData`, :297/:302) -> advertiser; `MeshRepository.kt:5977/5995/6192/6210` sends `encryptedData` via `bleClient.sendData` and `bleServer.sendData`. Inbound arrives via `onDataReceived`/`onProximityDataReceived` -> `meshService.onDataReceived` (Rust).

### Architecture note: this BLE path is app-level frame relay, NOT libp2p.
BLE messaging on Android and the CLI runs through the `PlatformBridge`/proximity + `IronCore::receive_message`/Drift-frame path (raw encrypted frames), **not** through the libp2p swarm. That is distinct from the WiFi Aware/Direct paths, which bridge into libp2p via a TCP/loopback dial. This matters for the exit test: BLE delivery does not depend on the libp2p negotiation bug P1-04 is chasing.

### CORE_SWEEP_03 — confirmed still true.
- `core/src/transport/ble/gatt.rs:279-313` defines `trait GattServer` and `trait GattClient`. Grep for `impl GattServer`/`impl GattClient`/`dyn GattServer`/`dyn GattClient` across the whole repo returns **zero** matches. `core/src/transport/ble/mod.rs:27` re-exports `GattServer` but nothing implements it. Neither the CLI (btleplug directly) nor Android (Android BLE stack directly) uses these traits. **They are dead architecture.**
- **Latent bug spotted:** `gatt.rs:10` `GATT_SERVICE_UUID = 0x0000_0DF0_1000_1000_...` is malformed — the nibbles are shifted, yielding `0DF01000` in the first 32 bits, not the intended `0000DF01`. The live paths use the correct `0xDF01` form (`ble_mesh.rs:25` `0x0000_DF01_...`, `BleGattServer.kt:541` `0000df01`, `beacon.rs:15` `0xDF01`). Currently harmless because the malformed constant is unused, but it would misfire the moment anyone wired the trait. Captured in the BLE gap ticket's "Do NOT / watch-outs".

**Net:** BLE is a **one-directional (Android->CLI) data path** today, not discovery-only and not fully bidirectional. Closing the worst-case bidirectional cell (P1-16) requires a **CLI BLE TX path** (CLI must write to Android's `0xDF03` write characteristic as a GATT central, OR advertise as a peripheral so Android can write to it). See gap ticket `P1_CLI_BLE_Outbound_TX_Path_Missing.md`.

---

## (d) Windows-side WiFi Direct — any Windows implementation, or Android-only/stub?

**Answer: Android-only. No Windows implementation exists; `wifi_direct.rs` compiles on Windows but is inert.**

- `core/src/transport/wifi_direct.rs` is gated `#[cfg(not(target_arch = "wasm32"))]` for its `PlatformWifiDirectBridge` (lines 85-212) and the module itself is declared unconditionally (`transport/mod.rs:28`). So it **compiles on Windows** — but that is not a Windows *implementation*.
- Every real action delegates to `crate::PlatformBridge` methods: `discover_peers` -> `b.wifi_direct_discover_peers()` (line 154), `connect` -> `b.wifi_direct_connect(...)` (line 174), `create_group` -> `b.wifi_direct_create_group(...)` (line 187). `is_available()` returns `true` only if a platform bridge is present (`Ok(self.with_platform(|_| true).unwrap_or(false))`, line 149).
- As with WiFi Aware, `PlatformBridge` is set in production **only on Android** (`AndroidPlatformBridge.kt:517-555` implements `wifiDirectDiscoverPeers`/`Connect`/`CreateGroup`/etc.). The CLI has **no** `PlatformBridge` impl and **no** WiFi Direct transport instantiation — only `enable_wifi_direct`-style config flags for status.
- `set_on_message_received` is a no-op even in the real Rust bridge (`wifi_direct.rs:211`): like WiFi Aware, data does not flow through a discrete send callback. The Android GO/client link carries a TCP dial instead — `mobile_bridge.rs:1398` hardcodes `/ip4/{group_owner_ip}/tcp/9001` on the client side (a known hardcoded-port target for P1-13).
- `desktop_bridge/` (the only Windows-ish native bridge crate) contains **no** wifi_direct/wifi_aware references at all (grep clean); it is Linux-BLE-only (`desktop_bridge/src/ble.rs`, itself gated per `P0_DESKTOP_BRIDGE_...`).
- The Kotlin `android/.../transport/WifiDirectTransport.kt` exists and is Android-only.

**Consequence for the matrix:** Android<->Windows WiFi Direct requires the **Windows peer** to join an Android-created P2P group as a legacy client (Windows joins the group's SoftAP over its normal Wi-Fi stack, then TCP-dials the group-owner IP). That is a **P1-17 implementation + [HUMAN] scope decision**, not something that exists today. There is no code path where Windows participates in WiFi Direct at all right now.

---

## 2.6 Exit matrix — filled with current status

Status vocabulary per cell:
- **WIRED** = both endpoints implemented and connected in code (device-unverified unless noted).
- **HALF** = one direction implemented, the other missing.
- **INERT-WIN** = compiles on Windows but no live Windows path (no platform bridge).
- **BLOCKED-HW** = physically needs hardware not available (2nd Android / public endpoint).
- **GAP** = implementation missing; gap ticket filed.
- **[DEVICE]** = code-wired but only closable with a live-hardware pass (P1-09/14/16/17/18).

| Transport | Windows -> Android | Android -> Windows | Worst-case variant | Current status (P1-15) |
|---|---|---|---|---|
| mDNS/LAN discovery | WIRED [DEVICE] | WIRED [DEVICE] | router client-isolation off/on documented | Discovery + dial wired both sides (`ble_mesh` unrelated; LAN via SwarmBridge `MeshRepository.kt:879`, mDNS filter gap tracked P1-06). **Gated by P1-04** negotiation bug for actual session. Testable now on hardware. **CORRECTION (2026-07-05):** a live tandem test found Android's LAN-discovery start path (`TransportManager.startAll()`, both `MdnsServiceDiscovery` and `SubnetProbe`) never actually engages on a fresh install despite being correctly wired — see `HANDOFF/todo/P1_ANDROID_TransportManager_LAN_Discovery_Never_Starts.md`. This is a GAP one stage earlier than P1-04's negotiation bug, not just a "gated by P1-04" wait state. Downgrade this cell's Android->Windows column to **GAP** until that ticket resolves. |
| TCP (laddered ports) | WIRED (single-port today) [DEVICE] | WIRED (single-port today) [DEVICE] | 9001/9002 blocked -> 443/80/ephemeral lands | Base TCP wired; **laddered ports NOT default** (`multiport.rs` exists, CLI default single `tcp/9001` `cli.rs:189`). Ladder = P1-11/12/13. Session blocked by P1-04. |
| WebSocket | WIRED (hardcoded 9002) [DEVICE] | WIRED (hardcoded 9002) [DEVICE] | same ladder | WS listener hardcoded `0.0.0.0:9002/ws` (`swarm.rs:1938`); laddering = P1-11. |
| QUIC | WIRED (udp/0 ephemeral) [DEVICE] | WIRED [DEVICE] | UDP blocked -> falls back TCP/WS | QUIC binds ephemeral already; fallback-to-TCP behavior device-unverified. Session blocked by P1-04. |
| BLE | **GAP (CLI has no TX)** | **WIRED [DEVICE]** (Android notify -> CLI subscribe/decode) | no WiFi/no internet, message lands | **HALF-DUPLEX.** Android->CLI wired end to end (`BleGattServer.kt` notify `0xDF03` <-> `ble_mesh.rs` subscribe/decode). CLI->Android **missing** (no BLE write in `cli/src/`). App-level frame relay, not libp2p (independent of P1-04). Gap ticket: `P1_CLI_BLE_Outbound_TX_Path_Missing.md`. |
| WiFi Direct | **GAP / INERT-WIN** (no Windows WiFi Direct code) | **GAP / INERT-WIN** | phone-hotspot-less direct group | Android side implemented (`WifiDirectTransport.kt`, GO-intent logic); **Windows has no WiFi Direct path at all** — `wifi_direct.rs` inert on Win. Needs P1-17 (Windows-as-legacy-client design) + [HUMAN] scope call. Client-side port hardcoded `tcp/9001` (`mobile_bridge.rs:1398`, P1-13). |
| WiFi Aware | Android<->Android only by physics | — (no Windows Aware peer) | [BLOCKED-HW: 2nd Aware Android or waiver] | Android<->Android path fully **WIRED** in code (discovery -> data path -> loopback proxy -> libp2p dial). **T12c stale** (see (a)/(b)) — no write-path code work needed; needs a ledger correction + [DEVICE][BLOCKED-HW] live proof. Windows equiv = WiFi Direct/LAN. Ticket: `P1_DOCS_WiFi_Aware_T12c_Ledger_Correction.md`. |
| Relay (LAN custody) | WIRED [DEVICE] (protocol) | WIRED [DEVICE] | phone offline during send, custody delivers on return | `integration_relay_custody` exists (protocol-level); 3-node LAN device pass = P1-18. |
| Relay (internet) | [HUMAN endpoint decision] | same | carrier-filter escape via WSS/443 | Relay client has WSS-on-443 rationale (`relay/client.rs`); live WAN proof blocked on public endpoint (AWS excluded). [HUMAN] per plan. |

**Matrix reading:** No cell is truly "green" (all device-unverified), but the *code-implementation* status is now specific per cell. Two cells carry genuine implementation GAPs surfaced by this audit: **BLE CLI->Android TX** and **Windows-side WiFi Direct**. WiFi Aware's apparent gap (T12c) is a stale-ledger artifact, not missing code.

---

## Gap tickets filed (see `HANDOFF/todo/`)

1. `P1_CLI_BLE_Outbound_TX_Path_Missing.md` — [SONNET][AUDIT-GATE][DEVICE] — CLI has no BLE write/TX; blocks the bidirectional BLE worst-case cell (P1-16). Touches `core/src/transport/` + `cli/` -> audit gate.
2. `P1_CORE_BLE_GATT_Traits_Dead_And_Malformed_UUID.md` — [SONNET][AUDIT-GATE] — resolve CORE_SWEEP_03: `GattServer`/`GattClient` traits have zero impls; also fix the malformed `GATT_SERVICE_UUID` in `gatt.rs:10` or remove the dead trait layer. Touches `core/src/transport/` -> audit gate.
3. `P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md` — [OPUS+ spec -> SONNET][AUDIT-GATE][DEVICE][HUMAN] — no Windows WiFi Direct path; needs the Windows-as-legacy-client design + operator scope call feeding P1-17.
4. `P1_DOCS_WiFi_Aware_T12c_Ledger_Correction.md` — [HAIKU] — correct the stale T12c finding in `docs/release-readiness-2026-07-02.md`; the `send()` `false` is a documented deliberate no-op, delivery runs via loopback proxy + libp2p dial. Docs-only, no audit gate.

Tiering and [AUDIT-GATE] flags follow the plan's Section 1.1 rubric. Tickets 1-3 touch `core/src/transport/` and so carry the mandatory `crypto-security-auditor` gate before any implementation is considered done.

## What this audit could NOT settle by reading (honesty ledger)

- Whether the WiFi Aware Android<->Android loopback-proxy path actually carries a live libp2p session — needs 2 Aware-capable Androids ([DEVICE], [BLOCKED-HW], one phone).
- Whether Android->CLI BLE actually delivers a decodable frame on real radios — code aligns on UUIDs/framing, but MTU/fragmentation behavior across a real Pixel<->Windows-adapter link is [DEVICE].
- Whether Windows can join an Android WiFi Direct group as a legacy client on this specific Windows machine's Wi-Fi stack — [DEVICE] + design-dependent (P1-17).
- No cargo/gradle was run; all code claims are [V-READ].
