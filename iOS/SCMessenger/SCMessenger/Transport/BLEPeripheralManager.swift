//
//  BLEPeripheralManager.swift
//  SCMessenger
//
//  Advertises self and serves GATT characteristics
//  Mirrors: android/.../transport/ble/BleAdvertiser.kt + BleGattServer.kt
//

import CoreBluetooth
import CryptoKit
import Foundation
import os

/// Advertises self and serves GATT characteristics (iOS Peripheral role)
///
/// Responsibilities:
/// - BLE advertising with mesh service
/// - GATT server for Message/Sync/Identity characteristics (matches Android GATT)
/// - Handle central subscriptions
/// - Privacy rotation (15 min intervals)
final class BLEPeripheralManager: NSObject {
    private let logger = Logger(subsystem: "com.scmessenger", category: "BLE-Peripheral")
    private var peripheralManager: CBPeripheralManager!
    private weak var meshRepository: MeshRepository?

    // GATT Service and Characteristics (names match Android BleGattServer)
    private var meshService: CBMutableService?
    private var messageCharacteristic: CBMutableCharacteristic?  // Write: central → peripheral
    private var syncCharacteristic: CBMutableCharacteristic?     // Notify: peripheral → central
    private var identityCharacteristic: CBMutableCharacteristic? // Read: identity beacon

    // Subscribed centrals
    private var subscribedCentrals: [CBCentral] = []
    private var subscribedCharacteristicUuids: [UUID: CBUUID] = [:]

    // Privacy rotation
    private var rotationInterval: TimeInterval = MeshBLEConstants.privacyRotationInterval
    private var rotationTimer: Timer?
    private var identityData: Data?

    // Notification Buffer (Performance Optimization)
    private var pendingNotifications: [(central: CBCentral, data: Data)] = []

    // Advertising state
    private var isAdvertising = false
    private var isRotationEnabled = true

    // Reassembly buffers per central
    private var reassemblyBuffers: [UUID: [Int: Data]] = [:]
    private var expectedFragments: [UUID: Int] = [:]
    
