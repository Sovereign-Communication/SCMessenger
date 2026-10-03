// android/app/src/test/java/com/scmessenger/android/service/MeshLifecycleSerializationTest.kt
package com.scmessenger.android.service

import android.content.Intent
import com.scmessenger.android.data.MeshRepository
import io.mockk.every
import io.mockk.mockk
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.api.ServiceState
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import kotlin.concurrent.thread

/**
 * AND-SS-001: the serialization half of the mesh Stop/Start fix.
 *
 * [StopStartFloodTest] pins the decision-latch semantics. This class pins the
 * other half -- [MeshForegroundService.lifecycleMutex] -- which is what stops
 * a start from interleaving with a teardown. The harness review of the fix
 * raised exactly this as an open BLOCK: latch semantics were covered, mutex
 * serialization was not, so a regression that removed the mutex would have
 * shipped green.
 *
 * These tests drive the real [MeshForegroundService.onStartCommand] entry
 * point, not the private helpers, so they exercise the actual lock scope and
 * the actual command-registration boundary. ACTION_STOP is used because it is
 * the one action that does not attempt synchronous foreground promotion, which
 * keeps the test on the JVM tier (no Robolectric; it was removed 2026-07-27).
 *
 * What is asserted is the INVARIANT -- two lifecycle bodies never occupy the
 * service at the same time -- and not the presence of any particular field or
 * lock. Deleting `lifecycleMutex.withLock` makes both tests fail on
 * `maxConcurrentBodies`, which is the regression we need to catch.
 */
class MeshLifecycleSerializationTest {

    private lateinit var service: MeshForegroundService
    private lateinit var repository: MeshRepository

    @Before
    fun setUp() {
        MeshForegroundService.userStoppedForSession = false
        service = MeshForegroundService()
        repository = mockk(relaxed = true)
        // onCreate() is never invoked on this tier, so the lateinit collaborators
        // the lifecycle bodies call must be supplied directly.
        inject("meshRepository", repository)
        inject("platformBridge", mockk<AndroidPlatformBridge>(relaxed = true))
        inject("anrWatchdog", mockk<AnrWatchdog>(relaxed = true))
        inject("performanceMonitor", mockk<PerformanceMonitor>(relaxed = true))
        inject("serviceHealthMonitor", mockk<ServiceHealthMonitor>(relaxed = true))
    }

    @Test
    fun secondLifecycleBody_cannotEnterUntilTeardownHasFinished() {
        val inBody = AtomicInteger(0)
        val maxConcurrentBodies = AtomicInteger(0)
        val bodyEntries = AtomicInteger(0)
        val teardownEntered = CountDownLatch(1)
        val releaseTeardown = CountDownLatch(1)

        every { repository.getServiceStateSync() } answers {
            trackEntry(inBody, maxConcurrentBodies)
            try {
                if (bodyEntries.getAndIncrement() == 0) {
                    // Hold the first body inside the service so a second one
                    // that is not serialized would visibly overlap it.
                    teardownEntered.countDown()
                    releaseTeardown.await(10, TimeUnit.SECONDS)
                }
                Thread.sleep(25)
            } finally {
                inBody.decrementAndGet()
            }
            ServiceState.RUNNING
        }
        every { repository.stopMeshService() } returns Unit

        // Body 1: a user Stop, which reaches the repository and parks there.
        val stopCaller = thread { service.onStartCommand(intentFor(MeshForegroundService.ACTION_STOP), 0, 1) }
        assertTrue(
            "teardown never reached the repository; the test proved nothing",
            teardownEntered.await(10, TimeUnit.SECONDS),
        )

        // Body 2: a Pause registered while the teardown is still running. A
        // user stop latches, and only an explicit START clears it, so mirror
        // that here to get a second command admitted.
        MeshForegroundService.userStoppedForSession = false
        service.onStartCommand(intentFor(MeshForegroundService.ACTION_PAUSE), 0, 2)

        Thread.sleep(400)
        assertEquals(
            "a second lifecycle body entered while the teardown was still in flight",
            1,
            bodyEntries.get(),
        )
        assertEquals(
            "two lifecycle bodies overlapped: lifecycleMutex is not serializing them",
            1,
            maxConcurrentBodies.get(),
        )

        releaseTeardown.countDown()
        stopCaller.join(10_000)
        assertTrue(
            "the queued Pause never ran after the teardown released the service",
            awaitAtLeast(bodyEntries, 2),
        )
        assertEquals(
            "lifecycle bodies overlapped at some point during the sequence",
            1,
            maxConcurrentBodies.get(),
        )
    }

