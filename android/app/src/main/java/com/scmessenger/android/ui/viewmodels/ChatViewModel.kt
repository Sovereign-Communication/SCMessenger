package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.MessageEvent
import com.scmessenger.android.utils.toEpochMillis
import com.scmessenger.android.utils.PeerIdValidator
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import timber.log.Timber
import javax.inject.Inject

/**
 * ViewModel for a single chat conversation.
 *
 * Handles message sending, receiving, delivery status tracking,
 * and real-time updates for a specific peer conversation.
 */
@HiltViewModel
class ChatViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {
    private val initialConversationLimit: UInt = 200u
    private val paginationStep: UInt = 100u

    // Current peer ID
    private val _peerId = MutableStateFlow<String?>(null)
    val peerId: StateFlow<String?> = _peerId.asStateFlow()

    // Messages for this conversation
    private val _messages = MutableStateFlow<List<uniffi.api.MessageRecord>>(emptyList())
    val messages: StateFlow<List<uniffi.api.MessageRecord>> = _messages.asStateFlow()

    // Message being composed
    private val _inputText = MutableStateFlow("")
    val inputText: StateFlow<String> = _inputText.asStateFlow()

    // Sending state
    private val _isSending = MutableStateFlow(false)
    val isSending: StateFlow<Boolean> = _isSending.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    // Contact info
    private val _contact = MutableStateFlow<uniffi.api.Contact?>(null)
    val contact: StateFlow<uniffi.api.Contact?> = _contact.asStateFlow()

    // Typing indicator state for local compose box.
    private val _isTyping = MutableStateFlow(false)
    val isTyping: StateFlow<Boolean> = _isTyping.asStateFlow()

    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    // Online status
    private val _isOnline = MutableStateFlow(false)
    val isOnline: StateFlow<Boolean> = _isOnline.asStateFlow()

    // Pending outbox items for retry timing display
    private val _pendingOutboxCount = MutableStateFlow(0)
    val pendingOutboxCount: StateFlow<Int> = _pendingOutboxCount.asStateFlow()

    private val _conversationLimit = MutableStateFlow(initialConversationLimit)

    init {
        observeMessageEvents()
        observeIncomingMessages()
        observeMessageUpdates()
        observePeerEvents()
        loadPendingOutboxCount()
    }

    /**
     * Set the peer for this chat conversation.
     */
    fun setPeer(peerId: String) {
        _peerId.value = peerId
        _conversationLimit.value = initialConversationLimit
        loadMessages()
        loadContact()
    }

