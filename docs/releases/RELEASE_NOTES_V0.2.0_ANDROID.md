# SCMessenger Android v0.2.0 (Alpha) Release Notes

## Google Play Console Version (Short - Max 500 chars)

**Note:** Copy and paste this directly into the Play Console Release Notes field.

```text
v0.2.0 Alpha Release:
• True Peer-to-Peer Mesh: Connect instantly via Bluetooth (BLE) and local WiFi without internet or central servers.
• Offline Resilience: Messages queue offline and auto-sync seamlessly when peers reconnect.
• Cross-Platform Sync: Bidirectional background history sync between Android and iOS.
• End-to-End Encryption: All messages and history sync payloads are securely encrypted.
• Rock-solid Reliability: Major fixes to battery usage, BLE routing, and delivery receipts.
```

## Full Release Notes (For GitHub / Internal reference)

###  Major Features

- **Multi-Transport Routing:** Messages now intelligently route across Wi-Fi (libp2p), Bluetooth Low Energy (BLE), and Internet relays based on the fastest available path.
- **Offline-First Architecture:** Messages sent while disconnected are queued locally and automatically converged (synced) the moment a peer is discovered via BLE or Wi-Fi.
- **Cross-Platform History Sync:** Complete history synchronization between Android and iOS peers, utilizing chunked, auto-resuming payloads that bypass traditional Bluetooth packet limits.
- **End-to-End Encryption (E2EE):** Full encryption utilizing X25519 and XChaCha20-Poly1305. All live messages and historical sync batches are encrypted before leaving the device.
- **Deterministic Delivery Receipts:** Cryptographically verified delivery receipts ensure you know exactly when a message has reached the destination device's local database.

###  Bug Fixes & Stability Improvements

- **BLE Transport Hardening:** Resolved crash loops when native GATT connection attempts returned null.
- **Payload Size Limits:** Resolved `IronCoreException` errors by chunking large history sync requests into optimized 20-message batches, preventing Android-to-iOS encryption failures.
- **Transport Priority:** Re-ordered BLE connection priorities (Peripheral over Central) to ensure stable background connections when Wi-Fi is disabled.
- **Concurrent Sync Guards:** Added memory locks to prevent parallel database syncs from crushing the Bluetooth radio.
- **Relay Reconnection:** Greatly improved reconnection logic to internet custody relays when transitioning from offline mesh to online cellular.

###  Known Alpha Limitations (Coming in v0.2.1)

- Background Push Notifications for Direct Messages are deferred to the upcoming v0.2.1 release.
- "Single Active Device" tight-pairing (linking multiple of your own devices to one identity) is currently in development.
