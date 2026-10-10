package com.scmessenger.android.data

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.Collections
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/**
 * STOP-TEARDOWN-TIMEOUT-001 follow-up: the property, not a proxy for it.
 *
 * StopTeardownTimeoutTest pins the pure decision functions. Seven earlier fixes
 * shipped with only that kind of test and the hang kept coming back, because
 * nothing ever handed the stop path a call that does not return. These tests do:
 * a fake swarm bridge whose shutdown() never completes, a fake Rust service
 * whose stop() blocks forever, and a fake repository stop that never returns.
 * Pure JVM -- no Robolectric, no native library.
 */
class StopTeardownBoundTest {

    private val lines = Collections.synchronizedList(mutableListOf<String>())
    private val release = CountDownLatch(1)

    @After
    fun releaseStuckThreads() {
        // Let the abandoned daemon threads finish so they do not outlive the test.
        release.countDown()
    }

    private fun teardown() = StopTeardown(log = { lines.add(it) })

    private fun elapsedMs(block: () -> Unit): Long {
        val start = System.nanoTime()
        block()
        return (System.nanoTime() - start) / 1_000_000L
    }

    // ---- swarm shutdown ------------------------------------------------------

    @Test
    fun `swarm shutdown that suspends forever returns within the bound`() {
        var outcome: StopOutcome? = null
        val ms = elapsedMs {
            outcome = teardown().shutdownSwarm(300L) { CompletableDeferred<Unit>().await() }
        }
        assertEquals(StopOutcome.TIMEOUT, outcome)
        assertTrue("took ${ms}ms, bound was 300ms", ms < 3_000L)
        assertTrue(lines.any { it.startsWith("swarm_shutdown timeout ms=") })
    }

    @Test
    fun `swarm shutdown that blocks the thread returns within the bound plus grace`() {
        var outcome: StopOutcome? = null
        val ms = elapsedMs {
            // Blocking without suspending: a coroutine timeout cannot interrupt this.
            outcome = teardown().shutdownSwarm(200L) { release.await(60, TimeUnit.SECONDS) }
        }
        assertEquals(StopOutcome.TIMEOUT, outcome)
        assertTrue("took ${ms}ms", ms < 200L + StopTeardown.JOIN_GRACE_MS + 2_000L)
    }

    @Test
    fun `swarm shutdown that completes reports ok`() {
        val outcome = teardown().shutdownSwarm(2_000L) { }
        assertEquals(StopOutcome.OK, outcome)
        assertTrue(lines.any { it.startsWith("swarm_shutdown ok ms=") })
    }

    @Test
    fun `swarm shutdown that throws reports failed and does not propagate`() {
        val outcome = teardown().shutdownSwarm(2_000L) { throw IllegalStateException("boom") }
        assertEquals(StopOutcome.FAILED, outcome)
        assertTrue(lines.any { it.startsWith("swarm_shutdown failed ms=") })
    }

    // ---- blocking Rust stop --------------------------------------------------

    @Test
    fun `rust stop that never returns is abandoned at the bound`() {
        var outcome: StopOutcome? = null
        val ms = elapsedMs {
            outcome = teardown().stopBlocking("rust_stop", 300L) { release.await(60, TimeUnit.SECONDS) }
        }
        assertEquals(StopOutcome.TIMEOUT, outcome)
        assertTrue("took ${ms}ms, bound was 300ms", ms < 3_000L)
        assertTrue(lines.any { it.startsWith("rust_stop timeout ms=") })
    }

    @Test
    fun `rust stop that returns reports ok`() {
        assertEquals(StopOutcome.OK, teardown().stopBlocking("rust_stop", 2_000L) { })
        assertTrue(lines.any { it.startsWith("rust_stop ok ms=") })
    }

    // ---- service side: stopForeground / stopSelf must be reached -------------

    private class Recorder {
        val steps = Collections.synchronizedList(mutableListOf<String>())
    }

