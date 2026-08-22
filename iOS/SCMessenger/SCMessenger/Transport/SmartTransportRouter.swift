//
//  SmartTransportRouter.swift
//  SCMessenger
//
//  Smart transport selection with 500ms timeout fallback and transport health tracking.
//  Implements parallel transport racing for optimal message delivery latency.
//

import Foundation
import os

/// Result of a transport delivery attempt
struct TransportDeliveryResult {
    let transport: TransportType
    let success: Bool
    let latencyMs: UInt64
    let error: String?
    let timestamp: Date
}

/// Health metrics for a specific transport to a specific peer
struct TransportHealth: Codable {
    var lastSuccessAt: Date?
    var lastFailureAt: Date?
    var successCount: UInt64 = 0
    var failureCount: UInt64 = 0
    var averageLatencyMs: Double = 0
    var lastLatencyMs: UInt64 = 0
    
    /// Success rate (0.0 to 1.0)
    var successRate: Double {
        let total: UInt64 = successCount + failureCount
        guard total > 0 else { return 0.5 } // Unknown = neutral
        return Double(successCount) / Double(total)
    }
    
    /// Is this transport considered healthy?
    var isHealthy: Bool {
        // Healthy if success rate > 50% and not too many recent failures
        guard successRate > 0.5 else { return false }
        // If last failure was very recent (< 5s), be cautious
        if let lastFailure = lastFailureAt {
            return Date().timeIntervalSince(lastFailure) > 5.0
        }
        return true
    }
    
    /// Score for transport selection (higher = better)
    var score: Double {
        // Weight: 70% success rate, 30% latency (inverted, lower latency = higher score)
        let latencyScore: Double = averageLatencyMs > 0 ? min(1.0, 500.0 / averageLatencyMs) : 0.5
        return (successRate * 0.7) + (latencyScore * 0.3)
    }
}

/// Message deduplication entry
struct MessageDedupEntry {
    let messageId: String
    let firstReceivedAt: Date
    let firstTransport: TransportType
    var duplicateCount: UInt32 = 0
    var duplicateTimestamps: [Date] = []
    var duplicateTransports: [TransportType] = []
}

/// Smart transport router with health tracking and parallel racing
@MainActor
final class SmartTransportRouter {
    private let logger: Logger = Logger(subsystem: "com.scmessenger", category: "TransportRouter")
    
    // Transport health tracking per peer
    private var transportHealth: [String: [TransportType: TransportHealth]] = [:]
    
    // Message deduplication cache
    private var messageDedupCache: [String: MessageDedupEntry] = [:]
    private let dedupCacheTtl: TimeInterval = 300 // 5 minutes
    
    // Timeout for "preferred" transport head start before racing all in parallel
    private let preferredTransportTimeoutMs: UInt64 = 100
    
    // Last successful transport per peer (for "previously used/good path")
    private var lastSuccessfulTransport: [String: TransportType] = [:]
    
    // MARK: - Transport Health Management
    
    /// Record a successful delivery
    func recordSuccess(peerId: String, transport: TransportType, latencyMs: UInt64) {
        var health: TransportHealth = getHealth(peerId: peerId, transport: transport)
        health.lastSuccessAt = Date()
        health.successCount += 1
        health.lastLatencyMs = latencyMs
        
        // Update rolling average latency
        let totalDeliveries: UInt64 = health.successCount + health.failureCount
        if totalDeliveries > 1 {
            health.averageLatencyMs = ((health.averageLatencyMs * Double(totalDeliveries - 1)) + Double(latencyMs)) / Double(totalDeliveries)
        } else {
            health.averageLatencyMs = Double(latencyMs)
        }
        
        setHealth(peerId: peerId, transport: transport, health: health)
        lastSuccessfulTransport[peerId] = transport
        
        logger.info("Transport health updated: peer=\(peerId.prefix(8)) transport=\(transport.rawValue) success rate=\(String(format: "%.2f", health.successRate)) avg_latency=\(String(format: "%.0f", health.averageLatencyMs))ms")
    }
    
