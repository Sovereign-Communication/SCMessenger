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
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import timber.log.Timber
import javax.inject.Inject

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

    // FIX: "nearby" must be online/recent only — not all ledger entries (up to 23). Exposes accurate online count for DashboardScreen.
    val nearbyPeersCount: StateFlow<Int> = _peers.map { list -> list.count { it.isOnline } }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 0)

    // Loading state
    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    // Error state
    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

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
     */
    private fun loadPeers() {
        try {
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
            }.toMutableMap()

            // Enrich/Add with ledger entries — canonicalize: 12D3 (libp2p) and 30d0fa (hex) for same identity must merge
            ledgerEntries.forEach { entry ->
                val rawPeerId = entry.peerId ?: return@forEach
                val resolvedPeerId = routeAliasToCanonical[rawPeerId]
                    ?: normalizePublicKey(entry.publicKey)?.let { canonicalByPublicKey[it] }
                    ?: rawPeerId
                // Canonicalize libp2p -> hex so 12D3KooWD6... and 30d0fa... share one key (both hashes needed but not duplicate nodes)
                // Use robust helper with PeerKeyUtils fallback for cold-start (ironCore null) + verbose log
                val peerId = canonicalHexForAnyId(resolvedPeerId, entry.publicKey)
                    ?: try { meshRepository.canonicalContactIdPublic(resolvedPeerId).takeIf { it.isNotEmpty() } ?: resolvedPeerId } catch (_: Exception) { resolvedPeerId }
                Timber.d("loadPeers ledger: $rawPeerId -> $peerId (via canonicalHex)")
                // For lookup, also try canonical variant of existing keys (peerMap is keyed by canonical hex)
                val lookupKey = peerId.trim().lowercase()
                val existing = peerMap[lookupKey] ?: peerMap[peerId] ?: peerMap[resolvedPeerId] ?: run {
                    // Last resort: try canonical of rawPeerId
                    val altKey = canonicalHexForAnyId(rawPeerId, entry.publicKey)?.lowercase()
                    if (altKey != null) peerMap[altKey] else null
                }
                if (existing != null) {
                    val entryLastSeen = entry.lastSeen
                    val existingLastSeen = existing.lastSeen
                    val authoritativeNick = selectAuthoritativeNickname(existing.nickname, entry.nickname)
                        ?: selectAuthoritativeNickname(entry.nickname, existing.nickname)
                        ?: existing.nickname
                    val authoritativeLocal = existing.localNickname
                    // Update via canonical key to avoid 12D3/30d0fa duplicate
                    peerMap[lookupKey] = existing.copy(
                        nickname = authoritativeNick,
                        localNickname = authoritativeLocal,
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
                    // Remove any stale alias key if lookup used alternative
                    if (lookupKey != peerId && peerMap.containsKey(peerId)) peerMap.remove(peerId)
                    if (lookupKey != resolvedPeerId && peerMap.containsKey(resolvedPeerId)) peerMap.remove(resolvedPeerId)
                } else {
                    // Preserve authoritative nickname: filter synthetic ledger nicknames
                    val ledgerNick = selectAuthoritativeNickname(entry.nickname, null)
                    peerMap[lookupKey] = PeerInfo(
                        peerId = peerId,
                        nickname = ledgerNick,
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
            for (hopId in relayHops) {
                if (!peerMap.containsKey(hopId)) {
                    val hopLastSeen = ledgerEntries.filter { it.multiaddr.contains("/p2p/$hopId/p2p-circuit/") }
                        .mapNotNull { it.lastSeen }.maxOrNull()
                    val hopTransports = MeshRepository.parseTransportsFromMultiaddrs(
                        ledgerEntries.filter { it.multiaddr.contains("/p2p/$hopId/p2p-circuit/") }.map { it.multiaddr }
                    ) + setOf(MeshRepository.TRANSPORT_RELAY_CIRCUIT)
                    peerMap[hopId] = PeerInfo(
                        peerId = hopId,
                        nickname = null,
                        localNickname = null,
                        multiaddr = ledgerEntries.firstOrNull { it.multiaddr.contains("/p2p/$hopId/p2p-circuit/") }?.multiaddr ?: "",
                        lastSeen = hopLastSeen,
                        transport = MeshRepository.TRANSPORT_RELAY_CIRCUIT,
                        transports = hopTransports.toList().sorted(),
                        isOnline = hopLastSeen != null && isRecent(hopLastSeen),
                        isFull = false,
                        isRelay = false
                    )
                }
            }

            val peerList = peerMap.values.toList()
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
            _peers.value = sortedPeerList

            Timber.d("Loaded ${sortedPeerList.size} discovered peers (${sortedPeerList.count { it.isFull }} full, ${sortedPeerList.count { it.isRelay }} relays) — nearby(online)=${sortedPeerList.count { it.isOnline }}")
        } catch (e: Exception) {
            Timber.e(e, "Failed to load peers")
        }
    }

    // UNIFICATION: canonical dedup by public_key_hex — 12D3 (libp2p) and 30d0fa (hex) for same identity must merge.
    private fun canonicalHexForAnyId(rawId: String, publicKeyHint: String?): String? {
        // Strongest: explicit publicKey field
        normalizePublicKey(publicKeyHint)?.let { return it }
        // Try MeshRepository canonical (uses ironCore when available) — verbose log
        try {
            val viaRepo = meshRepository.canonicalContactIdPublic(rawId)
            if (viaRepo.isNotEmpty() && viaRepo != rawId) {
                Timber.d("canonicalHex: $rawId -> $viaRepo via repo")
                normalizePublicKey(viaRepo)?.let { return it }
                if (viaRepo.length == 64) return viaRepo.lowercase()
            }
            // Fallback: manual libp2p -> hex extraction (no ironCore needed, e.g., cold start or degraded)
            if (com.scmessenger.android.utils.PeerIdValidator.isLibp2pPeerId(rawId)) {
                com.scmessenger.android.utils.PeerKeyUtils.extractPublicKeyFromPeerId(rawId)?.let { hex ->
                    normalizePublicKey(hex)?.let {
                        Timber.d("canonicalHex: $rawId -> $it via PeerKeyUtils fallback")
                        return it
                    }
                }
            }
            // If rawId itself is 64-hex public key, use it
            normalizePublicKey(viaRepo)?.let { return it }
            if (viaRepo.length == 64) return viaRepo.lowercase()
        } catch (e: Exception) {
            Timber.w(e, "canonicalHex failed for $rawId")
        }
        normalizePublicKey(rawId)?.let { return it }
        return null
    }

    private fun deduplicateDiscoveredPeers(
        discovered: Map<String, MeshRepository.PeerDiscoveryInfo>
    ): Map<String, MeshRepository.PeerDiscoveryInfo> {
        val merged = linkedMapOf<String, MeshRepository.PeerDiscoveryInfo>()
        discovered.values.forEach { info ->
            val rawId = info.peerId.trim()
            if (rawId.isEmpty()) return@forEach
            // Canonicalize: libp2p 12D3Koo... -> 64-hex public_key, hex stays lowercased — both hashes unified
            val canonicalByKey = canonicalHexForAnyId(rawId, info.publicKey) ?: rawId.lowercase()
            val mapKey = canonicalByKey.lowercase()
            val canonicalPeerId = try {
                meshRepository.canonicalContactIdPublic(rawId).takeIf { it.isNotEmpty() } ?: canonicalByKey
            } catch (_: Exception) { canonicalByKey }
            val existing = merged[mapKey]
            if (existing == null) {
                // Store with canonical peerId (public_key_hex) so UI shows correct hash (30d0fa...), not libp2p
                merged[mapKey] = info.copy(peerId = canonicalPeerId.lowercase())
            } else {
                val authoritativeNick = selectAuthoritativeNickname(existing.nickname, info.nickname)
                    ?: selectAuthoritativeNickname(info.nickname, existing.nickname)
                    ?: existing.nickname
                val authoritativeLocal = selectAuthoritativeNickname(existing.localNickname, info.localNickname)
                    ?: existing.localNickname ?: info.localNickname
                merged[mapKey] = existing.copy(
                    peerId = canonicalPeerId.lowercase(),
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

    private fun normalizePublicKey(value: String?): String? {
        val trimmed = value?.trim() ?: return null
        if (trimmed.length != 64) return null
        if (!trimmed.all { it in '0'..'9' || it in 'a'..'f' || it in 'A'..'F' }) return null
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
     * Observe network events for real-time updates.
     */
    private fun observeNetworkEvents() {
        viewModelScope.launch {
            meshRepository.discoveredPeers.collect {
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
        return seenAt <= now && (now - seenAt) < fiveMinutes
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
