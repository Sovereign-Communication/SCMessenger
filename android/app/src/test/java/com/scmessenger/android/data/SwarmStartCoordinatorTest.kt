package com.scmessenger.android.data

import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Concurrency tests for [SwarmStartCoordinator]. Every test runs under a
 * deadlock timeout so a wedged mutex fails fast instead of hanging CI.
 */
class SwarmStartCoordinatorTest {
    private val deadlockMs = 10_000L

    private class Fixture(
        identity: SwarmIdentityRead = SwarmIdentityRead(known = true, present = true),
        val startBody: suspend (Fixture) -> Boolean = { true }
    ) {
        @Volatile var bridge = false
        @Volatile var identity = identity
        val starts = AtomicInteger(0)
        val inFlight = AtomicInteger(0)
        val maxInFlight = AtomicInteger(0)

        val coordinator = SwarmStartCoordinator(
            bridgePresent = { bridge },
            readIdentity = { this.identity },
            start = {
                val now = inFlight.incrementAndGet()
                maxInFlight.accumulateAndGet(now) { a, b -> maxOf(a, b) }
                try {
                    starts.incrementAndGet()
                    val ok = startBody(this)
                    if (ok) bridge = true
                    ok
                } finally {
                    inFlight.decrementAndGet()
                }
            }
        )
    }

    @Test
    fun manyConcurrentTriggersStartExactlyOnceAndNeverOverlap() = runBlocking {
        val f = Fixture(startBody = { delay(50); true })
        withTimeout(deadlockMs) {
            val ran = (1..200).map {
                async(Dispatchers.Default) { f.coordinator.trigger() }
            }.awaitAll()
            assertEquals(1, ran.count { it })
        }
        assertEquals(1, f.starts.get())
        assertEquals(1, f.maxInFlight.get())
        assertEquals(true, f.coordinator.startedWithIdentity)
        assertFalse(f.coordinator.isStarting)
    }

    @Test
    fun failedStartReleasesTheMutexAndNextTriggerRetries() = runBlocking {
        val failures = AtomicInteger(1)
        val f = Fixture(startBody = {
            if (failures.getAndDecrement() > 0) throw IllegalStateException("bind failed")
            true
        })
        withTimeout(deadlockMs) {
            try {
                f.coordinator.trigger()
                throw AssertionError("expected the start failure to propagate")
            } catch (expected: IllegalStateException) {
                // the lock must be free again
            }
            assertFalse(f.coordinator.isStarting)
            assertNull(f.coordinator.startedWithIdentity)
            assertTrue(f.coordinator.trigger())
        }
        assertEquals(2, f.starts.get())
        assertEquals(true, f.coordinator.startedWithIdentity)
    }

    @Test
    fun startReportingFailureLeavesFlagUnchangedAndStaysRetryable() = runBlocking {
        val results = ArrayDeque(listOf(false, false, true))
        val f = Fixture(startBody = { results.removeFirst() })
        withTimeout(deadlockMs) {
            assertTrue(f.coordinator.trigger())
            assertNull(f.coordinator.startedWithIdentity)
            assertTrue(f.coordinator.trigger())
            assertNull(f.coordinator.startedWithIdentity)
            assertTrue(f.coordinator.trigger())
            assertFalse(f.coordinator.trigger())
        }
        assertEquals(3, f.starts.get())
    }

    @Test
    fun cancelledQueuedWaiterLeavesQueueAndDoesNotWedgeLaterTriggers() = runBlocking {
        val release = CompletableDeferred<Unit>()
        val entered = CompletableDeferred<Unit>()
        val f = Fixture(startBody = { entered.complete(Unit); release.await(); true })
        withTimeout(deadlockMs) {
            val holder = async(Dispatchers.Default) { f.coordinator.trigger() }
            entered.await()
            val queued = launch(Dispatchers.Default, CoroutineStart.UNDISPATCHED) {
                f.coordinator.trigger()
            }
            queued.cancelAndJoin()
            release.complete(Unit)
            assertTrue(holder.await())
            // Queue is clean: a fresh trigger returns immediately as a no-op.
            assertFalse(f.coordinator.trigger())
        }
        assertEquals(1, f.starts.get())
        assertFalse(f.coordinator.isStarting)
    }

    @Test
    fun cancellingTheInFlightStartReleasesTheMutex() = runBlocking {
        val entered = CompletableDeferred<Unit>()
        val f = Fixture(startBody = { entered.complete(Unit); delay(60_000); true })
        withTimeout(deadlockMs) {
            val holder = launch(Dispatchers.Default) { f.coordinator.trigger() }
            entered.await()
            assertTrue(f.coordinator.isStarting)
            holder.cancelAndJoin()
            assertFalse(f.coordinator.isStarting)
            assertEquals(0, f.inFlight.get())
        }
        // The cancelled start did not wire a bridge, so the next trigger starts.
        val g = Fixture()
        withTimeout(deadlockMs) { assertTrue(g.coordinator.trigger()) }
    }

    @Test
    fun headlessToFullUpgradeUnderConcurrencyStartsOncePerTransition() = runBlocking {
        val f = Fixture(identity = SwarmIdentityRead(known = true, present = false))
        withTimeout(deadlockMs) {
            (1..50).map { async(Dispatchers.Default) { f.coordinator.trigger() } }.awaitAll()
            assertEquals(1, f.starts.get())
            assertEquals(false, f.coordinator.startedWithIdentity)
            f.identity = SwarmIdentityRead(known = true, present = true)
            val ran = (1..50).map { async(Dispatchers.Default) { f.coordinator.trigger() } }.awaitAll()
            assertEquals(1, ran.count { it })
        }
        assertEquals(2, f.starts.get())
        assertEquals(1, f.maxInFlight.get())
        assertEquals(true, f.coordinator.startedWithIdentity)
    }

    @Test
    fun resetIsIdempotentAndAllowsRestartAfterStop() = runBlocking {
        val f = Fixture()
        withTimeout(deadlockMs) {
            assertTrue(f.coordinator.trigger())
            f.bridge = false // service stop clears the bridge
            repeat(5) { f.coordinator.reset() }
            assertNull(f.coordinator.startedWithIdentity)
            assertTrue(f.coordinator.trigger())
            assertFalse(f.coordinator.trigger())
        }
        assertEquals(2, f.starts.get())
    }

    @Test
    fun triggersRacingWithResetNeverDeadlockOrOverlap() = runBlocking {
        val f = Fixture(startBody = { delay(1); true })
        withTimeout(deadlockMs) {
            withContext(Dispatchers.Default) {
                val jobs = (1..100).map { i ->
                    async {
                        if (i % 5 == 0) {
                            f.bridge = false
                            f.coordinator.reset()
                        }
                        f.coordinator.trigger()
                    }
                }
                jobs.awaitAll()
            }
        }
        assertEquals(1, f.maxInFlight.get())
        assertFalse(f.coordinator.isStarting)
    }
}