    /// Record a failed delivery
    func recordFailure(peerId: String, transport: TransportType, error: String?) {
        var health: TransportHealth = getHealth(peerId: peerId, transport: transport)
        health.lastFailureAt = Date()
        health.failureCount += 1
        
        setHealth(peerId: peerId, transport: transport, health: health)
        
        logger.warning("Transport failure: peer=\(peerId.prefix(8)) transport=\(transport.rawValue) error=\(error ?? "unknown") success rate=\(String(format: "%.2f", health.successRate))")
    }
    
    /// Get health for a specific peer and transport
    private func getHealth(peerId: String, transport: TransportType) -> TransportHealth {
        return transportHealth[peerId]?[transport] ?? TransportHealth()
    }
    
    /// Set health for a specific peer and transport
    private func setHealth(peerId: String, transport: TransportType, health: TransportHealth) {
        if transportHealth[peerId] == nil {
            transportHealth[peerId] = [:]
        }
        transportHealth[peerId]?[transport] = health
    }
    
    // MARK: - Transport Selection
    
    /// Get the preferred transport for a peer (previously successful or highest score)
    func getPreferredTransport(peerId: String) -> TransportType? {
        // First check if we have a last successful transport
        if let lastSuccess = lastSuccessfulTransport[peerId] {
            let health: TransportHealth = getHealth(peerId: peerId, transport: lastSuccess)
            if health.isHealthy {
                return lastSuccess
            }
        }
        
        // Otherwise, find the transport with the highest score + situational default bias
        let allTransports: [TransportType] = TransportType.allCases
        var bestTransport: TransportType?
        var bestScore: Double = -1
        
        for transport in allTransports {
            let health: TransportHealth = getHealth(peerId: peerId, transport: transport)
            let situationalDefault: Double = {
                switch transport {
                case .tcpMdns, .multipeer: return 0.25 // High throughput local LAN
                case .ble: return 0.20                 // Low power proximity
                case .internet: return 0.15            // Cloud / wide area
                }
            }()
            let effectiveScore = health.score + situationalDefault
            if effectiveScore > bestScore {
                bestScore = effectiveScore
                bestTransport = transport
            }
        }
        
        return bestTransport
    }
    
    /// Get all available transports sorted by score (best first)
    func getAvailableTransportsSorted(peerId: String) -> [TransportType] {
        return TransportType.allCases.sorted { transport1, transport2 in
            let health1: TransportHealth = getHealth(peerId: peerId, transport: transport1)
            let health2: TransportHealth = getHealth(peerId: peerId, transport: transport2)
            return health1.score > health2.score
        }
    }
    
    // MARK: - Message Deduplication
    
    /// Check if a message is a duplicate and record it
    /// Returns: (isDuplicate, timeVarianceMs, firstTransport)
    func checkAndRecordMessage(
        messageId: String,
        transport: TransportType
    ) -> (isDuplicate: Bool, timeVarianceMs: UInt64?, firstTransport: TransportType?) {
        let now: Date = Date()
        
        // Clean up old entries
        cleanupDedupCache()
        
        if let existing = messageDedupCache[messageId] {
            // This is a duplicate
            let timeVarianceMs: UInt64 = UInt64(now.timeIntervalSince(existing.firstReceivedAt) * 1000)
            
            var updated: MessageDedupEntry = existing
            updated.duplicateCount += 1
            updated.duplicateTimestamps.append(now)
            updated.duplicateTransports.append(transport)
            messageDedupCache[messageId] = updated
            
            logger.info("Message duplicate detected: msg=\(messageId.prefix(8)) transport=\(transport.rawValue) time_variance=\(timeVarianceMs)ms first_transport=\(existing.firstTransport.rawValue) duplicate_count=\(updated.duplicateCount)")
            
            return (isDuplicate: true, timeVarianceMs: timeVarianceMs, firstTransport: existing.firstTransport)
        } else {
            // First receipt of this message
            let entry: MessageDedupEntry = MessageDedupEntry(
                messageId: messageId,
                firstReceivedAt: now,
                firstTransport: transport,
                duplicateCount: 0,
                duplicateTimestamps: [],
                duplicateTransports: []
            )
            messageDedupCache[messageId] = entry
            
            logger.info("Message first receipt: msg=\(messageId.prefix(8)) transport=\(transport.rawValue)")
            
            return (isDuplicate: false, timeVarianceMs: nil, firstTransport: nil)
        }
    }
    
