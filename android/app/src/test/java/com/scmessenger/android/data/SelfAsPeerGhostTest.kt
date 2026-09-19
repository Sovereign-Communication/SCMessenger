package com.scmessenger.android.data

import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * SELF-AS-PEER-001 (2026-09-11): own public key appearing as a ledger peer_id
 * (self-dial on own IPv6) must be treated as a ghost.
 */
class SelfAsPeerGhostTest {

    @Test
    fun ownPkAsPeerId_isGhostEvenWhenFailCountLow() {
        val ownPk = "f832730bab05dc5789a49f565e216dd611f208d59a895e75725f7e569a591e52"
        assertTrue(
            GhostIdentityGate.isGhost(
                peerId = ownPk,
                publicKey = ownPk,
                multiaddr = "/ip6/2600:381:9b76:7101:eca7:f1c2:2871:313c/tcp/80",
                successCount = 0u,
                failureCount = 1u,
                ownLanAddrs = setOf("2600:381:9b76:7101:eca7:f1c2:2871:313c"),
            )
        )
    }

    @Test
    fun ownPkAsPeerId_nullPk_successZero_isGhost() {
        val ownPk = "f832730bab05dc5789a49f565e216dd611f208d59a895e75725f7e569a591e52"
        assertTrue(
            GhostIdentityGate.isGhost(
                peerId = ownPk,
                publicKey = null,
                multiaddr = "/ip6/2600:381:9b76:7101:eca7:f1c2:2871:313c/tcp/443",
                successCount = 0u,
                failureCount = 1u,
                ownLanAddrs = setOf("2600:381:9b76:7101:eca7:f1c2:2871:313c"),
            )
        )
    }
}
