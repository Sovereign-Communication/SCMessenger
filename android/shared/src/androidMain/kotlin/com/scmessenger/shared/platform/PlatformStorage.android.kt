package com.scmessenger.shared.platform

/**
 * Android actual implementation of PlatformStorage.
 *
 * Delegates to DataStore preferences.
 */
actual class PlatformStorage {
    actual suspend fun getString(key: String, default: String): String {
        // TODO: Delegate to DataStore on Android
        return default
    }

    actual suspend fun putString(key: String, value: String) {
        // TODO: Delegate to DataStore on Android
    }

    actual suspend fun getBoolean(key: String, default: Boolean): Boolean {
        return default
    }

    actual suspend fun putBoolean(key: String, value: Boolean) {
        // TODO: Delegate to DataStore on Android
    }

    actual suspend fun remove(key: String) {
        // TODO: Delegate to DataStore on Android
    }
}
