package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import android.content.Context
import com.scmessenger.android.R
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.service.MeshEventBus
import com.scmessenger.android.service.StatusEvent
import com.scmessenger.android.utils.toEpochSeconds
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import timber.log.Timber
import java.math.BigInteger
import javax.inject.Inject

// UNIFICATION P1-1: Ed25519 field constants for curve-point validation.
// Roughly 50% of blake3 identity_id hashes are valid 64-hex but not valid curve points;
// Rust's is_valid_public_key checks VerifyingKey::from_bytes. We mirror that here.
private val ED25519_P: BigInteger = BigInteger("57896044618658097711785492504343953926634992332820282019728792003956564819949")
private val ED25519_D: BigInteger by lazy {
    val p = ED25519_P
    val inv121666 = BigInteger.valueOf(121666).modInverse(p)
    BigInteger.valueOf(121665).negate().mod(p).multiply(inv121666).mod(p)
}
private val ED25519_SQRT_M1: BigInteger by lazy {
    BigInteger.valueOf(2).modPow(ED25519_P.subtract(BigInteger.ONE).divide(BigInteger.valueOf(4)), ED25519_P)
}
private val ED25519_P_PLUS3_OVER8: BigInteger by lazy {
    ED25519_P.add(BigInteger.valueOf(3)).divide(BigInteger.valueOf(8))
}

private fun isValidEd25519Point(hex: String): Boolean {
    try {
        if (hex.length != 64) return false
        val bytes = ByteArray(32)
        for (i in 0 until 32) {
            val hi = Character.digit(hex[i * 2], 16)
            val lo = Character.digit(hex[i * 2 + 1], 16)
            if (hi == -1 || lo == -1) return false
            bytes[i] = ((hi shl 4) or lo).toByte()
        }
        val yBytes = bytes.clone()
        val signBit = (yBytes[31].toInt() and 0x80) != 0
        yBytes[31] = (yBytes[31].toInt() and 0x7F).toByte()
        // little-endian -> BigInteger (big-endian)
        val be = yBytes.reversedArray()
        val y = BigInteger(1, be)
        if (y >= ED25519_P) return false
        val yy = y.multiply(y).mod(ED25519_P)
        val u = yy.subtract(BigInteger.ONE).mod(ED25519_P)
        val v = ED25519_D.multiply(yy).add(BigInteger.ONE).mod(ED25519_P)
        if (v == BigInteger.ZERO) return false
        val vInv = try { v.modInverse(ED25519_P) } catch (_: ArithmeticException) { return false }
        val x2 = u.multiply(vInv).mod(ED25519_P)
        if (x2 == BigInteger.ZERO) return !signBit
        var x = x2.modPow(ED25519_P_PLUS3_OVER8, ED25519_P)
        var check = x.multiply(x).mod(ED25519_P)
        if (check != x2) {
            x = x.multiply(ED25519_SQRT_M1).mod(ED25519_P)
            check = x.multiply(x).mod(ED25519_P)
            if (check != x2) return false
        }
        return true
    } catch (_: Exception) {
        return false
    }
}

/**
 * ViewModel for the dashboard screen.
 *
 * Provides service statistics, peer list, mesh topology data,
 * and real-time network health metrics.
 */
