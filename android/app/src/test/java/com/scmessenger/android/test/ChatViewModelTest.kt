package com.scmessenger.android.test

import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.MessageEvent
import com.scmessenger.android.service.PeerEvent
import com.scmessenger.android.service.TransportType
import com.scmessenger.android.ui.viewmodels.ChatViewModel
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ChatViewModelTest {

    private lateinit var repository: MeshRepository
    private lateinit var viewModel: ChatViewModel
    private val testDispatcher = StandardTestDispatcher()
    private val incoming = MutableSharedFlow<uniffi.api.MessageRecord>(replay = 0)

    private fun message(id: String, peerId: String, delivered: Boolean = false): uniffi.api.MessageRecord {
        return uniffi.api.MessageRecord(
            id = id,
            peerId = peerId,
            direction = uniffi.api.MessageDirection.SENT,
            content = "hello",
            timestamp = 1u,
            senderTimestamp = 1u,
            delivered = delivered,
            status = if (delivered) uniffi.api.MessageStatus.DELIVERED else uniffi.api.MessageStatus.SENT,
            hidden = false
        )
    }

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        repository = mockk(relaxed = true)
        every { repository.incomingMessages } returns incoming
        every { repository.messageUpdates } returns MutableSharedFlow()
        every { repository.getConversation(any(), any()) } returns listOf(message("m1", "peer1"))
        every { repository.getContact(any()) } returns null
        coEvery { repository.sendMessage(any(), any()) } returns Unit
        viewModel = ChatViewModel(repository).apply {
            ioDispatcher = testDispatcher
        }
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `setPeer loads conversation`() = runTest(testDispatcher) {
        viewModel.setPeer("peer1")
        advanceUntilIdle()

        assertEquals("peer1", viewModel.peerId.value)
        assertEquals(1, viewModel.messages.value.size)
        verify { repository.getConversation("peer1", 200u) }
    }

    @Test
    fun `sendMessage sends and clears input`() = runTest(testDispatcher) {
        viewModel.setPeer("peer1")
        viewModel.updateInputText("hello world")
        viewModel.sendMessage()
        advanceUntilIdle()

        coVerify(exactly = 1) { repository.sendMessage("peer1", "hello world") }
        assertEquals("", viewModel.inputText.value)
    }

    @Test
    fun `sendMessage without selected peer sets error`() {
        viewModel.updateInputText("hello")
        viewModel.sendMessage()
        assertEquals("No peer selected", viewModel.error.value)
    }

    @Test
    fun `delivery status event marks message delivered`() = runTest(testDispatcher) {
        viewModel.setPeer("peer1")
        advanceUntilIdle()

        MeshEventBus.emitMessageEvent(MessageEvent.Delivered("m1"))
        advanceUntilIdle()

        assertTrue(viewModel.messages.value.first { it.id == "m1" }.delivered)
    }

    @Test
    fun `peer events update online status`() = runTest(testDispatcher) {
        val collectJob = launch { viewModel.isOnline.collect { } }
        viewModel.setPeer("peer1")
        advanceUntilIdle()

        MeshEventBus.emitPeerEvent(PeerEvent.Connected("peer1", TransportType.BLE))
        advanceUntilIdle()
        assertTrue(viewModel.isOnline.value)

        MeshEventBus.emitPeerEvent(PeerEvent.Disconnected("peer1"))
        advanceUntilIdle()
        assertFalse(viewModel.isOnline.value)
        collectJob.cancel()
    }

    @Test
    fun `loadMoreMessages increases conversation limit`() = runTest(testDispatcher) {
        viewModel.setPeer("peer1")
        advanceUntilIdle()

        viewModel.loadMoreMessages()
        advanceUntilIdle()

        verify { repository.getConversation("peer1", 300u) }
    }

    // ------------------------------------------------------------------
    // Ordering regression guard: MSG-ORDER-002 / P1_ANDROID_CHAT_ORDER_CROSS_CLOCK
    //
    // The race the operator named: an auto-reply rendered ABOVE the message that
    // triggered it, because ordering used `senderTimestamp` -- a REMOTE clock for
    // inbound rows and this phone's clock for outbound rows. #309 repointed every
    // sort site at the locally-assigned `timestamp`.
    //
    // The guard below drives the REAL ViewModel, so a regression AT A SORT SITE
    // fails here. The previous test could not catch that: it sorted local lists
    // itself and therefore passed no matter what ChatViewModel, ChatScreen or
    // ConversationsViewModel actually sorted by.
    //
    // Each fixture asserts its own discriminating power. If the reply's sender
    // stamp were not behind the trigger's, ordering by the sender stamp would
    // agree with ordering by the local stamp and the test would prove nothing.
    // ------------------------------------------------------------------

    private fun record(
        id: String,
        direction: uniffi.api.MessageDirection,
        timestamp: ULong,
        senderTimestamp: ULong
    ) = uniffi.api.MessageRecord(
        id = id,
        peerId = "peer1",
        direction = direction,
        content = id,
        timestamp = timestamp,
        senderTimestamp = senderTimestamp,
        delivered = true,
        status = uniffi.api.MessageStatus.DELIVERED,
        hidden = false
    )

    /** The 2026-09-17 shape: the peer's reply is stamped 1s BEHIND our own send. */
    private fun skewedReplyPair(): Pair<uniffi.api.MessageRecord, uniffi.api.MessageRecord> {
        val trigger = record(
            id = "trigger",
            direction = uniffi.api.MessageDirection.SENT,
            timestamp = 1_789_679_985uL,
            senderTimestamp = 1_789_679_995uL
        )
        val reply = record(
            id = "reply",
            direction = uniffi.api.MessageDirection.RECEIVED,
            timestamp = 1_789_679_990uL,
            senderTimestamp = 1_789_679_984uL
        )
        assertTrue(
            "fixture is vacuous unless the reply's SENDER stamp is behind the trigger's",
            reply.senderTimestamp < trigger.senderTimestamp
        )
        return trigger to reply
    }

    @Test
    fun `conversation load renders the reply after its trigger under sender clock skew`() =
        runTest(testDispatcher) {
            val (trigger, reply) = skewedReplyPair()
            every { repository.getConversation(any(), any()) } returns listOf(trigger, reply)

            viewModel.setPeer("peer1")
            advanceUntilIdle()

            // Ordering by senderTimestamp yields [reply, trigger] -- the inversion
            // the operator reported. Ordering by the local stamp cannot.
            assertEquals(listOf("trigger", "reply"), viewModel.messages.value.map { it.id })
        }

    @Test
    fun `same-millisecond local stamp still keeps the trigger first`() = runTest(testDispatcher) {
        val t = 1_789_679_985uL
        val trigger = record(
            id = "trigger",
            direction = uniffi.api.MessageDirection.SENT,
            timestamp = t,
            senderTimestamp = t + 10uL
        )
        val reply = record(
            id = "reply",
            direction = uniffi.api.MessageDirection.RECEIVED,
            timestamp = t,
            senderTimestamp = t - 1uL
        )
        assertTrue(
            "fixture is vacuous unless the reply's SENDER stamp is behind the trigger's",
            reply.senderTimestamp < trigger.senderTimestamp
        )
        every { repository.getConversation(any(), any()) } returns listOf(trigger, reply)

        viewModel.setPeer("peer1")
        advanceUntilIdle()

        // A same-second tie is common, not exotic: the 09-17 store had 10 distinct
        // seconds holding both a Sent and a Received row. The local stamps tie, so
        // the stable sort must preserve the store's causal insertion order; the
        // sender stamps do NOT tie, so a regression to that key reorders this pair.
        assertEquals(listOf("trigger", "reply"), viewModel.messages.value.map { it.id })
    }

    @Test
    fun `reply arriving on the live stream lands below the message it answers`() =
        runTest(testDispatcher) {
            val (trigger, reply) = skewedReplyPair()
            every { repository.getConversation(any(), any()) } returns listOf(trigger)

            viewModel.setPeer("peer1")
            advanceUntilIdle()
            assertEquals(listOf("trigger"), viewModel.messages.value.map { it.id })

            // observeIncomingMessages() reloads the conversation when a message
            // arrives, so this is the real arrival path: history now contains the
            // reply and the merge at the "assigned to _messages.value" sort site
            // decides the rendered order.
            every { repository.getConversation(any(), any()) } returns listOf(trigger, reply)
            incoming.emit(reply)
            advanceUntilIdle()

            assertEquals(listOf("trigger", "reply"), viewModel.messages.value.map { it.id })
        }
}
