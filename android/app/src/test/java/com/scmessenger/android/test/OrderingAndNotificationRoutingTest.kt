package com.scmessenger.android.test

import com.scmessenger.android.utils.NotificationHelper
import com.scmessenger.android.utils.inCausalOrder
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.api.MessageDirection
import uniffi.api.MessageRecord

/**
 * MSG-ORDER-002 regression tests: conversation ordering must be driven by the
 * locally-assigned `timestamp` field, never by the sender's clock
 * (`senderTimestamp`), which cross-compares devices and let a reply render
 * BEFORE the message that caused it (P1_ANDROID_CHAT_ORDER_CROSS_CLOCK).
 *
 * The sort expressions live in ChatViewModel / ConversationsViewModel /
 * ChatScreen and read `it.timestamp` after the fix. These tests pin the data
 * contract they rely on plus the pure routing pieces, because spinning the
 * full ViewModel stack in JVM tests needs the native lib. The two invariants
 * under test:
 *
 * 1. sorting a mixed clock-skew conversation by `timestamp` yields causal
 *    order regardless of senderTimestamp values;
 * 2. NotificationHelper.channelForMessage routes sound-off messages to the
 *    muted twin channels (the API 26+ sound contract).
 */
class OrderingAndNotificationRoutingTest {

    private fun record(
        id: String,
        direction: MessageDirection,
        timestamp: ULong,
        senderTimestamp: ULong,
        storedAtMillis: ULong = 0uL
    ) = MessageRecord(
        id = id,
        direction = direction,
        peerId = "peer",
        content = id,
        timestamp = timestamp,
        senderTimestamp = senderTimestamp,
        delivered = true,
        status = uniffi.api.MessageStatus.DELIVERED,
        hidden = false,
        storedAtMillis = storedAtMillis
    )

    @Test
    fun `reply never sorts before its trigger on a same-second reload`() {
        // The store hands the UI a newest-first list, so a stable single-key
        // sort on `timestamp` leaves a same-second reply ABOVE its trigger --
        // the P1_ANDROID_CHAT_ORDER_CROSS_CLOCK symptom, still live for 24 of
        // 332 real auto-reply pairs on the operator's Pixel. storedAtMillis is
        // the tie-break that fixes it, and the reply's id is deliberately BELOW
        // the trigger's so no id order can fake the result.
        val trigger = record(
            id = "zz-trigger", direction = MessageDirection.SENT,
            timestamp = 1_789_841_591uL, senderTimestamp = 1_789_841_591uL,
            storedAtMillis = 1_000uL
        )
        val reply = record(
            id = "aa-reply", direction = MessageDirection.RECEIVED,
            // same local second; the sender's own clock claims 1s earlier
            timestamp = 1_789_841_591uL, senderTimestamp = 1_789_841_590uL,
            storedAtMillis = 1_270uL
        )

        // newest first, exactly as the store returns the conversation
        val fromStore = listOf(reply, trigger)
        assertEquals(
            listOf("zz-trigger", "aa-reply"),
            fromStore.inCausalOrder().map { it.id }
        )
        // the pre-fix expression cannot order these rows at all, so this guard
        // is not vacuous: the same data still shows the defect under the old key
        assertEquals(
            listOf("aa-reply", "zz-trigger"),
            fromStore.sortedBy { it.timestamp }.map { it.id }
        )
        // and sender provenance is not a sort key either
        assertEquals(
            listOf("aa-reply", "zz-trigger"),
            fromStore.sortedBy { it.senderTimestamp }.map { it.id }
        )
    }

    @Test
    fun `legacy rows without an insertion fact still render the trigger first`() {
        // Rows written before the store recorded an insertion fact (storedAtMillis
        // is 0 on both) still have to put a same-second reply below its trigger.
        // The trigger's id sorts ABOVE the reply's, so id order cannot produce
        // this result and the direction rank is what does.
        val trigger = record(
            id = "zz-trigger", direction = MessageDirection.SENT,
            timestamp = 1_789_841_591uL, senderTimestamp = 1_789_841_591uL
        )
        val reply = record(
            id = "aa-reply", direction = MessageDirection.RECEIVED,
            timestamp = 1_789_841_591uL, senderTimestamp = 1_789_841_590uL
        )

        // newest first, as the store returns it under the same keys
        val fromStore = listOf(reply, trigger)
        assertEquals(
            listOf("zz-trigger", "aa-reply"),
            fromStore.inCausalOrder().map { it.id }
        )
        // and the pre-fix expression still shows the defect on this data
        assertEquals(
            listOf("aa-reply", "zz-trigger"),
            fromStore.sortedBy { it.timestamp }.map { it.id }
        )
    }

    @Test
    fun `locally stamped inbound rows order causally against locally sent rows`() {
        // Post-fix contract: inbound rows carry the receiver's local clock.
        // Even a wildly wrong sender clock cannot reorder the conversation.
        val sent1 = record("s1", MessageDirection.SENT, timestamp = 100uL, senderTimestamp = 100uL)
        val inbound = record("r1", MessageDirection.RECEIVED, timestamp = 101uL, senderTimestamp = 9_999_999uL)
        val sent2 = record("s2", MessageDirection.SENT, timestamp = 102uL, senderTimestamp = 102uL)

        val sorted = listOf(sent2, inbound, sent1).sortedBy { it.timestamp }
        assertEquals(listOf("s1", "r1", "s2"), sorted.map { it.id })
    }

    @Test
    fun `sound off routes messages to muted twin channels`() {
        assertEquals(
            NotificationHelper.CHANNEL_MESSAGES_MUTED,
            NotificationHelper.channelForMessage(isDmRequest = false, soundOn = false)
        )
        assertEquals(
            NotificationHelper.CHANNEL_MESSAGE_REQUESTS_MUTED,
            NotificationHelper.channelForMessage(isDmRequest = true, soundOn = false)
        )
    }

    @Test
    fun `sound on routes messages to the normal channels`() {
        assertEquals(
            NotificationHelper.CHANNEL_MESSAGES,
            NotificationHelper.channelForMessage(isDmRequest = false, soundOn = true)
        )
        assertEquals(
            NotificationHelper.CHANNEL_MESSAGE_REQUESTS,
            NotificationHelper.channelForMessage(isDmRequest = true, soundOn = true)
        )
    }
}
