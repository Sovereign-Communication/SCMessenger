package com.scmessenger.shared.platform

import java.io.File

/**
 * Desktop actual implementation of PlatformStorage.
 *
 * Uses file-based storage in XDG-compliant directories.
 */
actual class PlatformStorage {
    private val configDir: File by lazy {
        val xdgConfig = System.getenv("XDG_CONFIG_HOME")
        val dir = if (xdgConfig != null) {
            File(xdgConfig, "scmessenger")
        } else {
            File(System.getProperty("user.home"), ".config/scmessenger")
        }
        dir.mkdirs()
        dir
    }

    private val prefsFile get() = File(configDir, "preferences.properties")

    private fun loadPrefs(): java.util.Properties {
        val props = java.util.Properties()
        if (prefsFile.exists()) {
            prefsFile.inputStream().use { props.load(it) }
        }
        return props
    }

    private fun savePrefs(props: java.util.Properties) {
        prefsFile.outputStream().use { props.store(it, "SCMessenger Preferences") }
    }

    actual suspend fun getString(key: String, default: String): String {
        return loadPrefs().getProperty(key, default)
    }

    actual suspend fun putString(key: String, value: String) {
        val props = loadPrefs()
        props.setProperty(key, value)
        savePrefs(props)
    }

    actual suspend fun getBoolean(key: String, default: Boolean): Boolean {
        return loadPrefs().getProperty(key, default.toString()).toBooleanStrictOrNull() ?: default
    }

    actual suspend fun putBoolean(key: String, value: Boolean) {
        putString(key, value.toString())
    }

    actual suspend fun remove(key: String) {
        val props = loadPrefs()
        props.remove(key)
        savePrefs(props)
    }
}
