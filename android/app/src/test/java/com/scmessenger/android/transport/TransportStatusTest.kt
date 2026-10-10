package com.scmessenger.android.transport

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class TransportStatusTest {
    @Test
    fun formatMatchesRustGrammar() {
        assertEquals(
            "[TRANSPORT] kind=ble state=unavailable detail=no_adapter",
            TransportStatus.format("ble", "unavailable", null, "no adapter"),
        )
        assertEquals(
            "[TRANSPORT] kind=quic state=connected peers=3 detail=periodic",
            TransportStatus.format("quic", "connected", 3, "periodic"),
        )
    }

    @Test
    fun unknownVocabularyAndInjectionAreNeutralised() {
        val line = TransportStatus.format("evil\nkind", "up", null, "x=1\ny z")
        assertFalse(line.contains('\n'))
        assertTrue(line.contains("kind=invalid state=error"))
        assertEquals(3, line.count { it == '=' })
        assertTrue(TransportStatus.sanitize("a".repeat(500)).length <= 128)
    }

    @Test
    fun classifiesMultiaddrs() {
        assertEquals("tcp4", TransportStatus.classifyMultiaddr("/ip4/10.0.0.2/tcp/9001"))
        assertEquals("tcp6", TransportStatus.classifyMultiaddr("/ip6/fe80::1/tcp/9001"))
        assertEquals("quic", TransportStatus.classifyMultiaddr("/ip4/10.0.0.2/udp/9001/quic-v1"))
        assertEquals("circuit", TransportStatus.classifyMultiaddr("/ip4/1.2.3.4/tcp/9001/p2p/X/p2p-circuit"))
        assertNull(TransportStatus.classifyMultiaddr("/p2p/X"))
    }
}
