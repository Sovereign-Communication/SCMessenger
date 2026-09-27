# S2-T2: TopicManager Integration

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
`S2-T2`

## Sprint
Sprint 2: Core Wiring

## LoC Estimate
~150

## Depends
S2-T1 (SwarmBridge Wiring)

## Files
- `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt`
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `core/src/drift/` (gossipsub topic handling - Rust side)

## Actions
1. Verify `meshRepository.subscribeTopic(topic)` calls actual gossipsub subscribe
2. Wire `TopicManager.initialize()` into `MeshRepository.onMeshServiceStarted()`
3. Implement topic unsubscribe for cleanup
4. Document default topics:
   - `/scmessenger/global/v1` - global mesh chat
   - `/scmessenger/discovery/v1` - peer announcements
   - `/scmessenger/relay/v1` - message relaying
5. Test: subscribe to discovery topic  receive peer announcements within 30s
6. Handle topic message routing: topic  appropriate handler (chat vs discovery vs relay)

## Verification
- Subscribe to `/scmessenger/discovery/v1`  peer announcements appear
- Unsubscribe stops receiving messages for that topic
- Topic messages route to correct handlers

## Notes
- Discovery topic is critical for peer discovery over internet
- Ensure topic subscription survives network reconnection