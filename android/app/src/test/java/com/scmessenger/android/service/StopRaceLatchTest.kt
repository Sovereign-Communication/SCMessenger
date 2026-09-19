package com.scmessenger.android.service

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

/**
 * STOP-RACE-001 (2026-09-11): ACTION_STOP must set userStoppedForSession
 * SYNCHRONOUSLY inside decideCommand so a racing ensureServiceInitializedDeferred
 * cannot resurrect the mesh ~1s later.
 */
class StopRaceLatchTest {

    @Before
    fun resetLatch() {
        MeshForegroundService.userStoppedForSession = false
    }

    @Test
    fun actionStop_setsLatchSynchronously() {
        val decision = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_STOP,
            serviceRunning = true,
            repositoryRunning = true,
        )
        assertEquals("Stop", decision.name)
        assertTrue(
            "userStoppedForSession must be set inside decideCommand for STOP",
            MeshForegroundService.userStoppedForSession
        )
    }

    @Test
    fun ensureWhileLatched_isNoOp() {
        MeshForegroundService.userStoppedForSession = true
        val decision = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_ENSURE,
            serviceRunning = false,
            repositoryRunning = false,
        )
        assertEquals("NoOp", decision.name)
    }

    @Test
    fun explicitStart_clearsLatch() {
        MeshForegroundService.userStoppedForSession = true
        val decision = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_START,
            serviceRunning = false,
            repositoryRunning = false,
        )
        assertEquals("Start", decision.name)
        assertTrue(!MeshForegroundService.userStoppedForSession)
    }
}
