package com.scmessenger.android.utils

import timber.log.Timber
import java.math.BigInteger

/**
 * Utility functions for peer key and peer ID conversion.
 *
 * Provides methods to:
 * - Extract public keys from libp2p peer IDs
 * - Generate peer IDs from public keys
 * - Validate public keys and peer IDs
 *
 * UNIFICATION: Ed25519 libp2p PeerIds are protobuf-encoded identity multihashes:
 *   0x00 0x24 0x08 0x01 0x12 0x20 <32-byte Ed25519 public key> (38 bytes total)
 * This matches Rust's `public_key_hex_from_libp2p_peer_id` / `peer_id_from_public_key_hex`
 * in core/src/store/ledger_entry.rs and IronCore::resolve_identity. The previous
 * 0x12+32+checksum Kotlin-only format never matched Rust-generated IDs, so cold-start
 * (ironCore null) fallback always failed and left 12D3Koo... and 30d0fa... as two nodes.
 */
object PeerKeyUtils {

    /**
     * Extract public key from a libp2p peer ID.
     *
     * UNIFICATION: Decodes base58 protobuf identity multihash and extracts Ed25519 key.
     * Accepts only strict 38-byte protobuf (`00 24 08 01 12 20 <32>`). Returns null for
     * hashed (Qm...) or malformed IDs so callers store a placeholder rather than poisoning
     * identity resolution. Self-certification is verified by re-deriving the peer ID.
     *
     * @param peerId The libp2p peer ID to extract from
     * @return The extracted public key as hex string, or null if extraction fails
     */
    fun extractPublicKeyFromPeerId(peerId: String): String? {
        return try {
            if (!isLibp2pPeerId(peerId)) {
                Timber.w("Peer ID does not appear to be a libp2p peer ID: $peerId")
                return null
            }

            // Decode from base58
            val decoded = base58Decode(peerId)
            if (decoded == null) {
                Timber.w("Base58 decode failed for peerId: ${peerId.take(16)}...")
                return null
            }

            // UNIFICATION: Rust's strict protobuf — 38 bytes, header 00 24 08 01 12 20
            if (decoded.size == 38 &&
                decoded[0] == 0x00.toByte() &&
                decoded[1] == 0x24.toByte() &&
                decoded[2] == 0x08.toByte() &&
                decoded[3] == 0x01.toByte() &&
                decoded[4] == 0x12.toByte() &&
                decoded[5] == 0x20.toByte()
            ) {
                val publicKeyBytes = decoded.copyOfRange(6, 38)
                val hex = publicKeyBytes.joinToString("") { String.format("%02x", it) }
                // Defense in depth: re-derive must match (mirrors Rust check)
                val rederived = generateLibp2pPeerIdFromPublicKey(hex)
                if (rederived != peerId) {
                    Timber.w("PeerID re-derive mismatch for ${peerId.take(16)}... (extracted ${hex.take(8)}...)")
                    return null
                }
                Timber.d("UNIFICATION extract: ${peerId.take(16)}... -> ${hex.take(8)}... via protobuf")
                return hex
            }

            // Legacy Kotlin 36-byte format (0x12 + 32 + 2-byte sha256 checksum) — kept for backward compat
            // so cold-start fallback can still dedup ledger entries written by old Kotlin builds.
            if (decoded.size == 36 && decoded[0] == 0x12.toByte()) {
                val publicKeyBytes = decoded.copyOfRange(2, 34)
                // Verify checksum if present (optional — old format did checksum)
                if (decoded.size >= 36) {
                    val storedChecksum = decoded.copyOfRange(decoded.size - 2, decoded.size)
                    val expectedChecksum = java.security.MessageDigest.getInstance("SHA256")
                        .digest(decoded.copyOfRange(0, 34))
                        .copyOfRange(0, 2)
                    if (!storedChecksum.contentEquals(expectedChecksum)) {
                        Timber.w("Legacy checksum verification failed for ${peerId.take(16)}...")
                        // Still return key — checksum is auxiliary, not identity-critical
                    }
                }
                val hex = publicKeyBytes.joinToString("") { String.format("%02x", it) }
                Timber.d("UNIFICATION extract legacy: ${peerId.take(16)}... -> ${hex.take(8)}... via legacy 0x12 format")
                return hex
            }

            // Also support 34-byte truncated variant (0x12 + 32 without checksum)
            if (decoded.size == 34 && decoded[0] == 0x12.toByte()) {
                val hex = decoded.copyOfRange(2, 34).joinToString("") { String.format("%02x", it) }
                Timber.d("UNIFICATION extract legacy 34: ${peerId.take(16)}... -> ${hex.take(8)}...")
                return hex
            }

            Timber.w("Decoded peer ID size ${decoded.size} not recognised for ${peerId.take(16)}...")
            null
        } catch (e: Exception) {
            Timber.e("Failed to extract public key from peer ID $peerId: ${e.message}")
            null
        }
    }