    @Test
    fun concurrentStopFlood_runsExactlyOneTeardown() {
        val teardowns = AtomicInteger(0)
        val inBody = AtomicInteger(0)
        val maxConcurrentBodies = AtomicInteger(0)
        val registered = CountDownLatch(6)

        every { repository.getServiceStateSync() } answers {
            trackEntry(inBody, maxConcurrentBodies)
            try {
                // Keep the first teardown in flight until every caller has
                // been through command registration, so a STOP arriving after
                // the first teardown settled cannot be mistaken for a flood
                // regression.
                registered.await(10, TimeUnit.SECONDS)
                Thread.sleep(40)
            } finally {
                inBody.decrementAndGet()
            }
            ServiceState.RUNNING
        }
        every { repository.stopMeshService() } answers { teardowns.incrementAndGet() }

        val callers = (1..6).map { startId ->
            thread {
                service.onStartCommand(intentFor(MeshForegroundService.ACTION_STOP), 0, startId)
                registered.countDown()
            }
        }
        callers.forEach { it.join(10_000) }

        assertTrue("the one admitted teardown never ran", awaitAtLeast(teardowns, 1))
        // Let any wrongly-admitted body finish before asserting the count.
        Thread.sleep(400)
        assertEquals(
            "six concurrent STOPs produced more than one teardown",
            1,
            teardowns.get(),
        )
        assertEquals(
            "teardowns overlapped: lifecycleMutex is not serializing them",
            1,
            maxConcurrentBodies.get(),
        )
    }

    /**
     * Delegation proof, not a serialization proof.
     *
     * `decideCommand` is now the only writer of `userStoppedForSession` on the
     * live path -- `stopMeshServiceLocked` deliberately no longer writes it --
     * so a latch that flips as a result of a real `onStartCommand` can only
     * have been flipped by `decideCommand`. That is what distinguishes the
     * decision being REACHED IN PRODUCTION from the decision merely being
     * covered by the pure unit tests.
     *
     * The second half asserts the gate, not the write: a PAUSE arriving after
     * the user stop must be refused, so no second lifecycle body may enter the
     * repository. That is the actual field regression the latch exists to
     * prevent (a stopped mesh being resurrected by an automatic command).
     *
     * ACTION_STOP and ACTION_PAUSE are used because neither attempts
     * synchronous foreground promotion, which keeps the test on the JVM tier
     * (no Robolectric; it was removed 2026-07-27).
     */
    @Test
    fun onStartCommand_delegatesToDecideCommand() {
        val bodies = AtomicInteger(0)
        every { repository.getServiceStateSync() } answers {
            bodies.incrementAndGet()
            ServiceState.RUNNING
        }
        every { repository.stopMeshService() } returns Unit

        assertEquals(
            "the stop latch should start clear for this test",
            false,
            MeshForegroundService.userStoppedForSession,
        )

        service.onStartCommand(intentFor(MeshForegroundService.ACTION_STOP), 0, 1)

        assertTrue(
            "onStartCommand(ACTION_STOP) did not reach decideCommand: it is the " +
                "only writer of the user-stop latch on the live path",
            MeshForegroundService.userStoppedForSession,
        )
        assertTrue(
            "the admitted Stop never ran, so the rest of this test proves nothing",
            awaitAtLeast(bodies, 1),
        )

        service.onStartCommand(intentFor(MeshForegroundService.ACTION_PAUSE), 0, 2)

        // Let any wrongly-admitted body finish before asserting the count.
        Thread.sleep(400)
        assertEquals(
            "a PAUSE after a user stop opened a second lifecycle body: the stop " +
                "gate in decideCommand was not applied on the live path",
            1,
            bodies.get(),
        )
    }

    private fun trackEntry(inBody: AtomicInteger, maxConcurrentBodies: AtomicInteger) {
        val occupancy = inBody.incrementAndGet()
        maxConcurrentBodies.accumulateAndGet(occupancy) { current, next -> maxOf(current, next) }
    }

    private fun awaitAtLeast(counter: AtomicInteger, target: Int): Boolean {
        val deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(10)
        while (System.nanoTime() < deadline) {
            if (counter.get() >= target) return true
            Thread.sleep(20)
        }
        return counter.get() >= target
    }

    private fun intentFor(action: String): Intent =
        mockk<Intent>(relaxed = true).also { every { it.action } returns action }

    private fun inject(fieldName: String, value: Any) {
        val field = service.javaClass.getDeclaredField(fieldName)
        field.isAccessible = true
        field.set(service, value)
    }
}