@HiltViewModel
class DashboardViewModel @Inject constructor(
    private val meshRepository: MeshRepository,
    @ApplicationContext private val context: Context
) : ViewModel() {
    // Service stats
    private val _stats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    val stats: StateFlow<uniffi.api.ServiceStats?> = _stats.asStateFlow()

    // Active peers
    private val _peers = MutableStateFlow<List<PeerInfo>>(emptyList())
    val peers: StateFlow<List<PeerInfo>> = _peers.asStateFlow()

    // Network topology data (for graph visualization)
    private val _topology = MutableStateFlow<NetworkTopology>(NetworkTopology())
    val topology: StateFlow<NetworkTopology> = _topology.asStateFlow()

    // Live network stats from repository observable
    private val _networkStats = MutableStateFlow<uniffi.api.ServiceStats?>(null)
    val networkStats: StateFlow<uniffi.api.ServiceStats?> = _networkStats.asStateFlow()

    // Live peer list from repository observable
    private val _observablePeers = MutableStateFlow<List<String>>(emptyList())
    val observablePeers: StateFlow<List<String>> = _observablePeers.asStateFlow()

    // Peer counts from discovery tracking
    val fullPeersCount = meshRepository.discoveredPeers.map { discovered ->
        deduplicateDiscoveredPeers(discovered).values.count { peer -> peer.isFull }
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

    // Headless = any discovered node without a confirmed identity (relay and non-relay share this bucket).
    val headlessPeersCount = meshRepository.discoveredPeers.map { discovered ->
        deduplicateDiscoveredPeers(discovered).values.count { peer ->
            !peer.isFull
        }
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

    val totalPeersCount = meshRepository.discoveredPeers.map { discovered ->
        deduplicateDiscoveredPeers(discovered).size
    }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

    // UNIFICATION_V2: _peers.size is authoritative total; totalPeersCount is discovered-only legacy
    // unifiedTotalPeersCount reflects the single-list unified view (sorted peers).
    val unifiedTotalPeersCount: StateFlow<Int> = _peers.map { it.size }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

    // UNIFICATION FIX: nearby must be discovery-based (BLE/mDNS direct), not ledger history. Previous _peers-based count inflated to 9 vs 2 actual BLE/mDNS.
    // _peers includes dialable+seed+deadRecent ledger entries (7-day retention) that were counted as nearby if isOnline+direct,
    // even when not currently in BLE/mDNS range. deadRecent is explicitly NOT nearby — it is ledger history for "last seen Xm ago"
    // offline display, never counted in nearby. Correct: nearby = deduplicated discoveredPeers that are isRecent (<5min) + direct transport
    // (BLE, TCP/LAN, WiFi Aware/Direct) only. Ledger history (dialable/seed/deadRecent) excluded even if isOnline+direct.
    // Uses combine to log both nearby (discovery-based) vs total (unified _peers) for diagnosis. Verbose logs prove 9 vs 2 fix.
    // UNIFICATION: nearby is unified list isNearby (online + direct) — not discovered-only, not ledger. 2 nearby vs 9 total (ledger history excluded).
    val nearbyPeersCount: StateFlow<Int> = _peers.map { peersList ->
        val nearbyCount = peersList.count { isNearby(it) }
        if (peersList.isNotEmpty()) {
            Timber.d("UNIFICATION nearbyPeersCount: ${peersList.size} total, nearby=$nearbyCount (direct) — peers: ${peersList.joinToString { "${it.peerId.take(8)}:${it.transport}:${it.isOnline}:${it.lastSeen}:${isNearby(it)}" }}")
        }
        nearbyCount
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    private val loadPeersMutex = Mutex()

    init {
        observeNetworkEvents()
        observeLiveNetworkStats()
        observeLivePeers()
        refreshData()
    }

    /**
     * Refresh all dashboard data.
     */
    fun refreshData() {
        viewModelScope.launch(Dispatchers.IO) {
            try {
                _isLoading.value = true
                _error.value = null

                // Get service stats
                _stats.value = meshRepository.serviceStats.value

                // Get peer information
                loadPeers()

                // Build topology
                buildTopology()

                Timber.d("Dashboard data refreshed")
            } catch (e: Exception) {
                _error.value = "Failed to refresh data: ${e.message}"
                Timber.e(e, "Failed to refresh dashboard data")
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * Load active peers from discovery map and ledger.
     * FIX: persist offline nodes (seed + recently dead) and use authoritative nickname.
     * Holistic crash fix: mutex + distinct check + isActive to prevent Compose crash on destroy.
     */
    private fun loadPeers() {
        if (!viewModelScope.isActive) {
            Timber.d("loadPeers skipped — ViewModelScope not active (destroyed)")
            return
        }
        // Prevent concurrent loadPeers from racing and producing duplicate list sizes
        if (!loadPeersMutex.tryLock()) {
            Timber.d("loadPeers skipped — already in progress")
            return
        }
        try {
            if (!viewModelScope.isActive) return
            val discoveredSnapshot = meshRepository.discoveredPeers.value
            val discovered = deduplicateDiscoveredPeers(discoveredSnapshot)
            // FIX: include offline nodes so peer list persists "last seen Xm ago" even when dialable==0.
            // 75439a40 intent: nodes persist via contacts + ledger seeding, not just proven dialable.
            val dialable = meshRepository.getDialableAddresses()
            val seed = meshRepository.getSeedAddresses(16u)
            val deadRecent = meshRepository.getRecentlyDeadAddresses(7)
            val ledgerEntries = (dialable + seed + deadRecent).distinctBy { it.multiaddr }
            val relayHops = meshRepository.getRelayHopPeerIds()
            val routeAliasToCanonical = discoveredSnapshot
                .mapNotNull { (routeKey, info) ->
                    val alias = routeKey.trim()
                    val canonical = info.peerId.trim()
                    if (alias.isNotEmpty() && canonical.isNotEmpty() && alias != canonical) {
                        alias to canonical
                    } else {
                        null
                    }
                }
                .toMap()
            val canonicalByPublicKey = discovered.values
                .mapNotNull { info ->
                    normalizePublicKey(info.publicKey)?.let { publicKey ->
                        publicKey to info.peerId
                    }
                }
                .toMap()

            val peerMap = discovered.mapValues { (_, info) ->
                val primaryTransport = when (info.transport) {
                    com.scmessenger.android.service.TransportType.BLE -> "BLE"
                    com.scmessenger.android.service.TransportType.WIFI_AWARE -> "WiFi Aware"
                    com.scmessenger.android.service.TransportType.WIFI_DIRECT -> "WiFi Direct"
                    com.scmessenger.android.service.TransportType.INTERNET -> "Internet"
                    com.scmessenger.android.service.TransportType.TCP_MDNS -> "TCP/LAN"
                }
                // UNIFICATION_V2: All nodes are relays — isRelay no longer distinguishes.
                PeerInfo(
                    peerId = info.peerId,
                    nickname = info.nickname,
                    localNickname = info.localNickname,
                    multiaddr = "", // Might be empty for BLE/headless
                    lastSeen = info.lastSeen,
                    transport = primaryTransport,
                    // NODE-TRANSPORT-VIS-001: every transport known for this node
                    transports = (info.transports + primaryTransport).toList().sorted(),
                    isOnline = isRecent(info.lastSeen),
                    isFull = info.isFull,
                    isRelay = false
                )
            }.toMutableMap().also { map ->
                // FIX: enrich discovered peers with contact's authoritative localNickname (user-defined, primary)
                // so Dashboard does not show synthetic peer-... when contact has ChristyLove but discovered is stale.
                // Contacts are the source of truth for displayNames(); ledger/discovered may be stale.
                try {
                    val contacts = meshRepository.listContacts()
                    for ((k, peer) in map.toList()) {
                        val normalizedPeerKey = normalizePublicKey(peer.peerId) ?: peer.peerId.lowercase()
                        val contact = contacts.firstOrNull {
                            it.peerId.lowercase() == k.lowercase() ||
                                it.peerId.lowercase() == peer.peerId.lowercase() ||
                                normalizePublicKey(it.publicKey)?.lowercase() == normalizedPeerKey ||
                                normalizePublicKey(it.publicKey)?.lowercase() == normalizePublicKey(peer.peerId)?.lowercase()
                        }
                        if (contact != null) {
                            val contactLocal = normalizeNickname(contact.localNickname)?.takeUnless { isSyntheticFallbackNickname(it) }
                            val contactNick = normalizeNickname(contact.nickname)?.takeUnless { isSyntheticFallbackNickname(it) }
                            var enriched = peer
                            var changed = false
                            if (contactLocal != null && (peer.localNickname.isNullOrBlank() || isSyntheticFallbackNickname(peer.localNickname))) {
                                Timber.d("UNIFICATION loadPeers discovered enrich $k: local ${peer.localNickname?.take(16)} -> $contactLocal from contact ${contact.peerId.take(8)}")
                                enriched = enriched.copy(localNickname = contactLocal)
                                changed = true
                            }
                            if (contactNick != null && (peer.nickname.isNullOrBlank() || isSyntheticFallbackNickname(peer.nickname))) {
                                val newNick = selectAuthoritativeNickname(contactNick, peer.nickname) ?: contactNick
                                if (newNick != peer.nickname) {
                                    Timber.d("UNIFICATION loadPeers discovered enrich $k: nick ${peer.nickname?.take(16)} -> $newNick from contact")
                                    enriched = enriched.copy(nickname = newNick)
                                    changed = true
                                }
                            }
                            if (changed) map[k] = enriched
                        }
                    }
                } catch (e: Exception) {
                    Timber.w(e, "UNIFICATION loadPeers discovered enrich failed")
                }
            }

            // UNIFICATION: Enrich/Add with ledger entries — canonicalize: 12D3 (libp2p) and 30d0fa (hex) for same identity must merge to ONE node.
            // Robust even when ironCore is null at cold start: canonicalHexForAnyId uses PeerKeyUtils protobuf fallback.
            // Verbose logs persisted via FileLoggingTree for 2x claude-windows-driver diagnosis.
            ledgerEntries.forEach { entry ->
                val rawPeerId = entry.peerId ?: return@forEach
                // UNIFICATION: resolve via route alias AND via publicKey->canonical lookup (both hints)
                val resolvedPeerId = routeAliasToCanonical[rawPeerId]
                    ?: normalizePublicKey(entry.publicKey)?.let { canonicalByPublicKey[it] }
                    ?: rawPeerId
                // UNIFICATION: Canonicalize BOTH rawPeerId and publicKeyHint via same helper — strongest is publicKeyHint.
                // This ensures ledger entries with peer_id 12D3 + public_key 30d0fa correctly collapse when ironCore null.
                val peerId = canonicalHexForAnyId(resolvedPeerId, entry.publicKey)
                    ?: canonicalHexForAnyId(rawPeerId, entry.publicKey)
                    ?: try { meshRepository.canonicalContactIdPublic(resolvedPeerId).takeIf { it.isNotEmpty() } ?: resolvedPeerId } catch (_: Exception) { resolvedPeerId }
                Timber.d("UNIFICATION loadPeers ledger: raw $rawPeerId (pubKey=${entry.publicKey?.take(8)}) resolved $resolvedPeerId -> canonical $peerId")
                // UNIFICATION: peerMap is keyed by canonical hex (lowercase). Lookup via canonical variants only.
                val lookupKey = peerId.trim().lowercase()
                val canonicalRawKey = canonicalHexForAnyId(rawPeerId, entry.publicKey)?.lowercase()
                val canonicalResolvedKey = canonicalHexForAnyId(resolvedPeerId, entry.publicKey)?.lowercase()
                val existing = peerMap[lookupKey]
                    ?: canonicalRawKey?.let { peerMap[it] }
                    ?: canonicalResolvedKey?.let { peerMap[it] }
                    ?: peerMap[resolvedPeerId]
                    ?: peerMap[rawPeerId]
                if (existing != null) {
                    Timber.d("UNIFICATION loadPeers ledger MERGE $rawPeerId -> $lookupKey (existing ${existing.peerId.take(8)}... lastSeen ${existing.lastSeen})")
                    val entryLastSeen = entry.lastSeen
                    val existingLastSeen = existing.lastSeen
                    val authoritativeNick = selectAuthoritativeNickname(existing.nickname, entry.nickname)
                        ?: selectAuthoritativeNickname(entry.nickname, existing.nickname)
                        ?: existing.nickname
                    // FIX: enrich localNickname from contact if discovered's local is null/synthetic but contact has ChristyLove
                    var authoritativeLocal = existing.localNickname?.takeUnless { isSyntheticFallbackNickname(it) }
                    if (authoritativeLocal.isNullOrBlank()) {
                        try {
                            val contactByKey = normalizePublicKey(entry.publicKey)?.lowercase() ?: lookupKey
                            val contact = meshRepository.listContacts().firstOrNull {
                                it.peerId.lowercase() == lookupKey ||
                                    normalizePublicKey(it.publicKey)?.lowercase() == contactByKey ||
                                    normalizePublicKey(it.publicKey)?.lowercase() == existing.peerId.lowercase()
                            }
                            val contactLocal = contact?.localNickname?.trim()?.takeIf { it.isNotEmpty() }?.takeUnless { isSyntheticFallbackNickname(it) }
                            if (contactLocal != null) {
                                Timber.d("UNIFICATION loadPeers ledger MERGE enrich local from contact $lookupKey: ${contactLocal.take(16)} over existingLocal=${existing.localNickname?.take(16)}")
                                authoritativeLocal = contactLocal
                            }
                        } catch (e: Exception) {
                            Timber.w(e, "UNIFICATION loadPeers merge enrich failed for $lookupKey")
                        }
                    }
                    // UNIFICATION: Update via canonical lookupKey to avoid 12D3/30d0fa duplicate node
                    peerMap[lookupKey] = existing.copy(
                        nickname = authoritativeNick,
                        localNickname = authoritativeLocal ?: existing.localNickname,
                        multiaddr = if (entry.multiaddr.contains("/p2p-circuit/") || existing.multiaddr.isEmpty()) entry.multiaddr else existing.multiaddr,
                        lastSeen = when {
                            entryLastSeen == null -> existingLastSeen
                            existingLastSeen == null || entryLastSeen > existingLastSeen -> entryLastSeen
                            else -> existingLastSeen
                        },
                        isOnline = isRecent(entry.lastSeen) || existing.isOnline,
                        isRelay = false,
                        transports = (existing.transports +
                            MeshRepository.parseTransportsFromMultiaddrs(listOf(entry.multiaddr)))
                            .distinct()
                    )
                    // UNIFICATION: Remove any stale alias keys (12D3 vs 30d0fa) to prevent duplicate nodes
                    if (lookupKey != peerId && peerMap.containsKey(peerId)) {
                        Timber.d("UNIFICATION loadPeers dedup remove stale peerId key $peerId -> $lookupKey")
                        peerMap.remove(peerId)
                    }
                    if (lookupKey != resolvedPeerId && peerMap.containsKey(resolvedPeerId)) {
                        Timber.d("UNIFICATION loadPeers dedup remove stale resolved key $resolvedPeerId -> $lookupKey")
                        peerMap.remove(resolvedPeerId)
                    }
                    canonicalRawKey?.let { if (it != lookupKey && peerMap.containsKey(it)) { Timber.d("UNIFICATION dedup remove stale raw canonical $it -> $lookupKey"); peerMap.remove(it) } }
                    canonicalResolvedKey?.let { if (it != lookupKey && peerMap.containsKey(it)) { Timber.d("UNIFICATION dedup remove stale resolved canonical $it -> $lookupKey"); peerMap.remove(it) } }
                    if (rawPeerId != lookupKey && peerMap.containsKey(rawPeerId)) {
                        Timber.d("UNIFICATION dedup remove stale raw $rawPeerId -> $lookupKey")
                        peerMap.remove(rawPeerId)
                    }
                } else {
                    Timber.d("UNIFICATION loadPeers ledger NEW $rawPeerId -> $lookupKey (peerId $peerId)")
                    // Preserve authoritative nickname: filter synthetic ledger nicknames
                    // FIX: enrich offline ledger peers with contact's localNickname (user-defined, primary) if available.
                    // Previously ledger-only peers had localNickname=null, causing Dashboard to show synthetic peer-... or PK:...
                    // instead of ChristyLove. Now contact's display name is preferred.
                    val ledgerNick = selectAuthoritativeNickname(entry.nickname, null)
                    var contactLocalForLedger: String? = null
                    var contactNickForLedger: String? = null
                    try {
                        val contactByKey = normalizePublicKey(entry.publicKey)?.lowercase()
                        val contact = meshRepository.listContacts().firstOrNull {
                            it.peerId.lowercase() == lookupKey ||
                                normalizePublicKey(it.publicKey)?.lowercase() == contactByKey ||
                                normalizePublicKey(it.publicKey)?.lowercase() == peerId.lowercase()
                        }
                        if (contact != null) {
                            contactLocalForLedger = contact.localNickname?.trim()?.takeIf { it.isNotEmpty() }?.takeUnless { isSyntheticFallbackNickname(it) }
                            contactNickForLedger = contact.nickname?.trim()?.takeIf { it.isNotEmpty() }?.takeUnless { isSyntheticFallbackNickname(it) }
                            if (contactLocalForLedger != null || contactNickForLedger != null) {
                                Timber.d("UNIFICATION loadPeers ledger NEW enrich from contact $lookupKey: local=${contactLocalForLedger?.take(16)} nick=${contactNickForLedger?.take(16)} over ledgerNick=${ledgerNick?.take(16)}")
                            }
                        }
                    } catch (e: Exception) {
                        Timber.w(e, "UNIFICATION loadPeers ledger enrich failed for $lookupKey")
                    }
                    peerMap[lookupKey] = PeerInfo(
                        peerId = peerId.lowercase(),
                        nickname = contactNickForLedger ?: ledgerNick,
                        localNickname = contactLocalForLedger,
                        multiaddr = entry.multiaddr,
                        lastSeen = entry.lastSeen,
                        transport = determineTransport(entry.multiaddr),
                        transports = MeshRepository.parseTransportsFromMultiaddrs(listOf(entry.multiaddr))
                            .toList().sorted(),
                        isOnline = isRecent(entry.lastSeen),
                        isFull = false,
                        isRelay = false
                    )
                }
            }

            // UNIFICATION_V2: All nodes are relays — synthesize hop entries as regular mesh peers (no isRelay distinction).
            // UNIFICATION COALESCE: A relay/seed hop is a TRANSPORT alias (via-shared-node) of an identity, NOT a distinct
            // node. Canonicalize each hopId (libp2p 12D3KooW) to public_key_hex so it MERGES into the existing unified node
            // (Windows 12D3KooWJoW9 -> 30d0fa, etc.) instead of creating a raw 12D3KooW duplicate that splits the peer list
            // and conversation thread. Exclude our own identity entirely. This is what lets every transport (BLE, TCP/LAN,
            // Internet, via-shared-node circuit) coalesce into the single unified node/thread per identity.
            val selfHopKey = try {
                meshRepository.getIdentityInfoSync()?.publicKeyHex?.trim()?.lowercase()
            } catch (_: Exception) { null }
            for (rawHopId in relayHops) {
                val hopId = rawHopId.trim()
                if (hopId.isEmpty()) continue
                // Canonicalize 12D3KooW -> 64-hex public_key_hex so hop alias merges with the same identity across transports.
                val hopCanonical = canonicalHexForAnyId(hopId, null)?.lowercase() ?: hopId.lowercase()
                Timber.d("UNIFICATION coalesce hop: $hopId canonical-> ${hopCanonical.take(16)} (self=$selfHopKey)")
                // Never show our own identity as a separate/hop node.
                if (selfHopKey != null && hopCanonical == selfHopKey) {
                    Timber.d("UNIFICATION coalesce hop SKIP self: $hopId ($hopCanonical.take(16))")
                    continue
                }
                val hopMultiaddrs = ledgerEntries.filter { it.multiaddr.contains("/p2p/$hopId/p2p-circuit/") }.map { it.multiaddr }
                val hopLastSeen = ledgerEntries
                    .filter { it.multiaddr.contains("/p2p/$hopId/p2p-circuit/") }
                    .mapNotNull { it.lastSeen }.maxOrNull()
                val hopTransports = MeshRepository.parseTransportsFromMultiaddrs(hopMultiaddrs) + setOf(MeshRepository.TRANSPORT_RELAY_CIRCUIT)
                // Merge into an existing unified node if one already exists for this identity (e.g. Windows discovered as
                // 30d0fa via BLE/TCP-LAN, hop alias joins its transports) — do NOT create a duplicate raw hop node.
                val existing = peerMap[hopCanonical] ?: peerMap[hopId]
                if (existing != null) {
                    val merged = existing.copy(
                        multiaddr = if (existing.multiaddr.isEmpty() || existing.multiaddr.contains("/p2p-circuit/")) hopMultiaddrs.firstOrNull() ?: existing.multiaddr else existing.multiaddr,
                        lastSeen = when {
                            hopLastSeen == null -> existing.lastSeen
                            existing.lastSeen == null || hopLastSeen > existing.lastSeen -> hopLastSeen
                            else -> existing.lastSeen
                        },
                        isOnline = (hopLastSeen != null && isRecent(hopLastSeen)) || existing.isOnline,
                        isFull = existing.isFull,
                        isRelay = false,
                        transports = (existing.transports + hopTransports).distinct().sorted()
                    )
                    peerMap[hopCanonical] = merged
                    if (existing.peerId != hopCanonical) {
                        Timber.d("UNIFICATION coalesce hop MERGE $hopId into ${existing.peerId.take(16)} (unified transports ${merged.transports})")
                    }
                    if (hopId != hopCanonical && peerMap.containsKey(hopId)) {
                        Timber.d("UNIFICATION coalesce hop remove raw alias key $hopId -> $hopCanonical")
                        peerMap.remove(hopId)
                    }
                } else {
                    // Genuinely new identity seen only as a relay hop — key/advertise by canonical hex, NOT raw libp2p.
                    Timber.d("UNIFICATION coalesce hop NEW $hopId -> $hopCanonical (transport-only identity)")
                    peerMap[hopCanonical] = PeerInfo(
                        peerId = hopCanonical,
                        nickname = null,
                        localNickname = null,
                        multiaddr = hopMultiaddrs.firstOrNull() ?: "",
                        lastSeen = hopLastSeen,
                        transport = MeshRepository.TRANSPORT_RELAY_CIRCUIT,
                        transports = hopTransports.toList().sorted(),
                        isOnline = hopLastSeen != null && isRecent(hopLastSeen),
                        isFull = false,
                        isRelay = false
                    )
                }
            }

            // Exclude our OWN identity from the peer list entirely (a node must not appear as a "peer" of itself).
            if (selfHopKey != null) {
                val hadSelf = peerMap.remove(selfHopKey) != null
                if (hadSelf) {
                    Timber.d("UNIFICATION remove-self from peer list: $selfHopKey")
                }
            }

            // UNIFICATION ONLINE-AUTHORITY: A peer is genuinely ONLINE only if it was directly observed
            // (present in the discovery map from an actual BLE/TCP-LAN/Internet session) OR it has a DIRECT
            // (non /p2p-circuit) ledger address with a proven successful recent connection (failureCount==0).
            // Relay/seed HOP aliases shared through another node's circuit (e.g. AWS relays MacLane/Lucaso/some
            // other device) have only /p2p-circuit multiaddrs and success_count==0 — they are relayed references,
            // NOT currently-connected devices, so they must be marked OFFLINE regardless of their relay-refreshed
            // lastSeen. This is what prevents "5 online" phantom inflation vs the 2 real others (Windows + AWS).
            val discoveredOnlineKeys = discovered.entries
                .filter { (_, d) -> isRecent(d.lastSeen) && isNearbyDiscovered(d) }
                .map { (k, _) -> canonicalHexForAnyId(k, null)?.lowercase() ?: k.lowercase() }
                .toSet()
            Timber.d("UNIFICATION online-authority discoveredOnlineKeys=${discoveredOnlineKeys.map { it.take(8) }} (discovered=${discovered.keys.map { it.take(8) }} contactIds=${discovered.values.count()})")
            fun ledgerHasDirectRecent(key: String): Boolean {
                val k = key.lowercase()
                for (e in ledgerEntries) {
                    val eRawId = e.peerId ?: ""
                    val eKey = (canonicalHexForAnyId(eRawId, e.publicKey)?.lowercase() ?: eRawId.lowercase())
                    if (eKey.isEmpty() || eKey != k) continue
                    // Only DIRECT (non-circuit) addresses with a proven connection count as online; relayed alias = offline.
                    if (e.multiaddr.contains("/p2p-circuit/")) continue
                    if ((e.failureCount ?: 0u) > 0u) continue
                    if (isRecent(e.lastSeen)) return true
                }
                return false
            }
            val correctedPeerList = peerMap.mapValues { (_, p) ->
                val inDiscovered = p.peerId.lowercase() in discoveredOnlineKeys
                val ledgerDirect = ledgerHasDirectRecent(p.peerId)
                val genuineOnline = inDiscovered || ledgerDirect
                Timber.d("UNIFICATION online-authority ${p.peerId.take(16)} discovered=${inDiscovered} ledgerDirect=$ledgerDirect online=$genuineOnline (was ${p.isOnline})")
                if (genuineOnline) p else p.copy(isOnline = false)
            }.values.toList()
            val peerList = correctedPeerList
            // UNIFICATION_V2: single unified sorted list — online first then offline by recency.
            // Stale filter: "nearby" must be online/recent only for accurate counts. Very stale ledger
            // entries (lastSeen >7 days) are excluded from the display list unless they are saved
            // contacts — preserves offline contacts with "last seen Xm ago" while preventing the
            // "9 nearby" inflation where 23 ledger entries all counted as nearby.
            // Crash guard: blank peerIds and duplicate keys cause Compose LazyColumn ArrayIndexOutOfBounds (MutableVector)
            val deduped = peerList.filter { it.peerId.isNotBlank() }.distinctBy { it.peerId.trim() }
            val contactIds = try {
                meshRepository.listContacts().map { it.peerId.trim() }.toSet()
            } catch (e: Exception) {
                emptySet()
            }
            val nowSec = System.currentTimeMillis() / 1000
            val sevenDaysAgoSec = nowSec - 7L * 24 * 3600
            val filtered = deduped.filter { peer ->
                if (peer.isOnline) return@filter true
                if (peer.peerId.trim() in contactIds) return@filter true
                val lastSeenSec = peer.lastSeen?.toEpochSeconds() ?: 0L
                // Keep if lastSeen within 7-day retention window, drop very stale non-contacts.
                // Future timestamps (clock skew) are kept rather than dropped — treat as recent.
                lastSeenSec >= sevenDaysAgoSec
            }
            val sortedPeerList = sortPeersForUnifiedView(filtered)
            // Distinct check + isActive prevents Compose crash on destroy (SlotTable NPE at onChildRemoved)
            if (!viewModelScope.isActive) {
                Timber.d("loadPeers: ViewModelScope not active, skipping emit")
                return
            }
            if (_peers.value != sortedPeerList) {
                _peers.value = sortedPeerList
            } else {
                Timber.d("loadPeers: no change, skipping emit to avoid recomposition")
            }

            // UNIFICATION FIX: verbose diagnosis for 9 nearby vs 2 actual — log total, online, nearby(direct), and transports
            val onlineCount = sortedPeerList.count { it.isOnline }
            val nearbyDirectCount = sortedPeerList.count { isNearby(it) }
            // Discovery-based nearby (accurate) vs _peers-based naive nearby — diagnose inflation
            val discoveredDeduped = deduplicateDiscoveredPeers(discoveredSnapshot)
            val discoveredNearby = discoveredDeduped.values.count { isNearbyDiscovered(it) }
            Timber.d("UNIFICATION loadPeers: ${sortedPeerList.size} total unified (online=$onlineCount nearbyDirectInList=$nearbyDirectCount discoveredNearby=$discoveredNearby dedupedDiscovered=${discoveredDeduped.size} ledgerEntries=${ledgerEntries.size}) — peers: ${sortedPeerList.joinToString { "${it.peerId.take(8)}:${it.transport}:${it.transports.joinToString("/")}:${it.isOnline}:${it.lastSeen}" }}")
            Timber.d("Loaded ${sortedPeerList.size} peers (${sortedPeerList.count { it.isFull }} full) — online=$onlineCount nearbyDirectInList=$nearbyDirectCount discoveredNearby=$discoveredNearby (discovery-based nearby is authoritative for Dashboard 'nearby' stat)")
        } catch (e: Exception) {
            Timber.e(e, "Failed to load peers")
        } finally {
            loadPeersMutex.unlock()
        }
    }

    // UNIFICATION: canonical dedup by public_key_hex — 12D3 (libp2p) and 30d0fa (hex) for same identity must merge.
    // Robust even when ironCore is null at cold start: PeerKeyUtils protobuf fallback is authoritative.
    // Verbose Timber logs are persisted to file via FileLoggingTree for post-hoc verification.
    private fun canonicalHexForAnyId(rawId: String, publicKeyHint: String?): String? {
        val trimmedRaw = rawId.trim()
        if (trimmedRaw.isEmpty()) return null
        // Strongest: explicit publicKey field (already self-certified in MeshRepository read guards)
        normalizePublicKey(publicKeyHint)?.let {
            if (trimmedRaw.lowercase() != it.lowercase()) {
                Timber.d("UNIFICATION canonicalHex: $trimmedRaw + publicKeyHint ${publicKeyHint?.take(8)}... -> $it via hint")
            }
            return it
        }
        // Try MeshRepository canonical (uses ironCore.resolveIdentity when available)
        var viaRepo: String? = null
        try {
            viaRepo = meshRepository.canonicalContactIdPublic(trimmedRaw)
            if (viaRepo.isNotEmpty() && viaRepo != trimmedRaw) {
                Timber.d("UNIFICATION canonicalHex: $trimmedRaw -> $viaRepo via repo (ironCore)")
                normalizePublicKey(viaRepo)?.let { return it }
                if (viaRepo.length == 64 && viaRepo.all { c -> c in '0'..'9' || c in 'a'..'f' || c in 'A'..'F' }) {
                    return viaRepo.lowercase()
                }
            }
        } catch (e: Exception) {
            Timber.w(e, "UNIFICATION canonicalHex repo failed for $trimmedRaw")
        }
        // Fallback: manual libp2p -> hex extraction (no ironCore needed, e.g., cold start or degraded)
        // UNIFICATION: PeerKeyUtils now uses protobuf 00 24 08 01 12 20 extraction matching Rust.
        if (com.scmessenger.android.utils.PeerIdValidator.isLibp2pPeerId(trimmedRaw)) {
            com.scmessenger.android.utils.PeerKeyUtils.extractPublicKeyFromPeerId(trimmedRaw)?.let { hex ->
                normalizePublicKey(hex)?.let {
                    Timber.d("UNIFICATION canonicalHex: $trimmedRaw -> $it via PeerKeyUtils fallback (cold-start)")
                    return it
                }
            }
            // Verbose: why fallback failed (helps diagnose stale 12D3 entries that still duplicate)
            Timber.w("UNIFICATION canonicalHex: PeerKeyUtils failed to extract hex from libp2p $trimmedRaw (viaRepo=${viaRepo?.take(16)})")
        }
        // If rawId itself is 64-hex public key, use it (covers already-migrated ledger peer_ids)
        normalizePublicKey(trimmedRaw)?.let {
            Timber.d("UNIFICATION canonicalHex: $trimmedRaw is itself 64-hex -> $it")
            return it
        }
        // Last chance: viaRepo may have been 64-hex even though it equaled rawId (already canonical)
        viaRepo?.let { vr ->
            normalizePublicKey(vr)?.let {
                Timber.d("UNIFICATION canonicalHex: $trimmedRaw viaRepo fallback -> $it")
                return it
            }
            if (vr.length == 64) return vr.lowercase()
        }
        Timber.d("UNIFICATION canonicalHex: no canonical hex for $trimmedRaw (hint=${publicKeyHint?.take(8)}) viaRepo=${viaRepo?.take(16)}")
        return null
    }

    private fun deduplicateDiscoveredPeers(
        discovered: Map<String, MeshRepository.PeerDiscoveryInfo>
    ): Map<String, MeshRepository.PeerDiscoveryInfo> {
        val merged = linkedMapOf<String, MeshRepository.PeerDiscoveryInfo>()
        discovered.values.forEach { info ->
            val rawId = info.peerId.trim()
            if (rawId.isEmpty()) return@forEach
            // UNIFICATION: canonicalize via canonicalHexForAnyId for BOTH rawPeerId and publicKey hint
            // so 12D3Koo (libp2p) and 30d0fa (hex) for same identity merge to one node even at cold start.
            val canonicalByKey = canonicalHexForAnyId(rawId, info.publicKey) ?: rawId.lowercase()
            val mapKey = canonicalByKey.lowercase()
            // Use canonicalByKey as authoritative peerId — ensures key == peerId (previous bug: viaRepo
            // returned raw 12D3 when ironCore null, causing key=30d0fa but peerId=12D3 mismatch and duplicate).
            val canonicalPeerId = canonicalByKey.lowercase()
            val existing = merged[mapKey]
            if (existing == null) {
                // UNIFICATION: Store with canonical peerId (public_key_hex) so UI shows 30d0fa..., not libp2p
                Timber.d("UNIFICATION dedup: NEW $rawId (hint=${info.publicKey?.take(8)}) -> key $mapKey peerId $canonicalPeerId")
                merged[mapKey] = info.copy(peerId = canonicalPeerId)
            } else {
                Timber.d("UNIFICATION dedup: MERGE $rawId (hint=${info.publicKey?.take(8)}) into $mapKey (existing ${existing.peerId.take(8)}... + ${info.peerId.take(8)}...)")
                val authoritativeNick = selectAuthoritativeNickname(existing.nickname, info.nickname)
                    ?: selectAuthoritativeNickname(info.nickname, existing.nickname)
                    ?: existing.nickname
                val authoritativeLocal = selectAuthoritativeNickname(existing.localNickname, info.localNickname)
                    ?: existing.localNickname ?: info.localNickname
                merged[mapKey] = existing.copy(
                    peerId = canonicalPeerId,
                    publicKey = existing.publicKey ?: info.publicKey,
                    nickname = authoritativeNick,
                    localNickname = authoritativeLocal,
                    transport = if (
                        existing.transport == com.scmessenger.android.service.TransportType.INTERNET ||
                            info.transport == com.scmessenger.android.service.TransportType.INTERNET
                    ) {
                        com.scmessenger.android.service.TransportType.INTERNET
                    } else {
                        existing.transport
                    },
                    isFull = existing.isFull || info.isFull,
                    isRelay = existing.isRelay || info.isRelay,
                    lastSeen = maxOf(existing.lastSeen, info.lastSeen),
                    transports = existing.transports + info.transports
                )
            }
        }
        if (discovered.size != merged.size) {
            Timber.i("UNIFICATION dedup: ${discovered.size} discovered -> ${merged.size} merged (collapsed ${discovered.size - merged.size} duplicates)")
        }
        return merged
    }

    // UNIFICATION_V2: All nodes are relays — sort is simply online first, then offline by recency.
    // Formerly tiered online-user vs online-relay vs offline; now two tiers (all relays are equal).
    private fun sortPeersForUnifiedView(peers: List<PeerInfo>): List<PeerInfo> {
        return peers.sortedWith(
            compareBy<PeerInfo> { if (it.isOnline) 0 else 1 }
                .thenByDescending { it.lastSeen ?: 0uL }.thenBy { it.peerId }
        )
    }

    // UNIFICATION P1-1: Validate Ed25519 curve point — roughly 50% of blake3 identity_id hashes are valid 64-hex
    // but NOT valid curve points. Rust's is_valid_public_key checks VerifyingKey::from_bytes; Kotlin previously
    // treated any 64-hex as canonical pubkey, so identity_id was accepted as pubkey and failed to merge with 30d0fa.
    // We replicate the Rust check via pure Kotlin (BigInteger decompression). If not a valid point, return null
    // so caller treats it as identity_id, not pubkey.
    private fun normalizePublicKey(value: String?): String? {
        val trimmed = value?.trim() ?: return null
        if (trimmed.length != 64) return null
        if (!trimmed.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) return null
        if (!isValidEd25519Point(trimmed)) return null
        return trimmed.lowercase()
    }

    /**
     * Build network topology from ledger and stats.
     */
    private fun buildTopology() {
        try {
            val nodes = mutableListOf<TopologyNode>()
            val edges = mutableListOf<TopologyEdge>()

            // Add self node
            val identityInfo = meshRepository.getIdentityInfoSync()
            if (identityInfo != null) {
                nodes.add(
                    TopologyNode(
                        id = identityInfo.identityId ?: "Self",
                        isSelf = true,
                        isOnline = true
                    )
                )
            }

            // Add peer nodes and edges
            _peers.value.forEach { peer ->
                nodes.add(
                    TopologyNode(
                        id = peer.peerId,
                        isSelf = false,
                        isOnline = peer.isOnline
                    )
                )

                // Add edge from self to peer
                identityInfo?.let {
                    edges.add(
                        TopologyEdge(
                            source = it.identityId ?: "Self",
                            target = peer.peerId,
                            transport = peer.transport
                        )
                    )
                }
            }

            _topology.value = NetworkTopology(nodes, edges)

            Timber.d("Topology built: ${nodes.size} nodes, ${edges.size} edges")
        } catch (e: Exception) {
            Timber.e(e, "Failed to build topology")
        }
    }

    /**
     * Observe network events for real-time updates — debounced to prevent Compose crash from rapid updates.
     * FIX(holistic-crash): previous 500ms debounce + tryLock was insufficient when discoveredPeers
     * updates every 1-2s with lastSeen jitter; multiple refreshData() coroutines raced and
     * emitted new List instances with same content, triggering LazyColumn MutableVector corruption
     * via prefetch. Increase debounce to 700ms, use distinctUntilChanged on serialized key (size
     * + sorted peerIds + lastSeen bucket) to suppress jitter-only emits, and serialize refreshData
     * via mutex with verbose coalesce logging.
     */
    @OptIn(FlowPreview::class)
    private fun observeNetworkEvents() {
        viewModelScope.launch {
            meshRepository.discoveredPeers
                .debounce(700)
                .distinctUntilChanged { old, new ->
                    // Suppress jitter-only updates: compare stable snapshot string (size + sorted keys + lastSeen/10s bucket)
                    fun snapshot(map: Map<String, MeshRepository.PeerDiscoveryInfo>): String {
                        if (map.isEmpty()) return "empty"
                        return map.entries.sortedBy { it.key }.joinToString("|") { (k, v) ->
                            // Bucket lastSeen to 10s to avoid per-second jitter
                            val bucket = (v.lastSeen ?: 0uL) / 10uL
                            "${k.take(8)}:$bucket:${v.transport}"
                        }
                    }
                    val same = snapshot(old) == snapshot(new)
                    if (same) Timber.v("observeNetworkEvents: suppressed jitter-only emit (${old.size} peers)")
                    same
                }
                .collect {
                    withContext(Dispatchers.IO) {
                        refreshData()
                    }
                }
        }

        viewModelScope.launch {
            MeshEventBus.statusEvents.collect { event ->
                if (event is StatusEvent.StatsUpdated) {
                    _stats.value = event.stats
                }
            }
        }
    }

    /**
     * Observe live network stats from the repository (periodic refresh).
     */
    private fun observeLiveNetworkStats() {
        viewModelScope.launch {
            meshRepository.observeNetworkStats().collect { stats ->
                _networkStats.value = stats
            }
        }
    }

    /**
     * Observe live peer list from the repository.
     */
    private fun observeLivePeers() {
        viewModelScope.launch {
            meshRepository.observePeers().collect { peers ->
                _observablePeers.value = peers
            }
        }
    }

    /**
     * Determine primary transport type from multiaddr.
     */
    private fun determineTransport(multiaddr: String): String {
        return when {
            "/p2p-circuit/" in multiaddr -> MeshRepository.TRANSPORT_RELAY_CIRCUIT
            "/ble/" in multiaddr -> "BLE"
            "/wifi-aware/" in multiaddr -> "WiFi Aware"
            "/wifi-direct/" in multiaddr -> "WiFi Direct"
            "/ip4/" in multiaddr || "/ip6/" in multiaddr -> "Internet"
            else -> context.getString(R.string.unknown_transport)
        }
    }

    /**
     * Check if timestamp is recent (within last 5 minutes).
     */
    private fun isRecent(timestamp: ULong?): Boolean {
        if (timestamp == null) return false
        val now = System.currentTimeMillis() / 1000
        val seenAt = timestamp.toEpochSeconds()
        val fiveMinutes = 300L
        val isRecent = seenAt <= now && (now - seenAt) < fiveMinutes
        Timber.v("isRecent: timestamp=$timestamp seenAt=$seenAt now=$now diff=${now - seenAt} isRecent=$isRecent")
        return isRecent
    }

    /**
     * Check if peer is nearby (very recent, direct transport). Stricter than isRecent for accurate nearby count.
     * UNIFICATION FIX: Nearby means isOnline + direct transport (BLE, TCP/LAN, WiFi Aware/Direct) only — Internet/Relay excluded.
     * Previous allowed empty transports to count as nearby; now requires explicit direct transport to prevent ledger inflation.
     * Note: nearbyPeersCount is now discovery-based (isNearbyDiscovered) to exclude ledger history; this helper remains for _peers-based checks where needed.
     */
    private fun isNearby(peer: PeerInfo): Boolean {
        if (!peer.isOnline) return false
        // UNIFICATION: Direct transports only — BLE, TCP/LAN, WiFi Aware, WiFi Direct. Internet and Via shared node are not proximity.
        val hasDirect = peer.transports.any {
            it == MeshRepository.TRANSPORT_BLE || it == MeshRepository.TRANSPORT_TCP_LAN || it == MeshRepository.TRANSPORT_WIFI_AWARE || it == MeshRepository.TRANSPORT_WIFI_DIRECT ||
                it == "BLE" || it == "TCP/LAN" || it == "TCP/mDNS" || it == "WiFi Aware" || it == "WiFi Direct"
        }
        if (peer.transports.isNotEmpty()) {
            if (!hasDirect) {
                Timber.v("isNearby false: ${peer.peerId.take(8)} no direct transport in ${peer.transports} (primary ${peer.transport})")
            }
            return hasDirect
        }
        // Fallback when transports empty — check primary transport explicitly (must be direct)
        val primaryIsDirect = peer.transport == MeshRepository.TRANSPORT_BLE || peer.transport == MeshRepository.TRANSPORT_TCP_LAN || peer.transport == MeshRepository.TRANSPORT_WIFI_AWARE || peer.transport == MeshRepository.TRANSPORT_WIFI_DIRECT ||
            peer.transport == "BLE" || peer.transport == "TCP/LAN" || peer.transport == "TCP/mDNS" || peer.transport == "WiFi Aware" || peer.transport == "WiFi Direct"
        if (!primaryIsDirect) {
            Timber.v("isNearby false: ${peer.peerId.take(8)} primary ${peer.transport} not direct and empty transports")
        }
        return primaryIsDirect
    }

    /**
     * UNIFICATION: Discovery-based nearby check — used for nearbyPeersCount to exclude ledger history.
     * Counts only peers currently discovered via direct transport (BLE, TCP/mDNS, WiFi Aware/Direct) and isRecent (<5min).
     * Ledger entries (dialable+seed+deadRecent) are not counted even if they have direct multiaddrs, because they are not currently nearby via BLE/mDNS.
     */
    private fun isNearbyDiscovered(info: MeshRepository.PeerDiscoveryInfo): Boolean {
        if (!isRecent(info.lastSeen)) return false
        // Direct transport via enum — BLE, TCP_MDNS (LAN), WiFi Aware/Direct are direct; INTERNET is not
        val primaryIsDirect = when (info.transport) {
            com.scmessenger.android.service.TransportType.BLE,
            com.scmessenger.android.service.TransportType.TCP_MDNS,
            com.scmessenger.android.service.TransportType.WIFI_AWARE,
            com.scmessenger.android.service.TransportType.WIFI_DIRECT -> true
            com.scmessenger.android.service.TransportType.INTERNET -> false
        }
        if (primaryIsDirect) return true
        // Fallback: check transports set for direct (covers merged discovered entries where primary became INTERNET but still has BLE)
        if (info.transports.isNotEmpty()) {
            val hasDirect = info.transports.any {
                it == MeshRepository.TRANSPORT_BLE || it == MeshRepository.TRANSPORT_TCP_LAN || it == MeshRepository.TRANSPORT_WIFI_AWARE || it == MeshRepository.TRANSPORT_WIFI_DIRECT ||
                    it == "BLE" || it == "TCP/LAN" || it == "TCP/mDNS" || it == "WiFi Aware" || it == "WiFi Direct"
            }
            return hasDirect
        }
        return false
    }

    private fun normalizeNickname(value: String?): String? = value?.trim()?.takeIf { it.isNotEmpty() }

    private fun isSyntheticFallbackNickname(value: String?): Boolean {
        val n = normalizeNickname(value)?.lowercase() ?: return false
        return n.startsWith("peer-")
    }

    private fun selectAuthoritativeNickname(incoming: String?, existing: String?): String? {
        val incomingNormalized = normalizeNickname(incoming)
        val existingNormalized = normalizeNickname(existing)
        val incomingSynthetic = isSyntheticFallbackNickname(incomingNormalized)
        val existingSynthetic = isSyntheticFallbackNickname(existingNormalized)
        return when {
            incomingNormalized == null && existingSynthetic -> null
            incomingNormalized == null -> existingNormalized
            incomingSynthetic && existingNormalized == null -> null
            incomingSynthetic && existingSynthetic -> null
            incomingSynthetic -> existingNormalized
            existingSynthetic -> incomingNormalized
            else -> incomingNormalized
        }
    }

    /**
     * Clear error state.
     */
    fun clearError() {
        _error.value = null
    }
}

/**
 * Peer information for display.
 */
data class PeerInfo(
    val peerId: String,
    val nickname: String?,
    val localNickname: String? = null,
    val multiaddr: String,
    val lastSeen: ULong?,
    val transport: String,
    // NODE-TRANSPORT-VIS-001: all transports known for this node (BLE, TCP/LAN,
    // WiFi Aware, WiFi Direct, Relay-circuit, Internet).
    val transports: List<String> = emptyList(),
    val isOnline: Boolean,
    val isFull: Boolean,
    val isRelay: Boolean = false
)

/**
 * Network topology data structure.
 */
data class NetworkTopology(
    val nodes: List<TopologyNode> = emptyList(),
    val edges: List<TopologyEdge> = emptyList()
)

/**
 * Topology node (peer in the network).
 */
data class TopologyNode(
    val id: String,
    val isSelf: Boolean,
    val isOnline: Boolean
)

/**
 * Topology edge (connection between peers).
 */
data class TopologyEdge(
    val source: String,
    val target: String,
    val transport: String
)
