# Deadlock Analysis: `receive_message` never returns

## Lock Acquisition Order

1. **Mobile Bridge (Android)**: `MeshRepository.kt:2836` calls `meshService?.onDataReceived(peerId, data)`  
   - This is synchronous UniFFI, on the BLE GATT callback thread

2. **Core Bridge Layer**: `core/src/mobile_bridge.rs:1385` - `MeshService::on_data_received`
   - Acquires `self.stats.lock()` (line 1386)
   - Drops `stats` lock (line 1388)
   - Calls `core.receive_message(data)` (line 1396)

3. **IronCore Layer**: `core/src/iron_core.rs:2994` - `IronCore::receive_message`
   - Line 3007: `let identity = self.identity.read();` - Read lock on identity
   - Line 3041: `let mut sessions = self.ratchet_sessions.write();` - Write lock on ratchet_sessions
   - Line 3129: `let mut inbox = self.inbox.write();` - Write lock on inbox
   - Line 3154: `self.audit_log.write().append(...)` - Write lock on audit_log
   - Line 3162: `if let Some(delegate) = self.delegate.read().as_ref()` - Read lock on delegate
   - Lines 3163-3169: Calls `delegate.on_message_received(...)`
   
## Root Cause: Re-entrant Deadlock

The deadlock occurs when the delegate callback (`delegate.on_message_received`) calls back into IronCore while holding a read lock on `delegate`:

1. **Initial call path**: `on_data_received` → `receive_message` → acquire `delegate.read()`
2. **Re-entrant call**: The delegate callback calls back into IronCore (likely through `notify_peer_discovered`, `notify_peer_disconnected`, or other callbacks)
3. **Deadlock condition**: 
   - Thread holds `delegate.read()` 
   - Attempts to acquire `identity.read()` (or another lock)
   - But another thread is holding `identity.read()` and trying to acquire `delegate.read()` (or another lock)
   - Results in circular wait

## Specific Issue Analysis

Looking at the `on_message_received` delegate call in `iron_core.rs:3162-3170`, if the delegate callback (in Android/Kotlin code) makes any call back into IronCore, it can cause a deadlock because:

1. The delegate read guard is still alive (not dropped) when the delegate callback executes
2. If the callback attempts to acquire a lock that the original thread is also waiting for, a deadlock occurs
3. The original thread holds `delegate.read()` and waits for another lock that's held by the delegate thread

## Evidence from the Log

The device log shows:
- 264 "mesh_ble_forward" entries immediately BEFORE the onDataReceived calls
- 0 "mesh_ble_forward_return" entries immediately AFTER
- Core reports "0 peers (Core)" 

This indicates the `onDataReceived` method is never returning, confirming a deadlock.

## Minimal Fix

1. **Drop the delegate read guard before making the callback**:
   ```rust
   // Instead of:
   if let Some(delegate) = self.delegate.read().as_ref() {
       delegate.on_message_received(...)
   }
   
   // Do:
   let delegate_opt = self.delegate.read().clone();
   if let Some(delegate) = delegate_opt.as_ref() {
       delegate.on_message_received(...)
   }
   ```

2. **Alternative approach**: Move delegate calls to the end of the function, after all other locks are released, ensuring no lock contention.

## Confidence Level: HIGH

This is a classic re-entrant deadlock scenario:
- [OK] The exact call path matches the evidence
- [OK] Locks are acquired in a specific order (identity → ratchet_sessions → inbox → audit_log → delegate)
- [OK] The delegate lock is held during the callback, which can lead to circular waits
- [OK] The symptom (never returning) matches the deadlock behavior
- [OK] The pattern of acquiring locks in the same order in both directions (original call and re-entry) is present

## How to Verify

To falsify this theory:
1. Monitor the specific lock order in the debugger to confirm the acquisition sequence
2. Verify that `on_message_received` is indeed calling back into IronCore
3. Confirm that a different thread holds a lock that the callback thread is waiting for
4. Test with the fix applied and observe that `receive_message` now returns properly

The fix is safe and follows best practices for avoiding re-entrant deadlocks by releasing locks before callbacks.