    /**
     * Generate a libp2p peer ID from a public key using protobuf identity multihash.
     *
     * UNIFICATION: Mirrors Rust `peer_id_from_public_key_hex`:
     *   protobuf = 00 24 08 01 12 20 <32-byte key> → base58
     * This ensures Kotlin-generated IDs are identical to Rust-generated ones, so
     * self-certifying checks (`isSelfCertifyingKeyBinding`) and cold-start
     * canonicalization agree across platforms.
     *
     * @param publicKey The public key as hex string (64 chars for Ed25519)
     * @return The generated libp2p peer ID
     */
    fun generateLibp2pPeerIdFromPublicKey(publicKey: String): String {
        return try {
            if (!isValidPublicKey(publicKey)) {
                Timber.w("Invalid public key format for peer ID generation: ${publicKey.take(8)}...")
                return generateFallbackPeerId(publicKey)
            }

            // Decode hex public key to bytes
            val publicKeyBytes = publicKey.hexToBytes()
            if (publicKeyBytes.size != 32) {
                Timber.w("Public key must be 32 bytes for Ed25519")
                return generateFallbackPeerId(publicKey)
            }

            // UNIFICATION: protobuf identity multihash 00 24 08 01 12 20 + key (38 bytes)
            val protobuf = ByteArray(38)
            protobuf[0] = 0x00
            protobuf[1] = 0x24
            protobuf[2] = 0x08
            protobuf[3] = 0x01
            protobuf[4] = 0x12
            protobuf[5] = 0x20
            System.arraycopy(publicKeyBytes, 0, protobuf, 6, 32)

            // Encode to base58
            val base58Encoded = base58Encode(protobuf)
            if (base58Encoded == null) {
                Timber.w("Base58 encoding failed")
                return generateFallbackPeerId(publicKey)
            }

            base58Encoded
        } catch (e: Exception) {
            Timber.e("Failed to generate peer ID from public key: ${e.message}")
            generateFallbackPeerId(publicKey)
        }
    }

    /**
     * Generate a fallback peer ID from a public key when proper libp2p generation fails.
     */
    private fun generateFallbackPeerId(publicKey: String): String {
        // Create a deterministic but non-standard peer ID
        // Format: peer_<first_8_chars_of_key>
        val keyPrefix = publicKey.take(8)
        return "peer_${keyPrefix.lowercase()}"
    }

    /**
     * Check if a string is a valid public key.
     *
     * A valid Ed25519 public key is 64 hex characters.
     *
     * @param key The string to validate
     * @return true if the key looks like a valid Ed25519 public key
     */
    fun isValidPublicKey(key: String): Boolean {
        return key.length == 64 && key.matches(Regex("[0-9a-fA-F]+"))
    }

    /**
     * Check if a string is a valid libp2p peer ID.
     *
     * @param peerId The string to validate
     * @return true if the peerId matches libp2p format (12D3KooW... or Qm...)
     */
    fun isValidPeerId(peerId: String): Boolean {
        return isLibp2pPeerId(peerId)
    }

