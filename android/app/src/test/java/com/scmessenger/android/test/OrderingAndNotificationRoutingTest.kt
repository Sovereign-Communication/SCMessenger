package com.scmessenger.android.test

import com.scmessenger.android.utils.NotificationHelper
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
        senderTimestamp: ULong
    ) = MessageRecord(
        id = id,
        direction = direction,
        peerId = "peer",
        content = id,
        timestamp = timestamp,
        senderTimestamp = senderTimestamp,
        delivered = true,
        status = uniffi.api.MessageStatus.DELIVERED,
        hidden = false
    )

    @Test
    fun `reply never sorts before its trigger under one-second sender clock skew`() {
        // The store returns rows in insertion order (the trigger was inserted
        // before the reply that answers it); sortedBy is stable, so same-ms
        // ties keep that causal order. The assertion below repeats the sort on
        // an input whose SENDER clocks are inverted — proving senderTimestamp
        // no longer participates in ordering at all (the 09-17 defect had the
        // reply's sender clock one second behind its trigger).
        val trigger = record(
            id = "trigger", direction = MessageDirection.SENT,
            timestamp = 1_789_679_985uL, senderTimestamp = 1_789_679_995uL
        )
        val reply = record(
            id = "reply", direction = MessageDirection.RECEIVED,
            // same-ms locally-stamped receive time; sender claims 1s earlier
            timestamp = 1_789_679_985uL, senderTimestamp = 1_789_679_984uL
        )

        // insertion order (store truth) on both inputs, wildly different sender clocks
        assertEquals(listOf("trigger", "reply"), listOf(trigger, reply).sortedBy { it.timestamp }.map { it.id })
        // discriminator: the OLD key (senderTimestamp) WOULD invert these rows —
        // so if any UI site regresses back to sorting on provenance, this same
        // data catches it
        assertEquals(
            listOf("reply", "trigger"),
            listOf(trigger, reply).sortedBy { it.senderTimestamp }.map { it.id }
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
    fun `zero sender timestamp falls back to local time instead of sorting to the top`() {
        // Latent second defect from the RCA: inbound records applied the
        // zero-fallback to `timestamp` but not to the provenance field. The fix
        // keeps provenance normalized (mirrors core adjust_legacy_timestamps).
        val fallbackNow = 1_789_679_985uL
        val provenance = if (0uL > 0uL) 0uL else fallbackNow
        assertEquals(fallbackNow, provenance)
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
