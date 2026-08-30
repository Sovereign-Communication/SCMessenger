package com.scmessenger.android.ui.chat

data class PendingDeliverySnapshot(
    val attemptCount: Int,
    val nextAttemptAtEpochSec: Long,
    val terminalFailureCode: String? = null,
    val exhausted: Boolean = false
)

enum class DeliveryStateSurface(val label: String, val detail: String) {
    PENDING(
        label = "pending",
        detail = "Queued locally. First route attempt is still in progress."
    ),
    STORED(
        label = "stored",
        detail = "Stored for retry. The recipient is currently offline or unreachable."
    ),
    QUEUED_DELIVERING(
        label = "queued/delivering",
        detail = "Still queued and undelivered. Earlier attempts couldn't be confirmed; it stays queued (never dropped) and will try again when a route exists."
    ),
    FORWARDING(
        label = "forwarding",
        detail = "Actively retrying through direct or relay paths."
    ),
    REJECTED(
        label = "rejected",
        detail = "Rejected because the recipient identity is no longer valid for this device."
    ),
    DELIVERED(
        label = "delivered",
        detail = "Delivery receipt confirmed by the recipient node."
    )
}

data class DeliveryStatePresentation(
    val state: DeliveryStateSurface,
    val label: String,
    val detail: String
)

object DeliveryStateMapper {
    // Contract (precedence, top to bottom): DELIVERED > REJECTED > QUEUED_DELIVERING
    // > FORWARDING > STORED > PENDING. Deliberate: an at-cap retained entry shows
    // queued/delivering even during the 60s window when a real attempt is in
    // flight, because the message is not actively progressing -- it is waiting
    // out the patient backoff of an exhausted entry. delivered always wins so a
    // receipt never gets hidden behind a stale exhausted flag.
    fun resolve(
        delivered: Boolean,
        pending: PendingDeliverySnapshot?,
        nowEpochSec: Long
    ): DeliveryStatePresentation {
        val state = when {
            delivered -> DeliveryStateSurface.DELIVERED
            pending?.terminalFailureCode != null -> DeliveryStateSurface.REJECTED
            pending?.exhausted == true -> DeliveryStateSurface.QUEUED_DELIVERING
            pending != null && pending.nextAttemptAtEpochSec <= nowEpochSec -> DeliveryStateSurface.FORWARDING
            pending != null -> DeliveryStateSurface.STORED
            else -> DeliveryStateSurface.PENDING
        }
        val detail = when (pending?.terminalFailureCode) {
            "identity_device_mismatch" ->
                "Rejected because this identity moved to another device. Refresh the contact before retrying."
            "identity_abandoned" ->
                "Rejected because the contact abandoned this identity. Re-verify the contact before sending again."
            else -> state.detail
        }
        return DeliveryStatePresentation(
            state = state,
            label = state.label,
            detail = detail
        )
    }
}
