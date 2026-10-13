package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File
import java.util.concurrent.LinkedBlockingQueue

class LogFileSupportTest {
    @get:Rule
    val tmp = TemporaryFolder()

    @Test
    fun fullQueueDropsOldestAndCountsEveryLoss() {
        val ledger = LogLossLedger()
        val q = LinkedBlockingQueue<String>(2)
        assertTrue(offerDropOldest(q, "a", ledger))
        assertTrue(offerDropOldest(q, "b", ledger))
        assertNull("nothing lost yet", ledger.takeMarker())

        assertTrue(offerDropOldest(q, "c", ledger))
        assertTrue(offerDropOldest(q, "d", ledger))
        assertEquals(listOf("c", "d"), q.toList())
        assertEquals("[LOG-LOSS] queue_dropped=2 write_failed=0", ledger.takeMarker())
        assertNull("marker is taken once", ledger.takeMarker())
    }

    @Test
    fun rejectedNewLineIsCountedAsLoss() {
        val ledger = LogLossLedger()
        // offer() always refuses (another producer won the freed slot): the
        // evicted entry and the new line are both losses.
        val q = object : LinkedBlockingQueue<String>(1) {
            override fun offer(e: String): Boolean = false
        }
        q.put("x")
        assertFalse(offerDropOldest(q, "y", ledger))
        assertEquals("[LOG-LOSS] queue_dropped=2 write_failed=0", ledger.takeMarker())
    }

    @Test
    fun writeFailuresAreReportedByTheNextMarker() {
        val ledger = LogLossLedger()
        ledger.recordWriteFailure()
        ledger.recordWriteFailure()
        ledger.recordQueueDrop()
        assertEquals("[LOG-LOSS] queue_dropped=1 write_failed=2", ledger.takeMarker())
    }

    @Test
    fun rotationShiftsHistoryAndEmptiesCurrentFile() {
        val log = tmp.newFile("mesh.log")
        log.writeText("current")
        File(tmp.root, "mesh.log.1").writeText("one")
        File(tmp.root, "mesh.log.2").writeText("two")
        val problems = mutableListOf<String>()

        assertTrue(rotateLogFiles(log, 4, problems::add))
        assertTrue(problems.isEmpty())
        assertFalse(log.exists())
        assertEquals("current", File(tmp.root, "mesh.log.1").readText())
        assertEquals("one", File(tmp.root, "mesh.log.2").readText())
        assertEquals("two", File(tmp.root, "mesh.log.3").readText())
    }

    @Test
    fun rotationStepThatFailsIsReportedNotThrown() {
        val log = tmp.newFile("mesh.log")
        log.writeText("current")
        File(tmp.root, "mesh.log.1").writeText("one")
        // A non-empty directory squats on the .2 slot: it can be neither deleted nor replaced.
        val squatter = tmp.newFolder("mesh.log.2")
        File(squatter, "child").writeText("x")
        val problems = mutableListOf<String>()

        rotateLogFiles(log, 4, problems::add)

        assertTrue(problems.any { it.contains("could not delete mesh.log.2") })
    }
}
