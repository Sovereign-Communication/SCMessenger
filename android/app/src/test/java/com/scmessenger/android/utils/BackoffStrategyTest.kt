package com.scmessenger.android.utils

import org.junit.Assert.*
import org.junit.Test

class BackoffStrategyTest {

    @Test
    fun `nextDelay increases with attempts and stays within maxDelayMs`() {
        val strategy = BackoffStrategy(
            initialDelayMs = 100L,
            maxDelayMs = 1000L,
            multiplier = 2.0,
            jitterFactor = 0.0 // Zero jitter for deterministic test
        )

        val delay1 = strategy.nextDelay()
        val delay2 = strategy.nextDelay()
        val delay3 = strategy.nextDelay()
        val delay4 = strategy.nextDelay()
        val delay5 = strategy.nextDelay()

        assertEquals(100L, delay1)
        assertEquals(200L, delay2)
        assertEquals(400L, delay3)
        assertEquals(800L, delay4)
        assertEquals(1000L, delay5) // Capped at maxDelayMs
    }

    @Test
    fun `reset returns backoff to initial state`() {
        val strategy = BackoffStrategy(
            initialDelayMs = 100L,
            maxDelayMs = 1000L,
            multiplier = 2.0,
            jitterFactor = 0.0
        )

        strategy.nextDelay()
        strategy.nextDelay()
        strategy.nextDelay()

        strategy.reset()

        val delayAfterReset = strategy.nextDelay()
        assertEquals(100L, delayAfterReset)
    }
}
