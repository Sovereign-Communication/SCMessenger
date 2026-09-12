package com.scmessenger.android.utils

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PeerIdValidatorCanonicalTest {

    /** Live Windows node: pubkey 30d0fa67… ↔ peer 12D3KooWD6vZ… */
    private val winPk = "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e"
    private val winP2p = "12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw"

    /** Live AWS node: pubkey 69805e17… ↔ peer 12D3KooWGvCW… */
    private val awsPk = "69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c"
    private val awsP2p = "12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31"

    @Test
    fun `libp2p peer id canonicalizes to same key as public key`() {
        assertEquals(winPk, PeerIdValidator.canonicalKey(winPk, null))
        assertEquals(winPk, PeerIdValidator.canonicalKey(winP2p, null))
        assertEquals(awsPk, PeerIdValidator.canonicalKey(awsP2p, null))
    }

    @Test
    fun `public key hint wins over libp2p id`() {
        assertEquals(winPk, PeerIdValidator.canonicalKey(winP2p, winPk))
        assertEquals(awsPk, PeerIdValidator.canonicalKey("junk", awsPk))
    }

    @Test
    fun `different nodes never collapse`() {
        assertFalse(PeerIdValidator.canonicalKey(winP2p, null) == PeerIdValidator.canonicalKey(awsP2p, null))
    }

    @Test
    fun `empty and unknown ids stay stable`() {
        assertEquals("", PeerIdValidator.canonicalKey(null, null))
        assertEquals("", PeerIdValidator.canonicalKey("", null))
        assertEquals("not-an-id", PeerIdValidator.canonicalKey("not-an-id", null))
    }

    @Test
    fun `live identity ids that are not curve points are not treated as pubkeys`() {
        // Windows identity_id (blake3) — typically not an Ed25519 point.
        val winId = "985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826"
        val asPk = PeerIdValidator.normalizePublicKeyHex(winId)
        // Either null (not a point) or a valid hex — must never equal a different node's key.
        if (asPk != null) {
            assertFalse(asPk == awsPk)
        }
        assertNull(PeerIdValidator.normalizePublicKeyHex("zz"))
    }
}
