package com.scmessenger.android.ui.viewmodels

import android.content.Context
import android.net.Uri
import com.scmessenger.android.data.IdentityCreationCoordinator
import com.scmessenger.android.data.IdentityState
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.data.PreferencesRepository
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class MainViewModelTest {

    private lateinit var viewModel: MainViewModel
    private lateinit var mockMeshRepository: MeshRepository
    private lateinit var mockPreferencesRepository: PreferencesRepository
    private lateinit var mockIdentityCoordinator: IdentityCreationCoordinator
    private lateinit var mockContext: Context
    private val testDispatcher = StandardTestDispatcher()

    // Valid 64-hex char public key
    private val validPublicKey = "a".repeat(64)
    // Valid libp2p peer ID (starts with 12D3Koo, base58 chars, length in 46..56)
    private val validPeerId = "12D3KooWABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxy"

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        mockMeshRepository = mockk(relaxed = true)
        mockPreferencesRepository = mockk(relaxed = true)
        mockIdentityCoordinator = mockk(relaxed = true)
        mockContext = mockk(relaxed = true)

        every { mockPreferencesRepository.onboardingCompleted } returns MutableStateFlow(false)
        every { mockPreferencesRepository.installChoiceCompleted } returns MutableStateFlow(false)
        every { mockPreferencesRepository.themeMode } returns MutableStateFlow(PreferencesRepository.ThemeMode.SYSTEM)

        every { mockMeshRepository.serviceState } returns MutableStateFlow(uniffi.api.ServiceState.STOPPED)
        every { mockMeshRepository.identityInfo } returns MutableStateFlow(null)
        every { mockMeshRepository.getLocalIpAddress() } returns "192.168.1.100"
        every { mockMeshRepository.isIdentityInitialized() } returns false
        every { mockMeshRepository.getAvailableStorageMB() } returns 1000L
        every { mockMeshRepository.connectToPeer(any(), any()) } returns Unit

        every { mockIdentityCoordinator.identityState } returns MutableStateFlow(IdentityState.None)
        every { mockIdentityCoordinator.error } returns MutableStateFlow(null)
        every { mockIdentityCoordinator.progressStage } returns MutableStateFlow(IdentityProgressStage.Idle)
        every { mockIdentityCoordinator.progressSubDetail } returns MutableStateFlow(null)

        viewModel = MainViewModel(
            meshRepository = mockMeshRepository,
            preferencesRepository = mockPreferencesRepository,
            identityCreationCoordinator = mockIdentityCoordinator,
            context = mockContext
        )
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun createMockUri(
        scheme: String? = "scmessenger",
        host: String? = "invite",
        params: Map<String, List<String>> = emptyMap(),
        shouldThrowOnAccess: Boolean = false
    ): Uri {
        val mockUri = mockk<Uri>(relaxed = true)
        if (shouldThrowOnAccess) {
            every { mockUri.scheme } throws RuntimeException("Malformed URI")
            return mockUri
        }
        every { mockUri.scheme } returns scheme
        every { mockUri.host } returns host
        every { mockUri.getQueryParameter(any()) } answers {
            val key = firstArg<String>()
            params[key]?.firstOrNull()
        }
        every { mockUri.getQueryParameters(any()) } answers {
            val key = firstArg<String>()
            params[key] ?: emptyList()
        }
        return mockUri
    }

    @Test
    fun `valid invite deep link sets pendingDeepLink without dialing until confirmed`() = runTest {
        val uri = createMockUri(
            scheme = "scmessenger",
            host = "invite",
            params = mapOf(
                "public_key" to listOf(validPublicKey),
                "libp2p_peer_id" to listOf(validPeerId),
                "listeners" to listOf("/ip4/8.8.8.8/tcp/9001"),
                "nickname" to listOf("Alice")
            )
        )

        viewModel.handleDeepLink(uri)
        testDispatcher.scheduler.advanceUntilIdle()

        // Assert NO dial occurs on parse alone
        coVerify(exactly = 0) {
            mockMeshRepository.connectToPeer(any(), any())
        }

        val pending = viewModel.consumeDeepLink()
        assertNotNull(pending)
        assertEquals(validPublicKey, pending?.publicKey)
        assertEquals(validPeerId, pending?.peerId)
        assertEquals(listOf("/ip4/8.8.8.8/tcp/9001"), pending?.listeners)

        // Assert dial DOES occur after explicit confirmation.
        // confirmAndDialPendingDeepLink launches on Dispatchers.IO, which the
        // test scheduler cannot drain via advanceUntilIdle -- poll until the
        // IO coroutine lands its connectToPeer call (CI runners have lost this
        // race under load and failed the verification spuriously).
        viewModel.confirmAndDialPendingDeepLink(pending)
        val deadline = System.currentTimeMillis() + 5_000
        var dialed = false
        while (!dialed && System.currentTimeMillis() < deadline) {
            dialed = try {
                coVerify(exactly = 1) {
                    mockMeshRepository.connectToPeer(validPeerId, listOf("/ip4/8.8.8.8/tcp/9001"))
                }
                true
            } catch (_: AssertionError) {
                Thread.sleep(25)
                false
            }
        }
        assertTrue("connectToPeer was not dialed within 5s of confirmation", dialed)
    }

    @Test
    fun `invite deep link with absent routePeerId does not dial`() = runTest {
        val uri = createMockUri(
            scheme = "scmessenger",
            host = "invite",
            params = mapOf(
                "public_key" to listOf(validPublicKey),
                "listeners" to listOf("/ip4/8.8.8.8/tcp/9001")
            )
        )

        viewModel.handleDeepLink(uri)
        testDispatcher.scheduler.advanceUntilIdle()

        val pending = viewModel.consumeDeepLink()
        assertNotNull(pending)
        assertNull(pending?.peerId)
        assertEquals(emptyList<String>(), pending?.listeners)

        coVerify(exactly = 0) {
            mockMeshRepository.connectToPeer(any(), any())
        }
    }

    @Test
    fun `invite deep link when sanitisation yields empty list does not dial`() = runTest {
        // Loopback 127.0.0.1 is rejected by DeepLinkValidator
        val uri = createMockUri(
            scheme = "scmessenger",
            host = "invite",
            params = mapOf(
                "public_key" to listOf(validPublicKey),
                "libp2p_peer_id" to listOf(validPeerId),
                "listeners" to listOf("/ip4/127.0.0.1/tcp/9001")
            )
        )

        viewModel.handleDeepLink(uri)
        testDispatcher.scheduler.advanceUntilIdle()

        val pending = viewModel.consumeDeepLink()
        assertNotNull(pending)
        assertEquals(emptyList<String>(), pending?.listeners)

        coVerify(exactly = 0) {
            mockMeshRepository.connectToPeer(any(), any())
        }
    }

    @Test
    fun `malformed URI does not crash app`() = runTest {
        val malformedUri = createMockUri(shouldThrowOnAccess = true)

        viewModel.handleDeepLink(malformedUri)
        testDispatcher.scheduler.advanceUntilIdle()

        val pending = viewModel.consumeDeepLink()
        assertNull(pending)

        coVerify(exactly = 0) {
            mockMeshRepository.connectToPeer(any(), any())
        }
    }

    @Test
    fun `isStorageDegraded flow reflects meshRepository`() = runTest {
        val degradedFlow = MutableStateFlow(false)
        every { mockMeshRepository.isStorageDegraded } returns degradedFlow

        viewModel = MainViewModel(
            meshRepository = mockMeshRepository,
            preferencesRepository = mockPreferencesRepository,
            identityCreationCoordinator = mockIdentityCoordinator,
            context = mockContext
        )

        assertEquals(false, viewModel.isStorageDegraded.value)

        degradedFlow.value = true
        assertEquals(true, viewModel.isStorageDegraded.value)
    }

    // ---- #469 T3: shared invite hand-off to the join screen ----

    @Test
    fun `offerInvite keeps only a well-formed SCI1 token and consumeInvite takes it once`() {
        viewModel.offerInvite("hello, join me: SCI1:abcDEF123_-xyz. thanks")

        assertEquals("SCI1:abcDEF123_-xyz", viewModel.pendingInvite.value)
        assertEquals("SCI1:abcDEF123_-xyz", viewModel.consumeInvite())
        assertNull(viewModel.pendingInvite.value)
        assertNull(viewModel.consumeInvite())
    }

    @Test
    fun `offerInvite ignores text without an invite and keeps a pending one`() {
        viewModel.offerInvite("SCI1:firstToken")
        viewModel.offerInvite("just a chat message")
        viewModel.offerInvite(null)
        viewModel.offerInvite("SCI1:")

        assertEquals("SCI1:firstToken", viewModel.pendingInvite.value)
    }

    @Test
    fun `racing consumers get the pending invite exactly once`() {
        viewModel.offerInvite("SCI1:onlyOneWinner")
        val winners = java.util.concurrent.atomic.AtomicInteger(0)
        val start = java.util.concurrent.CountDownLatch(1)
        val threads = (0 until 12).map {
            Thread {
                start.await()
                if (viewModel.consumeInvite() != null) winners.incrementAndGet()
            }.also { t -> t.start() }
        }
        start.countDown()
        threads.forEach { it.join(20_000) }

        assertEquals(1, winners.get())
    }
}
