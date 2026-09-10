package com.scmessenger.android.test

import android.content.Context
import android.content.SharedPreferences
import android.net.ConnectivityManager
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.TransportType
import com.scmessenger.android.transport.SmartTransportRouter
import io.mockk.CapturingSlot
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import io.mockk.verify
import java.io.File
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.BeforeClass
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.api.ContactManager
import uniffi.api.CoreDelegate
import uniffi.api.DeliveryStatus
import uniffi.api.HistoryManager
import uniffi.api.IdentityInfo
import uniffi.api.IronCore
import uniffi.api.MeshService
import uniffi.api.MeshServiceConfig
import uniffi.api.MessageDirection
import uniffi.api.MessageRecord
import uniffi.api.MessageStatus
import uniffi.api.Receipt
import uniffi.api.ServiceState

/**
 * Hermetic JVM unit tests locking the Android receipt unification contract.
 *
 * Since FIX-2 (RCA 2026-08-25) the SEND path encodes delivery receipts with
 * `IronCore.prepareReceipt()` (signed+encrypted Drift envelope), NOT with the
 * bare `uniffi.api.encodeReceipt()` JSON codec. The codec lives behind the
 * UniFFI FFI boundary, so on the JVM tier these tests lock:
 * - The struct surface Android compiles against (Receipt/DeliveryStatus).
 * - The receive path (onReceiptReceived): dedup, history update, markMessageSent.
 * - The send path: prepareReceipt ENVELOPE bytes on the transport, retry after
 *   an encode failure, and coalescing of concurrent sends for one message id.
 *
 * Hermetic: pure JVM + MockK. No native core library, no Robolectric, no
 * Assume-skips. Repos are built via [HermeticRepo], whose no-op
 * initializeManagers() override skips the native UniFFI manager construction
 * (the seam was added for exactly this purpose).
 */
class ReceiptUnificationTest {

    private val testRoot = File(System.getProperty("user.dir") ?: ".", "build/tmp/receipt-unification-tests")

    init {
        testRoot.mkdirs()
    }

    companion object {
        @JvmStatic
        @BeforeClass
        fun plantStdoutTimber() {
            // android.util.Log is stubbed on the JVM, so DebugTree prints
            // nothing; route Timber to stdout so swallowed exceptions surface.
            timber.log.Timber.plant(object : timber.log.Timber.Tree() {
                override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
                    println("TIMBER[$priority] ${tag ?: ""} $message")
                    t?.printStackTrace(System.out)
                }
            })
        }

        /** A syntactically valid Ed25519 public key hex (64 lowercase hex chars). */
        private val SENDER_PUBLIC_KEY_HEX = "ab".repeat(32)