    /// Get dedup statistics for a message (for mesh enhancement logging)
    func getDedupStats(messageId: String) -> MessageDedupEntry? {
        return messageDedupCache[messageId]
    }
    
    /// Clean up old dedup cache entries
    private func cleanupDedupCache() {
        let cutoff: Date = Date().addingTimeInterval(-dedupCacheTtl)
        messageDedupCache = messageDedupCache.filter { _, entry in
            entry.firstReceivedAt > cutoff
        }
    }
    
    // MARK: - Smart Delivery
    
    /// Attempt delivery with smart transport selection
    /// - Tries preferred transport first
    /// - If no response within 500ms, races all available transports
    /// - Returns the first successful result
    func attemptDelivery(
        peerId: String,
        envelopeData: Data,
        multipeerPeerId: String?,
        blePeerId: String?,
        tcpMdnsPeerId: String?,
        routePeerCandidates: [String],
        addresses: [String],
        traceMessageId: String?,
        attemptContext: String?,
        tryMultipeer: @escaping (String) async -> Bool,
        tryBle: @escaping (String) async -> Bool,
        tryTcpMdns: @escaping (String) async -> Bool,
        tryCore: @escaping (String) async -> Bool
    ) async -> TransportDeliveryResult {
        let startTime: Date = Date()
        
        // Determine available transports
        var availableTransports: [(type: TransportType, target: String, attempt: () async -> Bool)] = []
        
        if let multipeerTarget = multipeerPeerId?.trimmingCharacters(in: .whitespacesAndNewlines), !multipeerTarget.isEmpty {
            availableTransports.append((.multipeer, multipeerTarget, { await tryMultipeer(multipeerTarget) }))
        }
        
        // BLE is always a viable proximity transport candidate
        let effectiveBleTarget = blePeerId?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        availableTransports.append((.ble, effectiveBleTarget, { await tryBle(effectiveBleTarget) }))
        
        if let tcpMdnsTarget = tcpMdnsPeerId?.trimmingCharacters(in: .whitespacesAndNewlines), !tcpMdnsTarget.isEmpty {
            availableTransports.append((.tcpMdns, tcpMdnsTarget, { await tryTcpMdns(tcpMdnsTarget) }))
        }
        
        // Internet/Core relay is always a viable network transport candidate
        let effectiveInternetTarget = routePeerCandidates.first?.trimmingCharacters(in: .whitespacesAndNewlines) ?? peerId
        if !effectiveInternetTarget.isEmpty {
            availableTransports.append((.internet, effectiveInternetTarget, { await tryCore(effectiveInternetTarget) }))
        }
        
        guard !availableTransports.isEmpty else {
            logger.warning("No available transports for peer \(peerId.prefix(8))")
            return TransportDeliveryResult(
                transport: .internet,
                success: false,
                latencyMs: 0,
                error: "no_available_transports",
                timestamp: Date()
            )
        }
        
        // Get preferred transport
        let preferredTransport: TransportType? = getPreferredTransport(peerId: peerId)
        
        // If we have a preferred transport, try it first with timeout
        if let preferred = preferredTransport,
           let preferredAttempt = availableTransports.first(where: { $0.type == preferred }) {
            logger.info("Trying preferred transport \(preferred.rawValue) for peer \(peerId.prefix(8))")
            
            // Race preferred transport against timeout
            let preferredResult: Bool? = await withTaskGroup(of: Bool?.self, returning: Bool?.self) { group in
                // Preferred transport attempt
                group.addTask {
                    await preferredAttempt.attempt()
                }
                
                // Timeout task
                group.addTask {
                    try? await Task.sleep(nanoseconds: self.preferredTransportTimeoutMs * 1_000_000)
                    return nil // Timeout signal
                }
                
                // Wait for first result
                guard let result = await group.next() else {
                    return nil
                }
                group.cancelAll()
                return result
            }
            
            if let success = preferredResult, success {
                let latencyMs: UInt64 = UInt64(Date().timeIntervalSince(startTime) * 1000)
                recordSuccess(peerId: peerId, transport: preferred, latencyMs: latencyMs)
                logger.info("[OK] Preferred transport \(preferred.rawValue) succeeded in \(latencyMs)ms")
                return TransportDeliveryResult(
                    transport: preferred,
                    success: true,
                    latencyMs: latencyMs,
                    error: nil,
                    timestamp: Date()
                )
            }
            
            // Preferred transport failed or timed out - race all transports
            logger.warning("Preferred transport \(preferred.rawValue) failed/timed out, racing all transports")
        }
        
        // Race all available transports in parallel
        logger.info("Racing \(availableTransports.count) transports for peer \(peerId.prefix(8))")
        
        let result: (transport: TransportType, success: Bool, latencyMs: UInt64)? = await withTaskGroup(
            of: (transport: TransportType, success: Bool, latencyMs: UInt64)?.self,
            returning: (transport: TransportType, success: Bool, latencyMs: UInt64)?.self
        ) { group in
            for transportAttempt in availableTransports {
                group.addTask {
                    let transportStart: Date = Date()
                    let success: Bool = await transportAttempt.attempt()
                    let latencyMs: UInt64 = UInt64(Date().timeIntervalSince(transportStart) * 1000)
                    return (transport: transportAttempt.type, success: success, latencyMs: latencyMs)
                }
            }
            
            // Return first successful result
            for await result in group {
                if let result = result, result.success {
                    group.cancelAll()
                    return result
                }
            }
            
            return nil
        }
        
        if let result = result {
            recordSuccess(peerId: peerId, transport: result.transport, latencyMs: result.latencyMs)
            logger.info("[OK] Transport \(result.transport.rawValue) succeeded in \(result.latencyMs)ms")
            return TransportDeliveryResult(
                transport: result.transport,
                success: true,
                latencyMs: result.latencyMs,
                error: nil,
                timestamp: Date()
            )
        } else {
            // All transports failed
            let latencyMs: UInt64 = UInt64(Date().timeIntervalSince(startTime) * 1000)
            for transportAttempt in availableTransports {
                recordFailure(peerId: peerId, transport: transportAttempt.type, error: "all_transports_failed")
            }
            logger.error("[FAIL] All transports failed for peer \(peerId.prefix(8))")
            return TransportDeliveryResult(
                transport: .internet,
                success: false,
                latencyMs: latencyMs,
                error: "all_transports_failed",
                timestamp: Date()
            )
        }
    }
    
    // MARK: - Diagnostics
    
    /// Get transport health summary for diagnostics
    func getHealthSummary() -> [String: [String: Any]] {
        var summary: [String: [String: Any]] = [:]
        
        for (peerId, transports) in transportHealth {
            var peerSummary: [String: Any] = [:]
            for (transport, health) in transports {
                peerSummary[transport.rawValue] = [
                    "success_rate": health.successRate,
                    "avg_latency_ms": health.averageLatencyMs,
                    "success_count": health.successCount,
                    "failure_count": health.failureCount,
                    "is_healthy": health.isHealthy,
                    "score": health.score
                ]
            }
            summary[peerId] = peerSummary
        }
        
        return summary
    }
    
    /// Reset health for a peer (e.g., after reconnection)
    func resetHealth(peerId: String) {
        transportHealth.removeValue(forKey: peerId)
        lastSuccessfulTransport.removeValue(forKey: peerId)
        logger.info("Transport health reset for peer \(peerId.prefix(8))")
    }
}
