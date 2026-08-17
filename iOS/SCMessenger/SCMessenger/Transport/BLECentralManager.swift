//
//  BLECentralManager.swift
//  SCMessenger
//
//  Scans for and connects to BLE mesh peers
//  Mirrors: android/.../transport/ble/BleScanner.kt + BleGattClient.kt
//

import CoreBluetooth
import CryptoKit
import Foundation
import os

/// Scans for and connects to BLE mesh peers (iOS Central role)
///
/// Responsibilities:
/// - Duty-cycled BLE scanning for mesh service
/// - Connect to discovered peripherals
/// - GATT client operations (read/write characteristics)
/// - Write queue management (mirrors Android pattern)
/// - State restoration for background operation
final class BLECentralManager: NSObject {
    private let logger: Logger = Logger(subsystem: "com.scmessenger", category: "BLE-Central")
    private var centralManager: CBCentralManager!
    private weak var meshRepository: MeshRepository?

    // Peripheral tracking
    private var discoveredPeripherals: [UUID: CBPeripheral] = [:]
    private var connectedPeripherals: [UUID: CBPeripheral] = [:]
    private var peerCache: [UUID: Date] = [:] // Dedup cache

    // Scanning parameters
    private var scanInterval: TimeInterval = MeshBLEConstants.defaultScanInterval
    private var scanWindow: TimeInterval = MeshBLEConstants.defaultScanWindow
    private var isBackgroundMode: Bool = false
    private var scanTimer: Timer?
    private var isScanning: Bool = false
    private var pendingScanOnReady: Bool = false  // P3: Defer scan until BLE is poweredOn

    // Write queue (mirrors Android BleGattClient pattern - CRITICAL)
    private var writeInProgress: [UUID: Bool] = [:]
    private var pendingWrites: [UUID: [Data]] = [:]
    private var inFlightWrites: [UUID: Data] = [:]
    private var writeRetryCounts: [UUID: Int] = [:]
    private let maxWriteAttempts = 3
    private let maxWriteWithoutResponseBurst = 16

    // Reassembly buffers per peripheral
    private var reassemblyBuffers: [UUID: [Int: Data]] = [:]
    private var expectedFragments: [UUID: Int] = [:]
    private let maxReassemblyFragments = 1024
    private let maxReassemblyBytes = 1_048_576

    // Characteristics cache (names match Android BleGattServer)
    private var messageCharacteristics: [UUID: CBCharacteristic] = [:] // Write: central → peripheral
    private var syncCharacteristics: [UUID: CBCharacteristic] = [:]    // Notify: peripheral → central
    // CoreBluetooth reports the result of the CCCD write asynchronously. Keep
    // the request state separate from the confirmed state so Android can rely
    // on an actual subscription before sending notifications.
    private var messageNotifyAttempts: [UUID: Int] = [:]
    private let maxMessageNotifyAttempts = 3
    
