package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.PeerEvent
import com.scmessenger.android.utils.ContactImportParseResult
import com.scmessenger.android.utils.PeerIdValidator
import com.scmessenger.android.utils.parseContactImportPayload
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import timber.log.Timber
import javax.inject.Inject

/** A peer discovered on the mesh but not yet saved as a contact. */
data class NearbyPeer(
    val peerId: String,
    val publicKey: String? = null,
    val nickname: String? = null,
    val blePeerId: String? = null,
    val libp2pPeerId: String? = null,
    val listeners: List<String> = emptyList(),
    val isOnline: Boolean = true,
    val transport: com.scmessenger.android.service.TransportType? = null
) {
    val displayName: String get() = nickname?.takeIf { it.isNotBlank() } ?: peerId.take(16)
    val hasFullIdentity: Boolean get() = publicKey != null
}

/**
 * ViewModel for the contacts screen.
 *
 * Manages contact list, search, and CRUD operations.
 */
@HiltViewModel
class ContactsViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {
    // Reduced from 30s to 5s to fix Issue #6: Gratuitous nearby entries persistence
    // Peers should be removed promptly after disconnect to avoid confusing UX
    private val nearbyDisconnectGraceMs = 5_000L
    private val pendingNearbyRemovalJobs = mutableMapOf<String, kotlinx.coroutines.Job>()
    
    // Debounce mechanism for nickname updates (NICKNAME-CRASH-001)
    private val nicknameDebounceMs = 500L
    private val pendingNicknameJobs = mutableMapOf<String, Job>()

    // All contacts
    private val _contacts = MutableStateFlow<List<uniffi.api.Contact>>(emptyList())
    val contacts: StateFlow<List<uniffi.api.Contact>> = _contacts.asStateFlow()

    // Search query
    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    // Filtered contacts based on search
    val filteredContacts = combine(contacts, searchQuery) { contacts, query ->
        if (query.isBlank()) {
            contacts
        } else {
            val needle = query.trim()
            contacts.filter { contact ->
                contact.peerId.contains(needle, ignoreCase = true) ||
                    contact.publicKey.contains(needle, ignoreCase = true) ||
                    (contact.nickname?.contains(needle, ignoreCase = true) == true) ||
                    (contact.localNickname?.contains(needle, ignoreCase = true) == true) ||
                    (contact.notes?.contains(needle, ignoreCase = true) == true)
            }
        }
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = emptyList()
    )

    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    // Peers discovered on the mesh but not yet saved as contacts.
    private val _nearbyPeers = MutableStateFlow<List<NearbyPeer>>(emptyList())
    val nearbyPeers: StateFlow<List<NearbyPeer>> = _nearbyPeers.asStateFlow()

    // Peer IDs the user dismissed from the nearby list. Live discovery events for
    // these peers are suppressed until the next rescan, which re-checks whether
    // they're still around (see refreshDiscovery / dismissNearbyPeer).
    private val _dismissedPeerIds = MutableStateFlow<Set<String>>(emptySet())

    // Nearby rescan state — true while a rescan is actively running
    private val _isScanning = MutableStateFlow(false)
    val isScanning: StateFlow<Boolean> = _isScanning.asStateFlow()
    private var scanStartTimeMs = 0L

    init {
        loadContacts()
        observeNearbyPeers()
        observeServiceState()
        viewModelScope.launch {
            // Contacts can open after discovery already happened; replay cached identities
            // so Nearby stays aligned with Dashboard/Settings discovery state.
            kotlinx.coroutines.delay(100)
            meshRepository.replayDiscoveredPeerEvents()
        }
    }

    private fun normalizeNickname(value: String?): String? {
        return value?.trim()?.takeIf { it.isNotEmpty() }
    }

    private fun isSyntheticFallbackNickname(value: String?): Boolean {
        val normalized = normalizeNickname(value)?.lowercase() ?: return false
        return normalized.startsWith("peer-")
    }

