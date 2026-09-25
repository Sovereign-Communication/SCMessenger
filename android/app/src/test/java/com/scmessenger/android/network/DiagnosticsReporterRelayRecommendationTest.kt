package com.scmessenger.android.network

import android.content.Context
import com.scmessenger.android.transport.NetworkDetector
import com.scmessenger.android.transport.NetworkType
import com.scmessenger.android.utils.CircuitBreaker
import com.scmessenger.android.utils.NetworkFailureMetrics
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Regression cover for the store-and-forward recommendation in
 * [DiagnosticsReporter.generateReport].
 *
 * Production state: `NetworkDiagnostics.testRelaySpecificConnectivity()` builds its
 * result from `val relayHosts = mapOf<String, Pair<String, Int>>()`, so
 * `relayConnectivity` is ALWAYS empty. The guard used to be
 * `testResults.relayConnectivity.values.all { !it }`, which is vacuously true for an
 * empty map, so the report told every operator "No node could be reached for
 * store-and-forward -- check firewall or try a different network" even though no node
 * had ever been probed.
 */
class DiagnosticsReporterRelayRecommendationTest {

    private val context = mockk<Context>(relaxed = true)
    private val networkDiagnostics = mockk<NetworkDiagnostics>()
    private val networkTypeDetector = mockk<NetworkTypeDetector>()
    private val failureMetrics = mockk<NetworkFailureMetrics>()
    private val networkDetector = mockk<NetworkDetector>()
    private val circuitBreaker = mockk<CircuitBreaker>()

    private val cleanSummary = NetworkFailureMetrics.Summary(
        totalNodes = 1,
        unreachableNodes = 0,
        totalDnsFailures = 0,
        totalTimeoutFailures = 0,
        totalTlsFailures = 0,
        totalPortBlockedFailures = 0,
        totalConnectionRefusedFailures = 0,
        nodeDetails = emptyMap()
    )

    private fun reporter(): DiagnosticsReporter = DiagnosticsReporter(
        context = context,
        networkDiagnostics = networkDiagnostics,
        networkTypeDetector = networkTypeDetector,
        failureMetrics = failureMetrics,
        networkDetector = networkDetector,
        circuitBreaker = circuitBreaker
    )

    private fun stubHappyPath(results: NetworkDiagnostics.NetworkTestResults) {
        coEvery { networkDiagnostics.testNetworkConnectivity() } returns results
        coEvery { networkTypeDetector.detectNetworkType() } returns NetworkType.WIFI
        every { failureMetrics.getSummary() } returns cleanSummary
        every { networkDetector.getTransportPriority() } returns emptyList()
        coEvery { networkDetector.probePorts(any(), any()) } returns emptyMap()
        every { circuitBreaker.getStats() } returns
            CircuitBreaker.CircuitBreakerStats(0, 0, 0, 0)
        every { circuitBreaker.getOpenCircuits() } returns emptyList()
        every { circuitBreaker.getHealthyRelays() } returns emptyList()
    }

    @Test
    fun `empty relayConnectivity does not claim no node could be reached`() = runTest {
        // Exactly what production produces: relayHosts is empty, so the map is empty.
        stubHappyPath(
            NetworkDiagnostics.NetworkTestResults(
                internetConnectivity = true,
                dnsResolution = mapOf("8.8.8.8" to true),
                portReachability = mapOf(443 to true),
                relayConnectivity = emptyMap(),
                networkType = NetworkType.WIFI
            )
        )

        val report = reporter().generateReport()

        assertFalse(
            "Empty relayConnectivity means no node was probed; the report must not " +
                "claim every node was unreachable. Got: ${report.recommendations}",
            report.recommendations.any { it.contains("No node could be reached") }
        )
    }

    @Test
    fun `a genuinely unreachable node still produces the recommendation`() = runTest {
        // The guard must keep working when probes actually ran and all of them failed.
        stubHappyPath(
            NetworkDiagnostics.NetworkTestResults(
                internetConnectivity = true,
                dnsResolution = mapOf("8.8.8.8" to true),
                portReachability = mapOf(443 to true),
                relayConnectivity = mapOf("node-a" to false, "node-b" to false),
                networkType = NetworkType.WIFI
            )
        )

        val report = reporter().generateReport()

        assertTrue(
            "When every probed node is unreachable the operator must still be warned. " +
                "Got: ${report.recommendations}",
            report.recommendations.any { it.contains("No node could be reached") }
        )
    }

    @Test
    fun `a reachable node suppresses the recommendation`() = runTest {
        stubHappyPath(
            NetworkDiagnostics.NetworkTestResults(
                internetConnectivity = true,
                dnsResolution = mapOf("8.8.8.8" to true),
                portReachability = mapOf(443 to true),
                relayConnectivity = mapOf("node-a" to true),
                networkType = NetworkType.WIFI
            )
        )

        val report = reporter().generateReport()

        assertFalse(
            "A reachable node must not produce an all-unreachable warning. " +
                "Got: ${report.recommendations}",
            report.recommendations.any { it.contains("No node could be reached") }
        )
    }
}
