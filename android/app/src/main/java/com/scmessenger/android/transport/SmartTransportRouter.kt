package com.scmessenger.android.transport

import kotlinx.coroutines.*
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong

/**
 * Smart transport selection with 500ms timeout fallback and transport health tracking.
 * Implements parallel transport racing for optimal message delivery latency.
 *
 * Mirrors: iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift
 */
class SmartTransportRouter {

    companion object {
        private const val TAG = "SmartTransportRouter"
        // A timeout here cancels the preferred attempt mid-dial and then
        // relaunches the same transport inside the race below. Too short a
        // value turns every LAN send into a cancel-and-retry cycle, which is
        // how a working Wi-Fi path stops flowing entirely. 500ms is the value
        // LAN delivery was last verified against; do not lower it without a
        // measurement showing the dial completes faster.
        private const val PREFERRED_TRANSPORT_TIMEOUT_MS = 500L
        private const val DEDUP_CACHE_TTL_MS = 300_000L // 5 minutes
    }

    /**
     * Transport types available for message delivery
     */
    enum class TransportType(val value: String) {
        WIFI_DIRECT("wifi_direct"),
        BLE("ble"),
        CORE("core"), // libp2p/internet relay
        TCP_MDNS("tcp_mdns"); // LAN-direct via libp2p TCP (mDNS-discovered peers)

        companion object {
            fun fromValue(value: String): TransportType? {
                return entries.find { it.value == value }
            }
        }
    }

    /**
     * Result of a transport delivery attempt
     */
    data class TransportDeliveryResult(
        val transport: TransportType,
        val success: Boolean,
        val latencyMs: Long,
        val error: String?,
        val timestamp: Long = System.currentTimeMillis()
    )

    /**
     * Health metrics for a specific transport to a specific peer
     */
    data class TransportHealth(
        var lastSuccessAt: Long? = null,
        var lastFailureAt: Long? = null,
        var successCount: Long = 0,
        var failureCount: Long = 0,
        var averageLatencyMs: Double = 0.0,
        var lastLatencyMs: Long = 0
    ) {
        /** Success rate (0.0 to 1.0) */
        val successRate: Double
            get() {
                val total = successCount + failureCount
                return if (total > 0) successCount.toDouble() / total.toDouble() else 0.5
            }

        /** Is this transport considered healthy? */
        val isHealthy: Boolean
            get() {
                if (successRate <= 0.5) return false
                // If last failure was very recent (< 5s), be cautious
                val lastFailure = lastFailureAt ?: return true
                return (System.currentTimeMillis() - lastFailure) > 5000
            }

        /** Score for transport selection (higher = better) */
        val score: Double
            get() {
                // Weight: 70% success rate, 30% latency (inverted, lower latency = higher score)
                val latencyScore = if (averageLatencyMs > 0) minOf(1.0, 500.0 / averageLatencyMs) else 0.5
                return (successRate * 0.7) + (latencyScore * 0.3)
            }
    }

    /**
     * Message deduplication entry
     */
    data class MessageDedupEntry(
        val messageId: String,
        val firstReceivedAt: Long,
        val firstTransport: TransportType,
        var duplicateCount: Int = 0,
        val duplicateTimestamps: MutableList<Long> = mutableListOf(),
        val duplicateTransports: MutableList<TransportType> = mutableListOf()
    )

    // Transport health tracking per peer
    private val transportHealth = ConcurrentHashMap<String, ConcurrentHashMap<TransportType, TransportHealth>>()

    // Message deduplication cache
    private val messageDedupCache = ConcurrentHashMap<String, MessageDedupEntry>()
    private val dedupMutex = Mutex()

    // Last successful transport per peer (for "previously used/good path")
    private val lastSuccessfulTransport = ConcurrentHashMap<String, TransportType>()

    // Mutex for health updates
    private val healthMutex = Mutex()

    // MARK: - Transport Health Management

