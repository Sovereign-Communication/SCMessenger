package com.scmessenger.android.utils

import uniffi.api.MessageDirection
import uniffi.api.MessageRecord

/**
 * MSG-ORDER-003: the single owner of message ordering.
 *
 * A row's `timestamp` is this device's clock in whole seconds, so two rows
 * written inside one second tie. The keys below break that tie; each is a
 * recorded fact or a direction that cannot contradict causality, and they are
 * the same four keys as `mobile_bridge::newest_first`, in reverse.
 *
 * 1. `storedAtMillis` -- the store's insertion fact for the row.
 * 2. direction rank   -- Sent before Received, for rows written before that
 *    fact existed. Inside one second only one order can contradict causality
 *    (an auto-reply cannot precede the message it answers), and message-id
 *    order -- what shipped -- was not it: 24 of 332 real auto-reply pairs on
 *    the operator's Pixel reloaded with the reply above its trigger.
 * 3. `id`             -- a deterministic last resort, which makes the order
 *    total: the displayed order never depends on how the list was assembled.
 *    A tie between two rows the peer sent (both Received) is decided here --
 *    the corpus has four such pairs and none tied.
 *
 * `senderTimestamp` is never a key: it is the sender's clock and cross-compares
 * two devices (P1_ANDROID_CHAT_ORDER_CROSS_CLOCK), and a key no case exercises
 * is a rule nobody can check.
 */
private val MESSAGE_ORDER: Comparator<MessageRecord> =
    compareBy(
        { it.timestamp },
        { it.storedAtMillis },
        { if (it.direction == MessageDirection.SENT) 0 else 1 },
        { it.id }
    )

/** Oldest first: the order a conversation is read in. */
fun List<MessageRecord>.inCausalOrder(): List<MessageRecord> = sortedWith(MESSAGE_ORDER)

/** Newest first: previews, thread summaries and history-sync payloads. */
fun List<MessageRecord>.newestFirst(): List<MessageRecord> = sortedWith(MESSAGE_ORDER.reversed())
