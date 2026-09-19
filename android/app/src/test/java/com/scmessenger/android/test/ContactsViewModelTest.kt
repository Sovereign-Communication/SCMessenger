package com.scmessenger.android.test

import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.PeerEvent
import com.scmessenger.android.service.TransportType
import com.scmessenger.android.ui.viewmodels.ContactsViewModel
import com.scmessenger.android.ui.viewmodels.NearbyPeer
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.launch
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ContactsViewModelTest {

    private lateinit var viewModel: ContactsViewModel
    private lateinit var repository: MeshRepository
    private val testDispatcher = UnconfinedTestDispatcher()

    private fun contact(peerId: String, nickname: String? = null): uniffi.api.Contact {
        return uniffi.api.Contact(
            peerId = peerId,
            nickname = nickname,
            localNickname = null,
            publicKey = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
            addedAt = 1u,
            lastSeen = null,
            notes = null,
            lastKnownDeviceId = null,
            verifiedAt = null,
            isTombstone = false
        )
    }

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        repository = mockk(relaxed = true)
        
        // Setup service state flow to avoid NPE in observeServiceState
        val serviceStateFlow = kotlinx.coroutines.flow.MutableStateFlow(uniffi.api.ServiceState.STOPPED)
        every { repository.serviceState } returns serviceStateFlow

        // Setup discoveredPeers flow to avoid NPE in addContact
        val discoveredPeersFlow = kotlinx.coroutines.flow.MutableStateFlow<Map<String, MeshRepository.PeerDiscoveryInfo>>(emptyMap())
        every { repository.discoveredPeers } returns discoveredPeersFlow

        every { repository.listContacts() } returns listOf(
            contact("peer-alice", "Alice"),
            contact("peer-bob", "Bob")
        )
        // Ensure basic methods return non-null defaults even if relaxed
        every { repository.replayDiscoveredPeerEvents() } returns Unit
        every { repository.isBootstrapRelayPeer(any()) } returns false
        every { repository.getContact(any()) } returns null
        every { repository.resolveToIdentityId(any()) } answers { firstArg<String?>()?.takeIf { it.length == 64 } }

        viewModel = ContactsViewModel(repository)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `loadContacts populates contacts state`() = runTest {
        advanceUntilIdle()
        assertEquals(2, viewModel.contacts.value.size)
        assertEquals("Alice", viewModel.contacts.value.first().nickname)
    }

    @Test
    fun `addContact validates key and persists contact`() = runTest {
        every { repository.addContact(any()) } returns Unit
        every { repository.connectToPeer(any(), any()) } returns Unit

        viewModel.addContact(
            peerId = "peer-new",
            publicKey = "a".repeat(64),
            nickname = "New",
            libp2pPeerId = "12D3KooWxyz",
            listeners = listOf("/ip4/1.2.3.4/tcp/4001")
        )
        advanceUntilIdle()

        verify(exactly = 1) { repository.addContact(any()) }
        verify(exactly = 1) { repository.connectToPeer("12D3KooWxyz", any()) }
    }

    @Test
    fun `removeContact forwards removal to repository`() = runTest {
        every { repository.removeContact(any()) } returns Unit

        viewModel.removeContact("peer-alice")
        advanceUntilIdle()

        verify(exactly = 1) { repository.removeContact("peer-alice") }
    }

    @Test
    fun `search filters contacts by nickname`() = runTest {
        val collectJob = launch { viewModel.filteredContacts.collect() }
        advanceUntilIdle()

        viewModel.setSearchQuery("Alice")
        advanceUntilIdle()

        assertEquals(1, viewModel.filteredContacts.value.size)
        assertEquals("peer-alice", viewModel.filteredContacts.value.first().peerId)
        collectJob.cancel()
    }

    @Test
    fun `online status updates nearby peers from peer events`() = runTest {
        val uniqueId = "nearby-" + System.currentTimeMillis()
        val testPeerId = uniqueId.padEnd(64, '0').take(64) // Valid 64-hex ID
        
        // Setup repository mock
        every { repository.replayDiscoveredPeerEvents() } returns Unit
        every { repository.isBootstrapRelayPeer(any()) } returns false
        every { repository.getContact(any()) } returns null
        
        // Use a real MutableStateFlow for serviceState so it works with collect
        val serviceStateFlow = kotlinx.coroutines.flow.MutableStateFlow(uniffi.api.ServiceState.RUNNING)
        every { repository.serviceState } returns serviceStateFlow

        // Initialize ViewModel fresh for this test
        viewModel = ContactsViewModel(repository)

        // Give the ViewModel's init blocks time to run
        advanceTimeBy(1000)
        
        // Emit identity discovery event
        val identityEvent = PeerEvent.IdentityDiscovered(
            peerId = testPeerId,
            publicKey = testPeerId, // Use same 64-hex for simplicity
            nickname = "Nearby Alice",
            libp2pPeerId = "12D3KooWxyz" + "a".repeat(39), // Valid LibP2P ID
            listeners = emptyList(),
            blePeerId = null
        )
        
        MeshEventBus.emitPeerEvent(identityEvent)
        
        // With UnconfinedTestDispatcher, processing should be immediate, 
        // but runCurrent() helps ensure all launches are executed.
        runCurrent()

        val nearby = viewModel.nearbyPeers.value
        
        assertTrue("Nearby peer should be discovered and online. Found: $nearby", 
            nearby.any { it.publicKey == testPeerId && it.isOnline })

        MeshEventBus.emitPeerEvent(PeerEvent.Disconnected(testPeerId))
        
        // Don't advance too much or the removal job will run and clear the peer
        // The removal job has a 5s delay.
        runCurrent()
        
        val nearbyAfterDisconnect = viewModel.nearbyPeers.value
        assertTrue("Nearby peer should be offline after disconnect (before removal). Found: $nearbyAfterDisconnect", 
            nearbyAfterDisconnect.any { it.publicKey == testPeerId && !it.isOnline })
            
        // Now advance time to trigger removal
        advanceTimeBy(6000)
        
        val nearbyAfterRemoval = viewModel.nearbyPeers.value
        assertTrue("Nearby peer should be removed after disconnect grace period. Found: $nearbyAfterRemoval", 
            nearbyAfterRemoval.none { it.publicKey == testPeerId })
    }
}
