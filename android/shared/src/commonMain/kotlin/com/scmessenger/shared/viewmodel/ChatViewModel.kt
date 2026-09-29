package com.scmessenger.shared.viewmodel

import com.scmessenger.shared.model.ChatMessage
import com.scmessenger.shared.model.MessageDeliveryStatus
import com.scmessenger.shared.platform.PlatformNetworking
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

/**
 * Shared ViewModel for chat with a specific contact.
 * Manages message history, sending, and delivery status.
 */
open class ChatViewModel(
    private val networking: PlatformNetworking,
    val contactId: String = "",
    val contactName: String = ""
) {
    private val scope = CoroutineScope(Dispatchers.Main + SupervisorJob())

    private val _messages = MutableStateFlow<List<ChatMessage>>(emptyList())
    val messages: StateFlow<List<ChatMessage>> = _messages.asStateFlow()

    private val _inputText = MutableStateFlow("")
    val inputText: StateFlow<String> = _inputText.asStateFlow()

    private val _isSending = MutableStateFlow(false)
    val isSending: StateFlow<Boolean> = _isSending.asStateFlow()

    fun updateInput(text: String) {
        _inputText.value = text
    }

    @OptIn(ExperimentalUuidApi::class)
    fun sendMessage() {
        val text = _inputText.value.trim()
        if (text.isEmpty()) return

        val message = ChatMessage(
            id = Uuid().toString(),
            contactId = contactId,
            text = text,
            timestamp = System.currentTimeMillis(),
            isOutgoing = true,
            deliveryStatus = MessageDeliveryStatus.PENDING
        )

        _messages.value = _messages.value + message
        _inputText.value = ""

        scope.launch {
            _isSending.value = true
            try {
                val sent = networking.sendMessage(contactId, text)
                _messages.value = _messages.value.map {
                    if (it.id == message.id) {
                        it.copy(deliveryStatus = if (sent) MessageDeliveryStatus.SENT else MessageDeliveryStatus.FAILED)
                    } else it
                }
            } catch (e: Exception) {
                _messages.value = _messages.value.map {
                    if (it.id == message.id) {
                        it.copy(deliveryStatus = MessageDeliveryStatus.FAILED)
                    } else it
                }
            } finally {
                _isSending.value = false
            }
        }
    }
}
