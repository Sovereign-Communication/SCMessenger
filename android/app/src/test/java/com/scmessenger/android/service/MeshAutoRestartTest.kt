package com.scmessenger.android.service

import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class MeshAutoRestartTest {

    private val lines = mutableListOf<String>()

    @After
    fun reset() {
        MeshForegroundService.userStopPersist = null
        MeshForegroundService.userStoppedForSession = false
    }

    private fun run(
        trigger: MeshStartTrigger,
        userStopped: Boolean,
        alive: Boolean,
        start: () -> Unit = {}
    ) = MeshAutoRestart.ensure(trigger, userStopped, alive, start) { lines += it }

    @Test
    fun userStop_blocksEveryTrigger_andNeverStarts() {
        var starts = 0
        MeshStartTrigger.values().forEach {
            assertEquals(MeshEnsureResult.UserStopped, run(it, userStopped = true, alive = false) { starts++ })
        }
        assertEquals(0, starts)
        assertTrue(lines.all { it.endsWith("result=user_stopped") })
    }

    @Test
    fun notStopped_andNotAlive_startsForEveryTrigger() {
        MeshStartTrigger.values().forEach {
            var started = false
            assertEquals(MeshEnsureResult.Started, run(it, userStopped = false, alive = false) { started = true })
            assertTrue(started)
        }
        assertEquals("[MESH-START] trigger=process_start result=started", lines.first())
    }

    @Test
    fun alreadyRunning_skipsStart_exceptActivity() {
        var starts = 0
        assertEquals(MeshEnsureResult.AlreadyRunning, run(MeshStartTrigger.WORKER, false, true) { starts++ })
        assertEquals(0, starts)
        assertEquals(MeshEnsureResult.Started, run(MeshStartTrigger.ACTIVITY, false, true) { starts++ })
        assertEquals(1, starts)
    }

    @Test
    fun blockedStart_isReportedWithReason() {
        val r = run(MeshStartTrigger.WORKER, false, false) { throw IllegalStateException("bg start not allowed") }
        assertTrue(r is MeshEnsureResult.Blocked)
        assertEquals(
            "[MESH-START] trigger=worker result=blocked reason=IllegalStateException:bg start not allowed",
            lines.single()
        )
    }

    @Test
    fun networkAlreadyRunning_isNotLogged_butOtherNetworkOutcomesAre() {
        run(MeshStartTrigger.NETWORK, false, true)
        assertTrue(lines.isEmpty())
        run(MeshStartTrigger.NETWORK, false, false)
        assertEquals("[MESH-START] trigger=network result=started", lines.single())
    }

    @Test
    fun logNames_matchSpec() {
        assertEquals(
            listOf("process_start", "worker", "boot", "package_replaced", "network", "activity"),
            MeshStartTrigger.values().map { it.logName }
        )
    }

    @Test
    fun userStopLatch_isMirroredToPersistence() {
        val written = mutableListOf<Boolean>()
        MeshForegroundService.userStopPersist = { written += it }
        MeshForegroundService.decideCommand(MeshForegroundService.ACTION_STOP, true, true)
        MeshForegroundService.decideCommand(MeshForegroundService.ACTION_START, false, false)
        assertEquals(listOf(true, false), written.takeLast(2))
        assertFalse(MeshForegroundService.userStoppedForSession)
    }

    @Test
    fun bootActions_includePackageReplaced() {
        assertTrue(BootReceiver.isBootAction("android.intent.action.MY_PACKAGE_REPLACED"))
        assertFalse(BootReceiver.isBootAction("android.intent.action.AIRPLANE_MODE"))
    }

    @Test
    fun procExitFormat_hasReasonAndTimestamp() {
        val line = ProcessExitLog.format(10, "user\nrequested", 1234L, 400, 99, 0)
        assertEquals("[PROC-EXIT] reason=USER_REQUESTED status=0 importance=400 pid=99 ts=1234 desc=\"user requested\"", line)
        assertEquals("UNKNOWN(99)", ProcessExitLog.reasonName(99))
    }
}
