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

    /**
     * Rough Ed25519 curve-point check matching Rust `is_valid_public_key`
     * (rejects most blake3 identity_ids that are 64-hex but not keys).
     * Uses BigInteger decompression of the compressed Edwards y-coordinate.
     */
    fun isValidEd25519Point(hex: String): Boolean {
        return try {
            if (hex.length != 64) return false
            val bytes = ByteArray(32)
            for (i in 0 until 32) {
                val hi = Character.digit(hex[i * 2], 16)
                val lo = Character.digit(hex[i * 2 + 1], 16)
                if (hi == -1 || lo == -1) return false
                bytes[i] = ((hi shl 4) or lo).toByte()
            }
            val yBytes = bytes.clone()
            val signBit = (yBytes[31].toInt() and 0x80) != 0
            yBytes[31] = (yBytes[31].toInt() and 0x7f).toByte()
            val p = java.math.BigInteger("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffed", 16)
            val d = java.math.BigInteger("52036cee2b6ffe738cc740797779e89800700a4d4141d8ab75eb4dca135978a3", 16)
            val y = java.math.BigInteger(1, yBytes.reversedArray())
            if (y >= p) return false
            val one = java.math.BigInteger.ONE
            val zero = java.math.BigInteger.ZERO
            val yy = y.multiply(y).mod(p)
            val u = yy.subtract(one).mod(p)
            val v = one.add(d.multiply(yy)).mod(p)
            if (v == zero) return false
            val x2 = try {
                u.multiply(v.modInverse(p)).mod(p)
            } catch (_: ArithmeticException) {
                return false
            }
            if (x2 == zero) return !signBit
            val sqrtM1 = java.math.BigInteger.valueOf(2)
                .modPow(p.subtract(one).divide(java.math.BigInteger.valueOf(4)), p)
            val exponent = p.add(java.math.BigInteger.valueOf(3))
                .divide(java.math.BigInteger.valueOf(8))
            var x = x2.modPow(exponent, p)
            var check = x.multiply(x).mod(p)
            if (check != x2) {
                x = x.multiply(sqrtM1).mod(p)
                check = x.multiply(x).mod(p)
                if (check != x2) return false
            }
            true
        } catch (_: Exception) {
            false
        }
    }
}
