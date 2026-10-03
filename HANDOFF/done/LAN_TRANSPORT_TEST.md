# CLIAndroid Native LAN Transport Test Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date:** 2026-04-23
**Task:** P0_TRANSPORT_001_CLI_Android_LAN_Unification
**Status:** IN_PROGRESS

## Summary of Fixes Applied

| Fix | Description | Status |
|-----|-------------|--------|
| 1 | API Server (port 9876) | Already implemented in `cmd_start()` |
| 2 | Address refresh via Identify | Added periodic refresh loop in CLI |
| 3 | LAN Discovery (mDNS) | Android already uses mDNS |
| 4 | Android static port 9001 | Changed from `/ip4/0.0.0.0/tcp/0` to `/ip4/0.0.0.0/tcp/9001` |
| 5 | CLIAndroid message pipeline | Outbox mechanism with retry logic |

## Test Environment

### CLI (Windows/Mac/Linux)
- Operating System: Windows 11 Pro
- Rust version: stable
- Data directory: `~/.scmessenger` or `%APPDATA%\SCMessenger`

### Android
- Device: Google Pixel 6a (or Android emulator)
- OS: Android 13+
- Location: Same LAN as CLI (`192.168.0.x`)

## Test Procedure

### Test 1: Verify Static Port Configuration

**Objective:** Confirm Android uses port 9001 instead of ephemeral

**Steps:**
1. Install Android app
2. Start the app and initialize
3. Check the listening addresses:
   ```bash
   adb shell am broadcast -n com.scmessenger.android/.service.MeshService \
     -e action "GET_LISTENERS"
   ```
4. Verify one of the listeners shows port 9001

**Expected Result:**
```
Listeners: ["/ip4/0.0.0.0/tcp/9001", ...]
```

**Pass Criteria:**
- At least one listener on port 9001
- No ephemeral ports (ports above 49152)

---

### Test 2: CLI API Server Test

**Objective:** Verify API server starts on port 9876

**Steps:**
1. Start CLI daemon:
   ```bash
   scm start
   ```
2. In another terminal, test API:
   ```bash
   curl http://127.0.0.1:9876/api/status
   ```

**Expected Result:**
```json
{
  "peers": [],
  "listeners": ["/ip4/0.0.0.0/tcp/9001"],
  "connection_path_state": "Bootstrapping"
}
```

**Pass Criteria:**
- Response is valid JSON
- Status code is 200
- `connection_path_state` shows valid state

---

### Test 3: AndroidCLI Message Delivery

**Objective:** Send message from Android to CLI on same LAN

**Steps:**
1. Start CLI daemon on `192.168.0.x`
2. Start Android app
3. Add CLI as contact using CLI's Peer ID (Network):
   ```bash
   # Get CLI's Peer ID
   scm identity | grep "Peer ID"
   ```
4. Send message from Android to CLI
5. Check CLI console for message receipt

**Expected Result:**
```
 [CLI Contact Name]: [Message content]
```

**Pass Criteria:**
- Message delivered within 30 seconds
- CLI console shows received message

---

### Test 4: CLIAndroid Message Delivery

**Objective:** Send message from CLI to Android on same LAN

**Steps:**
1. Start CLI daemon
2. Start Android app
3. Ensure Android has CLI as contact
4. Send message from CLI:
   ```bash
   scm send <android_peer_id> "Hello from CLI"
   ```
5. Check Android UI for message

**Expected Result:**
- Message appears in Android chat UI
- Notification triggered

**Pass Criteria:**
- Message delivered within 30 seconds

---

### Test 5: Address Staleness Recovery

**Objective:** Verify address refresh works when Android restarts with new port

**Steps:**
1. Start CLI daemon
2. Start Android app (gets port 9001)
3. Verify CLI can send to Android
4. Kill and restart Android app (may get new port)
5. Wait for Identify protocol update
6. Send message from CLI to Android

**Expected Result:**
- Message delivered despite port change
- No "connection refused" errors

**Pass Criteria:**
- Delivery succeeds within 60 seconds of Android restart
- No manual re-addition of contact needed

---

### Test 6: Outbox Retry Mechanism

**Objective:** Verify messages retry delivery when peer is temporarily unavailable

**Steps:**
1. Start CLI daemon
2. Send message from CLI to Android:
   ```bash
   scm send <android_peer_id> "Test message 1"
   ```
3. While Android is offline, queue more messages:
   ```bash
   scm send <android_peer_id> "Test message 2"
   scm send <android_peer_id> "Test message 3"
   ```
4. Restart Android app
5. Wait for reconnection

**Expected Result:**
- All 3 messages delivered upon reconnection
- No message loss

**Pass Criteria:**
- `scm status` shows pending messages before restart
- `scm status` shows 0 pending after all messages delivered

---

## Acceptance Criteria

| Criteria | Status |
|----------|--------|
| CLI `scm send` works while daemon is running |  |
| Android app receives messages from CLI peer on same LAN |  |
| No API "expected value at line 1 column 1" errors |  |
| Ledger addresses refresh before dialing |  |
| Both sides show 1 connected peer |  |
| Messages delivered within 30s on same LAN |  |
| Address stale after restart recovered automatically |  |

## Known Issues / Notes

- mDNS on Windows may require additional configuration
- Android Doze mode may delay connections
- Firewall settings must allow port 9001 on both sides

## Debug Commands

### CLI Debug
```bash
# Check API status
curl http://127.0.0.1:9876/api/peers
curl http://127.0.0.1:9876/api/listeners

# Check connections
scm status

# View logs
cat ~/.scmessenger/logs/scm.log | tail -100
```

### Android Debug
```bash
# Check mesh service status
adb shell dumpsys service scm.mesh | grep -A 20 "MeshService"

# Check Bluetooth permissions
adb shell pm list permissions | grep bluetooth

# Check network status
adb shell settings get global airplane_mode_on
```

## Rollback Plan

If issues are found:
1. Revert `MeshRepository.kt` port change: `9001`  `0`
2. Disable address refresh loop in CLI
3. Re-enable ephemeral port mode for Android

## Related Files Modified

1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
   - Changed: `startSwarm("/ip4/0.0.0.0/tcp/0")`  `startSwarm("/ip4/0.0.0.0/tcp/9001")`

2. `cli/src/main.rs`
   - Changed: Added import `PeerId`
   - Added: Periodic address refresh loop (every 120s)
   - Added: Identify address reflection calls

## Sign-Off

- [ ] Test 1: Static Port Configuration
- [ ] Test 2: API Server Test
- [ ] Test 3: AndroidCLI Message
- [ ] Test 4: CLIAndroid Message
- [ ] Test 5: Address Staleness
- [ ] Test 6: Outbox Retry
- [ ] All acceptance criteria met

---

**Test completed by:** [Runner Name]
**Date:** [Completion Date]
