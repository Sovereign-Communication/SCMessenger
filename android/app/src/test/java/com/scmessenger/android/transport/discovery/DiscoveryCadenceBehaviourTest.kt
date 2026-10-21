package com.scmessenger.android.transport.discovery

import com.scmessenger.android.transport.ble.BleBackoffStrategy
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * #469 T7/T8 behaviour: platform events reset scan cadence to the floor,
 * and cadence widens only through the scheduler, never through a timer.
 *
 * [DecayingPolicy] models the scheduler contract the shell relies on (reset
 * to floor on an affecting event, double on each attempt up to a ceiling). The
 * real decay math is covered by the Rust scheduler tests; here the assertions
 * are about what the Android driver and backoff do with it.
 */
class DiscoveryCadenceBehaviourTest {

    private class DecayingPolicy : DiscoveryPolicy {
        val floor = 1_000L
        val ceiling = 16_000L
        private val interval = DiscoveryLane.values().associateWith { floor }.toMutableMap()
        val eventsSeen = mutableListOf<DiscoveryEventKind>()
        var lastInputs: Triple<Int, DiscoveryPowerLevel, Boolean>? = null

        /** Mirrors core's affects() table for the events these tests send. */
        private fun lanesFor(event: DiscoveryEventKind): Set<DiscoveryLane> = when (event) {
            DiscoveryEventKind.BLE_ON -> setOf(DiscoveryLane.BLE)
            DiscoveryEventKind.BLE_OFF -> emptySet()
            DiscoveryEventKind.WIFI_CHANGED, DiscoveryEventKind.LAN_INTERFACE_CHANGED ->
                setOf(DiscoveryLane.LAN, DiscoveryLane.WIFI_DIRECT, DiscoveryLane.LEDGER)
            DiscoveryEventKind.CELLULAR_CHANGED, DiscoveryEventKind.INVITE_REDEEMED ->
                setOf(DiscoveryLane.LEDGER)
            DiscoveryEventKind.APP_FOREGROUND,
            DiscoveryEventKind.NEW_PEER_CONNECTED,
            DiscoveryEventKind.ALL_PEERS_LOST -> DiscoveryLane.values().toSet()
        }

        override fun onEvent(event: DiscoveryEventKind): Set<DiscoveryLane> {
            eventsSeen.add(event)
            val lanes = lanesFor(event)
            lanes.forEach { interval[it] = floor }
            return lanes
        }

        override fun setInputs(connectedPeers: Int, power: DiscoveryPowerLevel, foreground: Boolean) {
            lastInputs = Triple(connectedPeers, power, foreground)
        }

        override fun nextDelayMs(lane: DiscoveryLane): Long {
            val now = interval.getValue(lane)
            interval[lane] = minOf(now * 2, ceiling)
            return now
        }

        override fun pendingResetDueMs(lane: DiscoveryLane): Long? = null
        override fun applyPendingReset(lane: DiscoveryLane): Boolean = false
        override fun close() {}
    }

    private fun drain(cadence: ScanCadence, n: Int): List<Long> = List(n) { cadence.nextDelayMs() }

    @Test
    fun `turning bluetooth on returns ble scan windows to the floor after they widened`() {
        val policy = DecayingPolicy()
        val driver = DiscoveryDriver(policy)
        val backoff = BleBackoffStrategy(cadence = driver.cadenceFor(DiscoveryLane.BLE))

        val widened = List(4) { backoff.nextDelay() }
        assertEquals(listOf(1_000L, 2_000L, 4_000L, 8_000L), widened)

        driver.onEvent(DiscoveryEventKind.BLE_ON)

        assertEquals(policy.floor, backoff.nextDelay())
        assertEquals(2_000L, backoff.nextDelay())
    }

    @Test
    fun `ble scan delay only widens when no event arrives - elapsed time alone never resets it`() {
        val policy = DecayingPolicy()
        val driver = DiscoveryDriver(policy)
        val delays = drain(driver.cadenceFor(DiscoveryLane.BLE), 12)

        assertEquals(delays.sorted(), delays)
        assertEquals(policy.ceiling, delays.last())
        assertTrue(policy.eventsSeen.isEmpty())
    }

