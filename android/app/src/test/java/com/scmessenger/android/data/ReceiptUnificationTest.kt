package com.scmessenger.android.data

import android.content.Context
import android.content.SharedPreferences
import android.net.ConnectivityManager
import com.scmessenger.android.transport.SmartTransportRouter
import io.mockk.CapturingSlot
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import io.mockk.verify
import java.io.File
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.BeforeClass
import org.junit.Test
import uniffi.api.ContactManager
import uniffi.api.IronCore
import uniffi.api.MeshService
import uniffi.api.ServiceState

/**
 * REGRESSION GUARD for RCA 2026-08-25 (HANDOFF/todo/RCA_DELIVERY_ACK_IMPLEMENTATION_PLAN_2026-08-25.md).
 *
 * Bug: Android's `sendDeliveryReceiptAsync` encoded delivery receipts with
 * `uniffi.api.encodeReceipt()` (bare, unsigned JSON bytes) instead of
 * `ironCore.prepareReceipt()` (signed+encrypted MessageType::Receipt envelope).
 * Windows ingress rejected the undecodable payload, so delivery acks never
 * converged (sender status stayed pending forever).
 *
 * Contract under test: whatever `sendDeliveryReceiptAsync` hands to the
 * transport MUST be the exact byte array returned by
 * `IronCore.prepareReceipt(...)` -- i.e. ENVELOPE bytes -- never bare receipt
 * JSON.
 *
 * Hermetic: pure JVM + MockK. No native core library, no Robolectric. On the
 * buggy code the transport receives JSON (or nothing, when the native lib is
 * absent and `encodeReceipt` cannot link); either way the assertions below
 * fail loudly. On fixed code they pass.
 */
class ReceiptUnificationTest {

    companion object {
        @JvmStatic
        @BeforeClass
        fun plantStdoutTimber() {
            // android.util.Log is stubbed on the JVM; route Timber to stdout so
            // swallowed failures surface in test logs (same convention as
            // com.scmessenger.android.test.ReceiptUnificationTest).
            timber.log.Timber.plant(object : timber.log.Timber.Tree() {
                override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
                    println("TIMBER[$priority] ${tag ?: ""} $message")
                    t?.printStackTrace(System.out)
                }
            })
        }

        /** A syntactically valid Ed25519 public key hex (64 lowercase hex chars). */
        private val SENDER_PUBLIC_KEY_HEX = "ab".repeat(32)

        private val MESSAGE_ID = "msg-envelope-guard-018f3c2e-0001"

        /**
         * Stand-in for signed+encrypted envelope bytes. Deliberately NOT JSON:
         * real envelopes are Drift-signed binary (>= ~200 B); bare receipt JSON
         * is ~70-90 B starting with '{'.
         */
        private val PREPARED_ENVELOPE_BYTES: ByteArray = run {
            val head = byteArrayOf(0x44, 0x52, 0x46, 0x54, 0x01, 0x00) // "DRFT" magic
            head + ByteArray(220) { i -> ((i * 7 + 3) and 0xFF).toByte() }
        }

