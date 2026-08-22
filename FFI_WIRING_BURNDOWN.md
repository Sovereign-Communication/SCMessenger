# SCMessenger FFI & Function Wiring Burndown Matrix

**Generated**: 2026-08-14T03:44:54.612230+00:00
**Total Unwired/Stub Functions**: 162 (Unwired: 162, Stubs: 0)

## Overview & Burndown Priorities

This document tracks unwired and stubbed interface functions across **Rust Core**, **Mobile UniFFI**, **Android Kotlin**, and **iOS Swift**.

### High-Priority Stub Implementations (Must be implemented for Phase 4)
| Function | Location | Line | Target Integration Layer |
| :--- | :--- | :---: | :--- |

### Module Breakdown (Top Modules by Unwired Count)
| Module / File | Total Unwired | Stubs | Status |
| :--- | :---: | :---: | :--- |
| `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` | 84 | 0 | ⏳ Pending Audit |
| `AgentSwarmCline/scmessenger_swarm/observability_tests.rs` | 28 | 0 | ⏳ Pending Audit |
| `android/app/src/main/java/com/scmessenger/android/data/PreferencesRepository.kt` | 8 | 0 | ⏳ Pending Audit |
| `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt` | 8 | 0 | ⏳ Pending Audit |
| `android/app/src/main/java/com/scmessenger/android/network/NetworkDiagnostics.kt` | 7 | 0 | ⏳ Pending Audit |
| `adb_extractor.py` | 5 | 0 | ⏳ Pending Audit |
| `test_websocket.py` | 4 | 0 | ⏳ Pending Audit |
| `AgentSwarmCline/scmessenger_swarm/swarm.py` | 3 | 0 | ⏳ Pending Audit |
| `android/app/src/main/java/com/scmessenger/android/service/AndroidPlatformBridge.kt` | 3 | 0 | ⏳ Pending Audit |
| `android/app/src/main/java/com/scmessenger/android/MeshApplication.kt` | 2 | 0 | ⏳ Pending Audit |
| `android/app/src/main/java/com/scmessenger/android/network/DiagnosticsReporter.kt` | 2 | 0 | ⏳ Pending Audit |
| `android/app/src/main/java/com/scmessenger/android/service/AnrWatchdog.kt` | 2 | 0 | ⏳ Pending Audit |
| `AgentSwarmCline/scmessenger_swarm/observability.rs` | 1 | 0 | ⏳ Pending Audit |
| `AgentSwarmCline/scmessenger_swarm/surgeon_graph.py` | 1 | 0 | ⏳ Pending Audit |
| `android/app/src/main/java/com/scmessenger/android/di/AppModule.kt` | 1 | 0 | ⏳ Pending Audit |
| `count_braces.py` | 1 | 0 | ⏳ Pending Audit |
| `fix_swift_generation.py` | 1 | 0 | ⏳ Pending Audit |
| `fix_swift_strings_targeted.py` | 1 | 0 | ⏳ Pending Audit |

## Action Plan for Burndown
1. **Mobile UniFFI Surface**: Wire core transport stubs (`MobileBridge`, `CoreBridge.swift`) to active Kotlin/Swift view models.
2. **Observed Stubs**: Replace simulated mock channels with production libp2p and sled store calls.
3. **Dead Code Clearance**: Remove unreferenced diagnostic helpers that are obsolete.
