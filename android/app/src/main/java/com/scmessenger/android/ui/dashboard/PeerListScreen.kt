package com.scmessenger.android.ui.dashboard

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.scmessenger.android.R
import com.scmessenger.android.data.MeshRepository
import com.scmessenger.android.ui.components.ErrorBanner
import com.scmessenger.android.ui.components.IdenticonFromPeerId
import com.scmessenger.android.service.ConnectionQuality
import com.scmessenger.android.ui.components.ConnectionQualityIndicator
import com.scmessenger.android.ui.components.StatusIndicator
import com.scmessenger.android.ui.theme.*
import com.scmessenger.android.ui.viewmodels.DashboardViewModel
import com.scmessenger.android.utils.toEpochMillis
import timber.log.Timber
import java.text.SimpleDateFormat
import java.util.*

/**
 * Peer List screen - Display connected peers with transport info.
 *
 * Shows all active peers in the mesh network with:
 * - Identicon and peer ID
 * - Connection status and transport type
 * - Last seen timestamp
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PeerListScreen(
    onNavigateBack: () -> Unit,
    viewModel: DashboardViewModel = hiltViewModel(),
    onPeerClick: (com.scmessenger.android.ui.viewmodels.PeerInfo) -> Unit = {}
) {
    val peers by viewModel.peers.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val error by viewModel.error.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.refreshData()
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.peer_list_title)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = stringResource(R.string.chat_action_dismiss))
                    }
                },
                actions = {
                    IconButton(onClick = { viewModel.refreshData() }) {
                        Icon(Icons.Default.Refresh, contentDescription = stringResource(R.string.diagnostics_action_refresh))
                    }
                }
            )
        }
    ) { paddingValues ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
        ) {
            when {
                isLoading -> {
                    CircularProgressIndicator(
                        modifier = Modifier.align(Alignment.Center)
                    )
                }

                peers.isEmpty() -> {
                    Column(
                        modifier = Modifier
                            .align(Alignment.Center)
                            .padding(32.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = stringResource(R.string.peer_list_no_peers),
                            style = MaterialTheme.typography.titleLarge
                        )

                        Spacer(modifier = Modifier.height(8.dp))

                        Text(
                            text = stringResource(R.string.peer_list_no_peers_description),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }

                else -> {
                    // UNIFICATION FIX: Connected Peers = isOnline only — not all ledger entries. Fixes buggy window that showed offline peers as connected.
                    // isOnline is isRecent (<5min) from DashboardViewModel.loadPeers; direct+Internet both count as connected, but only isOnline shows.
                    // Preserves single-list unified sort (online first via DashboardViewModel.sortPeersForUnifiedView); displayPeers is filtered view of sorted _peers.
                    val displayPeers = remember(peers) {
                        val filtered = peers.filter { it.peerId.isNotBlank() && it.isOnline }.distinctBy { it.peerId }
                        Timber.d("UNIFICATION PeerListScreen: ${peers.size} total peers, ${filtered.size} connected(isOnline) — total peers: ${peers.joinToString { "${it.peerId.take(8)}:${it.isOnline}:${it.transport}" }} | display: ${filtered.joinToString { "${it.peerId.take(8)}:${it.transport}" }}")
                        if (peers.size != filtered.size) {
                            Timber.d("UNIFICATION PeerListScreen: filtered ${peers.size - filtered.size} offline peers (not connected) — connected=${filtered.size} offline=${peers.size - filtered.size}")
                        }
                        filtered
                    }
                    Column(modifier = Modifier.fillMaxSize()) {
                        // Error banner
                        error?.let {
                            ErrorBanner(
                                message = it,
                                onDismiss = { viewModel.clearError() }
                            )
                        }

                        // Peer count
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            tonalElevation = 1.dp
                        ) {
                            val countText = if (displayPeers.size == 1) {
                                stringResource(R.string.peer_list_count_format_singular, displayPeers.size)
                            } else {
                                stringResource(R.string.peer_list_count_format_plural, displayPeers.size)
                            }
                            Text(
                                text = countText,
                                modifier = Modifier.padding(16.dp),
                                style = MaterialTheme.typography.titleSmall,
                                fontWeight = FontWeight.Bold
                            )
                        }

                        // UNIFICATION_V2: single unified sorted list — classification via badge, not section
                        // FIX: Use Column+verticalScroll to avoid LazyColumn prefetch crash on rapid list updates
                        Column(
                            modifier = Modifier
                                .fillMaxSize()
                                .verticalScroll(rememberScrollState())
                                .padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp)
                        ) {
                            Column {
                                Text(
                                    text = "Nodes (${displayPeers.size})",
                                    style = MaterialTheme.typography.titleMedium,
                                    fontWeight = FontWeight.Bold
                                )
                                Text(
                                    text = stringResource(R.string.shared_nodes_subtitle),
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                            // FIX(Compose-crash): stable key per peerId prevents SlotTable corruption
                            // on rapid peer churn; displayPeers already distinctBy peerId, key() ensures
                            // Compose retains correct slot identity without LazyColumn keys/contentType.
                            displayPeers.forEach { peer ->
                                key(peer.peerId) {
                                    PeerCard(peer = peer, onClick = { onPeerClick(peer) })
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun PeerCard(
    peer: com.scmessenger.android.ui.viewmodels.PeerInfo,
    modifier: Modifier = Modifier,
    onClick: (() -> Unit)? = null
) {
    Card(
        modifier = modifier.fillMaxWidth().then(if (onClick != null) Modifier.clickable { onClick() } else Modifier)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.spacedBy(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Identicon
            IdenticonFromPeerId(
                peerId = peer.peerId,
                size = 56.dp
            )

            // Info
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                // Peer ID
                Text(
                    text = peer.displayName(),
                    style = MaterialTheme.typography.bodyMedium,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Medium
                )

                // UNIFICATION_V2: All nodes are relays — infrastructure label removed.

                // NODE-TRANSPORT-VIS-001: one badge per known transport.
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    peer.transports.ifEmpty { listOf(peer.transport) }.forEach { transport ->
                        TransportBadge(transport = transport)
                    }
                }

                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    StatusIndicator(
                        isOnline = peer.isOnline
                    )
                    ConnectionQualityIndicator(
                        quality = if (peer.isOnline) ConnectionQuality.GOOD
                                  else ConnectionQuality.UNKNOWN
                    )
                    Text(
                        text = if (peer.isOnline) stringResource(R.string.peer_list_status_online) else stringResource(R.string.peer_list_status_offline),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }

                // Last seen
                peer.lastSeen?.let {
                    Text(
                        text = stringResource(R.string.peer_list_last_seen, formatTimestamp(it)),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

private fun isSyntheticFallbackNickname(value: String?): Boolean {
    val n = value?.trim()?.takeIf { it.isNotEmpty() }?.lowercase() ?: return false
    return n.startsWith("peer-")
}

private fun com.scmessenger.android.ui.viewmodels.PeerInfo.displayName(): String {
    val primary = localNickname?.trim()?.takeIf { it.isNotEmpty() }?.takeUnless { isSyntheticFallbackNickname(it) }
    val secondary = nickname?.trim()?.takeIf { it.isNotEmpty() }?.takeUnless { isSyntheticFallbackNickname(it) }
    val authoritative = primary ?: secondary
    if (!authoritative.isNullOrEmpty()) return authoritative
    // Fallback to synthetic only when truly unknown (no authoritative name)
    val fallback = localNickname?.trim()?.takeIf { it.isNotEmpty() } ?: nickname?.trim()?.takeIf { it.isNotEmpty() }
    if (!fallback.isNullOrEmpty()) return fallback
    // UNIFICATION_V2_IDENTITY: peerId is canonical public_key_hex (64 hex), not libp2p 12D3 hash.
    // Use explicit PK:/P2P: prefix with truncated display to avoid identity hash confusion.
    return when {
        peerId.startsWith("12D3") -> "P2P:${peerId.take(12)}..."
        peerId.length >= 8 -> "PK:${peerId.take(8)}..."
        else -> peerId.take(16) + "..."
    }
}

@Composable
private fun TransportBadge(
    transport: String,
    modifier: Modifier = Modifier
) {
    val color = when (transport) {
        "BLE" -> TransportBLE
        "WiFi Aware" -> TransportWiFiAware
        "WiFi Direct" -> TransportWiFiDirect
        "Internet" -> TransportInternet
        "TCP/LAN", "TCP/mDNS" -> TransportTcpLan
        MeshRepository.TRANSPORT_RELAY_CIRCUIT -> TransportRelayCircuit
        else -> MaterialTheme.colorScheme.surfaceVariant
    }

    Surface(
        modifier = modifier,
        color = color,
        shape = MaterialTheme.shapes.small
    ) {
        val unknownTransport = stringResource(R.string.unknown_transport)
        Text(
            text = transport,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            style = MaterialTheme.typography.labelSmall,
            color = if (transport == unknownTransport) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onPrimary
        )
    }
}

private fun formatTimestamp(timestamp: ULong): String {
    val millis = timestamp.toEpochMillis()
    val date = Date(millis)
    val now = Date()

    val diff = (now.time - date.time) / 1000

    return when {
        diff < 60 -> "just now"
        diff < 3600 -> "${diff / 60}m ago"
        diff < 86400 -> "${diff / 3600}h ago"
        else -> {
            val sdf = SimpleDateFormat("MMM d", Locale.getDefault())
            sdf.format(date)
        }
    }
}
