package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * SELF-CERTIFYING KEY BINDING support: peer id format validation must accept
 * ONLY strict Base58BTC so malformed ids (non-ASCII lookalikes, '+', '/',
 * Base64 residue) can never masquerade as libp2p transport identities.
 */
class PeerIdValidatorTest {

    private val ed25519StylePeerId = "12D3KooWEfZ2fJ8AcGvVfEUi2wFQPo6z8kZVr5TsgP7JQF2B9kS1"
    private val rsaStylePeerId = "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N"

    @Test
    fun `accepts well formed ed25519 identity multihash peer ids`() {
        assertTrue(PeerIdValidator.isLibp2pPeerId(ed25519StylePeerId))
    }

    @Test
    fun `accepts well formed sha256 multihash peer ids`() {
        assertTrue(PeerIdValidator.isLibp2pPeerId(rsaStylePeerId))
    }

    @Test
    fun `rejects base64 plus slash characters`() {
        // '+' and '/' are valid Base64 but never valid Base58BTC.
        assertFalse(PeerIdValidator.isLibp2pPeerId(ed25519StylePeerId.dropLast(1) + "+"))
        assertFalse(PeerIdValidator.isLibp2pPeerId(ed25519StylePeerId.dropLast(1) + "/"))
    }

    @Test
    fun `rejects base58 forbidden lookalike characters`() {
        for (banned in listOf('0', 'O', 'I', 'l')) {
            val mutated = StringBuilder(ed25519StylePeerId).also { it.setCharAt(10, banned) }.toString()
            assertFalse("must reject '$banned'", PeerIdValidator.isLibp2pPeerId(mutated))
        }
    }

    @Test
    fun `rejects non-ascii letter lookalikes`() {
        // 'О' (Cyrillic O) passes Character.isLetterOrDigit but is not Base58BTC.
        val cyrillic = StringBuilder(ed25519StylePeerId).also { it.setCharAt(10, 'О') }.toString()
        assertFalse(PeerIdValidator.isLibp2pPeerId(cyrillic))
        assertFalse(PeerIdValidator.isLibp2pPeerId(ed25519StylePeerId + "é"))
    }

    @Test
    fun `rejects symbols punctuation and whitespace`() {
        assertFalse(PeerIdValidator.isLibp2pPeerId(ed25519StylePeerId.dropLast(1) + "-"))
        assertFalse(PeerIdValidator.isLibp2pPeerId(ed25519StylePeerId.dropLast(1) + "_"))
        assertFalse(PeerIdValidator.isLibp2pPeerId("$ed25519StylePeerId "))
        assertFalse(PeerIdValidator.isLibp2pPeerId(" $ed25519StylePeerId"))
    }

    @Test
    fun `rejects wrong lengths within prefix families`() {
        // Below the minimum length for each prefix family
        assertFalse(PeerIdValidator.isLibp2pPeerId(ed25519StylePeerId.take(45)))
        assertFalse(PeerIdValidator.isLibp2pPeerId(rsaStylePeerId.take(43)))
        // Above the maximum length for each prefix family
        assertFalse(PeerIdValidator.isLibp2pPeerId(ed25519StylePeerId + "111111"))
        assertFalse(PeerIdValidator.isLibp2pPeerId(rsaStylePeerId + "11111"))
    }

    @Test
    fun `normalize lowercases hex ids and preserves base58 case`() {
        val hex = "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789"
        assertEquals(hex.lowercase(), PeerIdValidator.normalize(hex))
        assertEquals(ed25519StylePeerId, PeerIdValidator.normalize(ed25519StylePeerId))
    }

    @Test
    fun `isSame is case-insensitive only for hex ids`() {
        assertTrue(PeerIdValidator.isSame(hexUpper(), hexLower()))
        assertFalse(
            PeerIdValidator.isSame(
                ed25519StylePeerId,
                ed25519StylePeerId.lowercase()
            )
        )
    }

    private fun hexUpper() = "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789"
    private fun hexLower() = hexUpper().lowercase()
}
