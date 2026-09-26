# MODEL: glm-5.1:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 4800

# P0_NETWORK_001: Bootstrap Relay Fallback Implementation

**Priority:** P0 CRITICAL  
**Platform:** Android (Cross-platform impact)  
**Status:** IN_PROGRESS  
**Assigned:** Lead Orchestrator (glm-5.1:cloud)  
**Routing Tags:** [REQUIRES: NETWORK_SYNC] [REQUIRES: FINALIZATION]

## Objective
Implement comprehensive fallback strategy for bootstrap relay connectivity failures. All 4 relay servers are failing with "Network error", preventing ANY mesh network connectivity and making cross-device messaging impossible.

## Root Cause
From ANDROID_PIXEL_6A_AUDIT_2026-04-17:
- All 4 bootstrap relay nodes failing: GCP (34.135.34.73) and Cloudflare (104.28.216.43)
- Both QUIC/UDP and TCP endpoints failing with "Network error"
- Likely cellular network blocking non-standard ports (9001, 9010)
- No peer connectivity established (0 peers in mesh stats)
- Complete network isolation

## Implementation Progress

### Phase 1: Racing Bootstrap with Transport Priority  DONE
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- Replaced sequential `bootstrapWithFallbackStrategy()` with `racingBootstrapWithFallback()`
- Races all candidate multiaddrs in parallel with 3s timeout
- Circuit-breaker gates each address before dialing
- First successful dial wins; remaining coroutines cancelled
- mDNS fallback triggered when all relay dials fail

### Phase 2: Network-Change-Triggered Re-Bootstrap  DONE
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- Added `startNetworkChangeWatch()` collecting `networkDetector.networkType` StateFlow
- On network type change: resets circuit breakers and re-bootstraps
- `stopNetworkChangeWatch()` called in `stopMeshService()`

### Phase 3: Proactive Port Probe  DONE
**File:** `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt`
- Added `probePorts()` method: parallel TCP socket probes with configurable timeout
- Advisory only: deprioritizes blocked addresses, doesn't exclude them
- Integrated into `racingBootstrapWithFallback()`: probed relay ports sorted by reachability

### Phase 4: mDNS Fallback After Relay Failure  DONE
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- Added `attemptMdnsFallback()`: waits up to 5s for LAN peer discovery
- Triggered when all relay bootstrap dials fail
- Uses existing `MdnsServiceDiscovery` via mesh stats peer count check

### Phase 5: Enhanced Error Classification  DONE
**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
- Cellular-specific error messages for `ConnectException` and `PortUnreachableException`
- Added `extractPortFromMultiaddr()` helper for port-aware diagnostics
- Identifies carrier port filtering vs generic connection failures

### Phase 6: Rust BootstrapManager Wiring  NOT STARTED
- Secondary optimization: wire Rust `BootstrapManager` into mobile bridge via UniFFI
- The Rust side already has multi-transport fallback and circuit breaker integration
- Currently the Kotlin-side `SwarmBridge.dial()` handles all transports (including WS)
- Low priority since the swarm already supports WebSocket multiaddrs

### Phase 7: Diagnostics UI Enhancement  DONE
- DiagnosticsReporter: Added transport priority, circuit breaker entries, and port probe results to `NetworkDiagnosticsReport`
- DiagnosticsReporter: New constructor params `NetworkDetector` and `CircuitBreaker` for accessing fallback state
- DiagnosticsReporter: `formatReportForUser()` now includes transport priority, port probe results, and circuit breaker states
- NetworkStatusDialog: Added sections for transport priority, port probe status, and circuit breaker states with color-coded indicators
- DiagnosticsScreen: Report card now shows transport priority, port probe summary, and circuit breaker summary

## Files Modified
1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`  Racing bootstrap, network watcher, mDNS fallback, enhanced error classification
2. `android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt`  Port probe method
3. `android/app/src/main/java/com/scmessenger/android/network/DiagnosticsReporter.kt`  Transport priority, circuit breaker entries, port probe results in report
4. `android/app/src/main/java/com/scmessenger/android/ui/dialogs/NetworkStatusDialog.kt`  Transport priority, port probe, circuit breaker UI sections
5. `android/app/src/main/java/com/scmessenger/android/ui/screens/DiagnosticsScreen.kt`  Report card with transport priority, port probe summary, circuit breaker summary

## Success Criteria
-  Bootstrap connectivity races all transports in parallel (not sequential)
-  Circuit breaker gates each address before dial
-  Port probe deprioritizes blocked addresses
-  Network change triggers circuit breaker reset and re-bootstrap
-  mDNS fallback when all relay dials fail
-  Cellular-specific error diagnostics
-  Rust BootstrapManager integration (Phase 6)
- [x] Diagnostics UI enhancement (Phase 7)

## Priority: URGENT
This issue prevents ALL network connectivity. Without relay access, devices cannot communicate across networks.