    // Connection state monitoring and auto-reconnection
    private var connectionRetries: [UUID: Int] = [:]
    private var reconnectionTimers: [UUID: Timer] = [:]
    private let maxReconnectionAttempts: Int = 3
    private let reconnectionDelay: TimeInterval = 2.0
    private var intentionalDisconnects: Set<UUID> = []

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        centralManager = CBCentralManager(
            delegate: self,
            // Keep mutable connection dictionaries on one queue to avoid races
            // with send paths invoked from repository/main actor code.
            queue: .main,
            options: [CBCentralManagerOptionRestoreIdentifierKey: MeshBLEConstants.centralRestoreId]
        )
    }

    // MARK: - Public API

    func startScanning() {
        logger.info("Starting BLE scanning")
        guard centralManager.state == .poweredOn else {
            logger.warning("Cannot start scanning: BLE not powered on (state=\(self.centralManager.state.rawValue)), will auto-start when ready")
            // P3: Don't log as failure — just defer until BLE is ready
            pendingScanOnReady = true
            if self.centralManager.state == .unknown {
                // State .unknown means CBCentralManager hasn't reported yet — this is normal at launch.
                // Scanning will begin automatically when centralManagerDidUpdateState fires with .poweredOn.
                return
            }
            appendRepositoryDiagnostic("ble_central_start_deferred state=\(self.centralManager.state.rawValue)")
            return
        }
        pendingScanOnReady = false
        appendRepositoryDiagnostic("ble_central_scan_start")
        scheduleDutyCycle()
    }

    func stopScanning() {
        logger.info("Stopping BLE scanning")
        scanTimer?.invalidate()
        scanTimer = nil
        centralManager.stopScan()
        isScanning = false
        disconnectAll()
    }

    func setBackgroundMode(_ background: Bool) {
        isBackgroundMode = background
        logger.info("Background mode: \(background)")
    }

    func applyScanSettings(intervalMs: UInt32) {
        scanInterval = TimeInterval(intervalMs) / 1000.0
        logger.debug("Scan interval updated: \(self.scanInterval)s")
    }

    @discardableResult
    func sendData(to peripheralId: UUID, data: Data) -> Bool {
        guard !data.isEmpty else {
            logger.warning("Cannot send empty BLE frame to \(peripheralId)")
            return false
        }
        guard let peripheral = connectedPeripherals[peripheralId] else {
            if let discovered = discoveredPeripherals[peripheralId] {
                logger.warning("Cannot send: peripheral \(peripheralId) not connected, reconnecting")
                attemptReconnection(to: discovered)
                appendRepositoryDiagnostic("ble_central_reconnect_requested id=\(peripheralId)")
            } else {
                logger.error("Cannot send: peripheral \(peripheralId) not connected and not discovered")
            }
            return false
        }
        
        // Validate connection state before proceeding
        guard validateConnectionState(for: peripheral) else {
            attemptReconnection(to: peripheral)
            return false
        }
        
        guard let messageCharacteristic = messageCharacteristics[peripheralId],
              messageCharacteristic.properties.contains(.write) ||
                messageCharacteristic.properties.contains(.writeWithoutResponse) else {
            logger.warning("Cannot send: Message characteristic missing for \(peripheralId), rediscovering")
            peripheral.discoverServices([MeshBLEConstants.serviceUUID])
            return false
        }

        let mtu = peripheral.maximumWriteValueLength(for: .withResponse)
        guard mtu > 4 else {
            logger.warning("Cannot send: negotiated BLE write MTU \(mtu) is too small for framing")
            return false
        }
        let fragments = fragmentData(data, mtu: mtu)
        guard !fragments.isEmpty else {
            logger.warning("Cannot fragment BLE frame for \(peripheralId)")
            return false
        }

        appendRepositoryDiagnostic(
            "ble_central_tx_start fragments=\(fragments.count) bytes=\(data.count) " +
                "payload_sha256_64=\(payloadFingerprint(data)) to=\(peripheralId.uuidString.prefix(8))"
        )
        for fragment in fragments {
            enqueueFragment(fragment, for: peripheralId)
        }
        return true
    }

    func connectedPeripheralIds() -> [String] {
        connectedPeripherals.keys.compactMap { peripheralId in
            guard messageCharacteristics[peripheralId] != nil else { return nil }
            return peripheralId.uuidString
        }
    }

    private func appendRepositoryDiagnostic(_ message: String) {
        let meshRepository = self.meshRepository
        Task { @MainActor in
            meshRepository?.appendDiagnostic(message)
        }
    }

    private func payloadFingerprint(_ data: Data) -> String {
        SHA256.hash(data: data)
            .prefix(8)
            .map { String(format: "%02x", $0) }
            .joined()
    }
    
    // MARK: - Connection State Monitoring and Auto-Reconnection
    
    private func attemptReconnection(to peripheral: CBPeripheral) {
        let peripheralId = peripheral.identifier
        
        // Cancel any existing reconnection timer
        reconnectionTimers[peripheralId]?.invalidate()
        reconnectionTimers.removeValue(forKey: peripheralId)
        
        // Increment retry count
        let retryCount = (connectionRetries[peripheralId] ?? 0) + 1
        connectionRetries[peripheralId] = retryCount
        
        if retryCount > maxReconnectionAttempts {
            logger.warning("Max reconnection attempts (\(self.maxReconnectionAttempts)) reached for \(peripheralId), giving up")
            connectionRetries.removeValue(forKey: peripheralId)
            return
        }
        
        logger.info("Attempting reconnection \(retryCount)/\(self.maxReconnectionAttempts) to \(peripheralId)")
        appendRepositoryDiagnostic("ble_central_reconnect_attempt attempt=\(retryCount) id=\(peripheralId)")
        
        // Attempt immediate connection
        centralManager.connect(peripheral, options: nil)
        
        // Schedule next retry if this fails
        scheduleReconnectionRetry(for: peripheral)
    }
    
    private func scheduleReconnectionRetry(for peripheral: CBPeripheral) {
        let peripheralId = peripheral.identifier
        
        // Cancel any existing timer first
        reconnectionTimers[peripheralId]?.invalidate()
        
        // Schedule retry with exponential backoff
        let retryDelay = reconnectionDelay * pow(2.0, Double(connectionRetries[peripheralId] ?? 1))
        
        let timer = Timer.scheduledTimer(withTimeInterval: retryDelay, repeats: false) { [weak self] _ in
            guard let self = self else { return }
            
            // Check if we're still not connected
            if self.connectedPeripherals[peripheralId] == nil {
                let currentRetryCount = self.connectionRetries[peripheralId] ?? 0
                if currentRetryCount <= self.maxReconnectionAttempts {
                    self.logger.info("Reconnection retry \(currentRetryCount) for \(peripheralId)")
                    self.centralManager.connect(peripheral, options: nil)
                    self.scheduleReconnectionRetry(for: peripheral) // Schedule next retry if needed
                }
            } else {
                // Connected successfully, clean up
                self.cleanupReconnectionState(for: peripheralId)
            }
        }
        
        reconnectionTimers[peripheralId] = timer
        logger.debug("Scheduled reconnection retry in \\(retryDelay)s for \\(peripheralId)")
    }
    
    private func cleanupReconnectionState(for peripheralId: UUID) {
        reconnectionTimers[peripheralId]?.invalidate()
        reconnectionTimers.removeValue(forKey: peripheralId)
        connectionRetries.removeValue(forKey: peripheralId)
        logger.debug("Cleaned up reconnection state for \\(peripheralId)")
    }
    
    private func validateConnectionState(for peripheral: CBPeripheral) -> Bool {
        if peripheral.state != .connected {
            logger.warning("Peripheral \\(peripheral.identifier) not in connected state: \\(peripheral.state.rawValue)")
            return false
        }
        return true
    }

    private func enqueueFragment(_ fragment: Data, for peripheralId: UUID) {
        guard connectedPeripherals[peripheralId] != nil,
              messageCharacteristics[peripheralId] != nil else { return }

        pendingWrites[peripheralId, default: []].append(fragment)
        drainWriteQueue(for: peripheralId)
    }

    private func drainWriteQueue(for peripheralId: UUID) {
        guard writeInProgress[peripheralId] != true,
              let peripheral = connectedPeripherals[peripheralId],
              let characteristic = messageCharacteristics[peripheralId] else { return }

        if characteristic.properties.contains(.write) {
            guard let fragment = pendingWrites[peripheralId]?.first else { return }
            pendingWrites[peripheralId]?.removeFirst()
            writeInProgress[peripheralId] = true
            inFlightWrites[peripheralId] = fragment
            peripheral.writeValue(fragment, for: characteristic, type: .withResponse)
        } else if characteristic.properties.contains(.writeWithoutResponse) {
            var sentCount = 0
            while sentCount < maxWriteWithoutResponseBurst,
                  pendingWrites[peripheralId]?.isEmpty == false {
                guard peripheral.canSendWriteWithoutResponse else {
                    return
                }
                guard let fragment = pendingWrites[peripheralId]?.first else { return }
                pendingWrites[peripheralId]?.removeFirst()
                peripheral.writeValue(fragment, for: characteristic, type: .withoutResponse)
                sentCount += 1
            }

            guard pendingWrites[peripheralId]?.isEmpty == false else { return }
            guard peripheral.canSendWriteWithoutResponse else { return }
            DispatchQueue.main.async { [weak self, weak peripheral] in
                guard let self, let peripheral,
                      peripheral.state == .connected,
                      peripheral.canSendWriteWithoutResponse else { return }
                self.drainWriteQueue(for: peripheralId)
            }
        } else {
            pendingWrites[peripheralId]?.removeAll()
        }
    }

    private func fragmentData(_ data: Data, mtu: Int) -> [Data] {
        guard !data.isEmpty else { return [] }
        let maxChunk = min(MeshBLEConstants.maxChunkSize, mtu)
        let maxPayload = maxChunk - 4
        if maxPayload <= 0 { return [] }

        let totalFragments = Int(ceil(Double(data.count) / Double(maxPayload)))
        guard totalFragments <= Int(UInt16.max) else { return [] }
        var fragments: [Data] = []

        for i in 0..<totalFragments {
            let start = i * maxPayload
            let end = min(start + maxPayload, data.count)
            let chunk = data.subdata(in: start..<end)

            var header = Data(count: 4)
            header[0] = UInt8(totalFragments & 0xFF)
            header[1] = UInt8((totalFragments >> 8) & 0xFF)
            header[2] = UInt8(i & 0xFF)
            header[3] = UInt8((i >> 8) & 0xFF)

            fragments.append(header + chunk)
        }
        return fragments
    }

    /// Broadcast data to all connected peripherals.
    func broadcastData(_ data: Data) {
        for peripheralId in connectedPeripherals.keys {
            sendData(to: peripheralId, data: data)
        }
    }

    // MARK: - Private Methods

    private func scheduleDutyCycle() {
        // Timer MUST run on the main RunLoop — background dispatch queues don't
        // have a running RunLoop, so Timer.scheduledTimer would silently never fire.
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            self.scanTimer?.invalidate()
            self.scanTimer = Timer.scheduledTimer(withTimeInterval: self.scanInterval, repeats: true) { [weak self] _ in
                self?.performScanCycle()
            }
            if let scanTimer = self.scanTimer {
                RunLoop.main.add(scanTimer, forMode: .common)
            }
            self.performScanCycle() // Start immediately
        }
    }

    private func performScanCycle() {
        if isBackgroundMode {
            // Background: duty-cycle to preserve battery
            if !isScanning {
                startScan()
                DispatchQueue.global(qos: .utility).asyncAfter(deadline: .now() + scanWindow) { [weak self] in
                    self?.stopScan()
                }
            }
        } else {
            // Foreground: scan continuously — never stop between cycles so we
            // don't miss advertisement windows during active use/testing.
            if !isScanning {
                startScan()
            }
        }
    }

    private func startScan() {
        let options: [String: Any] = isBackgroundMode ? [:] : [CBCentralManagerScanOptionAllowDuplicatesKey: true]
        centralManager.scanForPeripherals(
            withServices: [MeshBLEConstants.serviceUUID],
            options: options
        )
        isScanning = true
        logger.debug("Scan started")
    }

    private func stopScan() {
        centralManager.stopScan()
        isScanning = false
        logger.debug("Scan stopped")
    }

    private func disconnectAll() {
        for peripheral in connectedPeripherals.values {
            intentionalDisconnects.insert(peripheral.identifier)
            centralManager.cancelPeripheralConnection(peripheral)
        }
        reconnectionTimers.values.forEach { $0.invalidate() }
        reconnectionTimers.removeAll()
        connectionRetries.removeAll()
        connectedPeripherals.removeAll()
        messageCharacteristics.removeAll()
        syncCharacteristics.removeAll()
        messageNotifyAttempts.removeAll()
        writeInProgress.removeAll()
        pendingWrites.removeAll()
        inFlightWrites.removeAll()
        writeRetryCounts.removeAll()
        reassemblyBuffers.removeAll()
        expectedFragments.removeAll()
    }

    private func cleanupPeerCache() {
        let now = Date()
        peerCache = peerCache.filter { now.timeIntervalSince($0.value) < MeshBLEConstants.peerCacheTimeout }
    }
}

