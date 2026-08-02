package com.scmessenger.android.ui.viewmodels

import android.content.Context
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import com.scmessenger.android.utils.ContactImportParseResult
import com.scmessenger.android.utils.DeepLinkValidator
import com.scmessenger.android.utils.PeerIdValidator
import com.scmessenger.android.utils.parseContactImportPayload
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import com.scmessenger.android.utils.StorageManager
import java.security.SecureRandom
import java.util.concurrent.atomic.AtomicBoolean
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val meshRepository: MeshRepository,
    private val preferencesRepository: PreferencesRepository,
    private val identityCreationCoordinator: com.scmessenger.android.data.IdentityCreationCoordinator,
    @ApplicationContext private val context: Context
) : ViewModel() {

    private val _isReady = MutableStateFlow(false)
    val isReady = _isReady.asStateFlow()
    val hasIdentity = _isReady.asStateFlow()

    private val _onboardingCompleted = MutableStateFlow(false)
    val onboardingCompleted = _onboardingCompleted.asStateFlow()

    private val _installChoiceCompleted = MutableStateFlow(false)
    val installChoiceCompleted = _installChoiceCompleted.asStateFlow()

    // showOnboarding is true if NOT ready AND NOT install choice completed
    val showOnboarding = combine(
        _isReady,
        _installChoiceCompleted
    ) { ready, choiceCompleted ->
        !ready && !choiceCompleted
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.Eagerly,
        initialValue = !_isReady.value && !_installChoiceCompleted.value
    )

    val isCreatingIdentity: StateFlow<Boolean> = identityCreationCoordinator.identityState.map {
        it == com.scmessenger.android.data.IdentityState.Restoring || it == com.scmessenger.android.data.IdentityState.CachedPendingHydration
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    val identityError: StateFlow<String?> = identityCreationCoordinator.error

    val identityProgressStage: StateFlow<IdentityProgressStage> = identityCreationCoordinator.progressStage

    val identityProgressSubDetail: StateFlow<String?> = identityCreationCoordinator.progressSubDetail

    private val _importError = MutableStateFlow<String?>(null)
    val importError = _importError.asStateFlow()

    private val _importSuccess = MutableStateFlow(false)
    val importSuccess = _importSuccess.asStateFlow()

    val identityInfo: uniffi.api.IdentityInfo?
        get() = meshRepository.getIdentityInfoNonBlocking()

    private val _isStorageLow = MutableStateFlow(false)
    val isStorageLow = _isStorageLow.asStateFlow()

    private val _availableStorageMB = MutableStateFlow(0L)
    val availableStorageMB = _availableStorageMB.asStateFlow()

    private val _pendingDeepLink = MutableStateFlow<DeepLinkData?>(null)
    val pendingDeepLink: StateFlow<DeepLinkData?> = _pendingDeepLink.asStateFlow()

    private val _pendingRequestsInbox = MutableStateFlow<String?>(null)
    val pendingRequestsInbox: StateFlow<String?> = _pendingRequestsInbox.asStateFlow()

    private val _themeMode = MutableStateFlow(PreferencesRepository.ThemeMode.SYSTEM)
    val themeMode: StateFlow<PreferencesRepository.ThemeMode> = _themeMode.asStateFlow()

    // Guards against concurrent refreshIdentityState() calls that spam FFI
    // and drop 160+ UI frames during startup.
    private val isRefreshing = AtomicBoolean(false)

    init {
        Timber.d("MainViewModel init")
        refreshStorageStatus()

        // Observe preferences
        viewModelScope.launch {
            preferencesRepository.onboardingCompleted.collect { completed ->
                Timber.d("Preference onboardingCompleted: $completed")
                _onboardingCompleted.value = completed
            }
        }
        viewModelScope.launch {
            preferencesRepository.installChoiceCompleted.collect { completed ->
                Timber.d("Preference installChoiceCompleted: $completed")
                _installChoiceCompleted.value = completed
            }
        }
        viewModelScope.launch {
            preferencesRepository.themeMode.collect { mode ->
                Timber.d("Preference themeMode: $mode")
                _themeMode.value = mode
            }
        }

        // Auto-refresh identity state when service transitions to RUNNING (lazy start).
        // Track prior state to avoid triggering refreshIdentityState on repeated
        // RUNNING emissions, which can cascade through _isReady → MeshApp recomposition.
        var lastServiceState: uniffi.api.ServiceState? = null
        viewModelScope.launch {
            meshRepository.serviceState.collect { state ->
                Timber.d("MeshRepository service state: $state")
                if (state == uniffi.api.ServiceState.RUNNING &&
                    lastServiceState != uniffi.api.ServiceState.RUNNING) {
                    refreshIdentityState()
                }
                lastServiceState = state
            }
        }

        // P0_SHARED_IDENTITY: observe the centralized identityInfo flow. This
        // ensures _isReady (which drives hasIdentity → showOnboarding → MeshApp
        // nav graph) flips true the moment the repo knows about the identity,
        // even if it happens via a different code path (e.g., IdentityViewModel's
        // own loadIdentity call publishes first). This eliminates the race where
        // a freshly created VM reads hasIdentity=false because its own
        // refreshIdentityState hasn't run yet.
        viewModelScope.launch {
            meshRepository.identityInfo.collect { info ->
                val initialized = info?.initialized == true
                if (_isReady.value != initialized) {
                    Timber.d("P0_SHARED_IDENTITY: meshRepository.identityInfo reports initialized=$initialized; updating _isReady")
                    _isReady.value = initialized
                }
            }
        }

        refreshIdentityState()
    }

    fun grantConsent() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                meshRepository.grantConsent()
                Timber.i("Consent granted via MainViewModel")
            } catch (e: Exception) {
                Timber.e(e, "Failed to grant consent")
            }
        }
    }

    fun refreshIdentityState() {
        // Drop duplicate FFI requests: concurrent refreshIdentityState() calls
        // spike isIdentityInitialized() 4-5×, dropping 160+ UI frames at startup.
        if (!isRefreshing.compareAndSet(false, true)) {
            Timber.d("refreshIdentityState() skipped — already refreshing")
            return
        }
        viewModelScope.launch(Dispatchers.IO) {
            try {
                Timber.d("refreshIdentityState() called")
                val initialized = meshRepository.isIdentityInitialized()
                Timber.d("Identity initialized state: $initialized")
                identityCreationCoordinator.clearError()
                _isReady.value = initialized

                if (initialized) {
                    if (!_installChoiceCompleted.value) {
                        Timber.d("Identity is initialized but install choice not completed, fixing preference...")
                        preferencesRepository.setInstallChoiceCompleted(true)
                    }
                    if (!_onboardingCompleted.value) {
                        Timber.d("Identity is initialized but onboarding not completed, fixing preference...")
                        preferencesRepository.setOnboardingCompleted(true)
                    }
                }
            } finally {
                isRefreshing.set(false)
            }
        }
    }

    /**
     * Create a new identity with retry logic for _isReady verification.
     */
    fun createIdentity(nickname: String, salt: ByteArray? = null) {
        viewModelScope.launch {
            val success = identityCreationCoordinator.createIdentity(nickname, salt)
            if (success) {
                refreshIdentityState()
            }
        }
    }

    fun clearIdentityError() {
        identityCreationCoordinator.clearError()
    }

    fun refreshStorageStatus() {
        viewModelScope.launch(Dispatchers.IO) {
            val available = meshRepository.getAvailableStorageMB()
            _availableStorageMB.value = available
            _isStorageLow.value = StorageManager.isStorageStateCritical(context)
            Timber.d("Storage refreshed: $available MB available (Low=${_isStorageLow.value})")
        }
    }

    /**
     * Mirror of the public-key validation inside
     * [com.scmessenger.android.data.MeshRepository.addContact]. That function
     * returns Unit and drops invalid contacts silently, so callers must check
     * the same condition themselves to avoid reporting success for a contact
     * that was never stored.
     */
    internal fun isValidPublicKeyHex(key: String?): Boolean {
        val trimmed = key?.trim() ?: return false
        if (trimmed.length != 64) return false
        return trimmed.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }
    }

    fun importContact(jsonString: String) {
        viewModelScope.launch {
            try {
                _importError.value = null
                _importSuccess.value = false
                when (val parsed = parseContactImportPayload(jsonString)) {
                    is ContactImportParseResult.Invalid -> {
                        _importError.value = parsed.reason
                    }
                    is ContactImportParseResult.Valid -> {
                        val payload = parsed.payload
                        val notes = payload.libp2pPeerId?.let { pid ->
                            buildString {
                                append("libp2p_peer_id:$pid")
                                if (payload.listeners.isNotEmpty()) {
                                    append(";listeners:${payload.listeners.joinToString(",")}")
                                }
                            }
                        }
                        // Store contact with public_key as the canonical peerId.
                        val contact = uniffi.api.Contact(
                            peerId = payload.publicKey,
                            nickname = payload.nickname,
                            localNickname = null,
                            publicKey = payload.publicKey,
                            addedAt = (System.currentTimeMillis() / 1000).toULong(),
                            lastSeen = null,
                            notes = notes,
                            lastKnownDeviceId = null,
                            verifiedAt = null,
                            isTombstone = false
                        )
                        // SECURITY: MeshRepository.addContact returns Unit and
                        // silently drops a contact whose public key is empty,
                        // not 64 chars, or non-hex. Without mirroring that
                        // check here the import reported success -- and dialed
                        // the payload's addresses -- for a contact that was
                        // never stored. Validate at the boundary so failure is
                        // observable and nothing is dialed on a rejected import.
                        if (!isValidPublicKeyHex(payload.publicKey)) {
                            Timber.e("Contact import rejected: invalid public key")
                            _importError.value = "Invalid public key in contact payload"
                            return@launch
                        }
                        meshRepository.addContact(contact)
                        Timber.i("Contact imported: ${payload.publicKey.take(8)}...")
                        if (!payload.libp2pPeerId.isNullOrEmpty() && payload.listeners.isNotEmpty()) {
                            meshRepository.connectToPeer(payload.libp2pPeerId, payload.listeners)
                        }
                        _importSuccess.value = true
                    }
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to import contact")
                _importError.value = "Failed to import: ${e.message}"
            }
        }
    }

    fun clearImportState() {
        _importError.value = null
        _importSuccess.value = false
    }

    fun skipOnboardingForRelayOnlyInstall() {
        viewModelScope.launch {
            Timber.i("Skipping onboarding for relay-only install")
            preferencesRepository.setInstallChoiceCompleted(true)
            preferencesRepository.setOnboardingCompleted(true)
        }
    }

    fun handleDeepLink(uri: Uri) {
        if (uri.scheme != "scmessenger" || uri.host !in setOf("invite", "add")) {
            Timber.w("Ignoring unsupported deep link: $uri")
            return
        }
        val publicKey = uri.getQueryParameter("public_key")?.trim()
        if (publicKey.isNullOrBlank()) {
            Timber.w("Deep link missing public_key: $uri")
            return
        }
        if (!publicKey.matches(Regex("^[0-9a-fA-F]{64}$"))) {
            Timber.w("Deep link has invalid public_key; ignoring: $uri")
            return
        }
        val rawRoutePeerId = uri.getQueryParameter("libp2p_peer_id")?.trim()
            ?: uri.getQueryParameter("peer_id")?.trim()
        val routePeerId = rawRoutePeerId?.takeIf { PeerIdValidator.isLibp2pPeerId(it) }
        if (!rawRoutePeerId.isNullOrBlank() && routePeerId == null) {
            Timber.w("Deep link has invalid libp2p peer ID; suppressing automatic dial: $rawRoutePeerId")
        }
        // Parse multiaddrs from multiple query params: 'listeners' (repeated),
        // 'connection_hints', 'listener', and 'bootstrap' (single, comma-separated).
        // The APK share flow uses 'bootstrap' in the download URL.
        val rawListeners = (
            uri.getQueryParameters("listeners") +
                uri.getQueryParameters("connection_hints") +
                uri.getQueryParameters("listener") +
                listOfNotNull(uri.getQueryParameter("bootstrap"))
        )
            .flatMap { it.split(',') }
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .distinct()

        // Validate multiaddrs to prevent attacker-chosen addresses from untrusted QR codes.
        // SECURITY: This is a new attack surface -- validation is critical.
        val deviceIp = meshRepository.getLocalIpAddress()
        val listeners = if (routePeerId != null) {
            DeepLinkValidator.sanitizeDeepLinkMultiaddrs(rawListeners, deviceIp)
        } else {
            emptyList()
        }

        val data = DeepLinkData(
            publicKey = publicKey,
            peerId = routePeerId,
            nickname = uri.getQueryParameter("nickname")?.trim(),
            identityId = uri.getQueryParameter("identity_id")?.trim(),
            libp2pPeerId = routePeerId,
            listeners = listeners
        )
        Timber.i("Deep link parsed: peerId=${data.peerId}, nickname=${data.nickname}, listeners=${data.listeners.size}")
        _pendingDeepLink.value = data

        // TODO: Wire auto-dial via MeshRepository.connectToPeer(peerId, addresses)
        // once validation is reviewed and approved by the operator.
        // For now, only parse and expose via DeepLinkData.listeners.
    }

    fun consumeDeepLink(): DeepLinkData? {
        val data = _pendingDeepLink.value
        _pendingDeepLink.value = null
        return data
    }

    fun navigateToRequestsInbox(peerId: String? = null) {
        _pendingRequestsInbox.value = peerId
    }

    fun consumeRequestsInboxNav(): String? {
        val peerId = _pendingRequestsInbox.value
        _pendingRequestsInbox.value = null
        return peerId
    }
}

data class DeepLinkData(
    val publicKey: String,
    val peerId: String?,
    val nickname: String?,
    val identityId: String?,
    val libp2pPeerId: String?,
    val listeners: List<String>
)