    private fun selectAuthoritativeNickname(incoming: String?, existing: String?): String? {
        val incomingNormalized = normalizeNickname(incoming)
        val existingNormalized = normalizeNickname(existing)

        val incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        val existingSynthetic = isSyntheticFallbackNickname(existingNormalized)

        return when {
            incomingNormalized == null && existingSynthetic -> null
            incomingNormalized == null -> existingNormalized
            incomingSynthetic && existingNormalized == null -> null
            incomingSynthetic && existingSynthetic -> null
            incomingSynthetic -> existingNormalized
            existingSynthetic -> incomingNormalized
            else -> incomingNormalized
        }
    }

    // Identity validation logic centralized in PeerIdValidator

    private fun isBlePeerId(value: String?): Boolean {
        val normalized = value?.trim().orEmpty()
        if (normalized.isEmpty()) return false
        return runCatching { java.util.UUID.fromString(normalized) }.isSuccess
    }

    private fun selectStablePeerId(incomingPeerId: String, existingPeerId: String?): String {
        val incoming = incomingPeerId.trim()
        val existing = existingPeerId?.trim().orEmpty()
        if (existing.isEmpty() || existing == incoming) return incoming

        val incomingIsLibp2p = PeerIdValidator.isLibp2pPeerId(incoming)
        val existingIsLibp2p = PeerIdValidator.isLibp2pPeerId(existing)
        val incomingIsIdentity = PeerIdValidator.isIdentityId(incoming)
        val existingIsIdentity = PeerIdValidator.isIdentityId(existing)
        val incomingIsBle = isBlePeerId(incoming)
        val existingIsBle = isBlePeerId(existing)

        return when {
            existingIsIdentity && incomingIsLibp2p -> existing
            incomingIsIdentity && existingIsLibp2p -> incoming
            existingIsBle && !incomingIsBle -> incoming
            !existingIsBle && incomingIsBle -> existing
            else -> incoming
        }
    }

    /**
     * Check if a nearby peer and an identity discovery event represent the same entity.
     * Uses public key matching as the primary key to handle ID format mismatches:
     * - Identity/Discovery: SHA-256(publicKey)
     * - Storage/Database: Hash(pubkey+identity)
     * - Transport: LibP2P Peer ID
     *
     * @param peer Existing nearby peer
     * @param event New identity discovery event
     * @return True if they represent the same entity
     */
    private fun isSameNearbyIdentity(peer: NearbyPeer, event: PeerEvent.IdentityDiscovered): Boolean {
        // Primary key: public key matching (most reliable across ID schemes)
        val sameByPublicKey = peer.publicKey?.let { pk ->
            PeerIdValidator.isSame(pk, event.publicKey)
        } ?: false
        
        if (sameByPublicKey) return true
        
        // Secondary: ID-based matching (for cases where public key may not be available)
        val incomingPeerId = PeerIdValidator.normalize(event.peerId)
        val incomingLibp2p = event.libp2pPeerId?.let { PeerIdValidator.normalize(it) }.orEmpty()
        val incomingBle = event.blePeerId?.trim().orEmpty()
        val peerLibp2p = peer.libp2pPeerId?.let { PeerIdValidator.normalize(it) }.orEmpty()
        val peerBle = peer.blePeerId?.trim().orEmpty()

        val sameById = PeerIdValidator.isSame(peer.peerId, incomingPeerId) ||
            (incomingLibp2p.isNotEmpty() && (
                PeerIdValidator.isSame(peer.peerId, incomingLibp2p) ||
                    PeerIdValidator.isSame(peerLibp2p, incomingLibp2p)
                )) ||
            (incomingBle.isNotEmpty() && (
                peer.peerId == incomingBle ||
                    peerBle == incomingBle
                ))
        
        return sameById
    }
    
    /**
     * Check if a nearby peer matches a contact using comprehensive identity matching.
     * Uses public key as primary key to handle ID format mismatches.
     *
     * @param nearby Nearby peer to check
     * @param contact Contact to compare against
     * @return True if they represent the same entity
     */
    private fun isNearbyPeerContact(nearby: NearbyPeer, contact: uniffi.api.Contact): Boolean {
        // Primary key: public key matching (most reliable across ID schemes)
        val sameByPublicKey = nearby.publicKey?.let { npk ->
            PeerIdValidator.isSame(npk, contact.publicKey)
        } ?: false
        
        if (sameByPublicKey) return true
        
        // Secondary: ID-based matching
        val sameByPeerId = PeerIdValidator.isSame(nearby.peerId, contact.peerId)
        val sameByLibp2p = nearby.libp2pPeerId?.let { PeerIdValidator.isSame(it, contact.peerId) } ?: false
        val sameByBle = nearby.blePeerId != null && nearby.blePeerId == contact.peerId
        
        return sameByPeerId || sameByLibp2p || sameByBle
    }

