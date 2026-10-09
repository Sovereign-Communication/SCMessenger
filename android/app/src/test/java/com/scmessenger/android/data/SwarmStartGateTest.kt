package com.scmessenger.android.data

import org.junit.Assert.assertEquals
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

    @Test
    fun failedUpgradeKeepsBridgeAndLeavesUpgradePending() {
        // Headless start recorded, then identity appears and the upgrade fails.
        var flag: Boolean? = SwarmStartGate.nextStartedWithIdentity(null, true, true, false)
        assertEquals(false, flag)
        assertTrue(SwarmStartGate.shouldStart(bridgePresent = true, startedWithIdentity = flag, identityNow = true))
        flag = SwarmStartGate.nextStartedWithIdentity(flag, false, true, true)
        assertEquals(false, flag)
        // Old bridge still present; the next trigger must retry.
        assertTrue(SwarmStartGate.shouldStart(bridgePresent = true, startedWithIdentity = flag, identityNow = true))
    }

    @Test
    fun laterSuccessSetsFlagAndStopsRetrying() {
        var flag: Boolean? = false
        flag = SwarmStartGate.nextStartedWithIdentity(flag, false, true, true)
        flag = SwarmStartGate.nextStartedWithIdentity(flag, true, true, true)
        assertEquals(true, flag)
        assertFalse(SwarmStartGate.shouldStart(bridgePresent = true, startedWithIdentity = flag, identityNow = true))
    }

    @Test
    fun unknownIdentityReadRecordsNoIdentityOnSuccess() {
        assertEquals(false, SwarmStartGate.nextStartedWithIdentity(null, true, false, true))
    }
}
