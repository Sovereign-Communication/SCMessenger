# Android Overhaul Status  2026-04-23

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

This document is owned by SCMessenger (Sovereign-Communication/SCMessenger).

## Fixes Applied Today

| Fix | File | Status |
|-----|------|--------|
| Create Identity race condition | `MeshRepository.kt` | **DEPLOYED**  `ensureServiceInitializedBlocking()` waits for service startup |
| Settings screen defaults | `SettingsViewModel.kt` | **DEPLOYED**  defaults set immediately, no missing sections |
| Settings reload on service start | `SettingsScreen.kt` | **DEPLOYED**  `LaunchedEffect(serviceState)` refreshes identity |
| Diagnostics export crash | `file_paths.xml` | **DEPLOYED**  added `<cache-path>` for FileProvider |
| Bootstrap ANR | `MeshRepository.kt` | **DEPLOYED**  cached static nodes, no network I/O on init |
| Async settings loading | `MeshRepository.kt` | **DEPLOYED**  default settings first, async reload after |
| Settings debouncing | `SettingsViewModel.kt` | **DEPLOYED**  500ms timestamp-based throttle |
| Async diagnostics | `MeshRepository.kt` | **DEPLOYED**  `exportDiagnosticsAsync()` on IO dispatcher |
| Static listen port | `MeshRepository.kt` | **DEPLOYED**  port 9001 for LAN discovery |

## Pending Issues for Next Session

### Identity Display (CRITICAL-5 from ID audit)
- **File:** `ui/screens/SettingsScreen.kt`, `ui/identity/IdentityScreen.kt`
- **Issue:** Labels `identity_id` as "Peer ID"  users copy wrong ID
- **Fix:** Rename to "Identity Hash", add "Peer ID (Network)" showing `libp2p_peer_id`

### QR Code Format (CRITICAL-1 from ID audit)
- **File:** `data/MeshRepository.kt`  `getIdentityExportString()`
- **Issue:** QR exports `"identity_id"` as primary ID, creates broken contacts
- **Fix:** Change to `"peer_id": "<libp2p_peer_id>"`, add `"device_id"`

### Contact Import Parser (CRITICAL-1 from ID audit)
- **File:** `utils/ContactImportParser.kt`
- **Issue:** Reads `"identity_id"` into `contact.peerId`
- **Fix:** Read `"peer_id"` (libp2p format), resolve to `public_key_hex` via core

### Contact Storage Canonicalization (MEDIUM-6 from ID audit)
- **File:** `core/src/store/contacts.rs`, `core/src/api.udl`
- **Issue:** `Contact.peer_id` stores whatever was passed
- **Fix:** Enforce `public_key_hex` as canonical key, add `libp2p_peer_id` field

## Cross-OS Compatibility Checklist

| Feature | Android | iOS | CLI | Status |
|---------|---------|-----|-----|--------|
| Static port 9001 |  |  (ephemeral) |  (9001) | iOS needs fix |
| Bootstrap caching |  |  (network I/O) | N/A | iOS needs fix |
| Settings debounce |  |  | N/A | iOS needs fix |
| Async diagnostics |  |  (sync) | N/A | iOS needs fix |
| Peer ID display |  (wrong label) |  (shows hash) |  | All need fix |
| QR format |  (exports hash) |  (exports hash) | N/A | All need fix |
| mDNS TXT records |  (unknown format) |  (unknown format) | N/A | Needs spec |
| Contact add by Peer ID |  |  |  | Needs test |

## Next Steps

1. **User testing:** Reconnect Pixel 6a, deploy latest APK, test Create Identity + Settings screen
2. **Identity display fix:** Update SettingsScreen labels, add libp2p Peer ID display
3. **QR format fix:** Update `getIdentityExportString()` on both platforms
4. **iOS parity:** Implement all Android fixes in Swift
5. **Cross-OS test:** Android  iOS  CLI LAN messaging

## Agent Status

- **implementer_1776983499**  Running, investigating Settings screen regression
- ** architect_1776981804**  Stale (crashed)
- **precision-validator_1776981818**  Stale (crashed)

*Prepared for full Android overhaul + iOS parity implementation*
