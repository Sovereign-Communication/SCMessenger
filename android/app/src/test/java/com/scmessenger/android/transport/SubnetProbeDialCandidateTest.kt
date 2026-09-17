package com.scmessenger.android.transport

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Pins the SubnetProbe dial-candidate contract.
 *
 * A probe hit is a TCP connect that succeeded, which proves a peer is present
 * but says nothing about whether THIS client can complete a libp2p dial to that
 * port. Live evidence for the WebSocket case (2026-09-16):
 *
 *  - the phone's dial of `/ip4/192.168.0.121/tcp/9002/ws` failed client-side in
 *    about 80ms with IronCoreException$NetworkException, and the Windows node
 *    logged no connection attempt at that instant;
 *  - the node logs 0 `direction=inbound transport=ws` across the whole day while
 *    every accepted inbound (35 of them) is `transport=tcp`;
 *  - the same phone reaches that node over TCP.
 *
 * So a hit on the WebSocket port must not become a dial candidate.
 */
class SubnetProbeDialCandidateTest {

    @Test
    fun `websocket port yields no dial candidate`() {
        assertNull(SubnetProbe.dialCandidateFor("192.168.0.121", 9002))
    }

    @Test
    fun `raw tcp port yields the direct multiaddr`() {
        assertEquals(
            "/ip4/192.168.0.121/tcp/9001",
            SubnetProbe.dialCandidateFor("192.168.0.121", 9001)
        )
    }

    @Test
    fun `no probed port ever yields a websocket multiaddr`() {
        val probedPorts = listOf(9001, 9002, 443, 8080, 9090)
        val candidates = probedPorts.mapNotNull { SubnetProbe.dialCandidateFor("10.0.0.7", it) }

        assertTrue("no /ws candidate may be produced", candidates.none { it.contains("/ws") })
        assertEquals(
            listOf(
                "/ip4/10.0.0.7/tcp/9001",
                "/ip4/10.0.0.7/tcp/443",
                "/ip4/10.0.0.7/tcp/8080",
                "/ip4/10.0.0.7/tcp/9090"
            ),
            candidates
        )
    }

    @Test
    fun `raw tcp ladder mirrors the ports the node binds`() {
        // Must stay in step with core/src/transport/multiport.rs COMMON_PORTS.
        // The Windows node's own Listening on lines were 443, 80, 8080, 9090
        // plus one random port, which is this ladder.
        assertEquals(listOf(9001, 443, 80, 8080, 9090), SubnetProbe.RAW_TCP_PORTS)
        assertEquals(9001, SubnetProbe.RAW_TCP_PORTS.first())
        assertTrue(
            "the WebSocket port must never be probed as raw TCP",
            !SubnetProbe.RAW_TCP_PORTS.contains(SubnetProbe.WEBSOCKET_PORT)
        )
    }

    @Test
    fun `default probe targets keep the websocket port out of dials`() {
        val defaultTargets = listOf(9001, 9002)

        assertEquals(
            listOf("/ip4/192.168.1.50/tcp/9001"),
            defaultTargets.mapNotNull { SubnetProbe.dialCandidateFor("192.168.1.50", it) }
        )
    }
}