    /** True if any of the given (nullable) peer identifiers matches a dismissed peer. */
    private fun isDismissed(vararg ids: String?): Boolean {
        if (_dismissedPeerIds.value.isEmpty()) return false
        return ids.filterNotNull().any { id -> _dismissedPeerIds.value.any { PeerIdValidator.isSame(it, id) } }
    }

    /**
     * Hide a nearby peer from the list without saving or blocking it.
     *
     * Dismissed peers stop reappearing from live background discovery events until
     * the user rescans (see [refreshDiscovery]), which treats a rescan as an explicit
     * request to recheck whether dismissed peers are still nearby.
     */
    fun dismissNearbyPeer(peer: NearbyPeer) {
        val ids = listOfNotNull(peer.peerId, peer.libp2pPeerId, peer.blePeerId).toSet()
        _dismissedPeerIds.value = _dismissedPeerIds.value + ids
        _nearbyPeers.value = _nearbyPeers.value.filterNot { it.peerId == peer.peerId }
        Timber.d("Dismissed nearby peer: ${peer.peerId.take(16)}")
    }

    /**
     * Subscribe to MeshEventBus peer events.
     * Adds newly discovered peers to nearbyPeers if they aren't already contacts.
     * Removes peers from nearbyPeers when they disconnect.
     */
    private fun observeNearbyPeers() {
        viewModelScope.launch {
            MeshEventBus.peerEvents.collect { event ->
                when (event) {
                    is PeerEvent.IdentityDiscovered -> {
                        // Never surface bootstrap relay/headless nodes in the Contacts nearby list.
                        val isRelay = meshRepository.isBootstrapRelayPeer(event.peerId) ||
                            (event.libp2pPeerId?.let { meshRepository.isBootstrapRelayPeer(it) } ?: false)
                        if (isRelay) return@collect

                        cancelPendingNearbyRemoval(event.peerId)
                        cancelPendingNearbyRemoval(event.libp2pPeerId)
                        cancelPendingNearbyRemoval(event.blePeerId)

                        // Check both cached contacts and database directly to avoid race conditions
                        // on startup where contacts are still loading
                        // Use comprehensive identity matching with public key as primary key
                        val alreadyContact = _contacts.value.any { contact ->
                            isNearbyPeerContact(NearbyPeer(
                                peerId = event.peerId,
                                publicKey = event.publicKey,
                                libp2pPeerId = event.libp2pPeerId,
                                blePeerId = event.blePeerId
                            ), contact)
                        } ||
                        // Direct database queries for reliability even during startup
                        (meshRepository.getContact(event.peerId) != null ||
                            meshRepository.getContact(event.publicKey) != null ||
                            (event.libp2pPeerId?.let { meshRepository.getContact(it) } != null))

                        if (alreadyContact) {
                            Timber.d("Peer already saved as contact: ${event.peerId.take(16)}, skipping nearby")
                            // Federated nickname/route hints can update in repository upsert;
                            // refresh saved contacts so local UI reflects latest values.
                            loadContacts()
                            return@collect
                        }

                        if (isDismissed(event.peerId, event.libp2pPeerId, event.blePeerId)) {
                            return@collect
                        }

                        val current = _nearbyPeers.value.toMutableList()
                        val matches = current.filter { peer -> isSameNearbyIdentity(peer, event) }
                        val existing = matches.maxByOrNull { peer ->
                            val hasNickname = if (normalizeNickname(peer.nickname) != null) 2 else 0
                            val hasStableId = if (!PeerIdValidator.isLibp2pPeerId(peer.peerId)) 1 else 0
                            hasNickname + hasStableId
                        }
                        if (matches.isNotEmpty()) {
                            current.removeAll(matches.toSet())
                        }
                        cancelPendingNearbyRemoval(existing?.peerId)

                        val resolvedPeerId = selectStablePeerId(event.peerId, existing?.peerId)
                        val resolvedLibp2pPeerId = event.libp2pPeerId?.trim()?.takeIf { it.isNotEmpty() }
                            ?: existing?.libp2pPeerId?.trim()?.takeIf { it.isNotEmpty() }
                            ?: event.peerId.takeIf { PeerIdValidator.isLibp2pPeerId(it) }
                        val resolvedBlePeerId = event.blePeerId?.trim()?.takeIf { it.isNotEmpty() }
                            ?: existing?.blePeerId?.trim()?.takeIf { it.isNotEmpty() }
                        // Best-effort transport inference: BLE wins if a BLE id is present,
                        // else TCP_MDNS for private LAN listeners, else INTERNET.
                        val resolvedTransport: com.scmessenger.android.service.TransportType? =
                            existing?.transport
                                ?: if (resolvedBlePeerId != null) {
                                    com.scmessenger.android.service.TransportType.BLE
                                } else if ((event.listeners.isNotEmpty() ||
                                    (existing?.listeners?.isNotEmpty() == true))) {
                                    val listeners = event.listeners.ifEmpty { existing?.listeners ?: emptyList() }
                                    if (listeners.any { addr -> isPrivateLanAddress(addr) }) {
                                        com.scmessenger.android.service.TransportType.TCP_MDNS
                                    } else {
                                        com.scmessenger.android.service.TransportType.INTERNET
                                    }
                                } else null
                        val updated = NearbyPeer(
                            peerId = resolvedPeerId,
                            publicKey = event.publicKey,
                            nickname = selectAuthoritativeNickname(event.nickname, existing?.nickname),
                            blePeerId = resolvedBlePeerId,
                            libp2pPeerId = resolvedLibp2pPeerId,
                            listeners = if (event.listeners.isNotEmpty()) event.listeners else (existing?.listeners ?: emptyList()),
                            isOnline = true,
                            transport = resolvedTransport
                        )
                        current.add(updated)
                        _nearbyPeers.value = current
                    }
                    is PeerEvent.Discovered -> {
                        // Don't surface bootstrap relay nodes in the nearby list.
                        if (meshRepository.isBootstrapRelayPeer(event.peerId)) return@collect

                        // Check both cached contacts and database directly
                        // Use comprehensive identity matching with public key as primary key
                        val alreadyContact = _contacts.value.any { contact ->
                            isNearbyPeerContact(NearbyPeer(
                                peerId = event.peerId,
                                libp2pPeerId = event.peerId.takeIf { PeerIdValidator.isLibp2pPeerId(it) }
                            ), contact)
                        } || (meshRepository.getContact(event.peerId) != null)

                        cancelPendingNearbyRemoval(event.peerId)
                        val current = _nearbyPeers.value.toMutableList()
                        val existingIdx = current.indexOfFirst {
                            PeerIdValidator.isSame(it.peerId, event.peerId) ||
                            it.libp2pPeerId?.let { libp -> PeerIdValidator.isSame(libp, event.peerId) } ?: false
                        }
                        if (existingIdx >= 0) {
                            // Update isOnline, and refine transport if we now have a definite value.
                            val existing = current[existingIdx]
                            val refinedTransport = existing.transport ?: event.transport
                            current[existingIdx] = existing.copy(isOnline = true, transport = refinedTransport)
                            _nearbyPeers.value = current
                        } else if (!alreadyContact && !isDismissed(event.peerId)) {
                            _nearbyPeers.value = current + NearbyPeer(
                                peerId = event.peerId,
                                isOnline = true,
                                transport = event.transport
                            )
                        }
                    }
                    is PeerEvent.Disconnected -> {
                        val current = _nearbyPeers.value.toMutableList()
                        var changed = false
                        current.indices.forEach { idx ->
                            val peer = current[idx]
                            if (PeerIdValidator.isSame(peer.peerId, event.peerId) ||
                                peer.libp2pPeerId?.let { PeerIdValidator.isSame(it, event.peerId) } ?: false) {
                                if (peer.isOnline) {
                                    current[idx] = peer.copy(isOnline = false)
                                    changed = true
                                }
                            }
                        }
                        if (changed) {
                            _nearbyPeers.value = current
                            scheduleNearbyRemoval(event.peerId)
                        }
                    }
                    else -> Unit
                }
            }
        }
    }

