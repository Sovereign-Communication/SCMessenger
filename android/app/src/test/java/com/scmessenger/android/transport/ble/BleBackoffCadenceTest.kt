package com.scmessenger.android.transport.ble

import com.scmessenger.android.transport.discovery.ScanCadence
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** #469 B5/T8: the BLE backoff takes its delay from the scheduler, not a fixed cap. */
class BleBackoffCadenceTest {

    @Test
    fun `with a cadence the delay comes from the scheduler and ignores the fixed cap`() {
        var next = 90_000L
        val backoff = BleBackoffStrategy(maxDelayMs = 30_000, cadence = ScanCadence { next })

        assertEquals(90_000L, backoff.nextDelay())
        next = 1_000L
        assertEquals(1_000L, backoff.nextDelay())
        assertEquals(1_000L, backoff.getCurrentDelay())
    }

    @Test
    fun `scheduler delay is never below one millisecond`() {
        val backoff = BleBackoffStrategy(cadence = ScanCadence { 0L })
        assertEquals(1L, backoff.nextDelay())
    }

    @Test
    fun `without a cadence the legacy exponential backoff is unchanged`() {
        val backoff = BleBackoffStrategy(initialDelayMs = 1000, maxDelayMs = 30000)
        val first = backoff.nextDelay()
        assertTrue("first delay stays within the jitter envelope", first in 400L..1200L)
        repeat(10) { backoff.nextDelay() }
        assertTrue("legacy cap still bounds the delay", backoff.getCurrentDelay() <= 36_000L)
    }
}
