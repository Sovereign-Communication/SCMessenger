package com.scmessenger.android.utils

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import timber.log.Timber

/**
 * Utility for initializing Android KeyStore backed EncryptedSharedPreferences.
 * Ensures identity keys and sensitive device states are encrypted at rest using AES-256 GCM.
 *
 * Paranoid Mode Enforcement:
 * Unencrypted storage fallbacks (MODE_PRIVATE) are strictly prohibited to prevent secret leakage.
 */
object SecurityUtils {

    const val ENCRYPTED_PREFS_FILENAME = "scmessenger_secure_prefs"

    fun getEncryptedSharedPreferences(context: Context): SharedPreferences {
        return try {
            createEncryptedSharedPreferences(context)
        } catch (e: Exception) {
            Timber.e(e, "Primary EncryptedSharedPreferences initialization failed; attempting KeyStore reset recovery")
            try {
                // Recovery path: clear stale prefs file and retry KeyStore creation
                context.deleteSharedPreferences(ENCRYPTED_PREFS_FILENAME)
                createEncryptedSharedPreferences(context)
            } catch (recoveryException: Exception) {
                Timber.e(recoveryException, "Hardware KeyStore recovery failed")
                throw SecurityException(
                    "Hardware KeyStore initialization failed — unencrypted storage prohibited in Paranoid Mode",
                    recoveryException
                )
            }
        }
    }

    /**
     * Safely migrates an existing plaintext passphrase to the hardware-backed encrypted store
     * or generates a new 32-byte secure passphrase if none exists.
     *
     * Invariant: The legacy plaintext value is ONLY deleted after the encrypted write is committed
     * AND successfully verified via read-back. If commit or read-back fails, the legacy copy is preserved
     * to prevent identity backup orphaning.
     */
    fun migrateOrGeneratePassphrase(
        targetPrefs: SharedPreferences,
        legacyPrefs: SharedPreferences,
        keyName: String,
        randomBytesGenerator: (ByteArray) -> Unit = { java.security.SecureRandom().nextBytes(it) },
        base64Encoder: (ByteArray) -> String = { android.util.Base64.encodeToString(it, android.util.Base64.NO_WRAP) }
    ): String {
        var key = targetPrefs.getString(keyName, null)
        if (!key.isNullOrBlank()) {
            return key
        }

        val legacyKey = legacyPrefs.getString(keyName, null)
        if (!legacyKey.isNullOrBlank()) {
            Timber.i("[INFO] Migrating identity backup passphrase to hardware-backed EncryptedSharedPreferences")
            val committed = targetPrefs.edit().putString(keyName, legacyKey).commit()
            val verified = targetPrefs.getString(keyName, null)

            if (committed && verified == legacyKey) {
                legacyPrefs.edit().remove(keyName).commit()
                Timber.i("[OK] Successfully migrated and verified identity backup passphrase in EncryptedSharedPreferences")
                return verified
            } else {
                Timber.e("[FAIL] Failed to verify migrated passphrase in EncryptedSharedPreferences; retaining legacy plaintext copy")
                return legacyKey
            }
        }

        Timber.i("[INFO] Generating fresh hardware-secured identity backup passphrase")
        val bytes = ByteArray(32)
        randomBytesGenerator(bytes)
        val newKey = base64Encoder(bytes)
        val committed = targetPrefs.edit().putString(keyName, newKey).commit()
        if (!committed) {
            Timber.e("[FAIL] Failed to commit new hardware-secured passphrase")
        }
        return newKey
    }

    private fun createEncryptedSharedPreferences(context: Context): SharedPreferences {
        val masterKey = MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()

        return EncryptedSharedPreferences.create(
            context,
            ENCRYPTED_PREFS_FILENAME,
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
        )
    }
}
