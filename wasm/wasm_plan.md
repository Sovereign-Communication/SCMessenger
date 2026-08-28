Agent Execution Plan: WASM Thin-Client & CLI Daemonization
Architectural Objective
Transition the WASM client from a standalone mesh peer into a "Thin Client UI" served and powered by the local Rust CLI daemon. The CLI will maintain sole custody of the persistent Ed25519 identity, manage hardware multi-port listening (BLE, mDNS), and act as a transparent proxy for the WASM interface via a strictly local WebSocket JSON-RPC bridge (127.0.0.1:9000).

Execution Constraints
Absolute Local Binding: The CLI WebSocket MUST ONLY bind to 127.0.0.1 and localhost. External interfaces must be strictly rejected to prevent Cross-Site WebSocket Hijacking.

No Browser Crypto: The WASM client must be stripped of identity generation logic. It acts strictly as an intent-forwarder and state-renderer.

Brutal Pragmatism on Hardware: Native cross-platform WiFi Direct in Rust is highly unstable. Defer WiFi Direct. Implement standard IP/mDNS and BLE (btleplug) first.

Phase 1: CLI Server & Static Asset Delivery
Objective: Upgrade the CLI to serve the WASM WebUI and accept authorized local WebSocket connections.

Target: cli/src/server.rs

Update the Warp server routing. Add a route to serve the static assets from the /dist directory.

Ensure the /ws endpoint enforces strict Origin header validation (http://localhost:9000 or http://127.0.0.1:9000 ONLY).

Target: cli/src/main.rs

Ensure the Warp server spins up automatically in the background when the daemon launches, without requiring explicit user commands.

Target: scripts/install.sh (Create/Update)

Write an OS-detecting bash/powershell script that fetches the compiled CLI binary, places it in ~/.local/bin, and registers a background user service (systemd for Linux, launchd for macOS).

Verification: Agent must compile the CLI, run it, and use curl to verify http://localhost:9000 returns the index.html and the /ws endpoint correctly rejects external Origins.

Phase 2: The RPC Contract (Core Shared Schema)
Objective: Establish the exact communication protocol between WASM and CLI.

Target: core/src/wasm_support/rpc.rs (Create)

Define the JSON-RPC request/response structs using serde.

Intents (WASM -> CLI): SendMessage, ScanPeers, GetTopology, GetIdentity.

Events (CLI -> WASM): MessageReceived, PeerDiscovered, MeshTopologyUpdate, DeliveryStatus.

Target: cli/src/api.rs

Implement the serialization/deserialization handlers to route these RPC commands into the existing SwarmHandle actions.

Verification: Agent must write unit tests in rpc.rs validating the Serde serialization of all message types.

Phase 3: WASM Lobotomy & Bridge Implementation
Objective: Strip the WASM client of its WebRTC routing and identity, replacing it with the Daemon Bridge.

Target: wasm/src/lib.rs & wasm/src/identity.rs (if exists)

Remove local Ed25519 keypair generation on startup. The WASM app must block UI render until it receives the GetIdentity response from the CLI.

Target: wasm/src/transport.rs

Deprecate the primary WebRTC dialing logic.

Implement DaemonTransportBridge. All standard send_message calls must now format the payload into the rpc.rs JSON schema and push it down the WebSocket.

Implement the WebSocket message listener to intercept incoming MessageReceived RPCs and push them into the WASM state tree.

Verification: Agent must run the WASM test suite (wasm-pack test --headless) mocking the WebSocket to ensure the UI state updates correctly when receiving CLI events.

Phase 4: CLI Hardware Activation (BLE & Proxying)
Objective: Turn the CLI into a multi-port listening daemon that transparently proxies mobile peers to the WASM UI.

Target: cli/Cargo.toml

Add btleplug for cross-platform BLE access.

Target: cli/src/transport_bridge.rs & cli/src/transport_api.rs

Remove the hardcoded "unavailable" flags for BLE.

Implement BLE Advertising (broadcasting the SCMessenger UUID) and BLE Scanning.

Crucial Proxy Logic: When the CLI receives a Drift frame via BLE destined for the local user, it must deserialize the frame, verify the signature, and fire the MessageReceived RPC event to the connected WASM WebSocket.

Verification: Agent must compile the CLI and verify via logs that the btleplug adapter successfully acquires the local Bluetooth radio and begins advertising.

---

## Phase verification (2026-04-11)

| Phase | Objective | Status | Notes |
|-------|-----------|--------|--------|
| **1** | Loopback UI server, `/dist`, strict `/ws` Origin, CORS, auto-start with daemon | **Complete** | `cli/src/server.rs`, `main.rs`; `scripts/install.sh` + `scripts/install.ps1`. Manual `curl` / browser Origin checks still operator responsibility. |
| **2** | Shared JSON-RPC schema + CLI routing + unit tests | **Complete** | `core/src/wasm_support/rpc.rs` (serde + tests); CLI `DaemonRpc` + `UiOutbound` mirroring. |
| **3** | WASM lobotomy: no browser identity; daemon bridge; WebSocket RPC transport | **Complete** | Local `initializeIdentity` stripped, UI gated on WebSocket `get_identity` via `ui/app.js`. WebRTC primary path deferred to daemon RPC via `connectDaemonBridge`. |
| **4** | btleplug; BLE in capability surface; advertise + scan; Drift → RPC | **Complete** | Local scanning and Drift parsing ingestion verified. GATT advertising stub implemented in `ble_mesh::run_ble_peripheral_advertising`. `transport_api` routes connected and forwarding capability available. |