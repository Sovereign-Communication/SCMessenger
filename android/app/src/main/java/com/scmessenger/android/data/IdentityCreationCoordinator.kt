package com.scmessenger.android.data

import android.content.Context
import com.scmessenger.android.ui.viewmodels.IdentityProgressStage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import java.security.SecureRandom
import javax.inject.Inject
import javax.inject.Singleton

enum class IdentityState {
    None,
    CachedPendingHydration,
    Restoring,
    Ready,
    CorruptNeedsUserAction,
    Failed
}

@Singleton
class IdentityCreationCoordinator @Inject constructor(
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    private val _identityState = MutableStateFlow<IdentityState>(IdentityState.None)
    val identityState: StateFlow<IdentityState> = _identityState.asStateFlow()

    private val _progressStage = MutableStateFlow<IdentityProgressStage>(IdentityProgressStage.Idle)
    val progressStage: StateFlow<IdentityProgressStage> = _progressStage.asStateFlow()

    private val _progressSubDetail = MutableStateFlow<String?>(null)
    val progressSubDetail: StateFlow<String?> = _progressSubDetail.asStateFlow()

    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    init {
        // Observe MeshRepository identityInfo changes
        scope.launch {
            meshRepository.identityInfo.collect { info ->
                val initialized = info?.initialized == true
                if (initialized) {
                    _identityState.value = IdentityState.Ready
                } else {
                    val isReallyInitialized = meshRepository.isIdentityInitialized()
                    if (!isReallyInitialized) {
                        // Safe regression: no identity exists on disk.
                        _identityState.value = IdentityState.None
                        _progressStage.value = IdentityProgressStage.Idle
                        _progressSubDetail.value = null
                        _error.value = null
                    } else if (_identityState.value != IdentityState.Ready) {
                        determineInitialState()
                    }
                }
            }
        }
        determineInitialState()
    }

    fun determineInitialState() {
        val initialized = meshRepository.isIdentityInitialized()
        if (initialized) {
            // Trust cached SharedPreferences fields first — they're authoritative
            // for UI display and available before ironCore finishes starting up.
            val cached = meshRepository.readCachedIdentityFields()
            if (cached?.initialized == true) {
                _identityState.value = IdentityState.Ready
                return
            }
            val currentInfo = meshRepository.getIdentityInfoNonBlocking()
            if (currentInfo?.initialized == true) {
                _identityState.value = IdentityState.Ready
            } else {
                _identityState.value = IdentityState.CachedPendingHydration
            }
        } else {
            _identityState.value = IdentityState.None
        }
    }


    suspend fun createIdentity(nickname: String, explicitSalt: ByteArray? = null): Boolean {
        val trimmed = nickname.trim()
        if (trimmed.isEmpty()) {
            _error.value = "Nickname is required"
            return false
        }

        _error.value = null
        _progressStage.value = IdentityProgressStage.PreparingStorage
        _identityState.value = IdentityState.Restoring // Mark as preparing/restoring

        return withContext(Dispatchers.IO) {
            val salt = explicitSalt ?: generateSecureRandomSalt()

            try {
                meshRepository.createIdentity(salt) { event, subDetail ->
                    _progressStage.value = when (event) {
                        is IdentityCreationEvent.PreparingStorage -> IdentityProgressStage.PreparingStorage
                        is IdentityCreationEvent.GeneratingSalt -> IdentityProgressStage.GeneratingSalt
                        is IdentityCreationEvent.GeneratingKeypair -> IdentityProgressStage.GeneratingKeypair
                        is IdentityCreationEvent.ComputingFingerprint -> IdentityProgressStage.ComputingFingerprint
                        is IdentityCreationEvent.PersistingToStorage -> IdentityProgressStage.PersistingToStorage
                        is IdentityCreationEvent.VerifyingIdentity -> IdentityProgressStage.VerifyingIdentity
                    }
                    _progressSubDetail.value = subDetail
                }

                meshRepository.setNickname(trimmed)

                // Verify nickname persistence
                var info = meshRepository.getIdentityInfoSync()
                if (info?.nickname.isNullOrBlank()) {
                    meshRepository.setNickname(trimmed)
                    info = meshRepository.getIdentityInfoSync()
                }

                // Verify initialization
                var initialized = meshRepository.isIdentityInitialized()
                var retries = 0
                while (!initialized && retries < 5) {
                    kotlinx.coroutines.delay(100)
                    initialized = meshRepository.isIdentityInitialized()
                    retries++
                }

                if (initialized) {
                    preferencesRepository.setOnboardingCompleted(true)
                    preferencesRepository.setInstallChoiceCompleted(true)
                    preferencesRepository.setIdentityNickname(trimmed)
                    _identityState.value = IdentityState.Ready
                    return@withContext true
                } else {
                    _error.value = "Identity was created but verification failed."
                    _identityState.value = IdentityState.Failed
                    return@withContext false
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to create identity in coordinator")
                _error.value = e.message ?: "Unknown error creating identity"
                _identityState.value = IdentityState.Failed
                return@withContext false
            } finally {
                kotlinx.coroutines.delay(600)
                _progressStage.value = IdentityProgressStage.Idle
                _progressSubDetail.value = null
            }
        }
    }

    fun clearError() {
        _error.value = null
    }

    fun setError(msg: String?) {
        _error.value = msg
    }

    private fun generateSecureRandomSalt(): ByteArray {
        val salt = ByteArray(32)
        SecureRandom().nextBytes(salt)
        return salt
    }
}
