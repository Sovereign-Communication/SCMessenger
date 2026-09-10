package com.scmessenger.android.utils

import android.content.SharedPreferences
import org.junit.Assert.*
import org.junit.Test
import java.util.Base64

class SecurityUtilsTest {

    private class FakeEditor(private val backingMap: MutableMap<String, Any?>, private val parent: FakeSharedPreferences) : SharedPreferences.Editor {
        private val pending = mutableMapOf<String, Any?>()
        private val removed = mutableSetOf<String>()
        private var shouldClear = false
        var forceCommitFailure = false

        override fun putString(key: String, value: String?): SharedPreferences.Editor {
            pending[key] = value
            removed.remove(key)
            return this
        }

        override fun putStringSet(key: String, values: Set<String>?): SharedPreferences.Editor {
            pending[key] = values
            removed.remove(key)
            return this
        }

        override fun putInt(key: String, value: Int): SharedPreferences.Editor {
            pending[key] = value
            removed.remove(key)
            return this
        }

        override fun putLong(key: String, value: Long): SharedPreferences.Editor {
            pending[key] = value
            removed.remove(key)
            return this
        }

        override fun putFloat(key: String, value: Float): SharedPreferences.Editor {
            pending[key] = value
            removed.remove(key)
            return this
        }

        override fun putBoolean(key: String, value: Boolean): SharedPreferences.Editor {
            pending[key] = value
            removed.remove(key)
            return this
        }

        override fun remove(key: String): SharedPreferences.Editor {
            removed.add(key)
            pending.remove(key)
            return this
        }

        override fun clear(): SharedPreferences.Editor {
            shouldClear = true
            return this
        }

        override fun commit(): Boolean {
            if (forceCommitFailure) {
                return false
            }
            if (shouldClear) {
                backingMap.clear()
            }
            for (key in removed) {
                backingMap.remove(key)
            }
            for ((key, value) in pending) {
                if (value != null) {
                    backingMap[key] = value
                } else {
                    backingMap.remove(key)
                }
            }
            pending.clear()
            removed.clear()
            shouldClear = false
            return true
        }

        override fun apply() {
            commit()
        }
    }

    private class FakeSharedPreferences : SharedPreferences {
        val map = mutableMapOf<String, Any?>()
        var nextCommitFails = false

        override fun getAll(): Map<String, *> = map
        override fun getString(key: String, defValue: String?): String? = map[key] as? String ?: defValue
        override fun getStringSet(key: String, defValues: Set<String>?): Set<String>? =
            @Suppress("UNCHECKED_CAST") (map[key] as? Set<String>) ?: defValues
        override fun getInt(key: String, defValue: Int): Int = map[key] as? Int ?: defValue
        override fun getLong(key: String, defValue: Long): Long = map[key] as? Long ?: defValue
        override fun getFloat(key: String, defValue: Float): Float = map[key] as? Float ?: defValue
        override fun getBoolean(key: String, defValue: Boolean): Boolean = map[key] as? Boolean ?: defValue
        override fun contains(key: String): Boolean = map.containsKey(key)

        override fun edit(): SharedPreferences.Editor {
            val editor = FakeEditor(map, this)
            if (nextCommitFails) {
                editor.forceCommitFailure = true
                nextCommitFails = false
            }
            return editor
        }

        override fun registerOnSharedPreferenceChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener?) {}
        override fun unregisterOnSharedPreferenceChangeListener(listener: SharedPreferences.OnSharedPreferenceChangeListener?) {}
    }

    private val javaBase64Encoder: (ByteArray) -> String = { bytes ->
        Base64.getEncoder().encodeToString(bytes)
    }

    @Test
    fun `migrates legacy plaintext passphrase to target encrypted store and removes plaintext copy`() {
        val targetPrefs = FakeSharedPreferences()
        val legacyPrefs = FakeSharedPreferences()
        val legacyPassphrase = "legacy-test-passphrase-base64-bytes"

        legacyPrefs.map["backup_passphrase_v1"] = legacyPassphrase

        val result = SecurityUtils.migrateOrGeneratePassphrase(
            targetPrefs = targetPrefs,
            legacyPrefs = legacyPrefs,
            keyName = "backup_passphrase_v1",
            base64Encoder = javaBase64Encoder
        )

        assertEquals(legacyPassphrase, result)
        assertEquals(legacyPassphrase, targetPrefs.getString("backup_passphrase_v1", null))
        assertFalse("Legacy plaintext key must be removed after successful migration", legacyPrefs.contains("backup_passphrase_v1"))
    }

    @Test
    fun `uses existing encrypted passphrase directly without reading legacy store`() {
        val targetPrefs = FakeSharedPreferences()
        val legacyPrefs = FakeSharedPreferences()
        val encryptedPassphrase = "hardware-encrypted-passphrase"
        val staleLegacyPassphrase = "stale-plaintext-passphrase"

        targetPrefs.map["backup_passphrase_v1"] = encryptedPassphrase
        legacyPrefs.map["backup_passphrase_v1"] = staleLegacyPassphrase

        val result = SecurityUtils.migrateOrGeneratePassphrase(
            targetPrefs = targetPrefs,
            legacyPrefs = legacyPrefs,
            keyName = "backup_passphrase_v1",
            base64Encoder = javaBase64Encoder
        )

        assertEquals(encryptedPassphrase, result)
        assertEquals(encryptedPassphrase, targetPrefs.getString("backup_passphrase_v1", null))
        // Legacy store was untouched because target already had valid passphrase
        assertEquals(staleLegacyPassphrase, legacyPrefs.getString("backup_passphrase_v1", null))
    }

    @Test
    fun `generates and persists fresh 32-byte secure passphrase when no previous passphrase exists`() {
        val targetPrefs = FakeSharedPreferences()
        val legacyPrefs = FakeSharedPreferences()
        var generatedBytesCount = 0

        val result = SecurityUtils.migrateOrGeneratePassphrase(
            targetPrefs = targetPrefs,
            legacyPrefs = legacyPrefs,
            keyName = "backup_passphrase_v1",
            randomBytesGenerator = { bytes ->
                generatedBytesCount = bytes.size
                for (i in bytes.indices) bytes[i] = (i + 1).toByte()
            },
            base64Encoder = javaBase64Encoder
        )

        assertEquals(32, generatedBytesCount)
        assertNotNull(result)
        assertTrue(result.isNotEmpty())
        assertEquals(result, targetPrefs.getString("backup_passphrase_v1", null))
        assertFalse(legacyPrefs.contains("backup_passphrase_v1"))
    }

    @Test
    fun `preserves legacy plaintext copy if target commit or verification fails`() {
        val targetPrefs = FakeSharedPreferences()
        val legacyPrefs = FakeSharedPreferences()
        val legacyPassphrase = "legacy-precious-passphrase"
        legacyPrefs.map["backup_passphrase_v1"] = legacyPassphrase

        // Force next commit to target store to fail
        targetPrefs.nextCommitFails = true

        val result = SecurityUtils.migrateOrGeneratePassphrase(
            targetPrefs = targetPrefs,
            legacyPrefs = legacyPrefs,
            keyName = "backup_passphrase_v1",
            base64Encoder = javaBase64Encoder
        )

        // Must still return the legacy passphrase so operation does not crash
        assertEquals(legacyPassphrase, result)
        // MUST NOT delete from legacy store if target write failed
        assertTrue("Legacy plaintext key must be retained when migration verification fails", legacyPrefs.contains("backup_passphrase_v1"))
        assertEquals(legacyPassphrase, legacyPrefs.getString("backup_passphrase_v1", null))
    }
}
