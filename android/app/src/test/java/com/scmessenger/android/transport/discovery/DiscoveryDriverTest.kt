package com.scmessenger.android.transport.discovery

import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.atomic.AtomicInteger

/**
 * #469 T7/T8: event -> scheduler wiring on the JVM. The Rust scheduler is
 * replaced by [FakePolicy], which records what the driver asks of it; the
 * decay/ceiling math itself is covered by the core tests.
 */
class DiscoveryDriverTest {

    private class FakePolicy : DiscoveryPolicy {
        val events = mutableListOf<DiscoveryEventKind>()
        var closeCalls = 0
        var inputs: Triple<Int, DiscoveryPowerLevel, Boolean>? = null
        val resetMap = mutableMapOf<DiscoveryEventKind, Set<DiscoveryLane>>()
        var delayMs: Long = 60_000L
        var pendingDue: Long? = null
        val appliedPending = AtomicInteger(0)
        val delayCalls = AtomicInteger(0)

        override fun onEvent(event: DiscoveryEventKind): Set<DiscoveryLane> {
            events.add(event)
            return resetMap[event] ?: emptySet()
        }

        override fun setInputs(connectedPeers: Int, power: DiscoveryPowerLevel, foreground: Boolean) {
            inputs = Triple(connectedPeers, power, foreground)
        }

        override fun nextDelayMs(lane: DiscoveryLane): Long {
            delayCalls.incrementAndGet()
            return delayMs
        }

        override fun pendingResetDueMs(lane: DiscoveryLane): Long? = pendingDue

        override fun applyPendingReset(lane: DiscoveryLane): Boolean {
            appliedPending.incrementAndGet()
            pendingDue = null
            return true
        }

        override fun close() {
            closeCalls++
        }
    }

    @Test
    fun `event is forwarded to the scheduler and listeners see the reset lanes`() {
        val policy = FakePolicy()
        policy.resetMap[DiscoveryEventKind.BLE_ON] = setOf(DiscoveryLane.BLE)
        val driver = DiscoveryDriver(policy)
        val seen = mutableListOf<Set<DiscoveryLane>>()
        driver.addListener { _, lanes -> seen.add(lanes) }

        val reset = driver.onEvent(DiscoveryEventKind.BLE_ON)

        assertEquals(listOf(DiscoveryEventKind.BLE_ON), policy.events)
        assertEquals(setOf(DiscoveryLane.BLE), reset)
        assertEquals(listOf(setOf(DiscoveryLane.BLE)), seen)
    }

    @Test
    fun `event that resets nothing does not notify listeners`() {
        val policy = FakePolicy()
        val driver = DiscoveryDriver(policy)
        var notified = false
        driver.addListener { _, _ -> notified = true }

        driver.onEvent(DiscoveryEventKind.BLE_OFF)

        assertFalse(notified)
    }

    @Test
    fun `a throwing listener does not break event delivery to others`() {
        val policy = FakePolicy()
        policy.resetMap[DiscoveryEventKind.WIFI_CHANGED] = setOf(DiscoveryLane.LAN)
        val driver = DiscoveryDriver(policy)
        var second = false
        driver.addListener { _, _ -> throw IllegalStateException("boom") }
        driver.addListener { _, _ -> second = true }

        driver.onEvent(DiscoveryEventKind.WIFI_CHANGED)

        assertTrue(second)
    }

    @Test
    fun `an event wakes a long wait immediately instead of sleeping the full delay`() = runBlocking {
        val policy = FakePolicy()
        policy.delayMs = 60_000L
        policy.resetMap[DiscoveryEventKind.BLE_ON] = setOf(DiscoveryLane.BLE)
        val driver = DiscoveryDriver(policy)

        val waiter = async(kotlinx.coroutines.Dispatchers.Default) {
            driver.awaitNextAttempt(DiscoveryLane.BLE)
        }
        delay(100)
        driver.onEvent(DiscoveryEventKind.BLE_ON)

        val woken = kotlinx.coroutines.withTimeout(5_000) { waiter.await() }
        assertTrue("event must wake the wait before the 60s delay elapses", woken)
    }

    @Test
    fun `an event that arrived before the wait still triggers an immediate attempt`() = runBlocking {
        val policy = FakePolicy()
        policy.delayMs = 60_000L
        policy.resetMap[DiscoveryEventKind.WIFI_CHANGED] = setOf(DiscoveryLane.LEDGER)
        val driver = DiscoveryDriver(policy)

        driver.onEvent(DiscoveryEventKind.WIFI_CHANGED)
        val woken = kotlinx.coroutines.withTimeout(5_000) {
            driver.awaitNextAttempt(DiscoveryLane.LEDGER)
        }

        assertTrue(woken)
    }

