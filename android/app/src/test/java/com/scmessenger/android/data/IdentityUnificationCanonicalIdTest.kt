package com.scmessenger.android.data

import android.content.Context
import android.content.SharedPreferences
import android.net.ConnectivityManager
import io.mockk.every
import io.mockk.mockk
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assert.assertNull
import org.junit.Test
import com.scmessenger.android.utils.PeerIdValidator
import uniffi.api.Contact
import uniffi.api.ContactManager
import uniffi.api.IronCore

/**
 * REGRESSION GUARD for the D4 identity-split: ack messages from the Windows node
 * arrived attributed to the identity_id (blake3 hash, e.g. `985a25f9...`) instead
 * of the unified canonical public key (`30d0fa67...`), while the ledger merged the
 * same pair to the public key. The message path and the ledger disagreed.
 *
 * Root cause (two compounding defects, confirmed from live device logs):
 *  1. `resolveCanonicalPeerIdFromMessageHints` final fallback returned the hint's
 *     identity_id whenever `resolvedCanonicalPeerId == senderId` — but when the
 *     wire senderId IS already the canonical public key, that condition fires and
 *     downgrades the canonical key to the derived hash.
 *  2. The Pixel's contact record for `30d0fa67` had a BLANK publicKey, and
 *     `upsertFederatedContact`'s auth guard treated blank as "key mismatch" and
 *     rejected the fill — so the by-key lookup never matched and the fallback
 *     fired on every inbound message.
 *
 * Hermetic: pure JVM + MockK. No native core library, no Robolectric.
 */
private class HermeticIdentityRepo(context: Context) : MeshRepository(context) {
    override fun initializeManagers() { /* native managers unavailable on JVM */ }
}

class IdentityUnificationCanonicalIdTest {

    companion object {
        /** The Windows node's canonical public key (matches the live ledger). */
        private val CANONICAL_PUBKEY =
            "30d0fa678c218b225bd9c20c262b2aededc9e8cd5cd44c45187f8d71bf05967e"

        /** The same node's derived identity_id (blake3 of the raw pubkey bytes). */
        private val DERIVED_IDENTITY_ID =
            "985a25f9505372de3eeea4fe6220784a956da88cf6681f57f9e5ffd92bf65826"

        private val LEGACY_LIBP2P_SENDER =
            "12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw"
    }

    private val testRoot = File(System.getProperty("user.dir") ?: ".", "build/tmp/identity-unification-tests")

    init {
        testRoot.mkdirs()
    }

    private fun fakeContext(filesDir: File): Context =
        mockk<Context>(relaxed = true) {
            every { this@mockk.filesDir } returns filesDir
            every { getSystemService(Context.CONNECTIVITY_SERVICE) } returns
                mockk<ConnectivityManager>(relaxed = true)
            every { getSharedPreferences(any(), any()) } returns
                mockk<SharedPreferences>(relaxed = true)
        }

    private fun setField(target: Any, name: String, value: Any?) {
        val field = MeshRepository::class.java.getDeclaredField(name)
        field.isAccessible = true
        field.set(target, value)
    }

    /** Reflectively invokes the private resolveCanonicalPeerId entry point. */
    private fun invokeResolveCanonicalPeerId(
        repo: MeshRepository,
        senderId: String,
        senderPublicKeyHex: String
    ): String {
        val method = MeshRepository::class.java.getDeclaredMethod(
            "resolveCanonicalPeerId",
            String::class.java,
            String::class.java
        )
        method.isAccessible = true
        return method.invoke(repo, senderId, senderPublicKeyHex) as String
    }

    /** Reflectively invokes the private resolveCanonicalPeerIdFromMessageHints entry point. */
    private fun invokeResolveCanonicalPeerIdFromMessageHints(
        repo: MeshRepository,
        resolvedCanonicalPeerId: String,
        senderId: String,
        senderPublicKeyHex: String,
        hintedIdentityId: String?
    ): String {
        val method = MeshRepository::class.java.getDeclaredMethod(
            "resolveCanonicalPeerIdFromMessageHints",
            String::class.java,
            String::class.java,
            String::class.java,
            String::class.java
        )
        method.isAccessible = true
        return method.invoke(
            repo,
            resolvedCanonicalPeerId,
            senderId,
            senderPublicKeyHex,
            hintedIdentityId
        ) as String
    }

    private fun repoWithBlankKeyContact(): Pair<MeshRepository, Contact> {
        val repo = HermeticIdentityRepo(fakeContext(File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() }))

