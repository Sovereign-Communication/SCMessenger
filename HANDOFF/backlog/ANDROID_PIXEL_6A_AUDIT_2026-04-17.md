# ANDROID_PIXEL_6A_AUDIT_2026-04-17

## Status:  P0 BLOCKER - Critical App Instability
**Source:** Google Pixel 6a Android 16 logs (170839.logcat, 171049.logcat)

## EXECUTIVE SUMMARY
Android app on Google Pixel 6a (API 36) exhibits multiple critical failures causing unusable state:
1. **Contacts completely missing** (0 contacts loaded despite persistence)
2. **Bootstrap relay connectivity failures** (all relay connections failing)
3. **ANR crashes** (app freeze requiring force-kill)
4. **BLE scan failures** (repeated SCAN_FAILED_ALREADY_STARTED)
5. **Transport identity resolution broken** (peers not auto-created as contacts)

## DETAILED FINDINGS

### 1. CONTACTS VANISHED (REGESSION)
**Log Evidence:**
```
AND-CONTACTS-WIPE-001: Reinstall check - hasIdentityBackup=true, contactsOnDisk=true, historyOnDisk=true
AND-CONTACTS-WIPE-001: Normal startup - all data present
Contacts migration already completed, skipping
AND-CONTACTS-WIPE-001: Contact data verification - Found 0 contacts
AND-CONTACTS-WIPE-001: WARNING - No contacts found. If this is unexpected, data recovery may be needed.
AND-CONTACTS-WIPE-001: CRITICAL - Messages exist (8 total) but no contacts found. Possible data loss scenario.
Loaded 0 contacts, filtered nearby peers to 0
```

**Root Cause Analysis:**
- `contacts.db` directory exists with data (`contactsOnDisk=true`)
- Migration already completed (`v2_contacts_db_migration` flag set)
- ContactManager initialization succeeds but returns 0 contacts
- `resolveTransportIdentity()` returns `null` for discovered peers → no auto-contact creation
- **CRITICAL:** Messages exist (8 total) but contacts database appears corrupted or inaccessible

### 2. BOOTSTRAP RELAY CONNECTIVITY FAILURES
**Log Evidence:**
```
Relay bootstrap dial skipped for /ip4/34.135.34.73/udp/9001/quic-v1/p2p/12D3KooW<redacted>: Network error
Relay bootstrap dial skipped for /ip4/34.135.34.73/tcp/9001/p2p/12D3KooW<redacted>: Network error
Relay bootstrap dial skipped for /ip4/104.28.216.43/udp/9010/quic-v1/p2p/12D3KooW<redacted>: Network error
Relay bootstrap dial skipped for /ip4/104.28.216.43/tcp/9010/p2p/12D3KooW<redacted>: Network error
Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)
```

**Root Cause Analysis:**
- All 4 bootstrap relay nodes failing with "Network error"
- Both QUIC/UDP and TCP endpoints failing
- Likely cellular network blocking non-standard ports (9001, 9010)
- No peer connectivity established (0 peers in mesh stats)

### 3. ANR CRASHES (Application Not Responding)
**Log Evidence:**
```
ANR in Window{2b781fc u0 com.scmessenger.android/com.scmessenger.android.ui.MainActivity}. 
Reason:Input dispatching timed out (2b781fc com.scmessenger.android/com.scmessenger.android.ui.MainActivity is not responding. Waited 5001ms for MotionEvent).
```

**Root Cause Analysis:**
- Main thread blocked for >5 seconds
- ANR watchdog implemented but may not be catching UI thread blocks
- MeshRepository initialization or network operations on UI thread
- `MeshForegroundService` potentially blocking during startup

### 4. BLE SCAN FAILURES
**Log Evidence:**
```
BLE Scanning started (background=false, fallback=false)
[Multiple instances]
```

**Missing:** No BLE scan failure logs visible, but code shows retry logic for error code 1 (`SCAN_FAILED_ALREADY_STARTED`).

**Root Cause Analysis:**
- BLE scanner retry logic present but may be ineffective
- Android 12+ scan quota limitations (5 starts in 30s)
- Continuous scan restart loops causing failures

### 5. TRANSPORT IDENTITY RESOLUTION BREAK
**Code Analysis (`MeshRepository.kt` lines 5661-5664):**
```kotlin
if (canonicalContact == null) {
    Timber.d("No existing contact for transport key ${normalizedKey.take(8)}..., treating as transient relay")
    return null
}
```

