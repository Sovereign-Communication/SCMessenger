package com.scmessenger.android.transport.ble

import org.junit.Assert.assertEquals
import org.junit.Test

class BleL2capManagerTest {

    @Test
    fun acceptRecoveryBacksOffWithoutAProcessLifetimeFailureCeiling() {
        val policy = BleL2capAcceptRecoveryPolicy()

        assertEquals(250L, policy.recordFailure())
        assertEquals(500L, policy.recordFailure())
        assertEquals(1_000L, policy.recordFailure())
        assertEquals(2_000L, policy.recordFailure())
        assertEquals(4_000L, policy.recordFailure())
        assertEquals(8_000L, policy.recordFailure())
        assertEquals(16_000L, policy.recordFailure())
        assertEquals(L2CAP_ACCEPT_MAX_BACKOFF_MS, policy.recordFailure())
        repeat(100) {
            assertEquals(L2CAP_ACCEPT_MAX_BACKOFF_MS, policy.recordFailure())
        }
        assertEquals(108, policy.failureCount)
    }

    @Test
    fun acceptRecoveryDelayIsCapped() {
        val policy = BleL2capAcceptRecoveryPolicy()

        repeat(8) { policy.recordFailure() }

        assertEquals(L2CAP_ACCEPT_MAX_BACKOFF_MS, policy.recordFailure())
        assertEquals(L2CAP_ACCEPT_MAX_BACKOFF_MS, policy.recordFailure())
    }

    @Test
    fun successfulAcceptResetsRecoveryBudget() {
        val policy = BleL2capAcceptRecoveryPolicy()

        policy.recordFailure()
        policy.recordFailure()
        policy.recordSuccess()

        assertEquals(0, policy.failureCount)
        assertEquals(L2CAP_ACCEPT_INITIAL_BACKOFF_MS, policy.recordFailure())
    }
}