    /**
     * Record a successful delivery
     */
    suspend fun recordSuccess(peerId: String, transport: TransportType, latencyMs: Long) {
        healthMutex.withLock {
            val health = getHealth(peerId, transport)
            health.lastSuccessAt = System.currentTimeMillis()
            health.successCount++
            health.lastLatencyMs = latencyMs

            // Update rolling average latency
            val totalDeliveries = health.successCount + health.failureCount
            health.averageLatencyMs = if (totalDeliveries > 1) {
                ((health.averageLatencyMs * (totalDeliveries - 1)) + latencyMs) / totalDeliveries
            } else {
                latencyMs.toDouble()
            }

            setHealth(peerId, transport, health)
            lastSuccessfulTransport[peerId] = transport

            Timber.tag(TAG).i("Transport health updated: peer=${peerId.take(8)} transport=${transport.value} success_rate=${String.format("%.2f", health.successRate)} avg_latency=${String.format("%.0f", health.averageLatencyMs)}ms")
        }
    }

    /**
     * Record a failed delivery
     */
    suspend fun recordFailure(peerId: String, transport: TransportType, error: String?) {
        healthMutex.withLock {
            val health = getHealth(peerId, transport)
            health.lastFailureAt = System.currentTimeMillis()
            health.failureCount++

            setHealth(peerId, transport, health)

            Timber.tag(TAG).w("Transport failure: peer=${peerId.take(8)} transport=${transport.value} error=${error ?: "unknown"} success_rate=${String.format("%.2f", health.successRate)}")
        }
    }

    /**
     * Get health for a specific peer and transport
     */
    private fun getHealth(peerId: String, transport: TransportType): TransportHealth {
        return transportHealth.getOrPut(peerId) { ConcurrentHashMap() }
            .getOrPut(transport) { TransportHealth() }
    }

    /**
     * Set health for a specific peer and transport
     */
    private fun setHealth(peerId: String, transport: TransportType, health: TransportHealth) {
        transportHealth.getOrPut(peerId) { ConcurrentHashMap() }[transport] = health
    }

    // MARK: - Transport Selection

    /**
     * Get the preferred transport for a peer (previously successful or highest score)
     */
    fun getPreferredTransport(peerId: String): TransportType? {
        // First check if we have a last successful transport
        val lastSuccess = lastSuccessfulTransport[peerId]
        if (lastSuccess != null) {
            val health = getHealth(peerId, lastSuccess)
            if (health.isHealthy) {
                return lastSuccess
            }
        }

        // Otherwise, find the transport with the highest score
        var bestTransport: TransportType? = null
        var bestScore = -1.0

        for (transport in TransportType.entries) {
            val health = getHealth(peerId, transport)
            if (health.score > bestScore) {
                bestScore = health.score
                bestTransport = transport
            }
        }

        return bestTransport
    }

    /**
     * Get all available transports sorted by score (best first)
     */
    fun getAvailableTransportsSorted(peerId: String): List<TransportType> {
        return TransportType.entries.sortedByDescending { transport ->
            getHealth(peerId, transport).score
        }
    }

    /**
     * Get the list of available transport types for a peer.
     * Returns all transport types that have recorded health data or
     * that are potentially available (no failures recorded).
     */
    fun getAvailableTransports(peerId: String): List<TransportType> {
        val peerTransports = transportHealth[peerId]
        return if (peerTransports != null && peerTransports.isNotEmpty()) {
            // Return transports that have health data (indicating they've been used)
            peerTransports.keys.toList()
        } else {
            // No health data yet — return all transport types as potentially available
            TransportType.entries
        }
    }

    // MARK: - Message Deduplication

    /**
     * Check if a message is a duplicate and record it
     * @return Triple(isDuplicate, timeVarianceMs, firstTransport)
     */
    suspend fun checkAndRecordMessage(
        messageId: String,
        transport: TransportType
    ): Triple<Boolean, Long?, TransportType?> {
        return dedupMutex.withLock {
            val now = System.currentTimeMillis()

            // Clean up old entries
            cleanupDedupCache()

            val existing = messageDedupCache[messageId]
            if (existing != null) {
                // This is a duplicate
                val timeVarianceMs = now - existing.firstReceivedAt

                existing.duplicateCount++
                existing.duplicateTimestamps.add(now)
                existing.duplicateTransports.add(transport)

                Timber.tag(TAG).i("Message duplicate detected: msg=${messageId.take(8)} transport=${transport.value} time_variance=${timeVarianceMs}ms first_transport=${existing.firstTransport.value} duplicate_count=${existing.duplicateCount}")

                Triple(true, timeVarianceMs, existing.firstTransport)
            } else {
                // First receipt of this message
                val entry = MessageDedupEntry(
                    messageId = messageId,
                    firstReceivedAt = now,
                    firstTransport = transport,
                    duplicateCount = 0,
                    duplicateTimestamps = mutableListOf(),
                    duplicateTransports = mutableListOf()
                )
                messageDedupCache[messageId] = entry

                Timber.tag(TAG).i("Message first receipt: msg=${messageId.take(8)} transport=${transport.value}")

                Triple(false, null, null)
            }
        }
    }

