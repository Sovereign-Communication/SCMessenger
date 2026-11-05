package com.scmessenger.android.transport

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Regression cover for NetworkDetector logging (cell test 2026-10-10): the
 * "Network type updated" line must be emitted only when the type changes, not
 * on every ConnectivityManager callback.
 */
class NetworkTypeTransitionLoggingTest {

    @Test
    fun repeatedCallbacksForSameTypeDoNotLog() {
        for (type in NetworkType.values()) {
            assertFalse(shouldLogNetworkTypeTransition(type, type))
        }
    }

    @Test
    fun actualTransitionsLog() {
        assertTrue(shouldLogNetworkTypeTransition(NetworkType.WIFI, NetworkType.CELLULAR))
        assertTrue(shouldLogNetworkTypeTransition(NetworkType.CELLULAR, NetworkType.UNKNOWN))
        assertTrue(shouldLogNetworkTypeTransition(NetworkType.UNKNOWN, NetworkType.WIFI))
    }
}
