package com.scmessenger.android.service

import com.scmessenger.android.ui.viewmodels.MeshServiceViewModel
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.api.ServiceState

/**
 * STOP-TEARDOWN-TIMEOUT-001 (2026-09-21): the mesh could not be stopped on the
 * Pixel. Measured there: one Stop tap logged
 * `MeshServiceViewModel$stopService: stop requested` and
 * `MeshForegroundService$stopMeshService: Stopping mesh service`, the service
 * record and its foreground notification survived, and the UI kept reading
 * RUNNING -- so the user tapped again, and again.
 *
 * Cause: MeshRepository.stopMeshService() awaited an unbounded FFI call
 * (`runBlocking { swarmBridge?.shutdown() }`) while holding
 * serviceLifecycleLock, so stopForeground()/stopSelf() and the STOPPED state
 * assignment, which all run after it, never executed. That is fixed by bounding
 * both FFI awaits in MeshRepository.
 *
 * These tests pin the two *decision* rules that let the failure stay invisible,
 * because they are the parts a future edit can silently regress:
 *   1. A latched stop must not ask the system for a sticky restart.
 *   2. Only STOPPED may start the mesh; every other state is another stop.
 *
 * They are pure (no Android runtime, no native library), which is deliberate:
 * the previous six fixes to this behaviour each shipped with a unit test of a
 * pure decision function and none with an on-device completion check, which is
 * why the symptom kept returning. The completion half is covered by
 * scripts/pixel_stop_acceptance.sh.
 */
class StopTeardownTimeoutTest {

    @Before
    fun resetLatch() {
        MeshForegroundService.userStoppedForSession = false
    }

    // ---- 1. a latched stop must not request a sticky restart ----------------

    @Test
    fun `latched null-action delivery does not request a sticky restart`() {
        assertFalse(
            "a latched stop must not invite the system to redeliver the service",
            MeshForegroundService.shouldReturnSticky(latched = true, action = null)
        )
    }

    @Test
    fun `latched stop delivery does not request a sticky restart`() {
        assertFalse(
            MeshForegroundService.shouldReturnSticky(
                latched = true,
                action = MeshForegroundService.ACTION_STOP
            )
        )
    }

    @Test
    fun `explicit start after a latched stop does request a sticky restart`() {
        assertTrue(
            MeshForegroundService.shouldReturnSticky(
                latched = true,
                action = MeshForegroundService.ACTION_START
            )
        )
    }

    @Test
    fun `unlatched deliveries keep the sticky default`() {
        assertTrue(MeshForegroundService.shouldReturnSticky(latched = false, action = null))
        assertTrue(
            MeshForegroundService.shouldReturnSticky(
                latched = false,
                action = MeshForegroundService.ACTION_ENSURE
            )
        )
        assertTrue(
            MeshForegroundService.shouldReturnSticky(
                latched = false,
                action = MeshForegroundService.ACTION_RESUME
            )
        )
    }

    @Test
    fun `stop delivery composes with the sticky rule`() {
        val decision = MeshForegroundService.decideCommand(
            MeshForegroundService.ACTION_STOP,
            serviceRunning = true,
            repositoryRunning = true
        )
        assertEquals("Stop", decision.name)
        assertFalse(
            "onStartCommand must refuse the sticky restart for the delivery that latched the stop",
            MeshForegroundService.shouldReturnSticky(
                MeshForegroundService.userStoppedForSession,
                MeshForegroundService.ACTION_STOP
            )
        )
    }

    // ---- 2. only STOPPED may start the mesh ---------------------------------

    @Test
    fun `only STOPPED may start the mesh`() {
        assertTrue(MeshServiceViewModel.shouldStartFrom(ServiceState.STOPPED))
        assertFalse(
            "STARTING must not start again",
            MeshServiceViewModel.shouldStartFrom(ServiceState.STARTING)
        )
        assertFalse(
            "RUNNING must resolve to a stop",
            MeshServiceViewModel.shouldStartFrom(ServiceState.RUNNING)
        )
    }

    @Test
    fun `STOPPING resolves to another stop, never a restart`() {
        assertFalse(
            "a repeated tap during an asynchronous teardown must re-request the stop",
            MeshServiceViewModel.shouldStartFrom(ServiceState.STOPPING)
        )
    }
}
