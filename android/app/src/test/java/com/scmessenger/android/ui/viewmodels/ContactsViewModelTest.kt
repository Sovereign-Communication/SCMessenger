package com.scmessenger.android.ui.viewmodels

import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.PeerEvent
import com.scmessenger.android.service.TransportType
import io.mockk.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.*
import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test

/**
 * Unit tests for [ContactsViewModel] methods that are part of the Nearby Discovery
 * integration: [promoteNearbyPeerToContact] and [refreshDiscovery].
 */
@OptIn(ExperimentalCoroutinesApi::class)
class ContactsViewModelTest {

    private lateinit var viewModel: ContactsViewModel
    private lateinit var mockMeshRepository: MeshRepository
    private val testDispatcher = StandardTestDispatcher()

    // Reusable valid 64-char hex public key for tests.
    private val validPublicKey = "a".repeat(64)

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        mockMeshRepository = mockk(relaxed = true)

        // MeshRepository dependencies used by the ViewModel init + nearby flow.
        // NOTE: MeshEventBus.peerEvents is a global singleton, not on the repo,
        // so it isn't stubbed here. The init's observeNearbyPeers() launches
        // a collector on the global flow but we never emit into it, so it
        // just sits idle.
        every { mockMeshRepository.serviceState } returns MutableStateFlow(
            uniffi.api.ServiceState.STOPPED
        )
        every { mockMeshRepository.discoveredPeers } returns MutableStateFlow(emptyMap())
        every { mockMeshRepository.listContacts() } returns emptyList()
        every { mockMeshRepository.getContact(any<String>()) } returns null
        every { mockMeshRepository.isBootstrapRelayPeer(any<String>()) } returns false
        every { mockMeshRepository.replayDiscoveredPeerEvents() } returns Unit
        every { mockMeshRepository.addContact(any<uniffi.api.Contact>()) } returns Unit
        every { mockMeshRepository.connectToPeer(any<String>(), any<List<String>>()) } returns Unit

        viewModel = ContactsViewModel(mockMeshRepository)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    // -----------------------------------------------------------------------
    // promoteNearbyPeerToContact
    // -----------------------------------------------------------------------

    @Test
    fun `promoteNearbyPeerToContact rejects peer without public key`() = runTest {
        val peer = NearbyPeer(
            peerId = "12D3KooNoPKPeer",
            nickname = "Alice",
            publicKey = null
        )

        val result = viewModel.promoteNearbyPeerToContact(peer)

        assertFalse("Promotion should fail when publicKey is null", result)
        // addContact must not be invoked when we have no public key
        verify(exactly = 0) {
            mockMeshRepository.addContact(any<uniffi.api.Contact>())
        }
        // Error state is populated with a user-facing message
        val err = viewModel.error.value
        assertNotNull("error should be set on rejected promotion", err)
        assertTrue(
            "error should mention the missing public key",
            err!!.contains("public key", ignoreCase = true)
        )
    }

    @Test
    fun `promoteNearbyPeerToContact calls addContact with peer fields and returns true`() = runTest {
        val peer = NearbyPeer(
            peerId = "12D3KooGoodPeer",
            publicKey = validPublicKey,
            nickname = "Bob",
            libp2pPeerId = "12D3KooLibp2p",
            listeners = listOf("/ip4/192.168.1.50/tcp/9101"),
            transport = TransportType.TCP_MDNS
        )

        // Capture the Contact that addContact receives
        val contactSlot = slot<uniffi.api.Contact>()
        every { mockMeshRepository.addContact(capture(contactSlot)) } returns Unit

        val result = viewModel.promoteNearbyPeerToContact(peer)
        testDispatcher.scheduler.advanceUntilIdle()

        assertTrue("Promotion should succeed when publicKey is present", result)
        verify(exactly = 1) { mockMeshRepository.addContact(any<uniffi.api.Contact>()) }
        verify(exactly = 1) {
            mockMeshRepository.connectToPeer("12D3KooLibp2p", listOf("/ip4/192.168.1.50/tcp/9101"))
        }

        val contact = contactSlot.captured
        // Contact.peerId is the PUBLIC KEY, not the libp2p peer id. That is
        // deliberate, not a bug: ContactsViewModel.addContact builds the contact
        // with `val canonicalPeerId = trimmedKey.lowercase()` and passes it as
        // peerId (ContactsViewModel.kt:518-520). Core agrees -- it looks
        // contacts up by public-key hex, e.g.
        // `get_contact_bundle(&hex::encode(&sender_pubkey))` in
        // IronCore::receive_message.
        //
        // The libp2p peer id is not lost; it is carried in `notes`, which the
        // assertions below already check.
        //
        // The old expectation here (`peer.peerId`) had never actually run in CI:
        // this suite only reached the gate with PR 129, and before that the task
        // hung and reported nothing. So this is a stale expectation being
        // surfaced for the first time, not a regression.
        assertEquals(validPublicKey.lowercase(), contact.peerId)
        assertEquals(validPublicKey, contact.publicKey)
        // UNIFICATION: addContact stores user-provided name as localNickname (primary), not federated nickname
        assertEquals("Bob", contact.localNickname)
        assertEquals(null, contact.nickname)
        // The generated notes should encode the libp2p peer id and listeners
        assertTrue(
            "notes should include libp2p peer id",
            contact.notes?.contains("12D3KooLibp2p") == true
        )
        assertTrue(
            "notes should include listener",
            contact.notes?.contains("192.168.1.50") == true
        )
    }

