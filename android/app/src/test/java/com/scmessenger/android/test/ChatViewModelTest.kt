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
}
