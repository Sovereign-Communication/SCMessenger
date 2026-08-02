package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class ContactImportParserTest {

    @Test
    fun `parses iOS identity QR contract with peer id and routing fields`() {
        val result = parseContactImportPayload(
            """
            {
              "version": "1.0",
              "peer_id": "12D3KooW-ios-peer",
              "public_key": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
              "device_id": "ios-device",
              "identity_id": "identity-ios",
              "nickname": "iPhone",
              "libp2p_peer_id": "12D3KooW-ios-peer",
              "listeners": ["/ip4/192.168.1.50/tcp/9123"],
              "connection_hints": ["/ip4/192.168.1.50/tcp/9123"]
            }
            """.trimIndent()
        )

        assertTrue(result is ContactImportParseResult.Valid)
        val payload = (result as ContactImportParseResult.Valid).payload
        assertEquals("12D3KooW-ios-peer", payload.peerId)
        assertEquals(
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            payload.publicKey
        )
        assertEquals("12D3KooW-ios-peer", payload.libp2pPeerId)
        assertEquals(listOf("/ip4/192.168.1.50/tcp/9123"), payload.listeners)
    }

    @Test
    fun `does not invent a listener when identity QR omits routing hints`() {
        val result = parseContactImportPayload(
            """
            {
              "peer_id": "12D3KooW-ios-peer",
              "public_key": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
              "libp2p_peer_id": "12D3KooW-ios-peer"
            }
            """.trimIndent()
        )

        assertTrue(result is ContactImportParseResult.Valid)
        assertTrue((result as ContactImportParseResult.Valid).payload.listeners.isEmpty())
    }

    @Test
    fun `uses peer id as the transport id when legacy alias is absent`() {
        val result = parseContactImportPayload(
            """
            {
              "peer_id": "12D3KooW-ios-peer",
              "public_key": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
              "connection_hints": ["/ip4/192.168.1.50/tcp/9123"]
            }
            """.trimIndent()
        )

        assertTrue(result is ContactImportParseResult.Valid)
        val payload = (result as ContactImportParseResult.Valid).payload
        assertEquals(payload.peerId, payload.libp2pPeerId)
        assertEquals(listOf("/ip4/192.168.1.50/tcp/9123"), payload.listeners)
    }

    @Test
    fun `does not treat identity hash as a routable peer id`() {
        val result = parseContactImportPayload(
            """
            {
              "identity_id": "identity-only",
              "public_key": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
            }
            """.trimIndent()
        )

        assertTrue(result is ContactImportParseResult.Invalid)
    }

    @Test
    fun `listener list capped at six entries to match deep-link path`() {
        val nineListeners = (1..9).map { "/ip4/10.0.0.$it/tcp/9123" }
        val json = buildString {
            append("""{"peer_id":"p1","public_key":"aa","listeners":[""")
            append(nineListeners.joinToString(",") { "\"$it\"" })
            append("]}")
        }

        val result = parseContactImportPayload(json)

        assertTrue(result is ContactImportParseResult.Valid)
        val listeners = (result as ContactImportParseResult.Valid).payload.listeners
        assertEquals(6, listeners.size)
        assertEquals(nineListeners.take(6), listeners)
    }

    @Test
    fun `over-long listener entry dropped silently`() {
        val longEntry = "/ip4/10.0.0.1/tcp/9123" + "x".repeat(300)
        val shortEntry = "/ip4/10.0.0.2/tcp/9123"
        val json = """
            {
              "peer_id": "p1",
              "public_key": "aa",
              "listeners": ["$longEntry", "$shortEntry"]
            }
        """.trimIndent()

        val result = parseContactImportPayload(json)

        assertTrue(result is ContactImportParseResult.Valid)
        val listeners = (result as ContactImportParseResult.Valid).payload.listeners
        assertEquals(listOf(shortEntry), listeners)
    }

    @Test
    fun `normal listener list under cap is preserved intact`() {
        val listeners = listOf("/ip4/10.0.0.1/tcp/9123", "/ip4/10.0.0.2/tcp/9123")
        val json = """
            {
              "peer_id": "p1",
              "public_key": "aa",
              "listeners": ["/ip4/10.0.0.1/tcp/9123", "/ip4/10.0.0.2/tcp/9123"]
            }
        """.trimIndent()

        val result = parseContactImportPayload(json)

        assertTrue(result is ContactImportParseResult.Valid)
        assertEquals(listeners, (result as ContactImportParseResult.Valid).payload.listeners)
    }
}