    // Connection state monitoring and auto-reconnection
    private var connectionRetries: [UUID: Int] = [:]
    private var reconnectionTimers: [UUID: Timer] = [:]
    private let maxReconnectionAttempts = 3
    private let reconnectionDelay: TimeInterval = 2.0

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        peripheralManager = CBPeripheralManager(
            delegate: self,
            // Keep all mutable BLE state on main to avoid cross-queue races
            // with repository send paths (subscribedCentrals/pendingNotifications).
            queue: .main,
            options: [CBPeripheralManagerOptionRestoreIdentifierKey: MeshBLEConstants.peripheralRestoreId]
        )
    }

    // MARK: - Public API

    func startAdvertising() {
        logger.info("Starting BLE advertising")
        guard peripheralManager.state == .poweredOn else {
            logger.warning("Cannot start advertising: BLE not powered on")
            return
        }

        setupService()
        startPrivacyRotation()
    }

    func stopAdvertising() {
        logger.info("Stopping BLE advertising")
        peripheralManager.stopAdvertising()
        peripheralManager.removeAllServices()
        rotationTimer?.invalidate()
        rotationTimer = nil
        isAdvertising = false
    }

    func setIdentityData(_ data: Data) {
        // This payload is served over GATT identity characteristic reads, not
        // advertisement service data. Full JSON identity is expected here.
        identityData = data
        logger.debug("Identity data updated for dynamic read: \(data.count) bytes")
    }

    func setRotationInterval(_ interval: TimeInterval) {
        rotationInterval = interval
        logger.debug("Rotation interval set: \(interval)s")
        if isAdvertising && isRotationEnabled {
            rotationTimer?.invalidate()
            startPrivacyRotation()
        }
    }

    func setRotationEnabled(_ enabled: Bool) {
        isRotationEnabled = enabled
        logger.debug("Rotation enabled: \(enabled)")
        if isAdvertising {
            if enabled {
                startPrivacyRotation()
            } else {
                rotationTimer?.invalidate()
                rotationTimer = nil
            }
        }
    }

    func applyAdvertiseSettings(intervalMs: UInt32, txPowerDbm: Int8) {
        logger.debug("Advertise settings: interval=\(intervalMs)ms, txPower=\(txPowerDbm)dBm")
        // iOS doesn't allow direct control of advertising interval/power
        // Settings are advisory only
    }

    func broadcastDataToCentrals(_ data: Data) {
        if !Thread.isMainThread {
            DispatchQueue.main.async { [weak self] in
                self?.broadcastDataToCentrals(data)
            }
            return
        }
        for central in subscribedCentrals {
            _ = sendDataToCentral(central, data: data)
        }
    }

    /// Send data to a subscribed central identified by its UUID string.
    /// Used when we are acting as Peripheral and need to push data to a Central peer.
    @discardableResult
    func sendDataToConnectedCentral(peerId: String, data: Data) -> Bool {
        if !Thread.isMainThread {
            // Use async to avoid deadlock - result will be best-effort
            DispatchQueue.main.async { [weak self] in
                self?.sendDataToConnectedCentral(peerId: peerId, data: data)
            }
            return true // Optimistic return for async path
        }
        guard let uuid = UUID(uuidString: peerId) else {
            logger.warning("sendDataToConnectedCentral: invalid UUID string \(peerId)")
            return false
        }
        guard let central = subscribedCentrals.first(where: { $0.identifier == uuid }) else {
            logger.warning("sendDataToConnectedCentral: no subscribed central for \(peerId)")
            return false
        }
        return sendDataToCentral(central, data: data)
    }

    func subscribedCentralIds() -> [String] {
        if !Thread.isMainThread {
            // Return empty array for non-main-thread calls to avoid deadlock
            // Callers should invoke this from main thread for accurate results
            logger.warning("subscribedCentralIds called from background thread - returning empty")
            return []
        }
        return subscribedCentrals.map { $0.identifier.uuidString }
    }

    private func appendRepositoryDiagnostic(_ message: String) {
        let meshRepository = self.meshRepository
        Task { @MainActor in
            // Diagnostics are best-effort and intentionally dispatched asynchronously so
            // BLE delegate paths never synchronously touch MainActor-isolated repository state.
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
    
    private func attemptReconnection(to central: CBCentral) {
        let centralId = central.identifier
        
        // Cancel any existing reconnection timer
        reconnectionTimers[centralId]?.invalidate()
        reconnectionTimers.removeValue(forKey: centralId)
        
        // Increment retry count
        let retryCount = (connectionRetries[centralId] ?? 0) + 1
        connectionRetries[centralId] = retryCount
        
        if retryCount > maxReconnectionAttempts {
            logger.warning("Max reconnection attempts (\\(maxReconnectionAttempts)) reached for central \\(centralId), giving up")
            connectionRetries.removeValue(forKey: centralId)
            return
        }
        
        logger.info("Attempting reconnection \\(retryCount)/\\(maxReconnectionAttempts) to central \\(centralId)")
        appendRepositoryDiagnostic("ble_peripheral_reconnect_attempt attempt=\\$retryCount central=\\$centralId")
        
        // For peripheral role, we can't initiate connections directly
        // Instead, we ensure we're advertising so the central can reconnect
        ensureAdvertisingForReconnection()
        
        // Schedule next retry if this fails
        scheduleReconnectionRetry(for: central)
    }
    
    private func scheduleReconnectionRetry(for central: CBCentral) {
        let centralId = central.identifier
        
        // Cancel any existing timer first
        reconnectionTimers[centralId]?.invalidate()
        
        // Schedule retry with exponential backoff
        let retryDelay = reconnectionDelay * pow(2.0, Double(connectionRetries[centralId] ?? 1))
        
        let timer = Timer.scheduledTimer(withTimeInterval: retryDelay, repeats: false) { [weak self] _ in
            guard let self = self else { return }
            
            // Check if central is still not subscribed
            if !self.subscribedCentrals.contains(where: { $0.identifier == centralId }) {
                let currentRetryCount = self.connectionRetries[centralId] ?? 0
                if currentRetryCount <= self.maxReconnectionAttempts {
                    self.logger.info("Reconnection retry \\(currentRetryCount) for central \\(centralId)")
                    self.ensureAdvertisingForReconnection()
                    self.scheduleReconnectionRetry(for: central) // Schedule next retry if needed
                }
            } else {
                // Central reconnected successfully, clean up
                self.cleanupReconnectionState(for: centralId)
            }
        }
        
        reconnectionTimers[centralId] = timer
        logger.debug("Scheduled reconnection retry in \\(retryDelay)s for central \\(centralId)")
    }
    
    private func cleanupReconnectionState(for centralId: UUID) {
        reconnectionTimers[centralId]?.invalidate()
        reconnectionTimers.removeValue(forKey: centralId)
        connectionRetries.removeValue(forKey: centralId)
        logger.debug("Cleaned up reconnection state for central \\(centralId)")
    }
    
    private func ensureAdvertisingForReconnection() {
        if !isAdvertising {
            logger.info("Ensuring advertising is active for reconnection")
            // Restart advertising to ensure visibility
            peripheralManager.stopAdvertising()
            if let service = meshService {
                let advertisementData: [String: Any] = [
                    CBAdvertisementDataServiceUUIDsKey: [service.uuid],
                    CBAdvertisementDataLocalNameKey: "SCMessenger"
                ]
                peripheralManager.startAdvertising(advertisementData)
                isAdvertising = true
            }
        }
    }
    
    private func validateCentralConnection(_ central: CBCentral) -> Bool {
        // For peripheral role, we can't directly check connection state
        // Instead, we check if the central is still subscribed
        let isSubscribed = subscribedCentrals.contains(where: { $0.identifier == central.identifier })
        
        if !isSubscribed {
            logger.warning("Central \\(central.identifier) not subscribed")
            return false
        }
        return true
    }

    @discardableResult
    private func sendDataToCentral(_ central: CBCentral, data: Data) -> Bool {
        if !Thread.isMainThread {
            // Use async to avoid deadlock - result will be best-effort
            DispatchQueue.main.async { [weak self] in
                self?.sendDataToCentral(central, data: data)
            }
            return true // Optimistic return for async path
        }
        
        // Validate central connection state
        guard validateCentralConnection(central) else {
            attemptReconnection(to: central)
            return false
        }
        
        let notifyChar: CBMutableCharacteristic? = {
            if let uuid = subscribedCharacteristicUuids[central.identifier] {
                if uuid == MeshBLEConstants.messageCharUUID {
                    return messageCharacteristic
                } else if uuid == MeshBLEConstants.syncCharUUID {
                    return syncCharacteristic
                }
            }
            return messageCharacteristic ?? syncCharacteristic
        }()
        guard let notifyChar else {
            logger.warning("sendDataToCentral: notify characteristic unavailable")
            return false
        }
        guard subscribedCentrals.contains(where: { $0.identifier == central.identifier }) else {
            logger.warning("sendDataToCentral: central no longer subscribed \(central.identifier)")
            pendingNotifications.removeAll(where: { $0.central.identifier == central.identifier })
            return false
        }
        let mtu = central.maximumUpdateValueLength
        if mtu <= 4 {
            logger.warning("sendDataToCentral: invalid central MTU \(mtu) for \(central.identifier)")
            return false
        }
        let fragments = fragmentData(data, mtu: mtu)
        var accepted = false

        for fragment in fragments {
            if !subscribedCentrals.contains(where: { $0.identifier == central.identifier }) {
                logger.warning("sendDataToCentral: central unsubscribed mid-send \(central.identifier)")
                pendingNotifications.removeAll(where: { $0.central.identifier == central.identifier })
                return accepted
            }
            // IOS-CRASH-001: Guard against SIGTRAP when CBPeripheralManager is
            // not in .poweredOn state (e.g. transitioning after BT toggle).
            guard self.peripheralManager.state == .poweredOn else {
                self.logger.warning("sendDataToCentral: peripheralManager not poweredOn (\(self.peripheralManager.state.rawValue)), buffering fragment")
                self.pendingNotifications.append((central: central, data: fragment))
                accepted = true
                continue
            }
            let success = peripheralManager.updateValue(fragment, for: notifyChar, onSubscribedCentrals: [central])
            if !success {
                logger.warning("Failed to send fragment, buffering")
                appendRepositoryDiagnostic("ble_tx_buffer fragment to=\(central.identifier.uuidString.prefix(8))")
                pendingNotifications.append((central: central, data: fragment))
                // Buffered notifications are still tied to an active subscribed central.
                accepted = true
            } else {
                accepted = true
            }
        }
        if fragments.count > 1 {
            appendRepositoryDiagnostic(
                "ble_tx_start fragments=\(fragments.count) bytes=\(data.count) " +
                    "payload_sha256_64=\(payloadFingerprint(data)) to=\(central.identifier.uuidString.prefix(8))"
            )
        }
        return accepted
    }

    private func fragmentData(_ data: Data, mtu: Int) -> [Data] {
        let maxChunk = min(512, mtu)
        let maxPayload = maxChunk - 4
        if maxPayload <= 0 { return [data] }

        let totalFragments = Int(ceil(Double(data.count) / Double(maxPayload)))
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

    // MARK: - Private Methods

    private func setupService() {
        // P3: Guard against re-adding service when already set up
        guard meshService == nil else {
            // Service already configured — just ensure advertising is running
            if !isAdvertising {
                beginAdvertising()
            }
            return
        }

        // Create characteristics (names match Android BleGattServer)
        messageCharacteristic = CBMutableCharacteristic(
            type: MeshBLEConstants.messageCharUUID,
            properties: [.write, .writeWithoutResponse, .notify],
            value: nil,
            permissions: [.writeable, .readable]
        )

        syncCharacteristic = CBMutableCharacteristic(
            type: MeshBLEConstants.syncCharUUID,
            properties: [.read, .write, .notify],
            value: nil,
            permissions: [.readable, .writeable]
        )

        identityCharacteristic = CBMutableCharacteristic(
            type: MeshBLEConstants.identityCharUUID,
            properties: .read,
            value: nil,
            permissions: .readable
        )

        // Create service
        guard let messageCharacteristic,
              let syncCharacteristic,
              let identityCharacteristic else {
            logger.error("setupService: failed to initialize mesh characteristics")
            appendRepositoryDiagnostic("ble_peripheral_setup_fail reason=missing_characteristics")
            return
        }
        let service = CBMutableService(type: MeshBLEConstants.serviceUUID, primary: true)
        service.characteristics = [messageCharacteristic, syncCharacteristic, identityCharacteristic]
        meshService = service

        // Add service and start advertising
        peripheralManager.add(service)
    }

    private func beginAdvertising() {
        // P3: Guard against re-starting advertising when already active
        guard !isAdvertising else { return }

        let advertisementData: [String: Any] = [
            CBAdvertisementDataServiceUUIDsKey: [MeshBLEConstants.serviceUUID],
            CBAdvertisementDataLocalNameKey: MeshBLEConstants.advertisedName
        ]

        peripheralManager.startAdvertising(advertisementData)
        isAdvertising = true
        logger.info("BLE advertising started")
    }

    private func startPrivacyRotation() {
        guard isRotationEnabled else { return }
        rotationTimer?.invalidate()
        // Timer must be added to main RunLoop — background queues have no active RunLoop
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            self.rotationTimer = Timer.scheduledTimer(withTimeInterval: self.rotationInterval, repeats: true) { [weak self] _ in
                self?.rotateIdentity()
            }
            if let timer = self.rotationTimer {
                RunLoop.main.add(timer, forMode: .common)
            }
        }
    }

    private func rotateIdentity() {
        logger.info("Rotating identity...")
        stopAdvertising()
        isAdvertising = false // Ensure we clear state so beginAdvertising succeeds

        // Short delay to let hardware settle
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
            self?.beginAdvertising()
        }
    }
}

