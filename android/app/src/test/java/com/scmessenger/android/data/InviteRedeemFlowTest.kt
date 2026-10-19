package com.scmessenger.android.data

import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** #469 T3: invite redeem flow (SCI1 text -> core redeem -> dial + discovery reset). */
class InviteRedeemFlowTest {

    private val report = InviteReport(
        inviterId = "abcdef0123456789",
        inviterPeerId = "12D3KooWExample",
        addressesOffered = 3,
        addressesImported = 3,
        dialAddrs = listOf(
            "/ip4/192.0.2.10/tcp/9001/p2p/12D3KooWExample",
            "/ip4/192.0.2.11/tcp/9001"
        )
    )

    private class Harness(
        val redeemImpl: (String) -> InviteReport,
        val dialImpl: suspend (String) -> Unit = {},
        val classifyImpl: (Throwable) -> InviteFailure = { InviteFailure.UNKNOWN }
    ) {
        val redeemedTokens = mutableListOf<String>()
        val dialed = mutableListOf<String>()
        var redeemedEvents = 0

        val flow = InviteRedeemFlow(
            redeem = { token ->
                redeemedTokens.add(token)
                redeemImpl(token)
            },
            dial = { addr ->
                dialed.add(addr)
                dialImpl(addr)
            },
            classify = classifyImpl,
            onRedeemed = { redeemedEvents++ }
        )
    }

    @Test
    fun `valid invite imports dials every address and reports InviteRedeemed once`() = runBlocking {
        val h = Harness(redeemImpl = { report })

        val result = h.flow.run("SCI1:payload")

        assertTrue(result is InviteRedeemResult.Success)
        result as InviteRedeemResult.Success
        assertEquals(2, result.dialedCount)
        assertEquals(report.dialAddrs, h.dialed)
        assertEquals(1, h.redeemedEvents)
        assertEquals(listOf("SCI1:payload"), h.redeemedTokens)
    }

    @Test
    fun `invite embedded in a shared message is extracted`() = runBlocking {
        val h = Harness(redeemImpl = { report })

        h.flow.run("Join me on SCMessenger: SCI1:abc123XYZ+/= then say hi.")

        assertEquals(listOf("SCI1:abc123XYZ+/="), h.redeemedTokens)
    }

    @Test
    fun `blank input is rejected as EMPTY without calling core`() = runBlocking {
        val h = Harness(redeemImpl = { report })

        assertEquals(InviteRedeemResult.Failure(InviteFailure.EMPTY), h.flow.run("   "))
        assertEquals(InviteRedeemResult.Failure(InviteFailure.EMPTY), h.flow.run(null))
        assertTrue(h.redeemedTokens.isEmpty())
        assertEquals(0, h.redeemedEvents)
    }

    @Test
    fun `legacy JSON join bundle is not an invite`() = runBlocking {
        val h = Harness(redeemImpl = { report })

        val result = h.flow.run("{\"bootstrap_peers\":[\"/ip4/192.0.2.1/tcp/9001\"],\"topics\":[]}")

        assertEquals(InviteRedeemResult.Failure(InviteFailure.NOT_AN_INVITE), result)
        assertTrue(h.redeemedTokens.isEmpty())
    }

    @Test
    fun `core rejection is classified and nothing is dialed or reported`() = runBlocking {
        val h = Harness(
            redeemImpl = { throw IllegalArgumentException("bad signature") },
            classifyImpl = { InviteFailure.BAD_SIGNATURE }
        )

        val result = h.flow.run("SCI1:tampered")

        assertEquals(InviteRedeemResult.Failure(InviteFailure.BAD_SIGNATURE), result)
        assertTrue(h.dialed.isEmpty())
        assertEquals(0, h.redeemedEvents)
    }

    @Test
    fun `a failed seed dial is not a failure - the scheduler keeps retrying`() = runBlocking {
        val h = Harness(
            redeemImpl = { report },
            dialImpl = { throw IllegalStateException("unreachable") }
        )

        val result = h.flow.run("SCI1:payload")

        assertTrue(result is InviteRedeemResult.Success)
        assertEquals(0, (result as InviteRedeemResult.Success).dialedCount)
        assertEquals(1, h.redeemedEvents)
    }

    @Test
    fun `the discovery event fires before the first dial so a slow dial cannot delay it`() = runBlocking {
        val order = mutableListOf<String>()
        val flow = InviteRedeemFlow(
            redeem = { report },
            dial = { order.add("dial") },
            classify = { InviteFailure.UNKNOWN },
            onRedeemed = { order.add("event") }
        )

        flow.run("SCI1:payload")

        assertEquals(listOf("event", "dial", "dial"), order)
    }

    @Test
    fun `extract returns null when there is no token or only the bare prefix`() {
        assertNull(InviteText.extract(null))
        assertNull(InviteText.extract("no invite here"))
        assertNull(InviteText.extract("SCI1:"))
    }

    @Test
    fun `extract strips trailing sentence punctuation`() {
        assertEquals("SCI1:AbC", InviteText.extract("code is SCI1:AbC."))
        assertEquals("SCI1:AbC", InviteText.extract("(SCI1:AbC)"))
    }
}
