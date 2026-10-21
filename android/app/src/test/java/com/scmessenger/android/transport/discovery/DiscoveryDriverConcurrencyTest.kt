package com.scmessenger.android.transport.discovery

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger

/**
 * #469 T7/T8 concurrency: platform events arrive on binder, main and IO
 * threads and race with [DiscoveryDriver.close]. The native scheduler must
 * never be used after it is freed, never be freed while a call is inside it,
 * and close must be idempotent under concurrent callers. Every wait is
 * bounded, so a deadlock fails the test instead of hanging CI.
 */
class DiscoveryDriverConcurrencyTest {

    /** Policy that records any use-after-close or close-during-call. */
    private class GuardedPolicy : DiscoveryPolicy {
        val closed = AtomicBoolean(false)
        val inFlight = AtomicInteger(0)
        val violations = AtomicInteger(0)
        val closeCalls = AtomicInteger(0)
        val calls = AtomicInteger(0)

        private inline fun <T> guarded(block: () -> T): T {
            if (closed.get()) violations.incrementAndGet()
            inFlight.incrementAndGet()
            try {
                calls.incrementAndGet()
                Thread.yield() // widen the window a racing close() would hit
                return block()
            } finally {
                inFlight.decrementAndGet()
            }
        }

        override fun onEvent(event: DiscoveryEventKind): Set<DiscoveryLane> =
            guarded { setOf(DiscoveryLane.LEDGER) }

        override fun setInputs(connectedPeers: Int, power: DiscoveryPowerLevel, foreground: Boolean) =
            guarded { Unit }

        override fun nextDelayMs(lane: DiscoveryLane): Long = guarded { 60_000L }

        override fun pendingResetDueMs(lane: DiscoveryLane): Long? = guarded { null }

        override fun applyPendingReset(lane: DiscoveryLane): Boolean = guarded { false }

        override fun close() {
            closeCalls.incrementAndGet()
            if (inFlight.get() != 0) violations.incrementAndGet()
            closed.set(true)
        }
    }

    private val joinTimeoutSec = 20L

    private fun runThreads(count: Int, body: (Int) -> Unit): List<Throwable> {
        val failures = ConcurrentLinkedQueue<Throwable>()
        val start = CountDownLatch(1)
        val threads = (0 until count).map { i ->
            Thread {
                start.await()
                try {
                    body(i)
                } catch (t: Throwable) {
                    failures.add(t)
                }
            }.also { it.start() }
        }
        start.countDown()
        threads.forEach {
            it.join(TimeUnit.SECONDS.toMillis(joinTimeoutSec))
            assertTrue("a worker thread is still alive: deadlock", !it.isAlive)
        }
        return failures.toList()
    }

    @Test
    fun `events and input updates racing with close never touch the policy after it is freed`() {
        val policy = GuardedPolicy()
        val driver = DiscoveryDriver(policy)
        val events = DiscoveryEventKind.values()

        val failures = runThreads(10) { i ->
            if (i == 9) {
                Thread.sleep(2)
                driver.close()
            } else {
                for (n in 0 until 400) {
                    driver.onEvent(events[(i + n) % events.size])
                    driver.setInputs(n % 5, DiscoveryPowerLevel.NORMAL, n % 2 == 0)
                    driver.cadenceFor(DiscoveryLane.BLE).nextDelayMs()
                }
            }
        }

        assertEquals("worker threw: $failures", emptyList<Throwable>(), failures)
        assertEquals("policy used after close or closed mid-call", 0, policy.violations.get())
        assertEquals(1, policy.closeCalls.get())
        assertTrue("events ran before close", policy.calls.get() > 0)
    }

    @Test
    fun `concurrent close calls free the policy exactly once`() {
        val policy = GuardedPolicy()
        val driver = DiscoveryDriver(policy)

        val failures = runThreads(8) { driver.close() }

        assertEquals(emptyList<Throwable>(), failures)
        assertEquals(1, policy.closeCalls.get())
        assertEquals(0, policy.violations.get())
    }

    @Test
    fun `events after close are ignored without error`() {
        val policy = GuardedPolicy()
        val driver = DiscoveryDriver(policy)
        driver.close()

        val reset = driver.onEvent(DiscoveryEventKind.WIFI_CHANGED)
        driver.setInputs(1, DiscoveryPowerLevel.LOW, false)

        assertTrue(reset.isEmpty())
        assertEquals(0, policy.calls.get())
        assertEquals(60_000L, driver.cadenceFor(DiscoveryLane.LAN).nextDelayMs())
    }

    @Test
    fun `a scan loop waiting on the driver is cancelled when close races with events`() {
        val policy = GuardedPolicy()
        val driver = DiscoveryDriver(policy)
        val outcome = java.util.concurrent.atomic.AtomicReference("pending")
        val waiter = Thread {
            runBlocking {
                try {
                    // A woken wait loops back like the real scan loop does, until close ends it.
                    while (true) driver.awaitNextAttempt(DiscoveryLane.LEDGER)
                } catch (e: CancellationException) {
                    outcome.set("cancelled")
                }
            }
        }
        waiter.start()
        Thread.sleep(100)

        val failures = runThreads(6) { i ->
            if (i == 5) {
                Thread.sleep(2)
                driver.close()
            } else {
                for (n in 0 until 200) driver.onEvent(DiscoveryEventKind.NEW_PEER_CONNECTED)
            }
        }

        waiter.join(TimeUnit.SECONDS.toMillis(joinTimeoutSec))
        assertTrue("waiter still blocked after close", !waiter.isAlive)
        assertEquals(emptyList<Throwable>(), failures)
        assertEquals("cancelled", outcome.get())
        assertEquals(0, policy.violations.get())
    }

    @Test
    fun `a listener may close the driver from inside its callback without deadlocking`() {
        val policy = GuardedPolicy()
        val driver = DiscoveryDriver(policy)
        val notified = AtomicInteger(0)
        driver.addListener { _, _ ->
            notified.incrementAndGet()
            driver.close()
        }

        val failures = runThreads(1) { driver.onEvent(DiscoveryEventKind.APP_FOREGROUND) }

        assertEquals(emptyList<Throwable>(), failures)
        assertEquals(1, notified.get())
        assertEquals(1, policy.closeCalls.get())
        assertEquals(0, policy.violations.get())
    }
}