    private fun cancelPendingNearbyRemoval(peerId: String?) {
        val id = peerId?.trim().orEmpty()
        if (id.isEmpty()) return
        pendingNearbyRemovalJobs.remove(id)?.cancel()
    }

    private fun scheduleNearbyRemoval(peerId: String) {
        cancelPendingNearbyRemoval(peerId)
        pendingNearbyRemovalJobs[peerId] = viewModelScope.launch {
            kotlinx.coroutines.delay(nearbyDisconnectGraceMs)
            _nearbyPeers.value = _nearbyPeers.value.filterNot {
                PeerIdValidator.isSame(it.peerId, peerId) ||
                it.libp2pPeerId?.let { libp -> PeerIdValidator.isSame(libp, peerId) } ?: false
            }
            pendingNearbyRemovalJobs.remove(peerId)
        }
    }

    /**
     * Observe mesh service state and clear nearby peers immediately when service stops.
     * This fixes AND-STALE-PEER-001: Gratuitous Nearby Entries Persistence.
     *
     * When the mesh service stops (e.g., user toggles off, app goes to background),
     * nearby peers should be removed immediately rather than waiting for individual
     * disconnect events that may arrive late or not at all.
     */
    private fun observeServiceState() {
        viewModelScope.launch {
            meshRepository.serviceState.collect { state ->
                if (state == uniffi.api.ServiceState.STOPPED) {
                    // Cancel all pending removal jobs
                    pendingNearbyRemovalJobs.values.forEach { it.cancel() }
                    pendingNearbyRemovalJobs.clear()
                    
                    // Clear nearby peers immediately
                    _nearbyPeers.value = emptyList()
                    Timber.d("Cleared all nearby peers on service stop")
                }
            }
        }
    }

