package com.scmessenger.android.utils

import uniffi.api.MessageDirection
import uniffi.api.MessageRecord

/**
 * MSG-ORDER-003: the single owner of message ordering.
 *
 * A row's `timestamp` is this device's clock in whole seconds, so two rows
 * written inside one second tie. The keys below break that tie, and each one is
 * either a recorded fact or a direction that cannot contradict causality; they
 * mirror `mobile_bridge::newest_first` exactly, reversed.
 *
 * 1. `storedAtMillis` -- the store's insertion fact for the row.
 * 2. direction rank   -- Sent before Received, for rows written before that
 *    fact existed. Inside one second only one order can contradict causality
 *    (an auto-reply cannot precede the message it answers), and message-id
 *    order -- what shipped -- was not it: 24 of 332 real auto-reply pairs on
 *    the operator's Pixel reloaded with the reply above its trigger.
 * 3. `senderTimestamp` -- the sender's clock, consulted only when both rows
 *    share a direction and therefore a clock, where it is that sender's real
 *    send order.
 *
 * `senderTimestamp` is still never a *primary* key: it cross-compares two
 * devices (P1_ANDROID_CHAT_ORDER_CROSS_CLOCK). A full tie keeps the store's
 * order, as it does today.
 */
private val MESSAGE_ORDER: Comparator<MessageRecord> =
    compareBy(
        { it.timestamp },
        { it.storedAtMillis },
        { if (it.direction == MessageDirection.SENT) 0 else 1 },
        { it.senderTimestamp }
    )

/** Oldest first: the order a conversation is read in. */
fun List<MessageRecord>.inCausalOrder(): List<MessageRecord> = sortedWith(MESSAGE_ORDER)

/** Newest first: previews, thread summaries and history-sync payloads. */
fun List<MessageRecord>.newestFirst(): List<MessageRecord> = sortedWith(MESSAGE_ORDER.reversed())
