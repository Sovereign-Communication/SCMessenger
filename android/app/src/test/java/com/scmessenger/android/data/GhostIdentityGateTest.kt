package com.scmessenger.android.data

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * GHOST-IDENTITY-001 regression pins for the 2026-09-11 PK:577fd171 RCA.
 *
 * The ghost was: peer_id = old public key (64-hex), public_key = null,
 * multiaddr = this device's own LAN IP, success_count = 0, failure_count = 3.
 */
class GhostIdentityGateTest {

    private val ghostPk = "577fd1715f9f95fae10da5ea01aa20ac6789dfd898c3351ca3b6f62b249c4fb8"
    private val ownIp = "192.168.0.134"

    @Test
    fun rca_ghost_577fd171_is_ghost() {
        assertTrue(
            GhostIdentityGate.isGhost(
                peerId = ghostPk,
                publicKey = null,
                multiaddr = "/ip4/$ownIp/tcp/9001",
                successCount = 0u,
                failureCount = 3u,
                ownLanAddrs = setOf(ownIp),
            )
        )
    }

    @Test
    fun never_succeeded_and_dead_failures_is_ghost() {
        assertTrue(
            GhostIdentityGate.isGhost(
                peerId = "12D3KooWSomeRealPeerIdThatIsLongEnoughxxxx",
                publicKey = null,
                multiaddr = "/ip4/10.0.0.5/tcp/9001",
                successCount = 0u,
                failureCount = 3u,
            )
        )
    }

    @Test
    fun live_proven_peer_is_not_ghost() {
        assertFalse(
            GhostIdentityGate.isGhost(
                peerId = "12D3KooWGvCWJNoWnReNCT1q2LWb2gTbeBTa5sjxF49wZX3u2y31",
                publicKey = "69805e175cdc59b244f36001303e0b2e735f729557d2069a02688c0e69764a7c",
                multiaddr = "/ip4/18.234.62.247/tcp/9001",
                successCount = 15u,
                failureCount = 2u,
            )
        )
    }

    @Test
    fun self_dial_unproven_is_ghost() {
        assertTrue(
            GhostIdentityGate.isGhost(
                peerId = "12D3KooWGhostSelfDialxxxxxxxxxxxxxxxxxxxxx",
                publicKey = null,
                multiaddr = "/ip4/$ownIp/tcp/443",
                successCount = 0u,
                failureCount = 1u,
                ownLanAddrs = setOf(ownIp),
            )
        )
    }

    @Test
    fun discovered_live_always_renders() {
        assertTrue(
            GhostIdentityGate.shouldRenderAsNode(
                peerId = ghostPk,
                publicKey = null,
                multiaddr = "/ip4/$ownIp/tcp/9001",
                successCount = 0u,
                failureCount = 3u,
                isCurrentlyDiscovered = true,
                ownLanAddrs = setOf(ownIp),
            )
        )
    }

    @Test
    fun undiscovered_ghost_does_not_render() {
        assertFalse(
            GhostIdentityGate.shouldRenderAsNode(
                peerId = ghostPk,
                publicKey = null,
                multiaddr = "/ip4/$ownIp/tcp/9001",
                successCount = 0u,
                failureCount = 3u,
                isCurrentlyDiscovered = false,
                ownLanAddrs = setOf(ownIp),
            )
        )
    }
}