// MARK: - CBPeripheralManagerDelegate

extension BLEPeripheralManager: CBPeripheralManagerDelegate {
    func peripheralManagerDidUpdateState(_ peripheral: CBPeripheralManager) {
        logger.info("Peripheral manager state: \(peripheral.state.rawValue)")

        switch peripheral.state {
        case .poweredOn:
            if !isAdvertising {
                setupService()
            }
        case .poweredOff, .unauthorized, .unsupported:
            stopAdvertising()
        default:
            break
        }
    }

    func peripheralManager(_ peripheral: CBPeripheralManager, didAdd service: CBService, error: Error?) {
        if let error = error {
            logger.error("Failed to add service: \(error.localizedDescription)")
            appendRepositoryDiagnostic("ble_peripheral_add_service_fail err=\(error.localizedDescription)")
            return
        }
        logger.info("Service added successfully")
        appendRepositoryDiagnostic("ble_peripheral_service_added")
        beginAdvertising()
    }

    func peripheralManagerDidStartAdvertising(_ peripheral: CBPeripheralManager, error: Error?) {
        if let error = error {
            let nsError = error as NSError
            if nsError.domain == CBErrorDomain && nsError.code == 12 /* CBError.alreadyAdvertising */ {
                logger.warning("Advertising already active, syncing state")
                appendRepositoryDiagnostic("ble_peripheral_adv_already_active")
                isAdvertising = true
                return
            }
            logger.error("Failed to start advertising: \(error.localizedDescription)")
            appendRepositoryDiagnostic("ble_peripheral_adv_fail err=\(error.localizedDescription)")
            isAdvertising = false
            return
        }
        logger.info("Advertising started successfully")
        isAdvertising = true
        appendRepositoryDiagnostic("ble_peripheral_adv_start")
    }

    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveWrite requests: [CBATTRequest]) {
        for request in requests {
            guard let data = request.value, !data.isEmpty else {
                peripheral.respond(to: request, withResult: .success)
                continue
            }

            if request.characteristic.uuid == MeshBLEConstants.syncCharUUID {
                // Fragmented sync data
                handleFragmentedWrite(data: data, centralId: request.central.identifier, isSync: true)
                peripheral.respond(to: request, withResult: .success)
            } else if request.characteristic.uuid == MeshBLEConstants.messageCharUUID {
                // Fragmented message data
                handleFragmentedWrite(data: data, centralId: request.central.identifier, isSync: false)
                peripheral.respond(to: request, withResult: .success)
            } else {
                peripheral.respond(to: request, withResult: .requestNotSupported)
            }
        }
    }

    private func handleFragmentedWrite(data: Data, centralId: UUID, isSync: Bool) {
        if data.count < 4 {
            logger.warning("Received tiny BLE packet (<4 bytes) from \(centralId)")
            return
        }

        let totalFrags = Int(data[0]) | (Int(data[1]) << 8)
        let fragIndex = Int(data[2]) | (Int(data[3]) << 8)

        guard totalFrags > 0, totalFrags <= 2048, fragIndex >= 0, fragIndex < totalFrags else {
            logger.warning("Invalid BLE fragment parameters (index \(fragIndex) of \(totalFrags)) from \(centralId)")
            return
        }

        let payload = data.subdata(in: 4..<data.count)

        if fragIndex == 0 {
            // New message starting - clear any stale fragments from previous failed attempts
            reassemblyBuffers[centralId] = [0: payload]
            if totalFrags > 1 {
                appendRepositoryDiagnostic("ble_rx_start total=\(totalFrags) from=\(centralId.uuidString.prefix(8))")
            }
        } else {
            var buffer = reassemblyBuffers[centralId] ?? [:]
            buffer[fragIndex] = payload
            reassemblyBuffers[centralId] = buffer
        }

        let currentCount = reassemblyBuffers[centralId]?.count ?? 0
        if currentCount == totalFrags {
            var completeData = Data()
            let buffer = reassemblyBuffers[centralId] ?? [:]
            for i in 0..<totalFrags {
                if let chunk = buffer[i] {
                    completeData.append(chunk)
                } else {
                    logger.error("Missing fragment \(i) in complete buffer for \(centralId)")
                    return
                }
            }
            reassemblyBuffers.removeValue(forKey: centralId)

            logger.info("Reassembled complete \(isSync ? "sync" : "message") (\(completeData.count) bytes) from \(centralId)")
            appendRepositoryDiagnostic(
                "ble_rx_complete size=\(completeData.count) type=\(isSync ? "sync" : "msg") " +
                    "payload_sha256_64=\(payloadFingerprint(completeData))"
            )
            DispatchQueue.main.async { [weak self] in
                self?.meshRepository?.onBleDataReceived(peerId: centralId.uuidString, data: completeData)
            }
        }
    }

    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveRead request: CBATTRequest) {
        if request.characteristic.uuid == MeshBLEConstants.identityCharUUID {
            guard let data = identityData else {
                peripheral.respond(to: request, withResult: .unlikelyError)
                return
            }

            let offset = request.offset

            if offset > data.count {
                peripheral.respond(to: request, withResult: .invalidOffset)
                return
            }

            request.value = data.subdata(in: offset..<data.count)
            peripheral.respond(to: request, withResult: .success)
            logger.debug("Responded to read for identity beacon, offset: \(offset), returning \(request.value?.count ?? 0) bytes")
        } else {
            peripheral.respond(to: request, withResult: .requestNotSupported)
        }
    }

    func peripheralManager(_ peripheral: CBPeripheralManager, central: CBCentral, didSubscribeTo characteristic: CBCharacteristic) {
        logger.info("Central \(central.identifier.uuidString) subscribed to \(characteristic.uuid.shortUUID)")
        subscribedCharacteristicUuids[central.identifier] = characteristic.uuid
        if characteristic.uuid == MeshBLEConstants.messageCharUUID {
            logger.info("==> ANDROID CENTRAL \(central.identifier.uuidString) IS NOW SUBSCRIBED TO MESSAGE CHAR! This gives us the target to send data back over!")
        }
        appendRepositoryDiagnostic("ble_peripheral_subscribed central=\(central.identifier)")
        if !subscribedCentrals.contains(where: { $0.identifier == central.identifier }) {
            subscribedCentrals.append(central)
        }
    }

    func peripheralManager(_ peripheral: CBPeripheralManager, central: CBCentral, didUnsubscribeFrom characteristic: CBCharacteristic) {
        logger.info("Central \(central.identifier) unsubscribed from \(characteristic.uuid.shortUUID)")
        subscribedCharacteristicUuids.removeValue(forKey: central.identifier)
        subscribedCentrals.removeAll(where: { $0.identifier == central.identifier })
        pendingNotifications.removeAll(where: { $0.central.identifier == central.identifier })
        
        // Attempt reconnection for unintentional unsubscriptions
        // In a real implementation, you might want to distinguish between
        // intentional and unintentional unsubscriptions
        attemptReconnection(to: central)
    }

    func peripheralManager(_ peripheral: CBPeripheralManager, willRestoreState dict: [String: Any]) {
        // State restoration for background BLE
        if let services = dict[CBPeripheralManagerRestoredStateServicesKey] as? [CBMutableService] {
            logger.info("Restoring \(services.count) services")
            meshService = services.first
        }
    }

    func peripheralManagerIsReady(toUpdateSubscribers peripheral: CBPeripheralManager) {
        logger.debug("Peripheral manager ready to resume notifications")
        processPendingNotifications()
    }

    private func processPendingNotifications() {
        guard peripheralManager.state == .poweredOn else {
            logger.debug("Skipping pending notification flush: peripheralManager not poweredOn")
            return
        }
        while !self.pendingNotifications.isEmpty {
            let next = self.pendingNotifications[0]
            let notifyChar: CBMutableCharacteristic? = {
                if let uuid = subscribedCharacteristicUuids[next.central.identifier] {
                    if uuid == MeshBLEConstants.messageCharUUID {
                        return messageCharacteristic
                    } else if uuid == MeshBLEConstants.syncCharUUID {
                        return syncCharacteristic
                    }
                }
                return messageCharacteristic ?? syncCharacteristic
            }()
            guard self.subscribedCentrals.contains(where: { $0.identifier == next.central.identifier }),
                  let targetChar = notifyChar else {
                self.logger.debug("Dropping buffered notification for unsubscribed central \(next.central.identifier)")
                self.pendingNotifications.removeFirst()
                continue
            }
            let success = self.peripheralManager.updateValue(next.data, for: targetChar, onSubscribedCentrals: [next.central])
            if success {
                self.pendingNotifications.removeFirst()
                self.logger.debug("Processed buffered notification for \(next.central.identifier)")
            } else {
                // Queue still full, wait for next 'ready' callback
                self.logger.debug("Queue still full, remaining buffered: \(self.pendingNotifications.count)")
                break
            }
        }
    }
}
