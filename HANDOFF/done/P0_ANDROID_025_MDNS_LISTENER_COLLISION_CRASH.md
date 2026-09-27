# MODEL: qwen3-coder-next:cloud

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

# BUDGET: 1200
# token_budget: 12000

# P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1200s (LIGHT tier)
**Phase:** v0.2.1 P0 Android stability
**Source:** Live end-to-end test 2026-06-05 21:18 PT (Overseer session, auto-discovered)
**Depends on:** none (independent of P0_024, P1_022)
**Branch:** fix/p0-android-025-mdns-listener-collision (off origin/main dd109707)
**Worktree:** create a new worktree at `E:\SCMessenger-build-p0-025\` based on origin/main
**Assignee:** worker
**Note:** Per 2026-06-05 21:22 PT user directive, future work pivots to local models. The MODEL: line above is the historical default; the actual dispatch should be against the local `scm-coder:7b` or `scm-thinker:14b` model unless the user explicitly approves a cloud call.

---

# P0  Android: mDNS "listener already in use" crash on Android<->Windows discovery

**Status:** OPEN  NEWLY DISCOVERED
**Severity:** P0 (crash on any discovered mDNS peer; blocks Android<->Windows LAN discovery)
**Reported by:** Overseer session, 2026-06-05 21:19 PT (auto-discovered during end-to-end test)
**Detected in:** v0.2.3-debug on Pixel 6a (installed 21:18 PT from `E:\SCMessenger-build-p0-024\android\app\build\outputs\apk\debug\app-debug.apk`)
**Reporter's claim:** "Android app crashes when it discovers the Windows CLI over mDNS."

## Symptom (verified, logcat-attached)

After `adb install` of the v0.2.3-debug build, the app launched and successfully:
- Started mDNS discovery (mDNS advertiser on `_p2p._udp.local`).
- Received the Windows CLI relay's mDNS broadcast (Peer ID `12D3KooWFjyBaagUcyuweT26YVoAUtyM1u2K8YnKRgkMJ59zY8fD` at `192.168.0.230:9101`).
- Attempted to resolve the service via `NsdManager.resolveService(...)`.

It then **crashed** with a `FATAL EXCEPTION: ConnectivityThread` and `java.lang.IllegalArgumentException: listener already in use`:

```
06-05 21:19:45.462 25810 25865 E AndroidRuntime: FATAL EXCEPTION: ConnectivityThread
06-05 21:19:45.462 25810 25865 E AndroidRuntime: Process: com.scmessenger.android, PID: 25810
06-05 21:19:45.462 25810 25865 E AndroidRuntime: java.lang.IllegalArgumentException: listener already in use
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at android.net.nsd.NsdManager.putListener(NsdManager.java:1312)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at android.net.nsd.NsdManager.putListener(NsdManager.java:1295)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at android.net.nsd.NsdManager.resolveService(NsdManager.java:1781)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at com.scmessenger.android.transport.MdnsServiceDiscovery.resolveService(MdnsServiceDiscovery.kt:476)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at com.scmessenger.android.transport.MdnsServiceDiscovery.onServiceFound(MdnsServiceDiscovery.kt:106)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at com.scmessenger.android.transport.MdnsServiceDiscovery$startDiscovery$1.onServiceFound(MdnsServiceDiscovery.kt:445)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at android.net.nsd.NsdManager$ServiceHandler.lambda$handleMessage$2(NsdManager.java:1203)
```

`06-05 21:19:45.482 ActivityTaskManager: Force finishing activity com.scmessenger.android/.ui.MainActivity`
`06-05 21:19:47.855 Process com.scmessenger.android (pid 25810) has died: cch CRE`

## Root cause (confirmed in source)

`E:\SCMessenger-build-p0-024\android\app\src\main\java\com\scmessenger\android\transport\MdnsServiceDiscovery.kt:476`

```kotlin
private fun resolveService(serviceInfo: NsdServiceInfo) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
        nsdManager?.resolveService(serviceInfo, context.getMainExecutor(), getResolveListener())
    } else {
        @Suppress("DEPRECATION")
        nsdManager?.resolveService(serviceInfo, getResolveListener())
    }
}
```

`getResolveListener()` returns a **singleton** listener. `NsdManager.resolveService()` reuses it for every `onServiceFound` callback. Once the first resolve is in flight (or has not yet been detached), the second call throws `IllegalArgumentException: listener already in use`.

**This bug pre-exists my v0.2.3-debug fix set.** It was latent because the previous v0.2.3 install on the phone had not seen a peer yet, and the mDNS path on v0.2.2 was on port 9001 (which the Windows CLI's 9101 was not). The new test setup with `cli relay --listen /ip4/0.0.0.0/tcp/9101 --http-port 9102` finally produced a real `onServiceFound` callback chain and exposed the bug.

## Repro (deterministic, on this hardware)

1. `adb -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp uninstall com.scmessenger.android` (clean).
2. `adb install -r <debug-apk>` (any build with `MdnsServiceDiscovery.kt:476`).
3. `E:\SCMessenger-Github-Repo\SCMessenger\target\debug\scmessenger-cli.exe relay --listen /ip4/0.0.0.0/tcp/9101 --http-port 9102` (any Windows SCMessenger CLI on the same LAN, mDNS must reach the phone).
4. Launch the app via `am start -n com.scmessenger.android/.ui.MainActivity`.
5. Within ~30 sec, the app will crash with the listener-in-use exception when it discovers the Windows CLI.

## Files to investigate (precise)

| File | Why |
|---|---|
| `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt:476` | The crash site. `getResolveListener()` returns a singleton. |
| `MdnsServiceDiscovery.kt` (whole file, 490 lines) | The `ResolveListener` impl and its cleanup on `onResolveSucceeded` / `onResolveFailed` must detach the listener before allowing reuse. |
| `BleScanner.kt` (sibling cache) | After P1_ANDROID_022 fix, `clearPeerCache()` is the model. mDNS needs the analogous `markResolveComplete()` or listener-pool pattern. |
| `core/src/transport/` (Rust) | Not the cause  this is a pure Android NSD bug. Cross-check: does libp2p's mDNS ever trigger the same path? It does not (uses UDP multicast directly, not the Android NSD daemon). |

## Hypothesis (entry point for the subagent)

The most likely fix is one of:

1. **Per-service listener**  Create a fresh `NsdManager.ResolveListener` for each `resolveService()` call, and let Android GC it. Cheap, but loses error context. (smallest patch)
2. **Pending-resolve set + onComplete cleanup**  Track which `(serviceName, listener)` pairs are in flight, and only return the listener to the pool on `onResolveSucceeded` / `onResolveFailed`. (canonical fix)
3. **Debounce onServiceFound**  If the same `serviceName` is found twice within N seconds, ignore the duplicate. Doesn't address root cause, but masks it. (anti-pattern, do not use)

The fix is small (10-20 LoC) and lives entirely in `MdnsServiceDiscovery.kt`.

## Acceptance criteria

- [ ] App survives `onServiceFound` callbacks for at least 2 distinct mDNS peers without crashing.
- [ ] App survives the same peer re-broadcasting (the common case on Android 14+).
- [ ] No FATAL EXCEPTION in logcat during a 5-min Android<->Windows LAN session.
- [ ] The Windows CLI's `discovery peers` output shows the Android phone's peer-id within 60 sec.
- [ ] Build verification: `./gradlew :app:assembleDebug -x lint` succeeds.
- [ ] Hand off a post-mortem to `HANDOFF/STATE/`.

## Out of scope (do NOT do in this ticket)

- Refactoring the entire transport layer.
- Adding new mDNS features.
- The P0_ANDROID_024 / P1_ANDROID_022 fixes already in the worktree at `E:\SCMessenger-build-p0-024\`  they are SEPARATE and ship in a different commit.
- Cross-OS triangulation work.

## Build environment (for the subagent)

Use the verified env from `HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md`:
- `JAVA_HOME=E:\build-tools\jdk17\jdk-17.0.14`
- `ANDROID_NDK_HOME=E:\build-tools\android-sdk\ndk\26.1.10909125`
- Override: `-Pandroid.ndkVersion=26.1.10909125`
- Build dir: `E:\SCMessenger-build-p0-024\android`

## Reference

- P0_ANDROID_024 (separate): identity-generation re-entrancy
- P1_ANDROID_022 (separate): BLE stale-cache cleanup
- `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md`  build env, ticket queue
- `HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md`  verified build, env


# REPO_MAP Context for Task: P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH

**Target function: `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH`**

## core/src/transport/abstraction.rs (1 chunks, 535 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/abstraction.rs: Defines 9 types: TransportType, fmt, TransportCapabilities, TransportCapabilities, TransportEvent; 26 functions; 5 imports

### Structs/Classes
- TransportCapabilities
- TransportCommand
- TransportError
- TransportEvent
- TransportType
- fmt

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fmt` | 25 |  |
| `new` | 53 |  |
| `for_transport` | 70 |  |
| `fmt` | 144 |  |
| `fmt` | 206 |  |
| `test_transport_type_display` | 267 | for_transport, new |
| `test_transport_type_hash` | 276 | for_transport, new |
| `test_transport_capabilities_creation` | 286 | for_transport, new |
| `test_transport_capabilities_for_ble` | 296 | for_transport |
| `test_transport_capabilities_for_wifi_aware` | 306 | for_transport |
| `test_transport_capabilities_for_wifi_direct` | 314 | for_transport, PeerNotFound |
| `test_transport_capabilities_for_internet` | 322 | TransportNotAvailable, PeerNotFound, SendFailed, for_transport, ConnectionFailed |
| `test_transport_capabilities_for_local` | 329 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, PeerNotFound, SendFailed, Timeout, for_transport, ConnectionFailed |
| `test_transport_event_display` | 337 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, PeerNotFound, SendFailed, Timeout, ConnectionFailed |
| `test_transport_command_display` | 350 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, PeerNotFound, SendFailed, Timeout, ConnectionFailed |
| `test_transport_error_display` | 363 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, PeerNotFound, SendFailed, Timeout, ConnectionFailed |
| `test_transport_error_types` | 369 | InvalidPayload, Internal, TransportIoError, TransportNotAvailable, SerializationError, SendFailed, Timeout, ConnectionFailed |
| `test_transport_event_peer_discovered` | 381 |  |
| `test_transport_event_data_received` | 404 |  |
| `test_transport_command_send_data` | 427 |  |
| `test_transport_command_connect` | 450 | serialize, deserialize |
| `test_transport_command_disconnect` | 470 | serialize, deserialize |
| `test_serialization_transport_event` | 482 | serialize, for_transport, deserialize, PeerNotFound |
| `test_all_transport_types_distinct` | 506 | for_transport, PeerNotFound |
| `test_transport_capabilities_clone` | 522 | for_transport, PeerNotFound |
| `test_transport_error_clone` | 530 | PeerNotFound |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashSet`
- `use std::fmt`
- `use super::*`
- `use thiserror::Error`
---

## core/src/transport/ble/beacon.rs (1 chunks, 372 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/beacon.rs: Defines 7 types: BleBeaconError, BleBeacon, BleBeacon, BeaconBuilder, BeaconBuilder; 22 functions; 4 imports

### Structs/Classes
- BeaconBuilder
- BeaconParser
- BleBeacon
- BleBeaconError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `size` | 47 | new, now, SystemTimeError |
| `new` | 61 | from_slice, new, now, SystemTimeError |
| `with_rotation_period` | 70 | from_slice, new, now, SystemTimeError |
| `build` | 76 | now, SystemTimeError, new, CreationFailed, from_slice |
| `new` | 139 | new, now, SystemTimeError |
| `with_rotation_period` | 147 | new, now, SystemTimeError |
| `parse` | 153 | new, now, SystemTimeError |
| `is_fresh` | 159 | new, now, SystemTimeError |
| `test_beacon_builder_creates_beacon` | 176 | new |
| `test_beacon_size_matches_contents` | 188 | new |
| `test_beacon_parser_decrypts_valid_beacon` | 200 | new |
| `test_beacon_parser_rejects_wrong_key` | 217 | new |
| `test_beacon_parser_detects_fresh_beacons` | 232 | new |
| `test_beacon_parser_accepts_skewed_epochs` | 248 | new |
| `test_beacon_parser_rejects_stale_epochs` | 271 | new |
| `test_beacon_builder_with_custom_rotation_period` | 288 | new |
| `test_beacon_parser_with_custom_rotation_period` | 300 | new |
| `test_beacon_rotation_different_epochs` | 318 | new |
| `test_beacon_parser_rejects_invalid_format` | 341 | new |
| `test_beacon_service_uuid_constant` | 352 | new |
| `test_default_rotation_period_constant` | 357 | new |
| `test_beacon_builder_default_rotation` | 362 | new |

### Imports
- `use crate::transport::discovery::{decrypt_beacon_with_period, BeaconPayload}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/behaviour.rs (1 chunks, 681 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/behaviour.rs: Defines 19 types: IronCoreBehaviour, MessageRequest, MessageResponse, RelayRequest, RelayResponse; 21 functions; 9 imports

### Structs/Classes
- DeregistrationPayload
- DeregistrationRequest
- IronCoreBehaviour
- LedgerExchangeRequest
- LedgerExchangeResponse
- MessageRequest
- MessageResponse
- RegistrationMessage
- RegistrationPayload
- RegistrationRequest
- RegistrationResponse
- RelayRequest
- RelayResponse
- SharedPeerEntry

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `canonical_bytes` | 128 | verify, serialize |
| `validate_fields` | 131 | verify |
| `new_signed` | 150 | verify, serialize |
| `verify_for_public_key` | 166 | verify, serialize, parse_str |
| `canonical_bytes` | 198 | serialize, parse_str |
| `validate_fields` | 201 | parse_str |
| `new_signed` | 230 | verify |
| `verify_for_public_key` | 246 | verify, parse_str |
| `validate_identity_id` | 281 | encode, parse_str, hash |
| `validate_uuid_v4` | 293 | encode, parse_str, hash |
| `validate_signature_bytes` | 301 | hash, encode |
| `validate_identity_owner` | 308 | hash, encode |
| `new` | 367 | default, from_secs, new |
| `relay_request_carries_ws13_metadata_when_set` | 538 | generate, new_signed, from_str |
| `relay_request_missing_ws13_fields_deserialize_with_defaults` | 554 | generate, new_signed, from_str |
| `registration_payload_canonical_bytes_are_stable` | 571 | generate, new_signed |
| `signed_registration_request_verifies_against_matching_public_key` | 585 | generate, new_signed |
| `signed_registration_request_rejects_tampered_payload` | 599 | generate, new_signed |
| `signed_registration_request_rejects_malformed_identity_id` | 617 | generate, new_signed |
| `signed_deregistration_request_verifies_against_matching_public_key` | 635 | generate, new_signed |
| `signed_deregistration_request_rejects_same_source_and_target_device` | 649 | generate, new_signed |

### Imports
- `use crate::identity::IdentityKeys`
- `use libp2p::mdns`
- `use libp2p::swarm::behaviour::toggle::Toggle`
- `use libp2p::upnp`
- `use super::*`
- `use super::reflection::{AddressReflectionRequest, AddressReflectionResponse}`
- `use uuid::Uuid`
- `use web_time::Duration`
---

## android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt (2 chunks, 376 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt: Defines 1 types: BleAdvertiser; 15 functions; 14 imports android/app/src/main/java/com/scmessenger/android/transport/ble/BleAdvertiser.kt: Defines 1 types: BleAdvertiser; 15 functions; 14 imports

### Structs/Classes
- BleAdvertiser

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onStartSuccess` | 55 | w, successfully, stopAdvertising, onStartFailure, i, minOf, e |
| `onStartFailure` | 67 | w, stopAdvertising, minOf, e |
| `applyAdvertiseSettings` | 111 | d, stopAdvertising, AdvertiseCallback, onStartSuccess, startAdvertising |
| `onStartSuccess` | 139 | d, setIdentityData, stopAdvertising, onStartFailure, onStartSuccess, advertise, contentEquals, startAdvertising |
| `onStartFailure` | 142 | d, setIdentityData, stopAdvertising, onStartFailure, advertise, contentEquals, startAdvertising |
| `setIdentityData` | 151 | d, updateIdentityBeacon, setIdentityData, stopAdvertising, contentEquals, startAdvertising |
| `updateIdentityBeacon` | 175 | SuppressLint, d, setAdvertiseMode, w, startRotation, setIdentityData, setRotationInterval, Builder, startAdvertising, hasAdvertisePermission |
| `setRotationInterval` | 182 | SuppressLint, d, setAdvertiseMode, w, setTxPowerLevel, addServiceUuid, startRotation, build, Builder, ParcelUuid |
| `startAdvertising` | 192 | advertising, setAdvertiseMode, w, setTxPowerLevel, addServiceUuid, addServiceData, build, Builder, ParcelUuid, setTimeout |
| `startRotation` | 238 | setAdvertiseMode, run, w, setTxPowerLevel, addServiceUuid, stopRotation, startRotation, build, stopAdvertising, Builder |
| `run` | 244 | setAdvertiseMode, w, setTxPowerLevel, addServiceUuid, addServiceData, build, stopAdvertising, Builder, ParcelUuid, setTimeout |
| `stopRotation` | 294 | removeCallbacks, SuppressLint, catch, w, e, stopRotation, stopAdvertising, i, hasAdvertisePermission |
| `stopAdvertising` | 301 | SuppressLint, catch, w, e, stopRotation, stopAdvertising, i, sendData, hasAdvertisePermission |
| `sendData` | 327 | advertising, setAdvertiseMode, w, addServiceUuid, setTxPowerLevel, addServiceData, build, stopAdvertising, Builder, ParcelUuid |
| `hasAdvertisePermission` | 368 | checkSelfPermission, hasAdvertisePermission |
| `onStartSuccess` | 55 | onStartFailure, successfully, minOf, i, stopAdvertising, w, e |
| `onStartFailure` | 67 | minOf, stopAdvertising, w, e |
| `applyAdvertiseSettings` | 111 | d, AdvertiseCallback, onStartSuccess, stopAdvertising, startAdvertising |
| `onStartSuccess` | 139 | advertise, d, onStartFailure, onStartSuccess, contentEquals, stopAdvertising, startAdvertising, setIdentityData |
| `onStartFailure` | 142 | advertise, d, onStartFailure, contentEquals, stopAdvertising, startAdvertising, setIdentityData |
| `setIdentityData` | 151 | d, contentEquals, stopAdvertising, startAdvertising, updateIdentityBeacon, setIdentityData |
| `updateIdentityBeacon` | 175 | d, startRotation, setConnectable, hasAdvertisePermission, setRotationInterval, setAdvertiseMode, Builder, startAdvertising, SuppressLint, w |
| `setRotationInterval` | 182 | setIncludeDeviceName, d, setTimeout, startRotation, setConnectable, ParcelUuid, hasAdvertisePermission, build, setAdvertiseMode, Builder |
| `startAdvertising` | 192 | setIncludeDeviceName, setTimeout, setConnectable, ParcelUuid, hasAdvertisePermission, advertising, build, setAdvertiseMode, addServiceData, Builder |
| `startRotation` | 238 | setIncludeDeviceName, setTimeout, startRotation, setConnectable, ParcelUuid, hasAdvertisePermission, build, setAdvertiseMode, stopRotation, run |
| `run` | 244 | setIncludeDeviceName, setTimeout, setConnectable, ParcelUuid, hasAdvertisePermission, build, setAdvertiseMode, addServiceData, stopAdvertising, Builder |
| `stopRotation` | 294 | hasAdvertisePermission, catch, i, removeCallbacks, stopRotation, stopAdvertising, SuppressLint, w, e |
| `stopAdvertising` | 301 | hasAdvertisePermission, sendData, catch, i, stopRotation, stopAdvertising, SuppressLint, w, e |
| `sendData` | 327 | setIncludeDeviceName, ParcelUuid, setConnectable, advertising, hasAdvertisePermission, build, setAdvertiseMode, addServiceData, Builder, addServiceUuid |
| `hasAdvertisePermission` | 368 | hasAdvertisePermission, checkSelfPermission |

### Imports
- `import android.Manifest`
- `import android.annotation.SuppressLint`
- `import android.bluetooth.BluetoothManager`
- `import android.bluetooth.le.AdvertiseCallback`
- `import android.bluetooth.le.AdvertiseData`
- `import android.bluetooth.le.AdvertiseSettings`
- `import android.content.Context`
- `import android.content.pm.PackageManager`
- `import android.os.Build`
- `import android.os.Handler`
- `import android.os.Looper`
- `import android.os.ParcelUuid`
- `import androidx.core.content.ContextCompat`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/ble/BleBackoffStrategy.kt (2 chunks, 47 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/ble/BleBackoffStrategy.kt: Defines 1 types: BleBackoffStrategy; 3 functions; 4 imports android/app/src/main/java/com/scmessenger/android/transport/ble/BleBackoffStrategy.kt: Defines 1 types: BleBackoffStrategy; 3 functions; 4 imports

### Structs/Classes
- BleBackoffStrategy

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `nextDelay` | 23 | reset, pow, max, nextLong, minOf, jitter, nextDelay, getCurrentDelay, toDouble, toLong |
| `reset` | 40 | reset, getCurrentDelay |
| `getCurrentDelay` | 45 | getCurrentDelay |
| `nextDelay` | 23 | jitter, getCurrentDelay, pow, reset, nextDelay, toDouble, minOf, toLong, max, nextLong |
| `reset` | 40 | getCurrentDelay, reset |
| `getCurrentDelay` | 45 | getCurrentDelay |

### Imports
- `import kotlin.math.max`
- `import kotlin.math.min`
- `import kotlin.random.Random`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt (2 chunks, 872 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt: Defines 4 types: BleGattClient, GattOpGate, BleGattClientStats, ConnectionState; 21 functions; 12 imports android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattClient.kt: Defines 4 types: BleGattClient, GattOpGate, BleGattClientStats, ConnectionState; 21 functions; 12 imports

### Structs/Classes
- BleGattClient
- BleGattClientStats
- ConnectionState
- GattOpGate

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `gattQueue` | 120 | GattOpGate, catch, set, acquire, getOrPut, op, releaseGattOp, e, AtomicInteger |
| `enqueueGattOp` | 154 | trySend, compareAndSet, w, incrementAndGet, release, releaseGattOp, path, s, gattQueue |
| `releaseGattOp` | 167 | compareAndSet, release, incrementAndGet, w, catch, checkBluetoothAddress, connect, rejected, s, e |
| `connect` | 195 | checkBluetoothAddress, d, incrementAndGet, w, mismatch, full, containsKey, coerceAtLeast, currentTimeMillis |
| `disconnect` | 287 | d, catch, close, incrementAndGet, remove, disconnect, e |
| `sendData` | 322 | w, toList, connect, first |
| `reconnectAfterWriteFailure` | 428 | w, delay, fragmentData, failure, listOf, coerceAtLeast, total_fragments, connect, minOf, reconnectAfterWriteFailure |
| `fragmentData` | 441 | disconnectAll, fragmentData, fragment_index, listOf, add, coerceAtLeast, total_fragments, minOf, ByteArray, toByte |
| `disconnectAll` | 471 | d, catch, incrementAndGet, toList, requestMtu, onConnectionStateChange, BluetoothGattCallback, disconnect, e, AtomicInteger |
| `onConnectionStateChange` | 477 | d, catch, incrementAndGet, requestMtu, onConnectionStateChange, disconnect, e, AtomicInteger |
| `onServicesDiscovered` | 509 | d, incrementAndGet, w, set, readIdentityBeacon, getService, AtomicInteger, getOrPut, onServicesDiscovered, enableMessageNotifications |
| `onCharacteristicRead` | 571 | d, onIdentityReceived, delay, Supported, onCharacteristicRead, error, e |
| `onCharacteristicWrite` | 610 | get, incrementAndGet, set, decrementAndGet, v, coerceAtLeast, s, onCharacteristicWrite |
| `onDescriptorWrite` | 653 | d, w, releaseGattOp, onDescriptorWrite, onCharacteristicChanged, e, packet |
| `onCharacteristicChanged` | 669 | clear, start, w, v, onCharacteristicChanged, RX, getOrPut, toInt, or, deviceAddress |
| `onMtuChanged` | 718 | d, catch, w, discoverServices, disconnect, getClientStats, requestConnectionPriority, onMtuChanged, e |
| `getClientStats` | 747 | get, readCharacteristic, catch, readIdentityBeacon, e, releaseGattOp, getService, getClientStats, getCharacteristic, enqueueGattOp |
| `readIdentityBeacon` | 763 | readCharacteristic, catch, delay, readIdentityBeacon, releaseGattOp, getService, getCharacteristic, ordering, enqueueGattOp, e |
| `scheduleIdentityRefreshReads` | 784 | delay, readIdentityBeacon, enqueueGattOp, releaseGattOp, getService, getCharacteristic, routing, enableMessageNotifications, ordering, setCharacteristicNotification |
| `enableMessageNotifications` | 801 | catch, w, writeDescriptor, releaseGattOp, getCharacteristic, getService, routing, getDescriptor, enableMessageNotifications, enqueueGattOp |
| `cleanup` | 839 | cleanup, cancel, disconnectAll |
| `gattQueue` | 120 | releaseGattOp, AtomicInteger, set, op, catch, GattOpGate, acquire, getOrPut, e |
| `enqueueGattOp` | 154 | releaseGattOp, trySend, path, compareAndSet, release, incrementAndGet, w, gattQueue, s |
| `releaseGattOp` | 167 | connect, catch, compareAndSet, release, incrementAndGet, rejected, w, checkBluetoothAddress, e, s |
| `connect` | 195 | coerceAtLeast, d, currentTimeMillis, containsKey, incrementAndGet, mismatch, full, w, checkBluetoothAddress |
| `disconnect` | 287 | d, close, remove, catch, disconnect, incrementAndGet, e |
| `sendData` | 322 | connect, w, first, toList |
| `reconnectAfterWriteFailure` | 428 | fragmentData, coerceAtLeast, connect, failure, disconnect, minOf, listOf, delay, copyOfRange, toByte |
| `fragmentData` | 441 | fragmentData, coerceAtLeast, disconnectAll, add, listOf, minOf, copyOfRange, toByte, total_fragments, ByteArray |
| `disconnectAll` | 471 | d, requestMtu, AtomicInteger, catch, toList, disconnect, incrementAndGet, e, onConnectionStateChange, BluetoothGattCallback |
| `onConnectionStateChange` | 477 | d, requestMtu, AtomicInteger, catch, disconnect, incrementAndGet, e, onConnectionStateChange |
| `onServicesDiscovered` | 509 | d, enableMessageNotifications, AtomicInteger, set, w, getService, onServicesDiscovered, scheduleIdentityRefreshReads, incrementAndGet, readIdentityBeacon |
| `onCharacteristicRead` | 571 | d, onIdentityReceived, delay, error, onCharacteristicRead, Supported, e |
| `onCharacteristicWrite` | 610 | coerceAtLeast, set, decrementAndGet, onCharacteristicWrite, v, incrementAndGet, get, s |
| `onDescriptorWrite` | 653 | releaseGattOp, d, onCharacteristicChanged, packet, onDescriptorWrite, w, e |
| `onCharacteristicChanged` | 669 | toInt, packet, onCharacteristicChanged, deviceAddress, getOrPut, clear, RX, copyOfRange, v, totalFrags |
| `onMtuChanged` | 718 | d, discoverServices, catch, disconnect, getClientStats, requestConnectionPriority, w, e, onMtuChanged |
| `getClientStats` | 747 | releaseGattOp, enqueueGattOp, getCharacteristic, readCharacteristic, getService, catch, BleGattClientStats, getClientStats, readIdentityBeacon, e |
| `readIdentityBeacon` | 763 | releaseGattOp, ordering, enqueueGattOp, getCharacteristic, readCharacteristic, getService, catch, delay, scheduleIdentityRefreshReads, readIdentityBeacon |
| `scheduleIdentityRefreshReads` | 784 | routing, releaseGattOp, ordering, enableMessageNotifications, enqueueGattOp, getCharacteristic, getService, setCharacteristicNotification, delay, scheduleIdentityRefreshReads |
| `enableMessageNotifications` | 801 | routing, releaseGattOp, enableMessageNotifications, enqueueGattOp, getCharacteristic, writeDescriptor, getDescriptor, getService, catch, setCharacteristicNotification |
| `cleanup` | 839 | disconnectAll, cleanup, cancel |

### Imports
- `import android.bluetooth.*`
- `import android.content.Context`
- `import java.util.UUID`
- `import java.util.concurrent.ConcurrentHashMap`
- `import java.util.concurrent.CountDownLatch`
- `import java.util.concurrent.TimeUnit`
- `import java.util.concurrent.atomic.AtomicBoolean`
- `import java.util.concurrent.atomic.AtomicInteger`
- `import kotlinx.coroutines.*`
- `import kotlinx.coroutines.channels.Channel`
- `import kotlinx.coroutines.sync.Semaphore`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt (2 chunks, 554 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt: Defines 1 types: BleGattServer; 16 functions; 9 imports android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt: Defines 1 types: BleGattServer; 16 functions; 9 imports

### Structs/Classes
- BleGattServer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `setIdentityData` | 61 | start, d, w, set, openGattServer, getSystemService, e, BluetoothGattService |
| `start` | 65 | start, BluetoothGattCharacteristic, w, characteristic, openGattServer, getSystemService, e, BluetoothGattService |
| `stop` | 138 | clear, catch, cancelConnection, w, close, i, stop, e |
| `sendData` | 172 | w, getService, getCharacteristic, contains, sendFragmented, notifyCharacteristicChangedSafe, e |
| `getConnectedDeviceAddresses` | 219 | toList, coerceAtLeast, sendFragmented, keys, minOf, ByteArray, notifyCharacteristicChangedSafe, getConnectedDeviceAddresses, toByte, copyOfRange |
| `sendFragmented` | 223 | coerceAtLeast, sendFragmented, minOf, ByteArray, notifyCharacteristicChangedSafe, toByte, copyOfRange |
| `onConnectionStateChange` | 262 | onCharacteristicReadRequest, d, remove, onConnectionStateChange |
| `onCharacteristicReadRequest` | 284 | onCharacteristicReadRequest, sendResponseSafe, ByteArray, copyOfRange, toByteArray |
| `onCharacteristicWriteRequest` | 335 | d, onDataReceived, sendResponseSafe, message, handleReassembly, onCharacteristicWriteRequest |
| `onDescriptorWriteRequest` | 382 | d, sendResponseSafe, add, getOrPut, remove, newKeySet, contentEquals, onDescriptorWriteRequest |
| `handleReassembly` | 425 | clear, start, w, v, getOrPut, mutableMapOf, arraycopy, ByteArray, toInt, handleReassembly |
| `onMtuChanged` | 464 | d, onDataReceived, hasBluetoothConnectPermission, sendResponseSafe, checkSelfPermission, remove, onMtuChanged, onExecuteWrite |
| `onExecuteWrite` | 470 | d, onDataReceived, hasBluetoothConnectPermission, sendResponseSafe, checkSelfPermission, remove, onExecuteWrite |
| `hasBluetoothConnectPermission` | 491 | catch, hasBluetoothConnectPermission, w, sendResponseSafe, checkSelfPermission, e, notifyCharacteristicChangedSafe, sendResponse |
| `sendResponseSafe` | 499 | catch, hasBluetoothConnectPermission, w, sendResponseSafe, notifyCharacteristicChanged, e, notifyCharacteristicChangedSafe, sendResponse |
| `notifyCharacteristicChangedSafe` | 517 | catch, hasBluetoothConnectPermission, w, fromString, notifyCharacteristicChanged, UUID, notifyCharacteristicChangedSafe, e, Descriptor |
| `setIdentityData` | 61 | d, set, BluetoothGattService, getSystemService, openGattServer, start, w, e |
| `start` | 65 | characteristic, BluetoothGattService, BluetoothGattCharacteristic, getSystemService, openGattServer, start, w, e |
| `stop` | 138 | close, catch, clear, cancelConnection, i, w, e, stop |
| `sendData` | 172 | sendFragmented, getCharacteristic, getService, w, e, notifyCharacteristicChangedSafe, contains |
| `getConnectedDeviceAddresses` | 219 | coerceAtLeast, sendFragmented, keys, getConnectedDeviceAddresses, minOf, copyOfRange, toByte, ByteArray, notifyCharacteristicChangedSafe, toList |
| `sendFragmented` | 223 | coerceAtLeast, sendFragmented, minOf, copyOfRange, toByte, ByteArray, notifyCharacteristicChangedSafe |
| `onConnectionStateChange` | 262 | d, remove, onCharacteristicReadRequest, onConnectionStateChange |
| `onCharacteristicReadRequest` | 284 | ByteArray, toByteArray, copyOfRange, onCharacteristicReadRequest, sendResponseSafe |
| `onCharacteristicWriteRequest` | 335 | d, onDataReceived, handleReassembly, message, onCharacteristicWriteRequest, sendResponseSafe |
| `onDescriptorWriteRequest` | 382 | newKeySet, d, add, remove, onDescriptorWriteRequest, contentEquals, getOrPut, sendResponseSafe |
| `handleReassembly` | 425 | toInt, packet, deviceAddress, ByteArray, mutableMapOf, arraycopy, getOrPut, clear, handleReassembly, copyOfRange |
| `onMtuChanged` | 464 | hasBluetoothConnectPermission, d, onDataReceived, onExecuteWrite, remove, checkSelfPermission, sendResponseSafe, onMtuChanged |
| `onExecuteWrite` | 470 | hasBluetoothConnectPermission, d, onExecuteWrite, onDataReceived, remove, checkSelfPermission, sendResponseSafe |
| `hasBluetoothConnectPermission` | 491 | hasBluetoothConnectPermission, catch, notifyCharacteristicChangedSafe, checkSelfPermission, w, sendResponseSafe, e, sendResponse |
| `sendResponseSafe` | 499 | hasBluetoothConnectPermission, notifyCharacteristicChanged, catch, notifyCharacteristicChangedSafe, w, sendResponseSafe, e, sendResponse |
| `notifyCharacteristicChangedSafe` | 517 | hasBluetoothConnectPermission, Descriptor, notifyCharacteristicChanged, fromString, catch, UUID, w, notifyCharacteristicChangedSafe, e |

### Imports
- `import android.Manifest`
- `import android.bluetooth.*`
- `import android.content.Context`
- `import android.content.pm.PackageManager`
- `import android.os.Build`
- `import androidx.core.content.ContextCompat`
- `import java.util.UUID`
- `import java.util.concurrent.ConcurrentHashMap`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/ble/BleL2capManager.kt (2 chunks, 286 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/ble/BleL2capManager.kt: Defines 1 types: BleL2capManager; 11 functions; 9 imports android/app/src/main/java/com/scmessenger/android/transport/ble/BleL2capManager.kt: Defines 1 types: BleL2capManager; 11 functions; 9 imports

### Structs/Classes
- BleL2capManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `isSupported` | 45 | w, device, startListening, isSupported, i, listenUsingInsecureL2capChannel, e |
| `startListening` | 52 | w, device, accept, isSupported, handleIncomingConnection, i, listenUsingInsecureL2capChannel, e |
| `stopListening` | 103 | catch, d, close, w, containsKey, isSupported, connect, i, e |
| `connect` | 123 | d, w, containsKey, L2capConnection, startReading, isSupported, connect, getRemoteDevice, e, createInsecureL2capChannel |
| `disconnect` | 170 | d, cancel, close, w, toList, remove, shutdown, stopListening, handleIncomingConnection, send |
| `sendData` | 179 | d, cancel, w, containsKey, close, toList, handleIncomingConnection, shutdown, stopListening, send |
| `shutdown` | 191 | d, cancel, w, containsKey, close, toList, L2capConnection, startReading, handleIncomingConnection, stopListening |
| `handleIncomingConnection` | 199 | d, w, containsKey, close, L2capConnection, startReading, handleIncomingConnection |
| `startReading` | 227 | d, onDataReceived, catch, close, startReading, ByteArray, read, e, copyOfRange |
| `send` | 259 | d, catch, close, flush, w, write, remove, send, synchronized, e |
| `close` | 273 | catch, close, remove, w |
| `isSupported` | 45 | startListening, listenUsingInsecureL2capChannel, device, i, w, e, isSupported |
| `startListening` | 52 | accept, handleIncomingConnection, listenUsingInsecureL2capChannel, device, i, w, e, isSupported |
| `stopListening` | 103 | d, connect, close, catch, i, containsKey, w, e, isSupported |
| `connect` | 123 | d, connect, getRemoteDevice, createInsecureL2capChannel, L2capConnection, startReading, containsKey, w, e, isSupported |
| `disconnect` | 170 | d, handleIncomingConnection, close, sendData, remove, cancel, disconnect, send, stopListening, w |
| `sendData` | 179 | handleIncomingConnection, d, close, cancel, disconnect, send, containsKey, stopListening, w, shutdown |
| `shutdown` | 191 | handleIncomingConnection, d, close, disconnect, L2capConnection, startReading, containsKey, stopListening, w, cancel |
| `handleIncomingConnection` | 199 | handleIncomingConnection, d, close, L2capConnection, startReading, containsKey, w |
| `startReading` | 227 | d, onDataReceived, read, close, catch, startReading, copyOfRange, ByteArray, e |
| `send` | 259 | d, close, remove, catch, synchronized, send, write, flush, w, e |
| `close` | 273 | w, catch, remove, close |

### Imports
- `import android.annotation.TargetApi`
- `import android.bluetooth.*`
- `import android.content.Context`
- `import android.os.Build`
- `import java.io.InputStream`
- `import java.io.OutputStream`
- `import java.util.concurrent.ConcurrentHashMap`
- `import kotlinx.coroutines.*`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt (2 chunks, 65 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt: Defines 1 types: BleQuotaManager; 4 functions; 1 imports android/app/src/main/java/com/scmessenger/android/transport/ble/BleQuotaManager.kt: Defines 1 types: BleQuotaManager; 4 functions; 1 imports

### Structs/Classes
- BleQuotaManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `checkQuota` | 22 | exhausted, w, recordScanStart, currentCount, maxOf, currentTimeMillis, addLast, pruneOldTimestamps, first |
| `recordScanStart` | 40 | currentCount, addLast, currentTimeMillis, pruneOldTimestamps, removeFirst, isNotEmpty, first |
| `currentCount` | 50 | currentTimeMillis, pruneOldTimestamps, removeFirst, isNotEmpty, first |
| `pruneOldTimestamps` | 54 | pruneOldTimestamps, removeFirst, isNotEmpty, first |
| `checkQuota` | 22 | recordScanStart, currentCount, currentTimeMillis, maxOf, pruneOldTimestamps, exhausted, addLast, w, first |
| `recordScanStart` | 40 | currentCount, isNotEmpty, currentTimeMillis, removeFirst, pruneOldTimestamps, addLast, first |
| `currentCount` | 50 | isNotEmpty, currentTimeMillis, removeFirst, pruneOldTimestamps, first |
| `pruneOldTimestamps` | 54 | isNotEmpty, removeFirst, pruneOldTimestamps, first |

### Imports
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt (2 chunks, 628 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt: Defines 2 types: BleScanner, BleDiscoveryStats; 25 functions; 20 imports android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt: Defines 2 types: BleScanner, BleDiscoveryStats; 25 functions; 20 imports

### Structs/Classes
- BleDiscoveryStats
- BleScanner

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onScanResult` | 107 | pruneOldPeers, incrementAndGet, set, getServiceData, matchesMeshAdvertisement, currentTimeMillis |
| `onScanFailed` | 158 | catch, incrementAndGet, w, delay, invoke, startScanning, errorCode, stopScanning, e |
| `handleScanFailure` | 210 | catch, w, postDelayed, cycle, startScanning, minOf, applyScanSettings, maxOf, setScanDutyCycle, nextDelay |
| `applyScanSettings` | 233 | d, ScanCallback, onScanFailed, minOf, maxOf, setScanDutyCycle, onScanResult, toLong |
| `onScanResult` | 244 | d, setBackgroundMode, onScanFailed, startScanning, mode, setScanDutyCycle, stopScanning, onScanResult |
| `onScanFailed` | 247 | d, setBackgroundMode, onScanFailed, startScanning, mode, setScanDutyCycle, stopScanning |
| `setScanDutyCycle` | 256 | d, setBackgroundMode, startScanning, mode, i, setScanDutyCycle, stopScanning |
| `setBackgroundMode` | 273 | SuppressLint, d, w, startScanning, restart, i, setScanDutyCycle |
| `startScanning` | 288 | d, checkQuota, w, postDelayed, restart |
| `startDutyCycle` | 374 | d, run, postDelayed, stopDutyCycle, startScanningInternal, startDutyCycle, stopScanningInternal |
| `run` | 380 | removeCallbacks, d, postDelayed, stopDutyCycle, startScanningInternal, stopScanningInternal |
| `stopDutyCycle` | 405 | removeCallbacks, setScanMode, setCallbackType, setServiceUuid, stopDutyCycle, listOf, setNumOfMatches, build, currentFilters, buildScanSettings |
| `currentFilters` | 410 | removeCallbacks, setScanMode, setCallbackType, setServiceUuid, listOf, setNumOfMatches, build, currentFilters, buildScanSettings, Builder |
| `buildScanSettings` | 422 | removeCallbacks, setScanMode, setCallbackType, get, w, setNumOfMatches, build, buildScanSettings, Builder, currentTimeMillis |
| `scheduleFallbackPromotion` | 437 | removeCallbacks, get, SuppressLint, catch, w, postDelayed, restartActiveScan, stopScan, currentTimeMillis, scheduleFallbackPromotion |
| `restartActiveScan` | 462 | SuppressLint, catch, getServiceData, trim, startScanningInternal, stopScan, currentFilters, buildScanSettings, i, matchesMeshAdvertisement |
| `matchesMeshAdvertisement` | 477 | SuppressLint, catch, getServiceData, e, startScanningInternal, v, currentFilters, buildScanSettings, matchesMeshAdvertisement, startScan |
| `startScanningInternal` | 490 | SuppressLint, catch, v, currentFilters, buildScanSettings, stopScan, startScan, e, stopScanningInternal |
| `stopScanningInternal` | 505 | catch, w, postDelayed, nextDelay, v, forceRestartScanning, startScanning, stopScan, i, e |
| `forceRestartScanning` | 521 | removeCallbacks, SuppressLint, catch, w, postDelayed, stopDutyCycle, startScanning, stopScan, i, nextDelay |
| `stopScanning` | 538 | removeCallbacks, clear, catch, d, get, BleDiscoveryStats, stopDutyCycle, clearPeerCache, stopScan, i |
| `clearPeerCache` | 557 | clear, get, d, iterator, BleDiscoveryStats, currentCount, getQuotaCount, remove, hasNext, next |
| `getDiscoveryStats` | 561 | get, iterator, BleDiscoveryStats, currentCount, getQuotaCount, remove, hasNext, next, getDiscoveryStats, pruneOldPeers |
| `getQuotaCount` | 575 | iterator, currentCount, remove, hasNext, next, pruneOldPeers |
| `pruneOldPeers` | 582 | iterator, remove, hasNext, next |
| `onScanResult` | 107 | set, matchesMeshAdvertisement, currentTimeMillis, pruneOldPeers, getServiceData, incrementAndGet |
| `onScanFailed` | 158 | stopScanning, w, catch, errorCode, delay, incrementAndGet, invoke, e, startScanning |
| `handleScanFailure` | 210 | applyScanSettings, catch, nextDelay, maxOf, minOf, postDelayed, toLong, cycle, setScanDutyCycle, w |
| `applyScanSettings` | 233 | d, maxOf, minOf, ScanCallback, toLong, onScanResult, onScanFailed, setScanDutyCycle |
| `onScanResult` | 244 | d, stopScanning, mode, onScanFailed, onScanResult, setScanDutyCycle, setBackgroundMode, startScanning |
| `onScanFailed` | 247 | d, stopScanning, mode, onScanFailed, setScanDutyCycle, setBackgroundMode, startScanning |
| `setScanDutyCycle` | 256 | d, stopScanning, mode, i, setScanDutyCycle, setBackgroundMode, startScanning |
| `setBackgroundMode` | 273 | d, i, SuppressLint, w, restart, setScanDutyCycle, startScanning |
| `startScanning` | 288 | d, checkQuota, postDelayed, w, restart |
| `startDutyCycle` | 374 | d, stopScanningInternal, postDelayed, startScanningInternal, run, stopDutyCycle, startDutyCycle |
| `run` | 380 | d, stopScanningInternal, postDelayed, removeCallbacks, startScanningInternal, stopDutyCycle |
| `stopDutyCycle` | 405 | setServiceUuid, ParcelUuid, emptyList, setCallbackType, setMatchMode, buildScanSettings, build, setScanMode, listOf, setNumOfMatches |
| `currentFilters` | 410 | setServiceUuid, ParcelUuid, emptyList, setCallbackType, scheduleFallbackPromotion, setMatchMode, buildScanSettings, build, setScanMode, listOf |
| `buildScanSettings` | 422 | setCallbackType, scheduleFallbackPromotion, setMatchMode, buildScanSettings, setScanMode, setNumOfMatches, build, currentTimeMillis, removeCallbacks, Builder |
| `scheduleFallbackPromotion` | 437 | stopScan, scheduleFallbackPromotion, currentTimeMillis, restartActiveScan, catch, postDelayed, removeCallbacks, SuppressLint, w, get |
| `restartActiveScan` | 462 | startScan, stopScan, restarted, matchesMeshAdvertisement, buildScanSettings, catch, getServiceData, trim, i, startScanningInternal |
| `matchesMeshAdvertisement` | 477 | startScan, matchesMeshAdvertisement, stopScanningInternal, buildScanSettings, catch, getServiceData, trim, v, startScanningInternal, SuppressLint |
| `startScanningInternal` | 490 | startScan, stopScan, stopScanningInternal, buildScanSettings, catch, v, SuppressLint, e, currentFilters |
| `stopScanningInternal` | 505 | stopScan, catch, nextDelay, postDelayed, i, v, forceRestartScanning, w, e, startScanning |
| `forceRestartScanning` | 521 | stopDutyCycle, stopScanning, stopScan, catch, nextDelay, postDelayed, i, removeCallbacks, SuppressLint, w |
| `stopScanning` | 538 | d, stopScan, catch, clearPeerCache, BleDiscoveryStats, getDiscoveryStats, get, i, removeCallbacks, clear |
| `clearPeerCache` | 557 | d, currentCount, next, pruneOldPeers, remove, getQuotaCount, BleDiscoveryStats, hasNext, getDiscoveryStats, iterator |
| `getDiscoveryStats` | 561 | currentCount, next, pruneOldPeers, remove, getQuotaCount, BleDiscoveryStats, hasNext, getDiscoveryStats, iterator, get |
| `getQuotaCount` | 575 | currentCount, next, pruneOldPeers, remove, hasNext, iterator |
| `pruneOldPeers` | 582 | next, remove, hasNext, iterator |

### Imports
- `import android.annotation.SuppressLint`
- `import android.bluetooth.BluetoothAdapter`
- `import android.bluetooth.BluetoothManager`
- `import android.bluetooth.le.ScanCallback`
- `import android.bluetooth.le.ScanFilter`
- `import android.bluetooth.le.ScanResult`
- `import android.bluetooth.le.ScanSettings`
- `import android.content.Context`
- `import android.os.Handler`
- `import android.os.Looper`
- `import android.os.ParcelUuid`
- `import com.scmessenger.android.utils.BackoffStrategy`
- `import java.util.UUID`
- `import java.util.concurrent.ConcurrentHashMap`
- `import java.util.concurrent.atomic.AtomicInteger`
- `import java.util.concurrent.atomic.AtomicLong`
- `import kotlinx.coroutines.*`
- `import kotlinx.coroutines.sync.Mutex`
- `import kotlinx.coroutines.sync.withLock`
- `import timber.log.Timber`
---

## core/src/relay/bootstrap.rs (1 chunks, 470 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/relay/bootstrap.rs: Defines 8 types: BootstrapError, BootstrapMethod, SeedPeer, SeedPeer, InvitePayload; 39 functions; 4 imports

### Structs/Classes
- BootstrapError
- BootstrapManager
- BootstrapMethod
- InvitePayload
- SeedPeer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 50 | now |
| `new` | 78 | to_string, serialize, now, SerializationError, deserialize |
| `with_group_key` | 95 | to_string, serialize, now, SerializationError, deserialize, from_str |
| `with_expiry` | 101 | to_string, serialize, now, SerializationError, deserialize, from_str |
| `is_valid` | 107 | to_string, serialize, now, SerializationError, new, deserialize, from_str |
| `to_bytes` | 117 | to_string, serialize, SerializationError, new, deserialize, from_str |
| `from_bytes` | 122 | to_string, SerializationError, new, deserialize, from_str |
| `to_json` | 127 | to_string, SerializationError, new, from_str |
| `from_json` | 132 | SerializationError, new, from_str |
| `new` | 151 | generate_invite, new |
| `with_seed_peers` | 161 | generate_invite, accept_invite, from_bytes, new |
| `get_seed_peers` | 167 | generate_invite, accept_invite, from_bytes, new |
| `generate_invite` | 175 | generate_invite, accept_invite, from_bytes, new |
| `generate_qr_data` | 184 | accept_invite, from_bytes, generate_invite |
| `accept_invite` | 190 | new, accept_invite, from_bytes |
| `parse_qr_data` | 203 | new, accept_invite, from_bytes |
| `get_peer_addresses` | 209 | new |
| `set_addresses` | 220 | new |
| `get_addresses` | 225 | new |
| `test_seed_peer` | 237 | new |
| `test_bootstrap_manager` | 245 | new |
| `test_seed_peer_creation` | 255 | from_bytes, new |
| `test_invite_payload_creation` | 262 | from_bytes, new, from_json |
| `test_invite_payload_with_group_key` | 275 | sleep, from_bytes, new, from_json, from_millis |
| `test_invite_payload_serialization` | 288 | sleep, from_bytes, new, from_json, from_millis |
| `test_invite_payload_json_serialization` | 303 | from_millis, sleep, new, from_json |
| `test_invite_payload_expiry` | 317 | from_millis, sleep, new |
| `test_bootstrap_manager_creation` | 330 | new |
| `test_bootstrap_manager_with_seed_peers` | 337 | new |
| `test_get_seed_peers` | 350 | from_bytes |
| `test_get_seed_peers_empty` | 360 | new, from_bytes |
| `test_generate_invite` | 368 | new, from_bytes |
| `test_generate_qr_data` | 380 | new, sleep, from_bytes, from_millis |
| `test_parse_qr_data` | 394 | from_millis, sleep, new |
| `test_accept_invite_valid` | 407 | from_millis, sleep, new |
| `test_accept_invite_expired` | 420 | from_millis, sleep, new |
| `test_set_addresses` | 436 | new |
| `test_get_peer_addresses` | 446 | new |
| `test_bootstrap_method_enum` | 459 |  |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/bootstrap.rs (1 chunks, 683 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/bootstrap.rs: Defines 6 types: BootstrapConfig, Default, BootstrapState, BootstrapNode, BootstrapManager; 37 functions; 10 imports

### Structs/Classes
- BootstrapConfig
- BootstrapManager
- BootstrapNode
- BootstrapState
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 68 | default, from_secs |
| `new` | 126 | default, new |
| `with_defaults` | 164 | default, new |
| `state` | 169 |  |
| `connected_count` | 174 |  |
| `total_nodes` | 179 | next_connectable_node |
| `relay_discovery` | 184 | next_connectable_node, record_failure |
| `relay_discovery_mut` | 189 | backoff_for_node, next_connectable_node, record_failure |
| `add_bootstrap_node` | 194 | backoff_for_node, next_connectable_node, record_failure |
| `bootstrap` | 217 | random, sleep, backoff_for_node, next_connectable_node, record_attempt, record_success, record_failure |
| `record_success` | 349 | from_secs_f64, now |
| `record_failure` | 358 | from_secs_f64, new, now |
| `record_attempt` | 365 | from_secs_f64, new, now |
| `backoff_for_node` | 374 | from_secs_f64, new |
| `next_connectable_node` | 388 | new |
| `discover_fallback_nodes` | 400 | Ws, new, Wss |
| `is_websocket_address` | 440 | Ws, from_multiaddr, Wss |
| `try_websocket_connection` | 451 | from_multiaddr |
| `circuit_breaker` | 477 | var, new |
| `get_healthy_relays` | 484 | var, new |
| `get_all_relay_stats` | 490 | var, new |
| `get_fallback_relay_addresses` | 500 | var, new |
| `reset_circuit_breakers` | 505 | var, new |
| `resolve_env_bootstrap_nodes` | 511 | var, new |
| `discover_dns_bootstrap` | 525 | new |
| `discover_local_peers` | 545 | new |
| `discover_websocket_bootstrap` | 559 | new |
| `discover_hardcoded_backup_relays` | 600 | default, from_secs, new, with_defaults |
| `test_bootstrap_config_defaults` | 631 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_creation` | 642 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_add_node` | 650 | now, new, default, from_secs, with_defaults |
| `test_bootstrap_manager_no_duplicate` | 659 | now, new, default, from_secs, with_defaults |
| `test_exponential_backoff` | 669 | default, from_secs, new, now |
| `test_env_bootstrap_override` | 688 |  |
| `test_dns_discovery` | 697 |  |
| `test_local_discovery` | 706 |  |
| `test_websocket_discovery` | 713 |  |

### Imports
- `use crate::transport::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager}`
- `use crate::transport::internet::{InternetRelay, InternetTransportError}`
- `use crate::transport::relay_health::{RelayDiscovery, RelayFallback, RelayMetrics}`
- `use crate::transport::swarm::SwarmHandle`
- `use crate::transport::websocket::{diagnose_websocket_error, WebSocketTransport}`
- `use libp2p::{Multiaddr, PeerId}`
- `use std::collections::VecDeque`
- `use super::*`
- `use tracing::{debug, info, warn}`
- `use web_time::{Duration, SystemTime}`
---

## core/src/transport/circuit_breaker.rs (1 chunks, 519 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/circuit_breaker.rs: Defines 8 types: CircuitState, CircuitBreakerConfig, Default, CircuitBreakerEntry, CircuitBreakerEntry; 28 functions; 6 imports

### Structs/Classes
- CircuitBreakerConfig
- CircuitBreakerEntry
- CircuitBreakerManager
- CircuitBreakerStats
- CircuitState
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 42 | from_secs |
| `new` | 75 | default, new |
| `new` | 101 | default, transition_to_half_open, new |
| `with_defaults` | 109 | default, transition_to_half_open, new |
| `allow_request` | 117 | transition_to_half_open, now |
| `record_success` | 160 | now |
| `record_failure` | 202 | now |
| `get_state` | 242 |  |
| `get_failure_count` | 251 |  |
| `get_last_failure_reason` | 260 | default |
| `reset` | 268 | default |
| `reset_all` | 274 | default, now |
| `get_open_circuits` | 281 | default, now |
| `get_healthy_relays` | 291 | default, now |
| `get_stats` | 301 | default, from_secs, now |
| `transition_to_half_open` | 316 | default, from_secs, now, with_defaults |
| `test_circuit_breaker_default_config` | 345 | default, from_secs, with_defaults |
| `test_circuit_starts_closed` | 354 | with_defaults |
| `test_circuit_opens_after_failures` | 361 | with_defaults |
| `test_circuit_does_not_open_before_threshold` | 377 | with_defaults |
| `test_success_resets_closed_circuit` | 389 | with_defaults |
| `test_half_open_success_closes_circuit` | 402 | with_defaults |
| `test_half_open_failure_reopens_circuit` | 425 | with_defaults |
| `test_reset_specific_relay` | 444 | with_defaults |
| `test_reset_all` | 458 | with_defaults |
| `test_get_stats` | 474 | with_defaults |
| `test_get_open_circuits` | 495 | with_defaults |
| `test_last_failure_reason` | 509 | with_defaults |

### Imports
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
- `use tracing::{debug, info, warn}`
- `use web_time::{Duration, SystemTime}`
---

## core/src/transport/diagnostics.rs (1 chunks, 177 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/diagnostics.rs: Defines 3 types: NetworkDiagnosticsReport, PeerConnectionSummary, ExtendedNetworkDiagnostics; 7 functions; 4 imports

### Structs/Classes
- ExtendedNetworkDiagnostics
- NetworkDiagnosticsReport
- PeerConnectionSummary

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `get_network_diagnostics_report` | 44 |  |
| `get_extended_network_diagnostics` | 83 | now |
| `compute_avg_latency` | 113 | to_string, now |
| `now_ms` | 127 | to_string, new, now |
| `network_diagnostics_report_serializes` | 140 | to_string, new |
| `peer_connection_summary_formats_state` | 157 | new |
| `get_network_diagnostics_report_from_empty_monitor` | 170 | new |

### Imports
- `use crate::transport::health::{ConnectionState, ConnectionStats, TransportHealthMonitor}`
- `use libp2p::PeerId`
- `use serde::{Deserialize, Serialize}`
- `use super::*`
---

## core/src/transport/discovery.rs (1 chunks, 471 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/discovery.rs: Defines 9 types: BeaconError, DiscoveryMode, Default, DiscoveryMode, DiscoveryConfig; 21 functions; 3 imports

### Structs/Classes
- BeaconError
- BeaconPayload
- Default
- DiscoveryConfig
- DiscoveryMode

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 59 |  |
| `allows_mdns` | 66 |  |
| `allows_identify` | 71 |  |
| `advertises_identify` | 76 |  |
| `default` | 93 |  |
| `new` | 104 | from_le_bytes |
| `with_advertise_protocols` | 113 | from_le_bytes |
| `with_accept_unknown_peers` | 119 | from_le_bytes |
| `to_bytes` | 138 | from_le_bytes, new, now, EncryptionError |
| `from_bytes` | 147 | from_le_bytes, new, now, EncryptionError |
| `create_encrypted_beacon` | 174 | from_slice, new, now, EncryptionError |
| `decrypt_beacon` | 230 | from_slice, new, now, EncryptionError |
| `decrypt_beacon_with_period` | 238 | now, from_bytes, new, from_slice, EncryptionError |
| `test_beacon_encrypt_decrypt_roundtrip` | 298 | from_bytes |
| `test_beacon_decrypt_wrong_key_fails` | 313 | to_string, from_bytes, from_str |
| `test_beacon_payload_serialization` | 329 | to_string, from_bytes, from_str |
| `test_discovery_config_serialization` | 345 | to_string, from_str |
| `test_discovery_mode_serialization_all_variants` | 365 | to_string, from_str |
| `test_discovery_mode_properties` | 395 | new |
| `test_discovery_config_builder` | 420 | new |
| `test_epoch_rotation_changes_beacon` | 431 |  |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/transport/escalation.rs (1 chunks, 667 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/escalation.rs: Defines 6 types: EscalationPolicy, Default, EscalationError, EscalationState, EscalationEngine; 38 functions; 9 imports

### Structs/Classes
- Default
- EscalationEngine
- EscalationError
- EscalationPolicy
- EscalationState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 29 | new |
| `new` | 70 | select_best_transport, new, find_better_transport |
| `set_capabilities` | 79 | select_best_transport, find_better_transport |
| `init_peer` | 85 | select_best_transport, find_better_transport, now |
| `should_escalate` | 111 | find_worse_transport, find_better_transport, now |
| `escalate` | 122 | find_worse_transport, now |
| `deescalate` | 143 | find_worse_transport |
| `current_transport` | 166 | for_transport, escalation_score |
| `update_available_transports` | 172 | for_transport, escalation_score |
| `select_best_transport` | 187 | for_transport, escalation_score |
| `escalation_score` | 208 | for_transport, escalation_score |
| `find_better_transport` | 249 | escalation_score |
| `find_worse_transport` | 276 | escalation_score |
| `cleanup_peer` | 303 | new |
| `all_states` | 309 | new |
| `create_peer_id` | 321 | new |
| `test_escalation_engine_creation` | 329 | new |
| `test_init_peer_empty_transports` | 335 | new |
| `test_init_peer_success` | 344 | new |
| `test_select_best_transport_high_bandwidth` | 356 | new |
| `test_select_best_transport_low_latency` | 369 | new |
| `test_select_best_transport_low_power` | 382 | new |
| `test_select_best_transport_balanced` | 395 | new |
| `test_escalation_high_bandwidth_policy` | 409 | new |
| `test_escalation_low_latency_policy` | 426 | new |
| `test_escalation_low_power_policy` | 443 | new |
| `test_escalate_to_better_transport` | 464 | new |
| `test_deescalate_to_fallback` | 482 | new |
| `test_should_escalate_true` | 498 | new |
| `test_should_escalate_false` | 519 | new |
| `test_update_available_transports` | 532 | default, new |
| `test_cleanup_peer` | 548 | default, for_transport, new |
| `test_all_states` | 564 | default, for_transport, new |
| `test_escalation_policy_default` | 581 | default, for_transport, new |
| `test_set_capabilities` | 587 | default, for_transport, new |
| `test_escalation_order_high_bandwidth` | 601 | default, new |
| `test_escalation_order_low_latency` | 624 | default, new |
| `test_escalation_order_low_power` | 647 | default, new |

### Imports
- `use crate::transport::abstraction::TransportCapabilities`
- `use crate::transport::abstraction::TransportType`
- `use parking_lot::RwLock`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
- `use tracing::debug`
---

## core/src/transport/ble/gatt.rs (1 chunks, 563 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/gatt.rs: Defines 15 types: GattCharacteristic, GattCharacteristic, GattError, GattFragmentHeader, GattFragmentHeader; 46 functions; 4 imports

### Structs/Classes
- GattCharacteristic
- GattClient
- GattError
- GattFragmentHeader
- GattFragmenter
- GattReassembler
- GattServer
- GattWriteQueue
- GattWriteRequest

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `uuid` | 32 | FragmentationError |
| `new` | 74 | FragmentationError, ReassemblyError, new, max_payload_per_write, from_le_bytes |
| `to_bytes` | 87 | FragmentationError, ReassemblyError, new, max_payload_per_write, from_le_bytes |
| `from_bytes` | 95 | FragmentationError, ReassemblyError, new, max_payload_per_write, from_le_bytes |
| `max_payload_per_write` | 112 | FragmentationError, ReassemblyError, new, from_bytes, max_payload_per_write |
| `fragment` | 117 | FragmentationError, ReassemblyError, new, from_bytes, max_payload_per_write |
| `reassemble` | 150 | ReassemblyError, from_bytes, new |
| `new` | 206 | is_full, FragmentationError, new |
| `new` | 227 | is_full, new |
| `new_default` | 235 | is_full, new |
| `is_full` | 240 | is_full |
| `is_empty` | 245 | is_full |
| `len` | 250 | is_full |
| `enqueue` | 255 | is_full |
| `dequeue` | 264 |  |
| `peek` | 269 |  |
| `clear` | 274 |  |
| `on_write` | 282 |  |
| `on_read` | 289 | from_bytes, new |
| `notify` | 292 | from_bytes, new |
| `is_enabled` | 296 | from_bytes, new |
| `write` | 302 | fragment, from_bytes, new |
| `read` | 309 | fragment, from_bytes, new |
| `subscribe` | 312 | fragment, from_bytes, new |
| `unsubscribe` | 315 | fragment, from_bytes, new |
| `is_connected` | 318 | fragment, from_bytes, new |
| `test_gatt_characteristic_uuids` | 326 | fragment, from_bytes, new |
| `test_gatt_fragment_header_roundtrip` | 333 | fragment, from_bytes, new |
| `test_gatt_fragment_header_invalid_index` | 343 | reassemble, fragment, new |
| `test_gatt_fragmenter_small_message` | 349 | reassemble, fragment, new |
| `test_gatt_fragmenter_large_message` | 358 | reassemble, fragment, new |
| `test_gatt_fragmenter_empty_message` | 371 | reassemble, fragment, new |
| `test_gatt_reassembler_single_fragment` | 380 | reassemble, fragment, new |
| `test_gatt_fragmenter_reassembler_roundtrip` | 392 | new, fragment, reassemble |
| `test_gatt_reassembler_wrong_fragment_count` | 401 | reassemble, new_default, new |
| `test_gatt_reassembler_out_of_order` | 412 | reassemble, new_default, new |
| `test_gatt_write_request_creation` | 429 | new_default, new |
| `test_gatt_write_request_max_size` | 439 | new_default, new |
| `test_gatt_write_queue_empty` | 447 | new_default, new |
| `test_gatt_write_queue_enqueue_dequeue` | 456 | new |
| `test_gatt_write_queue_backpressure` | 472 | new |
| `test_gatt_write_queue_peek` | 494 | new, max_payload_per_write |
| `test_gatt_write_queue_clear` | 510 | new, max_payload_per_write |
| `test_gatt_fragment_payload_size` | 526 | new, max_payload_per_write |
| `test_gatt_characteristic_all_variants` | 532 | new |
| `test_gatt_write_queue_fifo_order` | 545 | new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use std::collections::VecDeque`
- `use super::*`
- `use thiserror::Error`
---

## core/src/transport/health.rs (1 chunks, 765 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/health.rs: Defines 10 types: ConnectionState, ConnectionStats, ConnectionStats, TransportHealthMonitor, std; 44 functions; 8 imports

### Structs/Classes
- ConnectionState
- ConnectionStats
- Default
- GlobalTransportMetrics
- TransportHealthMonitor
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 63 | new, now |
| `update_state` | 86 | now |
| `record_message_success` | 95 | now |
| `record_message_failure` | 113 | now |
| `record_bytes_received` | 122 | now |
| `record_connection_attempt` | 131 | now |
| `record_successful_connection` | 140 | now |
| `record_connection_failure` | 149 | now |
| `update_current_address` | 158 | now |
| `is_healthy` | 170 | now |
| `quality_score` | 190 | now |
| `fmt` | 236 | new |
| `default` | 252 | new |
| `new` | 259 | new |
| `update_connection_state` | 268 | new |
| `record_message_success` | 300 | new |
| `record_message_failure` | 311 | new |
| `record_bytes_received` | 322 | new |
| `record_connection_attempt` | 333 | new |
| `record_successful_connection` | 345 |  |
| `record_connection_failure` | 356 |  |
| `update_current_address` | 367 |  |
| `get_connection_stats` | 375 | new, now |
| `get_all_connection_stats` | 381 | new, now |
| `get_global_metrics` | 387 | new, now |
| `get_healthy_connections` | 393 | new, now |
| `get_unhealthy_connections` | 403 | new, now |
| `register_state_change_callback` | 413 | new, now |
| `cleanup_stale_connections` | 422 | now |
| `default` | 472 | new, now |
| `new` | 479 | new, now |
| `record_connection_attempt` | 501 |  |
| `record_successful_connection` | 506 |  |
| `record_connection_failure` | 515 | now |
| `record_message_success` | 520 | now |
| `record_message_failure` | 526 | now |
| `record_bytes_received` | 531 | now |
| `record_connection_state_change` | 536 | now |
| `health_score` | 560 | now |
| `uptime_seconds` | 601 | generate_ed25519, new, now |
| `test_connection_stats_quality_score` | 616 | generate_ed25519, new |
| `test_connection_stats_unhealthy` | 635 | generate_ed25519, new |
| `test_transport_health_monitor` | 651 | generate_ed25519, new |
| `test_global_metrics_health_score` | 677 | new |

### Imports
- `use libp2p::identity`
- `use libp2p::{Multiaddr, PeerId}`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use std::sync::{Arc, Mutex}`
- `use super::*`
- `use tracing::{debug, error, info, warn}`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/internet.rs (1 chunks, 903 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/internet.rs: Defines 10 types: InternetTransportError, NatStatus, RelayMode, InternetTransportConfig, Default; 40 functions; 9 imports

### Structs/Classes
- Default
- InternetRelay
- InternetTransportConfig
- InternetTransportError
- NatStatus
- PeerRelayInfo
- RelayMode
- RelayStats

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 87 |  |
| `new` | 144 | new, ConfigError |
| `get_nat_status` | 166 |  |
| `set_nat_status` | 171 |  |
| `connect_to_relay` | 177 |  |
| `connect_to_relay_via_swarm` | 254 | connect_to_relay, dial, ConnectionFailed |
| `relay_for_peer` | 296 | RelayPeerNotFound, BandwidthExceeded |
| `disconnect_relay` | 349 |  |
| `register_relay_peer` | 363 |  |
| `get_peer_relay_info` | 403 | get_active_relay_count |
| `get_relay_peers` | 408 | get_active_relay_count |
| `get_relay_stats` | 413 | get_active_relay_count |
| `get_all_relay_stats` | 418 | get_active_relay_count |
| `get_active_relay_count` | 423 | get_active_relay_count |
| `can_accept_relay` | 428 | get_active_relay_count |
| `cleanup_stale_relays` | 433 | RelayPeerNotFound |
| `shutdown` | 452 | RelayPeerNotFound, Other |
| `attempt_hole_punch` | 466 | RelayPeerNotFound, Other |
| `establish_relay_circuit` | 511 | RelayPeerNotFound, Other |
| `current_unix_timestamp` | 570 | default, new, now |
| `test_internet_relay_creation` | 586 | default, random, new |
| `test_invalid_listen_port` | 593 | default, random, new |
| `test_invalid_max_connections` | 602 | default, random, new |
| `test_nat_status_update` | 611 | default, random, new |
| `test_connect_to_relay` | 623 | default, random, new |
| `test_max_relay_connections` | 641 | default, random, new |
| `test_disconnect_relay` | 661 | default, random, new |
| `test_register_relay_peer` | 676 | default, random, new |
| `test_relay_for_peer` | 692 | default, random, new |
| `test_relay_peer_not_found` | 707 | default, random, new |
| `test_client_mode_relay_fails` | 717 | default, random, new |
| `test_relay_stats` | 731 | default, random, new |
| `test_get_relay_peers` | 751 | default, random, new |
| `test_can_accept_relay` | 771 | default, random, new |
| `test_cleanup_stale_relays` | 791 | default, random, new |
| `test_relay_mode_config` | 820 | default, random, new |
| `test_nat_traversal_hole_punch` | 835 | default, random, new |
| `test_nat_traversal_relay_circuit` | 856 | default, random, new |
| `test_invalid_relay_addresses` | 877 | default, random, new |
| `test_shutdown` | 888 | default, random, new |

### Imports
- `use crate::transport::SwarmHandle`
- `use libp2p::{Multiaddr, PeerId}`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::{SystemTime, UNIX_EPOCH}`
- `use super::*`
- `use thiserror::Error`
- `use tracing::{debug, info, warn}`
---

## core/src/transport/ble/l2cap.rs (1 chunks, 568 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/l2cap.rs: Defines 15 types: ProtocolServiceMultiplexer, ProtocolServiceMultiplexer, ChannelState, L2capConfig, Default; 41 functions; 3 imports

### Structs/Classes
- ChannelState
- Default
- FragmentHeader
- L2capChannel
- L2capConfig
- L2capError
- L2capFragmenter
- L2capReassembler
- ProtocolServiceMultiplexer

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `value` | 18 |  |
| `default` | 48 | InvalidMtu |
| `new` | 59 | InvalidMtu |
| `with_mtu` | 68 | InvalidMtu |
| `with_timeout` | 74 | InvalidMtu |
| `validate` | 80 | InvalidMtu, FragmentationError |
| `new` | 128 | from_le_bytes, FragmentationError, ReassemblyError, new |
| `to_bytes` | 141 | from_le_bytes, ReassemblyError, new |
| `from_bytes` | 149 | from_le_bytes, ReassemblyError, new |
| `new` | 169 | ConnectionFailed |
| `state` | 178 | ConnectionFailed |
| `config` | 183 | ConnectionFailed |
| `is_connected` | 188 | ConnectionFailed |
| `initiate_connection` | 193 | ConnectionFailed |
| `confirm_connection` | 209 | ConnectionFailed |
| `initiate_close` | 222 | max_payload_per_fragment, new |
| `confirm_close` | 234 | max_payload_per_fragment, new, FragmentationError |
| `new` | 252 | max_payload_per_fragment, new, FragmentationError |
| `max_payload_per_fragment` | 258 | max_payload_per_fragment, new, FragmentationError |
| `fragment` | 264 | max_payload_per_fragment, from_bytes, new, FragmentationError |
| `new` | 300 | ReassemblyError, from_bytes, new |
| `reassemble` | 306 | ReassemblyError, from_bytes, new |
| `test_psm_value` | 358 | default, from_bytes, new |
| `test_l2cap_config_default` | 364 | default, from_bytes, new |
| `test_l2cap_config_builder` | 372 | default, from_bytes, new |
| `test_l2cap_config_validation_valid` | 382 | default, from_bytes, new |
| `test_l2cap_config_validation_invalid_mtu` | 388 | default, from_bytes, new |
| `test_l2cap_config_validation_invalid_timeout` | 394 | default, from_bytes, new |
| `test_fragment_header_roundtrip` | 400 | default, from_bytes, new |
| `test_fragment_header_invalid_index` | 410 | default, new |
| `test_l2cap_channel_state_machine` | 416 | default, new |
| `test_l2cap_channel_invalid_double_connect` | 438 | default, new |
| `test_l2cap_fragmenter_small_message` | 448 | default, new |
| `test_l2cap_fragmenter_large_message` | 460 | default, new |
| `test_l2cap_fragmenter_empty_message` | 476 | default, new |
| `test_l2cap_reassembler_single_fragment` | 487 | default, new |
| `test_l2cap_fragmenter_reassembler_roundtrip` | 502 | default, new |
| `test_l2cap_reassembler_wrong_fragment_count` | 515 | default, new |
| `test_l2cap_reassembler_out_of_order` | 529 | default, new |
| `test_channel_state_transitions` | 549 | default, new |
| `test_l2cap_channel_new_validates_config` | 563 | default, new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
---

## core/src/transport/manager.rs (1 chunks, 1415 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/manager.rs: Defines 12 types: TransportState, TransportState, PendingSend, OutgoingQueue, OutgoingQueue; 68 functions; 8 imports

### Structs/Classes
- Default
- OutgoingQueue
- PendingSend
- ReconnectionState
- SendResult
- TransportManager
- TransportState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 30 | Reverse, new |
| `new` | 62 | from_secs, Reverse, new |
| `enqueue` | 67 | new, from_secs, Reverse |
| `dequeue` | 75 | from_secs, new |
| `len` | 84 | from_millis, from_secs, new |
| `is_empty` | 89 | from_millis, from_secs, new |
| `clear` | 94 | from_millis, from_secs, new |
| `default` | 100 | from_millis, from_secs, new |
| `new` | 146 | from_millis, backoff_interval, now |
| `backoff_interval` | 158 | from_millis, backoff_interval, now |
| `record_failure` | 168 | backoff_interval, now |
| `is_exhausted` | 174 | new, now |
| `is_ready` | 179 | new, now |
| `new` | 221 | new, now |
| `set_health_monitor` | 235 | Vacant, new, now |
| `register_transport` | 240 | Vacant, new, now |
| `handle_event` | 251 | new, Vacant, now |
| `send_to_peer` | 325 | encode, now, PeerNotFound, Queued, best_transport_for_peer |
| `best_transport_for_peer` | 355 | min, PeerNotFound, max |
| `connected_peers` | 415 | new |
| `peers_on_transport` | 421 | new |
| `is_peer_connected` | 431 |  |
| `transports_for_peer` | 440 |  |
| `pending_sends` | 449 |  |
| `add_target_peer` | 461 | now |
| `remove_target_peer` | 467 | now |
| `peers_needing_reconnect` | 482 | now |
| `record_reconnect_success` | 524 | now |
| `record_reconnect_failure` | 530 | now |
| `reconnection_queue_len` | 552 | now |
| `tick` | 557 | now |
| `expire_address_observations` | 594 | all_connections |
| `get_healthy_connections` | 602 | all_connections, empty |
| `get_unhealthy_connections` | 611 | all_connections, empty |
| `get_all_connection_stats` | 620 | all_connections, empty |
| `get_all_observed_connections` | 632 | empty |
| `cleanup_health_stale_connections` | 658 | new |
| `disable_transport` | 668 | for_transport, new |
| `default` | 693 | for_transport, new, now |
| `create_peer_id` | 701 | for_transport, new, now |
| `test_transport_state_creation` | 709 | for_transport, new, now |
| `test_outgoing_queue_fifo_with_priority` | 717 | new, now |
| `test_outgoing_queue_len` | 750 | for_transport, new, now |
| `test_outgoing_queue_clear` | 768 | for_transport, new, now |
| `test_transport_manager_creation` | 784 | for_transport, new |
| `test_register_transport` | 790 | for_transport, new |
| `test_peer_discovered_event` | 801 | new |
| `test_peer_disconnected_event` | 819 | for_transport, new |
| `test_multiple_transports_per_peer` | 842 | for_transport, new |
| `test_best_transport_prefers_connected` | 865 | for_transport, new |
| `test_best_transport_prefers_streaming` | 902 | for_transport, new |
| `test_best_transport_prefers_low_latency` | 933 | for_transport, new |
| `test_best_transport_fails_for_unknown_peer` | 964 | for_transport, new |
| `test_is_peer_connected` | 973 | for_transport, new |
| `test_peers_on_transport` | 990 | for_transport, new |
| `test_send_to_peer_queues_data` | 1029 | Queued, for_transport, new |
| `test_pending_sends_priority_ordering` | 1053 | from_secs, for_transport, new, now |
| `test_tick_cleanup` | 1079 | from_secs, new, now |
| `test_transports_for_peer` | 1107 | new |
| `test_connected_peers_deduplication` | 1132 | new |
| `test_target_peer_queued_for_reconnect_on_disconnect` | 1152 | new |
| `test_non_target_peer_not_queued_for_reconnect` | 1179 | new |
| `test_reconnection_backoff_increases` | 1202 | new |
| `test_reconnection_backoff_capped_at_max` | 1217 | new |
| `test_reconnection_exhaustion` | 1229 | new |
| `test_reconnect_success_removes_from_queue` | 1242 | new |
| `test_remove_target_peer_stops_reconnection` | 1268 | Queued, for_transport, new |
| `test_send_result_is_queued_not_delivered` | 1294 | Queued, for_transport, new |

### Imports
- `use crate::transport::health::TransportHealthMonitor`
- `use crate::transport::observation::AddressObserver`
- `use parking_lot::RwLock`
- `use std::collections::{HashMap, HashSet}`
- `use std::sync::Arc`
- `use super::*`
- `use tracing::{debug, info, warn}`
- `use web_time::{Duration, SystemTime}`
---

## android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt (2 chunks, 549 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt: Defines 1 types: MdnsServiceDiscovery; 30 functions; 8 imports android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt: Defines 1 types: MdnsServiceDiscovery; 30 functions; 8 imports

### Structs/Classes
- MdnsServiceDiscovery

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onDiscoveryStarted` | 59 | clear, resolveService, d, i, regType, onServiceFound, onDiscoveryStopped, equals |
| `onDiscoveryStopped` | 69 | clear, resolveService, d, remove, lost, i, onServiceLost, onServiceFound, equals |
| `onServiceFound` | 80 | resolveService, d, remove, onServiceRegistered, lost, i, onServiceLost, equals |
| `onServiceLost` | 94 | d, Suppress, resolved, onServiceRegistered, remove, i, onServiceResolved |
| `onServiceRegistered` | 109 | d, Suppress, resolved, String, i, onServiceResolved, attributes |
| `onServiceResolved` | 120 | d, Suppress, String, onPeerDiscovered, isNullOrBlank, attributes |
| `onServiceUnregistered` | 178 | d, postDelayed, startDiscovery, failure, errorCode, i, onStartDiscoveryFailed, shl, e |
| `onStartDiscoveryFailed` | 188 | d, postDelayed, startDiscovery, failure, errorCode, onStopDiscoveryFailed, shl, e |
| `onStopDiscoveryFailed` | 211 | registration, d, postDelayed, startDiscovery, onRegistrationFailed, errorCode, registerService, shl, e |
| `onRegistrationFailed` | 229 | registration, d, postDelayed, errorCode, registerService, shl, e, onResolveFailed |
| `onResolveFailed` | 252 | start, w, onUnregistrationFailed, e |
| `onUnregistrationFailed` | 264 | start, w, getSystemService, registerService, e |
| `start` | 276 | catch, w, startDiscovery, getSystemService, registerService, i, e |
| `stop` | 308 | stopServiceDiscovery, clear, catch, unregisterService, i, e |
| `registerService` | 340 | onServiceUnregistered, setAttribute, onServiceRegistered, onRegistrationFailed, onUnregistrationFailed, NsdServiceInfo |
| `onRegistrationFailed` | 355 | startDiscovery, onDiscoveryStarted, onServiceUnregistered, onServiceRegistered, onRegistrationFailed, onUnregistrationFailed, onDiscoveryStopped, registerService |
| `onUnregistrationFailed` | 358 | startDiscovery, onDiscoveryStarted, onServiceUnregistered, onServiceRegistered, onServiceFound, onUnregistrationFailed, onDiscoveryStopped, registerService |
| `onServiceRegistered` | 362 | startDiscovery, onDiscoveryStarted, onServiceUnregistered, onServiceRegistered, onServiceLost, onServiceFound, onDiscoveryStopped, registerService |
| `onServiceUnregistered` | 366 | startDiscovery, onDiscoveryStarted, onServiceUnregistered, onStartDiscoveryFailed, onServiceLost, onServiceFound, onDiscoveryStopped, registerService |
| `startDiscovery` | 378 | discoverServices, onDiscoveryStarted, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceFound, onDiscoveryStopped |
| `onDiscoveryStarted` | 380 | discoverServices, onDiscoveryStarted, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceFound, onDiscoveryStopped |
| `onDiscoveryStopped` | 383 | discoverServices, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceFound, onDiscoveryStopped |
| `onServiceFound` | 387 | resolveService, discoverServices, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceFound, onResolveFailed |
| `onServiceLost` | 391 | resolveService, discoverServices, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceLost, onServiceResolved, onResolveFailed |
| `onStartDiscoveryFailed` | 395 | resolveService, discoverServices, onStopDiscoveryFailed, onStartDiscoveryFailed, onServiceResolved, onResolveFailed |
| `onStopDiscoveryFailed` | 399 | resolveService, discoverServices, Suppress, onStopDiscoveryFailed, onServiceResolved, onResolveFailed |
| `resolveService` | 415 | cleanup, resolveService, Suppress, stop, onServiceResolved, onResolveFailed |
| `onResolveFailed` | 417 | cleanup, resolveService, Suppress, stop, onServiceResolved, onResolveFailed |
| `onServiceResolved` | 420 | cleanup, resolveService, Suppress, stop, onServiceResolved |
| `cleanup` | 433 | stop |
| `onDiscoveryStarted` | 59 | regType, d, onDiscoveryStopped, resolveService, i, equals, onServiceFound, clear |
| `onDiscoveryStopped` | 69 | d, remove, resolveService, lost, i, equals, onServiceFound, clear, onServiceLost |
| `onServiceFound` | 80 | d, remove, resolveService, lost, i, equals, onServiceLost, onServiceRegistered |
| `onServiceLost` | 94 | d, remove, resolved, onServiceResolved, i, Suppress, onServiceRegistered |
| `onServiceRegistered` | 109 | d, onServiceResolved, resolved, String, i, Suppress, attributes |
| `onServiceResolved` | 120 | d, String, Suppress, isNullOrBlank, attributes, onPeerDiscovered |
| `onServiceUnregistered` | 178 | d, failure, errorCode, postDelayed, i, startDiscovery, onStartDiscoveryFailed, e, shl |
| `onStartDiscoveryFailed` | 188 | onStopDiscoveryFailed, d, failure, errorCode, postDelayed, startDiscovery, e, shl |
| `onStopDiscoveryFailed` | 211 | d, registration, onRegistrationFailed, errorCode, postDelayed, startDiscovery, registerService, e, shl |
| `onRegistrationFailed` | 229 | d, onResolveFailed, registration, errorCode, postDelayed, registerService, e, shl |
| `onResolveFailed` | 252 | onUnregistrationFailed, start, w, e |
| `onUnregistrationFailed` | 264 | registerService, getSystemService, start, w, e |
| `start` | 276 | w, catch, getSystemService, i, startDiscovery, registerService, e |
| `stop` | 308 | unregisterService, catch, i, stopServiceDiscovery, clear, e |
| `registerService` | 340 | setAttribute, onServiceUnregistered, onRegistrationFailed, onUnregistrationFailed, NsdServiceInfo, onServiceRegistered |
| `onRegistrationFailed` | 355 | onServiceUnregistered, onRegistrationFailed, onDiscoveryStopped, startDiscovery, onUnregistrationFailed, registerService, onDiscoveryStarted, onServiceRegistered |
| `onUnregistrationFailed` | 358 | onServiceUnregistered, onDiscoveryStopped, startDiscovery, onUnregistrationFailed, onServiceFound, registerService, onDiscoveryStarted, onServiceRegistered |
| `onServiceRegistered` | 362 | onServiceUnregistered, onDiscoveryStopped, startDiscovery, onServiceFound, registerService, onServiceLost, onDiscoveryStarted, onServiceRegistered |
| `onServiceUnregistered` | 366 | onServiceUnregistered, onDiscoveryStopped, startDiscovery, onServiceFound, onStartDiscoveryFailed, registerService, onServiceLost, onDiscoveryStarted |
| `startDiscovery` | 378 | onStopDiscoveryFailed, discoverServices, onDiscoveryStopped, onServiceFound, onStartDiscoveryFailed, onServiceLost, onDiscoveryStarted |
| `onDiscoveryStarted` | 380 | onStopDiscoveryFailed, discoverServices, onDiscoveryStopped, onServiceFound, onStartDiscoveryFailed, onServiceLost, onDiscoveryStarted |
| `onDiscoveryStopped` | 383 | discoverServices, onDiscoveryStopped, onServiceFound, onStartDiscoveryFailed, onServiceLost, onStopDiscoveryFailed |
| `onServiceFound` | 387 | onResolveFailed, discoverServices, resolveService, onServiceFound, onStartDiscoveryFailed, onServiceLost, onStopDiscoveryFailed |
| `onServiceLost` | 391 | onResolveFailed, discoverServices, onServiceResolved, resolveService, onStartDiscoveryFailed, onServiceLost, onStopDiscoveryFailed |
| `onStartDiscoveryFailed` | 395 | onResolveFailed, discoverServices, onServiceResolved, resolveService, onStartDiscoveryFailed, onStopDiscoveryFailed |
| `onStopDiscoveryFailed` | 399 | onResolveFailed, discoverServices, onServiceResolved, resolveService, Suppress, onStopDiscoveryFailed |
| `resolveService` | 415 | onResolveFailed, cleanup, onServiceResolved, resolveService, Suppress, stop |
| `onResolveFailed` | 417 | onResolveFailed, cleanup, onServiceResolved, resolveService, Suppress, stop |
| `onServiceResolved` | 420 | cleanup, onServiceResolved, resolveService, Suppress, stop |
| `cleanup` | 433 | stop |

### Imports
- `import android.content.Context`
- `import android.net.nsd.NsdManager`
- `import android.net.nsd.NsdServiceInfo`
- `import android.os.Build`
- `import android.os.Handler`
- `import android.os.Looper`
- `import java.util.concurrent.ConcurrentHashMap`
- `import timber.log.Timber`
---

## core/src/transport/mesh_routing.rs (1 chunks, 699 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/mesh_routing.rs: Defines 20 types: RankedRoute, RouteCursorAdvance, RelayStats, RelayReputation, RelayReputation; 41 functions; 5 imports

### Structs/Classes
- BootstrapCapability
- Default
- DeliveryAttempt
- MultiPathDelivery
- RankedRoute
- RelayCandidate
- RelayReputation
- RelayStats
- ReputationTracker
- RetryStrategy
- RouteCursorAdvance

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `advance_route_cursor` | 43 | now |
| `unix_now_secs` | 64 | now |
| `calculate_score` | 112 | now |
| `default` | 166 | default, new, now |
| `new` | 172 | default, new, now |
| `record_relay_attempt` | 179 | default, now |
| `best_relays` | 214 | default |
| `add_relay` | 231 | default, from_millis, from_secs |
| `is_empty` | 243 | from_millis, from_secs |
| `all_reputations` | 248 | from_millis, from_secs |
| `default` | 274 | from_millis, from_secs |
| `calculate_delay` | 287 | default, from_millis, new, now |
| `should_retry` | 301 | default, new, now |
| `new` | 324 | default, new, now |
| `next_retry_delay` | 339 | new, now |
| `should_retry` | 344 | new, now |
| `record_failure` | 349 | new, now |
| `default` | 375 | record_recipient_seen_via_relay, new |
| `new` | 381 | with_capacity, record_recipient_seen_via_relay, new |
| `start_delivery` | 392 | with_capacity, record_recipient_seen_via_relay, new |
| `add_relay` | 398 | with_capacity, record_recipient_seen_via_relay, new |
| `record_recipient_seen_via_relay` | 405 | with_capacity, record_recipient_seen_via_relay, new |
| `record_recipient_seen_now` | 417 | with_capacity, record_recipient_seen_via_relay, new |
| `ranked_routes` | 422 | with_capacity, new |
| `get_best_paths` | 502 | ranked_routes |
| `record_success` | 510 |  |
| `record_failure` | 529 |  |
| `converge_delivery` | 546 | new |
| `delivery_attempt` | 551 | new |
| `pending_attempts` | 556 | new, now |
| `reputation` | 561 | new, now |
| `best_relays` | 566 | new, now |
| `default` | 585 | default, random, new, now |
| `new` | 591 | default, random, new, now |
| `add_peer` | 599 | default, random, now |
| `get_bootstrap_candidates` | 610 | default, from_millis, random |
| `can_bootstrap_others` | 615 | default, from_millis, random |
| `test_reputation_calculation` | 625 | default, from_millis, random, new |
| `test_retry_strategy` | 649 | default, from_millis, random, new |
| `test_multi_path_delivery` | 668 | random, new |
| `test_converge_delivery_clears_pending_retry_attempt` | 686 | random, new |

### Imports
- `use libp2p::PeerId`
- `use serde::{Deserialize, Serialize}`
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::{Duration, SystemTime, UNIX_EPOCH}`
---

## core/src/transport/multiport.rs (1 chunks, 345 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/multiport.rs: Defines 6 types: MultiPortConfig, Default, BindResult, BindAnalysis, BindAnalysis; 11 functions; 2 imports

### Structs/Classes
- BindAnalysis
- BindResult
- ConnectivityStatus
- Default
- MultiPortConfig

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 34 | new |
| `generate_listen_addresses` | 57 | new |
| `requires_elevated_privileges` | 97 | new |
| `analyze_bind_results` | 109 | new |
| `report` | 175 | new |
| `test_generate_listen_addresses_default` | 244 | default |
| `test_generate_listen_addresses_ipv4_only` | 256 |  |
| `test_generate_listen_addresses_custom_ports` | 276 |  |
| `test_requires_elevated_privileges` | 292 |  |
| `test_analyze_bind_results` | 303 |  |
| `test_bind_analysis_report` | 332 |  |

### Imports
- `use libp2p::Multiaddr`
- `use super::*`
---

## core/src/transport/nat.rs (1 chunks, 854 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/nat.rs: Defines 11 types: NatTraversalError, NatType, PeerAddressDiscovery, PeerAddressDiscovery, HolePunchAttempt; 37 functions; 10 imports

### Structs/Classes
- Default
- HolePunchAttempt
- HolePunchStatus
- NatConfig
- NatTraversal
- NatTraversalError
- NatType
- PeerAddressDiscovery
- RelayCircuit

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `with_peers` | 87 | ProbesFailed, new |
| `detect_nat_type` | 96 | ProbesFailed, new |
| `get_external_address` | 177 | StunError |
| `default` | 309 | new, with_peers, InvalidConfig |
| `new` | 338 | new, with_peers, InvalidConfig |
| `probe_nat` | 355 | with_peers, HolePunchFailed |
| `get_nat_type` | 378 | HolePunchFailed |
| `get_external_address` | 383 | HolePunchFailed |
| `start_hole_punch` | 388 | send_hole_punch_probes, HolePunchFailed |
| `send_hole_punch_probes` | 440 |  |
| `get_hole_punch_status` | 495 | RelayCircuitFailed |
| `establish_relay_circuit` | 508 | RelayCircuitFailed |
| `close_relay_circuit` | 544 |  |
| `get_active_circuits` | 560 |  |
| `get_relay_circuit` | 570 | now |
| `cleanup_old_attempts` | 581 | with_peers, now |
| `shutdown` | 599 | with_peers, now |
| `current_unix_timestamp` | 612 | default, new, with_peers, now |
| `test_peer_discovery_creation` | 628 | default, new, with_peers |
| `test_peer_discovery_no_peers` | 640 | default, new |
| `test_detect_nat_type_with_peers` | 647 | default, new |
| `test_get_external_address_from_peer` | 654 | default, new |
| `test_nat_traversal_creation` | 660 | default, random, new |
| `test_nat_traversal_invalid_config` | 667 | default, random, new |
| `test_probe_nat` | 677 | default, random, new |
| `test_hole_punch_start` | 684 | default, random, new |
| `test_hole_punch_disabled` | 691 | default, random, new |
| `test_get_hole_punch_status` | 698 | default, random, new |
| `test_establish_relay_circuit` | 704 | default, random, new |
| `test_relay_fallback_disabled` | 722 | default, random, new |
| `test_close_relay_circuit` | 741 | default, random, new |
| `test_get_relay_circuit` | 764 | default, random, new |
| `test_cleanup_old_attempts` | 785 | default, random, new |
| `test_shutdown` | 814 | default, random, new |
| `test_nat_type_equality` | 834 | default |
| `test_hole_punch_status_values` | 840 | default |
| `test_nat_config_defaults` | 847 | default |

### Imports
- `use libp2p::PeerId`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use super::*`
- `use super::swarm::SwarmHandle`
- `use thiserror::Error`
- `use tracing::{debug, info}`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt (2 chunks, 384 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt: Defines 4 types: NetworkDetector, NetworkType, FallbackTransport, NetworkDiagnostics; 13 functions; 22 imports android/app/src/main/java/com/scmessenger/android/transport/NetworkDetector.kt: Defines 4 types: NetworkDetector, NetworkType, FallbackTransport, NetworkDiagnostics; 13 functions; 22 imports

### Structs/Classes
- FallbackTransport
- NetworkDetector
- NetworkDiagnostics
- NetworkType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `startMonitoring` | 91 | d, onAvailable, detectNetworkType, onCapabilitiesChanged, build, remove, Builder, redetectCurrentNetwork, NetworkCallback, onLost |
| `onAvailable` | 102 | detectNetworkType, d, onCapabilitiesChanged, remove, i, redetectCurrentNetwork, onLost, registerNetworkCallback |
| `onLost` | 105 | d, detectNetworkType, onCapabilitiesChanged, remove, i, redetectCurrentNetwork, onLost, registerNetworkCallback, unregisterNetworkCallback, stopMonitoring |
| `onCapabilitiesChanged` | 111 | detectNetworkType, cancel, onCapabilitiesChanged, i, redetectCurrentNetwork, registerNetworkCallback, unregisterNetworkCallback, stopMonitoring |
| `stopMonitoring` | 133 | detectNetworkType, cancel, i, classifyNetworkType, getNetworkCapabilities, unregisterNetworkCallback |
| `detectNetworkType` | 150 | cancel, w, delay, emptySet, classifyNetworkType, i, s, detected, getNetworkCapabilities, cellular |
| `redetectCurrentNetwork` | 189 | detectNetworkType, hasTransport, hasCapability, classifyNetworkType, emptySet |
| `classifyNetworkType` | 202 | isPortLikelyBlocked, hasCapability, hasTransport |
| `isPortLikelyBlocked` | 230 | listOf, WebSocket, TCP, getTransportPriority |
| `getTransportPriority` | 242 | listOf |
| `getNetworkDiagnostics` | 287 | NetworkDiagnostics, hasCapability, reachability, getNetworkCapabilities, getTransportPriority, probePorts |
| `probePorts` | 313 | catch, d, close, InetSocketAddress, Socket, awaitAll, toMap, connect, async, toInt |
| `toLogString` | 375 | trimMargin, joinToString |
| `startMonitoring` | 91 | d, redetectCurrentNetwork, detectNetworkType, onAvailable, remove, build, registerNetworkCallback, NetworkCallback, onLost, addCapability |
| `onAvailable` | 102 | d, redetectCurrentNetwork, detectNetworkType, remove, registerNetworkCallback, onLost, i, onCapabilitiesChanged |
| `onLost` | 105 | d, redetectCurrentNetwork, detectNetworkType, stopMonitoring, remove, registerNetworkCallback, onLost, i, onCapabilitiesChanged, unregisterNetworkCallback |
| `onCapabilitiesChanged` | 111 | redetectCurrentNetwork, detectNetworkType, stopMonitoring, registerNetworkCallback, i, onCapabilitiesChanged, unregisterNetworkCallback, cancel |
| `stopMonitoring` | 133 | detectNetworkType, getNetworkCapabilities, classifyNetworkType, i, unregisterNetworkCallback, cancel |
| `detectNetworkType` | 150 | getNetworkCapabilities, classifyNetworkType, delay, i, detected, w, cellular, emptySet, cancel, s |
| `redetectCurrentNetwork` | 189 | detectNetworkType, hasCapability, hasTransport, emptySet, classifyNetworkType |
| `classifyNetworkType` | 202 | hasCapability, hasTransport, isPortLikelyBlocked |
| `isPortLikelyBlocked` | 230 | TCP, listOf, WebSocket, getTransportPriority |
| `getTransportPriority` | 242 | listOf |
| `getNetworkDiagnostics` | 287 | hasCapability, reachability, getNetworkCapabilities, getTransportPriority, probePorts, NetworkDiagnostics |
| `probePorts` | 313 | async, toInt, d, toMap, connect, close, catch, InetSocketAddress, awaitAll, Socket |
| `toLogString` | 375 | joinToString, trimMargin |

### Imports
- `import android.content.Context`
- `import android.net.ConnectivityManager`
- `import android.net.Network`
- `import android.net.NetworkCapabilities`
- `import android.net.NetworkRequest`
- `import android.os.Build`
- `import java.util.concurrent.ConcurrentHashMap`
- `import javax.inject.Inject`
- `import javax.inject.Singleton`
- `import kotlinx.coroutines.CoroutineScope`
- `import kotlinx.coroutines.Dispatchers`
- `import kotlinx.coroutines.Job`
- `import kotlinx.coroutines.SupervisorJob`
- `import kotlinx.coroutines.async`
- `import kotlinx.coroutines.awaitAll`
- `import kotlinx.coroutines.coroutineScope`
- `import kotlinx.coroutines.delay`
- `import kotlinx.coroutines.flow.MutableStateFlow`
- `import kotlinx.coroutines.flow.StateFlow`
- `import kotlinx.coroutines.flow.asStateFlow`
- `import kotlinx.coroutines.launch`
- `import timber.log.Timber`
---

## core/src/transport/observation.rs (1 chunks, 277 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/observation.rs: Defines 8 types: AddressObservation, AddressObserver, Default, AddressObserver, ConnectionEndpoint; 18 functions; 6 imports

### Structs/Classes
- AddressObservation
- AddressObserver
- ConnectionEndpoint
- ConnectionTracker
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 34 | new, now, recalculate_consensus |
| `new` | 41 | new, now, recalculate_consensus |
| `record_observation` | 49 | now, recalculate_consensus |
| `external_addresses` | 81 | Reverse, new, now, recalculate_consensus |
| `primary_external_address` | 86 | Reverse, new, now, recalculate_consensus |
| `all_observations` | 91 | Reverse, new, now, recalculate_consensus |
| `expire_old_observations` | 96 | Reverse, new, now, recalculate_consensus |
| `recalculate_consensus` | 109 | Reverse, new |
| `default` | 149 | new, now |
| `new` | 156 | new, now |
| `add_connection` | 163 | Ip6, now, Tcp, V4, Ip4, V6 |
| `remove_connection` | 188 | Ip6, Tcp, random, new, V4, Ip4, V6, Udp |
| `get_connection` | 193 | Ip6, Tcp, random, new, V4, Ip4, V6, Udp |
| `all_connections` | 198 | Ip6, Tcp, random, new, V4, Ip4, V6, Udp |
| `extract_socket_addr` | 203 | Ip6, Tcp, random, new, V4, Ip4, V6, Udp |
| `test_address_observer_consensus` | 231 | random, new, extract_socket_addr |
| `test_address_confirmation_count` | 256 | random, new, extract_socket_addr |
| `test_extract_socket_addr` | 272 | extract_socket_addr |

### Imports
- `use libp2p::multiaddr::Protocol`
- `use libp2p::{Multiaddr, PeerId}`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use super::*`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/peer_broadcast.rs (1 chunks, 160 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/peer_broadcast.rs: Defines 4 types: PeerBroadcaster, PeerInfo, Default, PeerBroadcaster; 10 functions; 6 imports

### Structs/Classes
- Default
- PeerBroadcaster
- PeerInfo

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 33 | full_relay, new, now |
| `new` | 40 | full_relay, new, now |
| `peer_connected` | 47 | full_relay, now |
| `peer_disconnected` | 64 | now |
| `create_peer_joined_message` | 70 | now |
| `create_peer_left_message` | 90 | new, now |
| `create_peer_list_response` | 98 | random, new, now |
| `get_peers_except` | 120 | random, new |
| `peer_count` | 129 | random, new |
| `test_peer_broadcaster` | 139 | random, new |

### Imports
- `use crate::relay::protocol::{RelayCapability, RelayMessage, RelayPeerInfoMessage}`
- `use crate::store::blocked::BlockedIdentity as RelayCapability`
- `use libp2p::PeerId`
- `use std::collections::HashMap`
- `use super::*`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## core/src/transport/reflection.rs (1 chunks, 360 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/reflection.rs: Defines 8 types: AddressReflectionRequest, AddressReflectionRequest, Default, AddressReflectionResponse, AddressReflectionResponse; 31 functions; 8 imports

### Structs/Classes
- AddressReflectionRequest
- AddressReflectionResponse
- AddressReflectionService
- Default

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 34 | new |
| `with_id` | 46 | new |
| `default` | 55 | new |
| `new` | 75 | new |
| `parse_address` | 84 | new |
| `new` | 111 | new |
| `handle_request` | 129 | new |
| `requests_served` | 151 | serialize, new |
| `enable` | 156 | deserialize, serialize, new |
| `disable` | 162 | deserialize, serialize, new |
| `is_enabled` | 168 | deserialize, serialize, new |
| `reset_stats` | 173 | deserialize, serialize, new |
| `default` | 179 | deserialize, serialize, new |
| `encode_request` | 197 | new, with_id, serialize, deserialize |
| `decode_request` | 202 | new, with_id, serialize, deserialize |
| `encode_response` | 207 | new, with_id, serialize, deserialize |
| `decode_response` | 213 | new, with_id, deserialize |
| `test_request_creation` | 227 | with_id, new |
| `test_request_with_id` | 234 | with_id, new |
| `test_response_creation` | 242 | with_id, new |
| `test_response_parse_address` | 253 | with_id, new |
| `test_service_creation` | 263 | with_id, new |
| `test_service_handle_request` | 270 | with_id, new |
| `test_service_multiple_requests` | 283 | with_id, encode_request, new, decode_request |
| `test_service_enable_disable` | 296 | with_id, encode_request, new, decode_response, encode_response, decode_request |
| `test_service_reset_stats` | 308 | with_id, encode_request, new, decode_response, default, encode_response, decode_request |
| `test_codec_request_roundtrip` | 321 | with_id, encode_request, new, decode_response, default, encode_response, decode_request |
| `test_codec_response_roundtrip` | 331 | decode_response, new, default, encode_response, decode_request |
| `test_codec_invalid_data` | 342 | default, decode_response, decode_request |
| `test_request_default` | 349 | default |
| `test_service_default` | 355 | default |

### Imports
- `use anyhow::Result`
- `use rand::RngCore`
- `use serde::{Deserialize, Serialize}`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use std::sync::atomic::{AtomicU64, Ordering}`
- `use super::*`
---

## core/src/transport/relay_health.rs (1 chunks, 395 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/relay_health.rs: Defines 6 types: RelayMetrics, RelayMetrics, RelayDiscovery, RelayDiscovery, RelayFallback; 23 functions; 6 imports

### Structs/Classes
- RelayDiscovery
- RelayFallback
- RelayMetrics

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `priority_score` | 47 | new, now |
| `is_healthy` | 62 | invalidate_cache, from_secs, new, now |
| `new` | 93 | now, invalidate_cache, new, rebuild_cache, from_secs |
| `update_relay_metrics` | 104 | invalidate_cache, rebuild_cache, now |
| `record_success` | 110 | invalidate_cache, rebuild_cache, now |
| `record_failure` | 126 | invalidate_cache, rebuild_cache |
| `get_priority_relays` | 139 | rebuild_cache, now |
| `get_fallback_relays` | 153 | now |
| `add_fallback_relay` | 158 | now |
| `relay_count` | 166 | invalidate_cache, now |
| `healthy_relay_count` | 171 | invalidate_cache, now |
| `get_all_metrics` | 180 | invalidate_cache, now |
| `cleanup_stale_relays` | 185 | invalidate_cache, now |
| `invalidate_cache` | 216 | healthy_relay_count, new, now |
| `rebuild_cache` | 221 | healthy_relay_count, new, now |
| `new` | 254 | generate_ed25519, from_secs, new |
| `should_retry` | 263 | generate_ed25519, from_secs, now |
| `record_attempt` | 271 | generate_ed25519, from_secs, now |
| `get_backoff_delay` | 277 | generate_ed25519, from_secs, new, now |
| `reset_attempts` | 284 | generate_ed25519, new, now |
| `test_relay_metrics_priority_score` | 295 | generate_ed25519, new, now |
| `test_relay_discovery_priority_ordering` | 324 | generate_ed25519, new, now |
| `test_relay_fallback_backoff` | 373 | new |

### Imports
- `use libp2p::identity`
- `use libp2p::{Multiaddr, PeerId}`
- `use std::collections::{HashMap, VecDeque}`
- `use super::*`
- `use tracing::{debug, info}`
- `use web_time::{Duration, SystemTime, UNIX_EPOCH}`
---

## core/src/transport/reputation.rs (1 chunks, 619 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/reputation.rs: Defines 8 types: ReputationScore, ReputationScore, std, AbuseSignal, PeerAbuseStats; 37 functions; 6 imports

### Structs/Classes
- AbuseReputationManager
- AbuseSignal
- PeerAbuseStats
- ReputationScore
- std

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 26 |  |
| `neutral` | 30 |  |
| `value` | 34 |  |
| `is_trusted` | 38 |  |
| `is_suspicious` | 42 |  |
| `is_abusive` | 46 |  |
| `fmt` | 53 |  |
| `new` | 103 | neutral, calculate_score, now |
| `record_signal` | 122 | new, calculate_score, now |
| `calculate_score` | 146 | new |
| `rate_limit_multiplier` | 165 | from_secs, new |
| `new` | 195 | from_secs, new, now |
| `with_backend` | 206 | from_secs, new, now |
| `load_from_storage` | 218 | to_vec, now |
| `persist_peer` | 257 | new, to_vec |
| `remove_peer_from_storage` | 274 | new |
| `apply_decay` | 286 | persist_peer, new |
| `flush_to_storage` | 335 | persist_peer, remove_peer_from_storage, new |
| `record_signal` | 351 | persist_peer, remove_peer_from_storage, new |
| `get_score` | 378 | remove_peer_from_storage, new, now |
| `rate_limit_multiplier` | 387 | remove_peer_from_storage, new, now |
| `all_reputations` | 396 | remove_peer_from_storage, new, now |
| `prune_stale` | 406 | remove_peer_from_storage, new, now, neutral |
| `len` | 429 | neutral, new, now |
| `is_empty` | 434 | neutral, new, now |
| `current_epoch_secs` | 438 | neutral, new, now |
| `test_neutral_score` | 451 | new, neutral |
| `test_successful_delivery_increases_score` | 460 | new |
| `test_rate_limiting_decreases_score` | 470 | from_secs, new |
| `test_rate_limit_multiplier` | 480 | from_secs, new |
| `test_reputation_manager_eviction` | 495 | from_secs, new, with_backend |
| `test_prune_stale` | 508 | from_secs, new, with_backend |
| `test_mixed_signals` | 518 | new, with_backend |
| `test_persistence_roundtrip` | 532 | from_utf8_lossy, new, with_backend |
| `test_persistence_eviction_cleans_storage` | 558 | from_utf8_lossy, new, with_backend |
| `test_decay_moves_toward_neutral` | 580 | new |
| `test_epoch_secs_recorded` | 608 | new |

### Imports
- `use crate::store::backend::StorageBackend`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::sync::Arc`
- `use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH}`
- `use super::*`
---

## core/src/transport/ble/scanner.rs (1 chunks, 628 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/ble/scanner.rs: Defines 13 types: BleScanConfig, Default, BleScanConfig, ScannerState, BatteryState; 54 functions; 4 imports

### Structs/Classes
- BatteryState
- BleScanConfig
- BleScanner
- Default
- DutyCycleManager
- ScanResult
- ScannerError
- ScannerState

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 23 | InvalidScanConfig |
| `new` | 34 | InvalidScanConfig |
| `with_duty_cycle` | 45 | InvalidScanConfig |
| `validate` | 54 | InvalidScanConfig |
| `from_percentage` | 95 |  |
| `new` | 112 | get_recommended_duty_cycle |
| `set_battery_state` | 120 | get_recommended_duty_cycle |
| `set_wifi_available` | 125 | get_recommended_duty_cycle |
| `get_recommended_duty_cycle` | 130 | get_recommended_duty_cycle |
| `get_mode_description` | 156 | get_recommended_duty_cycle, now |
| `new` | 199 | now, SystemTimeError |
| `estimate_distance` | 219 | new, now, SystemTimeError |
| `new` | 244 | update_timestamp, new, now, SystemTimeError |
| `state` | 260 | update_timestamp |
| `config` | 265 | update_timestamp |
| `is_active` | 270 | update_timestamp |
| `start_scanning` | 275 | update_timestamp |
| `pause_scanning` | 292 | update_timestamp, get_duty_cycle |
| `stop_scanning` | 304 | update_timestamp, get_duty_cycle, now |
| `set_battery_state` | 316 | get_duty_cycle, now |
| `set_wifi_available` | 321 | get_duty_cycle, now |
| `get_duty_cycle` | 326 | default, get_duty_cycle, now |
| `get_mode` | 331 | default, get_duty_cycle, now |
| `calculate_scan_duration_ms` | 336 | default, get_duty_cycle, now |
| `calculate_pause_duration_ms` | 342 | default, get_duty_cycle, now |
| `update_timestamp` | 349 | default, now |
| `time_since_state_change_ms` | 356 | default, now |
| `test_ble_scan_config_default` | 371 | default, from_percentage |
| `test_ble_scan_config_validation_valid` | 379 | default, from_percentage |
| `test_ble_scan_config_validation_window_exceeds_interval` | 385 | default, from_percentage, new |
| `test_ble_scan_config_validation_zero_interval` | 395 | default, from_percentage, new |
| `test_ble_scan_config_with_duty_cycle` | 405 | default, from_percentage, new |
| `test_ble_scan_config_invalid_duty_cycle` | 413 | default, from_percentage, new |
| `test_battery_state_from_percentage` | 419 | from_percentage, new |
| `test_duty_cycle_manager_aggressive_when_charging` | 429 | new |
| `test_duty_cycle_manager_aggressive_with_charging_and_wifi` | 436 | new |
| `test_duty_cycle_manager_standard_good_battery` | 443 | new |
| `test_duty_cycle_manager_reduced_low_battery` | 450 | default, new |
| `test_duty_cycle_manager_minimal_critical_battery` | 457 | default, new |
| `test_duty_cycle_manager_set_battery_state` | 464 | default, new |
| `test_scan_result_creation` | 473 | default, new |
| `test_scan_result_distance_estimate` | 487 | default, new |
| `test_ble_scanner_creation` | 496 | default, new |
| `test_ble_scanner_start_stop` | 505 | default, new |
| `test_ble_scanner_pause_resume` | 519 | default, new |
| `test_ble_scanner_double_start_error` | 532 | default, new |
| `test_ble_scanner_set_battery_state` | 542 | default, new |
| `test_ble_scanner_set_wifi_available` | 553 | default, new |
| `test_ble_scanner_calculate_scan_duration` | 565 | default, new |
| `test_ble_scanner_calculate_pause_duration` | 575 | default, new |
| `test_ble_scanner_mode_description` | 585 | default, new |
| `test_ble_scanner_time_since_state_change` | 593 | default, new |
| `test_ble_scanner_invalid_config` | 602 | new |
| `test_all_battery_states` | 614 | new |

### Imports
- `use serde::{Deserialize, Serialize}`
- `use super::*`
- `use thiserror::Error`
- `use web_time::{SystemTime, UNIX_EPOCH}`
---

## android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt (2 chunks, 629 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt: Defines 1 types: TransportManager; 20 functions; 8 imports android/app/src/main/java/com/scmessenger/android/transport/TransportManager.kt: Defines 1 types: TransportManager; 20 functions; 8 imports

### Structs/Classes
- TransportManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `initialize` | 64 | TransportManager, w, initializeWifiDirect, i, initializeWifiAware, initializeBle, startAll |
| `startAll` | 90 | start, d, onDataReceived, w, startListening, startScanning, discovery, onPeerDiscovered, MdnsServiceDiscovery, startAdvertising |
| `stopAll` | 135 | clear, stopAdvertising, stopListening, i, stop, stopScanning |
| `sendData` | 166 | sendViaTransport, shouldUseTransport |
| `sendViaTransport` | 204 | d, sendViaTransport, getConnectedDeviceAddresses, sendData, first |
| `attemptEscalation` | 252 | isAvailable, d, getAvailableTransports, toList, getActiveTransports |
| `getActiveTransports` | 275 | isAvailable, getAvailableTransports, add, toList, BleScanner, initializeBle |
| `getAvailableTransports` | 282 | isAvailable, d, add, attemptEscalation, BleScanner, onPeerDiscovered, initializeBle |
| `initializeBle` | 299 | d, onDataReceived, BleGattClient, attemptEscalation, BleScanner, onPeerDiscovered, BleAdvertiser, initializeBle, BleGattServer |
| `initializeWifiAware` | 347 | isAvailable, d, onDataReceived, catch, initializeWifiDirect, onPeerDiscovered, initializeWifiAware, WifiDirectTransport, WifiAwareTransport, e |
| `initializeWifiDirect` | 374 | d, onDataReceived, catch, enableTransport, startScanning, onPeerDiscovered, initializeWifiDirect, WifiDirectTransport, startAdvertising, e |
| `enableTransport` | 400 | start, onDataReceived, startScanning, onPeerDiscovered, MdnsServiceDiscovery, startAdvertising |
| `disableTransport` | 436 | cleanup, remove, stopAdvertising, stop, stopAll, stopScanning |
| `cleanup` | 463 | cleanup, cancel, getBleQuotaCount, getQuotaCount, setBleComponents, shutdown, i, initialization, stopAll |
| `getBleQuotaCount` | 481 | d, setBleComponents, getQuotaCount, applyScanSettings, initialization |
| `setBleComponents` | 490 | d, applyScanSettings, applyAdvertiseSettings |
| `applyScanSettings` | 506 | d, w, handleBleFailure, remove, stopAdvertising, transports, applyScanSettings, i, stopScanning, applyAdvertiseSettings |
| `applyAdvertiseSettings` | 514 | d, w, handleBleFailure, remove, stopAdvertising, transports, i, stopScanning, applyAdvertiseSettings |
| `handleBleFailure` | 523 | d, w, attemptBleRecovery, startScanning, transports, stopAdvertising, remove, i, startAdvertising, stopScanning |
| `attemptBleRecovery` | 546 | d, startScanning, i, startAdvertising |
| `initialize` | 64 | initializeBle, initializeWifiDirect, startAll, w, i, initializeWifiAware, TransportManager |
| `startAll` | 90 | d, onDataReceived, MdnsServiceDiscovery, startListening, onPeerDiscovered, discovery, startAdvertising, start, w, startScanning |
| `stopAll` | 135 | stopScanning, i, stopAdvertising, stopListening, clear, stop |
| `sendData` | 166 | shouldUseTransport, sendViaTransport |
| `sendViaTransport` | 204 | d, getConnectedDeviceAddresses, sendData, sendViaTransport, first |
| `attemptEscalation` | 252 | d, isAvailable, getActiveTransports, getAvailableTransports, toList |
| `getActiveTransports` | 275 | initializeBle, add, isAvailable, BleScanner, getAvailableTransports, toList |
| `getAvailableTransports` | 282 | attemptEscalation, d, initializeBle, add, isAvailable, BleScanner, onPeerDiscovered |
| `initializeBle` | 299 | attemptEscalation, d, initializeBle, onDataReceived, BleAdvertiser, BleGattServer, BleGattClient, BleScanner, onPeerDiscovered |
| `initializeWifiAware` | 347 | d, onDataReceived, initializeWifiDirect, catch, WifiDirectTransport, onPeerDiscovered, isAvailable, initializeWifiAware, e, WifiAwareTransport |
| `initializeWifiDirect` | 374 | d, onDataReceived, initializeWifiDirect, catch, WifiDirectTransport, startAdvertising, onPeerDiscovered, enableTransport, e, startScanning |
| `enableTransport` | 400 | onDataReceived, MdnsServiceDiscovery, startAdvertising, start, onPeerDiscovered, startScanning |
| `disableTransport` | 436 | stopScanning, cleanup, stopAll, remove, stopAdvertising, stop |
| `cleanup` | 463 | initialization, cleanup, shutdown, stopAll, getQuotaCount, getBleQuotaCount, i, setBleComponents, cancel |
| `getBleQuotaCount` | 481 | initialization, d, applyScanSettings, getQuotaCount, setBleComponents |
| `setBleComponents` | 490 | d, applyScanSettings, applyAdvertiseSettings |
| `applyScanSettings` | 506 | d, stopScanning, applyScanSettings, remove, transports, applyAdvertiseSettings, i, handleBleFailure, stopAdvertising, w |
| `applyAdvertiseSettings` | 514 | d, stopScanning, remove, transports, applyAdvertiseSettings, i, handleBleFailure, stopAdvertising, w |
| `handleBleFailure` | 523 | d, stopScanning, remove, transports, i, stopAdvertising, attemptBleRecovery, startAdvertising, w, startScanning |
| `attemptBleRecovery` | 546 | d, i, startAdvertising, startScanning |

### Imports
- `import android.content.Context`
- `import com.scmessenger.android.service.TransportType`
- `import com.scmessenger.android.transport.ble.*`
- `import java.util.concurrent.ConcurrentHashMap`
- `import kotlinx.coroutines.*`
- `import timber.log.Timber`
- `import uniffi.api.AdjustmentProfile`
- `import uniffi.api.BleAdjustment`
---

## core/src/transport/websocket.rs (1 chunks, 306 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/websocket.rs: Defines 3 types: WebSocketTransportError, WebSocketTransport, WebSocketTransport; 15 functions; 7 imports

### Structs/Classes
- WebSocketTransport
- WebSocketTransportError

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 47 | InvalidUrl, new, from_secs, Binary, timeout, Timeout, ConnectionFailed |
| `from_multiaddr` | 56 | InvalidUrl, new, from_secs, Binary, SendFailed, timeout, Timeout, ConnectionFailed |
| `connect` | 63 | InvalidUrl, Binary, from_secs, SendFailed, timeout, Timeout, ConnectionFailed |
| `send` | 91 | Text, ReceiveFailed, from_secs, Binary, SendFailed, timeout, Timeout, ConnectionFailed |
| `recv` | 109 | Text, ReceiveFailed, Binary, from_secs, timeout, Timeout, ConnectionFailed |
| `close` | 135 | Ip6, Dns4, Wss, Ws, Dns6, Dns, Ip4, Tcp, ConnectionFailed |
| `is_connected` | 148 | Ip6, InvalidUrl, Dns4, Wss, Ws, Dns6, Dns, Ip4, Tcp |
| `multiaddr_to_websocket_url` | 154 | Ip6, InvalidUrl, Dns4, Wss, Ws, Dns6, Dns, Ip4, Tcp |
| `diagnose_websocket_error` | 212 | InvalidUrl, ReceiveFailed, SendFailed, ConfigError, Timeout, HandshakeFailed, ConnectionFailed |
| `test_multiaddr_to_websocket_url_ws` | 258 | from_secs, new, ConnectionFailed |
| `test_multiaddr_to_websocket_url_wss` | 265 | from_secs, new, ConnectionFailed |
| `test_multiaddr_to_websocket_url_wss_protocol` | 272 | from_secs, new, ConnectionFailed |
| `test_multiaddr_to_websocket_url_invalid` | 279 | from_secs, new, ConnectionFailed |
| `test_websocket_transport_creation` | 286 | from_secs, new, ConnectionFailed |
| `test_websocket_transport_diagnostics` | 294 | ConnectionFailed |

### Imports
- `use crate::transport::internet::InternetTransportError`
- `use futures::{SinkExt, StreamExt}`
- `use libp2p::Multiaddr`
- `use std::time::Duration`
- `use super::*`
- `use tokio_tungstenite::{connect_async, tungstenite::client::IntoClientRequest}`
- `use tracing::{debug, info, warn}`
---

## core/src/transport/wifi_aware.rs (1 chunks, 796 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/transport/wifi_aware.rs: Defines 12 types: WifiAwareError, WifiAwareConfig, Default, WifiAwareState, DataPathInfo; 53 functions; 9 imports

### Structs/Classes
- DataPathInfo
- Default
- DiscoveredPeer
- MockWifiAwareBridge
- WifiAwareConfig
- WifiAwareError
- WifiAwarePlatformBridge
- WifiAwareState
- WifiAwareTransport

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default` | 56 | new |
| `is_available` | 128 |  |
| `publish_service` | 131 |  |
| `subscribe_to_services` | 138 | new |
| `unpublish_service` | 145 | new |
| `unsubscribe_from_services` | 148 | new |
| `create_data_path` | 151 | new |
| `close_data_path` | 158 | new |
| `set_on_service_discovered` | 161 | new |
| `set_on_message_received` | 164 | new |
| `set_on_data_path_confirmed` | 167 | new |
| `new` | 183 | new |
| `add_discovered_peer` | 189 |  |
| `is_available` | 198 |  |
| `publish_service` | 201 |  |
| `subscribe_to_services` | 213 |  |
| `unpublish_service` | 225 |  |
| `unsubscribe_from_services` | 229 |  |
| `create_data_path` | 233 | InvalidConfig |
| `close_data_path` | 242 | new, InvalidConfig |
| `set_on_service_discovered` | 247 | new, InvalidConfig |
| `set_on_message_received` | 253 | new, InvalidConfig |
| `set_on_data_path_confirmed` | 255 | new, InvalidConfig |
| `new` | 275 | new, get_state, InvalidConfig |
| `get_state` | 295 | get_state, InvalidConfig |
| `initialize` | 300 | get_state, InvalidConfig |
| `publish_service` | 316 | get_state, InvalidConfig |
| `subscribe` | 338 | get_state, InvalidConfig, PeerNotFound |
| `create_data_path` | 366 | get_state, DataPathFailed, PeerNotFound |
| `close_data_path` | 417 | register_peer |
| `get_data_path` | 429 | new, register_peer |
| `get_active_data_paths` | 434 | new, register_peer |
| `get_discovered_peers` | 439 | new, register_peer |
| `register_peer` | 444 | new, register_peer |
| `add_discovered_peer` | 454 | close_data_path, new, register_peer |
| `wire_discovery_callback` | 474 | close_data_path, new |
| `shutdown` | 499 | close_data_path |
| `estimate_bandwidth_from_rssi` | 523 | default, new |
| `test_wifi_aware_initialization` | 549 | default, new |
| `test_wifi_aware_unavailable` | 559 | default, new |
| `test_publish_service` | 569 | default, new |
| `test_subscribe_service` | 580 | default, random, new |
| `test_publish_disabled` | 591 | default, random, new |
| `test_invalid_config` | 604 | default, random, new |
| `test_create_data_path` | 613 | default, random, new |
| `test_data_path_not_found` | 638 | default, random, new |
| `test_close_data_path` | 653 | default, random, new |
| `test_get_active_data_paths` | 676 | default, random, new |
| `test_max_data_paths_limit` | 707 | default, random, new |
| `test_shutdown` | 736 | default, random, new |
| `test_bandwidth_estimation` | 747 | default, random, new |
| `test_get_discovered_peers` | 763 | default, random, new |
| `test_config_validation` | 787 | default, new |

### Imports
- `use async_trait::async_trait`
- `use libp2p::PeerId`
- `use parking_lot::RwLock`
- `use std::collections::HashMap`
- `use std::net::SocketAddr`
- `use std::sync::Arc`
- `use super::*`
- `use thiserror::Error`
- `use tracing::{info, warn}`
---

## android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt (1 chunks, 476 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt: Defines 6 types: SmartTransportRouter, TransportType, TransportDeliveryResult, TransportHealth, MessageDedupEntry; 14 functions; 6 imports

### Structs/Classes
- MessageDedupEntry
- SmartTransportRouter
- TransportAttempt
- TransportDeliveryResult
- TransportHealth
- TransportType

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `fromValue` | 34 | TransportHealth, currentTimeMillis, TransportDeliveryResult, rate, get |
| `recordSuccess` | 117 | recordFailure, currentTimeMillis, toDouble, setHealth, format, i, tag, take, getHealth |
| `recordFailure` | 142 | TransportHealth, peer, currentTimeMillis, getOrPut, ConcurrentHashMap, setHealth, format, tag, take, w |
| `getHealth` | 157 | TransportHealth, peer, getPreferredTransport, ConcurrentHashMap, setHealth, getOrPut, getHealth |
| `setHealth` | 165 | getPreferredTransport, peer, ConcurrentHashMap, getOrPut, getHealth |
| `getPreferredTransport` | 174 | score, getAvailableTransportsSorted, getHealth |
| `getAvailableTransportsSorted` | 202 | isNotEmpty, available, toList, checkAndRecordMessage, Triple, getAvailableTransports, getHealth, data |
| `getAvailableTransports` | 213 | data, cleanupDedupCache, isNotEmpty, currentTimeMillis, checkAndRecordMessage, Triple, toList |
| `checkAndRecordMessage` | 230 | cleanupDedupCache, mutableListOf, currentTimeMillis, add, i, Triple, tag, take, MessageDedupEntry |
| `getDedupStats` | 274 | currentTimeMillis, cleanupDedupCache, Suppress, attemptDelivery |
| `cleanupDedupCache` | 281 | currentTimeMillis, attemptDelivery, Suppress, suspend |
| `attemptDelivery` | 296 | suspend, TransportAttempt, isNotEmpty, currentTimeMillis, add, trim, Suppress, tryWifi |
| `getHealthSummary` | 447 | peer, remove, i, resetHealth, tag, take, mapOf |
| `resetHealth` | 471 | remove, i, take, tag |

### Imports
- `import java.util.concurrent.ConcurrentHashMap`
- `import java.util.concurrent.atomic.AtomicLong`
- `import kotlinx.coroutines.*`
- `import kotlinx.coroutines.sync.Mutex`
- `import kotlinx.coroutines.sync.withLock`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/TransportHealthMonitor.kt (1 chunks, 73 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/TransportHealthMonitor.kt: Defines 2 types: TransportHealthMonitor, TransportHealth; 6 functions; 1 imports

### Structs/Classes
- TransportHealth
- TransportHealthMonitor

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `recordSuccess` | 22 | TransportHealth, recordFailure, w, currentTimeMillis, recordSuccess, shouldUseTransport, getOrPut, getHealth |
| `recordFailure` | 30 | TransportHealth, recordFailure, w, currentTimeMillis, toDouble, shouldUseTransport, getOrPut, getHealth |
| `getHealth` | 41 | toMap, TransportHealth, getSummary, toDouble, isDegraded, shouldUseTransport, getHealth |
| `shouldUseTransport` | 50 | toMap, getSummary, toDouble, isDegraded, getHealth |
| `isDegraded` | 61 | toMap, getSummary, getHealth |
| `getSummary` | 65 | toMap, getSummary |

### Imports
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt (1 chunks, 458 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt: Defines 1 types: WifiAwareTransport; 22 functions; 13 imports

### Structs/Classes
- WifiAwareTransport

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `isAvailable` | 59 | catch, attach, i, isAvailable, start, w, e |
| `start` | 67 | stop, catch, attach, i, isAvailable, w, e |
| `stop` | 91 | close, w, catch, synchronized, clear, unregisterNetworkCallback, cancel, toList |
| `sendData` | 135 | onAttachFailed, AttachCallback, onAttached, send, i, startSubscribing, w, e, startPublishing |
| `onAttached` | 145 | onAttachFailed, d, onAttached, build, setServiceName, i, startSubscribing, publish, Builder, setPublishType |
| `onAttachFailed` | 159 | onAttachFailed, d, setSubscribeType, build, catch, setServiceName, publish, startSubscribing, Builder, setPublishType |
| `startPublishing` | 166 | d, setSubscribeType, subscribe, build, catch, setServiceName, publish, startSubscribing, Builder, setPublishType |
| `startSubscribing` | 182 | d, setSubscribeType, subscribe, build, catch, onServiceDiscovered, onPublishStarted, setServiceName, i, startSubscribing |
| `onPublishStarted` | 200 | d, initiateDataPath, w, DiscoverySessionCallback, onServiceDiscovered, onPublishStarted, RESPONDER, i, onSubscribeStarted, toString |
| `onServiceDiscovered` | 205 | d, initiateDataPath, w, DiscoverySessionCallback, onServiceDiscovered, RESPONDER, i, onSubscribeStarted, toString, onPeerDiscovered |
| `onSubscribeStarted` | 228 | d, initiateDataPath, w, INITIATOR, onServiceDiscovered, i, onSubscribeStarted, toString, onPeerDiscovered, RequiresApi |
| `onServiceDiscovered` | 233 | d, initiateDataPath, w, build, INITIATOR, onServiceDiscovered, Builder, toString, onPeerDiscovered, RequiresApi |
| `initiateDataPath` | 256 | onAvailable, build, NetworkCallback, i, setNetworkSpecifier, Builder, createResponderSocket, onCapabilitiesChanged, addTransportType, peerIdString |
| `onAvailable` | 271 | d, onAvailable, i, containsKey, createInitiatorSocket, onCapabilitiesChanged, createResponderSocket, peerIdString |
| `onCapabilitiesChanged` | 283 | d, close, remove, catch, synchronized, onLost, containsKey, createInitiatorSocket, onCapabilitiesChanged, unregisterNetworkCallback |
| `onLost` | 302 | d, close, remove, catch, synchronized, onLost, put, w, unregisterNetworkCallback, requestNetwork |
| `createResponderSocket` | 338 | d, accept, close, ServerSocket, catch, startReading, AwareConnection, i, withContext, Subscriber |
| `createInitiatorSocket` | 369 | connect, getInputStream, catch, getOutputStream, InetSocketAddress, createSocket, AwareConnection, startReading, i, withContext |
| `startReading` | 398 | onDataReceived, read, close, catch, startReading, send, copyOfRange, ByteArray, e |
| `send` | 426 | cleanup, stop, close, catch, cancel, send, write, flush, w, e |
| `close` | 437 | cleanup, close, catch, cancel, w, stop |
| `cleanup` | 447 | cleanup, cancel, stop |

### Imports
- `import android.content.Context`
- `import android.net.*`
- `import android.net.wifi.aware.*`
- `import android.os.Build`
- `import androidx.annotation.RequiresApi`
- `import java.io.InputStream`
- `import java.io.OutputStream`
- `import java.net.InetSocketAddress`
- `import java.net.ServerSocket`
- `import java.net.Socket`
- `import java.util.concurrent.ConcurrentHashMap`
- `import kotlinx.coroutines.*`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/WifiDirectTransport.kt (1 chunks, 456 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/WifiDirectTransport.kt: Defines 1 types: WifiDirectTransport; 23 functions; 16 imports

### Structs/Classes
- WifiDirectTransport

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `start` | 54 | registerReceiver, initialize, addAction, w, e, IntentFilter |
| `stop` | 102 | unregisterReceiver, clearLocalServices, close, removeGroup, i, clear, stopPeerDiscovery |
| `sendData` | 143 | d, startServiceDiscovery, newInstance, send, setDnsSdResponseListeners, onPeerDiscovered, connectToPeer, w |
| `startServiceDiscovery` | 151 | d, startPeerDiscovery, onSuccess, startServiceDiscovery, newInstance, setDnsSdResponseListeners, addServiceRequest, connectToPeer, onPeerDiscovered, e |
| `onSuccess` | 175 | d, discoverPeers, startPeerDiscovery, onSuccess, catch, e, onFailure |
| `onFailure` | 179 | discoverPeers, d, startPeerDiscovery, onSuccess, catch, e, onFailure |
| `startPeerDiscovery` | 190 | d, discoverPeers, startPeerDiscovery, onSuccess, catch, newInstance, put, registerService, e, onFailure |
| `onSuccess` | 196 | d, catch, newInstance, put, addLocalService, registerService, e, onFailure |
| `onFailure` | 199 | d, onSuccess, catch, newInstance, put, addLocalService, registerService, e, onFailure |
| `registerService` | 210 | d, onSuccess, catch, newInstance, put, addLocalService, registerService, e, onFailure |
| `onSuccess` | 227 | d, connect, onSuccess, catch, connectToPeer, containsKey, WifiP2pConfig, e, onFailure |
| `onFailure` | 230 | d, connect, onSuccess, catch, connectToPeer, containsKey, WifiP2pConfig, e, onFailure |
| `connectToPeer` | 241 | d, connect, onSuccess, catch, connectToPeer, containsKey, WifiP2pConfig, e, onFailure |
| `onSuccess` | 257 | d, catch, getIntExtra, Suppress, onReceive, BroadcastReceiver, getParcelableExtra, e, onFailure |
| `onFailure` | 260 | d, catch, getIntExtra, Suppress, onReceive, BroadcastReceiver, getParcelableExtra, e, onFailure |
| `onReceive` | 273 | d, requestConnectionInfo, handleConnectionInfo, getIntExtra, Suppress, getParcelableExtra |
| `handleConnectionInfo` | 306 | d, connectToGroupOwner, accept, P2pConnection, startServer, ServerSocket, handleConnectionInfo, startReading, i |
| `startServer` | 320 | accept, d, connectToGroupOwner, P2pConnection, connect, startServer, ServerSocket, catch, startReading, InetSocketAddress |
| `connectToGroupOwner` | 345 | connectToGroupOwner, connect, P2pConnection, getInputStream, catch, getOutputStream, InetSocketAddress, startReading, i, Socket |
| `startReading` | 375 | onDataReceived, read, startReading, wrap, ByteArray, e |
| `send` | 419 | allocate, cleanup, stop, putInt, close, w, remove, catch, synchronized, cancel |
| `close` | 434 | cleanup, close, remove, catch, cancel, w, stop |
| `cleanup` | 445 | cleanup, cancel, stop |

### Imports
- `import android.content.BroadcastReceiver`
- `import android.content.Context`
- `import android.content.Intent`
- `import android.content.IntentFilter`
- `import android.net.wifi.p2p.*`
- `import android.net.wifi.p2p.nsd.WifiP2pDnsSdServiceInfo`
- `import android.net.wifi.p2p.nsd.WifiP2pDnsSdServiceRequest`
- `import androidx.core.content.IntentCompat`
- `import java.io.InputStream`
- `import java.io.OutputStream`
- `import java.net.InetSocketAddress`
- `import java.net.ServerSocket`
- `import java.net.Socket`
- `import java.util.concurrent.ConcurrentHashMap`
- `import kotlinx.coroutines.*`
- `import timber.log.Timber`
---

## android/app/src/main/java/com/scmessenger/android/transport/WifiTransportManager.kt (1 chunks, 177 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
android/app/src/main/java/com/scmessenger/android/transport/WifiTransportManager.kt: Defines 1 types: WifiTransportManager; 12 functions; 11 imports

### Structs/Classes
- WifiTransportManager

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `onReceive` | 35 | d, requestPeers, initialize, WifiDirectTransport, startDiscovery, invoke, e, getMainLooper |
| `initialize` | 42 | d, initialize, w, WifiDirectTransport, startDiscovery, start, invoke, e, getMainLooper, hasDiscoveryPermissions |
| `startDiscovery` | 60 | registerReceiver, d, discoverPeers, onSuccess, catch, i, startDiscovery, start, w, e |
| `onSuccess` | 75 | registerReceiver, stopDiscovery, w, onSuccess, catch, i, stopPeerDiscovery, e, onFailure |
| `onFailure` | 80 | unregisterReceiver, stopDiscovery, w, onSuccess, catch, i, stopPeerDiscovery, e, onFailure |
| `stopDiscovery` | 91 | unregisterReceiver, stopDiscovery, stop, registerReceiver, onSuccess, catch, i, w, stopPeerDiscovery, IntentFilter |
| `onSuccess` | 97 | unregisterReceiver, registerReceiver, catch, i, addAction, w, stop, IntentFilter, onFailure |
| `onFailure` | 101 | unregisterReceiver, registerReceiver, requestPeers, catch, addAction, w, stop, IntentFilter, hasDiscoveryPermissions |
| `registerReceiver` | 118 | registerReceiver, requestPeers, catch, peerId, onPeerDiscovered, v, addAction, w, e, IntentFilter |
| `requestPeers` | 128 | requestPeers, catch, peerId, onPeerDiscovered, v, checkSelfPermission, w, e, hasDiscoveryPermissions |
| `hasDiscoveryPermissions` | 147 | d, sendData, trim, isEmpty, checkSelfPermission, w, hasDiscoveryPermissions |
| `sendData` | 160 | d, sendData, trim, isEmpty, w |

### Imports
- `import android.Manifest`
- `import android.content.BroadcastReceiver`
- `import android.content.Context`
- `import android.content.Intent`
- `import android.content.IntentFilter`
- `import android.content.pm.PackageManager`
- `import android.net.wifi.p2p.WifiP2pManager`
- `import android.os.Build`
- `import android.os.Looper`
- `import androidx.core.content.ContextCompat`
- `import timber.log.Timber`
---

## cli/src/bootstrap.rs (1 chunks, 219 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
cli/src/bootstrap.rs: 10 functions; 2 imports

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `default_bootstrap_nodes` | 47 | strip_peer_id, var |
| `promiscuous_bootstrap_addrs` | 92 | new, strip_peer_id |
| `parse_bootstrap_addr` | 102 | new, strip_peer_id |
| `merge_bootstrap_nodes` | 113 | new, strip_peer_id |
| `default_topics` | 137 |  |
| `test_default_bootstrap_nodes` | 146 |  |
| `test_promiscuous_addrs_strip_peer_id` | 164 |  |
| `test_parse_bootstrap_addr` | 181 |  |
| `test_merge_deduplicates_by_ip` | 197 |  |
| `test_default_topics` | 214 |  |

### Imports
- `use crate::ledger`
- `use super::*`
---

## core/src/abuse/reputation.rs (1 chunks, 274 lines)
Function `P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH` not found in REPO_MAP chunks. Full file listing below.

### Summary
core/src/abuse/reputation.rs: Defines 4 types: EnhancedAbuseReputationManager, EnhancedAbuseReputationManager, EnhancedReputationScore, EnhancedReputationScore; 24 functions; 9 imports

### Structs/Classes
- EnhancedAbuseReputationManager
- EnhancedReputationScore

### Functions
| Function | Line | Calls Out To |
|----------|------|-------------|
| `new` | 20 | new, with_backend |
| `with_backend` | 29 | with_backend |
| `apply_decay` | 41 |  |
| `flush_to_storage` | 46 |  |
| `record_signal` | 52 |  |
| `record_spam_signal` | 71 |  |
| `record_outbound_message` | 101 |  |
| `get_enhanced_score` | 112 |  |
| `get_score` | 124 |  |
| `rate_limit_multiplier` | 129 |  |
| `all_enhanced_scores` | 134 |  |
| `spam_detector` | 153 | new |
| `base_manager` | 158 | new, default |
| `overall_score` | 177 | new, default |
| `is_suspicious` | 183 | new, default |
| `is_abusive` | 188 | new, default |
| `make_manager` | 201 | new, default |
| `test_neutral_peer_has_neutral_score` | 212 |  |
| `test_positive_signals_increase_score` | 219 |  |
| `test_negative_signals_decrease_score` | 229 |  |
| `test_enhanced_score_combines_base_and_spam` | 239 |  |
| `test_spam_signal_recording` | 248 |  |
| `test_all_enhanced_scores` | 257 |  |
| `test_outbound_message_tracking` | 266 |  |

### Imports
- `use crate::abuse::spam_detection::SpamDetectionConfig`
- `use crate::abuse::spam_detection::{SpamDetectionEngine, SpamSignal}`
- `use crate::store::backend::MemoryStorage`
- `use crate::store::blocked::BlockedManager`
- `use crate::store::contacts::ContactManager`
- `use crate::transport::reputation::{AbuseReputationManager, AbuseSignal, ReputationScore}`
- `use std::sync::Arc`
- `use super::*`
---
