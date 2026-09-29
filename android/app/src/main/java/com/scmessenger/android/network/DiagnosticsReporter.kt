package com.scmessenger.android.network

import android.content.Context
import com.scmessenger.android.transport.FallbackTransport
import com.scmessenger.android.transport.NetworkDetector
import com.scmessenger.android.transport.NetworkType
import com.scmessenger.android.utils.CircuitBreaker
import com.scmessenger.android.utils.NetworkFailureMetrics
import timber.log.Timber
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

/**
 * P0_ANDROID_007 / P0_NETWORK_001 Phase 7: Aggregates network diagnostics into a
 * user-facing report including transport priority, circuit breaker states, and port
 * probe results from the bootstrap fallback system.
 */
@Singleton
open class DiagnosticsReporter @Inject constructor(
    @ApplicationContext private val context: Context,
    private val networkDiagnostics: NetworkDiagnostics,
    private val networkTypeDetector: NetworkTypeDetector,
    private val failureMetrics: NetworkFailureMetrics,
    private val networkDetector: NetworkDetector,
    private val circuitBreaker: CircuitBreaker
) {
    data class CircuitBreakerEntry(
        val address: String,
        val state: CircuitBreaker.CircuitState,
        val failureCount: Int,
        val lastFailureReason: String?
    )

    data class NetworkDiagnosticsReport(
        val timestampMs: Long = System.currentTimeMillis(),
        val networkType: NetworkType,
        val hasInternet: Boolean,
        val dnsResults: Map<String, Boolean>,
        val portResults: Map<Int, Boolean>,
        val relayResults: Map<String, Boolean>,
        val failureSummary: NetworkFailureMetrics.Summary,
        val recommendations: List<String>,
        val transportPriority: List<FallbackTransport>,
        val circuitBreakerEntries: List<CircuitBreakerEntry>,
        val portProbeResults: Map<String, Boolean>
    )

    open suspend fun generateReport(): NetworkDiagnosticsReport {
        val testResults = networkDiagnostics.testNetworkConnectivity()
        val networkType = networkTypeDetector.detectNetworkType()
        val failureSummary = failureMetrics.getSummary()

        // P0_NETWORK_001 Phase 7: Include transport priority and circuit breaker state
        val transportPriority = networkDetector.getTransportPriority()
        val cbStats = circuitBreaker.getStats()
        val circuitBreakerEntries = buildCircuitBreakerEntries()
        val portProbeResults = runPortProbes()

        val recommendations = generateRecommendations(
            testResults, failureSummary, networkType, transportPriority, cbStats
        )

        val report = NetworkDiagnosticsReport(
            networkType = networkType,
            hasInternet = testResults.internetConnectivity,
            dnsResults = testResults.dnsResolution,
            portResults = testResults.portReachability,
            relayResults = testResults.relayConnectivity,
            failureSummary = failureSummary,
            recommendations = recommendations,
            transportPriority = transportPriority,
            circuitBreakerEntries = circuitBreakerEntries,
            portProbeResults = portProbeResults
        )

        Timber.i("Diagnostics report generated: internet=%b dns_fail=%d port_block=%d relay_fail=%d " +
                "transports=%s circuits=%d/%d recs=%d",
            report.hasInternet,
            failureSummary.totalDnsFailures,
            failureSummary.totalPortBlockedFailures,
            report.relayResults.count { !it.value },
            transportPriority.joinToString("->") { it.scheme },
            cbStats.openCount + cbStats.halfOpenCount,
            cbStats.total,
            recommendations.size
        )

        return report
    }

    private fun buildCircuitBreakerEntries(): List<CircuitBreakerEntry> {
        val openCircuits = circuitBreaker.getOpenCircuits()
        val healthyRelays = circuitBreaker.getHealthyRelays()
        val entries = mutableListOf<CircuitBreakerEntry>()

        for (addr in openCircuits) {
            val lastFailure = circuitBreaker.getLastFailure(addr)
            entries.add(CircuitBreakerEntry(
                address = addr,
                state = CircuitBreaker.CircuitState.OPEN,
                failureCount = lastFailure?.failureCount ?: 0,
                lastFailureReason = lastFailure?.lastFailureReason
            ))
        }

        for (addr in healthyRelays) {
            val lastFailure = circuitBreaker.getLastFailure(addr)
            entries.add(CircuitBreakerEntry(
                address = addr,
                state = CircuitBreaker.CircuitState.CLOSED,
                failureCount = 0,
                lastFailureReason = null
            ))
        }

        return entries
    }

    private suspend fun runPortProbes(): Map<String, Boolean> {
        val probes = emptyList<Pair<String, Int>>()
        return try {
            networkDetector.probePorts(probes)
        } catch (e: Exception) {
            Timber.w(e, "Port probe failed during diagnostics")
            emptyMap()
        }
    }

    private fun generateRecommendations(
        testResults: NetworkDiagnostics.NetworkTestResults,
        failureSummary: NetworkFailureMetrics.Summary,
        networkType: NetworkType,
        transportPriority: List<FallbackTransport>,
        cbStats: CircuitBreaker.CircuitBreakerStats
    ): List<String> {
        val recs = mutableListOf<String>()

        if (!testResults.internetConnectivity) {
            recs.add("No internet connectivity -- check Wi-Fi or cellular data")
        }

        when (networkType) {
            NetworkType.CELLULAR_RESTRICTED ->
                recs.add("Cellular network is restricting non-standard ports -- WebSocket on port 443 should work")
            NetworkType.CELLULAR_NO_INTERNET ->
                recs.add("Cellular network has no internet -- enable data or switch to Wi-Fi")
            NetworkType.WIFI_RESTRICTED ->
                recs.add("Wi-Fi connected but no internet validation -- check captive portal or login page")
            else -> { /* no specific network-type recommendation */ }
        }

        if (failureSummary.totalDnsFailures > 0) {
            recs.add("DNS resolution failing -- try alternative DNS (8.8.8.8 or 1.1.1.1)")
        }

        val blockedPorts = testResults.portReachability.filterValues { !it }.keys
        if (blockedPorts.isNotEmpty() && networkType in listOf(NetworkType.CELLULAR, NetworkType.CELLULAR_RESTRICTED)) {
            recs.add("Ports ${blockedPorts.joinToString(",")} blocked on cellular -- enable WebSocket fallback")
        }

        if (failureSummary.totalTlsFailures > 0) {
            recs.add("TLS handshake failures detected -- check device date/time and certificate stores")
        }

        if (failureSummary.totalConnectionRefusedFailures > 0) {
            recs.add("Connection refused -- relay servers may be down, try again later")
        }

        if (testResults.relayConnectivity.values.all { !it }) {
            recs.add("All relay servers unreachable -- check firewall or try a different network")
        }

        // P0_NETWORK_001 Phase 7: Circuit breaker recommendations
        if (cbStats.openCount > 0) {
            recs.add("${cbStats.openCount} relay(s) circuit-broken (open) -- will retry after cooldown. " +
                "Switching networks may help.")
        }
        if (cbStats.halfOpenCount > 0) {
            recs.add("${cbStats.halfOpenCount} relay(s) in half-open probe state -- recovery in progress")
        }

        // Transport priority recommendation
        val preferredTransport = transportPriority.firstOrNull()
        if (preferredTransport != null && networkType in listOf(
                NetworkType.CELLULAR, NetworkType.CELLULAR_RESTRICTED
            ) && preferredTransport != FallbackTransport.WEBSOCKET_WSS
        ) {
            recs.add("On cellular, expected WSS as preferred transport but got ${preferredTransport.scheme}")
        }

        return recs
    }

    fun formatReportForUser(report: NetworkDiagnosticsReport): String = buildString {
        appendLine("Network Status: ${report.networkType}")
        appendLine("Internet: ${if (report.hasInternet) "Connected" else "Disconnected"}")

        // P0_NETWORK_001 Phase 7: Transport priority
        if (report.transportPriority.isNotEmpty()) {
            appendLine("Transport Priority: ${report.transportPriority.joinToString(" -> ") { it.scheme }}")
        }

        // P0_NETWORK_001 Phase 7: Port probe results
        if (report.portProbeResults.isNotEmpty()) {
            val reachable = report.portProbeResults.filterValues { it }.keys
            val blocked = report.portProbeResults.filterValues { !it }.keys
            if (reachable.isNotEmpty()) {
                appendLine("Ports Reachable: ${reachable.joinToString(", ")}")
            }
            if (blocked.isNotEmpty()) {
                appendLine("Ports Blocked: ${blocked.joinToString(", ")}")
            }
        }

        // P0_NETWORK_001 Phase 7: Circuit breaker states
        if (report.circuitBreakerEntries.isNotEmpty()) {
            appendLine("Circuit Breakers:")
            report.circuitBreakerEntries.forEach { entry ->
                val stateLabel = when (entry.state) {
                    CircuitBreaker.CircuitState.CLOSED -> "[OK]"
                    CircuitBreaker.CircuitState.OPEN -> "[BLOCKED]"
                    CircuitBreaker.CircuitState.HALF_OPEN -> "[PROBING]"
                }
                val detail = entry.lastFailureReason?.let { " -- $it" } ?: ""
                appendLine("  $stateLabel ${entry.address}$detail")
            }
        }

        val failedDns = report.dnsResults.filterValues { !it }.keys
        if (failedDns.isNotEmpty()) {
            appendLine("DNS Failed: ${failedDns.joinToString(", ")}")
        }

        val failedRelays = report.relayResults.filterValues { !it }.keys
        if (failedRelays.isNotEmpty()) {
            appendLine("Relays Unreachable: ${failedRelays.joinToString(", ")}")
        }

        if (report.recommendations.isNotEmpty()) {
            appendLine()
            appendLine("Recommendations:")
            report.recommendations.forEach { appendLine("  - $it") }
        }
    }
}