        /**
         * What the bug actually put on the wire: uniffi.api.encodeReceipt()
         * output shape (canonical receipt JSON).
         */
        private val BARE_RECEIPT_JSON_BYTES = """
            {"message_id":"$MESSAGE_ID","status":"Delivered","timestamp":1700000000}
        """.trimIndent().toByteArray(Charsets.UTF_8)
    }

    private val testRoot = File(System.getProperty("user.dir") ?: ".", "build/tmp/receipt-envelope-guard-tests")

    init {
        testRoot.mkdirs()
    }

    private val activeRepos = mutableListOf<MeshRepository>()

    private fun freshFilesDir(): File =
        File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() }

    private fun fakeContext(filesDir: File): Context =
        mockk<Context>(relaxed = true) {
            every { this@mockk.filesDir } returns filesDir
            every { getSystemService(Context.CONNECTIVITY_SERVICE) } returns
                mockk<ConnectivityManager>(relaxed = true)
            every { getSharedPreferences(any(), any()) } returns
                mockk<SharedPreferences>(relaxed = true)
        }

    private fun trackRepo(repo: MeshRepository): MeshRepository {
        activeRepos += repo
        return repo
    }

    private fun setField(target: Any, name: String, value: Any?) {
        val field = target.javaClass.getDeclaredField(name)
        field.isAccessible = true
        field.set(target, value)
    }

    @Suppress("UNCHECKED_CAST")
    private fun <T> getField(target: Any, name: String): T? {
        val field = target.javaClass.getDeclaredField(name)
        field.isAccessible = true
        return field.get(target) as? T
    }

    private fun cancelRepoScope(repo: MeshRepository) {
        getField<CoroutineScope>(repo, "repoScope")?.cancel()
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

    /** Reflectively invokes the private sendDeliveryReceiptAsync(...) entry point. */
    private fun invokeSendDeliveryReceiptAsync(repo: MeshRepository) {
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
        method.invoke(
            repo,
            SENDER_PUBLIC_KEY_HEX,
            MESSAGE_ID,
            "12D3KooWTestPeeridForReceiptEnvelopeGuard01",
            null,
            null,
            null,
            emptyList<String>()
        )
    }

    private fun awaitSlotCapture(captureSlot: CapturingSlot<ByteArray>, timeoutMs: Long = 8_000L) {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (!captureSlot.isCaptured && System.currentTimeMillis() < deadline) {
            Thread.sleep(50)
        }
    }

    @Test
    fun `send path delivers prepareReceipt envelope bytes to transport not bare receipt json`() = runTest {
        val repo = trackRepo(MeshRepository(fakeContext(freshFilesDir())))

        // RUNNING state keeps ensureServiceInitializedFireAndForget() a no-op
        // (MeshRepository.ensureServiceInitializedDeferred returns immediately),
        // so the test never boots the real service stack.
        val meshService = mockk<MeshService>(relaxed = true) {
            every { getState() } returns ServiceState.RUNNING
        }
        setField(repo, "meshService", meshService)

        val ironCore = mockk<IronCore>(relaxed = true) {
            every { isPeerBlocked(any(), any()) } returns false
            every { prepareReceipt(any(), any()) } returns PREPARED_ENVELOPE_BYTES
        }
        setField(repo, "ironCore", ironCore)
        setField(repo, "contactManager", mockk<ContactManager>(relaxed = true))

        // Capture the exact bytes the receipt path hands to the transport layer.
        val envelopeSlot = slot<ByteArray>()
        val router = mockk<SmartTransportRouter>()
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
        setField(repo, "smartTransportRouter", router)

        invokeSendDeliveryReceiptAsync(repo)
        awaitSlotCapture(envelopeSlot)

        assertTrue(
            "Receipt bytes were never handed to the transport (encode step failed or path dead)",
            envelopeSlot.isCaptured
        )
        val sentBytes = envelopeSlot.captured

        // 1. The transport received EXACTLY what prepareReceipt produced.
        assertArrayEquals(
            "Transport must receive the prepareReceipt() envelope bytes verbatim",
            PREPARED_ENVELOPE_BYTES,
            sentBytes
        )

        // 2. prepareReceipt was the encoding mechanism (right recipient key, right id).
        verify(exactly = 1) {
            ironCore.prepareReceipt(SENDER_PUBLIC_KEY_HEX, MESSAGE_ID)
        }

        // 3. The bytes are NOT bare receipt JSON -- the exact bug this guards.
        assertFalse(
            "Receipt bytes must not be a JSON object (bare encodeReceipt() payload leaked onto the wire)",
            sentBytes.firstOrNull() == '{'.code.toByte()
        )
        val asText = String(sentBytes, Charsets.UTF_8).trim()
        assertFalse(
            "Receipt bytes must not parse as a receipt JSON object",
            asText.startsWith("{") && asText.endsWith("}")
        )
        assertFalse(
            "Receipt bytes must not embed receipt JSON fields",
            asText.contains("\"message_id\"") || asText.contains("\"messageId\"")
        )

        cancelRepoScope(repo)
    }

    @Test
    fun `guard itself flags the historical bare-json wire format`() {
        // Documents the bug signature: the old wire payload (~80 B canonical
        // receipt JSON) must trip every bare-JSON detector used above, so the
        // guard cannot silently rot into accepting the broken format again.
        assertTrue(BARE_RECEIPT_JSON_BYTES.firstOrNull() == '{'.code.toByte())
        val asText = String(BARE_RECEIPT_JSON_BYTES, Charsets.UTF_8).trim()
        assertTrue(asText.startsWith("{") && asText.endsWith("}"))
        assertTrue(asText.contains("\"message_id\""))
        // And the prepared envelope must look nothing like it.
        assertFalse(PREPARED_ENVELOPE_BYTES.contentEquals(BARE_RECEIPT_JSON_BYTES))
    }
}
