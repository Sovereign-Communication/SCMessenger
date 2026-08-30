package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.MessageEvent
import com.scmessenger.android.ui.chat.DeliveryStateMapper
import com.scmessenger.android.ui.chat.DeliveryStatePresentation
import com.scmessenger.android.ui.chat.PendingDeliverySnapshot
import com.scmessenger.android.utils.PeerIdValidator
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import javax.inject.Inject

/**
 * ViewModel for the conversations/chat list screen.
 *
 * Manages message history and conversation threads.
 */
@HiltViewModel
class ConversationsViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {

    // Recent messages
    private val _messages = MutableStateFlow<List<uniffi.api.MessageRecord>>(emptyList())
    val messages: StateFlow<List<uniffi.api.MessageRecord>> = _messages.asStateFlow()

    // Grouped conversations (by peer)
    val conversations = messages.map { messageList ->
        messageList
            .groupBy { it.peerId }
            // MSG-ORDER-001: Sort strictly by sender-assigned timestamp to ensure consistent ordering across platforms
            .mapValues { (_, msgs) -> msgs.sortedByDescending { it.senderTimestamp } }
            .toList()
            .sortedByDescending { (_, msgs) -> msgs.firstOrNull()?.senderTimestamp ?: 0u }
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

    // Stats
    private val _stats = MutableStateFlow<uniffi.api.HistoryStats?>(null)
    val stats: StateFlow<uniffi.api.HistoryStats?> = _stats.asStateFlow()

    // Inbox count for badge display
    private val _inboxCount = MutableStateFlow(0u)
    val inboxCount: StateFlow<UInt> = _inboxCount.asStateFlow()

    // Blocked peers
    private val _blockedPeers = MutableStateFlow<List<uniffi.api.BlockedIdentity>>(emptyList())
    val blockedPeers: StateFlow<List<uniffi.api.BlockedIdentity>> = _blockedPeers.asStateFlow()

    init {
        loadMessages()
        loadStats()
        loadBlockedPeers()
        loadInboxCount()

        // Listen for message updates (sent or received) to refresh the list
        viewModelScope.launch {
            meshRepository.messageUpdates.collect {
                loadMessages()
            }
        }

        // Receipt/transport events can change delivery state without a new message
        // body; refresh to keep conversation badges and previews accurate.
        viewModelScope.launch {
            MeshEventBus.messageEvents.collect { event ->
                when (event) {
                    is MessageEvent.Delivered,
                    is MessageEvent.Failed -> loadMessages()
                    else -> Unit
                }
            }
        }
    }

    /**
     * Load recent messages.
     * FIX(Compose-crash): guard StateFlow emits with viewModelScope.isActive to prevent
     * SlotTable crash during onDestroy. Updating StateFlow after ViewModel is cleared
     * triggers recomposition while LayoutNodeSubcompositionsState is releasing, causing
     * ArrayIndexOutOfBounds via PrefetchHandleProvider.
     */
    fun loadMessages(limit: UInt = 100u) {
        if (!viewModelScope.isActive) {
            Timber.d("loadMessages skipped — ViewModelScope not active (destroyed)")
            return
        }
        viewModelScope.launch {
            try {
                if (!viewModelScope.isActive) return@launch
                _isLoading.value = true
                _error.value = null

                val messageList = meshRepository.getRecentMessages(limit = limit)
                if (!viewModelScope.isActive) return@launch
                _messages.value = messageList
                _inboxCount.value = meshRepository.getInboxCount()

                Timber.d("Loaded ${messageList.size} messages")
            } catch (e: Exception) {
                if (!viewModelScope.isActive) return@launch
                _error.value = "Failed to load messages: ${e.message}"
                Timber.e(e, "Failed to load messages")
            } finally {
                if (viewModelScope.isActive) _isLoading.value = false
            }
        }
    }

    /**
     * Load conversation with specific peer.
     */
    fun loadConversation(peerId: String, limit: UInt = 100u): Flow<List<uniffi.api.MessageRecord>> {
        return flow {
            try {
                val messages = meshRepository.getConversation(peerId, limit)
                emit(messages)
                Timber.d("Loaded ${messages.size} messages for $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to load conversation")
                emit(emptyList())
            }
        }
    }

    /**
     * Send a message to a peer.
     */
    /**
     * Send a message to a peer.
     */
    suspend fun sendMessage(peerId: String, content: String): Boolean {
        Timber.d("VIEWMODEL_SEND: Entered sendMessage method")
        return withContext(Dispatchers.IO) {
            try {
                Timber.d("VIEWMODEL_SEND: peerId='$peerId', contentLen=${content.length}")
                val normalizedPeerId = PeerIdValidator.normalize(peerId)
                Timber.d("VIEWMODEL_SEND: normalized='$normalizedPeerId'")

                // Call repository to handle encryption and transmission
                Timber.d("VIEWMODEL_SEND: Calling meshRepository.sendMessage")
                meshRepository.sendMessage(normalizedPeerId, content)
                Timber.d("VIEWMODEL_SEND: meshRepository.sendMessage completed")

                // Reload messages to show the sent message
                Timber.d("VIEWMODEL_SEND: Reloading messages")
                loadMessages()

                Timber.i("Message sent to $normalizedPeerId")
                true
            } catch (e: Exception) {
                _error.value = "Failed to send message: ${e.message}"
                Timber.e(e, "Failed to send message")
                false
            } finally {
                Timber.d("VIEWMODEL_SEND: Exiting sendMessage method")
            }
        }
    }

    /**
     * Mark a message as delivered.
     */
    fun markDelivered(messageId: String) {
        viewModelScope.launch {
            try {
                meshRepository.markMessageDelivered(messageId)
                loadMessages()

                Timber.d("Message marked as delivered: $messageId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to mark message as delivered")
            }
        }
    }

    /**
     * Clear conversation with a peer.
     */
    fun clearConversation(peerId: String) {
        viewModelScope.launch {
            try {
                meshRepository.clearConversation(peerId)
                loadMessages()

                Timber.i("Conversation cleared: $peerId")
            } catch (e: Exception) {
                _error.value = "Failed to clear conversation: ${e.message}"
                Timber.e(e, "Failed to clear conversation")
            }
        }
    }

    /**
     * Clear all message history.
     */
    fun clearAllHistory() {
        viewModelScope.launch {
            try {
                meshRepository.clearHistory()
                loadMessages()
                loadStats()

                Timber.i("All history cleared")
            } catch (e: Exception) {
                _error.value = "Failed to clear history: ${e.message}"
                Timber.e(e, "Failed to clear history")
            }
        }
    }

    /**
     * Check if a peer can be messaged (exists in contacts or discovered peers).
     * AND-SEND-BTN-001: Benefiting from in-memory ID resolution cache.
     */
    fun isPeerAvailable(peerId: String): Boolean {
        val contact = getContactForPeer(peerId)
        if (contact != null) return true

        // Check discovered peers
        val discoveredPeers = meshRepository.discoveredPeers.value
        return discoveredPeers[peerId]?.publicKey != null
    }

    /**
     * Get peer info for adding to contacts quickly.
     */
    fun getPeerInfo(peerId: String): Pair<String, String>? {
        // Check discovered peers for public key (case-insensitive)
        val discovered = meshRepository.discoveredPeers.value.entries.firstOrNull {
            it.key.equals(peerId, ignoreCase = true)
        }?.value
        return discovered?.publicKey?.let { pubKey ->
            val nickname = discovered.nickname ?: peerId.take(8)
            pubKey to nickname
        }
    }

    /**
     * Block a peer by ID
     */
    fun blockPeer(peerId: String, reason: String? = null) {
        viewModelScope.launch {
            try {
                meshRepository.blockPeer(peerId, reason)
                loadBlockedPeers()
                Timber.i("Blocked peer: $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to block peer: $peerId")
            }
        }
    }

    /**
     * WS14: Mute notifications for a peer (blocks the peer).
     */
    fun mutePeer(peerId: String, reason: String? = null) {
        viewModelScope.launch {
            try {
                meshRepository.mutePeer(peerId, reason)
                loadBlockedPeers()
                Timber.i("Muted peer: $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to mute peer: $peerId")
            }
        }
    }

    /**
     * Unblock a peer by ID
     */
    fun unblockPeer(peerId: String) {
        viewModelScope.launch {
            try {
                meshRepository.unblockPeer(peerId)
                loadBlockedPeers()
                Timber.i("Unblocked peer: $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to unblock peer: $peerId")
            }
        }
    }

    /**
     * Block a peer AND delete all their stored messages (cascade purge).
     */
    fun blockAndDeletePeer(peerId: String, reason: String? = null) {
        viewModelScope.launch {
            try {
                meshRepository.blockAndDeletePeer(peerId, reason)
                loadBlockedPeers()
                loadMessages()
                Timber.i("Blocked and deleted peer: $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to block and delete peer: $peerId")
            }
        }
    }

    /**
     * Check if a peer is blocked
     */
    fun isBlocked(peerId: String): Boolean {
        // First check our reactive state if possible, fallback to repository
        val inList = _blockedPeers.value.any { it.peerId == peerId }
        if (inList) return true
        
        return try {
            meshRepository.isBlocked(peerId)
        } catch (e: Exception) {
            Timber.e(e, "Failed to check if peer is blocked: $peerId")
            false
        }
    }

    /**
     * Load blocked peers from repository.
     * FIX: isActive guard to prevent SlotTable crash on destroy (see loadMessages).
     */
    fun loadBlockedPeers() {
        if (!viewModelScope.isActive) {
            Timber.d("loadBlockedPeers skipped — ViewModelScope not active")
            return
        }
        viewModelScope.launch {
            try {
                if (!viewModelScope.isActive) return@launch
                val blocked = meshRepository.listBlockedPeers()
                if (!viewModelScope.isActive) return@launch
                _blockedPeers.value = blocked
                Timber.d("Loaded ${blocked.size} blocked peers")
            } catch (e: Exception) {
                if (!viewModelScope.isActive) return@launch
                Timber.e(e, "Failed to load blocked peers")
            }
        }
    }

    /**
     * Load message statistics.
     */
    private fun loadStats() {
        viewModelScope.launch {
            try {
                val historyStats = meshRepository.getHistoryStats()
                _stats.value = historyStats

                Timber.d("Loaded stats: $historyStats")
            } catch (e: Exception) {
                Timber.e(e, "Failed to load stats")
            }
        }
    }

    /**
     * Load inbox count for badge display.
     */
    private fun loadInboxCount() {
        viewModelScope.launch {
            try {
                _inboxCount.value = meshRepository.getInboxCount()
            } catch (e: Exception) {
                Timber.e(e, "Failed to load inbox count")
            }
        }
    }

    /**
     * Search messages.
     */
    fun searchMessages(query: String, limit: UInt = 50u): Flow<List<uniffi.api.MessageRecord>> {
        return flow {
            try {
                val results = meshRepository.searchMessages(query, limit)
                emit(results)
                Timber.d("Found ${results.size} messages matching '$query'")
            } catch (e: Exception) {
                Timber.e(e, "Failed to search messages")
                emit(emptyList())
            }
        }
    }

    /**
     * Get contact info for a peer (for displaying nickname).
     * Tries multiple lookup strategies to handle ID format mismatches:
     * 1. Direct lookup by peerId (canonicalized)
     * 2. Lookup by libp2p peer ID from discovered peers
     *
     * AND-SEND-BTN-001: The underlying canonicalContactId() now uses an in-memory
     * cache, so repeated calls for the same peer ID are fast and avoid FFI.
     */
    fun getContactForPeer(peerId: String): uniffi.api.Contact? {
        return try {
            // Strategy 1: Direct lookup (handles canonical ID resolution)
            val directLookup = meshRepository.getContact(peerId)
            if (directLookup != null) return directLookup

            // Strategy 2: Try looking up by libp2p peer ID from discovered peers
            val discoveredPeers = meshRepository.discoveredPeers.value
            val discovered = discoveredPeers.entries.firstOrNull { (key, info) ->
                PeerIdValidator.isSame(key, peerId) ||
                    info.peerId.equals(peerId, ignoreCase = true)
            }?.value

            // Try libp2p peer ID if available
            discovered?.libp2pPeerId?.let { libp2pId ->
                meshRepository.getContact(libp2pId)
            }
        } catch (e: Exception) {
            Timber.d("Failed to get contact for peer $peerId: ${e.message}")
            null
        }
    }

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }

    /**
     * Get total message count.
     */
    fun getMessageCount(): UInt {
        return meshRepository.getMessageCount()
    }

    fun resolveDeliveryState(
        message: uniffi.api.MessageRecord,
        nowEpochSec: Long = System.currentTimeMillis() / 1000
    ): DeliveryStatePresentation {
        // One outbox read per pending row: getPendingDeliverySnapshot carries
        // attemptCount, nextAttemptAtEpochSec AND the exhausted flag together,
        // so composition never pays a second (or third) synchronous file read.
        val info = meshRepository.getPendingDeliverySnapshot(message.id)
        val terminalFailureCode = meshRepository.getPendingTerminalFailureCode(message.id)
        val pending = info?.let {
            PendingDeliverySnapshot(
                attemptCount = it.attemptCount,
                nextAttemptAtEpochSec = it.nextAttemptAtEpochSec,
                terminalFailureCode = terminalFailureCode,
                exhausted = it.exhausted
            )
        }
        return DeliveryStateMapper.resolve(
            delivered = message.delivered,
            pending = pending,
            nowEpochSec = nowEpochSec
        )
    }
}
