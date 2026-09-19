package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import java.security.SecureRandom
import javax.inject.Inject

/**
 * ViewModel for identity management.
 *
 * Handles identity creation, display, QR code generation,
 * and key export/backup operations.
 */
@HiltViewModel
class IdentityViewModel @Inject constructor(
    private val meshRepository: MeshRepository,
    private val identityCreationCoordinator: com.scmessenger.android.data.IdentityCreationCoordinator
) : ViewModel() {

    private val _identityInfo = MutableStateFlow<uniffi.api.IdentityInfo?>(meshRepository.identityInfo.value)
    val identityInfo: StateFlow<uniffi.api.IdentityInfo?> = _identityInfo.asStateFlow()

    val identityState: StateFlow<com.scmessenger.android.data.IdentityState> = identityCreationCoordinator.identityState

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = combine(
        _isLoading,
        identityCreationCoordinator.identityState
    ) { loading, state ->
        loading || state == com.scmessenger.android.data.IdentityState.Restoring || state == com.scmessenger.android.data.IdentityState.CachedPendingHydration
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    val isCreating: StateFlow<Boolean> = identityCreationCoordinator.identityState.map {
        it == com.scmessenger.android.data.IdentityState.Restoring
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    val error: StateFlow<String?> = identityCreationCoordinator.error

    private val _successMessage = MutableStateFlow<String?>(null)
    val successMessage: StateFlow<String?> = _successMessage.asStateFlow()

    private val _qrCodeData = MutableStateFlow<String?>(null)
    val qrCodeData: StateFlow<String?> = _qrCodeData.asStateFlow()

    val progressStage: StateFlow<IdentityProgressStage> = identityCreationCoordinator.progressStage

    val progressSubDetail: StateFlow<String?> = identityCreationCoordinator.progressSubDetail

    // Companion object: shared SecureRandom instance (CSPRNG) used to generate
    // the 256-bit salt for both the onboarding and in-settings entry points.
    // Even if the user does not engage with the entropy canvas, the identity
    // pipeline still receives a fresh random salt on every call.
    companion object {
        private val secureRandom = SecureRandom()

        /** Draw 32 bytes of fresh entropy from the platform CSPRNG. */
        fun generateSecureRandomSalt(): ByteArray {
            val salt = ByteArray(32)
            secureRandom.nextBytes(salt)
            return salt
        }
    }

    init {
        // P0_SHARED_IDENTITY: Immediately read the published StateFlow value
        // SYNCHRONOUSLY before any coroutines. This eliminates the race where
        // the UI renders "Restoring identity" because the collector hasn't
        // received the first value yet. StateFlow.value is always available.
        val published = meshRepository.identityInfo.value
        if (published != null && published.initialized) {
            _identityInfo.value = published
        }

        // Mirror the centralized meshRepository.identityInfo StateFlow for
        // ongoing updates (e.g., nickname change, service restart).
        viewModelScope.launch(Dispatchers.IO) {
            meshRepository.identityInfo.collect { info ->
                if (_identityInfo.value != info) {
                    _identityInfo.value = info
                }
            }
        }

        // Also kick a loadIdentity() to trigger an FFI read (which will publish
        // to meshRepository.identityInfo, propagating back here via the observer
        // above). loadIdentity() also handles the retry-with-backoff case where
        // the Rust core is still hydrating.
        loadIdentity()

        viewModelScope.launch(Dispatchers.Default) {
            _identityInfo.collect { info ->
                if (info?.initialized == true) {
                    _qrCodeData.value = getQrCodeData()
                } else {
                    _qrCodeData.value = null
                }
            }
        }

        // P0: Refresh identity from Rust core when service transitions to RUNNING,
        // replacing SharedPreferences cache with live data.
        var lastServiceState: uniffi.api.ServiceState? = null
        viewModelScope.launch(Dispatchers.IO) {
            meshRepository.serviceState.collect { state ->
                if (state == uniffi.api.ServiceState.RUNNING &&
                    lastServiceState != uniffi.api.ServiceState.RUNNING) {
                    Timber.d("IdentityViewModel: service -> RUNNING, force-refreshing identity")
                    loadIdentity(forceRefresh = true)
                }
                lastServiceState = state
            }
        }
    }

    /**
     * Load identity information.
     *
     * P0_ANDROID_QR_FIX: When the user navigates from Settings -> Identity QR, the
     * IdentityViewModel is constructed fresh. Its `init` calls `loadIdentity()` which
     * uses `getIdentityInfoNonBlocking()`. On a cold start, the ironCore may be null
     * or `core.getIdentityInfo()` may return null because the service is still spinning
     * up — even though the identity is fully on disk in SharedPreferences backup. The
     * old code returned null in that case and the user saw "Identity not initialized"
     * on the QR screen, even though the mesh was working and identity was real.
     *
     * The fix:
     *   1. Retry with exponential backoff for up to ~1.55s total if the first read
     *      returns null/uninitialized. The Rust core typically hydrates within 100-500ms
     *      on a warm start.
     *   2. As a defensive check, use `meshRepository.isIdentityInitialized()` to know
     *      whether the SharedPreferences backup exists. If no backup, identity was
     *      never created on this device — short-circuit, no retry.
     *   3. The `forceRefresh` flag (used on service RUNNING transition) bypasses the
     *      in-VM cache and re-reads from the Rust core.
     */
    fun loadIdentity(forceRefresh: Boolean = false) {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                _isLoading.value = true
                identityCreationCoordinator.clearError()

                // P0_ANDROID_QR_FIX: bounded retry-with-backoff so the QR screen
                // doesn't show "not initialized" during a normal cold start.
                val identity = readIdentityWithRetry(forceRefresh = forceRefresh)
                if (_identityInfo.value != identity) {
                    _identityInfo.value = identity
                }

                if (identity == null || !identity.initialized) {
                    Timber.w("P0_QR_FIX: Identity not initialized after retry loop")
                } else {
                    Timber.d("Identity loaded: ${identity.identityId ?: "Unknown"}")
                }
            } catch (e: Exception) {
                identityCreationCoordinator.setError("Failed to load identity: ${e.message}")
                Timber.e(e, "Failed to load identity")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * P0_ANDROID_QR_FIX: Read identity with bounded retry-with-backoff.
     *
     * Strategy:
     *   - First attempt: use `getIdentityInfoNonBlocking()` (no service init).
     *   - If null/uninitialized AND `isIdentityInitialized()` (backup check) says
     *     identity exists on disk, retry up to 5 times with exponential backoff
     *     (50ms, 100ms, 200ms, 400ms, 800ms = ~1.55s total). The Rust core typically
     *     hydrates from sled within the first 1-2 retries.
     *   - If `isIdentityInitialized()` returns false, identity was never created
     *     on this device — short-circuit immediately, no retry, render create form.
     */
    private suspend fun readIdentityWithRetry(forceRefresh: Boolean): uniffi.api.IdentityInfo? {
        // P0_SHARED_IDENTITY: Check the published StateFlow first — this catches
        // identity that was just created or already loaded by another ViewModel.
        val published = meshRepository.identityInfo.value
        if (published != null && published.initialized) {
            return published
        }
        val firstAttempt = meshRepository.getIdentityInfoNonBlocking()
        if (firstAttempt != null && firstAttempt.initialized) {
            return firstAttempt
        }
        // Identity not yet visible via non-blocking read. Check the backup before
        // retrying: if the SharedPreferences backup is missing, identity was never
        // created here and we should render the create form, not spin forever.
        val backupExists = try {
            meshRepository.isIdentityInitialized()
        } catch (e: Exception) {
            Timber.w(e, "P0_QR_FIX: isIdentityInitialized() threw during retry check")
            false
        }
        if (!backupExists) {
            Timber.d("P0_QR_FIX: no identity backup on disk; rendering create form")
            return firstAttempt // null or uninitialized
        }
        // Backup exists but the core returned null. Retry with backoff so the QR
        // screen doesn't show "not initialized" during a normal cold start.
        if (!forceRefresh && _identityInfo.value?.initialized == true) {
            // Already have initialized data in this VM; don't churn.
            return _identityInfo.value
        }
        val backoffMs = longArrayOf(100L, 200L, 400L, 800L, 1_000L, 1_500L)
        for ((index, delay) in backoffMs.withIndex()) {
            kotlinx.coroutines.delay(delay)
            val attempt = try {
                meshRepository.getIdentityInfoNonBlocking()
            } catch (e: Exception) {
                Timber.w(e, "P0_QR_FIX: retry $index threw")
                continue
            }
            if (attempt != null && attempt.initialized) {
                Timber.d("P0_QR_FIX: identity hydrated on retry $index after ${backoffMs.take(index + 1).sum()}ms")
                return attempt
            }
        }
        Timber.w("P0_QR_FIX: identity not visible from Rust core after retry loop; backup exists but core unhydrated")
        return firstAttempt
    }

    /**
     * Create a new identity (first-time setup).
     *
     * P0_ANDROID_IDENTITY_PROOF_OF_WORK: Emits 6 named stages via [progressStage]
     * so the UI can show high-level proof of work. The same stages are emitted
     * by [MainViewModel.createIdentity] so onboarding and in-settings share one
     * progress narrative.
     *
     * Always passes a fresh 32-byte SecureRandom salt to meshRepository.createIdentity
     * — fixes the "no random salt for settings launched identity generation"
     * regression where the in-settings path was silently dropping the salt parameter.
     */
    fun createIdentity(nickname: String? = null) {
        viewModelScope.launch {
            val success = identityCreationCoordinator.createIdentity(nickname ?: "")
            if (success) {
                _successMessage.value = "Identity created successfully"
                loadIdentity(forceRefresh = true)
            }
        }
    }

    /**
     * Refresh QR code data from core.
     */
    fun refreshQrCode() {
        viewModelScope.launch(Dispatchers.Default) {
            _qrCodeData.value = getQrCodeData()
        }
    }

    /**
     * Get QR code data for sharing identity.
     * Returns canonical identity export JSON so contact imports fully autofill.
     * Suspend function to avoid blocking Main thread on FFI calls.
     */
    suspend fun getQrCodeData(): String? {
        return withContext(Dispatchers.IO) {
            try {
                // P0_ANDROID_QR_FIX: Do not depend on the async _identityInfo StateFlow here.
                // Compose can call this before the StateFlow processes the upstream emission,
                // causing a race condition where QR data stays null forever.
                // getIdentityExportString() safely fetches directly from the authoritative core.
                val info = meshRepository.getIdentityInfoNonBlocking()
                if (info?.initialized != true) return@withContext null
                
                meshRepository.getIdentityExportString(minimalForQr = true, identityInfo = info)
            } catch (e: Exception) {
                Timber.e(e, "Failed to generate QR code data")
                null
            }
        }
    }

    /**
     * Clear success message.
     */
    fun clearSuccessMessage() {
        _successMessage.value = null
    }

    /**
     * Clear error state.
     */
    fun clearError() {
        identityCreationCoordinator.clearError()
    }
}
