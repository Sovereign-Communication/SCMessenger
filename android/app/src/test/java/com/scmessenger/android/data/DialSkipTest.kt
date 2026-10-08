package com.scmessenger.android.data

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.api.IronCoreException

class DialSkipTest {
    @Test
    fun typedDialSkippedIsNeutral() {
        assertTrue(DialSkip.isSkipped(IronCoreException.DialSkipped("Dial skipped")))
    }

    @Test
    fun realFailuresAreNotSkips() {
        assertFalse(DialSkip.isSkipped(IronCoreException.NetworkException("Network error")))
        assertFalse(DialSkip.isSkipped(IronCoreException.DialSelf("Dial self")))
        // A message that merely looks like a skip is no longer treated as one.
        assertFalse(DialSkip.isSkipped(Exception("skipped: target is self")))
        assertFalse(DialSkip.isSkipped(Exception()))
    }
}