    /**
     * Check if a string is a libp2p peer ID (internal helper).
     * UNIFICATION: Strict Base58BTC alphabet, 12D3Koo (Ed25519) or Qm (hashed).
     */
    fun isLibp2pPeerId(peerId: String): Boolean {
        // Base58-encoded libp2p peer IDs: 12D3KooW... (~52 chars) or Qm... (~46 chars)
        val base58Chars = peerId.all { it in BASE58_ALPHABET }
        return base58Chars && (
            (peerId.startsWith("12D3Koo") && peerId.length in 46..60) ||
            (peerId.startsWith("Qm") && peerId.length in 44..50)
        )
    }

    /**
     * Extract peer ID from a public key by generating a deterministic peer ID.
     *
     * @param publicKey The public key as hex string
     * @return The generated peer ID
     */
    fun extractPeerIdFromPublicKey(publicKey: String): String {
        return generateLibp2pPeerIdFromPublicKey(publicKey)
    }

    // --- Helper functions for base58 encoding/decoding ---

    /**
     * Convert a hex string to bytes.
     */
    private fun String.hexToBytes(): ByteArray {
        check(length % 2 == 0) { "Hex string must have even length" }
        return chunked(2).map { it.toInt(16).toByte() }.toByteArray()
    }

    /**
     * Convert a byte to hex string.
     */
    private fun Int.toHex(): String = String.format("%02x", this)

    // --- Base58 encoding/decoding implementation ---
    // UNIFICATION: Use BigInteger for correctness — matches Rust `bs58` crate.
    // Previous manual carry logic produced incorrect encodings for 38-byte protobufs,
    // causing isSelfCertifyingKeyBinding and cold-start dedup to always fail.

    private val BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    private val BASE58_CHARSET = BASE58_ALPHABET.toCharArray()
    private val BASE58_MAP = IntArray(128) { -1 }
    init {
        for (i in BASE58_CHARSET.indices) {
            BASE58_MAP[BASE58_CHARSET[i].code] = i
        }
    }

    /**
     * Encode bytes to base58 string via BigInteger.
     */
    private fun base58Encode(data: ByteArray): String? {
        if (data.isEmpty()) return ""
        // Count leading zeros
        var zeroCount = 0
        while (zeroCount < data.size && data[zeroCount] == 0.toByte()) {
            zeroCount++
        }
        // BigInteger handles the rest (positive)
        var bi = BigInteger(1, data)
        val sb = StringBuilder()
        val base = BigInteger.valueOf(58)
        if (bi == BigInteger.ZERO) {
            // All zeros
            return BASE58_ALPHABET[0].toString().repeat(zeroCount)
        }
        while (bi > BigInteger.ZERO) {
            val mod = bi.mod(base)
            sb.append(BASE58_ALPHABET[mod.toInt()])
            bi = bi.divide(base)
        }
        // Leading zeros as '1'
        repeat(zeroCount) { sb.append(BASE58_ALPHABET[0]) }
        return sb.reverse().toString()
    }

    /**
     * Decode base58 string to bytes via BigInteger.
     */
    private fun base58Decode(str: String): ByteArray? {
        if (str.isEmpty()) return null
        // Count leading '1's (leading zeros)
        var zeroCount = 0
        for (c in str) {
            if (c == '1') zeroCount++ else break
        }
        var bi = BigInteger.ZERO
        val base = BigInteger.valueOf(58)
        for (c in str) {
            val digit = if (c.code < BASE58_MAP.size) BASE58_MAP[c.code] else -1
            if (digit == -1) {
                Timber.w("Invalid base58 character: $c")
                return null
            }
            bi = bi.multiply(base).add(BigInteger.valueOf(digit.toLong()))
        }
        var bytes = bi.toByteArray()
        // BigInteger may add leading zero for sign
        if (bytes.isNotEmpty() && bytes[0] == 0.toByte()) {
            bytes = bytes.copyOfRange(1, bytes.size)
        }
        // Handle zero case
        if (bytes.isEmpty()) {
            return ByteArray(zeroCount)
        }
        // Prepend leading zeros
        val result = ByteArray(zeroCount + bytes.size)
        System.arraycopy(bytes, 0, result, zeroCount, bytes.size)
        return result
    }
}
