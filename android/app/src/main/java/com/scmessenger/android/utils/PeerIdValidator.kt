package com.scmessenger.android.utils

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

    fun isIdentityId(id: String): Boolean =
        id.matches(IDENTITY_ID_REGEX)

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

    /**
     * Rough Ed25519 curve-point check matching Rust `is_valid_public_key`
     * (rejects most blake3 identity_ids that are 64-hex but not keys).
     * Uses BigInteger decompression of the compressed Edwards y-coordinate.
     */
    fun isValidEd25519Point(hex: String): Boolean {
        return try {
            val bytes = hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray()
            if (bytes.size != 32) return false
            // Sign bit is high bit of last byte; y is little-endian 255-bit.
            val y = java.math.BigInteger(1, bytes.reversedArray().let { arr ->
                val copy = arr.copyOf()
                copy[31] = (copy[31].toInt() and 0x7f).toByte()
                copy
            })
            val p = java.math.BigInteger("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffed", 16)
            val d = java.math.BigInteger("52036cee2b6ffe738cc740797779e89800700a4d4141d8ab75eb4dca135978a3", 16)
            val one = java.math.BigInteger.ONE
            val two = java.math.BigInteger.valueOf(2)
            val zero = java.math.BigInteger.ZERO
            val yy = y.multiply(y).mod(p)
            val u = yy.subtract(one).mod(p)
            val v = one.add(d.multiply(yy)).mod(p)
            // Edwards: x^2 = (y^2-1)/(1+d y^2)
            val denomInv = v.modInverse(p)
            val x2 = u.multiply(denomInv).mod(p)
            val legendre = x2.modPow(p.subtract(one).divide(two), p)
            legendre == zero || legendre == one
        } catch (_: Exception) {
            false
        }
    }
}
