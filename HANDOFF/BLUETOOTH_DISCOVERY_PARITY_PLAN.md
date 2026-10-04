# Bluetooth Discovery & Implementation Parity Plan

<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->

**Date**: 2026-05-06  
**Goal**: Achieve full Bluetooth discovery and messaging parity between Windows CLI and Android

---

## Executive Summary

**Current State**:
-  **Android**: Full BLE implementation (GATT Server + Client, Scanner, Advertiser)
- ️ **Windows CLI**: Partial BLE (scanning only, no advertising, no peer discovery)

**Gap**: Windows CLI can scan for BLE devices but cannot:
1. Advertise itself as a BLE peripheral
2. Discover Android peers via BLE
3. Exchange messages over BLE

**Root Cause**: `btleplug` library (used on Windows) is **central-only** - it doesn't support peripheral advertising on desktop platforms.

---

## Current Implementation Analysis

### Android BLE Stack ( Complete)

**Files**:
- `BleGattServer.kt` - GATT server (peripheral role)
- `BleGattClient.kt` - GATT client (central role)
- `BleScanner.kt` - Active scanning for peripherals
- `BleAdvertiser.kt` - BLE advertising
- `BleL2capManager.kt` - L2CAP socket management
- `BleQuotaManager.kt` - Rate limiting
- `BleBackoffStrategy.kt` - Retry logic

**Service UUID**: `0000df01-0000-1000-8000-00805f9b34fb`

**Characteristics**:
- `0xDF02` - Identity beacon (READ) - Public key + node info
- `0xDF03` - Message exchange (WRITE/NOTIFY) - Encrypted frames
- `0xDF04` - Sync handshake (READ/WRITE) - Drift protocol

**Features**:
-  GATT server advertising
-  GATT client scanning
-  MTU negotiation (up to 512 bytes)
-  Message fragmentation/reassembly
-  Connection pooling (max 5)
-  Identity beacon broadcasting
-  Bidirectional messaging
-  Graceful error handling

### Windows CLI BLE Stack (️ Partial)

**Files**:
- `cli/src/ble_mesh.rs` - GATT central (scanning only)
- `cli/src/ble_daemon.rs` - BLE adapter management

**Library**: `btleplug` (Rust BLE library)

**Current Capabilities**:
-  Adapter detection
-  Scanning for SCM service UUID
-  Connecting to peripherals
-  Reading characteristics
-  Subscribing to notifications
-  Receiving messages from Android

