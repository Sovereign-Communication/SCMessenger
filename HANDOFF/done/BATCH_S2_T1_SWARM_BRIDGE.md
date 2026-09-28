# S2-T1: SwarmBridge Wiring

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

## Status
- [ ] TODO

## Task ID
`S2-T1`

## Sprint
Sprint 2: Core Wiring

## LoC Estimate
~200

## Depends
S1-T3 (Core Integration Audit)

## Files
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`
- `core/src/transport/swarm.rs` (Rust side - coordinate with Rust team if needed)

## Actions
1. Locate `swarmBridge` initialization in `MeshRepository.kt`
2. Verify `swarmBridge` is created with correct config (relay servers, transport options)
3. Wire `swarmBridge.dial(peerId)` for INTERNET transport:
   - When `TransportManager.sendData()` fails on BLE/WiFi, route to INTERNET
   - Ensure peerId is properly formatted for libp2p dialing
4. Wire `TransportManager.enableTransport(TransportType.INTERNET)`  `swarmBridge.startListening()`
5. Test peer connection: start service on two devices  verify peer count > 0
6. Handle connection lifecycle: onPeerConnected  MeshEventBus  UI update

## Verification
- Two devices can discover each other over internet transport
- Peer list shows remote peers after connection established
- Messages can be sent over INTERNET transport when BLE/WiFi unavailable

## Notes
- Coordinate with Rust team if SwarmBridge API changes
- Ensure proper error handling for dial failures
- Test with real relay infrastructure (not just localhost)