// MARK: - CBCentralManagerDelegate

extension BLECentralManager: CBCentralManagerDelegate {
    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        logger.info("Central manager state: \(central.state.rawValue)")
        if central.state == .poweredOn {
            // P3: If startScanning() was called before BLE was ready, start now
            if pendingScanOnReady {
                logger.info("BLE now powered on — starting deferred scan")
                pendingScanOnReady = false
                appendRepositoryDiagnostic("ble_central_scan_start_deferred")
                scheduleDutyCycle()
            }
        }
    }

    func centralManager(_ central: CBCentralManager, didDiscover peripheral: CBPeripheral, advertisementData: [String: Any], rssi RSSI: NSNumber) {
        logger.debug("Discovered peripheral: \(peripheral.identifier)")

        // Check cache to avoid duplicate processing
        cleanupPeerCache()
        if peerCache[peripheral.identifier] != nil {
            return // Recently processed
        }
        peerCache[peripheral.identifier] = Date()

        // Store and connect
        discoveredPeripherals[peripheral.identifier] = peripheral
        peripheral.delegate = self
        centralManager.connect(peripheral, options: nil)
    }

    func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        logger.info("Connected to \(peripheral.identifier)")
        appendRepositoryDiagnostic("ble_central_connected id=\(peripheral.identifier)")
        intentionalDisconnects.remove(peripheral.identifier)
        cleanupReconnectionState(for: peripheral.identifier)
        connectedPeripherals[peripheral.identifier] = peripheral
        // Request maximum write size (negotiate higher MTU) before discovering services.
        // iOS will use this hint when negotiating the connection's ATT MTU.
        // The actual MTU is determined during service discovery.
        peripheral.discoverServices([MeshBLEConstants.serviceUUID])
    }

    func centralManager(_ central: CBCentralManager, didFailToConnect peripheral: CBPeripheral, error: Error?) {
        logger.error("Failed to connect to \(peripheral.identifier): \(error?.localizedDescription ?? "unknown")")
        appendRepositoryDiagnostic("ble_central_connect_fail id=\(peripheral.identifier) err=\(error?.localizedDescription ?? "none")")
        if intentionalDisconnects.remove(peripheral.identifier) == nil {
            attemptReconnection(to: peripheral)
        }
    }

    func centralManager(_ central: CBCentralManager, didDisconnectPeripheral peripheral: CBPeripheral, error: Error?) {
        logger.info("Disconnected from \(peripheral.identifier)")
        appendRepositoryDiagnostic("ble_central_disconnected id=\(peripheral.identifier) err=\(error?.localizedDescription ?? "none")")
        let wasIntentional = intentionalDisconnects.remove(peripheral.identifier) != nil
        connectedPeripherals.removeValue(forKey: peripheral.identifier)
        messageCharacteristics.removeValue(forKey: peripheral.identifier)
        syncCharacteristics.removeValue(forKey: peripheral.identifier)
        messageNotifyAttempts.removeValue(forKey: peripheral.identifier)
        writeInProgress.removeValue(forKey: peripheral.identifier)
        pendingWrites.removeValue(forKey: peripheral.identifier)
        inFlightWrites.removeValue(forKey: peripheral.identifier)
        writeRetryCounts.removeValue(forKey: peripheral.identifier)
        reassemblyBuffers.removeValue(forKey: peripheral.identifier)
        expectedFragments.removeValue(forKey: peripheral.identifier)
        // Clear the peer cache entry so the peer is immediately eligible for
        // re-discovery and reconnection on the next scan result — without this,
        // the 5-second dedup window prevents reconnecting after a brief drop.
        peerCache.removeValue(forKey: peripheral.identifier)
        
        // Attempt automatic reconnection unless it was intentional disconnection
        if !wasIntentional {
            attemptReconnection(to: peripheral)
        }
    }

    func centralManager(_ central: CBCentralManager, willRestoreState dict: [String: Any]) {
        // State restoration (iOS-specific for background BLE)
        if let peripherals = dict[CBCentralManagerRestoredStatePeripheralsKey] as? [CBPeripheral] {
            logger.info("Restoring \(peripherals.count) peripherals")
            for peripheral in peripherals {
                peripheral.delegate = self
                connectedPeripherals[peripheral.identifier] = peripheral
                peripheral.discoverServices([MeshBLEConstants.serviceUUID])
            }
        }
    }
}

