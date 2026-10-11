package com.scmessenger.android.transport

import android.content.Context
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Issue #523: NetworkDetector must forward every real network type change to
 * the core bridge listener, and must stay silent on no-op redetections.
 */
class NetworkDetectorChangeListenerTest {

    private val detector = NetworkDetector(mockk<Context>(relaxed = true))

    private fun record(): MutableList<Pair<Boolean, Boolean>> {
        val calls = mutableListOf<Pair<Boolean, Boolean>>()
        detector.networkChangeListener =
            NetworkDetector.NetworkChangeListener { wifi, cell -> calls.add(wifi to cell) }
        return calls
    }

    @Test
    fun wifiToCellularInvokesBridgeOnce() {
        detector.updateNetworkType(NetworkType.WIFI)
        val calls = record()
        detector.updateNetworkType(NetworkType.CELLULAR)
        assertEquals(listOf(false to true), calls)
    }

    @Test
    fun availableAndLostInvokeBridge() {
        val calls = record()
        detector.updateNetworkType(NetworkType.WIFI)
        detector.updateNetworkType(NetworkType.UNKNOWN)
        assertEquals(listOf(true to false, false to false), calls)
    }

    @Test
    fun repeatedSameTypeDoesNotInvokeBridge() {
        detector.updateNetworkType(NetworkType.CELLULAR)
        val calls = record()
        detector.updateNetworkType(NetworkType.CELLULAR)
        detector.updateNetworkType(NetworkType.CELLULAR)
        assertTrue(calls.isEmpty())
    }

    @Test
    fun throwingListenerDoesNotBreakClassification() {
        detector.networkChangeListener =
            NetworkDetector.NetworkChangeListener { _, _ -> throw IllegalStateException("boom") }
        detector.updateNetworkType(NetworkType.WIFI)
        assertEquals(NetworkType.WIFI, detector.networkType.value)
    }
}
