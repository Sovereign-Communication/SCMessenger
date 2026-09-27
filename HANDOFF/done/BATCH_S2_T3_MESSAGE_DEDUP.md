# S2-T3: Message Deduplication

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
`S2-T3`

## Sprint
Sprint 2: Core Wiring

## LoC Estimate
~150

## Depends
S2-T1 (SwarmBridge Wiring)

## Files
- `android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt`
- `android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt`
- `android/app/src/main/java/com/scmessenger/android/service/MeshEventBus.kt`

## Actions
1. Verify `SmartTransportRouter.checkAndRecordMessage()` called on every send
2. Verify received messages also go through dedup (cross-transport)
3. Implement 300s TTL cache expiration (per spec)
4. Test cross-transport dedup:
   - Send message over BLE  delivery confirmed
   - Same message arrives over WiFi within 5 minutes  suppress duplicate
   - Verify single notification appears
5. Handle cache eviction gracefully (no crashes on expired entries)
6. Add metrics: dedup_hits, dedup_misses, dedup_cache_size

## Verification
- Cross-transport duplicate detection works (BLE + WiFi)
- No duplicate notifications for same message
- Cache expires entries after 5 minutes
- No crashes under high message volume

## Notes
- Deduplication is critical for user experience
- Must handle both sent and received messages
- Monitor cache memory usage