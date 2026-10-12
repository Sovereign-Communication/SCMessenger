package com.scmessenger.android.transport

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestCoroutineScheduler
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

/**
 * Lifecycle of [TransportStatusMonitor] with a fake platform: every resource it
 * acquires (network callback, Bluetooth receiver, periodic counting job) is
 * released on stop, stop is idempotent, and a restart re-acquires exactly once.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class TransportStatusMonitorLifecycleTest {

    private class FakeHooks : TransportStatusHooks {
        var probeAllCalls = 0
        var probeBleCalls = 0
        var probeCellularCalls = 0
        var networkRegistered = 0
        var networkUnregistered = 0
        var btRegistered = 0
        var btUnregistered = 0
        var failNetworkRegistration = false
        var networkListener: (() -> Unit)? = null
        var btListener: (() -> Unit)? = null

        val liveNetwork: Int get() = networkRegistered - networkUnregistered
        val liveBt: Int get() = btRegistered - btUnregistered

        override fun probeAll() { probeAllCalls++ }
        override fun probeBle() { probeBleCalls++ }
        override fun probeCellular() { probeCellularCalls++ }

        override fun registerNetworkChange(onChange: () -> Unit): () -> Unit {
            if (failNetworkRegistration) throw IllegalStateException("fake registration failure")
            networkRegistered++
            networkListener = onChange
            return { networkUnregistered++ }
        }

        override fun registerBluetoothStateChange(onChange: () -> Unit): () -> Unit {
            btRegistered++
            btListener = onChange
            return { btUnregistered++ }
        }
    }

    private val scheduler = TestCoroutineScheduler()
    private val hooks = FakeHooks()
    private var peerCountCalls = 0

    private fun monitor() = TransportStatusMonitor(
        hooks = hooks,
        localPeerCounts = { emptyMap() },
        swarmPeerCount = { peerCountCalls++; 0 },
        peerCountIntervalMs = 1_000L,
        dispatcher = StandardTestDispatcher(scheduler),
    )

    @Before
    fun setUp() = TransportStatus.reset()

    @After
    fun tearDown() = TransportStatus.reset()

    private fun tick(ms: Long) {
        scheduler.advanceTimeBy(ms)
        scheduler.runCurrent()
    }

    @Test
    fun stopReleasesEverything() {
        val m = monitor()
        m.start()
        tick(2_500)
        assertEquals(1, hooks.liveNetwork)
        assertEquals(1, hooks.liveBt)
        assertEquals(2, peerCountCalls)

        m.stop()
        assertEquals(0, hooks.liveNetwork)
        assertEquals(0, hooks.liveBt)
        val seen = peerCountCalls
        tick(10_000)
        assertEquals("periodic job must not run after stop", seen, peerCountCalls)
    }

    @Test
    fun doubleStopIsSafe() {
        val m = monitor()
        m.start()
        m.stop()
        m.stop()
        assertEquals(1, hooks.networkUnregistered)
        assertEquals(1, hooks.btUnregistered)
    }

    @Test
    fun stopWithoutStartIsNoOp() {
        val m = monitor()
        m.stop()
        assertEquals(0, hooks.networkUnregistered)
        assertEquals(0, hooks.btUnregistered)
    }

    @Test
    fun doubleStartRegistersOnce() {
        val m = monitor()
        m.start()
        m.start()
        assertEquals(1, hooks.networkRegistered)
        assertEquals(1, hooks.btRegistered)
        assertEquals(1, hooks.probeAllCalls)
        tick(1_500)
        assertEquals("one periodic job, not two", 1, peerCountCalls)
    }

    @Test
    fun restartAfterStopReacquiresCleanly() {
        val m = monitor()
        m.start()
        m.stop()
        m.start()
        assertEquals(2, hooks.networkRegistered)
        assertEquals(2, hooks.btRegistered)
        assertEquals(1, hooks.liveNetwork)
        assertEquals(1, hooks.liveBt)
        assertEquals(2, hooks.probeAllCalls)
        tick(1_500)
        assertEquals("exactly one periodic job after restart", 1, peerCountCalls)

        m.stop()
        assertEquals(0, hooks.liveNetwork)
        assertEquals(0, hooks.liveBt)
    }

    @Test
    fun lateCallbackAfterStopIsIgnored() {
        val m = monitor()
        m.start()
        val net = hooks.networkListener!!
        val bt = hooks.btListener!!
        m.stop()
        net()
        bt()
        assertEquals(0, hooks.probeCellularCalls)
        assertEquals(0, hooks.probeBleCalls)
    }

    @Test
    fun failedRegistrationLeavesNothingToLeakAndRestartRetries() {
        hooks.failNetworkRegistration = true
        val m = monitor()
        m.start()
        assertEquals(0, hooks.liveNetwork)
        assertEquals(1, hooks.liveBt)
        m.stop()
        assertEquals(0, hooks.liveBt)

        hooks.failNetworkRegistration = false
        m.start()
        assertEquals(1, hooks.liveNetwork)
        m.stop()
        assertEquals(0, hooks.liveNetwork)
    }

    @Test
    fun stopForgetsRememberedStatesSoRestartLogsAgain() {
        TransportStatus.report("ble", "available", "adapter on")
        val m = monitor()
        m.start()
        m.stop()
        assertEquals(null, TransportStatus.lastState("ble"))
    }

    @Test
    fun unknownKindIsNotRemembered() {
        TransportStatus.report("not_a_kind", "available", "x")
        assertEquals(null, TransportStatus.lastState("not_a_kind"))
    }
}
