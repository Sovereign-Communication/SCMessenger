package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Dual-nickname display-name resolver tests:
 * localNickname (user-set) is PRIMARY; nickname (peer's self-reported
 * identifier from scm.message.identity.v1) is SECONDARY.
 */
class ContactDisplayNameTest {

    private fun contact(
        localNickname: String? = null,
        nickname: String? = null,
        peerId: String = "12D3KooWTestPeer"
    ): uniffi.api.Contact {
        return uniffi.api.Contact(
            peerId = peerId,
            nickname = nickname,
            localNickname = localNickname,
            publicKey = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            addedAt = 1u,
            lastSeen = null,
            notes = null,
            lastKnownDeviceId = null,
            verifiedAt = null,
            isTombstone = false
        )
    }

    @Test
    fun `both present - primary with secondary in parentheses`() {
        val c = contact(localNickname = "Alice", nickname = "Claude-Windows-Driver")
        val names = c.displayNames()
        assertEquals("Alice", names.primary)
        assertEquals("Claude-Windows-Driver", names.secondary)
        assertEquals("Alice (Claude-Windows-Driver)", c.displayName("abcd1234..."))
    }

    @Test
    fun `only localNickname - rendered alone`() {
        val c = contact(localNickname = "Alice", nickname = null)
        assertEquals("Alice", c.displayName("abcd1234..."))
        assertNull(c.displayNames().secondary)
    }

    @Test
    fun `only chosen nickname - rendered alone`() {
        // Auto-added peer whose chosen nickname arrives via identity envelope.
        val c = contact(localNickname = null, nickname = "Claude-Windows-Driver")
        assertEquals("Claude-Windows-Driver", c.displayName("abcd1234..."))
        assertNull(c.displayNames().primary)
    }

    @Test
    fun `neither - falls back to truncated id`() {
        val c = contact(peerId = "12D3KooWAbCdEfGh")
        assertEquals("abcd1234...", c.displayName("abcd1234..."))
    }

    @Test
    fun `identical nicknames - secondary suppressed`() {
        val c = contact(localNickname = "Alice", nickname = "Alice")
        assertEquals("Alice", c.displayName("abcd1234..."))
        assertNull(c.displayNames().secondary)
    }

    @Test
    fun `synthetic peer- fallback nickname treated as blank`() {
        val c = contact(localNickname = null, nickname = "peer-a1b2c3d4")
        assertNull(c.displayNames().secondary)
        assertEquals("abcd1234...", c.displayName("abcd1234..."))
    }

    @Test
    fun `localNickname beats synthetic fallback`() {
        val c = contact(localNickname = "Alice", nickname = "peer-a1b2c3d4")
        assertEquals("Alice", c.displayName("abcd1234..."))
        assertNull(c.displayNames().secondary)
    }

    @Test
    fun `whitespace-only values treated as blank`() {
        val c = contact(localNickname = "   ", nickname = "  ")
        assertNull(c.displayNames().primary)
        assertNull(c.displayNames().secondary)
        assertEquals("abcd1234...", c.displayName("abcd1234..."))
    }

    @Test
    fun `isSyntheticFallbackNickname matches only peer- prefix`() {
        assertTrue(isSyntheticFallbackNickname("peer-12345678"))
        assertTrue(isSyntheticFallbackNickname("PEER-12345678"))
        assertTrue(isSyntheticFallbackNickname("  peer-x "))
        assertFalse(isSyntheticFallbackNickname("Claude-Windows-Driver"))
        assertFalse(isSyntheticFallbackNickname(null))
        assertFalse(isSyntheticFallbackNickname(""))
    }
}
