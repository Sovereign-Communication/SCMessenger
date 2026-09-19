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

    @Test
    fun `isBlePeerId correctly identifies UUID format`() {
        val validUuid = java.util.UUID.randomUUID().toString()
        assertTrue(PeerIdValidator.isBlePeerId(validUuid))
        assertFalse(PeerIdValidator.isBlePeerId(ed25519StylePeerId))
        assertFalse(PeerIdValidator.isBlePeerId(hexLower()))
        assertFalse(PeerIdValidator.isBlePeerId(null))
        assertFalse(PeerIdValidator.isBlePeerId(""))
    }

    @Test
    fun `isTransportPeerId returns true for libp2p and ble but false for sovereign hashes`() {
        val validUuid = java.util.UUID.randomUUID().toString()
        // Transport addresses
        assertTrue("libp2p ed25519 peer ID is transport", PeerIdValidator.isTransportPeerId(ed25519StylePeerId))
        assertTrue("libp2p rsa peer ID is transport", PeerIdValidator.isTransportPeerId(rsaStylePeerId))
        assertTrue("BLE UUID peer ID is transport", PeerIdValidator.isTransportPeerId(validUuid))

        // Sovereign hashes / keys must NOT be identified as transport IDs
        val valid64Hex = "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e"
        assertFalse("64-hex public key is not transport", PeerIdValidator.isTransportPeerId(valid64Hex))

        val blake3IdentityId = "985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826"
        assertFalse("64-hex identity hash is not transport", PeerIdValidator.isTransportPeerId(blake3IdentityId))

        assertFalse("empty id is not transport", PeerIdValidator.isTransportPeerId(""))
        assertFalse("null id is not transport", PeerIdValidator.isTransportPeerId(null))
    }

    @Test
    fun `identity triad strictly categorizes identity hash, public key, and peer id`() {
        val pubKey = "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e"
        val identityHash = "985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826"
        val peerId = ed25519StylePeerId

        // 1. Identity Hash (Blake3): 64 hex chars, used for sovereign identity
        assertTrue(PeerIdValidator.isIdentityId(identityHash))
        assertTrue(PeerIdValidator.isIdentityHash(identityHash))
        assertFalse(PeerIdValidator.isTransportPeerId(identityHash))

        // 2. Public Key: 64 hex chars, valid Edwards curve point
        assertTrue(PeerIdValidator.isIdentityId(pubKey))
        assertTrue(PeerIdValidator.isPublicKeyHex(pubKey))
        assertFalse(PeerIdValidator.isTransportPeerId(pubKey))

        // 3. Peer ID: Libp2p Base58 multihash, transport only
        assertTrue(PeerIdValidator.isLibp2pPeerId(peerId))
        assertTrue(PeerIdValidator.isTransportPeerId(peerId))
        assertFalse(PeerIdValidator.isIdentityId(peerId))
        assertFalse(PeerIdValidator.isIdentityHash(peerId))
        assertFalse(PeerIdValidator.isPublicKeyHex(peerId))
    }

    private fun hexUpper() = "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789"
    private fun hexLower() = hexUpper().lowercase()
}
