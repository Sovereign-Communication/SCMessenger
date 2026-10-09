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
}
