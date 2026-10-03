# P0_IOS_003: iOS Parity Gap Remediation

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-04-23
**Priority:** P0
**Status:** Completed
**Scope:** iOS Swift codebase parity with Android fixes

## Background
The iOS parity plan (P0_IOS_002) has been reviewed. Many fixes are already partially implemented in the Swift codebase. This task covers verifying all checklist items and implementing any remaining gaps.

## Verification Checklist (from P0_IOS_002 Appendix B)

### Already Implemented 
1. **Bootstrap caching**  `MeshRepository.swift:82-85` uses computed property returning `staticBootstrapNodes`
2. **Async diagnostics export**  `MeshRepository.swift:2632-2651` has `exportDiagnostics()` + `exportDiagnosticsAsync()` with 1s TTL cache
3. **Settings debouncing**  `SettingsViewModel.swift:22-36` has `debouncedUpdateSettings()` with 500ms interval
4. **Static port 9001**  `MeshRepository.swift:643,800` uses `/ip4/0.0.0.0/tcp/9001`
5. **Identity display**  `SettingsView.swift:157-188` shows "Peer ID (Network)" (libp2p_peer_id) and "Identity Hash" separately

### Remaining Gaps Fixed 

| Gap | Fix Applied |
|-----|-------------|
| QR format unification | Added `"peer_id"` as primary key in `getIdentityExportString()` JSON |
| Diagnostics share sheet | Already implemented via `UIActivityViewController` wrapper |
| Async settings loading | Already implemented via default-settings-first pattern in `startMeshService` |
| mDNS TXT records | Added `"device_id"` to iOS mDNS advertisement |
| Device ID in export | Added `"device_id"` to identity export payload |

## Files Modified

| File | Change |
|------|--------|
| `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:6037-6073` | Added `"peer_id"` and `"device_id"` as primary keys in JSON export |
| `iOS/SCMessenger/SCMessenger/Transport/mDNSServiceDiscovery.swift:86-100` | Added `"device_id"` to mDNS TXT records |

## Acceptance Criteria - VERIFIED

- [x] iOS QR code scans correctly on Android (`peer_id` is now primary key, matching Android format)
- [x] Android QR code scans correctly on iOS (symmetric `peer_id` format)
- [x] Diagnostics export uses share sheet (`UIActivityViewController` already implemented)
- [x] mDNS TXT records match Android format (`peer_id`, `pubkey`, `device_id`, `version`, `transport`)
- [x] No blocking I/O in `startMeshService` (default-settings-first pattern verified)

## Implementation Details

### Identity Export String Format
The `getIdentityExportString()` function now exports:
```json
{
  "peer_id": "<libp2p Peer ID>",        // PRIMARY: for cross-OS QR scanning
  "public_key": "<hex-encoded Ed25519>",
  "device_id": "<installation UUID>",
  "identity_id": "<Blake3 hash>",       // SECONDARY: backward compatibility
  "nickname": "<user nickname>",
  "libp2p_peer_id": "<libp2p Peer ID>", // Backward compatibility
  "listeners": [...],
  "external_addresses": [...],
  "connection_hints": [...],
  "relay": "<relay string>"
}
```

### mDNS TXT Records Format
The iOS mDNS advertisement now includes:
```swift
[
  "peer_id": "<libp2p Peer ID>",
  "pubkey": "<16 char prefix of public key>",
  "device_id": "<installation UUID>",
  "version": "1.0",
  "transport": "tcp"
]
```

## Verification Status

All acceptance criteria verified against:
- Android `MeshRepository.kt:7637-7647` identity export format
- Android mDNS service registration format
- iOS DiagnosticsView share sheet implementation
- iOS `startMeshService` settings loading pattern

---
*Task completed: 2026-04-23*
