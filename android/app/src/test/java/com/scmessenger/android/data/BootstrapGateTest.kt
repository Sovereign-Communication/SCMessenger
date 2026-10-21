package com.scmessenger.android.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.ConcurrentLinkedQueue
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger

/**
 * #469 T8: the bootstrap spacing/backoff gate that the outbox loop and the
 * discovery reset listener (any thread) both mutate. A ledger-lane reset must
 * win over a bootstrap round that began before it.
 */
class BootstrapGateTest {

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
            it.join(TimeUnit.SECONDS.toMillis(20))
            assertTrue("a worker thread is still alive: deadlock", !it.isAlive)
        }
        return failures.toList()
    }

    @Test
    fun `a round is spaced at least ten seconds from the previous one`() {
        val gate = BootstrapGate()
        assertNotEquals(BootstrapGate.DENIED, gate.tryBegin(100_000L))
        assertEquals(BootstrapGate.DENIED, gate.tryBegin(105_000L))
        assertNotEquals(BootstrapGate.DENIED, gate.tryBegin(110_000L))
    }

    @Test
    fun `failures arm the 10s 30s then capped backoff and a success clears it`() {
        val gate = BootstrapGate()
        var now = 1_000_000L
        val expected = listOf(10_000L, 30_000L, 30_000L)
        expected.forEachIndexed { i, backoff ->
            val token = gate.tryBegin(now)
            val b = gate.recordFailure(token, now)
            assertNotNull(b)
            assertEquals(i + 1, b!!.consecutiveFailures)
            assertEquals(backoff, b.backoffMs)
            now += backoff
        }
        val token = gate.tryBegin(now)
        assertEquals(16_000L, gate.recordFailure(token, now)!!.backoffMs)

        val later = gate.tryBegin(now + 70_000L)
        assertTrue(gate.recordSuccess(later))
        assertEquals(0, gate.consecutiveFailures())
        // Backoff cleared: a round is allowed as soon as spacing permits.
        assertNotEquals(BootstrapGate.DENIED, gate.tryBegin(now + 80_000L))
    }

    @Test
    fun `a round gated by backoff is denied until the deadline`() {
        val gate = BootstrapGate()
        val token = gate.tryBegin(1_000_000L)
        gate.recordFailure(token, 1_000_000L)
        assertEquals(BootstrapGate.DENIED, gate.tryBegin(1_005_000L))
        assertNotEquals(BootstrapGate.DENIED, gate.tryBegin(1_010_000L))
    }

    @Test
    fun `a reset clears spacing and failures so the next round starts at once`() {
        val gate = BootstrapGate()
        val token = gate.tryBegin(1_000_000L)
        gate.recordFailure(token, 1_000_000L)

        gate.reset()

        assertEquals(0, gate.consecutiveFailures())
        assertNotEquals(BootstrapGate.DENIED, gate.tryBegin(1_000_001L))
    }

    @Test
    fun `a failure from a round that began before a reset cannot re-arm the gate`() {
        val gate = BootstrapGate()
        val staleToken = gate.tryBegin(1_000_000L)

        gate.reset()

        assertNull(gate.recordFailure(staleToken, 1_000_100L))
        assertEquals(false, gate.holdUntil(staleToken, 9_999_999L))
        assertEquals(false, gate.recordSuccess(staleToken))
        assertEquals(0, gate.consecutiveFailures())
        // The stale hold(9_999_999) was ignored: the gate still admits a round.
        assertNotEquals(BootstrapGate.DENIED, gate.tryBegin(1_020_000L))
    }

    @Test
    fun `racing rounds at the same instant let exactly one begin`() {
        val gate = BootstrapGate()
        val winners = AtomicInteger(0)

        val failures = runThreads(12) {
            if (gate.tryBegin(5_000_000L) != BootstrapGate.DENIED) winners.incrementAndGet()
        }

        assertEquals(emptyList<Throwable>(), failures)
        assertEquals(1, winners.get())
    }

    @Test
    fun `concurrent failure records are all counted`() {
        val gate = BootstrapGate()
        val token = gate.tryBegin(1_000_000L)

        val failures = runThreads(8) {
            repeat(100) { gate.recordFailure(token, 1_000_000L) }
        }

        assertEquals(emptyList<Throwable>(), failures)
        assertEquals(800, gate.consecutiveFailures())
    }

    @Test
    fun `resets racing with stale failures always end with a clear gate`() {
        val gate = BootstrapGate()
        val staleToken = gate.tryBegin(1_000_000L)
        gate.reset()

        val failures = runThreads(8) { i ->
            repeat(200) {
                if (i == 7) gate.reset() else gate.recordFailure(staleToken, 2_000_000L)
            }
        }

        assertEquals(emptyList<Throwable>(), failures)
        assertEquals(0, gate.consecutiveFailures())
        // Backoff from stale failures (up to +60s) would deny this; none was armed.
        assertNotEquals(BootstrapGate.DENIED, gate.tryBegin(2_000_001L))
    }
}
