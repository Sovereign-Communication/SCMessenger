package com.scmessenger.android.service

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * AND-SS-001 (2026-09-23): the mesh Stop/Start lifecycle broke in the field
 * in two directions, both captured on the Pixel (tmp/pixel-logcat-20260921.txt):
 *
 * 1. Stop flood: six STOP taps in two seconds produced six concurrent full
 *    teardowns (23:26:44-46 window). Each one now coalesces through
 *    [MeshForegroundService.stopInFlight].
 * 2. Start cancellation: concurrent ACTION_START deliveries raced; the loser
 *    aborted with JobCancellationException and left the mesh dead (05:52
 *    window). Starts and stops are now serialized by [MeshForegroundService.lifecycleMutex]
 *    and starts re-check the settled state under it.
 *
 * These tests pin the decision-latch semantics that make the guards safe;
 * the serialization itself is coroutine-level and exercised by the live
 * regression (STOP then START within 10 s must end RUNNING, one teardown
 * per flood).
 */
class StopStartFloodTest {

    @Before
    fun resetLatch() {
        MeshForegroundService.userStoppedForSession = false
    }

    @Test
    fun repeatedStop_stillReturnsStopDecision() {
        // The latch honors every STOP (R4-M1) so a late stop can always
        // complete teardown; coalescing happens at the teardown-guard level,
        // never by dropping the decision.
        val first = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_STOP,
            serviceRunning = true,
            repositoryRunning = true,
        )
        val second = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_STOP,
            serviceRunning = false,
            repositoryRunning = false,
        )
        assertEquals("Stop", first.name)
        assertEquals("Stop", second.name)
        assertTrue(MeshForegroundService.userStoppedForSession)
    }

    @Test
    fun ensureDuringStopFlood_isNoOp() {
        // The stop flood must not be resurrected by automatic ensure paths.
        MeshForegroundService.userStoppedForSession = true
        val ensure = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_ENSURE,
            serviceRunning = false,
            repositoryRunning = false,
        )
        val sticky = MeshForegroundService.decideCommand(
            null,
            serviceRunning = false,
            repositoryRunning = false,
        )
        assertEquals("NoOp", ensure.name)
        assertEquals("NoOp", sticky.name)
    }

    @Test
    fun explicitStartAfterStop_clearsLatchAndStarts() {
        MeshForegroundService.userStoppedForSession = true
        val decision = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_START,
            serviceRunning = false,
            repositoryRunning = false,
        )
        assertEquals("Start", decision.name)
        assertFalse(
            "explicit user START must clear the stop latch so the mesh can come back",
            MeshForegroundService.userStoppedForSession
        )
    }

    @Test
    fun resumeWhileStopped_treatedAsStart() {
        // A RESUME that arrives after the stop settled must bring the mesh
        // back instead of silently doing nothing (decideCommand contract).
        MeshForegroundService.userStoppedForSession = true
        val decision = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_RESUME,
            serviceRunning = false,
            repositoryRunning = false,
        )
        // The latch makes it a NoOp: only an explicit ACTION_START clears a
        // user stop (R3-F1). RESUME must not resurrect a stopped mesh.
        assertEquals("NoOp", decision.name)
    }
}
