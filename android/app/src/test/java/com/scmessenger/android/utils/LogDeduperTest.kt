package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class LogDeduperTest {
    private val debug = 3
    private val warn = 5

    @Test
    fun normalLinesAndMarkersAlwaysPass() {
        val d = LogDeduper()
        assertTrue(d.evaluate(debug, "something ordinary", 0).emit)
        assertTrue(d.evaluate(debug, "something ordinary", 1).emit)
        assertTrue(d.evaluate(debug, "[TRANSPORT] kind=ble state=available detail=x", 1).emit)
        assertNull(LogDeduper.keyFor("[TRANSPORT] kind=ble state=available detail=x"))
    }

    @Test
    fun spamLineIsRateLimitedAndSummarised() {
        val d = LogDeduper(minIntervalMs = 60_000, refreshIntervalMs = 300_000)
        val m = "Refreshed address snapshots: listeners=[a], external=[]"
        assertTrue(d.evaluate(debug, m, 0).emit)
        for (t in 1..50) assertFalse(d.evaluate(debug, m, t * 1000L).emit)
        assertFalse(d.evaluate(debug, m, 299_000).emit)
        val next = d.evaluate(debug, m, 300_000)
        assertTrue(next.emit)
        assertNotNull(next.summary)
        assertTrue(next.summary!!.contains("suppressed=51"))
    }

    @Test
    fun changedContentReemitsAfterMinInterval() {
        val d = LogDeduper(minIntervalMs = 60_000, refreshIntervalMs = 300_000)
        assertTrue(d.evaluate(debug, "Refreshed address snapshots: listeners=[a]", 0).emit)
        assertFalse(d.evaluate(debug, "Refreshed address snapshots: listeners=[b]", 10_000).emit)
        assertTrue(d.evaluate(debug, "Refreshed address snapshots: listeners=[b]", 61_000).emit)
    }

    @Test
    fun warningsAreNeverSuppressed() {
        val d = LogDeduper()
        val m = "Mesh Stats: 3 peers (Core), 1 full, 2 headless (Repo)"
        assertTrue(d.evaluate(debug, m, 0).emit)
        assertTrue(d.evaluate(warn, m, 1).emit)
    }

    @Test
    fun mdnsLinesAreBucketed() {
        val k = LogDeduper.keyFor("mDNS service resolved 192.168.1.5:9001")
        assertNotNull(k)
        assertEquals(k, LogDeduper.keyFor("mDNS service resolved 10.0.0.7:9002"))
    }

    @Test
    fun idleKeysAreEvictedByAgeNotByCount() {
        val d = LogDeduper(minIntervalMs = 60_000, refreshIntervalMs = 300_000)
        // Many distinct volatile-looking mDNS keys: nothing is capped while they are fresh.
        for (i in 0 until 200) d.evaluate(debug, "mDNS key${i}x event", 0)
        assertTrue(d.trackedKeyCount() > 64)
        // Past 2x refresh with no sightings, the next sweep drops them all.
        d.evaluate(debug, "Mesh Stats: peers=1", 700_000)
        assertEquals(1, d.trackedKeyCount())
    }

    @Test
    fun activeKeyIsNotEvictedAndStillRateLimited() {
        val d = LogDeduper(minIntervalMs = 60_000, refreshIntervalMs = 300_000)
        val m = "Mesh Stats: peers=1"
        assertTrue(d.evaluate(debug, m, 0).emit)
        for (t in 1..11) assertFalse(d.evaluate(debug, m, t * 25_000L).emit)
        assertEquals(1, d.trackedKeyCount())
    }

    @Test
    fun trimMemoryDropsAllAndIsIdempotent() {
        val d = LogDeduper()
        d.evaluate(debug, "Mesh Stats: peers=1", 0)
        d.trimMemory()
        d.trimMemory()
        assertEquals(0, d.trackedKeyCount())
        // Tracking resumes cleanly: first sighting emits again.
        assertTrue(d.evaluate(debug, "Mesh Stats: peers=1", 1).emit)
        assertEquals(1, d.trackedKeyCount())
    }
}