        // Live defect: the Pixel's contact for the Windows node is keyed by the
        // canonical pubkey but its publicKey field is BLANK (ledger/discovery-created
        // before the key was known), so by-key lookups never match.
        val blankKeyContact = Contact(
            peerId = CANONICAL_PUBKEY,
            nickname = null,
            localNickname = null,
            publicKey = "",
            addedAt = 1u,
            lastSeen = null,
            notes = null,
            lastKnownDeviceId = null,
            verifiedAt = null,
            isTombstone = false
        )
        val contactManager = mockk<ContactManager>(relaxed = true) {
            every { list() } returns listOf(blankKeyContact)
        }
        val ironCore = mockk<IronCore>(relaxed = true) {
            // resolveIdentity on a valid pubkey returns the pubkey itself.
            every { resolveIdentity(CANONICAL_PUBKEY) } returns CANONICAL_PUBKEY
        }
        setField(repo, "contactManager", contactManager)
        setField(repo, "ironCore", ironCore)
        return repo to blankKeyContact
    }

    @Test
    fun `message path keeps canonical pubkey when hint identity_id differs`() {
        val (repo, _) = repoWithBlankKeyContact()

        // Priority-1 resolution: senderId IS the canonical pubkey.
        val resolved = invokeResolveCanonicalPeerId(repo, CANONICAL_PUBKEY, CANONICAL_PUBKEY)
        assertEquals(CANONICAL_PUBKEY, resolved)

        // Hint downgrade must NOT happen: identity_id hint != resolved canonical.
        val finalCanonical = invokeResolveCanonicalPeerIdFromMessageHints(
            repo,
            resolvedCanonicalPeerId = CANONICAL_PUBKEY,
            senderId = CANONICAL_PUBKEY,
            senderPublicKeyHex = CANONICAL_PUBKEY,
            hintedIdentityId = DERIVED_IDENTITY_ID
        )
        assertEquals(
            "Canonical pubkey must not be downgraded to the hint identity_id",
            CANONICAL_PUBKEY,
            finalCanonical
        )
    }

    @Test
    fun `legacy libp2p sender still prefers hint identity_id`() {
        val repo = HermeticIdentityRepo(
            fakeContext(File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() })
        )
        setField(repo, "contactManager", mockk<ContactManager>(relaxed = true) {
            every { list() } returns emptyList()
        })

        // Legacy behavior preserved: a libp2p routing ID is NOT a canonical 64-hex
        // identity, so the hint identity_id remains the preferred canonical target.
        val finalCanonical = invokeResolveCanonicalPeerIdFromMessageHints(
            repo,
            resolvedCanonicalPeerId = LEGACY_LIBP2P_SENDER,
            senderId = LEGACY_LIBP2P_SENDER,
            senderPublicKeyHex = CANONICAL_PUBKEY,
            hintedIdentityId = DERIVED_IDENTITY_ID
        )
        assertEquals(DERIVED_IDENTITY_ID, finalCanonical)
    }

    @Test
    fun `blank stored contact key is not a federated key conflict`() {
        // Blank/null stored key = record predates the key; the verified incoming
        // key must be allowed to fill it (previously rejected as "key mismatch").
        assertFalse(MeshRepository.federatedKeyConflict("", CANONICAL_PUBKEY))
        assertFalse(MeshRepository.federatedKeyConflict(null, CANONICAL_PUBKEY))
        assertFalse(MeshRepository.federatedKeyConflict("  ", CANONICAL_PUBKEY))
    }

    @Test
    fun `matching stored key is not a conflict and mismatching key is`() {
        assertFalse(MeshRepository.federatedKeyConflict(CANONICAL_PUBKEY, CANONICAL_PUBKEY))
        assertTrue(MeshRepository.federatedKeyConflict(CANONICAL_PUBKEY, DERIVED_IDENTITY_ID))
    }

    /** Reflectively invokes the private isSelfCertifyingKeyBinding entry point. */
    private fun invokeIsSelfCertifyingKeyBinding(
        repo: MeshRepository,
        peerId: String,
        publicKey: String
    ): Boolean {
        val method = MeshRepository::class.java.getDeclaredMethod(
            "isSelfCertifyingKeyBinding",
            String::class.java,
            String::class.java
        )
        method.isAccessible = true
        return method.invoke(repo, peerId, publicKey) as Boolean
    }

    @Test
    fun `canonical hex peer_id bound to itself is self-certifying`() {
        // REGRESSION (ledger-share of the AWS parity node): the ledger stores
        // peer_id as the 64-hex canonical public key itself (e.g. 014b8105...),
        // matching every Windows entry (30d0fa67...). isSelfCertifyingKeyBinding
        // previously only accepted the base58-derived form, so the canonical hex
        // binding was classified "poisoned" and stripped on every cold start —
        // the app then had no AWS seed ("no proven ledger relay candidates").
        val repo = HermeticIdentityRepo(
            fakeContext(File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() })
        )
        assertTrue(
            "hex peer_id == hex public_key must be self-certifying",
            invokeIsSelfCertifyingKeyBinding(repo, CANONICAL_PUBKEY, CANONICAL_PUBKEY)
        )
    }

    @Test
    fun `genuinely unrelated binding stays poisoned`() {
        val repo = HermeticIdentityRepo(
            fakeContext(File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() })
        )
        // A different node's hex key bound as peer_id of this entry is NOT
        // self-certifying and must keep being dropped (the poison path remains).
        val unrelatedPeerId =
            "014b81057387384c89570a652ea604967388b0f67573d9e31d5719aab3f58cc8"
        assertFalse(
            "mismatched hex binding must stay poisoned",
            invokeIsSelfCertifyingKeyBinding(repo, unrelatedPeerId, CANONICAL_PUBKEY)
        )
        // A libp2p base58 peer id that does not derive from the key also stays poisoned.
        assertFalse(
            "unrelated base58 peer id must stay poisoned",
            invokeIsSelfCertifyingKeyBinding(
                repo,
                "12D3KooWNkx3AjDmXDHpweEsnNm164MS23nuMRVLajgaASyxBrow",
                CANONICAL_PUBKEY
            )
        )
    }

    @Test
    fun `base58 peer id derived from key is still self-certifying`() {
        val repo = HermeticIdentityRepo(
            fakeContext(File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() })
        )
        assertTrue(
            "base58-derived binding must remain self-certifying",
            invokeIsSelfCertifyingKeyBinding(repo, LEGACY_LIBP2P_SENDER, CANONICAL_PUBKEY)
        )
    }

    /** Reflectively invokes the private toDialableRoutePeerId entry point. */
    private fun invokeToDialableRoutePeerId(
        repo: MeshRepository,
        peerId: String,
        recipientPublicKey: String?
    ): String? {
        val method = MeshRepository::class.java.getDeclaredMethod(
            "toDialableRoutePeerId",
            String::class.java,
            String::class.java
        )
        method.isAccessible = true
        return method.invoke(repo, peerId, recipientPublicKey) as String?
    }

    @Test
    fun `hex canonical peer id is dialable via derived libp2p form`() {
        // REGRESSION (dial-layer, ledger-share of the AWS parity node): the ledger
        // stores peer_id as the 64-hex canonical public key (014b8105...). The
        // route-candidate builder previously rejected it with
        // `buildRoutePeerCandidates: peer_id=invalid_format`, so AWS could never be
        // dialed even after the sanitizer/discovery fixes. Now the self-certifying
        // hex form must yield the dialable libp2p peer id.
        val repo = HermeticIdentityRepo(
            fakeContext(File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() })
        )
        val awsHexKey =
            "014b81057387384c89570a652ea604967388b0f67573d9e31d5719aab3f58cc8"
        val dialable = invokeToDialableRoutePeerId(repo, awsHexKey, awsHexKey)
        assertTrue(
            "hex-bound self-certifying peer_id must derive to a libp2p peer id",
            dialable != null && PeerIdValidator.isLibp2pPeerId(dialable)
        )
        // The derived id must re-extract back to the same canonical key (the
        // recipient), proving it is the same node's dialable id.
        val reRecipient = invokeToDialableRoutePeerId(repo, dialable!!, awsHexKey)
        assertEquals(dialable, reRecipient)
    }

    @Test
    fun `unrelated hex peer id stays un-dialable`() {
        // A canonical hex id that does NOT certify against the recipient key must
        // still yield null (dial rejection preserved, matching the poison path).
        val repo = HermeticIdentityRepo(
            fakeContext(File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() })
        )
        val unrelatedHex =
            "f1c4f2aa5f3a47fbbc9f3b0a1d2c4e5d6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d"
        assertNull(
            "hex peer_id unrelated to the recipient must not become dialable",
            invokeToDialableRoutePeerId(repo, unrelatedHex, CANONICAL_PUBKEY)
        )
    }

    @Test
    fun `random non-id string stays un-dialable`() {
        val repo = HermeticIdentityRepo(
            fakeContext(File(testRoot, "test-${System.nanoTime()}").apply { mkdirs() })
        )
        assertNull(invokeToDialableRoutePeerId(repo, "not-a-peer-id", CANONICAL_PUBKEY))
        assertNull(invokeToDialableRoutePeerId(repo, "", CANONICAL_PUBKEY))
    }
}
