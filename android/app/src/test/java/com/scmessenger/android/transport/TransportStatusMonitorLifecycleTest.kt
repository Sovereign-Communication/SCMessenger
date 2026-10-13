package com.scmessenger.android.transport

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestCoroutineScheduler
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import timber.log.Timber

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
        var failProbeAll = false
        var failProbeCellular = false
        var failProbeBle = false
        var failUnregister = false
        var networkListener: (() -> Unit)? = null
        var btListener: (() -> Unit)? = null

        val liveNetwork: Int get() = networkRegistered - networkUnregistered
        val liveBt: Int get() = btRegistered - btUnregistered

        override fun probeAll() {
            probeAllCalls++
            if (failProbeAll) throw IllegalStateException("fake probeAll failure")
        }
        override fun probeBle() {
            probeBleCalls++
            if (failProbeBle) throw IllegalStateException("fake probeBle failure")
        }
        override fun probeCellular() {
            probeCellularCalls++
            if (failProbeCellular) throw IllegalStateException("fake probeCellular failure")
        }

        override fun registerNetworkChange(onChange: () -> Unit): () -> Unit {
            if (failNetworkRegistration) throw IllegalStateException("fake registration failure")
            networkRegistered++
            networkListener = onChange
            return {
                networkUnregistered++
                if (failUnregister) throw IllegalStateException("fake unregister failure")
            }
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
    private var swarmFails = false
    private var localFails = false

    private class CapturingTree : Timber.Tree() {
        val lines = mutableListOf<String>()
        override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
            lines.add(message)
        }
    }

    private fun monitor() = TransportStatusMonitor(
        hooks = hooks,
        localPeerCounts = {
            if (localFails) throw IllegalStateException("fake local failure")
            emptyMap()
        },
        swarmPeerCount = {
            peerCountCalls++
            if (swarmFails) throw IllegalStateException("fake swarm failure")
            0
        },
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

    @Test
    fun failedInitialProbeStillStartsEveryResource() {
        hooks.failProbeAll = true
        val m = monitor()
        m.start()
        assertEquals(1, hooks.liveNetwork)
        assertEquals(1, hooks.liveBt)
        tick(1_500)
        assertEquals("periodic job runs despite the failed probe", 1, peerCountCalls)
        m.stop()
        assertEquals(0, hooks.liveNetwork)
        assertEquals(0, hooks.liveBt)
    }

    @Test
    fun throwingCallbackProbeIsContainedAndReportedAsTransportError() {
        hooks.failProbeCellular = true
        hooks.failProbeBle = true
        val m = monitor()
        m.start()
        // An exception escaping an OS callback would crash the process.
        hooks.networkListener!!.invoke()
        hooks.btListener!!.invoke()
        assertEquals(1, hooks.probeCellularCalls)
        assertEquals(1, hooks.probeBleCalls)
        assertEquals("error", TransportStatus.lastState("cellular"))
        assertEquals("error", TransportStatus.lastState("ble"))
        m.stop()
    }

    @Test
    fun failingUnregisterDoesNotBlockOtherReleasesOrRestart() {
        hooks.failUnregister = true
        val m = monitor()
        m.start()
        m.stop()
        assertEquals(1, hooks.networkUnregistered)
        assertEquals("other resources are still released", 0, hooks.liveBt)
        val seen = peerCountCalls
        tick(5_000)
        assertEquals("periodic job stopped", seen, peerCountCalls)

        hooks.failUnregister = false
        m.start()
        assertEquals(2, hooks.networkRegistered)
        m.stop()
    }

    @Test
    fun failingSwarmOrLocalCountsAreLoggedAndTheJobKeepsRunning() {
        val tree = CapturingTree()
        Timber.plant(tree)
        try {
            swarmFails = true
            localFails = true
            val m = monitor()
            m.start()
            tick(2_500)
            assertEquals("job survives repeated failures", 2, peerCountCalls)
            assertTrue(tree.lines.any { it.contains("swarm peer count unavailable") })
            assertTrue(tree.lines.any { it.contains("local peer counts unavailable") })
            assertTrue(
                "counts are still reported with an unknown swarm total",
                tree.lines.any { it.startsWith("[TRANSPORT]") && it.contains("swarm_total_-1") },
            )
            m.stop()
        } finally {
            Timber.uproot(tree)
        }
    }
}