    @Test
    fun `an event for another lane does not wake this lane`() = runBlocking {
        val policy = FakePolicy()
        policy.delayMs = 300L
        policy.resetMap[DiscoveryEventKind.BLE_ON] = setOf(DiscoveryLane.BLE)
        val driver = DiscoveryDriver(policy)

        driver.onEvent(DiscoveryEventKind.BLE_ON)
        val woken = driver.awaitNextAttempt(DiscoveryLane.LAN)

        assertFalse("LAN timer elapsed on its own; BLE event must not wake it", woken)
    }

    @Test
    fun `wait always ends at the scheduler delay - there is no give-up path`() = runBlocking {
        val policy = FakePolicy()
        policy.delayMs = 50L
        val driver = DiscoveryDriver(policy)

        repeat(5) {
            assertFalse(driver.awaitNextAttempt(DiscoveryLane.LEDGER))
        }

        assertEquals(5, policy.delayCalls.get())
    }

    @Test
    fun `a flap-damped reset is applied when its window ends`() = runBlocking {
        val policy = FakePolicy()
        policy.delayMs = 60_000L
        policy.pendingDue = 80L
        val driver = DiscoveryDriver(policy)

        val attemptNow = kotlinx.coroutines.withTimeout(5_000) {
            driver.awaitNextAttempt(DiscoveryLane.BLE)
        }

        assertTrue("deferred reset must trigger an attempt", attemptNow)
        assertEquals(1, policy.appliedPending.get())
    }

    @Test
    fun `cadence hands out the scheduler delay for its lane`() {
        val policy = FakePolicy()
        policy.delayMs = 1234L
        val driver = DiscoveryDriver(policy)

        assertEquals(1234L, driver.cadenceFor(DiscoveryLane.BLE).nextDelayMs())
        assertEquals(1, policy.delayCalls.get())
    }

    @Test
    fun `inputs are forwarded to the scheduler`() {
        val policy = FakePolicy()
        val driver = DiscoveryDriver(policy)

        driver.setInputs(4, DiscoveryPowerLevel.LOW, false)

        assertEquals(Triple(4, DiscoveryPowerLevel.LOW, false), policy.inputs)
    }

    // ---- Lifecycle pairing (#469): every driver is closed exactly once ----

    @Test
    fun `close releases the policy once and is idempotent`() {
        val policy = FakePolicy()
        val driver = DiscoveryDriver(policy)

        driver.close()
        driver.close()

        assertEquals(1, policy.closeCalls)
    }

    @Test
    fun `a closed driver ignores events and inputs`() {
        val policy = FakePolicy()
        policy.resetMap[DiscoveryEventKind.WIFI_CHANGED] = setOf(DiscoveryLane.LAN)
        val driver = DiscoveryDriver(policy)
        var notified = false
        driver.addListener { _, _ -> notified = true }
        driver.close()

        val reset = driver.onEvent(DiscoveryEventKind.WIFI_CHANGED)
        driver.setInputs(1, DiscoveryPowerLevel.NORMAL, true)

        assertTrue(reset.isEmpty())
        assertFalse(notified)
        assertTrue(policy.events.isEmpty())
        assertEquals(null, policy.inputs)
    }

    @Test
    fun `a wait in flight ends with cancellation when the driver closes`() = runBlocking {
        val policy = FakePolicy()
        policy.delayMs = 60_000L
        val driver = DiscoveryDriver(policy)
        val waiter = async {
            try {
                driver.awaitNextAttempt(DiscoveryLane.LEDGER)
                "returned"
            } catch (e: kotlinx.coroutines.CancellationException) {
                "cancelled"
            }
        }
        delay(50)

        driver.close()

        assertEquals("cancelled", waiter.await())
    }

    @Test
    fun `awaiting on a closed driver cancels instead of spinning`() = runBlocking {
        val driver = DiscoveryDriver(FakePolicy())
        driver.close()

        val outcome = try {
            driver.awaitNextAttempt(DiscoveryLane.BLE)
            "returned"
        } catch (e: kotlinx.coroutines.CancellationException) {
            "cancelled"
        }

        assertEquals("cancelled", outcome)
    }

    @Test
    fun `a cadence handed out before close stays finite after close`() {
        val driver = DiscoveryDriver(FakePolicy())
        val cadence = driver.cadenceFor(DiscoveryLane.LAN)
        driver.close()

        assertTrue(cadence.nextDelayMs() >= 1L)
    }
}