    @Test
    fun `promoteNearbyPeerToContact drops the peer from nearbyPeers optimistically`() = runTest {
        val peer = NearbyPeer(
            peerId = "12D3KooDropMe",
            publicKey = validPublicKey,
            nickname = "Carol"
        )

        // No need to seed the list — empty list is fine, the call should not crash
        // and the filter is a no-op when the peer is absent.
        val result = viewModel.promoteNearbyPeerToContact(peer)
        testDispatcher.scheduler.advanceUntilIdle()

        assertTrue(result)
        val remaining = viewModel.nearbyPeers.value.filter { it.peerId == peer.peerId }
        assertTrue(
            "Peer should be removed from nearbyPeers after successful promotion",
            remaining.isEmpty()
        )
    }

    // -----------------------------------------------------------------------
    // Nearby peer identity distinction tests
    // -----------------------------------------------------------------------

    @Test
    fun `PeerEvent Discovered does not create nearby peer for raw transport connection`() = runTest {
        val rawTransportPeerId = "12D3KooWRawTransportSocket"
        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            PeerEvent.Discovered(
                peerId = rawTransportPeerId,
                transport = TransportType.INTERNET
            )
        )
        testDispatcher.scheduler.advanceUntilIdle()

        // Raw transport connections without identity must NEVER become NearbyPeer entries
        val nearby = viewModel.nearbyPeers.value
        assertTrue(
            "Raw transport Peer ID must not appear in nearby peers list",
            nearby.none { it.peerId == rawTransportPeerId }
        )
    }

    @Test
    fun `PeerEvent IdentityDiscovered creates nearby peer with sovereign identity id`() = runTest {
        val rawPeerId = "12D3KooWAnnouncedPeer"
        val expectedIdentityId = "5f2566a7c0d8d5410643fe32dfc531fa79ea9c0a958785e0c85bdf3514752f7c"
        every { mockMeshRepository.resolveToIdentityId(validPublicKey) } returns expectedIdentityId

        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            PeerEvent.IdentityDiscovered(
                peerId = rawPeerId,
                publicKey = validPublicKey,
                nickname = "Dave",
                libp2pPeerId = rawPeerId,
                listeners = listOf("/ip4/127.0.0.1/tcp/9001"),
                blePeerId = null
            )
        )
        testDispatcher.scheduler.advanceUntilIdle()

        val nearby = viewModel.nearbyPeers.value
        val peer = nearby.firstOrNull { it.publicKey == validPublicKey }
        assertNotNull("Peer should be discovered as nearby", peer)
        assertEquals(
            "NearbyPeer peerId must be the sovereign identity hash, never a libp2p Peer ID",
            expectedIdentityId,
            peer?.peerId
        )
        assertEquals("Dave", peer?.nickname)
        assertEquals(rawPeerId, peer?.libp2pPeerId)
    }

    @Test
    fun `PeerEvent IdentityDiscovered with unresolvable transport peer id is rejected`() = runTest {
        val rawTransportId = "12D3KooWUnresolvableTransport"
        every { mockMeshRepository.resolveToIdentityId(any()) } returns null

        com.scmessenger.android.service.MeshEventBus.emitPeerEvent(
            PeerEvent.IdentityDiscovered(
                peerId = rawTransportId,
                publicKey = "not_a_valid_hex_key",
                nickname = "Ghost",
                libp2pPeerId = rawTransportId,
                listeners = emptyList(),
                blePeerId = null
            )
        )
        testDispatcher.scheduler.advanceUntilIdle()

        val nearby = viewModel.nearbyPeers.value
        assertTrue(
            "Nearby peers must strictly reject unresolvable transport Peer IDs",
            nearby.none { it.peerId == rawTransportId || it.nickname == "Ghost" }
        )
    }

    // -----------------------------------------------------------------------
    // refreshDiscovery
    // -----------------------------------------------------------------------

    @Test
    fun `refreshDiscovery calls meshRepository replayDiscoveredPeerEvents at least once`() = runTest {
        // Clear interactions from the init's delayed replayDiscoveredPeerEvents() call
        // (the init schedules one after a 100ms delay).
        testDispatcher.scheduler.advanceUntilIdle()
        clearMocks(mockMeshRepository, answers = false, recordedCalls = true, childMocks = false)

        viewModel.refreshDiscovery()
        testDispatcher.scheduler.advanceUntilIdle()

        verify(atLeast = 1) { mockMeshRepository.replayDiscoveredPeerEvents() }
    }
}