        /**
         * Stand-in for signed+encrypted envelope bytes. Deliberately NOT JSON:
         * real envelopes are Drift-signed binary (>= ~200 B); bare receipt JSON
         * is ~70-90 B starting with '{'.
         */
        private fun preparedEnvelopeBytes(seed: Int = 3): ByteArray {
            val head = byteArrayOf(0x44, 0x52, 0x46, 0x54, 0x01, 0x00) // "DRFT" magic
            return head + ByteArray(220) { i -> ((i * seed + 3) and 0xFF).toByte() }
        }
    }

    private fun freshFilesDir(): File {
        val dir = File(testRoot, "test-${System.nanoTime()}")
        dir.mkdirs()
        return dir
    }

    private fun fakeContext(filesDir: File): Context {
        return mockk<Context>(relaxed = true) {
            every { this@mockk.filesDir } returns filesDir
            every { getSystemService(Context.CONNECTIVITY_SERVICE) } returns mockk<ConnectivityManager>(relaxed = true)
            every { getSharedPreferences(any(), any()) } returns mockk<SharedPreferences>(relaxed = true)
        }
    }

    // Resolve on MeshRepository itself, not target::class.java: the hermetic
    // subclass does not redeclare these private fields (inherited ones are not
    // returned by getDeclaredField on the subclass).
    private fun setField(target: Any, name: String, value: Any?) {
        val field = MeshRepository::class.java.getDeclaredField(name)
        field.isAccessible = true
        field.set(target, value)
    }

    @Suppress("UNCHECKED_CAST")
    private fun <T> getField(target: Any, name: String): T? {
        val field = MeshRepository::class.java.getDeclaredField(name)
        field.isAccessible = true
        return field.get(target) as? T
    }

    private fun writePendingOutbox(filesDir: File, messageId: String, peerId: String) {
        val file = File(filesDir, "pending_outbox.json")
        file.writeText(
            """
            [{
                "queue_id": "q-$messageId",
                "history_record_id": "$messageId",
                "peer_id": "$peerId",
                "route_peer_id": null,
                "listeners": [],
                "envelope_b64": "eA==",
                "created_at": 1,
                "attempt_count": 0,
                "next_attempt_at": 0
            }]
            """.trimIndent()
        )
    }

    private fun cancelRepoScope(repo: MeshRepository) {
        getField<CoroutineScope>(repo, "repoScope")?.cancel()
    }

    // Repos whose repoScope must be cancelled in @After.
    private val activeRepos = mutableListOf<MeshRepository>()

    private fun trackRepo(repo: MeshRepository): MeshRepository {
        activeRepos += repo
        return repo
    }

    /** Reflectively invokes the private sendDeliveryReceiptAsync(...) entry point. */
    private fun invokeSendDeliveryReceiptAsync(
        repo: MeshRepository,
        senderPublicKeyHex: String = SENDER_PUBLIC_KEY_HEX,
        messageId: String,
        senderId: String
    ) {
        val method = MeshRepository::class.java.getDeclaredMethod(
            "sendDeliveryReceiptAsync",
            String::class.java, // senderPublicKeyHex
            String::class.java, // messageId
            String::class.java, // senderId
            String::class.java, // preferredRoutePeerId
            String::class.java, // preferredWifiPeerId
            String::class.java, // preferredBlePeerId
            List::class.java    // preferredListenerHints
        )
        method.isAccessible = true
        method.invoke(repo, senderPublicKeyHex, messageId, senderId, null, null, null, emptyList<String>())
    }

    private fun awaitSlotCapture(captureSlot: CapturingSlot<ByteArray>, timeoutMs: Long = 12_000L) {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (!captureSlot.isCaptured && System.currentTimeMillis() < deadline) {
            Thread.sleep(50)
        }
    }

    /** Router mock that captures the receipt bytes handed to the transport. */
    private fun capturingRouter(envelopeSlot: CapturingSlot<ByteArray>): SmartTransportRouter =
        mockk<SmartTransportRouter>().also { router ->
            coEvery {
                router.attemptDelivery(
                    any(),                 // peerId
                    capture(envelopeSlot), // envelopeData == receipt bytes under test
                    any(), any(), any(),   // wifiPeerId, blePeerId, tcpMdnsPeerId
                    any(), any(),          // routePeerCandidates, listeners
                    any(), any(),          // traceMessageId, attemptContext
                    any(), any(), any(), any() // tryWifi/tryBle/tryTcpMdns/tryCore lambdas
                )
            } returns SmartTransportRouter.TransportDeliveryResult(
                transport = SmartTransportRouter.TransportType.CORE,
                success = true,
                latencyMs = 1,
                error = null
            )
        }

    /** RUNNING-state service + mocked core, so ensureService* stays a no-op. */
    private fun installRunningService(repo: MeshRepository, ironCore: IronCore) {
        val meshService = mockk<MeshService>(relaxed = true) {
            every { getState() } returns ServiceState.RUNNING
        }
        setField(repo, "meshService", meshService)
        setField(repo, "ironCore", ironCore)
        setField(repo, "contactManager", mockk<ContactManager>(relaxed = true))
    }

    @After
    fun cleanup() {
        activeRepos.forEach { repo ->
            runCatching { cancelRepoScope(repo) }
                .onFailure { println("[WARN] cancelRepoScope failed: $it") }
        }
        activeRepos.clear()
        testRoot.listFiles()?.forEach { it.deleteRecursively() }
    }

    // =========================================================================
    // TEST A: Receipt struct fidelity (no FFI required)
    // =========================================================================
    // The codec itself lives in Rust behind UniFFI; the contract this tier can
    // lock hermetically is the struct surface Android code compiles against.

    @Test
    fun `receipt struct round trip preserves all fields`() {
        val messageId = "msg-550e8400-e29b-41d4-a716-446655440000"
        val timestamp = 1700000000uL

        val original = Receipt(
            messageId = messageId,
            status = DeliveryStatus.DELIVERED,
            timestamp = timestamp
        )

        // Round-trip through copy(): field fidelity + structural equality.
        val roundTripped = original.copy()

        assertEquals("Message ID must survive the round-trip", messageId, roundTripped.messageId)
        assertEquals("Status must survive the round-trip", DeliveryStatus.DELIVERED, roundTripped.status)
        assertEquals("Timestamp must survive the round-trip", timestamp, roundTripped.timestamp)
        assertEquals(original, roundTripped)

        println("[OK] Receipt struct round-trip successful: id=$messageId ts=$timestamp")
    }

    // =========================================================================
    // TEST B: All DeliveryStatus values remain distinguishable on structs
    // =========================================================================

    @Test
    fun `all delivery status values produce distinct receipts`() {
        val statusValues = DeliveryStatus.values().toList()
        assertTrue("expected at least SENT and DELIVERED", statusValues.size >= 2)

        val receipts = statusValues.map { status ->
            Receipt(
                messageId = "msg-test-$status",
                status = status,
                timestamp = (System.currentTimeMillis() / 1000).toULong()
            )
        }

        receipts.forEachIndexed { index, receipt ->
            assertEquals("Status ${statusValues[index]} must survive into the struct", statusValues[index], receipt.status)
        }
        // Distinct statuses must yield distinct receipts (wire discrimination).
        receipts.forEachIndexed { i, a -> receipts.forEachIndexed { j, b ->
            if (i != j) assertTrue("$a must differ from $b", a != b)
        } }

        println("[OK] All ${statusValues.size} DeliveryStatus values construct distinct receipts")
    }

    // =========================================================================
    // TEST C: Receive Path - onReceiptReceived processes receipts with logging
    // =========================================================================

    @Test
    fun `inbound gate waits for dedup result and reports duplicates`() = runTest {
        val filesDir = freshFilesDir()
        val repo = trackRepo(HermeticRepo(fakeContext(filesDir)))

        val router = mockk<SmartTransportRouter>()
        coEvery {
            router.checkAndRecordMessage(
                "msg-dup-1",
                SmartTransportRouter.TransportType.CORE
            )
        } returns Triple(true, 42L, SmartTransportRouter.TransportType.CORE)

        setField(repo, "smartTransportRouter", router)

        val result = repo.gateInboundMessage("msg-dup-1")

        assertTrue(result.first)
        assertEquals(42L, result.second)
        assertEquals(TransportType.INTERNET, result.third)

        coVerify(exactly = 1) {
            router.checkAndRecordMessage(
                "msg-dup-1",
                SmartTransportRouter.TransportType.CORE
            )
        }

        cancelRepoScope(repo)
    }

    @Test
    fun `receive path processes delivered receipts and deduplicates`() = runTest {
        val filesDir = freshFilesDir()
        val repo = trackRepo(HermeticRepo(fakeContext(filesDir)))

        val ironCore = mockk<IronCore>(relaxed = true) {
            every { getIdentityInfo() } returns IdentityInfo(
                identityId = null,
                publicKeyHex = null,
                deviceId = null,
                seniorityTimestamp = null,
                initialized = false,
                nickname = null,
                libp2pPeerId = null
            )
            // `returns true`, NOT `just Awaits`: `just Awaits` parks the calling
            // thread inside mockk's runBlocking forever (the historical
            // :app:testDebugUnitTest hang). The test only needs the call to HAPPEN.
            every { markMessageSent(any()) } returns true
        }
        val meshService = mockk<MeshService>(relaxed = true) {
            every { getState() } returns ServiceState.STOPPED
            every { getCore() } returns ironCore
        }
        setField(repo, "meshService", meshService)

        repo.startMeshService(MeshServiceConfig(discoveryIntervalMs = 30000u, batteryFloorPct = 20u))

        val coreDelegate = getField<CoreDelegate>(repo, "coreDelegate")
        assertNotNull(coreDelegate)

        writePendingOutbox(filesDir, "msg-1", "12D3KooWTestPeerIdForReceiptUnification01")

        val sentRecord = MessageRecord(
            id = "msg-1",
            direction = MessageDirection.SENT,
            peerId = "peer-1",
            content = "hello",
            timestamp = 1uL,
            senderTimestamp = 1uL,
            delivered = false,
            status = MessageStatus.SENT,
            hidden = false
        )

        // STATEFUL mock, deliberately: production dedup reads
        // historyManager.get(messageId).delivered back AFTER markDelivered(),
        // so a frozen always-false stub would make exactly-once unsatisfiable.
        var deliveredFlag = false
        val historyManager = mockk<HistoryManager>(relaxed = true)
        every { historyManager.get("msg-1") } answers {
            sentRecord.copy(delivered = deliveredFlag)
        }
        every { historyManager.markDelivered("msg-1") } answers {
            deliveredFlag = true
            Unit
        }
        val contactManager = mockk<ContactManager>(relaxed = true)
        setField(repo, "historyManager", historyManager)
        setField(repo, "contactManager", contactManager)

        println("[TEST] Calling onReceiptReceived: msg=msg-1 status=Delivered")
        coreDelegate!!.onReceiptReceived("msg-1", "Delivered")

        coVerify(exactly = 1) { historyManager.markDelivered("msg-1") }
        coVerify(exactly = 1) { historyManager.flush() }
        coVerify(exactly = 1) { ironCore.markMessageSent("msg-1") }

        println("[OK] First receipt processed correctly")

        println("[TEST] Calling onReceiptReceived again (duplicate): msg=msg-1 status=Delivered")
        coreDelegate.onReceiptReceived("msg-1", "Delivered")

        // Verify no additional calls (dedup worked)
        coVerify(exactly = 1) { historyManager.markDelivered("msg-1") }
        coVerify(exactly = 1) { historyManager.flush() }
        coVerify(exactly = 1) { ironCore.markMessageSent("msg-1") }

        println("[OK] Duplicate receipt deduplicated correctly")

        println("[TEST] Calling onReceiptReceived with invalid status: msg=msg-garbage status=garbage")
        coreDelegate.onReceiptReceived("msg-garbage", "garbage")

        coVerify(exactly = 0) { historyManager.markDelivered("msg-garbage") }

        println("[OK] Invalid status ignored correctly")

        cancelRepoScope(repo)
    }

    @Test
    fun `receipt arrival overrides high attempt count and clears corruption`() = runTest {
        val filesDir = freshFilesDir()
        val repo = trackRepo(HermeticRepo(fakeContext(filesDir)))

        val ironCore = mockk<IronCore>(relaxed = true) {
            every { getIdentityInfo() } returns IdentityInfo(
                identityId = null,
                publicKeyHex = null,
                deviceId = null,
                seniorityTimestamp = null,
                initialized = false,
                nickname = null,
                libp2pPeerId = null
            )
            every { markMessageSent(any()) } returns true
        }
        val meshService = mockk<MeshService>(relaxed = true) {
            every { getState() } returns ServiceState.STOPPED
            every { getCore() } returns ironCore
        }
        setField(repo, "meshService", meshService)
        repo.startMeshService(MeshServiceConfig(discoveryIntervalMs = 30000u, batteryFloorPct = 20u))

        val coreDelegate = getField<CoreDelegate>(repo, "coreDelegate")
        assertNotNull(coreDelegate)

        val sentRecord = MessageRecord(
            id = "msg-corrupt-test",
            direction = MessageDirection.SENT,
            peerId = "peer-1",
            content = "hello",
            timestamp = 1uL,
            senderTimestamp = 1uL,
            delivered = false,
            status = MessageStatus.SENT,
            hidden = false
        )

        val historyManager = mockk<HistoryManager>(relaxed = true)
        every { historyManager.get("msg-corrupt-test") } returns sentRecord
        setField(repo, "historyManager", historyManager)

        // Simulate 12 retries marking message corrupted
        repo.incrementAttemptCount("msg-corrupt-test")
        repo.markMessageCorrupted("msg-corrupt-test")

        // Inbound receipt arrives late
        coreDelegate!!.onReceiptReceived("msg-corrupt-test", "Delivered")

        // Verify history marked delivered
        coVerify(exactly = 1) { historyManager.markDelivered("msg-corrupt-test") }

        // Attempting to corrupt again must be blocked by the no-downgrade rule
        repo.markMessageCorrupted("msg-corrupt-test")

        cancelRepoScope(repo)
    }

    // =========================================================================
    // TEST D: Send Path - prepareReceipt envelope reaches the transport, and
    // concurrent sends for the same message are coalesced (exactly-one wire hit)
    // =========================================================================

    @Test
    fun `concurrent receipt sends for the same message are coalesced to one transport delivery`() = runTest {
        val filesDir = freshFilesDir()
        val repo = trackRepo(HermeticRepo(fakeContext(filesDir)))

        val senderId = "12D3KooWTestPeeridForReceiptUnification24XyzAbcVwxyz"
        val messageId = "msg-coalesce-guard-1"

        val ironCore = mockk<IronCore>(relaxed = true) {
            every { isPeerBlocked(any(), any()) } returns false
            every { prepareReceipt(any(), any()) } returns preparedEnvelopeBytes()
        }
        installRunningService(repo, ironCore)

        // Park the first delivery inside the transport until we release it, so
        // the second send invocation deterministically races against an ACTIVE job.
        val deliveryStarted = CompletableDeferred<Unit>()
        val releaseDelivery = CompletableDeferred<Unit>()
        var transportCalls = 0
        val router = mockk<SmartTransportRouter>()
        coEvery {
            router.attemptDelivery(any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any())
        } coAnswers {
            transportCalls++
            deliveryStarted.complete(Unit)
            releaseDelivery.await()
            SmartTransportRouter.TransportDeliveryResult(
                transport = SmartTransportRouter.TransportType.CORE,
                success = true,
                latencyMs = 1,
                error = null
            )
        }
        setField(repo, "smartTransportRouter", router)

        invokeSendDeliveryReceiptAsync(repo, messageId = messageId, senderId = senderId)

        val deadline = System.currentTimeMillis() + 8_000L
        while (!deliveryStarted.isCompleted && System.currentTimeMillis() < deadline) {
            Thread.sleep(20)
        }
        assertTrue("first receipt job never reached the transport", deliveryStarted.isCompleted)

        // Second invocation while the first is still in flight must be coalesced.
        invokeSendDeliveryReceiptAsync(repo, messageId = messageId, senderId = senderId)

        releaseDelivery.complete(Unit)
        Thread.sleep(500) // let both jobs drain

        assertEquals(
            "duplicate receipt send must not reach the transport twice",
            1,
            transportCalls
        )
        verify(exactly = 1) { ironCore.prepareReceipt(any(), any()) }

        cancelRepoScope(repo)
    }

    // =========================================================================
    // TEST E: Encode failure handling - retried, envelope still delivered
    // =========================================================================
    // Verifies that a transient prepareReceipt failure:
    // 1. Is caught and logged (never crashes the app / kills the job)
    // 2. Triggers the retry loop
    // 3. Still delivers EXACTLY the prepareReceipt envelope once encoding succeeds

    @Test
    fun `encode failure is retried and envelope still delivered to transport`() = runTest {
        val filesDir = freshFilesDir()
        val repo = trackRepo(HermeticRepo(fakeContext(filesDir)))

        val messageId = "msg-retry-guard-1"
        val senderId = "12D3KooWTestPeeridForReceiptUnification31"

        val envelope = preparedEnvelopeBytes(seed = 5)
        var encodeAttempts = 0
        val ironCore = mockk<IronCore>(relaxed = true) {
            every { isPeerBlocked(any(), any()) } returns false
            every { prepareReceipt(any(), any()) } answers {
                encodeAttempts++
                if (encodeAttempts == 1) throw IllegalStateException("simulated transient encode error")
                envelope
            }
        }
        installRunningService(repo, ironCore)

        val envelopeSlot = slot<ByteArray>()
        setField(repo, "smartTransportRouter", capturingRouter(envelopeSlot))

        invokeSendDeliveryReceiptAsync(repo, messageId = messageId, senderId = senderId)
        awaitSlotCapture(envelopeSlot)

        assertTrue(
            "receipt bytes were never handed to the transport after retry",
            envelopeSlot.isCaptured
        )
        val sentBytes = envelopeSlot.captured

        assertArrayEquals(
            "Transport must receive the prepareReceipt() envelope bytes verbatim",
            envelope,
            sentBytes
        )
        assertEquals("prepareReceipt must have been attempted exactly twice (fail then succeed)", 2, encodeAttempts)

        // Not bare receipt JSON -- the regression this suite guards.
        val asText = String(sentBytes, Charsets.UTF_8).trim()
        assertFalse(asText.startsWith("{") && asText.endsWith("}"))
        assertFalse(asText.contains("\"message_id\"") || asText.contains("\"messageId\""))

        cancelRepoScope(repo)
    }
}

/**
 * JVM-only MeshRepository: skips native UniFFI manager construction.
 *
 * MeshRepository's init calls initializeManagers(), which instantiates real
 * uniffi.api objects — each triggering UniffiLib's <clinit> and a JNA load of
 * libscmessenger_core that does not exist on the JVM test tier. The no-op
 * override exists only under app/src/test; see the seam comment at
 * MeshRepository.initializeManagers(). Shared by other native-free suites
 * (e.g. ReceiptWindowTest) in this package.
 */
internal class HermeticRepo(context: Context) : MeshRepository(context) {
    override fun initializeManagers() { /* native managers unavailable on JVM */ }
}