// MARK: - CBPeripheralDelegate

extension BLECentralManager: CBPeripheralDelegate {
    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        if let error = error {
            logger.error("Failed to discover services for \(peripheral.identifier): \(error.localizedDescription)")
            appendRepositoryDiagnostic("ble_central_discover_services_fail id=\(peripheral.identifier) err=\(error.localizedDescription)")
            return
        }

        guard let services = peripheral.services, !services.isEmpty else {
            logger.warning("No services found for \(peripheral.identifier)")
            appendRepositoryDiagnostic("ble_central_no_services id=\(peripheral.identifier)")
            return
        }

        appendRepositoryDiagnostic("ble_central_services_discovered id=\(peripheral.identifier) count=\(services.count)")

        for service in services where service.uuid == MeshBLEConstants.serviceUUID {
            peripheral.discoverCharacteristics([
                MeshBLEConstants.messageCharUUID,
                MeshBLEConstants.syncCharUUID,
                MeshBLEConstants.identityCharUUID
            ], for: service)
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        if let error = error {
            logger.error("Failed to discover characteristics for \(peripheral.identifier): \(error.localizedDescription)")
            appendRepositoryDiagnostic("ble_central_discover_chars_fail id=\(peripheral.identifier) err=\(error.localizedDescription)")
            return
        }

        guard let characteristics = service.characteristics else {
            appendRepositoryDiagnostic("ble_central_no_chars id=\(peripheral.identifier)")
            return
        }

        appendRepositoryDiagnostic("ble_central_chars_discovered id=\(peripheral.identifier) count=\(characteristics.count)")

        for characteristic in characteristics {
            switch characteristic.uuid {
            case MeshBLEConstants.messageCharUUID:
                messageCharacteristics[peripheral.identifier] = characteristic
                requestMessageNotifications(for: peripheral, characteristic: characteristic)
            case MeshBLEConstants.syncCharUUID:
                syncCharacteristics[peripheral.identifier] = characteristic
                appendRepositoryDiagnostic("ble_central_found_sync id=\(peripheral.identifier)")
            case MeshBLEConstants.identityCharUUID:
                appendRepositoryDiagnostic("ble_central_reading_identity id=\(peripheral.identifier)")
                peripheral.readValue(for: characteristic)
                // Schedule retry reads at T+900ms and T+2200ms (mirrors Android
                // IDENTITY_REFRESH_DELAYS_MS) for peripherals whose GATT server
                // may not be fully populated at characteristic discovery time.
                scheduleIdentityRefreshReads(peripheral: peripheral, characteristic: characteristic)
            default:
                break
            }
        }
    }

