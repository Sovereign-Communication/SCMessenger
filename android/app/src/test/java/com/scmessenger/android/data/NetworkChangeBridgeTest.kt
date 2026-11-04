package com.scmessenger.android.data

import android.content.Context
import com.scmessenger.android.transport.NetworkDetector
import com.scmessenger.android.transport.NetworkType
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Issue #523: the repository wiring from NetworkDetector to the core bridge must
 * apply events in order, survive bursts, and deliver each real change once.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class NetworkChangeBridgeTest {

    private val applied = mutableListOf<Pair<Boolean, Boolean>>()
    private var settled = 0

    private fun TestScope.newBridge(
        apply: suspend (Boolean, Boolean) -> Unit = { w, c -> applied.add(w to c) }
    ) = NetworkChangeBridge(this, apply) { settled++ }

    private fun detector() = NetworkDetector(mockk<Context>(relaxed = true))

    @Test
    fun burstIsAppliedInOrderAndFinalStateWins() = runTest(StandardTestDispatcher()) {
        val bridge = newBridge()
        val expected = mutableListOf<Pair<Boolean, Boolean>>()
        repeat(50) { i ->
            val wifi = i % 2 == 0
            bridge.submit(wifi, !wifi)
            expected.add(wifi to !wifi)
        }
        advanceUntilIdle()
        assertEquals(expected, applied)
        assertEquals(false to true, applied.last())
        assertEquals("one settle per drained burst", 1, settled)
        bridge.close()
    }

    @Test
    fun slowApplyDoesNotLetALaterEventOvertakeAnEarlierOne() = runTest(StandardTestDispatcher()) {
        val gate = CompletableDeferred<Unit>()
        var first = true
        val bridge = newBridge { w, c ->
            if (first) { first = false; gate.await() }
            applied.add(w to c)
        }
        bridge.submit(true, false)
        bridge.submit(false, true)
        advanceUntilIdle()
        assertTrue("second event must wait for the first", applied.isEmpty())
        gate.complete(Unit)
        advanceUntilIdle()
        assertEquals(listOf(true to false, false to true), applied)
        bridge.close()
    }

    @Test
    fun coreFailureDoesNotStopLaterEventsOrTheSettle() = runTest(StandardTestDispatcher()) {
        var calls = 0
        val bridge = newBridge { w, c ->
            calls++
            if (calls == 1) throw IllegalStateException("boom")
            applied.add(w to c)
        }
        bridge.submit(true, false)
        bridge.submit(false, true)
        advanceUntilIdle()
        assertEquals(listOf(false to true), applied)
        assertEquals(1, settled)
        bridge.close()
    }

    @Test
    fun realChangeReachesBridgeExactlyOnce() = runTest(StandardTestDispatcher()) {
        val d = detector()
        val bridge = newBridge()
        bridge.attach(d)
        d.updateNetworkType(NetworkType.WIFI)
        advanceUntilIdle()
        applied.clear()
        d.updateNetworkType(NetworkType.CELLULAR)
        d.updateNetworkType(NetworkType.CELLULAR) // redetection, no change
        advanceUntilIdle()
        assertEquals(listOf(false to true), applied)
        bridge.close()
    }

    @Test
    fun vpnOverWifiIsNotReportedAsLostNorDuplicated() = runTest(StandardTestDispatcher()) {
        val d = detector()
        val bridge = newBridge()
        bridge.attach(d)
        d.updateNetworkType(NetworkType.WIFI)
        d.updateNetworkType(NetworkType.VPN)
        advanceUntilIdle()
        assertEquals(listOf(true to false), applied)
        bridge.close()
    }

    @Test
    fun detectorToBridgeBurstEndsOnLastDetectedState() = runTest(StandardTestDispatcher()) {
        val d = detector()
        val bridge = newBridge()
        bridge.attach(d)
        d.updateNetworkType(NetworkType.WIFI)
        d.updateNetworkType(NetworkType.CELLULAR)
        d.updateNetworkType(NetworkType.WIFI)
        advanceUntilIdle()
        assertEquals(listOf(true to false, false to true, true to false), applied)
        bridge.close()
    }
}
