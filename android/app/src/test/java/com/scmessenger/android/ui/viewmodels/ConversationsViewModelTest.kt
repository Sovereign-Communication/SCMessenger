package com.scmessenger.android.ui.viewmodels

import com.scmessenger.android.data.MeshRepository
import io.mockk.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.*
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.Assert.*
import uniffi.api.MessageRecord
import uniffi.api.MessageDirection
import uniffi.api.HistoryStats
import java.util.UUID

@OptIn(ExperimentalCoroutinesApi::class)
class ConversationsViewModelTest {

    private lateinit var viewModel: ConversationsViewModel
    private lateinit var mockMeshRepository: MeshRepository
    private val testDispatcher = StandardTestDispatcher()

    // Mock flow for message updates
    private val messageUpdatesFlow = MutableSharedFlow<MessageRecord>(replay = 0)

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        mockMeshRepository = mockk(relaxed = true)

        // Mock message updates flow
        every { mockMeshRepository.messageUpdates } returns messageUpdatesFlow

        // Mock getRecentMessages to return a list (empty initially, then populated)
        every { mockMeshRepository.getRecentMessages(any(), any()) } returns emptyList()

        // Mock getHistoryStats to perform cleanly
        every { mockMeshRepository.getHistoryStats() } returns HistoryStats(0u, 0u, 0u, 0u)

        viewModel = ConversationsViewModel(mockMeshRepository)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `viewModel loads messages on initialization`() = runTest {
        // Run any pending coroutines (init block)
        testDispatcher.scheduler.advanceUntilIdle()

        // Verify loadMessages was called with any args
        verify { mockMeshRepository.getRecentMessages(any(), any()) }
    }

    @Test
    fun `viewModel reloads messages when a new message update is received`() = runTest {
        // Given viewModel is initialized
        testDispatcher.scheduler.advanceUntilIdle()
        // The init load runs on Dispatchers.IO (ConversationsViewModel.kt:107),
        // which this test scheduler cannot order; without waiting for it to
        // land, the late-recorded init call inflates the exact-count verify
        // below nondeterministically (CI run 35410042169, 2026-09-19).
        verify(timeout = 5_000) { mockMeshRepository.getRecentMessages(any(), any()) }
        clearMocks(mockMeshRepository, answers = false) // Clear previous calls but keep mocks

        // Setup mock to return a message now
        val testMessage = MessageRecord(
            id = UUID.randomUUID().toString(),
            direction = MessageDirection.SENT,
            peerId = "peer1",
            content = "Hello",
            timestamp = 1000uL,
            senderTimestamp = 1000uL,
            delivered = true,
            status = uniffi.api.MessageStatus.DELIVERED,
            hidden = false
        )
        every { mockMeshRepository.getRecentMessages(any(), any()) } returns listOf(testMessage)

        // When a message update is emitted
        messageUpdatesFlow.emit(testMessage)
        testDispatcher.scheduler.advanceUntilIdle()

        // Then verify getRecentMessages is called again (reloading list)
        verify(exactly = 1) { mockMeshRepository.getRecentMessages(any(), any()) }

        // And verify state is updated
        assertEquals(1, viewModel.messages.value.size)
        assertEquals("Hello", viewModel.messages.value[0].content)
    }

    @Test
    fun `clearConversation delegates to repository and refreshes list`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()
        // ConversationsViewModel.loadMessages() dispatches on Dispatchers.IO
        // (ConversationsViewModel.kt:107), which this test's scheduler cannot
        // order. The init load can therefore still be in flight when clearMocks()
        // runs and be recorded AFTER it, inflating the recorded call count
        // nondeterministically (observed: 3 getRecentMessages where 1 was
        // expected). Wait for the init load to be recorded first.
        verify(timeout = 5_000) { mockMeshRepository.getRecentMessages(any(), any()) }
        clearMocks(mockMeshRepository, answers = false)
        every { mockMeshRepository.getRecentMessages(any(), any()) } returns emptyList()

        viewModel.clearConversation("peer-delete")
        testDispatcher.scheduler.advanceUntilIdle()

        // The post-clear refresh also runs on Dispatchers.IO; wait for it to land
        // so the exact-count assertions below stay deterministic AND strict.
        verify(timeout = 5_000) { mockMeshRepository.getRecentMessages(any(), any()) }
        verify(exactly = 1) { mockMeshRepository.clearConversation("peer-delete") }
        verify(exactly = 1) { mockMeshRepository.getRecentMessages(any(), any()) }
    }

    @Test
    fun `blockPeer delegates to repository`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        viewModel.blockPeer("blocked-peer", "Spam")
        testDispatcher.scheduler.advanceUntilIdle()

        verify { mockMeshRepository.blockPeer("blocked-peer", "Spam") }
    }

    @Test
    fun `unblockPeer delegates to repository`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        viewModel.unblockPeer("blocked-peer")
        testDispatcher.scheduler.advanceUntilIdle()

        verify { mockMeshRepository.unblockPeer("blocked-peer") }
    }

    @Test
    fun `blockAndDeletePeer delegates to repository`() = runTest {
        testDispatcher.scheduler.advanceUntilIdle()

        viewModel.blockAndDeletePeer("blocked-peer", "Abuse")
        testDispatcher.scheduler.advanceUntilIdle()

        verify { mockMeshRepository.blockAndDeletePeer("blocked-peer", "Abuse") }
    }

    @Test
    fun `isBlocked delegates to repository and returns correct value`() = runTest {
        every { mockMeshRepository.isBlocked("peer-blocked") } returns true
        every { mockMeshRepository.isBlocked("peer-not-blocked") } returns false

        assertTrue(viewModel.isBlocked("peer-blocked"))
        assertFalse(viewModel.isBlocked("peer-not-blocked"))

        verify { mockMeshRepository.isBlocked("peer-blocked") }
        verify { mockMeshRepository.isBlocked("peer-not-blocked") }
    }

    @Test
    fun `isBlocked returns false on exception`() = runTest {
        every { mockMeshRepository.isBlocked("peer-error") } throws RuntimeException("Test error")

        assertFalse(viewModel.isBlocked("peer-error"))
    }
}