    /**
     * Get dedup statistics for a message (for mesh enhancement logging)
     */
    fun getDedupStats(messageId: String): MessageDedupEntry? {
        return messageDedupCache[messageId]
    }

    /**
     * Clean up old dedup cache entries
     */
    private fun cleanupDedupCache() {
        val cutoff = System.currentTimeMillis() - DEDUP_CACHE_TTL_MS
        messageDedupCache.entries.removeIf { (_, entry) ->
            entry.firstReceivedAt < cutoff
        }
    }

    // MARK: - Smart Delivery

    /**
     * Attempt delivery with smart transport selection
     * - Tries preferred transport first
     * - If no response within 500ms, races all available transports
     * - Returns the first successful result
     */
    suspend fun attemptDelivery(
        peerId: String,
        envelopeData: ByteArray,
        wifiPeerId: String?,
        blePeerId: String?,
        tcpMdnsPeerId: String?,
        routePeerCandidates: List<String>,
        listeners: List<String>,
        traceMessageId: String?,
        attemptContext: String?,
        tryWifi: suspend (String) -> Boolean,
        tryBle: suspend (String) -> Boolean,
        tryTcpMdns: suspend (String) -> Boolean,
        tryCore: suspend (String) -> Boolean
    ): TransportDeliveryResult {
        val logCtx = "[traceId=${traceMessageId?.take(8) ?: "none"}, ctx=${attemptContext ?: "none"}]"

        if (envelopeData.isEmpty()) {
            Timber.tag(TAG).w("$logCtx Rejected delivery attempt: empty envelope data")
            return TransportDeliveryResult(
                transport = TransportType.CORE,
                success = false,
                latencyMs = 0,
                error = "empty_envelope_data"
            )
        }

        val startTime = System.currentTimeMillis()

        // Determine available transports
        data class TransportAttempt(
            val type: TransportType,
            val target: String,
            val attempt: suspend () -> Boolean
        )

        val availableTransports = mutableListOf<TransportAttempt>()

        val wifiTarget = wifiPeerId?.trim()?.takeIf { it.isNotEmpty() }
        if (wifiTarget != null) {
            availableTransports.add(TransportAttempt(TransportType.WIFI_DIRECT, wifiTarget) { tryWifi(wifiTarget) })
        }

        val bleTarget = blePeerId?.trim().orEmpty()
        availableTransports.add(TransportAttempt(TransportType.BLE, bleTarget) { tryBle(bleTarget) })

        val tcpMdnsTarget = tcpMdnsPeerId?.trim()?.takeIf { it.isNotEmpty() }
        if (tcpMdnsTarget != null) {
            availableTransports.add(TransportAttempt(TransportType.TCP_MDNS, tcpMdnsTarget) { tryTcpMdns(tcpMdnsTarget) })
        }

        // Fallback: If no route candidates are found but direct listener addresses exist, attempt dialing peer via core
        var coreTarget = routePeerCandidates.firstOrNull()?.trim()?.takeIf { it.isNotEmpty() }
        if (coreTarget == null && listeners.isNotEmpty()) {
            coreTarget = peerId.trim().takeIf { it.isNotEmpty() }
            Timber.tag(TAG).i("$logCtx No route candidates, using listeners direct fallback for $coreTarget")
        }
        if (coreTarget != null) {
            availableTransports.add(TransportAttempt(TransportType.CORE, coreTarget) { tryCore(coreTarget) })
        }

        if (availableTransports.isEmpty()) {
            Timber.tag(TAG).w("$logCtx No available transports for peer ${peerId.take(8)}")
            return TransportDeliveryResult(
                transport = TransportType.CORE,
                success = false,
                latencyMs = 0,
                error = "no_available_transports"
            )
        }

        // Get preferred transport
        val preferredTransport = getPreferredTransport(peerId)

        // If we have a preferred transport, try it first with timeout
        if (preferredTransport != null) {
            val preferredAttempt = availableTransports.find { it.type == preferredTransport }
            if (preferredAttempt != null) {
                Timber.tag(TAG).i("$logCtx Trying preferred transport ${preferredTransport.value} for peer ${peerId.take(8)}")

                // Race preferred transport against timeout
                val preferredResult = withTimeoutOrNull(PREFERRED_TRANSPORT_TIMEOUT_MS) {
                    preferredAttempt.attempt()
                }

                if (preferredResult == true) {
                    val latencyMs = System.currentTimeMillis() - startTime
                    recordSuccess(peerId, preferredTransport, latencyMs)
                    Timber.tag(TAG).i("$logCtx [OK] Preferred transport ${preferredTransport.value} succeeded in ${latencyMs}ms")
                    return TransportDeliveryResult(
                        transport = preferredTransport,
                        success = true,
                        latencyMs = latencyMs,
                        error = null
                    )
                }

                // Preferred transport failed or timed out - race all transports
                Timber.tag(TAG).w("$logCtx Preferred transport ${preferredTransport.value} failed/timed out, racing all transports")
            }
        }

        // Race all available transports in parallel
        Timber.tag(TAG).i("$logCtx Racing ${availableTransports.count()} transports for peer ${peerId.take(8)}")

        val result = coroutineScope {
            // Only a successful attempt can complete the winner. A failed or
            // no-route candidate therefore cannot overwrite a success that
            // completed concurrently.
            val winner = CompletableDeferred<Triple<TransportType, Boolean, Long>?>()

            val jobs = availableTransports.map { transportAttempt ->
                launch {
                    val transportStart = System.currentTimeMillis()
                    val success = try {
                        transportAttempt.attempt()
                    } catch (e: Exception) {
                        Timber.tag(TAG).w(e, "$logCtx Transport ${transportAttempt.type.value} failed")
                        false
                    }
                    val latencyMs = System.currentTimeMillis() - transportStart
                    if (success) {
                        winner.complete(Triple(transportAttempt.type, true, latencyMs))
                    }
                }
            }

            // Complete with null only after every candidate has finished. A
            // successful candidate always completes winner first.
            val waitJob = launch {
                jobs.forEach { it.join() }
                winner.complete(null)
            }

            val resolved = winner.await()

            // Stop all candidates before joining the watcher; this also keeps
            // a slow/no-route candidate from delaying or replacing the winner.
            jobs.forEach { it.cancelAndJoin() }
            waitJob.cancelAndJoin()
            resolved
        }

        return if (result != null) {
            recordSuccess(peerId, result.first, result.third)
            Timber.tag(TAG).i("$logCtx [OK] Transport ${result.first.value} succeeded in ${result.third}ms")
            TransportDeliveryResult(
                transport = result.first,
                success = true,
                latencyMs = result.third,
                error = null
            )
        } else {
            // All transports failed
            val latencyMs = System.currentTimeMillis() - startTime
            availableTransports.forEach { transportAttempt ->
                recordFailure(peerId, transportAttempt.type, "all_transports_failed")
            }
            Timber.tag(TAG).e("$logCtx [FAIL] All transports failed for peer ${peerId.take(8)}")
            TransportDeliveryResult(
                transport = TransportType.CORE,
                success = false,
                latencyMs = latencyMs,
                error = "all_transports_failed"
            )
        }
    }

    // MARK: - Diagnostics

    /**
     * Get transport health summary for diagnostics
     */
    fun getHealthSummary(): Map<String, Map<String, Any>> {
        val summary = mutableMapOf<String, Map<String, Any>>()

        for ((peerId, transports) in transportHealth) {
            val peerSummary = mutableMapOf<String, Any>()
            for ((transport, health) in transports) {
                peerSummary[transport.value] = mapOf(
                    "success_rate" to health.successRate,
                    "avg_latency_ms" to health.averageLatencyMs,
                    "success_count" to health.successCount,
                    "failure_count" to health.failureCount,
                    "is_healthy" to health.isHealthy,
                    "score" to health.score
                )
            }
            summary[peerId] = peerSummary
        }

        return summary
    }

    /**
     * Reset health for a peer (e.g., after reconnection)
     */
    fun resetHealth(peerId: String) {
        transportHealth.remove(peerId)
        lastSuccessfulTransport.remove(peerId)
        Timber.tag(TAG).i("Transport health reset for peer ${peerId.take(8)}")
    }
}
