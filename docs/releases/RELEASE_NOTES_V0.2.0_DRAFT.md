# SCMessenger v0.2.0-alpha

Status: Draft
Last updated: 2026-03-03

This major update introduces significant improvements to reliability, transport variety, and cross-platform feature parity, moving SCMessenger closer to a public beta.

## Major Highlights
- **Infinite Retry Engine**: Replaced the finite retry cap with a persistent, infinite retry state machine. Messages now retry indefinitely across app restarts until delivery succeeds or the recipient is purged.
- **Relay Custody (Store-and-Forward)**: Relays now accept and persist messages for offline recipients. Messages are delivered automatically when the recipient reconnects, without requiring the sender to be online.
- **Multi-Transport Expansion**:
  - **Android WiFi Direct**: Fully wired high-throughput local transport path.
  - **iOS Multipeer**: Fully integrated local high-throughput transport path.
  - **Deterministic Fallback**: Sophisticated local fallback ordering (`LAN -> BLE -> Relay`) ensures the best path is always used.
- **Desktop GUI Parity**: The Desktop app now features a full Graphical User Interface (GUI) driven by local WASM/Core APIs, reaching parity with mobile onboarding, contacts, and messaging workflows.
- **Headless Mode support**: Improved support for identity-less relay nodes, including stable network PeerIDs and role-based UI gating.

## Reliability & Performance
- **Receipt Convergence**: Implemented network-wide receipt fanout to stop duplicate concurrent retries once a final delivery is observed.
- **Dynamic Storage Controls**: Added a storage-pressure quota and rolling purge policy to protect device storage. SCMessenger will now never push a device above 90% utilization.
- **Anti-Abuse Guardrails**: Introduced per-peer rate limits and global inflight caps for relay queue protection to ensure network stability under high load.
- **Message Payload Cap**: Standardized a strict 8KB cap for text-only payloads to ensure optimal performance over low-bandwidth BLE and Relay paths.

## UX Improvements
- **Delivery States**: Clearer mapping of message states (`pending`, `stored`, `forwarding`, `delivered`) in the UI.
- **Auto-Resume**: All platforms now proactively check for existing local data on startup, hydrating the session without manual user intervention.
- **First-Run Consent**: Added a mandatory privacy and security consent gate to all platform onboarding flows.
- **Install-Mode Choice + Late Identity Creation**: GUI variants now restore explicit first-run choice (`Generate Identity Now` vs `Skip for Relay-Only Install`) and allow identity creation later from Settings -> Identity without reinstall.

## Developer & Ops
- **Unified Core Settings**: Consolidated settings models into a single canonical schema in `IronCore`.
- **Release Matrix Expansion**: Verified build targets for Android, iOS, Windows, macOS, and Linux.
- **Deterministic Integration Tests**: Added comprehensive offline and partition scenario coverage in the core test suite.
