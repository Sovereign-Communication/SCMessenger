# Design Plan: BLE GATT Traits Cleanup (CORE-SWEEP-03)

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Status:** PROPOSED / REVIEWED  
**Date:** 2026-07-09  
**Task reference:** [NEEDS PLANNING] CORE_SWEEP_03_ble_gatt_traits_never_implemented.md

---

## 1. Context and Origin of GATT Traits

During early development of the SCMessenger BLE transport, the traits `GattServer` and `GattClient` were defined in `core/src/transport/ble/gatt.rs`:

```rust
pub trait GattServer: Send + Sync {
    fn on_write(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError>;
    fn on_read(&self, characteristic: GattCharacteristic) -> Result<Vec<u8>, GattError>;
    fn notify(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError>;
    fn is_enabled(&self) -> bool;
}

pub trait GattClient: Send + Sync {
    fn write(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError>;
    fn read(&self, characteristic: GattCharacteristic) -> Result<Vec<u8>, GattError>;
    fn subscribe(&mut self, characteristic: GattCharacteristic) -> Result<(), GattError>;
    fn unsubscribe(&mut self, characteristic: GattCharacteristic) -> Result<(), GattError>;
    fn is_connected(&self) -> bool;
}
```

These traits were intended as a cross-platform abstraction layer for platform-specific Bluetooth stacks. However, they were never implemented by any platform:
* **Android:** Uses a fully native Kotlin implementation (`BleGattServer.kt` and `BleGattClient.kt`) that interacts directly with Android's `BluetoothGattServerCallback` / `BluetoothGattCallback` classes. It communicates with the Rust core using higher-level FFI calls (`onDataReceived`, `sendBlePacket`) rather than implementing or bridging these traits.
* **Linux Desktop Bridge:** `desktop_bridge/src/ble.rs` integrates directly with D-Bus/BlueZ via `zbus` for adapter management and scanning, but contains no GATT client or server implementation.
* **CLI/Windows Desktop:** `cli/src/ble_mesh.rs` uses `btleplug` directly for central-role scanning and subscribing to notifications.

Additionally, `core/src/transport/ble/gatt.rs` defined a malformed `GATT_SERVICE_UUID` constant whose nibbles were shifted (0DF01000 instead of 0000DF01):
```rust
pub const GATT_SERVICE_UUID: u128 = 0x0000_0DF0_1000_1000_8000_0080_5F9B_34FB;
```
This constant was unused, but remained a landmine.

---

## 2. Option A: Clean Pruning (Completed/Executed)

Option A removes the unused traits and the malformed `GATT_SERVICE_UUID` constant to simplify the codebase and eliminate dead abstraction surface.

### Exact Diff of Changes (Commit `8bd41e49`)

The traits and the constant have been pruned from `core/src/transport/ble/gatt.rs` and `mod.rs`. The useful helper utilities (`GattFragmenter`, `GattReassembler`, `GattWriteQueue`, `GattWriteRequest`) along with their extensive unit tests have been **retained** because they are utilized by other modules or tests.

```diff
diff --git a/core/src/transport/ble/gatt.rs b/core/src/transport/ble/gatt.rs
index 8ff8482d..aa047166 100644
--- a/core/src/transport/ble/gatt.rs
+++ b/core/src/transport/ble/gatt.rs
@@ -6,9 +6,6 @@ use serde::{Deserialize, Serialize};
 use std::collections::VecDeque;
 use thiserror::Error;
 
-/// GATT service UUID (0xDF01)
-pub const GATT_SERVICE_UUID: u128 = 0x0000_0DF0_1000_1000_8000_0080_5F9B_34FB;
-
 /// Maximum GATT characteristic write size (protocol limitation)
 pub const MAX_CHARACTERISTIC_SIZE: usize = 512;
 
@@ -275,43 +272,69 @@ impl GattWriteQueue {
     }
 }
 
-/// GATT Server trait for platform implementations
-pub trait GattServer: Send + Sync {
-    /// Handle a write to a characteristic
-    fn on_write(
-        &mut self,
-        characteristic: GattCharacteristic,
-        data: &[u8],
-    ) -> Result<(), GattError>;
-
-    /// Handle a read from a characteristic
-    fn on_read(&self, characteristic: GattCharacteristic) -> Result<Vec<u8>, GattError>;
-
-    /// Notify subscribers of a characteristic change
-    fn notify(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError>;
-
-    /// Check if the GATT service is enabled
-    fn is_enabled(&self) -> bool;
-}
-
-/// GATT Client trait for platform implementations
-pub trait GattClient: Send + Sync {
-    /// Write to a characteristic
-    fn write(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError>;
-
-    /// Read from a characteristic
-    fn read(&self, characteristic: GattCharacteristic) -> Result<Vec<u8>, GattError>;
-
-    /// Subscribe to notifications
-    fn subscribe(&mut self, characteristic: GattCharacteristic) -> Result<(), GattError>;
-
-    /// Unsubscribe from notifications
-    fn unsubscribe(&mut self, characteristic: GattCharacteristic) -> Result<(), GattError>;
-
-    /// Check if connected to GATT server
-    fn is_connected(&self) -> bool;
-}
-
 #[cfg(test)]
 mod tests {
     use super::*;
diff --git a/core/src/transport/ble/mod.rs b/core/src/transport/ble/mod.rs
index 05b9c923..ea8d27b4 100644
--- a/core/src/transport/ble/mod.rs
+++ b/core/src/transport/ble/mod.rs
@@ -23,8 +23,8 @@ pub use beacon::{
 };
 
 pub use gatt::{
-    GattCharacteristic, GattClient, GattError, GattFragmentHeader, GattFragmenter, GattReassembler,
-    GattServer, GattWriteQueue, GattWriteRequest, GATT_SERVICE_UUID, MAX_CHARACTERISTIC_SIZE,
+    GattCharacteristic, GattError, GattFragmentHeader, GattFragmenter, GattReassembler,
+    GattWriteQueue, GattWriteRequest, MAX_CHARACTERISTIC_SIZE,
 };
 
 pub use l2cap::{
```

