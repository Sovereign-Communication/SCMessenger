package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.BuildConfig
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import com.scmessenger.android.network.DiagnosticsReporter
import com.scmessenger.android.utils.NotificationHelper
import com.scmessenger.android.utils.Permissions
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.delay
import timber.log.Timber
import javax.inject.Inject
import java.util.concurrent.atomic.AtomicLong
import kotlin.concurrent.Volatile

/**
 * ViewModel for the settings screen.
 *
 * Manages mesh settings, app preferences, and configuration.
 */
@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository,
    private val diagnosticsReporter: DiagnosticsReporter
) : ViewModel() {

    // Mesh settings
    private val _settings = MutableStateFlow<uniffi.api.MeshSettings?>(null)
    val settings: StateFlow<uniffi.api.MeshSettings?> = _settings.asStateFlow()

    // ANR FIX: Debouncing for settings updates to prevent UI thread spew
    // Use AtomicLong for thread-safe timestamp tracking without locks
    private val lastSettingUpdateNs = AtomicLong(0L)
    private val settingDebounceNs = 500_000_000L  // 500ms debounce

    // App preferences
    val autoStart = preferencesRepository.serviceAutoStart
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )

    val vpnMode = preferencesRepository.vpnModeEnabled
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = false
        )

    val themeMode = preferencesRepository.themeMode
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = PreferencesRepository.ThemeMode.SYSTEM
        )

    val notificationsEnabled = preferencesRepository.notificationsEnabled
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = true
        )

    // Notification sub-settings (backed by MeshSettings)
    private val _notifyDmEnabled = MutableStateFlow(true)
    val notifyDmEnabled: StateFlow<Boolean> = _notifyDmEnabled.asStateFlow()

    private val _notifyDmRequestEnabled = MutableStateFlow(true)
    val notifyDmRequestEnabled: StateFlow<Boolean> = _notifyDmRequestEnabled.asStateFlow()

    private val _notifyDmInForeground = MutableStateFlow(false)
    val notifyDmInForeground: StateFlow<Boolean> = _notifyDmInForeground.asStateFlow()

    private val _notifyDmRequestInForeground = MutableStateFlow(true)
    val notifyDmRequestInForeground: StateFlow<Boolean> = _notifyDmRequestInForeground.asStateFlow()

    private val _soundEnabled = MutableStateFlow(true)
    val soundEnabled: StateFlow<Boolean> = _soundEnabled.asStateFlow()

    private val _badgeEnabled = MutableStateFlow(true)
    val badgeEnabled: StateFlow<Boolean> = _badgeEnabled.asStateFlow()

    val showPeerCount = preferencesRepository.showPeerCount
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = true
        )

    val autoAdjustEnabled = preferencesRepository.autoAdjustEnabled
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = true
        )

    val adjustmentProfile = preferencesRepository.manualAdjustmentProfile
        .map { name ->
            try {
                uniffi.api.AdjustmentProfile.valueOf(name)
            } catch (e: IllegalArgumentException) {
                uniffi.api.AdjustmentProfile.STANDARD
            }
        }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = uniffi.api.AdjustmentProfile.STANDARD
        )

    // BLE Privacy (mirrors iOS BLE rotation settings)
    val bleRotationEnabled = preferencesRepository.bleRotationEnabled
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = true
        )

    val bleRotationIntervalSec = preferencesRepository.bleRotationIntervalSec
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = 900
        )

    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()
    val isSaving: StateFlow<Boolean> = _isLoading.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    // Identity
    private val _identityInfo = MutableStateFlow<uniffi.api.IdentityInfo?>(meshRepository.identityInfo.value)
    val identityInfo: StateFlow<uniffi.api.IdentityInfo?> = _identityInfo.asStateFlow()
    private val _hasIdentity = MutableStateFlow(meshRepository.isIdentityInitialized())
    val hasIdentity: StateFlow<Boolean> = _hasIdentity.asStateFlow()

    // Import result
    private val _importResult = MutableStateFlow<String?>(null)
    val importResult: StateFlow<String?> = _importResult.asStateFlow()

    // Encrypted identity backup export result (the backup string, or an error message)
    private val _backupExportResult = MutableStateFlow<Result<String>?>(null)
    val backupExportResult: StateFlow<Result<String>?> = _backupExportResult.asStateFlow()

    // ANR FIX: Cached settings to avoid re-calculating on every composition
    private var cachedSettings: uniffi.api.MeshSettings? = null
    private var cachedIdentityInfo: uniffi.api.IdentityInfo? = null
    private var isCacheValid = false

    // ANR FIX: Guard identity emission — only emit when data actually changed.
    // Prevents infinite recomposition loops when getIdentityInfo() returns
    // structurally equal objects that would otherwise cascade through
    // StateFlow → hasIdentity → LaunchedEffect(hasIdentity) → loadIdentity().
    private fun emitIdentityInfo(info: uniffi.api.IdentityInfo?) {
        if (_identityInfo.value != info) {
            _identityInfo.value = info
            cachedIdentityInfo = info
        }
        if (info?.initialized == true) {
            _hasIdentity.value = true
        } else {
            _hasIdentity.value = meshRepository.isIdentityInitialized()
        }
    }

    /** Contact/message counts + core build provenance for the Info section. */
    data class InfoCounts(
        val contactCount: UInt = 0u,
        val messageCount: UInt = 0u,
        val buildProvenance: String = ""
    )

    // Declared BEFORE the init block on purpose: refreshInfoCounts() is called
    // from init and its IO coroutine can complete before later property
    // initializers run. Kotlin executes initializers in declaration order, so
    // the StateFlow must exist before any writer can touch it (init-order NPE
    // surfaced by the full unit suite, 2026-09-10).
    private val _infoCounts = MutableStateFlow(InfoCounts())
    val infoCounts: StateFlow<InfoCounts> = _infoCounts.asStateFlow()

    init {
        // P0_SHARED_IDENTITY: mirror the centralized meshRepository.identityInfo
        // StateFlow into the local _identityInfo so any identity change from
        // anywhere (MainVM, IdentityVM, manual import) propagates here without
        // needing to call loadIdentity() again.
        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            meshRepository.identityInfo.collect { info ->
                emitIdentityInfo(info)
            }
        }

        // ANR FIX (P0_ANDROID_017): Defer heavy initialization to background thread
        // Settings screen should appear within 500ms for good UX
        // Set defaults immediately so UI never shows missing sections
        _settings.value = uniffi.api.MeshSettings(
            relayEnabled = true,
            maxRelayBudget = 200u,
            batteryFloor = 20u,
            bleEnabled = true,
            wifiAwareEnabled = false,
            wifiDirectEnabled = false,
            internetEnabled = true,
            discoveryMode = uniffi.api.DiscoveryMode.NORMAL,
            onionRouting = false,
            coverTrafficEnabled = false,
            messagePaddingEnabled = false,
            timingObfuscationEnabled = false,
            notificationsEnabled = true,
            notifyDmEnabled = true,
            notifyDmRequestEnabled = true,
            notifyDmInForeground = false,
            notifyDmRequestInForeground = true,
            soundEnabled = true,
            badgeEnabled = true,
            requirePq = false,
        )

        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                _isLoading.value = true
                loadSettingsInternal()
                loadIdentityInternal()
                isCacheValid = true
            } catch (e: Exception) {
                Timber.e(e, "Failed to initialize SettingsViewModel")
            } finally {
                _isLoading.value = false
            }
        }

        // Reactive identity refresh: when mesh service transitions to RUNNING,
        // immediately refresh identity so the UI shows PeerID without polling delay.
        // Force-refresh replaces cached SharedPreferences data with live Rust core data.
        var lastServiceState: uniffi.api.ServiceState? = null
        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            meshRepository.serviceState.collect { state ->
                if (state == uniffi.api.ServiceState.RUNNING &&
                    lastServiceState != uniffi.api.ServiceState.RUNNING) {
                    Timber.d("SettingsViewModel: service -> RUNNING, force-refreshing identity")
                    loadIdentityInternal(forceRefresh = true)
                    // P1_ANDROID_020: Sync nickname from DataStore fallback to Rust Core
                    Timber.d("SettingsViewModel: syncing nickname from DataStore fallback to Rust Core")
                    meshRepository.syncNicknameFromDatastore()
                }
                lastServiceState = state
            }
        }

        // ANR-2026-09-09 recurrence fix: InfoSection used to call
        // getContactCount()/getMessageCount()/getBuildProvenance() directly in
        // composition — synchronous FFI on the main thread on EVERY
        // recomposition of SettingsScreen (verified ANR source in the 4-23
        // device logs). Load once on IO here and publish as StateFlow; the
        // screen collects instead of calling.
        refreshInfoCounts()
    }

    /**
     * Reload the Info-section counts on IO. Cheap enough to re-run on service
     * RUNNING transitions; never called from composition. Safe to call from
     * init because [_infoCounts] is declared before the init block.
     */
    fun refreshInfoCounts() {
        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            val contactCount = try {
                meshRepository.getContactCount()
            } catch (e: Exception) {
                Timber.w(e, "Failed to load contact count")
                0u
            }
            val messageCount = try {
                meshRepository.getMessageCount()
            } catch (e: Exception) {
                Timber.w(e, "Failed to load message count")
                0u
            }
            val buildProvenance = try {
                meshRepository.getBuildProvenance()
            } catch (e: Exception) {
                Timber.w(e, "Failed to load build provenance")
                ""
            }
            _infoCounts.value = InfoCounts(
                contactCount = contactCount,
                messageCount = messageCount,
                buildProvenance = buildProvenance
            )
        }
    }

    /**
     * Internal settings loader that runs on IO dispatcher.
     * ANR FIX: Uses cached settings when available to avoid redundant I/O.
     */
    private suspend fun loadSettingsInternal() {
        // ANR FIX: Return cached settings immediately if already loaded
        if (cachedSettings != null && isCacheValid) {
            _settings.value = cachedSettings
            return
        }
        try {
            val settings = meshRepository.loadSettings()
            cachedSettings = settings  // ANR FIX: Cache for future use
            _settings.value = settings
            // Hydrate notification sub-settings from loaded MeshSettings
            _notifyDmEnabled.value = settings.notifyDmEnabled
            _notifyDmRequestEnabled.value = settings.notifyDmRequestEnabled
            _notifyDmInForeground.value = settings.notifyDmInForeground
            _notifyDmRequestInForeground.value = settings.notifyDmRequestInForeground
            _soundEnabled.value = settings.soundEnabled
            _badgeEnabled.value = settings.badgeEnabled
            // NOTIF-GATE FIX: push all notification runtime gates to the helper
            // singleton (updateSettings previously had no call sites, so helper
            // fields stayed at their WS14 defaults regardless of saved settings).
            NotificationHelper.updateSettings(
                dmEnabled = settings.notifyDmEnabled,
                dmRequestEnabled = settings.notifyDmRequestEnabled,
                dmInForeground = settings.notifyDmInForeground,
                dmRequestInForeground = settings.notifyDmRequestInForeground,
                sound = settings.soundEnabled,
                badge = settings.badgeEnabled
            )
            Timber.d("Loaded mesh settings: $settings")
        } catch (e: Exception) {
            _error.value = "Failed to load settings: ${e.message}"
            Timber.e(e, "Failed to load mesh settings")
        }
    }

    /**
     * Internal identity loader that runs on IO dispatcher.
     * ANR FIX: Uses cached identity info when available to avoid redundant FFI calls.
     * Uses non-blocking identity access to avoid main thread blocking during service startup.
     *
     * @param forceRefresh When true, skips the cache and loads fresh data from Rust core.
     *                     Used when service transitions to RUNNING to ensure live data replaces
     *                     cached data from SharedPreferences.
     */
    private suspend fun loadIdentityInternal(forceRefresh: Boolean = false) {
        // ANR FIX: Return cached identity if already loaded (unless force-refreshing)
        if (!forceRefresh && cachedIdentityInfo != null && isCacheValid) {
            emitIdentityInfo(cachedIdentityInfo)
            return
        }
        try {
            // Use non-blocking identity access to avoid blocking main thread
            // during service startup when identity might not be fully initialized yet
            val info = meshRepository.getIdentityInfoNonBlocking()
            // Defensive fallback: if Rust core returns blank nickname, use cached preference
            if (info != null && info.nickname.isNullOrBlank()) {
                val cached = preferencesRepository.identityNickname.first()
                if (!cached.isNullOrBlank()) {
                    Timber.w("Rust core nickname blank; using DataStore fallback: $cached")
                    // Push fallback to Rust Core to permanently fix the null state
                    meshRepository.setNickname(cached)
                    emitIdentityInfo(info.copy(nickname = cached))
                    return
                }
            }
            emitIdentityInfo(info)
        } catch (e: Exception) {
            Timber.e(e, "Failed to load identity")
        }
    }

    fun loadIdentity(forceRefresh: Boolean = false) {
        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                // ANR FIX: Return cached identity if already loaded — prevents
                // redundant FFI calls during recomposition cascades.
                // Skip cache when force-refreshing (service RUNNING transition).
                if (!forceRefresh && cachedIdentityInfo != null && isCacheValid) {
                    emitIdentityInfo(cachedIdentityInfo)
                    return@launch
                }
                // P0_SHARED_IDENTITY: Check the published StateFlow first — this
                // catches identity that was just created in another ViewModel
                // (e.g., onboarding) before the non-blocking FFI path runs.
                val published = meshRepository.identityInfo.value
                if (published != null && published.initialized) {
                    emitIdentityInfo(published)
                    return@launch
                }
                // Use non-blocking variant to avoid triggering service init
                // from the UI layer (prevents serviceState → loadIdentity loop).
                val info = meshRepository.getIdentityInfoNonBlocking()
                // Defensive fallback: if Rust core returns blank nickname, use cached preference
                if (info != null && info.nickname.isNullOrBlank()) {
                    val cached = preferencesRepository.identityNickname.first()
                    if (!cached.isNullOrBlank()) {
                        Timber.w("Rust core nickname blank; using DataStore fallback: $cached")
                        // Push fallback to Rust Core to permanently fix the null state
                        meshRepository.setNickname(cached)
                        emitIdentityInfo(info.copy(nickname = cached))
                        return@launch
                    }
                }
                emitIdentityInfo(info)
            } catch (e: Exception) {
                Timber.e(e, "Failed to load identity")
            }
        }
    }

    suspend fun getIdentityExportString(): String {
        // ANR FIX: Run on IO dispatcher to avoid blocking Main thread
        return kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            meshRepository.getIdentityExportString()
        }
    }

    /**
     * P1_ANDROID_003: Import identity from a backup string.
     *
     * If [passphrase] is blank, falls back to the device-bound auto-backup
     * passphrase (for restoring this app's own local backup). If non-blank,
     * it's used as-is to decrypt a backup created via [exportIdentityBackup].
     */
    fun importIdentityBackup(backup: String, passphrase: String = "") {
        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            try {
                meshRepository.restoreIdentityFromBackup(backup, passphrase.ifBlank { null })
                _importResult.value = "Identity restored successfully"
                loadIdentity() // Refresh UI after import
            } catch (e: Exception) {
                _importResult.value = "Failed to restore identity: ${e.message}"
                Timber.e(e, "Failed to restore identity from backup")
            }
        }
    }

    fun clearImportResult() {
        _importResult.value = null
    }

    /**
     * Export a passphrase-encrypted identity backup (identity key + ratchet
     * sessions + contacts). Distinct from [getIdentityExportString], which
     * exports the public identity card with no encryption.
     */
    fun exportIdentityBackup(passphrase: String) {
        viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
            _backupExportResult.value = try {
                Result.success(meshRepository.exportIdentityBackup(passphrase))
            } catch (e: Exception) {
                Timber.e(e, "Failed to export identity backup")
                Result.failure(e)
            }
        }
    }

    fun clearBackupExportResult() {
        _backupExportResult.value = null
    }

    fun updateNickname(name: String) {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                meshRepository.setNickname(name)
                loadIdentity() // Refresh to reflect change
            } catch (e: Exception) {
                _error.value = "Failed to update nickname: ${e.message}"
                Timber.e(e, "Failed to update nickname")
            }
        }
    }

    // ========================================================================
    // MESH SETTINGS
    // ========================================================================

    /**
     * Load mesh settings from repository.
     */
    fun loadSettings() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null

                val settings = meshRepository.loadSettings()
                _settings.value = settings

                Timber.d("Loaded mesh settings: $settings")
            } catch (e: Exception) {
                _error.value = "Failed to load settings: ${e.message}"
                Timber.e(e, "Failed to load mesh settings")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * ANR FIX (P0_ANDROID_017): Debounced settings update.
     * Prevents rapid successive updates from causing excessive I/O and UI thread blocking.
     * Uses timestamp-based throttling with 500ms minimum interval between updates.
     */
    fun debouncedUpdateSettings(settings: uniffi.api.MeshSettings) {
        val nowNs = System.nanoTime()
        val lastUpdateNs = lastSettingUpdateNs.get()
        val timeSinceLastUpdateNs = nowNs - lastUpdateNs

        if (timeSinceLastUpdateNs < settingDebounceNs) {
            // Still within debounce window - log and skip
            Timber.d("Settings update throttled (${timeSinceLastUpdateNs / 1_000_000}ms since last)")
            return
        }

        // Try to atomically update the timestamp
        if (lastSettingUpdateNs.compareAndSet(lastUpdateNs, nowNs)) {
            // Success - we got the lock, proceed with update
            viewModelScope.launch {
                try {
                    // Validate settings
                    if (!meshRepository.validateSettings(settings)) {
                        _error.value = "Invalid settings configuration"
                        Timber.w("Invalid settings configuration")
                        return@launch
                    }

                    meshRepository.saveSettings(settings)
                    _settings.value = settings
                    Timber.i("Mesh settings saved (debounced)")
                } catch (e: Exception) {
                    _error.value = "Failed to save settings: ${e.message}"
                    Timber.e(e, "Failed to save mesh settings")
                }
            }
        } else {
            // Failed to atomically update - someone else beat us, skip this update
            Timber.d("Settings update skipped (concurrent update in progress)")
        }
    }

    /**
     * Update mesh settings - now redirects to debounced version.
     */
    fun updateSettings(settings: uniffi.api.MeshSettings) {
        debouncedUpdateSettings(settings)
        _settings.value?.let { current ->
            if (current.wifiAwareEnabled != settings.wifiAwareEnabled ||
                current.wifiDirectEnabled != settings.wifiDirectEnabled ||
                current.bleEnabled != settings.bleEnabled ||
                current.internetEnabled != settings.internetEnabled) {
                meshRepository.applyTransportSettings(settings)
            }
        }
    }

    /**
     * Update specific mesh setting fields - now uses debounced updates.
     */
    fun updateRelayEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            // Allow toggling - when disabled, stops ALL communication (bidirectional)
            debouncedUpdateSettings(current.copy(relayEnabled = enabled))
            // Apply transport settings when mesh participation changes
            meshRepository.applyTransportSettings(current.copy(relayEnabled = enabled))
        }
    }

    fun updateMaxRelayBudget(budget: UInt) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(maxRelayBudget = budget))
        }
    }

    fun updateBatteryFloor(floor: UByte) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(batteryFloor = floor))
        }
    }

    fun updateBleEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(bleEnabled = enabled))
            // Apply transport settings to enable/disable BLE
            meshRepository.applyTransportSettings(current.copy(bleEnabled = enabled))
        }
    }

    fun updateWifiAwareEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(wifiAwareEnabled = enabled))
            // Apply transport settings to enable/disable WiFi Aware
            meshRepository.applyTransportSettings(current.copy(wifiAwareEnabled = enabled))
        }
    }

    fun updateWifiDirectEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(wifiDirectEnabled = enabled))
            // Apply transport settings to enable/disable WiFi Direct
            meshRepository.applyTransportSettings(current.copy(wifiDirectEnabled = enabled))
        }
    }

    fun updateInternetEnabled(enabled: Boolean) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(internetEnabled = enabled))
            // Apply transport settings to enable/disable the internet transport
            meshRepository.applyTransportSettings(current.copy(internetEnabled = enabled))
        }
    }

    fun updateDiscoveryMode(mode: uniffi.api.DiscoveryMode) {
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(discoveryMode = mode))
        }
    }

    /**
     * Reset mesh settings to factory defaults.
     */
    fun resetSettingsToDefault() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                _isLoading.value = true
                val defaults = meshRepository.getDefaultSettings()
                debouncedUpdateSettings(defaults)
                Timber.i("Mesh settings reset to defaults")
            } catch (e: Exception) {
                _error.value = "Failed to reset settings: ${e.message}"
                Timber.e(e, "Failed to reset mesh settings")
            } finally {
                _isLoading.value = false
            }
        }
    }



    // ========================================================================
    // APP PREFERENCES
    // ========================================================================

    fun setAutoStart(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setServiceAutoStart(enabled)
        }
    }

    fun setVpnMode(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setVpnMode(enabled)
        }
    }

    fun setThemeMode(mode: PreferencesRepository.ThemeMode) {
        viewModelScope.launch {
            preferencesRepository.setThemeMode(mode)
        }
    }

    fun setNotificationsEnabled(enabled: Boolean) {
        // NOTIF-GATE FIX (2026-09-10): the NotificationHelper singleton's
        // updateSettings() had no call sites, so this toggle was persisted but
        // never reached the runtime gate that suppresses message notifications.
        // Apply to the helper immediately (same-process singleton) AND persist.
        NotificationHelper.updateSettings(enabled = enabled)
        viewModelScope.launch {
            preferencesRepository.setNotificationsEnabled(enabled)
        }
    }

    // ========================================================================
    // NOTIFICATION SUB-SETTINGS
    // ========================================================================

    fun setNotifyDmEnabled(enabled: Boolean) {
        _notifyDmEnabled.value = enabled
        NotificationHelper.updateSettings(dmEnabled = enabled)
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(notifyDmEnabled = enabled))
        }
    }

    fun setNotifyDmRequestEnabled(enabled: Boolean) {
        _notifyDmRequestEnabled.value = enabled
        NotificationHelper.updateSettings(dmRequestEnabled = enabled)
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(notifyDmRequestEnabled = enabled))
        }
    }

    fun setNotifyDmInForeground(enabled: Boolean) {
        _notifyDmInForeground.value = enabled
        NotificationHelper.updateSettings(dmInForeground = enabled)
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(notifyDmInForeground = enabled))
        }
    }

    fun setNotifyDmRequestInForeground(enabled: Boolean) {
        _notifyDmRequestInForeground.value = enabled
        NotificationHelper.updateSettings(dmRequestInForeground = enabled)
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(notifyDmRequestInForeground = enabled))
        }
    }

    fun setSoundEnabled(enabled: Boolean) {
        _soundEnabled.value = enabled
        NotificationHelper.updateSettings(sound = enabled)
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(soundEnabled = enabled))
        }
    }

    fun setBadgeEnabled(enabled: Boolean) {
        _badgeEnabled.value = enabled
        NotificationHelper.updateSettings(badge = enabled)
        _settings.value?.let { current ->
            debouncedUpdateSettings(current.copy(badgeEnabled = enabled))
        }
    }

    fun setShowPeerCount(show: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setShowPeerCount(show)
        }
    }

    // ========================================================================
    // POWER SETTINGS
    // ========================================================================

    fun setAutoAdjust(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setAutoAdjustEnabled(enabled)
            if (enabled) {
                meshRepository.clearAdjustmentOverrides()
            }
        }
    }

    fun setManualProfile(profile: uniffi.api.AdjustmentProfile) {
        viewModelScope.launch {
            preferencesRepository.setManualAdjustmentProfile(profile.name)

            // Apply profile settings if manual
            // Note: In a real implementation, we might apply these to the engine directly
            // or the engine observes this preference.
            // For now, we just save the preference.
        }
    }

    fun overrideBleScanInterval(intervalMs: UInt) {
        meshRepository.overrideBleInterval(intervalMs)
    }

    fun overrideRelayMax(max: UInt) {
        meshRepository.overrideRelayMax(max)
    }

    fun clearAdjustmentOverrides() {
        meshRepository.clearAdjustmentOverrides()
    }

    // ========================================================================
    // UTILITIES
    // ========================================================================

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }

    /**
     * ANR FIX (P0_ANDROID_017): Export diagnostics asynchronously.
     * Uses the repository's async variant to avoid blocking on file I/O and Rust FFI calls.
     */
    suspend fun exportDiagnostics(): String {
        return meshRepository.exportDiagnosticsAsync()
    }

    fun getDiagnosticsLogPath(): String {
        return meshRepository.getDiagnosticsLogPath()
    }

    /**
     * ANR FIX (P0_ANDROID_017): Get diagnostics logs asynchronously.
     * Reading log files was blocking on Main thread.
     */
    suspend fun getDiagnosticsLogs(limit: Int = 500): String {
        return kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            meshRepository.getDiagnosticsLogs(limit)
        }
    }

    fun clearDiagnosticsLogs() {
        meshRepository.clearDiagnosticsLogs()
    }

    /** P0_ANDROID_007: Retry bootstrap with fallback strategy from diagnostics UI. */
    fun retryBootstrap() {
        viewModelScope.launch {
            try {
                meshRepository.bootstrapWithFallbackStrategy()
                Timber.i("Bootstrap retry initiated")
            } catch (e: Exception) {
                Timber.e(e, "Bootstrap retry failed")
            }
        }
    }

    /**
     * ANR FIX (P0_ANDROID_017): Build diagnostics bundle asynchronously.
     * All blocking I/O operations (exportDiagnostics, getPendingOutboxCount, getDiagnosticsLogs)
     * are moved to IO dispatcher to prevent UI thread blocking.
     */
    suspend fun buildTesterDiagnosticsBundle(): String {
        return kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) {
            "Paranoid Mode active: Telemetry and diagnostic exports disabled."
        }
    }

    // ========================================================================
    // DIAGNOSTICS
    // ========================================================================

    /**
     * Get network diagnostics report from DiagnosticsReporter.
     */
    suspend fun getNetworkDiagnosticsReport(): DiagnosticsReporter.NetworkDiagnosticsReport? {
        return try {
            diagnosticsReporter.generateReport()
        } catch (e: Exception) {
            Timber.e(e, "Failed to generate diagnostics report")
            null
        }
    }



    // MARK: - Identity Helpers
    /**
     * Get blocked peer count for info display.
     */
    fun getBlockedCount(): UInt {
        return meshRepository.getBlockedCount()
    }

    /**
     * Get inbox message count for badge display.
     */
    fun getInboxCount(): UInt {
        return meshRepository.getInboxCount()
    }

    /**
     * Get bootstrap nodes for settings display.
     */
    fun getBootstrapNodesForSettings(): List<String> {
        return meshRepository.getBootstrapNodesForSettings()
    }

    /**
     * Reset service runtime stats for a fresh diagnostics window.
     */
    fun resetServiceStats() {
        meshRepository.resetServiceStats()
    }

    /**
     * Check if a transport should be used based on health status.
     * Wired from MeshRepository.shouldUseTransport.
     *
     * @param transport The transport type (BLE, WiFi, etc.)
     * @return True if the transport is healthy and should be used
     */
    fun shouldUseTransport(transport: String): Boolean {
        return meshRepository.shouldUseTransport(transport)
    }

    /**
     * Handle BLE transport failure with graceful degradation.
     * Reduces BLE usage and prioritizes other transports when BLE fails.
     */
    fun handleBleFailure() {
        viewModelScope.launch {
            try {
                meshRepository.handleBleFailure()
                Timber.d("BLE transport failure handled")
            } catch (e: Exception) {
                Timber.e(e, "Failed to handle BLE failure")
            }
        }
    }

    /**
     * Attempt BLE recovery after degradation.
     * Should be called after a cooldown period to retry BLE operations.
     */
    fun attemptBleRecovery() {
        viewModelScope.launch {
            try {
                meshRepository.attemptBleRecovery()
                Timber.d("BLE recovery attempted")
            } catch (e: Exception) {
                Timber.e(e, "Failed to attempt BLE recovery")
            }
        }
    }

    /**
     * Force restart BLE scanning with proper backoff after a failure.
     */
    fun forceRestartScanning() {
        viewModelScope.launch {
            try {
                meshRepository.forceRestartScanning()
                Timber.d("BLE scanning force restarted")
            } catch (e: Exception) {
                Timber.e(e, "Failed to force restart BLE scanning")
            }
        }
    }

    /**
     * Clear BLE peer cache to allow re-discovery.
     */
    fun clearPeerCache() {
        viewModelScope.launch {
            try {
                meshRepository.clearPeerCache()
                Timber.d("BLE peer cache cleared")
            } catch (e: Exception) {
                Timber.e(e, "Failed to clear BLE peer cache")
            }
        }
    }

    /**
     * Get a specific message by ID.
     *
     * @param id The message ID
     * @return The message record or null if not found
     */
    fun getMessage(id: String): uniffi.api.MessageRecord? {
        return meshRepository.getMessage(id)
    }

    /**
     * Increment attempt count for a message in the retry tracking.
     * Called when a delivery attempt is made.
     */
    fun incrementAttemptCount(messageId: String) {
        viewModelScope.launch {
            try {
                meshRepository.incrementAttemptCount(messageId)
                Timber.d("Incremented attempt count for message $messageId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to increment attempt count for $messageId")
            }
        }
    }

    /**
     * Log a message delivery attempt for tracking purposes.
     *
     * @param messageId The message ID
     * @param attempt The attempt number
     * @param outcome The outcome ("success", "failed", etc.)
     */
    fun logMessageDeliveryAttempt(messageId: String, attempt: Int, outcome: String) {
        viewModelScope.launch {
            try {
                meshRepository.logMessageDeliveryAttempt(messageId, attempt, outcome)
                Timber.d("Logged delivery attempt for $messageId: attempt=$attempt, outcome=$outcome")
            } catch (e: Exception) {
                Timber.e(e, "Failed to log delivery attempt for $messageId")
            }
        }
    }

    /**
     * Record a connection failure for a specific multiaddress.
     * Used to track and deprecate failing routes.
     */
    fun recordConnectionFailure(multiaddr: String) {
        viewModelScope.launch {
            try {
                meshRepository.recordConnectionFailure(multiaddr)
                Timber.d("Recorded connection failure for $multiaddr")
            } catch (e: Exception) {
                Timber.e(e, "Failed to record connection failure for $multiaddr")
            }
        }
    }

    /**
     * Record a transport event for health tracking.
     *
     * @param transport The transport type (BLE, WiFi, etc.)
     * @param success Whether the transport operation succeeded
     * @param latencyMs Optional latency in milliseconds
     */
    fun recordTransportEvent(transport: String, success: Boolean, latencyMs: Long? = null) {
        viewModelScope.launch {
            try {
                meshRepository.recordTransportEvent(transport, success, latencyMs)
                Timber.d("Recorded transport event: $transport success=$success, latencyMs=$latencyMs")
            } catch (e: Exception) {
                Timber.e(e, "Failed to record transport event for $transport")
            }
        }
    }

    /**
     * Prime relay bootstrap connections for improved connectivity.
     * Pre-warms relay connections to reduce initial connection latency.
     */
    fun primeRelayBootstrapConnections() {
        viewModelScope.launch {
            try {
                meshRepository.primeRelayBootstrapConnectionsLegacy()
                Timber.i("Primed relay bootstrap connections")
            } catch (e: Exception) {
                Timber.e(e, "Failed to prime relay bootstrap connections")
            }
        }
    }

    /**
     * Observe peers flow for real-time peer list updates.
     * Used by PeerListScreen and other screens showing active peers.
     */
    fun observePeers(): kotlinx.coroutines.flow.Flow<List<String>> {
        return meshRepository.observePeers()
    }

    /**
     * Observe network stats flow for dashboard and settings display.
     * Provides real-time service statistics updates.
     */
    fun observeNetworkStats(): kotlinx.coroutines.flow.Flow<uniffi.api.ServiceStats> {
        return meshRepository.observeNetworkStats()
    }

    /**
     * Search contacts by query string.
     *
     * @param query The search query (peer ID, public key, or nickname)
     * @return List of matching contacts
     */
    fun searchContacts(query: String): List<uniffi.api.Contact> {
        return meshRepository.searchContacts(query)
    }

    /**
     * Update contact nickname at the repository level.
     *
     * @param peerId The peer ID
     * @param nickname The new nickname (nullable to remove)
     */
    fun setContactNickname(peerId: String, nickname: String?) {
        viewModelScope.launch {
            try {
                meshRepository.setContactNickname(peerId, nickname)
                Timber.d("Set contact nickname: $peerId -> $nickname")
            } catch (e: Exception) {
                Timber.e(e, "Failed to set contact nickname for $peerId")
            }
        }
    }

    /**
     * Update contact device ID for multi-device tracking.
     *
     * @param peerId The peer ID
     * @param deviceId The device ID to associate (nullable to clear)
     */
    fun updateContactDeviceId(peerId: String, deviceId: String?) {
        viewModelScope.launch {
            try {
                meshRepository.updateContactDeviceId(peerId, deviceId)
                Timber.d("Updated contact device ID: $peerId -> $deviceId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to update contact device ID for $peerId")
            }
        }
    }

    /**
     * Check if a message should be retried based on delivery state.
     *
     * @param messageId The message ID
     * @return True if message should be retried
     */
    fun shouldRetryMessage(messageId: String): Boolean {
        return meshRepository.shouldRetryMessage(messageId)
    }

    /**
     * Load pending outbox messages asynchronously.
     * Returns list of envelopes waiting to be sent.
     */
    suspend fun loadPendingOutboxAsync(): List<*> {
        return meshRepository.loadPendingOutboxAsync()
    }

    /**
     * Mark a message as corrupted in tracking cache.
     * Used when message processing encounters unrecoverable errors.
     *
     * @param messageId The message ID to mark as corrupted
     */
    fun markMessageCorrupted(messageId: String) {
        viewModelScope.launch {
            try {
                meshRepository.markMessageCorrupted(messageId)
                Timber.w("Message $messageId marked as corrupted")
            } catch (e: Exception) {
                Timber.e(e, "Failed to mark message $messageId as corrupted")
            }
        }
    }

    /**
     * Export diagnostics asynchronously to a file.
     * Returns the path to the exported diagnostics file.
     */
    suspend fun exportDiagnosticsAsync(): String {
        return meshRepository.exportDiagnosticsAsync()
    }

    // ========================================================================
    // BLE PRIVACY (mirrors iOS SettingsViewModel BLE rotation)
    // ========================================================================

    fun setBleRotationEnabled(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setBleRotationEnabled(enabled)
        }
    }

    fun setBleRotationIntervalSec(intervalSec: Int) {
        viewModelScope.launch {
            preferencesRepository.setBleRotationIntervalSec(intervalSec)
        }
    }

    /**
     * Resets all application data and preferences.
     */
    fun resetAllData() {
        viewModelScope.launch {
            try {
                _isLoading.value = true

                // Clear app-level preferences
                preferencesRepository.clearAll()

                // Reset mesh data (identity, history, etc)
                meshRepository.resetAllData()

                Timber.i("Application reset complete")
            } catch (e: Exception) {
                _error.value = "Failed to reset application: ${e.message}"
                Timber.e(e, "Reset failed")
            } finally {
                _isLoading.value = false
            }
        }
    }
}
