# QA E2E Android Discovery Test

**Date:** 2026-04-30
**Agent:** Swarm QA Agent (Claude Code)
**Test Harness:** `scripts/scmdriver.ps1` v2.0 (updated with reset/clean/scan/send commands)

## Test Objective

Execute an E2E network discovery test against the local SCMessenger daemon to determine if it can discover and communicate with an Android phone on the local network after fixing the libp2p PeerID null-drop bug and network thrashing.

## Pre-Flight Fixes Applied (2026-04-30)

### Phase 1: libp2p Peer ID Null-Drop Fix (CRITICAL)
- **`core/src/identity/keys.rs:111`** — `to_libp2p_peer_id()` now has a fallback path.
  If `ed25519::PublicKey::try_from_bytes()` fails (libp2p 0.53 API skew), it falls back to
  deriving the PeerId via the full keypair route: `to_libp2p_keypair() → public() → to_peer_id()`.
- **`android/.../MeshRepository.kt:3496`** — `getIdentityInfoNonBlocking()` no longer returns null
  just because the mesh service isn't RUNNING. It tries `ironCore.getIdentityInfo()` directly first.
- **`android/.../SettingsViewModel.kt:150`** — Added reactive identity refresh. Observes
  `meshRepository.serviceState` and auto-refreshes identity when service transitions to RUNNING.

### Phase 2: Android Zero-Warning Build
- **`Theme.kt:46`** — `@Suppress("DEPRECATION")` on `window.statusBarColor`.
- **`NetworkDiagnostics.kt:85`** — Fixed always-true `InetAddress.getByName() != null`.
- **`BleGattClient.kt:35`** — Class-level `@Suppress("DEPRECATION")` for `writeCharacteristic`, `ByteArray` setters.
- **`BleGattServer.kt:26`** — Class-level `@Suppress("DEPRECATION")` for `notifyCharacteristicChanged`, `writeDescriptor`, `ByteArray` setters.
- **`MeshRepository.kt:2743`** — Removed unused `canonicalPublicKey` variable.
- **`MeshRepository.kt:3165`** — Renamed `canonicalId` → `resolvedCanonicalId` (shadowing fix).
- **`MeshRepository.kt:2746`** — Unused destructuring `candidate` → `_`.

### Phase 3: Network Thrashing & Circuit Breaker Fix
- **`MeshRepository.kt:7324`** — `racingBootstrapWithFallback()` no longer blindly calls `resetAll()`.
  Only resets when transitioning TO WiFi/Ethernet AND there are open circuits to clear.
- **`MeshRepository.kt:7468`** — `startNetworkChangeWatch()` now uses exponential backoff cooldown.
  Each flap doubles cooldown: 30s → 60s → 2m → 4m → 8m → 10m cap. Resets after 2min stability.

---

## Test Procedure

### Step 1: Wipe Windows CLI Identity
```powershell
powershell.exe -NoProfile -File .\scripts\scmdriver.ps1 "reset"
```
Expected: Removes `C:\Users\kanal\AppData\Local\scmessenger`, forcing fresh identity + clean DHT.

### Step 2: Deploy Android App to Phone
```bash
cd android && ./gradlew assembleDebug -x lint --quiet
./android/install-clean.sh
```
**WAIT for user confirmation:**
- [ ] Phone screen is ON and app is in foreground
- [ ] Settings screen shows identity loaded
- [ ] libp2p Peer ID is NOT null (should show `12D3KooW...` format)

### Step 3: Start Windows Daemon (Fresh Identity)
```powershell
powershell.exe -NoProfile -File .\scripts\scmdriver.ps1 "start"
```
Wait 5-8 seconds for all port binds.

### Step 4: Scan for Peers
```powershell
powershell.exe -NoProfile -File .\scripts\scmdriver.ps1 "scan"
```

### Step 5: List Contacts
```powershell
powershell.exe -NoProfile -File .\scripts\scmdriver.ps1 "contact list"
```

### Step 6: Send Test Message
If an Android peer ID is discovered:
```powershell
powershell.exe -NoProfile -File .\scripts\scmdriver.ps1 "send <ANDROID_PEER_ID> 'Hello from the Swarm QA Agent!'"
```

---

## Results

| Step | Status | Notes |
|------|--------|-------|
| 1. CLI reset | PENDING | Run: `powershell -File .\scripts\scmdriver.ps1 reset` |
| 2. Android deploy | PENDING | User must rebuild + reinstall on phone |
| 3. Daemon start | PENDING | Run: `powershell -File .\scripts\scmdriver.ps1 start` |
| 4. Peer scan | PENDING | Run: `powershell -File .\scripts\scmdriver.ps1 scan` |
| 5. Contact list | PENDING | Run: `powershell -File .\scripts\scmdriver.ps1 "contact list"` |
| 6. Test message | PENDING | Run if peer discovered in step 4 |

### Final Verdict: [ ] PASS / [ ] FAIL

---

## Previous Test Reference (2026-04-30, pre-fix)

- Daemon started successfully: Peer ID `12D3KooW<redacted>`
- Contact list: 2 contacts (LukeAndroid `12D3KooWDYF6...`, Treystu `12D3KooWHEY9...`)
- 0/114 known peers reachable; connection path stuck at "Bootstrapping"
- Root cause: Android phone likely offline + stale ledger entries

## Artifacts

| Artifact | Path | Status |
|----------|------|--------|
| PS1 Bridge v2 | `scripts/scmdriver.ps1` | Updated (reset/clean/scan/send commands) |
| Rust PeerID fix | `core/src/identity/keys.rs` | Updated (fallback derivation) |
| Android FFI fix | `android/.../MeshRepository.kt` | Updated (non-blocking identity) |
| Android thrash fix | `android/.../MeshRepository.kt` | Updated (exponential backoff cooldown) |
| Test Report | `HANDOFF/todo/QA_E2E_ANDROID_DISCOVERY.md` | This file |
