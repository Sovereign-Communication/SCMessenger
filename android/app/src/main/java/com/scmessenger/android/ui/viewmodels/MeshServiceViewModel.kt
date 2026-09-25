package com.scmessenger.android.ui.viewmodels

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import com.scmessenger.android.service.MeshForegroundService
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * ViewModel managing the mesh service lifecycle and state.
 *
 * This is shared across the app to provide consistent service state
 * and control methods.
 */
@HiltViewModel
@SuppressLint("StaticFieldLeak")
class MeshServiceViewModel @Inject constructor(
    @ApplicationContext private val context: Context,
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository
) : ViewModel() {

    // Service state from repository
    val serviceState = meshRepository.serviceState
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = uniffi.api.ServiceState.STOPPED
        )

    // Service stats from repository
    val serviceStats = meshRepository.serviceStats
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = null
        )

    // Preferences
    val autoStart = preferencesRepository.serviceAutoStart
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )

    // Derived state: is service running
    val isRunning = serviceState.map { it == uniffi.api.ServiceState.RUNNING }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )

    // Degraded storage tracking
    val isStorageDegraded = meshRepository.isStorageDegraded
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )

    init {
        Timber.d("MeshServiceViewModel initialized")
    }

    /**
     * Start the mesh service.
     */
    fun startService() {
        viewModelScope.launch {
            try {
                // Identity hydration is handled by MeshRepository.startMeshService()
                // during service initialization (ensureLocalIdentityFederation).
                // Calling getIdentityInfo() here would trigger ensureServiceInitializedFireAndForget()
                // which starts the service recursively, causing a service-start loop.

                val intent = Intent(context, MeshForegroundService::class.java).apply {
                    action = MeshForegroundService.ACTION_START
                }

                context.startForegroundService(intent)

                Timber.i("Mesh service start requested")
            } catch (e: Exception) {
                Timber.e(e, "Failed to start mesh service")
            }
        }
    }

    /**
     * Stop the mesh service.
     */
    fun stopService() {
        viewModelScope.launch {
            try {
                val intent = Intent(context, MeshForegroundService::class.java).apply {
                    action = MeshForegroundService.ACTION_STOP
                }
                context.startService(intent)

                Timber.i("Mesh service stop requested")
            } catch (e: Exception) {
                Timber.e(e, "Failed to stop mesh service")
                kotlin.runCatching {
                    context.stopService(Intent(context, MeshForegroundService::class.java))
                }.onFailure {
                    Timber.w(it, "Fallback stopService() call also failed")
                }
            }
        }
    }

    /**
     * Toggle the mesh service on/off.
     */
    fun toggleService() {
        // STOP-TEARDOWN-TIMEOUT-001 (2026-09-21): teardown is asynchronous, and
        // a stop the user thinks failed is a stop the user repeats. Measured on
        // the Pixel: every tap did call stopService(), each one re-entered the
        // wedged teardown and blocked on the same lifecycle monitor, and the
        // state read RUNNING throughout, so the UI kept claiming Active. A tap in
        // STOPPING or STARTING used to be a silent no-op (the old else-branch only
        // logged). Restart stays reachable only from STOPPED, and every other
        // state resolves to another stop request rather than a no-op.
        val state = serviceState.value
        if (shouldStartFrom(state)) {
            startService()
        } else {
            if (state != uniffi.api.ServiceState.RUNNING) {
                Timber.i("Toggle in state %s: re-requesting stop", state)
            }
            stopService()
        }
    }

    /**
     * Set auto-start preference.
     */
    fun setAutoStart(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setServiceAutoStart(enabled)
            Timber.d("Auto-start set to: $enabled")
        }
    }

    /**
     * Apply transport settings to enable/disable specific transports.
     * Called after settings change to dynamically update transport state.
     */
    fun applyTransportSettings(settings: uniffi.api.MeshSettings) {
        viewModelScope.launch {
            try {
                meshRepository.applyTransportSettings(settings)
            } catch (e: Exception) {
                Timber.e(e, "Failed to apply transport settings")
            }
        }
    }

    /**
     * Get formatted stats for display.
     */
    fun getStatsText(): String {
        val stats = serviceStats.value ?: return "No stats available"

        return buildString {
            appendLine("Peers Discovered: ${stats.peersDiscovered}")
            appendLine("Messages Relayed: ${stats.messagesRelayed}")
            appendLine("Bytes Transferred: ${formatBytes(stats.bytesTransferred)}")
            appendLine("Uptime: ${formatDuration(stats.uptimeSecs)}")
        }
    }

    companion object {
        /**
         * STOP-TEARDOWN-TIMEOUT-001: only STOPPED may start the mesh. Any other
         * state -- RUNNING, STARTING, and especially the asynchronous STOPPING --
         * resolves to a stop request, so a retry can never resurrect a mesh the
         * user just stopped. Pure, so it is unit-testable without an Android
         * runtime.
         */
        internal fun shouldStartFrom(state: uniffi.api.ServiceState): Boolean =
            state == uniffi.api.ServiceState.STOPPED
    }

    private fun formatBytes(bytes: ULong): String {
        return when {
            bytes < 1024u -> "$bytes B"
            bytes < 1024u * 1024u -> "${bytes / 1024u} KB"
            bytes < 1024u * 1024u * 1024u -> "${bytes / (1024u * 1024u)} MB"
            else -> "${bytes / (1024u * 1024u * 1024u)} GB"
        }
    }

    private fun formatDuration(seconds: ULong): String {
        val secs = seconds.toLong()
        val hours = secs / 3600
        val minutes = (secs % 3600) / 60
        val remainingSeconds = secs % 60

        return when {
            hours > 0 -> "${hours}h ${minutes}m"
            minutes > 0 -> "${minutes}m ${remainingSeconds}s"
            else -> "${remainingSeconds}s"
        }
    }
}
