package com.scmessenger.android.utils

/**
 * Sovereign Identity Triad validator and normalizer.
 *
 * SCMessenger enforces a strict distinction between the three identity hashes/identifiers:
 * 1. Identity Hash (identity_id): Blake3(ed25519_pubkey) (64 hex characters, lowercase).
 *    - The sovereign identifier for contacts, chats, outbox/inbox routing, and UI presentation.
 * 2. Public Key (public_key): Raw Ed25519 signing key (64 hex characters, valid Edwards curve point).
 *    - For cryptographic signatures and X25519 ECDH key agreement only. Never a transport address.
 * 3. Peer ID (libp2p_peer_id / ble_peer_id): Libp2p multi-hash (Base58 string starting with 12D3Koo
 *    or Qm) or BLE UUID.
 *    - Ephemeral socket transport routing ONLY. Never a contact, and never in nearby contacts list.
 *
 * The Ed25519 curve-point check in this object is the single authoritative Kotlin
 * implementation (UNIFICATION P1); platform copies were consolidated here.
 */
object PeerIdValidator {
    private val IDENTITY_ID_REGEX = Regex("^[a-fA-F0-9]{64}$")

    // Strict Base58BTC alphabet (Bitcoin order). `Character.isLetterOrDigit`
    // accepts non-ASCII letters and would let malformed ids through.
    private const val BASE58_ALPHABET =
        "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

    fun validate(id: String): Boolean =
        normalize(id).let { normalized ->
            normalized.matches(IDENTITY_ID_REGEX) || isLibp2pPeerId(normalized)
        }

    fun normalize(id: String): String {
        val trimmed = id.trim()
        // 64 hex chars are case-insensitive public keys - normalize to lower
        if (trimmed.length == 64 && trimmed.matches(IDENTITY_ID_REGEX)) {
            return trimmed.lowercase()
        }
        // Base58 libp2p IDs (starting with 12D3Koo or Qm) are case-sensitive - preserve case
        return trimmed
    }

    fun isLibp2pPeerId(id: String): Boolean {
        // Base58-encoded libp2p peer IDs: 12D3KooW... (~52 chars) or Qm... (~46 chars)
        // Validate prefix + reasonable length + STRICT Base58BTC ASCII charset
        // (no 0, O, I, l; no '+', '/', no non-ASCII lookalikes)
        val base58Chars = id.all { it in BASE58_ALPHABET }
        return base58Chars && (
            (id.startsWith("12D3Koo") && id.length in 46..56) ||
            (id.startsWith("Qm") && id.length in 44..50)
        )
    }

    fun isBlePeerId(id: String?): Boolean {
        val trimmed = id?.trim().orEmpty()
        if (trimmed.isEmpty()) return false
        return runCatching { java.util.UUID.fromString(trimmed) }.isSuccess
    }

    /**
     * Returns true if the identifier is a transport-only address (libp2p Peer ID or BLE UUID).
     * Transport addresses must NEVER be used as sovereign contact identifiers.
     */
    fun isTransportPeerId(id: String?): Boolean {
        val trimmed = id?.trim().orEmpty()
        if (trimmed.isEmpty()) return false
        return isLibp2pPeerId(trimmed) || isBlePeerId(trimmed)
    }

    fun isIdentityId(id: String): Boolean =
        id.matches(IDENTITY_ID_REGEX)

    fun isIdentityHash(id: String?): Boolean =
        id?.trim()?.matches(IDENTITY_ID_REGEX) == true

    fun isPublicKeyHex(id: String?): Boolean =
        normalizePublicKeyHex(id) != null

    fun isSame(id1: String, id2: String): Boolean =
        normalize(id1) == normalize(id2)

    /**
     * Canonical identity key for UI list dedup: same node under PeerID and
     * public_key must collapse to one entry.
     *
     * - 64-hex Ed25519 public keys → lowercase hex (contact-canonical form)
     * - libp2p PeerIDs with embedded Ed25519 → extracted public key hex
     * - anything else (Qm…, identity_id that is not a curve point) → normalize()
     */
    fun canonicalKey(id: String?, publicKeyHint: String? = null): String {
        normalizePublicKeyHex(publicKeyHint)?.let { return it }
        val trimmed = id?.trim().orEmpty()
        if (trimmed.isEmpty()) return ""
        normalizePublicKeyHex(trimmed)?.let { return it }
        if (isLibp2pPeerId(trimmed)) {
            PeerKeyUtils.extractPublicKeyFromPeerId(trimmed)?.let { extracted ->
                normalizePublicKeyHex(extracted)?.let { return it }
            }
        }
        return normalize(trimmed)
    }

