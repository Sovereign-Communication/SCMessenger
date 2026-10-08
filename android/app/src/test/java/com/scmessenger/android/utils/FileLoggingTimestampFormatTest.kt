package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.Instant
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import java.util.Locale

/** G8: mesh_diagnostics.log timestamp pattern is ISO-8601 with year and UTC offset. */
class FileLoggingTimestampFormatTest {
    private val pattern = "yyyy-MM-dd'T'HH:mm:ss.SSSXXX"

    @Test
    fun formatCarriesYearAndOffset() {
        val fmt = DateTimeFormatter.ofPattern(pattern, Locale.US).withZone(ZoneOffset.ofHours(2))
        val out = fmt.format(Instant.parse("2026-10-07T12:03:09.123Z"))
        assertEquals("2026-10-07T14:03:09.123+02:00", out)
    }

    @Test
    fun utcRendersAsZ() {
        val fmt = DateTimeFormatter.ofPattern(pattern, Locale.US).withZone(ZoneOffset.UTC)
        assertTrue(fmt.format(Instant.parse("2026-10-07T12:03:09.123Z")).endsWith("Z"))
    }
}
