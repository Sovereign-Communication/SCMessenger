package com.scmessenger.android.transport.ble

import java.security.MessageDigest

/**
 * Produces a short, non-content fingerprint for correlating BLE reassembly
 * with the payload submitted to the mesh core. The full message remains
 * encrypted and is never written to diagnostics.
 */
internal object BlePayloadDiagnostics {
    fun fingerprint(payload: ByteArray): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(payload)
        return digest.take(8).joinToString(separator = "") { byte ->
            "%02x".format(byte.toInt() and 0xff)
        }
    }
}
