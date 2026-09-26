# S2-T4: Relay Bootstrap Infrastructure

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
`S2-T4`

## Sprint
Sprint 2: Core Wiring

## LoC Estimate
~200

## Depends
S2-T1 (SwarmBridge Wiring)

## Files
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- `core/src/relay/` (bootstrap nodes - Rust side)

## Actions
1. Implement `BootstrapSource` chain:
   - `EnvironmentBootstrapSource`: reads `SC_BOOTSTRAP_NODES` env var
   - `StaticFallbackSource`: hardcoded fallback nodes (QUIC prioritized over TCP)
2. Add relay health monitoring:
   - Track connection failures per relay
   - Mark node unreachable after 3 consecutive failures
   - Log failure reason (timeout, refused, protocol error)
3. Implement automatic failover:
   - On relay failure  try next healthy node
   - Circuit breaker pattern (allowRequest check)
   - Backoff after repeated failures
4. Add QUIC preference (per spec: cellular-friendly for NAT traversal)
5. Test: kill primary relay  verify automatic switch to fallback

## Verification
- Relay connection succeeds within 10s of service start
- Survives single node failure (automatic failover)
- No repeated connection attempts to dead nodes
- Circuit breaker prevents cascade failures

## Notes
- QUIC preferred for cellular (UDP-friendly)
- TCP fallback for WiFi/enterprise networks
- Health monitoring prevents wasted connection attempts