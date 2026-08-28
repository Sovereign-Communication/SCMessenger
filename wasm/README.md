# scmessenger-wasm

WebAssembly bindings crate for SCMessenger.

## Purpose

`scmessenger-wasm` wraps `scmessenger-core` with a browser-friendly `wasm-bindgen` API.
It provides identity/crypto/message operations and a browser-native libp2p swarm runtime.

## Key Exports

- `IronCore` wrapper (`new`, `withStorage`, `start`, `stop`)
- Identity and signature helpers
- Message prepare/receive methods
- `startSwarm(bootstrapAddrs)` to start libp2p swarm networking in browser
- `stopSwarm()` to cleanly shut down the swarm runtime
- `sendPreparedEnvelope(peerId, envelopeBytes)` for encrypted envelope delivery
- `getPeers()` for connected-peer enumeration
- `getConnectionPathState()` for canonical route-state diagnostics
- `exportDiagnostics()` for partner-support JSON snapshots
- `drainReceivedMessages()` for batched JS-side consumption

`startReceiveLoop(relayUrl)` remains available as a deprecated compatibility shim.
It now maps relay URLs to websocket multiaddrs and delegates to `startSwarm`.

## Source Map

- Main API: `wasm/src/lib.rs`
- Transport helper: `wasm/src/transport.rs`
- Connection state: `wasm/src/connection_state.rs`
- Worker helper: `wasm/src/worker.rs`

## Build and Test

From repository root:

```bash
cargo build -p scmessenger-wasm
cargo test -p scmessenger-wasm
```

Browser-runtime tests require `wasm-pack` in the environment.