    private func requestMessageNotifications(for peripheral: CBPeripheral, characteristic: CBCharacteristic) {
        let peripheralId = peripheral.identifier
        guard characteristic.properties.contains(.notify) || characteristic.properties.contains(.indicate) else {
            logger.error("Message characteristic does not support notifications for \(peripheralId)")
            appendRepositoryDiagnostic("ble_central_subscribe_message_fail id=\(peripheralId) reason=notify_not_supported")
            return
        }

        let attempt = (messageNotifyAttempts[peripheralId] ?? 0) + 1
        messageNotifyAttempts[peripheralId] = attempt
        appendRepositoryDiagnostic("ble_central_notify_request id=\(peripheralId) attempt=\(attempt)")
        peripheral.setNotifyValue(true, for: characteristic)
    }

    func peripheral(_ peripheral: CBPeripheral, didUpdateNotificationStateFor characteristic: CBCharacteristic, error: Error?) {
        guard characteristic.uuid == MeshBLEConstants.messageCharUUID else { return }

        let peripheralId = peripheral.identifier
        if let error {
            logger.error("Failed to subscribe to message notifications for \(peripheralId): \(error.localizedDescription)")
            appendRepositoryDiagnostic("ble_central_subscribe_message_fail id=\(peripheralId) err=\(error.localizedDescription)")
        } else if characteristic.isNotifying {
            messageNotifyAttempts.removeValue(forKey: peripheralId)
            appendRepositoryDiagnostic("ble_central_subscribed_message id=\(peripheralId)")
            return
        } else {
            appendRepositoryDiagnostic("ble_central_subscribe_message_fail id=\(peripheralId) reason=not_notifying")
        }

        guard messageNotifyAttempts[peripheralId, default: 0] < maxMessageNotifyAttempts,
              peripheral.state == .connected else {
            appendRepositoryDiagnostic("ble_central_subscribe_message_exhausted id=\(peripheralId)")
            return
        }

        // Android may expose the service before its CCCD is ready. Retry only
        // after CoreBluetooth reports the failed request, preserving GATT's
        // serial operation ordering.
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.35) { [weak self, weak peripheral] in
            guard let self, let peripheral,
                  let characteristic = self.messageCharacteristics[peripheral.identifier] else { return }
            self.requestMessageNotifications(for: peripheral, characteristic: characteristic)
        }
    }

    private func scheduleIdentityRefreshReads(peripheral: CBPeripheral, characteristic: CBCharacteristic) {
        let peripheralId = peripheral.identifier
        for delayNs: UInt64 in [900_000_000, 2_200_000_000] {
            Task { [weak self] in
                try? await Task.sleep(nanoseconds: delayNs)
                await MainActor.run {
                    guard self?.connectedPeripherals[peripheralId] != nil else { return }
                    peripheral.readValue(for: characteristic)
                }
            }
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didUpdateValueFor characteristic: CBCharacteristic, error: Error?) {
        if let error = error {
            logger.error("Characteristic update error for \(characteristic.uuid.shortUUID): \(error.localizedDescription)")
            return
        }
        guard let data = characteristic.value, !data.isEmpty else { return }

        if characteristic.uuid == MeshBLEConstants.identityCharUUID {
            // Parse identity beacon — extract Ed25519 public key, do NOT treat as message data
            logger.debug("Identity beacon from \(peripheral.identifier): \(data.count) bytes")
            if let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
               let publicKeyHex = json["public_key"] as? String,
               publicKeyHex.count == 64 {
                DispatchQueue.main.async { [weak self] in
                    self?.meshRepository?.onPeerIdentityRead(
                        blePeerId: peripheral.identifier.uuidString,
                        info: json
                    )
                }
            } else {
                logger.warning("Could not parse identity beacon from \(peripheral.identifier)")
            }
        } else {
            // Message or sync data — handle reassembly
            if data.count < 4 {
                logger.warning("Received tiny BLE packet (<4 bytes) from \(peripheral.identifier)")
                clearReassembly(for: peripheral.identifier)
                return
            }

            let totalFrags = Int(data[0]) | (Int(data[1]) << 8)
            let fragIndex = Int(data[2]) | (Int(data[3]) << 8)
            guard totalFrags > 0,
                  totalFrags <= maxReassemblyFragments,
                  fragIndex < totalFrags else {
                logger.warning("Invalid BLE fragment header from \(peripheral.identifier)")
                clearReassembly(for: peripheral.identifier)
                return
            }
            let payload = data.subdata(in: 4..<data.count)
            guard payload.count <= maxReassemblyBytes else {
                logger.warning("BLE fragment exceeds reassembly byte limit from \(peripheral.identifier)")
                clearReassembly(for: peripheral.identifier)
                return
            }

            let peripheralID = peripheral.identifier
            if fragIndex == 0 {
                // A new first fragment supersedes any stale incomplete message.
                clearReassembly(for: peripheralID)
                reassemblyBuffers[peripheralID] = [0: payload]
                expectedFragments[peripheralID] = totalFrags
                if totalFrags > 1 {
                    appendRepositoryDiagnostic("ble_central_rx_start total=\(totalFrags) from=\(peripheralID.uuidString.prefix(8))")
                }
            } else {
                guard expectedFragments[peripheralID] == totalFrags,
                      reassemblyBuffers[peripheralID] != nil else {
                    logger.warning("BLE fragment total changed or frame missing from \(peripheralID)")
                    clearReassembly(for: peripheralID)
                    return
                }
                var buffer = reassemblyBuffers[peripheralID] ?? [:]
                buffer[fragIndex] = payload
                let bufferedBytes = buffer.values.reduce(0) { $0 + $1.count }
                guard bufferedBytes <= maxReassemblyBytes else {
                    logger.warning("BLE reassembly exceeds byte limit from \(peripheralID)")
                    clearReassembly(for: peripheralID)
                    return
                }
                reassemblyBuffers[peripheralID] = buffer
            }

            let currentCount = reassemblyBuffers[peripheralID]?.count ?? 0
            if currentCount == totalFrags,
               reassemblyBuffers[peripheralID]?[0] != nil {
                var completeData = Data()
                let buffer = reassemblyBuffers[peripheralID] ?? [:]
                for i in 0..<totalFrags {
                    if let chunk = buffer[i] {
                        completeData.append(chunk)
                    } else {
                        logger.error("Missing fragment \(i) in complete buffer for \(peripheralID)")
                        return
                    }
                }
                clearReassembly(for: peripheralID)

                logger.info("Reassembled complete message (\(completeData.count) bytes) from \(peripheralID)")
                appendRepositoryDiagnostic(
                    "ble_central_rx_complete size=\(completeData.count) " +
                        "payload_sha256_64=\(payloadFingerprint(completeData))"
                )
                DispatchQueue.main.async { [weak self] in
                    self?.meshRepository?.onBleDataReceived(peerId: peripheralID.uuidString, data: completeData)
                }
            }
        }
    }

    private func clearReassembly(for peripheralId: UUID) {
        reassemblyBuffers.removeValue(forKey: peripheralId)
        expectedFragments.removeValue(forKey: peripheralId)
    }

    func peripheral(_ peripheral: CBPeripheral, didWriteValueFor characteristic: CBCharacteristic, error: Error?) {
        let peripheralId = peripheral.identifier
        if let error = error {
            logger.error("Write error for \(peripheralId): \(error.localizedDescription)")
            appendRepositoryDiagnostic("ble_central_write_fail id=\(peripheralId) err=\(error.localizedDescription)")
            // Clear current write state to allow retry/next
            writeInProgress[peripheralId] = false
            if let failedFragment = inFlightWrites.removeValue(forKey: peripheralId) {
                let attempt = (writeRetryCounts[peripheralId] ?? 0) + 1
                writeRetryCounts[peripheralId] = attempt
                if attempt <= maxWriteAttempts {
                    pendingWrites[peripheralId, default: []].insert(failedFragment, at: 0)
                    let delay = min(4.0, 0.25 * pow(2.0, Double(attempt - 1)))
                    DispatchQueue.main.asyncAfter(deadline: .now() + delay) { [weak self] in
                        self?.drainWriteQueue(for: peripheralId)
                    }
                } else {
                    writeRetryCounts.removeValue(forKey: peripheralId)
                    logger.error("Dropping BLE fragment after \(self.maxWriteAttempts) attempts for \(peripheralId)")
                }
            }
            if peripheral.state != .connected {
                attemptReconnection(to: peripheral)
            }
            return
        }

        // Dequeue next write
        appendRepositoryDiagnostic("ble_central_write_ok id=\(peripheralId.uuidString.prefix(8))")
        writeInProgress[peripheralId] = false
        inFlightWrites.removeValue(forKey: peripheralId)
        writeRetryCounts.removeValue(forKey: peripheralId)
        drainWriteQueue(for: peripheralId)
    }

    func peripheralIsReady(toSendWriteWithoutResponse peripheral: CBPeripheral) {
        drainWriteQueue(for: peripheral.identifier)
    }

    func validateConnection(to peripheralId: UUID) -> Bool {
        guard let peripheral = connectedPeripherals[peripheralId] else {
            logger.warning("BLE connection validation failed: peripheral not found for id=\(peripheralId)")
            return false
        }

        // Check if peripheral is still connected
        if peripheral.state != .connected {
            logger.warning("BLE connection validation failed: peripheral not in connected state (state=\\(peripheral.state.rawValue))")
            return false
        }

        // Check if we have the required characteristics
        guard let messageChar = messageCharacteristics[peripheralId],
              let syncChar = syncCharacteristics[peripheralId] else {
            logger.warning("BLE connection validation failed: missing required characteristics")
            return false
        }

        guard (messageChar.properties.contains(.write) ||
               messageChar.properties.contains(.writeWithoutResponse)),
              messageChar.properties.contains(.notify) ||
                messageChar.properties.contains(.indicate),
              syncChar.properties.contains(.read) ||
                syncChar.properties.contains(.write) else {
            logger.warning("BLE connection validation failed: characteristic capabilities incomplete")
            return false
        }

        logger.debug("BLE connection validation successful for \(peripheralId)")
        return true
    }
    
    // MARK: - Enhanced Error Handling
    
    private func handleBleError(_ error: Error?, operation: String, peripheralId: UUID? = nil) {
        var errorMessage: String = "BLE error in \(operation)"
        if let peripheralId = peripheralId {
            errorMessage += " for peripheral \(peripheralId)"
        }
        
        if let error = error {
            errorMessage += ": \(error.localizedDescription)"
            
            // Handle specific BLE error codes
            let nsError = error as NSError
            switch nsError.code {
            case CBError.connectionFailed.rawValue:
                errorMessage += " (Connection Failed)"
                if let peripheralId = peripheralId, let peripheral = discoveredPeripherals[peripheralId] {
                    attemptReconnection(to: peripheral)
                }
            
            case CBError.peripheralDisconnected.rawValue:
                errorMessage += " (Peripheral Disconnected)"
                // Disconnection is handled by didDisconnectPeripheral
                
            case CBError.connectionTimeout.rawValue:
                errorMessage += " (Connection Timeout)"
                if let peripheralId = peripheralId, let peripheral = discoveredPeripherals[peripheralId] {
                    attemptReconnection(to: peripheral)
                }
                
            case CBError.operationCancelled.rawValue:
                errorMessage += " (Operation Cancelled)"
                // This is expected during cleanup
                
            default:
                errorMessage += " (Code: \(nsError.code))"
                
                // For unknown errors, attempt reconnection if it's a connection-related operation
                if operation.contains("connect") || operation.contains("send") {
                    if let peripheralId = peripheralId, let peripheral = discoveredPeripherals[peripheralId] {
                        attemptReconnection(to: peripheral)
                    }
                }
            }
        } else {
            errorMessage += " (unknown error)"
        }
        
        logger.error("Error: {\(errorMessage)}")
        appendRepositoryDiagnostic("ble_error operation=\(operation) error=\(errorMessage)")
    }
    
    private func logBleWarning(_ message: String, operation: String, peripheralId: UUID? = nil) {
        var fullMessage: String = "BLE warning in \(operation): \(message)"
        if let peripheralId = peripheralId {
            fullMessage += " (peripheral: \(peripheralId))"
        }
        
        logger.warning("Warning: {\(fullMessage)}")
        appendRepositoryDiagnostic("ble_warning operation=\(operation) message=\(message)")
    }
}