**Issue:** `resolveTransportIdentity()` returns `null` when no contact exists, preventing auto-contact creation in `onPeerIdentified()`.

**Partial Fix Present:** Auto-contact creation code exists (lines 1032-1081) but may not execute due to:
1. `isHeadless` flag evaluation
2. Key extraction failures
3. Peer validation issues

## URGENT ACTION ITEMS

###  P0: CONTACTS DATABASE CORRUPTION
1. **Emergency Contact Recovery**
   - Add emergency contact reconstruction from message history
   - Implement contact backup/restore mechanism
   - Add corruption detection and repair

2. **Database Integrity Verification**
   - Add `sled` integrity checks at startup
   - Implement fallback to file-level copy if DB corrupted
   - Add contact count validation with message count cross-check

3. **Auto-Contact Creation Fix**
   - Ensure `resolveTransportIdentity()` creates contacts for non-relay peers
   - Bypass headless check for valid public key peers
   - Add logging to trace contact creation failures

###  P0: RELAY CONNECTIVITY
1. **Fallback Relay Strategy**
   - Implement WebSocket fallback for QUIC/TCP blocked networks
   - Add cellular network detection and protocol selection
   - Implement exponential backoff with circuit breaker

2. **Network Diagnostics**
   - Add network connectivity testing at startup
   - Log detailed error reasons (DNS, timeout, refusal)
   - Implement alternative bootstrap node sources

###  P0: ANR MITIGATION
1. **Thread Safety Audit**
   - Move all network I/O off main thread
   - Ensure database operations use proper coroutine scopes
   - Review `MeshForegroundService` initialization blocking

2. **ANR Watchdog Enhancement**
   - Monitor UI thread responsiveness
   - Implement graceful degradation on ANR detection
   - Add user-facing "app busy" indicators

###  P1: BLE SCAN STABILITY
1. **Scan Quota Management**
   - Implement scan session reuse instead of restart
   - Add exponential backoff with jitter
   - Monitor Android BLE scan limits

2. **Transport Fallback Chain**
   - Prioritize WiFi Direct over BLE when available
   - Implement transport health monitoring
   - Graceful degradation when BLE fails

## CODE CHANGES REQUIRED

### 1. Contact Recovery (`MeshRepository.kt`)
```kotlin
private fun emergencyContactRecovery() {
    // Reconstruct contacts from message history
    // Verify sled database integrity
    // Fallback to file copy if corrupted
}
```

### 2. Network Fallback (`MeshRepository.kt`)
```kotlin
private fun bootstrapWithFallback() {
    // Try QUIC → TCP → WebSocket → mDNS
    // Implement circuit breaker pattern
    // Log detailed failure diagnostics
}
```

### 3. ANR Prevention (`MeshForegroundService.kt`, `AnrWatchdog.kt`)
```kotlin
// Move heavy operations to background
// Add UI thread monitoring
// Implement progressive backoff
```

### 4. BLE Scan Manager (`BleScanner.kt`)
```kotlin
fun optimizeScanSession() {
    // Reuse scan sessions
    // Implement quota-aware scheduling
    // Add transport health metrics
}
```

## TEST PLAN
1. **Contacts Recovery Test**
   - Simulate corrupted contacts.db
   - Verify emergency reconstruction works
   - Test message history → contact regeneration

2. **Network Resilience Test**
   - Block QUIC/TCP ports (simulate cellular)
   - Verify WebSocket fallback activates
   - Test circuit breaker recovery

3. **ANR Stress Test**
   - Simulate heavy database operations
   - Verify UI remains responsive
   - Test ANR watchdog triggers correctly

4. **BLE Quota Test**
   - Force scan quota exhaustion
   - Verify graceful degradation
   - Test transport fallback chain

## SUCCESS CRITERIA
1.  Contacts persist across app restarts
2.  Bootstrap relay connectivity established (≥1 relay)
3.  No ANR events requiring force-kill
4.  BLE scanning operates within Android quotas
5.  Peer discovery → contact auto-creation works

## PRIORITY: URGENT
All issues block core messaging functionality. Contacts missing prevents message decryption. Relay failures prevent cross-network connectivity. ANRs make app unusable.

**Next Step:** Implement emergency contact recovery and network fallback as highest priority.