**Missing Capabilities**:
-  **Advertising** (btleplug doesn't support peripheral mode on desktop)
-  **GATT Server** (cannot act as peripheral)
-  **Peer discovery** (Android can't discover Windows)
-  **Bidirectional messaging** (Windows → Android works, Android → Windows doesn't)

**Code Evidence**:
```rust
// cli/src/ble_mesh.rs:238
pub async fn run_ble_peripheral_advertising(_core: Arc<IronCore>) {
    tracing::info!(
        "BLE: GATT advertising stub started for service {:x} (Awaiting full platform advertising support).",
        GATT_SERVICE_UUID
    );
    // Does nothing - just sleeps forever
}
```

---

## The btleplug Limitation

**Problem**: `btleplug` is designed for **central role only** (scanning and connecting to peripherals). It does **not** support:
- Peripheral advertising
- GATT server creation
- Characteristic serving

**Why**: Desktop Bluetooth stacks (Windows, macOS, Linux) have limited/inconsistent peripheral mode APIs compared to mobile platforms.

**Evidence**: 
- btleplug GitHub issues confirm no peripheral support
- Windows BLE APIs require UWP for peripheral mode
- Linux BlueZ has peripheral support but requires root/capabilities

---

## Solution Options

### Option 1: Platform-Specific BLE Implementation (Recommended)

**Approach**: Use native Windows BLE APIs for peripheral mode

**Windows BLE Stack**:
- **WinRT Bluetooth APIs** (via `windows` crate)
- Supports GATT server creation
- Supports advertising
- Requires Windows 10+

**Implementation**:
1. Add `windows` crate dependency
2. Create `cli/src/ble_windows.rs` with WinRT GATT server
3. Implement advertising via `BluetoothLEAdvertisementPublisher`
4. Implement GATT server via `GattServiceProvider`
5. Keep `btleplug` for scanning (central role)

**Pros**:
- Native Windows support
- Full peripheral capabilities
- Better performance
- Consistent with Android approach (platform APIs)

**Cons**:
- Windows-specific code
- More complex implementation
- Requires Windows 10+

**Estimated LoC**: ~400-600 lines

---

### Option 2: Hybrid Approach (Quick Win)

**Approach**: Use mDNS/TCP for discovery, BLE for messaging only

**Strategy**:
1. Windows and Android discover each other via mDNS (already working)
2. Once discovered, establish BLE connection for low-latency messaging
3. Windows acts as BLE central, Android as peripheral
4. Unidirectional BLE: Windows → Android only
5. Bidirectional TCP/IP for full messaging

**Pros**:
- Works with current btleplug
- No platform-specific code
- Quick to implement

**Cons**:
- Not true BLE discovery
- Requires mDNS to work first
- Asymmetric (Windows can't be discovered via BLE alone)

**Estimated LoC**: ~50-100 lines

---

### Option 3: Wait for btleplug Peripheral Support

**Approach**: Monitor btleplug development for peripheral support

**Status**: 
- btleplug maintainers aware of the limitation
- No timeline for peripheral support
- Would require significant refactoring

**Pros**:
- Cross-platform when available
- Consistent API

**Cons**:
- Indefinite wait
- May never happen
- Blocks BLE parity

**Estimated LoC**: N/A (waiting)

---

### Option 4: Alternative BLE Library

**Approach**: Use a different Rust BLE library with peripheral support

**Options**:
- `bluer` (Linux only, BlueZ bindings)
- `bluest` (experimental, limited platform support)
- Direct FFI to platform APIs

**Pros**:
- Potentially cross-platform
- Rust-native

**Cons**:
- Limited maturity
- Platform-specific anyway
- More dependencies

**Estimated LoC**: High (research + implementation, ~500+ lines)

---

## Recommended Implementation Plan

### Phase 1: Windows GATT Server (Native APIs)

**Goal**: Enable Windows CLI to advertise and act as BLE peripheral

**Tasks**:

1. **Add Windows BLE Dependencies**
   ```toml
   [target.'cfg(windows)'.dependencies]
   windows = { version = "0.52", features = [
       "Devices_Bluetooth",
       "Devices_Bluetooth_Advertisement",
       "Devices_Bluetooth_GenericAttributeProfile",
       "Foundation",
       "Storage_Streams"
   ]}
   ```

2. **Create `cli/src/ble_windows.rs`**
   - Implement `WindowsGattServer` struct
   - Create GATT service with SCM UUID
   - Add characteristics (Identity, Message, Sync)
   - Handle read/write requests
   - Implement fragmentation/reassembly

3. **Implement BLE Advertising**
   - Use `BluetoothLEAdvertisementPublisher`
   - Advertise SCM service UUID
   - Include identity in manufacturer data
   - Handle start/stop

4. **Update `cli/src/main.rs`**
   - Conditionally compile Windows BLE server
   - Start advertising on daemon start
   - Integrate with existing message handling

5. **Testing**
   - Verify Android can discover Windows
   - Test bidirectional messaging
   - Verify MTU negotiation
   - Test connection stability

**Estimated LoC**: ~300-400 lines

---

### Phase 2: Discovery Integration

**Goal**: Ensure BLE discovery works alongside mDNS/DHT

**Tasks**:

1. **Update Discovery Status API**
   - Report BLE advertising status
   - Show discovered BLE peers separately
   - Indicate transport type (BLE vs mDNS vs DHT)

2. **Peer Deduplication**
   - Same peer discovered via multiple transports
   - Merge peer records by identity
   - Prefer faster transport (BLE > mDNS > DHT)

3. **Connection Management**
   - Maintain BLE connections alongside TCP
   - Route messages via best available transport
   - Fallback when BLE unavailable

4. **UI Updates**
   - Show BLE status in CLI
   - Display transport type per peer
   - Indicate connection quality

**Estimated LoC**: ~100-150 lines

---

### Phase 3: Parity Testing & Optimization

**Goal**: Verify full parity and optimize performance

**Tasks**:

1. **Cross-Platform Testing**
   - Windows ↔ Android BLE discovery
   - Windows ↔ Android BLE messaging
   - Multiple simultaneous connections
   - Connection recovery

2. **Performance Optimization**
   - MTU negotiation tuning
   - Fragmentation optimization
   - Connection pooling
   - Battery impact (Android)

3. **Error Handling**
   - Bluetooth disabled
   - Permission denied
   - Adapter not available
   - Connection failures

4. **Documentation**
   - Update README with BLE requirements
   - Document Windows 10+ requirement
   - Add troubleshooting guide

**Estimated LoC**: ~50-100 lines (error handling + docs)

---

## Implementation Details

### Windows GATT Server Structure

```rust
// cli/src/ble_windows.rs

use windows::Devices::Bluetooth::Advertisement::*;
use windows::Devices::Bluetooth::GenericAttributeProfile::*;

pub struct WindowsGattServer {
    service_provider: Option<GattServiceProvider>,
    advertiser: Option<BluetoothLEAdvertisementPublisher>,
    identity_characteristic: Option<GattLocalCharacteristic>,
    message_characteristic: Option<GattLocalCharacteristic>,
    sync_characteristic: Option<GattLocalCharacteristic>,
}

impl WindowsGattServer {
    pub async fn new() -> Result<Self> {
        // Create GATT service
        let service_uuid = Guid::from("0000df01-0000-1000-8000-00805f9b34fb");
        let provider = GattServiceProvider::CreateAsync(service_uuid).await?;
        
        // Add characteristics
        let identity_char = create_identity_characteristic(&provider).await?;
        let message_char = create_message_characteristic(&provider).await?;
        let sync_char = create_sync_characteristic(&provider).await?;
        
        Ok(Self {
            service_provider: Some(provider),
            advertiser: None,
            identity_characteristic: Some(identity_char),
            message_characteristic: Some(message_char),
            sync_characteristic: Some(sync_char),
        })
    }
    
    pub async fn start_advertising(&mut self) -> Result<()> {
        let advertiser = BluetoothLEAdvertisementPublisher::new()?;
        
        // Configure advertisement
        let advertisement = advertiser.Advertisement()?;
        advertisement.LocalName()?.SetValue("SCMessenger")?;
        
        // Add service UUID
        let service_uuid = Guid::from("0000df01-0000-1000-8000-00805f9b34fb");
        advertisement.ServiceUuids()?.Append(service_uuid)?;
        
        // Start advertising
        advertiser.Start()?;
        self.advertiser = Some(advertiser);
        
        Ok(())
    }
}
```

### Characteristic Handlers

```rust
async fn create_identity_characteristic(
    provider: &GattServiceProvider
) -> Result<GattLocalCharacteristic> {
    let char_uuid = Guid::from("0000df02-0000-1000-8000-00805f9b34fb");
    
    let params = GattLocalCharacteristicParameters::new()?;
    params.SetCharacteristicProperties(
        GattCharacteristicProperties::Read
    )?;
    params.SetReadProtectionLevel(
        GattProtectionLevel::Plain
    )?;
    
    let result = provider.Service()?
        .CreateCharacteristicAsync(char_uuid, params)?
        .await?;
    
    let characteristic = result.Characteristic()?;
    
    // Register read handler
    characteristic.ReadRequested(
        TypedEventHandler::new(|sender, args| {
            // Return identity beacon
            let identity_data = get_identity_beacon();
            args.GetDeferral()?.Complete()?;
            Ok(())
        })
    )?;
    
    Ok(characteristic)
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_windows_gatt_server_creation() {
        // Test GATT server initialization
    }
    
    #[test]
    fn test_characteristic_read() {
        // Test identity beacon read
    }
    
    #[test]
    fn test_message_fragmentation() {
        // Test large message handling
    }
}
```

### Integration Tests

1. **Discovery Test**
   - Start Windows CLI with BLE
   - Start Android app
   - Verify mutual discovery within 10 seconds

2. **Messaging Test**
   - Send message Windows → Android
   - Send message Android → Windows
   - Verify delivery and decryption

3. **Connection Test**
   - Establish BLE connection
   - Disconnect and reconnect
   - Verify state recovery

4. **Stress Test**
   - Multiple simultaneous connections
   - Large message throughput
   - Connection churn

---

## Success Criteria

### Functional Requirements

- [ ] Windows CLI advertises BLE service
- [ ] Android discovers Windows via BLE scan
- [ ] Windows discovers Android via BLE scan
- [ ] Bidirectional messaging works over BLE
- [ ] MTU negotiation succeeds (512 bytes)
- [ ] Message fragmentation/reassembly works
- [ ] Connection pooling (max 5) enforced
- [ ] Graceful degradation when BLE unavailable

### Performance Requirements

- [ ] Discovery latency < 10 seconds
- [ ] Message delivery latency < 500ms
- [ ] Throughput > 10 KB/s per connection
- [ ] Battery impact < 5% per hour (Android)
- [ ] Memory usage < 50 MB additional

### Compatibility Requirements

- [ ] Windows 10+ (1809 or later)
- [ ] Android 8.0+ (API 26+)
- [ ] Works alongside mDNS/DHT discovery
- [ ] No conflicts with existing transports

---

## Risks & Mitigation

### Risk 1: Windows BLE API Complexity

**Impact**: High  
**Probability**: Medium  
**Mitigation**: 
- Start with minimal implementation
- Reference existing WinRT Bluetooth examples
- Use `windows` crate documentation

### Risk 2: Permission Issues

**Impact**: Medium  
**Probability**: High  
**Mitigation**:
- Clear error messages
- Documentation on enabling Bluetooth
- Graceful fallback to other transports

### Risk 3: Platform Fragmentation

**Impact**: Medium  
**Probability**: Medium  
**Mitigation**:
- Conditional compilation per platform
- Feature flags for BLE support
- Runtime capability detection

### Risk 4: Performance Issues

**Impact**: Low  
**Probability**: Low  
**Mitigation**:
- Benchmark early
- Optimize MTU usage
- Implement connection pooling

---

## Alternative: Quick Win (Hybrid Approach)

If full Windows GATT server is too complex, implement this simpler approach:

### Hybrid Discovery Strategy

1. **Primary Discovery**: mDNS (already working after our fix)
2. **Secondary Discovery**: DHT via bootstrap nodes
3. **BLE Role**: Windows = Central only, Android = Peripheral only
4. **Message Flow**: 
   - Windows → Android: via BLE (fast, low latency)
   - Android → Windows: via TCP/IP (reliable, bidirectional)

### Implementation

```rust
// cli/src/ble_hybrid.rs

pub struct HybridBleTransport {
    // Use btleplug for scanning only
    scanner: BleScanner,
    // Use TCP for receiving from Android
    tcp_listener: TcpListener,
}

impl HybridBleTransport {
    pub async fn discover_peers(&mut self) -> Vec<Peer> {
        // 1. Scan for BLE peripherals (Android devices)
        let ble_peers = self.scanner.scan().await;
        
        // 2. For each BLE peer, establish TCP connection
        for peer in ble_peers {
            let tcp_addr = resolve_tcp_address(&peer).await;
            self.tcp_listener.connect(tcp_addr).await;
        }
        
        // 3. Use BLE for Windows → Android (low latency)
        // 4. Use TCP for Android → Windows (reliable)
    }
}
```

**Pros**:
- Works with current btleplug
- No Windows-specific code
- Quick to implement (1 day)

**Cons**:
- Not pure BLE
- Requires mDNS to work
- Asymmetric transport

**Estimated LoC**: ~50-100 lines

---

## Recommendation

**Implement Option 1 (Windows GATT Server)** for full parity:

1. **Week 1**: Implement Windows GATT server with native APIs
2. **Week 2**: Integration testing and optimization
3. **Week 3**: Documentation and deployment

**Fallback**: If Windows APIs prove too complex, implement Hybrid Approach as interim solution (~50-100 LoC).

---

## Next Steps

1. **Immediate**: Review this plan with team
2. **Phase 1**: Set up Windows BLE development environment
3. **Phase 2**: Implement basic GATT server (~200 LoC)
4. **Phase 3**: Add advertising and characteristics (~200 LoC)
5. **Phase 4**: Integration testing with Android
6. **Phase 5**: Optimization and documentation (~100 LoC)

---

**Plan Status**: Ready for Implementation  
**Estimated Total LoC**: ~450-650 lines (Option 1) or ~50-100 lines (Option 2)  
**Priority**: High (blocks full cross-platform discovery)  
**Dependencies**: Windows 10+ requirement acceptable