    @Test
    fun `service stop reaches stopForeground and stopSelf when the repository stop never returns`() {
        val rec = Recorder()
        var repositoryOk: Boolean? = null
        val ms = elapsedMs {
            repositoryOk = runBlocking {
                ServiceStopSequence.run(
                    repositoryStop = { release.await(60, TimeUnit.SECONDS) },
                    localTeardown = { rec.steps.add("local") },
                    platformCleanup = { rec.steps.add("platform") },
                    removeForeground = { rec.steps.add("stopForeground") },
                    stopSelf = { rec.steps.add("stopSelf") },
                    repositoryBoundMs = 300L,
                    platformBoundMs = 300L,
                    log = { lines.add(it) }
                )
            }
        }
        assertEquals(false, repositoryOk)
        assertTrue("took ${ms}ms, bound was 300ms", ms < 5_000L)
        assertEquals(listOf("local", "platform", "stopForeground", "stopSelf"), rec.steps.toList())
        assertTrue(lines.any { it.startsWith("repository_stop timeout ms=") })
        assertMarkersInOrder("requested", "repository_stop timeout", "foreground_removed", "complete")
    }

    @Test
    fun `service stop reaches stopForeground and stopSelf when platform cleanup hangs`() {
        val rec = Recorder()
        val ms = elapsedMs {
            runBlocking {
                ServiceStopSequence.run(
                    repositoryStop = { },
                    localTeardown = { },
                    platformCleanup = { release.await(60, TimeUnit.SECONDS) },
                    removeForeground = { rec.steps.add("stopForeground") },
                    stopSelf = { rec.steps.add("stopSelf") },
                    platformBoundMs = 300L,
                    log = { lines.add(it) }
                )
            }
        }
        assertTrue("took ${ms}ms", ms < 5_000L)
        assertEquals(listOf("stopForeground", "stopSelf"), rec.steps.toList())
    }

    @Test
    fun `service stop reaches stopForeground and stopSelf when local teardown throws`() {
        val rec = Recorder()
        runBlocking {
            ServiceStopSequence.run(
                repositoryStop = { throw IllegalStateException("repo") },
                localTeardown = { throw IllegalStateException("local") },
                platformCleanup = { },
                removeForeground = { rec.steps.add("stopForeground") },
                stopSelf = { rec.steps.add("stopSelf") },
                log = { lines.add(it) }
            )
        }
        assertEquals(listOf("stopForeground", "stopSelf"), rec.steps.toList())
        assertTrue(lines.any { it.startsWith("repository_stop failed") })
    }

    @Test
    fun `healthy service stop emits the full marker sequence`() {
        val rec = Recorder()
        val ok = runBlocking {
            ServiceStopSequence.run(
                repositoryStop = { rec.steps.add("repo") },
                localTeardown = { },
                platformCleanup = { },
                removeForeground = { rec.steps.add("stopForeground") },
                stopSelf = { rec.steps.add("stopSelf") },
                log = { lines.add(it) }
            )
        }
        assertTrue(ok)
        assertEquals(listOf("repo", "stopForeground", "stopSelf"), rec.steps.toList())
        assertMarkersInOrder("requested", "repository_stop ok", "foreground_removed", "complete")
        assertTrue(lines.last().contains("repository=ok"))
    }

    // ---- file-log marker contract -------------------------------------------

    @Test
    fun `MeshStopLog prefixes every line with the audit marker`() {
        val captured = mutableListOf<String>()
        val previous = MeshStopLog.sink
        MeshStopLog.sink = { captured.add(it) }
        try {
            MeshStopLog.emit("requested")
        } finally {
            MeshStopLog.sink = previous
        }
        assertEquals(listOf("[MESH-STOP] requested"), captured)
    }

    @Test
    fun `MeshStopLog swallows a failing sink`() {
        val previous = MeshStopLog.sink
        MeshStopLog.sink = { throw RuntimeException("disk full") }
        try {
            MeshStopLog.emit("requested") // must not throw into the stop path
        } finally {
            MeshStopLog.sink = previous
        }
        assertFalse(lines.contains("never"))
    }

    private fun assertMarkersInOrder(vararg prefixes: String) {
        var from = 0
        val snapshot = lines.toList()
        for (p in prefixes) {
            val idx = snapshot.drop(from).indexOfFirst { it.startsWith(p) }
            assertTrue("marker '$p' missing or out of order in $snapshot", idx >= 0)
            from += idx + 1
        }
    }
}