    /** Lowercase 64-hex only when it is a valid Ed25519 curve point. */
    fun normalizePublicKeyHex(value: String?): String? {
        val trimmed = value?.trim() ?: return null
        if (trimmed.length != 64) return null
        if (!trimmed.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) return null
        if (!isValidEd25519Point(trimmed)) return null
        return trimmed.lowercase()
    }

    // UNIFICATION P1 (bod-dd336324 / P1_ANDROID_UNIFFI_CURVE_CHECK_RELOCATION):
    // Single authoritative Kotlin implementation of Rust `is_valid_public_key`
    // (core/src/identity/keys.rs). Consolidates the three previously divergent
    // copies (this file, DashboardViewModel, ContactsViewModel). All curve
    // constants are derived lazily from p — no hand-typed literals beyond p.
    private val P: java.math.BigInteger =
        java.math.BigInteger("57896044618658097711785492504343953926634992332820282019728792003956564819949")
    private val D: java.math.BigInteger by lazy {
        val inv121666 = java.math.BigInteger.valueOf(121666).modInverse(P)
        java.math.BigInteger.valueOf(121665).negate().mod(P).multiply(inv121666).mod(P)
    }
    private val SQRT_M1: java.math.BigInteger by lazy {
        java.math.BigInteger.valueOf(2).modPow(P.subtract(java.math.BigInteger.ONE).divide(java.math.BigInteger.valueOf(4)), P)
    }
    private val P_PLUS3_OVER8: java.math.BigInteger by lazy {
        P.add(java.math.BigInteger.valueOf(3)).divide(java.math.BigInteger.valueOf(8))
    }

    /**
     * Ed25519 curve-point check — the Kotlin mirror of Rust `is_valid_public_key`
     * (`ed25519_dalek::VerifyingKey::from_bytes`, i.e. strict RFC 8032 decoding):
     * field-range check on y, full square-root recovery of x (no Legendre
     * shortcut), and the non-canonical x=0/sign-bit rejection.
     *
     * JVM unit tests run without the native library, so this pure-Kotlin copy
     * is the single fallback authority; its byte-equivalence to core is pinned
     * by dalek-derived test vectors in PeerIdValidatorCurveVectorTest.
     * Relocation of call sites behind the UniFFI binding (isValidPublicKeyHexViaCore)
     * is staged for a follow-up after the binding-init audit (see ticket).
     */
    fun isValidEd25519Point(hex: String): Boolean {
        if (hex.length != 64) return false
        val bytes = ByteArray(32)
        for (i in 0 until 32) {
            val hi = Character.digit(hex[i * 2], 16)
            val lo = Character.digit(hex[i * 2 + 1], 16)
            if (hi == -1 || lo == -1) return false
            bytes[i] = ((hi shl 4) or lo).toByte()
        }
        return try {
            val signBit = (bytes[31].toInt() and 0x80) != 0
            val yBytes = bytes.copyOf()
            yBytes[31] = (yBytes[31].toInt() and 0x7F).toByte()
            // y is little-endian; BigInteger wants big-endian.
            val y = java.math.BigInteger(1, yBytes.reversedArray())
            if (y >= P) return false
            val yy = y.multiply(y).mod(P)
            val u = yy.subtract(java.math.BigInteger.ONE).mod(P)
            val v = D.multiply(yy).add(java.math.BigInteger.ONE).mod(P)
            if (v == java.math.BigInteger.ZERO) return false
            val vInv = v.modInverse(P)
            val x2 = u.multiply(vInv).mod(P)
            // x = 0 encodes canonically only with sign bit 0 (RFC 8032).
            if (x2 == java.math.BigInteger.ZERO) return !signBit
            // sqrt via (p+3)/8 (p = 5 mod 8), corrected by sqrt(-1) when needed.
            var x = x2.modPow(P_PLUS3_OVER8, P)
            if (x.multiply(x).mod(P) != x2) {
                x = x.multiply(SQRT_M1).mod(P)
                if (x.multiply(x).mod(P) != x2) return false
            }
            true
        } catch (_: ArithmeticException) {
            false
        } catch (_: Exception) {
            false
        }
    }
}
