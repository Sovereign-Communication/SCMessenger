# SCMessenger v0.1.2-alpha

Status: Draft
Last updated: 2026-03-03

This release focuses on bringing the Web/WASM platform to parity with mobile networking and hardening the core storage layer.

## Highlights
- **WASM libp2p Networking**: The browser client now uses a full libp2p swarm via `wasm-bindgen` and `websocket-websys` transport, enabling peer discovery and messaging directly from the browser.
- **Diagnostics API**: Added `exportDiagnostics()` to both UniFFI (mobile) and WASM (web) APIs, allowing testers to export structured bundles for easier debugging.
- **Connection Diagnostics**: Added `ConnectionPathState` to track whether a peer is reached via LAN, BLE, or Relay.
- **Storage Resilience**: Hardened the persistence layer with explicit schema versioning and dedicated sub-stores for identity, inbox, and outbox data.
- **Identity Maintenance**: Fixed issues where identities were being re-generated on app update; identities now hydrate from secure local storage or backups on startup.

## Tri-Platform Parity
- Web/WASM networking parity with mobile clients.
- Unified settings schema across Android, iOS, and Web.
- Consistent identity and nickname handling across all platforms.

## Developer Changes
- Refactored `IronCore` to manage all storage-backed operations.
- Updated UniFFI surface for improved type safety in mobile integrations.
