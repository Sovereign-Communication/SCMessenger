package com.scmessenger.shared.model

import kotlinx.serialization.Serializable

/**
 * Platform-agnostic contact model shared between Android and Desktop.
 */
@Serializable
data class Contact(
    val id: String,
    val publicKey: String,
    val nickname: String? = null,
    val localNickname: String? = null,
    val lastSeen: Long = 0L,
    val isOnline: Boolean = false,
    val unreadCount: Int = 0
) {
    val displayName: String
        get() = localNickname ?: nickname ?: publicKey.take(12) + "..."
}

/**
 * Platform-agnostic message model.
 */
@Serializable
data class ChatMessage(
    val id: String,
    val contactId: String,
    val text: String,
    val timestamp: Long,
    val isOutgoing: Boolean,
    val deliveryStatus: MessageDeliveryStatus = MessageDeliveryStatus.SENT
)

@Serializable
enum class MessageDeliveryStatus {
    PENDING, SENT, DELIVERED, READ, FAILED
}