    @Test
    fun `backoff reset does not override the scheduler delay when a cadence is supplied`() {
        val policy = DecayingPolicy()
        val driver = DiscoveryDriver(policy)
        val backoff = BleBackoffStrategy(cadence = driver.cadenceFor(DiscoveryLane.BLE))
        backoff.nextDelay()
        backoff.nextDelay()

        backoff.reset()

        assertEquals(4_000L, backoff.nextDelay())
    }

    @Test
    fun `a wifi change resets lan and ledger cadence but leaves a widened ble cadence alone`() {
        val policy = DecayingPolicy()
        val driver = DiscoveryDriver(policy)
        val ble = driver.cadenceFor(DiscoveryLane.BLE)
        val ledger = driver.cadenceFor(DiscoveryLane.LEDGER)
        drain(ble, 3)
        drain(ledger, 3)

        val reset = driver.onEvent(DiscoveryEventKind.WIFI_CHANGED)

        assertTrue(DiscoveryLane.LEDGER in reset)
        assertFalse(DiscoveryLane.BLE in reset)
        assertEquals(policy.floor, ledger.nextDelayMs())
        assertEquals(8_000L, ble.nextDelayMs())
    }

    @Test
    fun `a cellular change resets only the ledger lane`() {
        val driver = DiscoveryDriver(DecayingPolicy())

        val reset = driver.onEvent(DiscoveryEventKind.CELLULAR_CHANGED)

        assertEquals(setOf(DiscoveryLane.LEDGER), reset)
    }

    @Test
    fun `bluetooth turning off resets nothing and notifies nobody`() {
        val driver = DiscoveryDriver(DecayingPolicy())
        var notified = false
        driver.addListener { _, _ -> notified = true }

        val reset = driver.onEvent(DiscoveryEventKind.BLE_OFF)

        assertTrue(reset.isEmpty())
        assertFalse(notified)
    }

    @Test
    fun `new peer connected and all peers lost reach every lane and wake every waiting loop`() = runBlocking {
        for (event in listOf(DiscoveryEventKind.NEW_PEER_CONNECTED, DiscoveryEventKind.ALL_PEERS_LOST)) {
            val driver = DiscoveryDriver(DecayingPolicy())

            val reset = driver.onEvent(event)

            assertEquals("event $event", DiscoveryLane.values().toSet(), reset)
            for (lane in DiscoveryLane.values()) {
                // The event queued a wake, so the wait returns true at once
                // rather than sleeping for the scheduler delay.
                assertTrue("lane $lane after $event", driver.awaitNextAttempt(lane))
            }
        }
    }

    @Test
    fun `app returning to foreground resets every lane to the floor`() {
        val policy = DecayingPolicy()
        val driver = DiscoveryDriver(policy)
        DiscoveryLane.values().forEach { drain(driver.cadenceFor(it), 3) }

        driver.onEvent(DiscoveryEventKind.APP_FOREGROUND)

        DiscoveryLane.values().forEach {
            assertEquals("lane $it", policy.floor, driver.cadenceFor(it).nextDelayMs())
        }
    }

    @Test
    fun `connected peer count power and foreground reach the scheduler without resetting cadence`() {
        val policy = DecayingPolicy()
        val driver = DiscoveryDriver(policy)
        val cadence = driver.cadenceFor(DiscoveryLane.LAN)
        drain(cadence, 2)

        driver.setInputs(7, DiscoveryPowerLevel.LOW, false)

        assertEquals(Triple(7, DiscoveryPowerLevel.LOW, false), policy.lastInputs)
        assertTrue(policy.eventsSeen.isEmpty())
        assertEquals(4_000L, cadence.nextDelayMs())
    }

    @Test
    fun `a peer not seen before is a new peer event and a repeat session is not`() {
        assertEquals(DiscoveryEventKind.NEW_PEER_CONNECTED, PeerPresence.onSessionStarted(true))
        assertNull(PeerPresence.onSessionStarted(false))
    }

    @Test
    fun `only losing the last peer reports all peers lost`() {
        assertNull(PeerPresence.onSessionEnded(2))
        assertNull(PeerPresence.onSessionEnded(1))
        assertEquals(DiscoveryEventKind.ALL_PEERS_LOST, PeerPresence.onSessionEnded(0))
    }
}