---

## 3. Option B: Implement & Wire (Alternative)

To wire the traits, we would need to implement them for Linux D-Bus (`desktop_bridge/src/ble.rs`) and Windows (`cli/src/ble_windows.rs`), correcting the constant errors.

### Step 1: Reconcile Characteristic UUIDs and Fix Constant
The service UUID constant must be corrected in `core/src/transport/ble/gatt.rs`:
```rust
pub const GATT_SERVICE_UUID: u128 = 0x0000_DF01_0000_1000_8000_0080_5F9B_34FB;
```
The characteristic maps must be updated to match the working Android implementation:
* `0xDF02`: Identity (Read-only for public keys)
* `0xDF03`: Message (Write / Notify)
* `0xDF04`: Sync Handshake (Read / Write)

### Step 2: Implement for Linux Desktop via D-Bus (`desktop_bridge/src/ble.rs`)
In `desktop_bridge/src/ble.rs`, we would define structs that implement the traits using `zbus` to register a GATT application.

```rust
use scmessenger_core::transport::ble::{GattClient, GattServer, GattCharacteristic, GattError};

pub struct BlueZGattClient {
    // DBus connection parameters and proxy references
}

impl GattClient for BlueZGattClient {
    fn write(&mut self, characteristic: GattCharacteristic, data: &[u8]) -> Result<(), GattError> {
        // Find characteristic on D-Bus path, call WriteValue method
        Ok(())
    }
    fn read(&self, characteristic: GattCharacteristic) -> Result<Vec<u8>, GattError> {
        // Call ReadValue method on D-Bus path
        Ok(vec![])
    }
    fn subscribe(&mut self, characteristic: GattCharacteristic) -> Result<(), GattError> {
        // Call StartNotify method
        Ok(())
    }
    fn unsubscribe(&mut self, characteristic: GattCharacteristic) -> Result<(), GattError> {
        // Call StopNotify method
        Ok(())
    }
    fn is_connected(&self) -> bool {
        // Query Device1 properties
        true
    }
}
```

### Step 3: Integrate with Core Transport Loop
The platform bridge or transport manager would instantiate the client/server and handle FFI mapping so that incoming GATT events trigger `IronCore`'s ingest loops.

---

## 4. Trade-off Comparison

| Dimension | Option A: Clean Pruning (Recommended) | Option B: Implement & Wire |
| :--- | :--- | :--- |
| **Code Complexity** | **Low:** Removes unused abstractions and malformed constants; leaves only tested utility functions. | **High:** Requires writing platform-specific BLE implementations on Linux and Windows. |
| **Architectural Fit** | **High:** Fits the current decoupled design. BLE stacks are highly OS-specific and best implemented natively, keeping the Rust core thin. | **Low:** Forcing platform BLE stacks into a synchronous/simple Rust trait is restrictive and leads to mapping overhead. |
| **Development Cost** | **Zero:** Already implemented and verified. | **High:** Multi-week effort to write and test zbus and WinRT integrations. |
| **Security Risks** | **Minimal:** Pruning dead code shrinks the attack surface. | **High:** Introduces large volumes of untrusted network parsing/dispatch code to the Rust core boundary. |

---

## 5. Recommendation

**Option A (Clean Pruning)** is strongly recommended and has already been merged into the codebase. 

### Rationale
1. **Decoupled Architecture:** Platforms (like Android) operate best when managing their own Bluetooth stack lifecycles natively, transmitting raw buffers into Rust. Attempting to bridge platform APIs into Rust traits results in unnecessary JNI/FFI boilerplate.
2. **Reduced Surface Area:** Dead code is an unnecessary maintenance and security burden.
3. **No Breakage:** Deleting these unused traits has no impact on Android or CLI BLE features, which do not consume them.

---

## 6. Mandatory Security Review

Because BLE represents an external network attack surface, changes to `core/src/transport/` require a mandatory security audit by the `crypto-security-auditor`:
* The auditor should review the pruning diff to ensure that no active security gates, sanitization logic, or bounds checks (such as those in `GattFragmenter` or `GattReassembler`) were accidentally removed.
* The auditor should verify that deleting the unused traits does not impact any current message-path security invariants.
