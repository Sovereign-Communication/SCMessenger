package com.scmessenger.android.data

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test
import java.io.File

/**
 * #469 T3: malformed invites are rejected before they touch the ledger, and
 * a good invite reaches the ledger-facing report and the dial path intact.
 */
class InviteRedeemRejectionTest {

    private val report = InviteReport(
        inviterId = "0123456789abcdef",
        inviterPeerId = "12D3KooWInviter",
        addressesOffered = 4,
        addressesImported = 3,
        dialAddrs = listOf("/ip4/192.0.2.1/tcp/9001", "/ip4/192.0.2.2/tcp/9001", "/ip4/192.0.2.3/tcp/9001")
    )

    private class Probe(
        val redeemImpl: (String) -> InviteReport,
        val dialImpl: suspend (String) -> Unit = {},
        val classifyImpl: (Throwable) -> InviteFailure = { InviteFailure.INVALID },
        val onRedeemedImpl: () -> Unit = {}
    ) {
        val tokens = mutableListOf<String>()
        val dialed = mutableListOf<String>()
        var events = 0
        val flow = InviteRedeemFlow(
            redeem = { t ->
                tokens.add(t)
                redeemImpl(t)
            },
            dial = { a ->
                dialImpl(a)
                dialed.add(a)
            },
            classify = classifyImpl,
            onRedeemed = {
                events++
                onRedeemedImpl()
            }
        )
    }

    @Test
    fun `a bare SCI1 prefix is rejected as not an invite and never reaches core`() = runBlocking {
        val p = Probe(redeemImpl = { report })

        assertEquals(InviteRedeemResult.Failure(InviteFailure.NOT_AN_INVITE), p.flow.run("SCI1:"))
        assertTrue(p.tokens.isEmpty())
        assertEquals(0, p.events)
    }

    @Test
    fun `a truncated invite that core rejects dials nothing and sends no discovery event`() = runBlocking {
        val p = Probe(redeemImpl = { throw IllegalArgumentException("truncated") })

        val result = p.flow.run("SCI1:AAAA")

        assertEquals(InviteRedeemResult.Failure(InviteFailure.INVALID), result)
        assertEquals(listOf("SCI1:AAAA"), p.tokens)
        assertTrue(p.dialed.isEmpty())
        assertEquals(0, p.events)
    }

    @Test
    fun `every classified core rejection is returned as the failure reason`() = runBlocking {
        for (reason in listOf(InviteFailure.INVALID, InviteFailure.BAD_SIGNATURE, InviteFailure.NO_IDENTITY, InviteFailure.UNKNOWN)) {
            val p = Probe(
                redeemImpl = { throw IllegalStateException("rejected") },
                classifyImpl = { reason }
            )
            assertEquals(InviteRedeemResult.Failure(reason), p.flow.run("SCI1:x"))
            assertTrue(p.dialed.isEmpty())
        }
    }

    @Test
    fun `a rejected invite does not poison the next valid redeem`() = runBlocking {
        var calls = 0
        val p = Probe(redeemImpl = {
            calls++
            if (calls == 1) throw IllegalArgumentException("bad") else report
        })

        assertTrue(p.flow.run("SCI1:bad") is InviteRedeemResult.Failure)
        val second = p.flow.run("SCI1:good")

        assertTrue(second is InviteRedeemResult.Success)
        assertEquals(1, p.events)
    }

    @Test
    fun `the imported ledger count and inviter are carried to the success result unchanged`() = runBlocking {
        val p = Probe(redeemImpl = { report })

        val result = p.flow.run("SCI1:payload") as InviteRedeemResult.Success

        assertEquals(3, result.report.addressesImported)
        assertEquals(4, result.report.addressesOffered)
        assertEquals("12D3KooWInviter", result.report.inviterPeerId)
    }

    @Test
    fun `seeds are dialed in the order core returned them`() = runBlocking {
        val p = Probe(redeemImpl = { report })

        p.flow.run("SCI1:payload")

        assertEquals(report.dialAddrs, p.dialed)
    }

    @Test
    fun `one failing dial does not stop the remaining seeds from being dialed`() = runBlocking {
        val p = Probe(
            redeemImpl = { report },
            dialImpl = { addr -> if (addr.contains("192.0.2.1/")) throw IllegalStateException("down") }
        )

        val result = p.flow.run("SCI1:payload") as InviteRedeemResult.Success

        assertEquals(2, result.dialedCount)
        assertEquals(report.dialAddrs.drop(1), p.dialed)
    }

    @Test
    fun `a failing discovery event dispatch still dials every seed`() = runBlocking {
        val p = Probe(
            redeemImpl = { report },
            onRedeemedImpl = { throw IllegalStateException("driver closed") }
        )

        val result = p.flow.run("SCI1:payload") as InviteRedeemResult.Success

        assertEquals(3, result.dialedCount)
    }

    @Test
    fun `cancellation during a dial propagates instead of being swallowed`() = runBlocking {
        val p = Probe(
            redeemImpl = { report },
            dialImpl = { throw CancellationException("scope cancelled") }
        )

        try {
            p.flow.run("SCI1:payload")
            fail("expected cancellation")
        } catch (e: CancellationException) {
            assertTrue(p.dialed.isEmpty())
        }
    }

    @Test
    fun `an invite wrapped in punctuation or lines is extracted without the wrapping`() {
        assertEquals("SCI1:abc", InviteText.extract("(SCI1:abc)"))
        assertEquals("SCI1:abc", InviteText.extract("\"SCI1:abc\""))
        assertEquals("SCI1:abc", InviteText.extract("line one\nSCI1:abc\nline three"))
    }

    @Test
    fun `the app strings carry no no-network-peers message`() {
        val file = listOf(
            File("src/main/res/values/strings.xml"),
            File("app/src/main/res/values/strings.xml")
        ).firstOrNull { it.isFile }
        if (file == null) {
            fail("strings.xml not found from " + File(".").absolutePath)
            return
        }

        val lower = file.readText().lowercase()

        assertFalse(lower.contains("no network peers"))
        assertFalse(lower.contains("no peers found"))
        assertFalse(lower.contains("no_network_peers"))
    }
}
