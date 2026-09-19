package com.scmessenger.android.data

import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Regression tests for the AWS-unreachable bootstrap defect (2026-09-10).
 *
 * primeRelayBootstrapConnections()/racingBootstrapWithFallback() swept only
 * the proven ledger tier (success_count > 0), so a ledger-exchanged cloud
 * relay (the AWS bootstrap node shared via the 2-peer list) could never be
 * dialed and therefore never become proven. Live evidence: the phone logged
 * "Bootstrap: attempting 1 proven ledger relay candidate(s)" on every cycle
 * while the cloud node sat in its knowledge base unproven, and AWS never
 * appeared online to the Pixel on WiFi.
 *
 * The fix sweeps the seed tier (invite/QR/ledger-exchange addresses) after
 * the proven tier — the documented D3c design ("sweep the proven set first,
 * then this one"). A first successful dial promotes the seed via
 * record_connection in core, after which it ranks from the proven tier.
 * mergeBootstrapCandidates is the pure merge policy under test: order,
 * dedup, blank-tolerance, and the poison-fanout cap.
 */
class BootstrapCandidateMergeTest {

    @Test
    fun emptyLedgerYieldsNoCandidates() {
        val out = MeshRepository.mergeBootstrapCandidates(emptyList(), emptyList())
        assertEquals(emptyList<String>(), out)
    }

    @Test
    fun provenCandidatesPassThroughInOrderWithNoSeeds() {
        val proven = listOf("/ip4/192.168.0.222/tcp/9001", "/ip4/18.234.62.247/tcp/9001")
        val out = MeshRepository.mergeBootstrapCandidates(proven, emptyList())
        assertEquals(proven, out)
    }

    @Test
    fun seedsAppendedAfterProvenAndDedupedAgainstIt() {
        val proven = listOf("/ip4/18.234.62.247/tcp/9001")
        val seeds = listOf("/ip4/18.234.62.247/tcp/9001", "/ip4/192.168.0.222/tcp/9001")
        val out = MeshRepository.mergeBootstrapCandidates(proven, seeds)
        assertEquals(
            listOf("/ip4/18.234.62.247/tcp/9001", "/ip4/192.168.0.222/tcp/9001"),
            out
        )
    }

    @Test
    fun seedSweepIsCappedToPreventPoisonedLedgerFanout() {
        val seeds = (1..10).map { "/ip4/10.0.0.$it/tcp/9001" }
        val out = MeshRepository.mergeBootstrapCandidates(emptyList(), seeds)
        assertEquals(MeshRepository.MAX_BOOTSTRAP_SEEDS, out.size)
    }

    @Test
    fun blankEntriesDroppedFromBothTiers() {
        val out = MeshRepository.mergeBootstrapCandidates(
            listOf("  ", "/ip4/18.234.62.247/tcp/9001", ""),
            listOf("", "/ip4/10.0.0.9/tcp/9001", "   ")
        )
        assertEquals(
            listOf("/ip4/18.234.62.247/tcp/9001", "/ip4/10.0.0.9/tcp/9001"),
            out
        )
    }

    @Test
    fun duplicateSeedsCollapse() {
        val seeds = listOf("/ip4/10.0.0.9/tcp/9001", "/ip4/10.0.0.9/tcp/9001")
        val out = MeshRepository.mergeBootstrapCandidates(emptyList(), seeds)
        assertEquals(listOf("/ip4/10.0.0.9/tcp/9001"), out)
    }

    @Test
    fun trimNormalizesWhitespacePaddedEntries() {
        val out = MeshRepository.mergeBootstrapCandidates(
            listOf("  /ip4/18.234.62.247/tcp/9001  "),
            listOf("/ip4/18.234.62.247/tcp/9001")
        )
        assertEquals(listOf("/ip4/18.234.62.247/tcp/9001"), out)
    }
}
