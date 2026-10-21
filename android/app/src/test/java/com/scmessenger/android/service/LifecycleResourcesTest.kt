package com.scmessenger.android.service

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class LifecycleResourcesTest {

    /** Counts OS-side registrations so tests can assert register/unregister pairing. */
    private class FakeRegistry {
        var registered = 0
        var unregistered = 0
        val live: Int get() = registered - unregistered
    }

    private class FakeRevivalBackend : RevivalWorkBackend {
        val calls = mutableListOf<String>()
        override fun enqueuePeriodicSync() { calls += "enqueue" }
        override fun cancelPeriodicSync() { calls += "cancelPeriodic" }
        override fun cancelEnsureWorker() { calls += "cancelEnsure" }
    }

    private fun managed(registry: FakeRegistry) = ManagedResource(
        onOpen = { registry.registered++ },
        onClose = { registry.unregistered++ }
    )

    @Test
    fun managedResource_openThenClose_isPaired() {
        val reg = FakeRegistry()
        val r = managed(reg)
        r.open()
        assertEquals(1, reg.live)
        r.close()
        assertEquals(1, reg.registered)
        assertEquals(1, reg.unregistered)
        assertEquals(0, reg.live)
    }

    @Test
    fun managedResource_doubleOpen_registersOnce() {
        val reg = FakeRegistry()
        val r = managed(reg)
        r.open()
        r.open()
        assertEquals(1, reg.registered)
        assertEquals(1, reg.live)
    }

    @Test
    fun managedResource_doubleClose_unregistersOnce() {
        val reg = FakeRegistry()
        val r = managed(reg)
        r.open()
        r.close()
        r.close()
        assertEquals(1, reg.unregistered)
        assertEquals(0, reg.live)
    }

    @Test
    fun managedResource_closeBeforeOpen_isNoOp() {
        val reg = FakeRegistry()
        val r = managed(reg)
        r.close()
        assertEquals(0, reg.unregistered)
        assertEquals(0, reg.registered)
    }

    @Test
    fun managedResource_failedOpen_staysClosed_andCloseDoesNotUnregister() {
        val reg = FakeRegistry()
        val r = ManagedResource(
            onOpen = { throw IllegalStateException("no network service") },
            onClose = { reg.unregistered++ }
        )
        try {
            r.open()
            fail("open should propagate the failure")
        } catch (expected: IllegalStateException) {
            // expected
        }
        assertEquals(0, reg.live)
        r.close()
        assertEquals(0, reg.unregistered)
    }

    @Test
    fun managedResource_reopenAfterClose_registersAgain() {
        val reg = FakeRegistry()
        val r = managed(reg)
        r.open(); r.close(); r.open(); r.close()
        assertEquals(2, reg.registered)
        assertEquals(2, reg.unregistered)
        assertEquals(0, reg.live)
    }

    @Test
    fun revival_userStop_cancelsPeriodicAndEnsure() {
        val backend = FakeRevivalBackend()
        RevivalWork(backend).onUserStopped()
        assertEquals(listOf("cancelPeriodic", "cancelEnsure"), backend.calls)
    }

    @Test
    fun revival_userStop_cancelsEvenWithoutStartInThisProcess() {
        // After a process death the in-memory state is empty but WorkManager still
        // holds the periodic work, so stop must cancel unconditionally.
        val backend = FakeRevivalBackend()
        val work = RevivalWork(backend)
        work.onUserStopped()
        assertTrue(backend.calls.contains("cancelPeriodic"))
    }

    @Test
    fun revival_startThenStop_thenStartAgain_reenqueues() {
        val backend = FakeRevivalBackend()
        val work = RevivalWork(backend)
        work.onMeshStarted()
        work.onUserStopped()
        work.onMeshStarted()
        assertEquals(
            listOf("enqueue", "cancelPeriodic", "cancelEnsure", "enqueue"),
            backend.calls
        )
    }

    @Test
    fun revival_repeatedStop_isSafe_andEveryStopCancels() {
        val backend = FakeRevivalBackend()
        val work = RevivalWork(backend)
        work.onUserStopped()
        work.onUserStopped()
        assertEquals(4, backend.calls.size)
        assertEquals(0, backend.calls.count { it == "enqueue" })
    }
}
