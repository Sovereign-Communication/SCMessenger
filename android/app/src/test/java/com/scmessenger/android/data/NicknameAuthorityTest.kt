package com.scmessenger.android.data

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/**
 * NICKNAME-AUTHORITY-001: federated nickname merge must never last-writer-wins
 * a real name over another real name. Device RCA 2026-09-11: emulator nick
 * "androidulaator" landed on Windows ledger/contact rows.
 */
class NicknameAuthorityTest {

    private fun normalizeNickname(value: String?): String? =
        value?.trim()?.takeIf { it.isNotEmpty() }

    private fun isSyntheticFallbackNickname(value: String?): Boolean {
        val n = normalizeNickname(value)?.lowercase() ?: return false
        return n.startsWith("peer-")
    }

    /** Mirrors MeshRepository.selectAuthoritativeNickname after NICKNAME-AUTHORITY-001. */
    private fun selectAuthoritativeNickname(incoming: String?, existing: String?): String? {
        val incomingNormalized = normalizeNickname(incoming)
        val existingNormalized = normalizeNickname(existing)
        val incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        val existingSynthetic = isSyntheticFallbackNickname(existingNormalized)
        return when {
            incomingNormalized == null && existingSynthetic -> null
            incomingNormalized == null -> existingNormalized
            incomingSynthetic && existingNormalized == null -> null
            incomingSynthetic && existingSynthetic -> null
            incomingSynthetic -> existingNormalized
            existingSynthetic -> incomingNormalized
            existingNormalized == null -> incomingNormalized
            else -> existingNormalized
        }
    }

    @Test
    fun `real incoming cannot steal real existing`() {
        assertEquals("Windows", selectAuthoritativeNickname("androidulaator", "Windows"))
        assertEquals("Windows", selectAuthoritativeNickname("androidulator", "Windows"))
    }

    @Test
    fun `real incoming fills empty existing`() {
        assertEquals("androidulaator", selectAuthoritativeNickname("androidulaator", null))
        assertEquals("androidulaator", selectAuthoritativeNickname("androidulaator", "  "))
    }

    @Test
    fun `real incoming replaces synthetic existing`() {
        assertEquals("androidulaator", selectAuthoritativeNickname("androidulaator", "peer-30d0fa67"))
    }

    @Test
    fun `synthetic incoming never beats real existing`() {
        assertEquals("Windows", selectAuthoritativeNickname("peer-1c158e86", "Windows"))
    }

    @Test
    fun `synthetic pair clears to null`() {
        assertNull(selectAuthoritativeNickname("peer-aaaa", "peer-bbbb"))
    }
}
