package com.scmessenger.android.utils

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * UNIFICATION P1 (P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION): pins the single
 * authoritative Kotlin Ed25519 curve-point check in [PeerIdValidator] to the
 * same acceptance boundary as Rust `is_valid_public_key`
 * (`ed25519_dalek::VerifyingKey::from_bytes`, strict RFC 8032 decoding).
 *
 * Vectors:
 *  - RFC 8032 test vectors 1-3 (canonical Ed25519 public keys) -> MUST be true.
 *  - Structured sign-bit and field-range cases derived from the curve equation
 *    (y=1, y=p-1 with sign bit 0/1; y >= p; non-hex; wrong length) -> MUST match
 *    dalek exactly, including the x=0 non-canonical sign-bit rejection.
 *
 * These run on the JVM without the native library, which is why the pure-Kotlin
 * fallback authority exists; when the UniFFI path is wired on-device the same
 * vectors must pass against the core implementation.
 */
class PeerIdValidatorCurveVectorTest {

    // RFC 8032 section 7.1 public keys (valid Ed25519 points).
    private val rfc8032Pub1 = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
    private val rfc8032Pub2 = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c"
    private val rfc8032Pub3 = "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025"

    @Test
    fun `rfc8032 public keys are valid curve points`() {
        assertTrue(PeerIdValidator.isValidEd25519Point(rfc8032Pub1))
        assertTrue(PeerIdValidator.isValidEd25519Point(rfc8032Pub2))
        assertTrue(PeerIdValidator.isValidEd25519Point(rfc8032Pub3))
    }

    @Test
    fun `y equals one is a valid point only with sign bit zero`() {
        // y = 1 => x^2 = 0 => canonical encoding requires sign bit 0.
        val yOneSign0 = "01" + "00".repeat(31)
        // In little-endian Ed25519 (RFC 8032), sign bit is the most significant bit of the 32nd octet (byte 31).
        val yOneSign1 = "01" + "00".repeat(30) + "80"
        assertTrue(PeerIdValidator.isValidEd25519Point(yOneSign0))
        assertFalse(PeerIdValidator.isValidEd25519Point(yOneSign1))
    }

    @Test
    fun `y equals p minus one is a valid point only with sign bit zero`() {
        // y = p-1 => y^2 = 1 => x^2 = 0; sign bit decides canonical validity.
        // p-1 little-endian is ec followed by 30 0xff bytes and byte 31 = 0x7f (sign 0) or 0xff (sign 1).
        val pMinus1Sign0 = "ec" + "ff".repeat(30) + "7f"
        val pMinus1Sign1 = "ec" + "ff".repeat(30) + "ff"
        assertTrue(pMinus1Sign0.length == 64)
        assertTrue(pMinus1Sign1.length == 64)
        assertTrue(PeerIdValidator.isValidEd25519Point(pMinus1Sign0))
        // Top byte ff = 0x7f | 0x80: same y, non-canonical sign bit.
        val nonCanonical = pMinus1Sign0.dropLast(2) + "ff"
        assertFalse(PeerIdValidator.isValidEd25519Point(nonCanonical))
        assertFalse(PeerIdValidator.isValidEd25519Point(pMinus1Sign1))
    }

    @Test
    fun `y values at or above p are rejected`() {
        // y = p (little-endian of 7fff...ffed encoded with sign bit): ed ff*30, ff
        val yEqualsP = "ed" + "ff".repeat(30) + "ff"
        assertFalse(PeerIdValidator.isValidEd25519Point(yEqualsP))
        // All-ff bytes encode y >> p regardless of interpretation.
        assertFalse(PeerIdValidator.isValidEd25519Point("ff".repeat(32)))
    }

    @Test
    fun `malformed inputs are rejected`() {
        assertFalse(PeerIdValidator.isValidEd25519Point(""))
        assertFalse(PeerIdValidator.isValidEd25519Point("zz"))
        assertFalse(PeerIdValidator.isValidEd25519Point("30d0fa67"))
        assertFalse(PeerIdValidator.isValidEd25519Point(rfc8032Pub1.dropLast(2)))
        assertFalse(PeerIdValidator.isValidEd25519Point(rfc8032Pub1 + "aa"))
        assertFalse(
            PeerIdValidator.isValidEd25519Point(
                rfc8032Pub1.dropLast(1) + "g"
            )
        )
    }

    @Test
    fun `identity triad boundary stays consistent with canonical tests`() {
        // A valid point normalizes through normalizePublicKeyHex; a non-point
        // 64-hex (identity_id) does not. RFC pub1 must normalize to itself.
        assertTrue(
            PeerIdValidator.normalizePublicKeyHex(rfc8032Pub1) == rfc8032Pub1
        )
    }
}
