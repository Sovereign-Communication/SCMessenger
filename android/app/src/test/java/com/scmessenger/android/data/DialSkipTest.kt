package com.scmessenger.android.data

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class DialSkipTest {
    @Test
    fun skippedPrefixIsNeutral() {
        assertTrue(DialSkip.isSkipped(Exception("skipped: host already connected -- respond over existing link")))
        assertTrue(DialSkip.isSkipped(RuntimeException("  Skipped: target is self (local peer id)")))
    }

    @Test
    fun realFailuresAreNotSkips() {
        assertFalse(DialSkip.isSkipped(Exception("Network error")))
        assertFalse(DialSkip.isSkipped(Exception("marked as dead, skipped: later")))
        assertFalse(DialSkip.isSkipped(Exception()))
        assertFalse(DialSkip.isSkippedMessage(null))
    }
}
