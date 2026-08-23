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

    private const val ENCRYPTED_PREFS_FILENAME = "scmessenger_secure_prefs"

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
