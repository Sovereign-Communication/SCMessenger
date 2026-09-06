package com.scmessenger.android.service

import com.scmessenger.android.service.MeshForegroundService
import org.junit.Assert.assertEquals
import org.junit.Test

class MeshForegroundServiceTest {

    @Test
    fun `null action resolves to Start`() {
        val result = MeshForegroundService.decideCommand(
            action = null,
            serviceRunning = false,
            repositoryRunning = false
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
    }

    @Test
    fun `pause resolves to NoOp when service not running`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_PAUSE,
            serviceRunning = false,
            repositoryRunning = false
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.NoOp, result)
    }

    @Test
    fun `pause resolves to Pause when repository running`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_PAUSE,
            serviceRunning = false,
            repositoryRunning = true
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Pause, result)
    }

    @Test
    fun `resume resolves to Resume only when both running`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_RESUME,
            serviceRunning = true,
            repositoryRunning = true
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Resume, result)
    }

    @Test
    fun `resume resolves to Start when state is incomplete`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_RESUME,
            serviceRunning = true,
            repositoryRunning = false
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
    }

    @Test
    fun `stop resolves to Stop`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_STOP,
            serviceRunning = true,
            repositoryRunning = true
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Stop, result)
    }

    @Test
    fun `unknown action defaults to Start`() {
        val result = MeshForegroundService.decideCommand(
            action = "unknown",
            serviceRunning = true,
            repositoryRunning = true
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
    }

    @Test
    fun `any action after user stop resolves to NoOp`() {
        MeshForegroundService.userStoppedForSession = true
        try {
            for (action in listOf(null, MeshForegroundService.ACTION_PAUSE, MeshForegroundService.ACTION_RESUME, "unknown")) {
                val result = MeshForegroundService.decideCommand(
                    action = action,
                    serviceRunning = true,
                    repositoryRunning = true
                )
                assertEquals(MeshForegroundService.Companion.StartDecision.NoOp, result)
            }
        } finally {
            MeshForegroundService.userStoppedForSession = false
        }
    }

    @Test
    fun `explicit start clears user stop and starts`() {
        MeshForegroundService.userStoppedForSession = true
        try {
            val result = MeshForegroundService.decideCommand(
                action = MeshForegroundService.ACTION_START,
                serviceRunning = false,
                repositoryRunning = false
            )
            assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
            assertEquals(false, MeshForegroundService.userStoppedForSession)
        } finally {
            MeshForegroundService.userStoppedForSession = false
        }
    }

    @Test
    fun `stop decision recorded via latch does not block a later explicit start`() {
        MeshForegroundService.userStoppedForSession = true
        try {
            // The latch survives other triggers...
            assertEquals(
                MeshForegroundService.Companion.StartDecision.NoOp,
                MeshForegroundService.decideCommand(null, true, true)
            )
            // ...but an explicit start clears it.
            assertEquals(
                MeshForegroundService.Companion.StartDecision.Start,
                MeshForegroundService.decideCommand(MeshForegroundService.ACTION_START, false, false)
            )
            assertEquals(false, MeshForegroundService.userStoppedForSession)
        } finally {
            MeshForegroundService.userStoppedForSession = false
        }
    }

    @Test
    fun `stop is honored even when latch already set`() {
        MeshForegroundService.userStoppedForSession = true
        try {
            // R4-M1: a repeated or late STOP must complete teardown, not be
            // swallowed by the latch.
            val result = MeshForegroundService.decideCommand(
                action = MeshForegroundService.ACTION_STOP,
                serviceRunning = true,
                repositoryRunning = true
            )
            assertEquals(MeshForegroundService.Companion.StartDecision.Stop, result)
        } finally {
            MeshForegroundService.userStoppedForSession = false
        }
    }

    @Test
    fun `ensure action never clears the user stop latch`() {
        MeshForegroundService.userStoppedForSession = true
        try {
            val result = MeshForegroundService.decideCommand(
                action = MeshForegroundService.ACTION_ENSURE,
                serviceRunning = false,
                repositoryRunning = false
            )
            assertEquals(MeshForegroundService.Companion.StartDecision.NoOp, result)
            assertEquals(true, MeshForegroundService.userStoppedForSession)
        } finally {
            MeshForegroundService.userStoppedForSession = false
        }
    }

    @Test
    fun `explicit start resolves to Start`() {
        val result = MeshForegroundService.decideCommand(
            action = MeshForegroundService.ACTION_START,
            serviceRunning = false,
            repositoryRunning = false
        )
        assertEquals(MeshForegroundService.Companion.StartDecision.Start, result)
    }
}
