# S1-T3: Core Integration Audit

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Status
- [ ] TODO

## Task ID
`S1-T3`

## Sprint
Sprint 1: Build & Bindings

## LoC Estimate
~100

## Depends
S1-T2 (UniFFI Binding Verification)

## Files
- All 34 Android files calling `uniffi.api.*` (enumerated below)
- Rust core source files (`core/src/**/*.rs`)

## Actions
1. Enumerate all UniFFI functions called across Android codebase:
   - `MeshRepository.kt` - mesh service, contact, history, ledger, settings, AutoAdjust
   - `TransportManager.kt` - BLE, WiFi, INTERNET, TCP_MDNS transports
   - `ChatViewModel.kt` - message send/receive, delivery tracking
   - `ConversationsViewModel.kt` - conversation list, message loading
   - `ContactsViewModel.kt` - contact CRUD operations
   - `SettingsViewModel.kt` - mesh settings, preferences
   - `MeshForegroundService.kt` - service state, notifications
   - `AndroidPlatformBridge.kt` - battery, network, motion callbacks
   - ... (24 more files)
2. For each function: grep Rust core for existence
3. Mark each as:
   -  Implemented (exists in both Kotlin and Rust)
   -  Stub exists (Kotlin has placeholder, needs Rust)
   -  Missing (Kotlin calls it, Rust doesn't have it)
4. Document findings in `docs/UNIFFI_GAP_ANALYSIS.md`

## Verification
- `docs/UNIFFI_GAP_ANALYSIS.md` complete with function-by-function status
- No  Missing items without an owner/plan

## Notes
- Gap analysis feeds into Sprint 2 planning
- Prioritize:  and  items determine S2 scope