package com.scmessenger.android.data

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SwarmStartGateTest {
    @Test
    fun firstStartProceeds() {
        assertTrue(SwarmStartGate.shouldStart(bridgePresent = false, startedWithIdentity = null, identityNow = true))
    }

    @Test
    fun retriggersAfterStartAreNoOps() {
        // Permission grant, setNickname, network recovery, identity creation
        // all firing after a completed start must not start again.
        repeat(3) {
            assertFalse(SwarmStartGate.shouldStart(bridgePresent = true, startedWithIdentity = true, identityNow = true))
            assertFalse(SwarmStartGate.shouldStart(bridgePresent = true, startedWithIdentity = false, identityNow = false))
        }
    }

    @Test
    fun headlessToFullUpgradeStillProceeds() {
        assertTrue(SwarmStartGate.shouldStart(bridgePresent = true, startedWithIdentity = false, identityNow = true))
    }

    @Test
    fun startAfterStopOrFailureProceeds() {
        assertTrue(SwarmStartGate.shouldStart(bridgePresent = false, startedWithIdentity = true, identityNow = true))
    }
}