    /**
     * Load messages for the current peer.
     */
    private fun loadMessages() {
        viewModelScope.launch {
            try {
                _isLoading.value = true
                val currentPeer = _peerId.value ?: return@launch
                val messageList = meshRepository.getConversation(currentPeer, limit = _conversationLimit.value)

                // MSG-PERSIST-001: Merge with existing optimistic messages, don't replace
                val currentMessages = _messages.value
                val mergedMessages = mutableListOf<uniffi.api.MessageRecord>()

                // Add all messages from history
                mergedMessages.addAll(messageList)

                // Add optimistic messages that aren't in history yet
                for (optimistic in currentMessages) {
                    val alreadyExistedInHistory = mergedMessages.any {
                        it.id == optimistic.id ||
                        (it.content == optimistic.content && it.direction == optimistic.direction && Math.abs(it.senderTimestamp.toLong() - optimistic.senderTimestamp.toLong()) < 2)
                    }
                    if (!alreadyExistedInHistory) {
                        // Keep optimistic message if not confirmed yet
                        mergedMessages.add(optimistic)
                    }
                }

                // MSG-ORDER-001: Sort strictly by sender-assigned timestamp to ensure consistent ordering across platforms
                _messages.value = mergedMessages.sortedBy { it.senderTimestamp }

                Timber.d("Loaded ${messageList.size} messages for $currentPeer (merged with ${currentMessages.size} existing)")
            } catch (e: Exception) {
                _error.value = "Failed to load messages: ${e.message}"
                Timber.e(e, "Failed to load messages")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * Load contact information.
     */
    private fun loadContact() {
        viewModelScope.launch {
            try {
                val currentPeer = _peerId.value ?: return@launch
                val contactInfo = meshRepository.getContact(currentPeer)
                _contact.value = contactInfo

                if (contactInfo == null) {
                    Timber.w("No contact found for peer: $currentPeer")
                }
            } catch (e: Exception) {
                Timber.e(e, "Failed to load contact")
            }
        }
    }

    /**
     * Send a message to the current peer.
     */
    fun sendMessage() {
        val currentPeer = _peerId.value
        val content = _inputText.value.trim()

        if (currentPeer == null) {
            _error.value = "No peer selected"
            return
        }

        if (content.isEmpty()) {
            return
        }

        viewModelScope.launch {
            try {
                _isSending.value = true
                _error.value = null

                // 1. Resolve symbols and normalize
                val normalizedPeerId = PeerIdValidator.normalize(currentPeer)

                // Optimistically add message to UI immediately with temporary ID
                val tempMessage = uniffi.api.MessageRecord(
                    id = java.util.UUID.randomUUID().toString(),
                    peerId = normalizedPeerId,
                    direction = uniffi.api.MessageDirection.SENT,
                    content = content,
                    timestamp = (System.currentTimeMillis() / 1000).toULong(),
                    senderTimestamp = (System.currentTimeMillis() / 1000).toULong(),
                    delivered = false,
                    status = uniffi.api.MessageStatus.QUEUED,
                    hidden = false
                )

                // Add to UI immediately
                val currentMessages = _messages.value.toMutableList()
                currentMessages.add(tempMessage)
                _messages.value = currentMessages.sortedBy { it.senderTimestamp }

                meshRepository.sendMessage(normalizedPeerId, content)

                // Clear input on success
                _inputText.value = ""
                _isTyping.value = false

                // Message persistence is guaranteed - it's in history and UI

                Timber.i("Message sent to $currentPeer")
            } catch (e: Exception) {
                _error.value = "Failed to send message: ${e.message}"
                Timber.e(e, "Failed to send message")
                // On error, reload to get accurate state
                loadMessages()
            } finally {
                _isSending.value = false
            }
        }
    }

    /**
     * Send a message with specific content (for testing/quick sends).
     */
    fun sendMessage(content: String) {
        _inputText.value = content
        sendMessage()
    }

    /**
     * Update input text.
     */
    fun updateInputText(text: String) {
        _inputText.value = text
        _isTyping.value = text.isNotBlank()
    }

    /**
     * Clear input text.
     */
    fun clearInput() {
        _inputText.value = ""
        _isTyping.value = false
    }

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }

    /**
     * Compare two peer IDs canonically (checks direct equality and canonical contact identity).
     */
    private fun isSamePeer(id1: String, id2: String): Boolean {
        if (id1.equals(id2, ignoreCase = true) || PeerIdValidator.isSame(id1, id2)) return true
        val c1 = meshRepository.canonicalContactId(id1)
        val c2 = meshRepository.canonicalContactId(id2)
        return (c1.isNotEmpty() && c1.equals(c2, ignoreCase = true)) ||
               (c1.isNotEmpty() && (c1.equals(id2, ignoreCase = true) || PeerIdValidator.isSame(c1, id2))) ||
               (c2.isNotEmpty() && (c2.equals(id1, ignoreCase = true) || PeerIdValidator.isSame(c2, id1)))
    }

    /**
     * Observe message events from MeshEventBus.
     */
    private fun observeMessageEvents() {
        viewModelScope.launch {
            MeshEventBus.messageEvents.collect { event ->
                when (event) {
                    is MessageEvent.Received -> {
                        // Reload if message is for current peer (consistent ID comparison)
                        val currentPeer = _peerId.value.orEmpty()
                        if (isSamePeer(event.messageRecord.peerId, currentPeer)) {
                            loadMessages()
                        }
                    }
                    is MessageEvent.Delivered -> {
                        // Update delivery status
                        updateMessageStatus(event.messageId, delivered = true)
                    }
                    is MessageEvent.Failed -> {
                        Timber.w("Message failed: ${event.messageId} - ${event.error}")
                    }
                    else -> {
                        // Handle other events if needed
                    }
                }
            }
        }
    }

    /**
     * Observe incoming messages from repository.
     */
    private fun observeIncomingMessages() {
        viewModelScope.launch {
            meshRepository.incomingMessages.collect { message ->
                if (isSamePeer(message.peerId, _peerId.value.orEmpty())) {
                    loadMessages()
                }
            }
        }
    }

    /**
     * Observe message updates (sent messages).
     */
    private fun observeMessageUpdates() {
        viewModelScope.launch {
            meshRepository.messageUpdates.collect { message ->
                if (isSamePeer(message.peerId, _peerId.value.orEmpty())) {
                    // Replace or add the message
                    val currentMessages = _messages.value.toMutableList()

                    // Find and replace if exists, otherwise add
                    val existingIndex = currentMessages.indexOfFirst { it.id == message.id }
                    if (existingIndex >= 0) {
                        currentMessages[existingIndex] = message
                        Timber.d("Updated message ${message.id.take(8)} in UI")
                    } else {
                        // Check if we have a duplicate by content and timestamp (optimistic message)
                        val duplicateIndex = currentMessages.indexOfFirst { existing ->
                            existing.content == message.content &&
                            existing.direction == message.direction &&
                            Math.abs(existing.senderTimestamp.toLong() - message.senderTimestamp.toLong()) < 2
                        }

                        if (duplicateIndex >= 0) {
                            // Replace optimistic message with real one
                            currentMessages[duplicateIndex] = message
                            Timber.d("Replaced optimistic message with real message ${message.id.take(8)}")
                        } else {
                            // New message, add it
                            currentMessages.add(message)
                            Timber.d("Added sent message ${message.id.take(8)} to UI")
                        }
                    }

                    _messages.value = currentMessages.sortedBy { it.senderTimestamp }
                }
            }
        }
    }

    private fun observePeerEvents() {
        viewModelScope.launch {
            MeshEventBus.peerEvents.collect { event ->
                val current = _peerId.value ?: return@collect
                when (event) {
                    is com.scmessenger.android.service.PeerEvent.Connected -> {
                        if (event.peerId == current) _isOnline.value = true
                    }
                    is com.scmessenger.android.service.PeerEvent.Disconnected -> {
                        if (event.peerId == current) _isOnline.value = false
                    }
                    else -> Unit
                }
            }
        }
    }

    /**
     * Load pending outbox count for display (messages awaiting delivery).
     */
    private fun loadPendingOutboxCount() {
        viewModelScope.launch {
            try {
                val pending = meshRepository.loadPendingOutboxAsync()
                _pendingOutboxCount.value = pending.size
            } catch (e: Exception) {
                Timber.e(e, "Failed to load pending outbox count")
            }
        }
    }

    /**
     * Get retry delay for a given attempt count (for display in message retry timing).
     */
    fun getRetryDelayForAttempt(attemptCount: Int): Long {
        return meshRepository.getRetryDelay(attemptCount)
    }

    /**
     * Check if a message should be retried.
     */
    fun shouldRetryMessage(messageId: String): Boolean {
        return meshRepository.shouldRetryMessage(messageId)
    }

    /**
     * Increment attempt count for a message being retried.
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
     * Log a message delivery attempt.
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
     * Update message delivery status in the UI.
     */
    private fun updateMessageStatus(messageId: String, delivered: Boolean) {
        val updatedMessages = _messages.value.map { msg ->
            if (msg.id == messageId) {
                uniffi.api.MessageRecord(
                    id = msg.id,
                    peerId = msg.peerId,
                    direction = msg.direction,
                    content = msg.content,
                    timestamp = msg.timestamp,
                    senderTimestamp = msg.senderTimestamp,
                    delivered = delivered,
                    status = if (delivered) uniffi.api.MessageStatus.DELIVERED else msg.status,
                    hidden = msg.hidden
                )
            } else {
                msg
            }
        }
        _messages.value = updatedMessages
    }

    fun loadMoreMessages() {
        _conversationLimit.value = _conversationLimit.value + paginationStep
        loadMessages()
    }

    /**
     * Get formatted timestamp for a message.
     */
    fun formatTimestamp(timestamp: ULong): String {
        val millis = timestamp.toEpochMillis()
        val date = java.util.Date(millis)
        val now = java.util.Date()

        val sdf = if (isSameDay(date, now)) {
            java.text.SimpleDateFormat("HH:mm", java.util.Locale.getDefault())
        } else {
            java.text.SimpleDateFormat("MMM d, HH:mm", java.util.Locale.getDefault())
        }

        return sdf.format(date)
    }

    private fun isSameDay(date1: java.util.Date, date2: java.util.Date): Boolean {
        val cal1 = java.util.Calendar.getInstance().apply { time = date1 }
        val cal2 = java.util.Calendar.getInstance().apply { time = date2 }

        return cal1.get(java.util.Calendar.YEAR) == cal2.get(java.util.Calendar.YEAR) &&
               cal1.get(java.util.Calendar.DAY_OF_YEAR) == cal2.get(java.util.Calendar.DAY_OF_YEAR)
    }
}