    /**
     * Load all contacts from repository.
     */
    fun loadContacts() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                _error.value = null

                val contactList = meshRepository.listContacts()
                _contacts.value = contactList
                
                // Drop any nearby entry that is now a saved contact
                // Use comprehensive identity matching with public key as primary key
                _nearbyPeers.value = _nearbyPeers.value.filter { nearby ->
                    !contactList.any { contact -> isNearbyPeerContact(nearby, contact) }
                }

                Timber.d("Loaded ${contactList.size} contacts, filtered nearby peers to ${_nearbyPeers.value.size}")
            } catch (e: Exception) {
                _error.value = "Failed to load contacts: ${e.message}"
                Timber.e(e, "Failed to load contacts")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * Add a new contact.
     */
    fun addContact(
        peerId: String,
        publicKey: String,
        nickname: String? = null,
        libp2pPeerId: String? = null,
        listeners: List<String> = emptyList(),
        notes: String? = null,
        onComplete: ((Boolean) -> Unit)? = null
    ) {
        viewModelScope.launch {
            try {
                val trimmedKey = publicKey.trim()

                // Validate public key format before storing
                if (trimmedKey.isEmpty()) {
                    _error.value = "Public key cannot be empty"
                    onComplete?.invoke(false)
                    return@launch
                }
                if (trimmedKey.length != 64) {
                    _error.value = "Public key must be exactly 64 hex characters (got ${trimmedKey.length})"
                    onComplete?.invoke(false)
                    return@launch
                }
                if (!trimmedKey.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) {
                    _error.value = "Public key contains invalid characters (must be hex: 0-9, a-f)"
                    onComplete?.invoke(false)
                    return@launch
                }

                val generatedNotes = if (!libp2pPeerId.isNullOrEmpty()) {
                    "libp2p_peer_id:$libp2pPeerId;listeners:${listeners.joinToString(",")}"
                } else null

                val finalNotes = listOfNotNull(
                    notes?.trim()?.takeIf { it.isNotEmpty() },
                    generatedNotes?.trim()?.takeIf { it.isNotEmpty() }
                ).joinToString(";").takeIf { it.isNotEmpty() }

                // UNIFIED ID FIX: the contact key is always the public-key
                // identity. The libp2p peer ID remains in notes as routing
                // metadata and must never become the persisted contact ID.
                val canonicalPeerId = trimmedKey.lowercase()
                val contact = uniffi.api.Contact(
                    peerId = canonicalPeerId,
                    nickname = nickname,
                    localNickname = null,
                    publicKey = trimmedKey,
                    addedAt = (System.currentTimeMillis() / 1000).toULong(),
                    lastSeen = null,
                    notes = finalNotes,
                    lastKnownDeviceId = null,
                    verifiedAt = null,
                    isTombstone = false
                )

                meshRepository.addContact(contact)

                // Update device ID for the contact if we have routing info
                val discovered = meshRepository.discoveredPeers.value.entries.firstOrNull {
                    PeerIdValidator.isSame(it.key, peerId.trim()) ||
                    PeerIdValidator.isSame(it.value.peerId, peerId.trim()) ||
                    it.value.publicKey?.trim()?.equals(trimmedKey, ignoreCase = true) == true
                }
                val deviceId = discovered?.value?.let { info ->
                    // Extract device ID from discovered peer info if available
                    info.publicKey?.takeIf { it.length == 64 }
                }
                if (deviceId != null) {
                    meshRepository.updateContactDeviceId(canonicalPeerId, deviceId)
                }

                if (listeners.isNotEmpty()) {
                    val peerIdForDial = libp2pPeerId ?: peerId.trim()
                    meshRepository.connectToPeer(peerIdForDial, listeners)
                    Timber.i("Dialing nearby local peer after adding contact: $peerIdForDial")
                }

                loadContacts()
                Timber.i("Contact added: $canonicalPeerId (route=${libp2pPeerId ?: peerId.trim()})")
                onComplete?.invoke(true)
            } catch (e: Exception) {
                _error.value = "Failed to add contact: ${e.message}"
                Timber.e(e, "Failed to add contact")
                onComplete?.invoke(false)
            }
        }
    }

    /**
     * Remove a contact.
     */
    fun removeContact(peerId: String) {
        viewModelScope.launch {
            try {
                meshRepository.removeContact(peerId)
                loadContacts()

                Timber.i("Contact removed: $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to remove contact: ${e.message}"
                Timber.e(e, "Failed to remove contact")
            }
        }
    }

    /**
     * Update contact nickname with debouncing to prevent crashes from rapid updates.
     *
     * Implements debounced updates (NICKNAME-CRASH-001) - waits for user to finish
     * typing before propagating nickname changes to prevent real-time character-by-character
     * propagation that causes crashes.
     */
    fun setLocalNickname(peerId: String, nickname: String?) {
        // Cancel any pending nickname update for this peer
        pendingNicknameJobs[peerId]?.cancel()
        
        // Launch a new debounced update
        pendingNicknameJobs[peerId] = viewModelScope.launch {
            try {
                // Wait for debounce period before syncing
                delay(nicknameDebounceMs)
                
                meshRepository.setLocalNickname(peerId, nickname)
                loadContacts()

                Timber.d("Local nickname updated for $peerId (debounced)")
            } catch (e: kotlinx.coroutines.CancellationException) {
                // Expected when user continues typing - don't log as error
                Timber.d("Nickname update cancelled for $peerId (user still typing)")
            } catch (e: Exception) {
                _error.value = "Failed to update local nickname: ${e.message}"
                Timber.e(e, "Failed to update local nickname")
            } finally {
                // Clean up the job reference
                pendingNicknameJobs.remove(peerId)
            }
        }
    }

    fun setNickname(peerId: String, nickname: String?) {
        // Backward-compatible alias for existing callers.
        setLocalNickname(peerId, nickname)
    }

    /**
     * Get count of blocked contacts.
     */
    fun getBlockedCount(): UInt {
        return meshRepository.getBlockedCount()
    }

    /**
     * Set contact nickname at the repository level (updates both local and core).
     * Use this for contact detail screen saves to ensure nickname persists across sessions.
     */
    fun setContactNickname(peerId: String, nickname: String?) {
        viewModelScope.launch {
            try {
                meshRepository.setContactNickname(peerId, nickname)
                loadContacts()
                Timber.d("Contact nickname set at repository level: $peerId -> $nickname")
            } catch (e: Exception) {
                _error.value = "Failed to set contact nickname: ${e.message}"
                Timber.e(e, "Failed to set contact nickname")
            }
        }
    }

    /**
     * Mark a contact as verified after the user has compared safety numbers
     * out-of-band and confirmed they match.
     */
    fun markContactVerified(peerId: String) {
        viewModelScope.launch {
            try {
                meshRepository.markContactVerified(peerId)
                loadContacts()
                Timber.i("Contact marked verified: $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to mark contact verified: ${e.message}"
                Timber.e(e, "Failed to mark contact verified")
            }
        }
    }

    /** Clear a contact's verification status (e.g. after a key change). */
    fun unverifyContact(peerId: String) {
        viewModelScope.launch {
            try {
                meshRepository.unverifyContact(peerId)
                loadContacts()
                Timber.i("Contact verification cleared: $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to clear contact verification: ${e.message}"
                Timber.e(e, "Failed to clear contact verification")
            }
        }
    }

    /**
     * Compute the safety number for comparing our identity with [theirPublicKeyHex]
     * out-of-band. Returns null if our own identity isn't initialized yet.
     */
    fun computeSafetyNumber(theirPublicKeyHex: String): String? {
        return meshRepository.computeSafetyNumber(theirPublicKeyHex)
    }

    /**
     * Our own identity's readiness, so UI that depends on [computeSafetyNumber]
     * (which needs our identity's public key) can key off of it becoming
     * available after first composition (T9).
     */
    val identityInfo: StateFlow<uniffi.api.IdentityInfo?> = meshRepository.identityInfo

    /**
     * Update contact device ID (for multi-device tracking).
     *
     * @param peerId The peer ID
     * @param deviceId The device ID to associate with this contact
     */
    fun updateContactDeviceId(peerId: String, deviceId: String?) {
        viewModelScope.launch {
            try {
                meshRepository.updateContactDeviceId(peerId, deviceId)
                Timber.d("Updated contact device ID for $peerId: $deviceId")
            } catch (e: Exception) {
                _error.value = "Failed to update contact device ID: ${e.message}"
                Timber.e(e, "Failed to update contact device ID for $peerId")
            }
        }
    }

    /**
     * Update search query.
     */
    fun setSearchQuery(query: String) {
        _searchQuery.value = query
    }

    /**
     * Clear search query.
     */
    fun clearSearch() {
        _searchQuery.value = ""
    }

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }

    /**
     * Get contact count.
     */


    /**
     * Import contact from JSON export string.
     */
    fun importContact(json: String) {
        viewModelScope.launch {
            try {
                when (val parsed = parseContactImportPayload(json)) {
                    is ContactImportParseResult.Invalid -> {
                        _error.value = parsed.reason
                    }
                    is ContactImportParseResult.Valid -> {
                        val payload = parsed.payload
                        addContact(
                            payload.peerId,
                            payload.publicKey,
                            payload.nickname,
                            libp2pPeerId = payload.libp2pPeerId,
                            listeners = payload.listeners
                        )

                        if (payload.listeners.isNotEmpty()) {
                            val peerIdForDial = payload.libp2pPeerId ?: payload.peerId
                            meshRepository.connectToPeer(peerIdForDial, payload.listeners)
                            Timber.i("Connecting to imported peer (libp2p: ${payload.libp2pPeerId != null}): $peerIdForDial with addresses: ${payload.listeners}")
                        }
                    }
                }
            } catch (e: Exception) {
                _error.value = "Failed to import: ${e.message}"
            }
        }
    }

    /**
     * Promote a [NearbyPeer] to a saved [uniffi.api.Contact].
     *
     * This is the wiring used by the Nearby Discovery UI: tapping the per-row "Add"
     * button calls this method. The peer's public key must be known (i.e. the peer
     * has emitted an [PeerEvent.IdentityDiscovered]) before a contact can be saved.
     *
     * On success, [_error] is left unchanged and the underlying [meshRepository] is
     * asked to refresh its contact list. On failure (typically: missing public key
     * for a peer we only heard about via transport-level discovery), a user-facing
     * message is set in [_error] and the method returns false.
     *
     * @param peer The peer to promote; must be present in [nearbyPeers].
     * @return true on success, false if the peer cannot be promoted.
     */
    fun promoteNearbyPeerToContact(peer: NearbyPeer): Boolean {
        if (peer.publicKey.isNullOrBlank()) {
            _error.value = "Cannot add ${peer.displayName}: no public key yet (wait for identity announcement)"
            Timber.w("promoteNearbyPeerToContact rejected: missing public key for ${peer.peerId}")
            return false
        }
        addContact(
            peerId = peer.peerId,
            publicKey = peer.publicKey,
            nickname = peer.nickname,
            libp2pPeerId = peer.libp2pPeerId,
            listeners = peer.listeners
        )
        // addContact is fire-and-forget via viewModelScope; the contact will appear
        // after the next loadContacts() round-trip. We also drop the peer from the
        // nearby list optimistically so the UI updates immediately.
        _nearbyPeers.value = _nearbyPeers.value.filterNot { it.peerId == peer.peerId }
        return true
    }

    /**
     * Force a rediscovery of nearby peers by replaying the cached
     * [com.scmessenger.android.data.MeshRepository.discoveredPeers] map through the
     * event bus.
     *
     * Used by the "Rescan" button in the Nearby Discovery tab. Useful after the
     * user toggles a transport back on and we want the UI to reflect cached
     * discoveries immediately rather than waiting for the next periodic scan.
     *
     * A rescan is also treated as an explicit request to recheck any peers the
     * user previously dismissed: the dismissed set is cleared first, so the
     * replay below can re-surface them if they're still nearby.
     */
    fun refreshDiscovery() {
        if (_isScanning.value) return // already scanning
        _isScanning.value = true
        scanStartTimeMs = System.currentTimeMillis()
        _dismissedPeerIds.value = emptySet()
        Timber.d("refreshDiscovery: starting rescan")

        viewModelScope.launch {
            // Replay cached discoveries immediately
            meshRepository.replayDiscoveredPeerEvents()

            // Trigger actual BLE/WiFi rescan by restarting transport discovery
            try {
                meshRepository.triggerTransportRescan()
            } catch (e: Exception) {
                Timber.w(e, "Transport rescan failed")
            }

            // Keep scanning state active for at least 10s to allow BLE/WiFi
            // discovery to find new peers, then auto-clear
            delay(10_000L)
            _isScanning.value = false
            Timber.d("refreshDiscovery: rescan complete (${System.currentTimeMillis() - scanStartTimeMs}ms)")
        }
    }

    /**
     * Best-effort: classify a multiaddr/libp2p listener string as private LAN.
     *
     * Recognises IPv4 private ranges (10/8, 172.16/12, 192.168/16, 127/8) and the
     * emulator host (`/ip4/10.0.2.2`). IPv6 link-local and unique-local addresses
     * are also treated as LAN. Anything that includes a public IPv4 or hostname
     * resolves to false (treated as Internet by the caller).
     */
    private fun isPrivateLanAddress(addr: String): Boolean {
        val needle = addr.lowercase()
        // Common private IPv4 patterns inside multiaddrs (e.g. /ip4/192.168.1.5/tcp/9101)
        val privateV4Prefixes = listOf("/ip4/10.", "/ip4/172.16.", "/ip4/172.17.",
            "/ip4/172.18.", "/ip4/172.19.", "/ip4/172.2", "/ip4/172.30.", "/ip4/172.31.",
            "/ip4/192.168.", "/ip4/127.", "/ip4/169.254.", "/ip4/0.0.0.0",
            // Emulator host bridge
            "/ip4/10.0.2.2")
        if (privateV4Prefixes.any { needle.contains(it) }) return true
        // LAN IPv6 (link-local fe80::/10 and unique-local fc00::/7)
        if (needle.contains("/ip6/fe80:") || needle.contains("/ip6/fc") || needle.contains("/ip6/fd")) return true
        return false
    }

    override fun onCleared() {
        pendingNearbyRemovalJobs.values.forEach { it.cancel() }
        pendingNearbyRemovalJobs.clear()
        pendingNicknameJobs.values.forEach { it.cancel() }
        pendingNicknameJobs.clear()
        super.onCleared()
    }
}
